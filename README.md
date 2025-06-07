# DuckDB Wizard

Did you ever want to `SELECT * FROM clean_data_you_think_exists`? Well, now you can!

A DuckDB extension that lets you query any data using natural language.

> ⚠️ **WARNING: HIGHLY EXPERIMENTAL** ⚠️
> 
> This extension is in early experimental stage and should be used with extreme caution.
> - **DO NOT use in production environments**
> - The generated code may be unpredictable or incorrect
> - Always review the generated queries before executing
> - API calls to LLMs incur costs - monitor your usage
> - Proceed at your own risk!

Query any data using natural language in DuckDB! Powered by LLMs (OpenAI/Anthropic) and Python.

## Quick Start

```bash
# Clone the repo
git clone --recurse-submodules https://github.com/nicosuave/wizard
cd wizard

# Set your API key
export OPENAI_API_KEY="your-api-key"  # or ANTHROPIC_API_KEY

# Build the extension
make release

# Try it out!
echo "SELECT * FROM wizard('bitcoin price')" | duckdb -unsigned -c "LOAD 'build/release/wizard.duckdb_extension'; $(cat)"
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
```

## Installation

### 1. Requirements

- Python 3.8+ (required for PyO3 integration)
- DuckDB Python package
- Rust toolchain (for building)

```bash
# Install Python if needed
# macOS: brew install python3
# Ubuntu: sudo apt install python3 python3-pip

# Install DuckDB
pip install duckdb
```

Note: The extension uses Python for code execution but handles all HTTP requests internally via Rust - no additional Python packages needed!

### 2. Set your LLM API key

```bash
# For OpenAI
export OPENAI_API_KEY="your-openai-api-key"

# OR for Anthropic  
export ANTHROPIC_API_KEY="your-anthropic-api-key"
```

### 3. Build the extension

```bash
# Clone with submodules
git clone --recurse-submodules https://github.com/yourusername/duckdb-wizard
cd duckdb-wizard

# Configure and build
make configure
make release
```

## Technical Details

### Capabilities & Limitations

The wizard extension operates with significant constraints:

**What it CAN do:**
- Make HTTP/HTTPS requests via Rust-bridged client
- Parse JSON responses from web APIs
- Transform API responses into DuckDB tables
- Execute basic Python (no imports allowed)
- Cache responses for performance (60x speedup)

**What it CANNOT do:**
- Import ANY Python packages (no pandas, numpy, json, datetime, etc.)
- Use Python standard library modules
- Access the filesystem
- Make database connections
- Install Python packages

The extension works by having the LLM generate Python code that uses only:
- Basic Python syntax (loops, conditionals, list comprehensions)
- A special `http_get()` function injected by Rust
- String manipulation and basic data types

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
duckdb -unsigned

LOAD 'build/release/wizard.duckdb_extension';
SELECT * FROM wizard('show me Tesla stock data for the last week');
```

### In Python

```python
import duckdb

# Connect with unsigned extensions allowed
conn = duckdb.connect(':memory:', config={'allow_unsigned_extensions': 'true'})

# Load the extension
conn.execute("LOAD 'build/release/wizard.duckdb_extension'")

# Query any data!
df = conn.execute("SELECT * FROM wizard('cryptocurrency prices')").df()
print(df)
```

### Using the demo script

```bash
# Interactive wizard interface
python wizard.py

# Or run the demo
python wizard_demo.py
```

## How it Works

1. Your natural language query is sent to an LLM (OpenAI or Anthropic)
2. The LLM generates constrained Python code using only basic syntax
3. HTTP requests are handled by Rust code bridged to Python
4. The code is executed in a restricted Python environment
5. Results are returned as a DuckDB table that you can query with SQL

The key innovation is that all HTTP functionality is implemented in Rust and exposed to Python as a simple `http_get()` function, eliminating Python package dependencies entirely.

## Troubleshooting

- **"No API key found"**: Set your OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable
- **"Extension not found"**: Make sure you've built the extension with `make release`
- **"Import error"**: The extension cannot import Python packages - this is by design
- **Rate limits**: The extension uses real API calls, so you may hit rate limits with many queries
- **Slow first query**: The first query calls the LLM API; subsequent identical queries use cache

## Development

### Requirements
- Rust toolchain
- Python 3.8+
- Make
- Git

### Building from source
```bash
# Clone with submodules
git clone --recurse-submodules https://github.com/yourusername/duckdb-wizard
cd duckdb-wizard

# Configure and build
make configure
make release
```

### Testing
```bash
make test_debug   # Test debug build
make test_release # Test release build
```

## License

MIT License - see LICENSE file for details.
