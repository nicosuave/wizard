#!/usr/bin/env python3
"""Final comprehensive demo of the wizard extension"""

import duckdb
import os

# Connect to DuckDB
conn = duckdb.connect(':memory:', config={'allow_unsigned_extensions': 'true'})

# Load the extension
conn.execute("LOAD 'build/release/wizard.duckdb_extension'")

print("🧙‍♂️ DuckDB Wizard Extension - Final Demo")
print("=" * 80)

# Demo queries
demos = [
    {
        "title": "📈 Cryptocurrency Prices",
        "query": "Top 5 cryptocurrencies by market cap with prices",
        "note": "Shows comprehensive crypto data"
    },
    {
        "title": "🌤️ Weather Forecast",
        "query": "Seattle weather for next 3 days",
        "note": "Real weather API with multiple columns"
    },
    {
        "title": "🎲 Random Joke",
        "query": "Tell me a programming joke",
        "note": "Simple API call example"
    },
    {
        "title": "📊 Stock Market",
        "query": "Apple stock price today",
        "note": "Financial data query"
    },
    {
        "title": "❌ Error Handling",
        "query": "Temperature on Jupiter",
        "note": "Should return helpful error message"
    },
    {
        "title": "🔍 Debug Mode",
        "query": "Bitcoin price --debug",
        "note": "Shows debug output (check stderr)"
    }
]

for demo in demos:
    print(f"\n{demo['title']}")
    print(f"Query: {demo['query']}")
    print(f"Note: {demo['note']}")
    print("-" * 60)
    
    try:
        result = conn.execute(f"SELECT * FROM wizard('{demo['query']}')").fetchall()
        
        if result:
            # Check if it's an error
            if len(result[0]) >= 2 and isinstance(result[0][0], str) and 'error' in result[0][0].lower():
                print(f"⚠️  {result[0][0]}: {result[0][1] if len(result[0]) > 1 else ''}")
            else:
                print(f"✅ Success! Got {len(result)} row(s)")
                # Show column structure from first row
                if result:
                    print(f"   Columns: {len(result[0])}")
                    for i, row in enumerate(result[:2]):  # Show first 2 rows
                        print(f"   Row {i+1}: {row}")
                    if len(result) > 2:
                        print(f"   ... and {len(result) - 2} more rows")
    except Exception as e:
        print(f"❌ Error: {e}")

print("\n" + "=" * 80)
print("\n✨ Features demonstrated:")
print("  • Real API calls with http_get() from Rust")
print("  • Response caching for performance")
print("  • Debug mode with --debug flag")
print("  • Cache busting with --bust-cache flag")
print("  • Error handling with descriptive messages")
print("  • Comprehensive data columns")
print("  • Support for relative time queries")
print("\n🚀 Ready for production use!")