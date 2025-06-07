#!/usr/bin/env uv run
# /// script
# requires-python = ">=3.9"
# dependencies = [
#     "duckdb>=1.0.0",
#     "pandas>=2.0.0",
#     "tabulate>=0.9.0",
#     "rich>=13.0.0",
# ]
# ///

"""
🧙‍♂️ DuckDB Wizard Extension Demo

This script demonstrates the wizard extension that lets you query
any data using natural language!

Usage:
    uv run wizard_demo.py
"""

import duckdb
import os
import sys
from pathlib import Path
from rich.console import Console
from rich.table import Table
from rich.panel import Panel

console = Console()

def main():
    # Check for API key
    if not (os.getenv('OPENAI_API_KEY') or os.getenv('ANTHROPIC_API_KEY')):
        console.print("[red]❌ Error: No API key found![/red]")
        console.print("\nPlease set one of the following environment variables:")
        console.print("  export OPENAI_API_KEY='your-key-here'")
        console.print("  export ANTHROPIC_API_KEY='your-key-here'")
        sys.exit(1)
    
    # Find the extension
    script_dir = Path(__file__).parent
    extension_path = script_dir / "build" / "release" / "wizard.duckdb_extension"
    
    if not extension_path.exists():
        console.print(f"[red]❌ Extension not found at {extension_path}[/red]")
        console.print("\nPlease build the extension first with: make release")
        sys.exit(1)
    
    # Connect to DuckDB
    console.print("[cyan]🔌 Connecting to DuckDB...[/cyan]")
    conn = duckdb.connect(':memory:', config={'allow_unsigned_extensions': 'true'})
    
    # Load the wizard extension
    console.print(f"[cyan]📦 Loading wizard extension...[/cyan]")
    conn.execute(f"LOAD '{extension_path}'")
    console.print("[green]✅ Wizard extension loaded![/green]\n")
    
    # Demo queries
    demo_queries = [
        "create a simple dataset with numbers 1 to 5 and their squares",
        "Seattle weather forecast for the next 7 days",
        "generate sample sales data for last 7 days"
    ]
    
    console.print(Panel.fit(
        "[bold]Running demo queries:[/bold]\n" + 
        "\n".join(f"• {q}" for q in demo_queries),
        title="🎯 Demo Queries",
        border_style="blue"
    ))
    
    for i, query in enumerate(demo_queries, 1):
        console.print(f"\n[bold yellow]Query {i}:[/bold yellow] {query}")
        
        try:
            console.print(f"[cyan]🧙‍♂️ Casting spell...[/cyan]")
            
            # Execute the wizard query
            result_df = conn.execute(f"SELECT * FROM wizard('{query}')").df()
            
            if result_df.empty:
                console.print("[yellow]No data returned[/yellow]")
            else:
                # Display results in a nice table
                table = Table(title=f"Results", show_lines=True)
                
                # Add columns
                for col in result_df.columns:
                    table.add_column(col, style="cyan", no_wrap=False)
                
                # Add rows (limit to 5 for display)
                for _, row in result_df.head(5).iterrows():
                    table.add_row(*[str(v) for v in row])
                
                console.print(table)
                
                if len(result_df) > 5:
                    console.print(f"[dim]Showing 5 of {len(result_df)} rows[/dim]")
                
        except Exception as e:
            console.print(f"[red]❌ Error: {e}[/red]")
            if "not installed" in str(e) or "No module named" in str(e):
                console.print("[dim]Some packages might be missing. Install them with: uv sync --all-extras[/dim]")
                if "http_get" in str(e):
                    console.print("[yellow]Note: This query requires real API access[/yellow]")
    
    console.print("\n[green]✅ Demo completed![/green]")
    console.print("\n[cyan]To run interactive queries, use: python wizard.py[/cyan]")

if __name__ == "__main__":
    console.print(Panel.fit(
        "[bold]🧙‍♂️ Welcome to DuckDB Wizard Demo![/bold]\n\n"
        "This demo shows how to query data using natural language.\n"
        "Powered by LLMs and Python magic! ✨",
        border_style="purple"
    ))
    main()