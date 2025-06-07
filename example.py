#!/usr/bin/env python3
"""
Example of using the wizard extension in DuckDB
"""
import duckdb
import os

# Make sure you have an API key set
if not (os.getenv('OPENAI_API_KEY') or os.getenv('ANTHROPIC_API_KEY')):
    print("Please set OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable")
    exit(1)

# Connect to DuckDB with unsigned extensions allowed
conn = duckdb.connect(':memory:', config={'allow_unsigned_extensions': 'true'})

# Load the wizard extension
extension_path = os.path.join(os.path.dirname(__file__), 'build/release/rusty_quack.duckdb_extension')
conn.execute(f"LOAD '{extension_path}'")

# Example queries
queries = [
    "NFLX stock data last 5 days",
    "current bitcoin price",
    "population of top 5 US cities",
]

for query in queries:
    print(f"\n🔮 Querying: {query}")
    print("-" * 50)
    try:
        result = conn.execute(f"SELECT * FROM wizard('{query}')").fetchdf()
        print(result)
    except Exception as e:
        print(f"Error: {e}")

# You can also use it in complex queries!
print("\n📊 Complex query example:")
conn.execute("""
    SELECT 
        ticker,
        AVG(close) as avg_close,
        MAX(high) as max_high,
        MIN(low) as min_low
    FROM wizard('tech stocks AAPL MSFT GOOGL last week')
    GROUP BY ticker
    ORDER BY avg_close DESC
""").show()