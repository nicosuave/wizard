#!/usr/bin/env python3
"""
🧙‍♂️ DuckDB Wizard - Interactive Query Interface

Query any data using natural language!
"""

import sys
import os
from pathlib import Path

def main():
    # Check API key
    if not (os.getenv('OPENAI_API_KEY') or os.getenv('ANTHROPIC_API_KEY')):
        print("\n❌ No API key found!")
        print("\nSet one of these environment variables:")
        print("  export OPENAI_API_KEY='your-key'")
        print("  export ANTHROPIC_API_KEY='your-key'")
        sys.exit(1)
    
    # Check for required packages
    required_packages = ['duckdb', 'pandas']
    missing_packages = []
    
    for package in required_packages:
        try:
            __import__(package)
        except ImportError:
            missing_packages.append(package)
    
    if missing_packages:
        print(f"\n❌ Missing required packages: {', '.join(missing_packages)}")
        print("\nInstall them with:")
        print("  uv sync --extra basic")
        print("\nOr for all packages:")
        print("  uv sync --all-extras")
        sys.exit(1)
    
    # Now import DuckDB
    import duckdb
    
    # Find extension
    extension_path = Path(__file__).parent / "build" / "release" / "wizard.duckdb_extension"
    if not extension_path.exists():
        print(f"\n❌ Extension not found at {extension_path}")
        print("Build it with: make release")
        sys.exit(1)
    
    print(f"\n🔌 Loading wizard extension...")
    conn = duckdb.connect(':memory:', config={'allow_unsigned_extensions': 'true'})
    conn.execute(f"LOAD '{extension_path}'")
    print("✅ Ready to cast spells!\n")
    
    # Show examples
    print("Example queries:")
    print("  • tech stocks AAPL MSFT last week")
    print("  • bitcoin price in USD")
    print("  • weather in San Francisco")
    print("  • create sample data with numbers 1 to 10")
    
    # Interactive mode
    print("\nEnter your data queries in natural language (or 'quit' to exit):\n")
    
    while True:
        try:
            query = input("🔮 wizard> ").strip()
            
            if query.lower() in ['quit', 'exit', 'q']:
                print("\n👋 Farewell, data seeker!")
                break
            
            if not query:
                continue
            
            # Execute wizard query
            result = conn.execute(f"SELECT * FROM wizard('{query}')").fetchdf()
            
            if result.empty:
                print("No data returned")
            else:
                print(f"\n{result}\n")
                print(f"({len(result)} rows)")
                
        except KeyboardInterrupt:
            print("\n\n👋 Farewell!")
            break
        except Exception as e:
            print(f"\n❌ Error: {e}")
            if "not installed" in str(e) or "No module named" in str(e):
                print("\n💡 Some packages might be missing. Install them with:")
                print("   uv sync --extra basic   # For basic packages")
                print("   uv sync --all-extras    # For all supported packages")

if __name__ == "__main__":
    print("🧙‍♂️ Welcome to DuckDB Wizard!\n")
    main()