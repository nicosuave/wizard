#!/usr/bin/env python3
"""Test the cache and debug functionality"""

import duckdb
import time

# Connect to DuckDB
conn = duckdb.connect(':memory:', config={'allow_unsigned_extensions': 'true'})

# Load the extension
conn.execute("LOAD 'build/release/wizard.duckdb_extension'")

print("🧙‍♂️ Testing Cache and Debug Features\n")
print("=" * 70)

# Test 1: First query (will hit API)
print("\n1️⃣ First query - should make API call:")
start = time.time()
result1 = conn.execute("SELECT * FROM wizard('Bitcoin price in USD')").fetchall()
time1 = time.time() - start
print(f"   Result: {result1[0] if result1 else 'No data'}")
print(f"   Time: {time1:.2f}s")

# Test 2: Same query (should use cache)
print("\n2️⃣ Same query - should use cache (faster):")
start = time.time()
result2 = conn.execute("SELECT * FROM wizard('Bitcoin price in USD')").fetchall()
time2 = time.time() - start
print(f"   Result: {result2[0] if result2 else 'No data'}")
print(f"   Time: {time2:.2f}s")
print(f"   ⚡ Speed up: {time1/time2:.1f}x faster")

# Test 3: With debug=true
print("\n3️⃣ With debug enabled:")
print("   (Check stderr for debug output)")
result3 = conn.execute("SELECT * FROM wizard('Seattle weather --debug')").fetchall()
print(f"   Result rows: {len(result3)}")

# Test 4: Bust cache
print("\n4️⃣ Bust cache - force new API call:")
start = time.time()
result4 = conn.execute("SELECT * FROM wizard('Bitcoin price in USD --bust-cache')").fetchall()
time4 = time.time() - start
print(f"   Result: {result4[0] if result4 else 'No data'}")
print(f"   Time: {time4:.2f}s (should be similar to first call)")

# Test 5: Relative time query
print("\n5️⃣ Relative time query:")
result5 = conn.execute("SELECT * FROM wizard('Weather for Seattle yesterday --debug')").fetchall()
print(f"   Result rows: {len(result5)}")
print("   (Check debug output to see if current date is used)")

print("\n" + "=" * 70)
print("✨ Test completed!")
print("\nUsage: wizard('query [--debug] [--bust-cache]')")
print("  - --debug: show debug output")
print("  - --bust-cache: bypass cache and force new API call")