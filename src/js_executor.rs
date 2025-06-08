use rustyscript::{Runtime, Module, RuntimeOptions};
use std::collections::{HashMap, HashSet};

pub struct JsExecutor;

impl JsExecutor {
    pub fn new() -> Self {
        JsExecutor
    }

    pub fn execute_code(&self, code: &str, debug: bool) -> Result<Vec<HashMap<String, JsValue>>, Box<dyn std::error::Error>> {
        if debug {
            eprintln!("Executing JavaScript code:\n{}", code);
        }

        // Create runtime with default options
        // The url_import feature enables https:// imports automatically
        let mut runtime = Runtime::new(RuntimeOptions::default())?;

        // Wrap the code in a module with an exported function
        let module_code = format!(r#"
{}

export {{ fetch_data }};
"#, code);

        // Create and load the module
        let module = Module::new("wizard.js", &module_code);
        let module_handle = runtime.load_module(&module)?;

        // Call the async fetch_data function
        let result: serde_json::Value = runtime.tokio_runtime().block_on(async {
            runtime.call_function_async(
                Some(&module_handle),
                "fetch_data",
                rustyscript::json_args!()
            ).await
        })?;

        if debug {
            eprintln!("Result: {}", serde_json::to_string_pretty(&result)?);
        }

        // Convert result to expected format
        let mut rows = Vec::new();
        
        match result {
            serde_json::Value::Array(array) => {
                for item in array {
                    if let serde_json::Value::Object(obj) = item {
                        let mut row = HashMap::new();
                        for (key, value) in obj {
                            row.insert(key, json_to_value(value));
                        }
                        rows.push(row);
                    }
                }
            },
            serde_json::Value::Object(obj) => {
                let mut row = HashMap::new();
                for (key, value) in obj {
                    row.insert(key, json_to_value(value));
                }
                rows.push(row);
            },
            _ => return Err("Unexpected result type from JavaScript execution".into()),
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