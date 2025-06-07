use std::collections::HashMap;
use std::process::Command;
use serde_json;

pub struct DenoExecutor;

impl DenoExecutor {
    pub fn new() -> Self {
        DenoExecutor
    }
    
    pub fn execute_code(&self, code: &str, debug: bool) -> Result<Vec<HashMap<String, JsValue>>, Box<dyn std::error::Error>> {
        if debug {
            eprintln!("Executing JavaScript code:\n{}", code);
        }
        
        // Write code to temporary file
        let mut temp_file = std::env::temp_dir();
        temp_file.push(format!("duckdb_wizard_{}.js", std::process::id()));
        
        // Wrap the code to ensure it outputs JSON
        let wrapped_code = format!(r#"
{}

// Execute and output result
const result = await fetch_data();
console.log(JSON.stringify(result));
"#, code);
        
        std::fs::write(&temp_file, wrapped_code)?;
        
        // Execute with Deno
        let output = Command::new("deno")
            .args(&["run", "--allow-net", "--quiet"])
            .arg(&temp_file)
            .output()?;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_file);
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Deno execution failed: {}", error).into());
        }
        
        // Parse output
        let output_str = String::from_utf8(output.stdout)?;
        let json_result: Vec<serde_json::Value> = serde_json::from_str(&output_str)?;
        
        // Convert to our format
        let mut rows = Vec::new();
        for item in json_result {
            if let serde_json::Value::Object(obj) = item {
                let mut row = HashMap::new();
                for (key, value) in obj {
                    row.insert(key, json_to_value(value));
                }
                rows.push(row);
            }
        }
        
        Ok(rows)
    }
}

#[derive(Debug, Clone)]
pub enum JsValue {
    String(String),
    Float(f64),
    Integer(i64),
    Boolean(bool),
    Null,
}

fn json_to_value(value: serde_json::Value) -> JsValue {
    match value {
        serde_json::Value::String(s) => JsValue::String(s),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                JsValue::Float(f)
            } else {
                JsValue::Float(0.0)
            }
        },
        serde_json::Value::Bool(b) => JsValue::Boolean(b),
        serde_json::Value::Null => JsValue::Null,
        _ => JsValue::String(value.to_string()),
    }
}