-- Test the wizard extension with real LLM API
LOAD 'build/debug/wizard.duckdb_extension';

-- Test query - should use actual LLM API if key is set
SELECT * FROM wizard('weather data for San Francisco last 3 days');