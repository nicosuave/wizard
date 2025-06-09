# DuckDB Wizard

Did you ever want to `SELECT * FROM clean_data_you_think_exists`? Well, now you can!

A DuckDB extension that lets you query any data using natural language or run arbitrary JavaScript code directly in your SQL queries.

> ⚠️ **WARNING: HIGHLY EXPERIMENTAL** ⚠️
> 
> This extension is in early experimental stage and should be used with extreme caution.
> - **DO NOT use in production environments**
> - The generated code may be unpredictable or incorrect
> - Always review the generated queries before executing
> - API calls to LLMs incur costs - monitor your usage
> - Proceed at your own risk!

Query any data using natural language in DuckDB, or execute JavaScript code directly! Powered by LLMs (OpenAI/Anthropic) and Deno.

## Quick Start

```bash
# Clone the repo
git clone --recurse-submodules https://github.com/nicosuave/wizard
cd wizard

# Set your API key
export OPENAI_API_KEY="your-api-key"  # or ANTHROPIC_API_KEY

# Build the extension
make build    # for debug build
# OR
make release  # for optimized build

# Try it out!
duckdb -unsigned
```

Then in DuckDB:
```sql
LOAD 'build/release/wizard.duckdb_extension';
SELECT * FROM wizard('bitcoin price');
```

## Examples

```sql
-- Load the extension
LOAD 'path/to/wizard.duckdb_extension';

-- Fetch recent earthquake data
SELECT * FROM wizard('recent earthquakes for past month');

-- Get cryptocurrency prices
SELECT * FROM wizard('Bitcoin price in USD');

-- Weather information  
SELECT * FROM wizard('current weather in Seattle');

-- Programming jokes
SELECT * FROM wizard('random programming joke');

-- Complex queries work too!
SELECT magnitude, place, time 
FROM wizard('earthquakes magnitude > 5.0 past week')
WHERE place LIKE '%California%'
ORDER BY magnitude DESC;

-- Run arbitrary JavaScript code directly!
SELECT * FROM js('
async function fetch_data() {
    // You can use any JavaScript here, including npm packages
    const response = await fetch("https://api.github.com/repos/duckdb/duckdb");
    const data = await response.json();
    return [{
        name: data.name,
        stars: data.stargazers_count,
        language: data.language
    }];
}
');

-- Use npm packages in your JavaScript
SELECT * FROM js('
import dayjs from "npm:dayjs";

async function fetch_data() {
    const now = dayjs();
    return [{
        current_time: now.format(),
        unix_timestamp: now.unix(),
        day_of_week: now.format("dddd")
    }];
}
');
```

## Building from Source

### Prerequisites

- Rust toolchain (for building)
- Make
- Git

### Build Steps

1. **Clone the repository**
```bash
git clone --recurse-submodules https://github.com/nicosuave/wizard
cd wizard
```

2. **Set your LLM API key** (you'll need one of these)
```bash
# For OpenAI
export OPENAI_API_KEY="your-openai-api-key"

# OR for Anthropic (untested)
export ANTHROPIC_API_KEY="your-anthropic-api-key"
```

3. **Build the extension**
```bash
# Debug build (faster compilation, includes debug symbols)
make build

# OR Release build (optimized for performance)
make release
```

The extension will be built to:
- Debug: `build/debug/wizard.duckdb_extension`
- Release: `build/release/wizard.duckdb_extension`

### Running DuckDB with the Extension

Always run DuckDB in unsigned mode to load custom extensions:

```bash
# Start DuckDB in unsigned mode
duckdb -unsigned

# Or with a database file
duckdb -unsigned mydata.db
```

Then load the extension:
```sql
-- For release build
LOAD 'build/release/wizard.duckdb_extension';

-- Or for debug build
LOAD 'build/debug/wizard.duckdb_extension';

-- Now you can use it!
SELECT * FROM wizard('current weather in Seattle');
```

## Technical Details

### Capabilities & Limitations

The wizard extension operates with significant constraints:

**What it CAN do:**
- Make HTTP/HTTPS requests using Deno's built-in fetch
- Import and use any npm package via Deno's npm specifier
- Parse JSON responses natively
- Transform API responses into DuckDB tables
- Execute modern JavaScript/TypeScript code
- Cache responses for performance (60x speedup)

**What it CANNOT do:**
- Access the local filesystem (sandboxed)
- Make direct database connections
- Execute arbitrary system commands

The extension works by having the LLM generate JavaScript code that:
- Uses Deno's built-in fetch() for HTTP requests
- Can import npm packages like `npm:yahoo-finance2` or `npm:dayjs`
- Returns data as an array of objects

### Response Caching

Responses are cached to improve performance:
- Same queries return instantly from cache (60x faster)
- Cache persists for the session
- Use `--bust-cache` flag to force a fresh API call:
  ```sql
  SELECT * FROM wizard('bitcoin price --bust-cache');
  ```
- Debug mode available with `--debug` flag to see generated code

## Usage

### In DuckDB CLI

```bash
# Start DuckDB in unsigned mode
duckdb -unsigned

# Load the extension (adjust path based on your build type)
LOAD 'build/release/wizard.duckdb_extension';

# Query away!
SELECT * FROM wizard('show me Tesla stock data for the last week');
```

### Direct JavaScript Execution with js()

You can also execute JavaScript code directly without going through an LLM:

```sql
-- Simple example
SELECT * FROM js('
async function fetch_data() {
    return [
        { message: "Hello from JavaScript!", value: 42 }
    ];
}
');

-- Fetch data from APIs
SELECT * FROM js('
async function fetch_data() {
    const response = await fetch("https://api.coinbase.com/v2/exchange-rates?currency=BTC");
    const data = await response.json();
    return [{
        currency: "BTC",
        usd_price: parseFloat(data.data.rates.USD),
        timestamp: new Date().toISOString()
    }];
}
');

-- Use npm packages
SELECT * FROM js('
import { format } from "npm:date-fns";

async function fetch_data() {
    const dates = ["2024-01-01", "2024-06-15", "2024-12-25"];
    return dates.map(date => ({
        original: date,
        formatted: format(new Date(date), "MMMM do, yyyy"),
        day_of_week: format(new Date(date), "EEEE")
    }));
}
');
```

The `js()` function:
- Executes arbitrary JavaScript/TypeScript code
- Has access to Deno's fetch API and npm packages
- Must define an async `fetch_data()` function that returns an array of objects
- Runs in the same sandboxed Deno environment as wizard-generated code

## How it Works

1. Your natural language query is sent to an LLM (OpenAI or Anthropic)
2. The LLM generates JavaScript code that fetches the requested data
3. The code is executed in a sandboxed Deno environment
4. Deno handles all HTTP requests and npm package imports
5. Results are returned as a DuckDB table that you can query with SQL

The extension leverages Deno's secure runtime and built-in fetch API, plus its ability to import npm packages directly.

## Troubleshooting

- **"No API key found"**: Set your OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable
- **"Extension not found"**: Make sure you've built the extension with `make release`
- **Rate limits**: The extension uses real API calls, so you may hit rate limits with many queries
- **Slow first query**: The first query calls the LLM API; subsequent identical queries use cache

## Development

### Testing
```bash
make test_debug   # Test debug build
make test_release # Test release build
```

## License

MIT License - see LICENSE file for details.
