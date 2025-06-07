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
    pub python_code: String,
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

Generate Python code that fetches this data and returns it as a list of dictionaries.
Also provide the schema of the data that will be returned.

IMPORTANT RULES:
1. The Python code should define a function called `fetch_data()` that returns a list of dictionaries
2. Each dictionary represents a row of data
3. For HTTP requests, use the built-in http_get(url) function (already available, no import needed)
4. CRITICAL: You CANNOT import these modules: json, datetime, random, math, urllib, requests
5. For JSON parsing, use eval() carefully: eval(response.replace('true','True').replace('false','False').replace('null','None'))
6. ALWAYS use REAL, FREE APIs. NEVER make up API endpoints. Examples of real free APIs:
   - Weather: wttr.in (e.g., http_get('https://wttr.in/Seattle?format=j1'))
   - Stock data: Yahoo Finance via yfinance API endpoints
   - Crypto: CoinGecko free tier (e.g., http_get('https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd'))
   - IP info: http_get('https://ipapi.co/json/')
   - Jokes: http_get('https://official-joke-api.appspot.com/random_joke')
7. If no suitable FREE API exists for the query, return an error:
   return [{{'error': 'No free API available', 'message': 'Cannot find a free API for: [query description]'}}]
8. If API call fails, return the error details:
   try:
       response = http_get(url)
       # process response
   except Exception as e:
       return [{{'error': 'API call failed', 'message': str(e), 'url': url}}]
9. NEVER return empty list [] - always return error details if something goes wrong
10. IMPORTANT: Include ALL relevant columns that make sense for the query:
    - For weather: include date, location, temperature, conditions, humidity, wind, etc.
    - For stocks: include date/time, symbol, open, high, low, close, volume
    - For crypto: include coin name, symbol, price, market cap, 24h change if available
    - Always prefer more complete data over minimal responses
11. The schema should match the data exactly and include all columns you return

Return your response as JSON in this exact format:
{{
    "python_code": "def fetch_data():\n    # Your code here\n    return data",
    "schema": [
        {{"name": "column1", "data_type": "varchar"}},
        {{"name": "column2", "data_type": "double"}}
    ]
}}

Supported data types: varchar, double, bigint, date

Example for "tech stock prices":
{{
    "python_code": "def fetch_data():\n    import yfinance as yf\n    import datetime\n    \n    end_date = datetime.datetime.now()\n    start_date = end_date - datetime.timedelta(days=7)\n    \n    tickers = ['AAPL', 'GOOGL', 'MSFT']\n    data = []\n    \n    for ticker in tickers:\n        stock = yf.Ticker(ticker)\n        hist = stock.history(start=start_date, end=end_date)\n        \n        for date, row in hist.iterrows():\n            data.append({{\n                'date': date.strftime('%Y-%m-%d'),\n                'ticker': ticker,\n                'open': float(row['Open']),\n                'high': float(row['High']),\n                'low': float(row['Low']),\n                'close': float(row['Close']),\n                'volume': int(row['Volume'])\n            }})\n    \n    return data",
    "schema": [
        {{"name": "date", "data_type": "varchar"}},
        {{"name": "ticker", "data_type": "varchar"}},
        {{"name": "open", "data_type": "double"}},
        {{"name": "high", "data_type": "double"}},
        {{"name": "low", "data_type": "double"}},
        {{"name": "close", "data_type": "double"}},
        {{"name": "volume", "data_type": "bigint"}}
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