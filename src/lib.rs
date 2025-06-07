extern crate duckdb;
extern crate duckdb_loadable_macros;
extern crate libduckdb_sys;

mod llm;
mod deno_executor;

use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vtab::{BindInfo, InitInfo, TableFunctionInfo, VTab},
    Connection, Result,
};
use duckdb_loadable_macros::duckdb_entrypoint_c_api;
use libduckdb_sys as ffi;
use std::{
    error::Error,
    ffi::CString,
    sync::atomic::Ordering,
    collections::HashMap,
    sync::Mutex,
};
use chrono::Local;

use crate::llm::{LLMClient, ColumnSchema};
use crate::deno_executor::{DenoExecutor, JsValue};

// Global cache for LLM responses
lazy_static::lazy_static! {
    static ref RESPONSE_CACHE: Mutex<HashMap<String, CachedResponse>> = Mutex::new(HashMap::new());
}

#[derive(Clone)]
struct CachedResponse {
    javascript_code: String,
    schema: Vec<ColumnSchema>,
    timestamp: chrono::DateTime<Local>,
}

#[repr(C)]
struct WizardBindData {
    query: String,
    schema: Vec<ColumnSchema>,
    data: Vec<HashMap<String, JsValue>>,
}

#[repr(C)]
struct WizardInitData {
    current_row: std::sync::atomic::AtomicUsize,
}

struct WizardVTab;

impl VTab for WizardVTab {
    type InitData = WizardInitData;
    type BindData = WizardBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn std::error::Error>> {
        let full_query = bind.get_parameter(0).to_string();
        
        // Check if query ends with debug/cache flags
        let mut query = full_query.clone();
        let mut debug = false;
        let mut bust_cache = false;
        
        // Simple flag parsing from query string
        if query.ends_with(" --debug") {
            debug = true;
            query = query.trim_end_matches(" --debug").to_string();
        }
        if query.ends_with(" --bust-cache") {
            bust_cache = true;
            query = query.trim_end_matches(" --bust-cache").to_string();
        }
        if query.ends_with(" --debug --bust-cache") {
            debug = true;
            bust_cache = true;
            query = query.trim_end_matches(" --debug --bust-cache").to_string();
        }
        
        // Check cache first (unless bust_cache is true)
        let cached_response = if !bust_cache {
            RESPONSE_CACHE.lock().unwrap().get(&query).cloned()
        } else {
            None
        };
        
        let (javascript_code, schema) = if let Some(cached) = cached_response {
            if debug {
                eprintln!("Using cached response for query: {}", query);
            }
            (cached.javascript_code, cached.schema)
        } else {
            // Initialize LLM client and get code + schema
            let llm_client = LLMClient::new()
                .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn std::error::Error>)?;
            
            let llm_response = llm_client.generate_data_fetch_code(&query, debug)?;
            
            // Cache the response
            RESPONSE_CACHE.lock().unwrap().insert(
                query.clone(),
                CachedResponse {
                    javascript_code: llm_response.javascript_code.clone(),
                    schema: llm_response.schema.clone(),
                    timestamp: Local::now(),
                }
            );
            
            (llm_response.javascript_code, llm_response.schema)
        };
        
        // Add columns based on the schema
        for col in &schema {
            let logical_type = match col.data_type.as_str() {
                "varchar" => LogicalTypeId::Varchar,
                "double" => LogicalTypeId::Double,
                "bigint" => LogicalTypeId::Bigint,
                "date" => LogicalTypeId::Varchar, // We'll use varchar for dates for simplicity
                _ => LogicalTypeId::Varchar,
            };
            bind.add_result_column(&col.name, LogicalTypeHandle::from(logical_type));
        }
        
        // Execute the JavaScript code to get the data
        let executor = DenoExecutor::new();
        let data = executor.execute_code(&javascript_code, debug)?;
        
        Ok(WizardBindData { 
            query,
            schema,
            data,
        })
    }

    fn init(_: &InitInfo) -> Result<Self::InitData, Box<dyn std::error::Error>> {
        Ok(WizardInitData {
            current_row: std::sync::atomic::AtomicUsize::new(0),
        })
    }

    fn func(func: &TableFunctionInfo<Self>, output: &mut DataChunkHandle) -> Result<(), Box<dyn std::error::Error>> {
        let init_data = func.get_init_data();
        let bind_data = func.get_bind_data();
        
        let current_row = init_data.current_row.load(Ordering::Relaxed);
        
        if current_row >= bind_data.data.len() {
            output.set_len(0);
            return Ok(());
        }
        
        // DuckDB processes in chunks, so we can return multiple rows at once
        let chunk_size = std::cmp::min(2048, bind_data.data.len() - current_row);
        let end_row = current_row + chunk_size;
        
        // Fill columns based on schema
        for (col_idx, col_schema) in bind_data.schema.iter().enumerate() {
            match col_schema.data_type.as_str() {
                "varchar" => {
                    for (chunk_idx, row_idx) in (current_row..end_row).enumerate() {
                        let row = &bind_data.data[row_idx];
                        let value = row.get(&col_schema.name)
                            .and_then(|v| match v {
                                JsValue::String(s) => Some(s.as_str()),
                                _ => None,
                            })
                            .unwrap_or("");
                        let c_str = CString::new(value)?;
                        output.flat_vector(col_idx).insert(chunk_idx, c_str);
                    }
                },
                "double" => {
                    let mut vec = output.flat_vector(col_idx);
                    let slice = vec.as_mut_slice::<f64>();
                    for (chunk_idx, row_idx) in (current_row..end_row).enumerate() {
                        let row = &bind_data.data[row_idx];
                        let value = row.get(&col_schema.name)
                            .and_then(|v| match v {
                                JsValue::Float(f) => Some(*f),
                                JsValue::Integer(i) => Some(*i as f64),
                                _ => None,
                            })
                            .unwrap_or(0.0);
                        slice[chunk_idx] = value;
                    }
                },
                "bigint" => {
                    let mut vec = output.flat_vector(col_idx);
                    let slice = vec.as_mut_slice::<i64>();
                    for (chunk_idx, row_idx) in (current_row..end_row).enumerate() {
                        let row = &bind_data.data[row_idx];
                        let value = row.get(&col_schema.name)
                            .and_then(|v| match v {
                                JsValue::Integer(i) => Some(*i),
                                JsValue::Float(f) => Some(*f as i64),
                                _ => None,
                            })
                            .unwrap_or(0);
                        slice[chunk_idx] = value;
                    }
                },
                _ => {
                    // Default to varchar
                    for (chunk_idx, row_idx) in (current_row..end_row).enumerate() {
                        let row = &bind_data.data[row_idx];
                        let value = row.get(&col_schema.name)
                            .map(|v| match v {
                                JsValue::String(s) => s.clone(),
                                JsValue::Float(f) => f.to_string(),
                                JsValue::Integer(i) => i.to_string(),
                                JsValue::Boolean(b) => b.to_string(),
                                JsValue::Null => "".to_string(),
                            })
                            .unwrap_or_else(|| "".to_string());
                        let c_str = CString::new(value)?;
                        output.flat_vector(col_idx).insert(chunk_idx, c_str);
                    }
                }
            }
        }
        
        output.set_len(chunk_size);
        init_data.current_row.store(end_row, Ordering::Relaxed);
        
        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        Some(vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)])
    }
}

const EXTENSION_NAME: &str = env!("CARGO_PKG_NAME");

#[duckdb_entrypoint_c_api()]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>> {
    con.register_table_function::<WizardVTab>("wizard")
        .expect("Failed to register wizard table function");
    Ok(())
}