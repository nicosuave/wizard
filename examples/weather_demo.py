#!/usr/bin/env python3
"""Final weather test with nice formatting"""

import duckdb
import os

# Connect to DuckDB
conn = duckdb.connect(':memory:', config={'allow_unsigned_extensions': 'true'})

# Load the extension
conn.execute("LOAD 'build/release/wizard.duckdb_extension'")

try:
    # Query for Seattle weather
    print("🧙‍♂️ Wizard Extension - Seattle Weather Forecast\n")
    print("Querying: 'Seattle weather forecast for the next 7 days'")
    print("-" * 60)
    
    result = conn.execute("SELECT * FROM wizard('Seattle weather forecast for the next 7 days')").fetchall()
    
    print(f"\n✅ Success! Retrieved {len(result)} days of weather data\n")
    print("Seattle 7-Day Weather Forecast:")
    print("-" * 60)
    print(f"{'Date':<15} {'Temperature (°F)':<18} {'Conditions':<20}")
    print("-" * 60)
    
    for row in result:
        # Extract values, handling the String() wrapper
        date = str(row[0]).replace('String("', '').replace('")', '')
        temp = row[1]
        condition = row[2]
        print(f"{date:<15} {temp:<18.1f} {condition:<20}")
    
    print("-" * 60)
    print("\n💡 Note: Due to PyO3 limitations, this generates realistic mock data")
    print("   instead of making real API calls. For production use, consider")
    print("   using a separate service for API calls.")
    
except Exception as e:
    print(f"❌ Error: {e}")
    print(f"Error type: {type(e)}")