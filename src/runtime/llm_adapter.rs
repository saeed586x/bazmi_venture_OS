//! LLM adapter for AI assistance

use serde::{Deserialize, Serialize};

/// LLM adapter for AI assistance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMAdapter {
    /// Configuration for the LLM adapter
    config: LLMConfig,
    #[serde(skip)]
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub model: String,
    pub api_key: Option<String>,
    pub temperature: f32,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenAI,
    Anthropic,
    Google,
    Custom(String),
}

impl LLMAdapter {
    /// Create a new LLM adapter
    pub fn new(config: LLMConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    /// Get the API endpoint URL based on provider
    fn get_api_url(&self) -> String {
        if let Some(custom_url) = &self.config.base_url {
            return custom_url.clone();
        }

        match self.config.provider {
            LLMProvider::OpenAI => "https://api.openai.com/v1/chat/completions".to_string(),
            LLMProvider::Anthropic => "https://api.anthropic.com/v1/messages".to_string(),
            LLMProvider::Google => {
                "https://generativelanguage.googleapis.com/v1beta/models".to_string()
            }
            LLMProvider::Custom(_) => "http://localhost:8000/v1/chat/completions".to_string(),
        }
    }

    /// Build request body based on provider
    fn build_request_body(&self, prompt: &str) -> Result<serde_json::Value, LLMError> {
        match self.config.provider {
            LLMProvider::OpenAI | LLMProvider::Custom(_) => Ok(serde_json::json!({
                "model": self.config.model,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }],
                "temperature": self.config.temperature
            })),
            LLMProvider::Anthropic => Ok(serde_json::json!({
                "model": self.config.model,
                "max_tokens": 4096,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }]
            })),
            LLMProvider::Google => Ok(serde_json::json!({
                "contents": [{
                    "parts": [{
                        "text": prompt
                    }]
                }],
                "generationConfig": {
                    "temperature": self.config.temperature
                }
            })),
        }
    }

    /// Parse response based on provider
    fn parse_response(&self, response: serde_json::Value) -> Result<String, LLMError> {
        match self.config.provider {
            LLMProvider::OpenAI | LLMProvider::Custom(_) => response
                .pointer("/choices/0/message/content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| LLMError::ApiError("Invalid response format".to_string())),
            LLMProvider::Anthropic => response
                .pointer("/content/0/text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| LLMError::ApiError("Invalid response format".to_string())),
            LLMProvider::Google => response
                .pointer("/candidates/0/content/parts/0/text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| LLMError::ApiError("Invalid response format".to_string())),
        }
    }

    /// Generate text using the LLM
    pub async fn generate_text(&self, prompt: &str) -> Result<String, LLMError> {
        let api_url = self.get_api_url();
        let request_body = self.build_request_body(prompt)?;

        let mut request = self
            .http_client
            .post(&api_url)
            .header("Content-Type", "application/json");

        // Add provider-specific headers
        if let Some(api_key) = &self.config.api_key {
            match self.config.provider {
                LLMProvider::OpenAI | LLMProvider::Custom(_) => {
                    request = request.header("Authorization", format!("Bearer {}", api_key));
                }
                LLMProvider::Anthropic => {
                    request = request
                        .header("x-api-key", api_key)
                        .header("anthropic-version", "2023-06-01");
                }
                LLMProvider::Google => {
                    // Google uses query parameter for API key
                }
            }
        }

        let response = request
            .json(&request_body)
            .send()
            .await
            .map_err(|e| LLMError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::ApiError(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LLMError::ApiError(format!("JSON parse failed: {}", e)))?;
        self.parse_response(response_json)
    }

    /// Process structured data with the LLM
    pub async fn process_structured<T>(
        &self,
        data: &T,
        instruction: &str,
    ) -> Result<ProcessedData, LLMError>
    where
        T: serde::Serialize,
    {
        let data_json = serde_json::to_string(data).map_err(LLMError::SerializationError)?;
        let prompt = format!("{}\n\nData:\n{}", instruction, data_json);

        let content = self.generate_text(&prompt).await?;

        Ok(ProcessedData {
            original_size: data_json.len(),
            processed_content: content,
            confidence: 0.9,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedData {
    pub original_size: usize,
    pub processed_content: String,
    pub confidence: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
}
