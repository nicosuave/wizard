use serde::{Deserialize, Serialize};
use std::env;
use async_openai::{
    types::{CreateChatCompletionRequestArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs},
    Client as OpenAIClient,
};
use misanthropy::{Anthropic, MessagesRequest, Message, Role, Content};
use chrono;

#[derive(Debug, Serialize, Deserialize)]
pub struct LLMResponse {
    #[serde(alias = "python_code", alias = "javascript_code")]
    pub javascript_code: String,
    pub schema: Vec<ColumnSchema>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: String, // "varchar", "double", "bigint", "date"
}

pub enum LLMProvider {
    OpenAI(String),
    Anthropic(String),
}

pub struct LLMClient {
    provider: LLMProvider,
}

impl LLMClient {
    pub fn new() -> Result<Self, String> {
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            Ok(LLMClient {
                provider: LLMProvider::OpenAI(api_key),
            })
        } else if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
            Ok(LLMClient {
                provider: LLMProvider::Anthropic(api_key),
            })
        } else {
            Err("No API key found. Set OPENAI_API_KEY or ANTHROPIC_API_KEY".to_string())
        }
    }
    
    pub fn generate_data_fetch_code(&self, query: &str, debug: bool) -> Result<LLMResponse, Box<dyn std::error::Error>> {
        let prompt = self.build_prompt(query);
        
        if debug {
            eprintln!("Calling LLM API with query: {}", query);
        }
        
        // Create a tokio runtime for async operations
        let rt = tokio::runtime::Runtime::new()?;
        
        let result = match &self.provider {
            LLMProvider::OpenAI(api_key) => {
                if debug {
                    eprintln!("Using OpenAI API");
                }
                rt.block_on(self.call_openai(api_key, &prompt, debug))
            }
            LLMProvider::Anthropic(api_key) => {
                if debug {
                    eprintln!("Using Anthropic API");
                }
                rt.block_on(self.call_anthropic(api_key, &prompt, debug))
            }
        };
        
        if debug {
            match &result {
                Ok(response) => {
                    eprintln!("LLM Response received successfully");
                    eprintln!("Generated Python code:\n{}", response.python_code);
                    eprintln!("Schema: {:?}", response.schema);
                }
                Err(e) => {
                    eprintln!("LLM API Error: {}", e);
                }
            }
        }
        
        result
    }
    
    fn build_prompt(&self, query: &str) -> String {
        let current_time = chrono::Local::now();
        format!(
            r#"You are a data wizard that helps fetch data based on natural language queries.

Current date and time: {}

User query: "{}"

IMPORTANT: If the user asks for data with relative time periods (e.g., "last 7 days", "past week", "yesterday"), 
calculate the dates based on the current date above. Do NOT use fixed dates.

Generate JavaScript code that fetches this data and returns it as an array of objects.
Also provide the schema of the data that will be returned.

IMPORTANT RULES:
1. The JavaScript code should define an async function called `fetch_data()` that returns an array of objects
2. Each object represents a row of data
3. Use the built-in fetch() function for HTTP requests (Deno has it built-in)
4. You have access to all modern JavaScript/TypeScript features and Deno APIs
5. Parse JSON responses with await response.json()
6. For external packages, use Deno's npm specifier to import npm packages directly:
   - import package from "npm:package-name@version"
   - Example: import dayjs from "npm:dayjs@1.11.10"
   - Example: import _ from "npm:lodash@4.17.21"
7. Prefer using real npm packages when they provide better functionality:
   - For stock data: import yahooFinance from "npm:yahoo-finance2"
   - For crypto data: Consider using a proper SDK if available
   - Date formatting: import dayjs from "npm:dayjs" or import { format } from "npm:date-fns"
   - Data processing: import _ from "npm:lodash" for complex operations
   - CSV parsing: import { parse } from "npm:csv-parse/sync"
   - However, for simple HTTP APIs, fetch() is often sufficient
8. ALWAYS use REAL, FREE APIs. NEVER make up API endpoints. Examples of real free APIs:
   - Weather: wttr.in (e.g., await fetch('https://wttr.in/Seattle?format=j1'))
   - Earthquakes: USGS (e.g., await fetch('https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_month.geojson'))
   - Crypto: CoinGecko free tier (e.g., await fetch('https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd'))
   - IP info: await fetch('https://ipapi.co/json/')
   - Jokes: await fetch('https://official-joke-api.appspot.com/random_joke')
9. If no suitable FREE API exists for the query, return an error:
   return [{error: 'No free API available', message: 'Cannot find a free API for: [query description]'}]
10. If API call fails, return the error details:
   try {
       const response = await fetch(url);
       const data = await response.json();
       // process data
   } catch (e) {
       return [{error: 'API call failed', message: e.message, url: url}];
   }
11. NEVER return empty array [] - always return error details if something goes wrong
12. IMPORTANT: Include ALL relevant columns that make sense for the query:
    - For weather: include date, location, temperature, conditions, humidity, wind, etc.
    - For earthquakes: include magnitude, place, time, depth, coordinates, etc.
    - For crypto: include coin name, symbol, price, market cap, 24h change if available
    - Always prefer more complete data over minimal responses
13. The schema should match the data exactly and include all columns you return

Return your response as JSON in this exact format:
{{
    "javascript_code": "async function fetch_data() {{\n    // Your code here\n    return data;\n}}",
    "schema": [
        {{"name": "column1", "data_type": "varchar"}},
        {{"name": "column2", "data_type": "double"}}
    ]
}}

Supported data types: varchar, double, bigint, date

Example for "recent earthquakes":
{{
    "javascript_code": "async function fetch_data() {{\n    const response = await fetch('https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/significant_month.geojson');\n    const data = await response.json();\n    \n    return data.features.map(feature => ({{\n        magnitude: feature.properties.mag,\n        place: feature.properties.place,\n        time: new Date(feature.properties.time).toISOString(),\n        depth: feature.geometry.coordinates[2],\n        latitude: feature.geometry.coordinates[1],\n        longitude: feature.geometry.coordinates[0],\n        type: feature.properties.type\n    }}));\n}}",
    "schema": [
        {{"name": "magnitude", "data_type": "double"}},
        {{"name": "place", "data_type": "varchar"}},
        {{"name": "time", "data_type": "varchar"}},
        {{"name": "depth", "data_type": "double"}},
        {{"name": "latitude", "data_type": "double"}},
        {{"name": "longitude", "data_type": "double"}},
        {{"name": "type", "data_type": "varchar"}}
    ]
}}"#,
            current_time,
            query
        )
    }
    
    async fn call_openai(&self, api_key: &str, prompt: &str, debug: bool) -> Result<LLMResponse, Box<dyn std::error::Error>> {
        let client = OpenAIClient::with_config(
            async_openai::config::OpenAIConfig::default()
                .with_api_key(api_key)
        );
        
        // Add JSON instruction to prompt for OpenAI
        let json_prompt = format!("{}\n\nIMPORTANT: Return ONLY valid JSON matching the specified format.", prompt);
        
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o")
            .messages([
                ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content("You are a helpful assistant that generates Python code and data schemas. Always respond with valid JSON.")
                        .build()?
                ),
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(json_prompt)
                        .build()?
                ),
            ])
            .build()?;
        
        let response = client.chat().create(request).await?;
        let content = response.choices[0].message.content.as_ref()
            .ok_or("No content in response")?;
        
        if debug {
            eprintln!("Raw OpenAI response: {}", content);
        }
        
        // Strip markdown code blocks if present
        let cleaned_content = if content.starts_with("```json") && content.ends_with("```") {
            content.strip_prefix("```json").unwrap()
                .strip_suffix("```").unwrap()
                .trim()
        } else if content.starts_with("```") && content.ends_with("```") {
            content.strip_prefix("```").unwrap()
                .strip_suffix("```").unwrap()
                .trim()
        } else {
            content
        };
        
        serde_json::from_str(cleaned_content).map_err(|e| {
            if debug {
                eprintln!("JSON parse error: {}", e);
                eprintln!("Attempted to parse: {}", cleaned_content);
            }
            e.into()
        })
    }
    
    async fn call_anthropic(&self, api_key: &str, prompt: &str, _debug: bool) -> Result<LLMResponse, Box<dyn std::error::Error>> {
        let client = Anthropic::from_string_or_env(api_key)?;
        
        // Add explicit JSON instruction for Claude
        let json_prompt = format!("{}\n\nIMPORTANT: Return ONLY valid JSON, no additional text.", prompt);
        
        let mut request = MessagesRequest::default();
        request.model = "claude-3-haiku-20240307".to_string();
        request.messages = vec![
            Message {
                role: Role::User,
                content: vec![Content::text(json_prompt)],
            }
        ];
        request.max_tokens = 4096;
        
        let response = client.messages(&request).await?;
        
        // Extract text from response
        let text = response.content.iter()
            .filter_map(|content| {
                match content {
                    Content::Text(text_content) => Some(text_content.text.as_str()),
                    _ => None,
                }
            })
            .next()
            .ok_or("No text content in response")?;
        
        serde_json::from_str(text).map_err(|e| e.into())
    }
}