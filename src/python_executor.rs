use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;

#[pyfunction]
fn http_get(url: String) -> PyResult<String> {
    // Use reqwest blocking client for simplicity
    let client = reqwest::blocking::Client::new();
    
    match client.get(&url).send() {
        Ok(response) => {
            match response.text() {
                Ok(text) => Ok(text),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
                    format!("Failed to read response: {}", e)
                ))
            }
        },
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
            format!("HTTP request failed: {}", e)
        ))
    }
}

pub struct PythonExecutor {
    // We'll initialize Python on demand
}

impl PythonExecutor {
    pub fn new() -> Self {
        PythonExecutor {}
    }
    
    pub fn execute_code(&self, code: &str, debug: bool) -> Result<Vec<HashMap<String, PyValue>>, Box<dyn std::error::Error>> {
        Python::with_gil(|py| {
            // Register our http_get function with Python
            let fun = wrap_pyfunction_bound!(http_get, py)?;
            let globals = py.eval_bound("globals()", None, None)?;
            globals.set_item("http_get", fun)?;
            
            // Execute the provided code
            if debug {
                eprintln!("Executing Python code:\n{}", code);
            }
            
            // Execute the code in the main namespace
            py.run_bound(code, None, None)?;
            
            // Call the fetch_data function
            let fetch_data = py.eval_bound("fetch_data", None, None)?;
            let result = fetch_data.call0()?;
            
            // Convert Python result to Rust
            let mut rows = Vec::new();
            
            if let Ok(py_list) = result.downcast::<PyList>() {
                for item in py_list.iter() {
                    if let Ok(py_dict) = item.downcast::<PyDict>() {
                        let mut row = HashMap::new();
                        
                        for (key, value) in py_dict.iter() {
                            let key_str: String = key.extract()?;
                            let py_value = python_to_value(&value)?;
                            row.insert(key_str, py_value);
                        }
                        
                        rows.push(row);
                    }
                }
            }
            
            Ok(rows)
        })
    }
}

#[derive(Debug, Clone)]
pub enum PyValue {
    String(String),
    Float(f64),
    Integer(i64),
    Boolean(bool),
    None,
}

fn python_to_value(obj: &Bound<'_, PyAny>) -> PyResult<PyValue> {
    if let Ok(s) = obj.extract::<String>() {
        Ok(PyValue::String(s))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(PyValue::Float(f))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(PyValue::Integer(i))
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(PyValue::Boolean(b))
    } else if obj.is_none() {
        Ok(PyValue::None)
    } else {
        // Try to convert to string as fallback
        Ok(PyValue::String(obj.str()?.to_string()))
    }
}