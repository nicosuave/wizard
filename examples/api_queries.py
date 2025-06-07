#!/usr/bin/env python3
"""Test various API queries with the wizard extension"""

import duckdb
import os

# Connect to DuckDB
conn = duckdb.connect(':memory:', config={'allow_unsigned_extensions': 'true'})

# Load the extension
conn.execute("LOAD 'build/release/wizard.duckdb_extension'")

# Test queries
test_queries = [
    ("Bitcoin price in USD", "Cryptocurrency data"),
    ("Recent earthquakes for past month", "USGS earthquake data"),
    ("Random programming joke", "Joke API"),
    ("Current weather in Seattle", "Weather information"),
    ("Top 5 cryptocurrencies by market cap", "Crypto market overview"),
]

print("🧙‍♂️ DuckDB Wizard - Testing Various API Queries\n")
print("=" * 80)

for query, description in test_queries:
    print(f"\n📊 {description}")
    print(f"Query: '{query}'")
    print("-" * 60)
    
    try:
        result = conn.execute(f"SELECT * FROM wizard('{query}')").fetchall()
        
        if result:
            # Check if it's an error response
            first_row = result[0]
            if len(first_row) >= 2 and 'error' in str(first_row[0]).lower():
                print(f"❌ Error: {first_row[0]}")
                if len(first_row) > 1:
                    print(f"   Message: {first_row[1]}")
            else:
                print(f"✅ Success! Got {len(result)} row(s)")
                for i, row in enumerate(result[:3]):  # Show first 3 rows
                    print(f"   Row {i+1}: {row}")
                if len(result) > 3:
                    print(f"   ... and {len(result) - 3} more rows")
        else:
            print("⚠️  No data returned")
            
    except Exception as e:
        print(f"❌ Extension error: {e}")

print("\n" + "=" * 80)
print("✨ Test completed!")