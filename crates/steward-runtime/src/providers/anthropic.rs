//! Anthropic Claude provider implementation.
//!
//! Supports Claude 4.5 family with prompt caching.

use super::{
    factory::ProviderFactory, ChatMessage, CompletionConfig, CompletionResponse, LlmProvider,
    ProviderError, TokenUsage,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use std::time::Duration;

/// Anthropic Claude provider.
pub struct AnthropicProvider {
    api_key: String,
    base_url: String,
    #[allow(dead_code)] // Reserved for future connection pooling
    client: Option<reqwest::Client>,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider.
    ///
    /// # Arguments
    /// * `api_key` - Anthropic API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            client: None,
        }
    }

    /// Create from environment variable.
    pub fn from_env() -> Result<Self, ProviderError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| ProviderError::NotConfigured("ANTHROPIC_API_KEY not set".to_string()))?;
        Ok(Self::new(api_key))
    }

    /// Set custom base URL.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    #[cfg(feature = "anthropic")]
    fn get_client(&self) -> &reqwest::Client {
        // Lazy initialization would go here
        static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
        CLIENT.get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client")
        })
    }
}

/// Anthropic API request format.
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<ContentBlock>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlock {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
}

#[derive(Debug, Serialize)]
struct CacheControl {
    #[serde(rename = "type")]
    type_: String,
}

/// Anthropic API response format.
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlockResponse>,
    model: String,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct ContentBlockResponse {
    #[serde(rename = "type")]
    #[allow(dead_code)] // Required for deserialization, not read directly
    type_: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
    #[serde(default)]
    cache_creation_input_tokens: u32,
    #[serde(default)]
    cache_read_input_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicError {
    error: AnthropicErrorDetail,
}

#[derive(Debug, Deserialize)]
struct AnthropicErrorDetail {
    #[serde(rename = "type")]
    #[allow(dead_code)] // Required for deserialization, not read directly
    type_: String,
    message: String,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    #[cfg(feature = "anthropic")]
    async fn complete(
        &self,
        messages: Vec<ChatMessage>,
        config: &CompletionConfig,
    ) -> Result<CompletionResponse, ProviderError> {
        let client = self.get_client();

        // Extract system message if present
        let (system_msg, user_messages): (Option<String>, Vec<ChatMessage>) = {
            let mut system = None;
            let mut others = Vec::new();

            for msg in messages {
                if msg.role == "system" {
                    system = Some(msg.content);
                } else {
                    others.push(msg);
                }
            }
            (system, others)
        };

        // Convert to Anthropic format
        let api_messages: Vec<AnthropicMessage> = user_messages
            .into_iter()
            .map(|msg| AnthropicMessage {
                role: msg.role,
                content: vec![ContentBlock::Text {
                    text: msg.content,
                    cache_control: if config.prompt_caching {
                        Some(CacheControl {
                            type_: "ephemeral".to_string(),
                        })
                    } else {
                        None
                    },
                }],
            })
            .collect();

        let request = AnthropicRequest {
            model: config.model.clone(),
            max_tokens: config.max_tokens,
            system: system_msg,
            messages: api_messages,
            temperature: if config.temperature == 0.0 {
                None
            } else {
                Some(config.temperature)
            },
        };

        let response = client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .timeout(config.timeout)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ProviderError::Timeout(config.timeout)
                } else {
                    ProviderError::HttpError(e.to_string())
                }
            })?;

        let status = response.status();

        if status == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .map(Duration::from_secs);
            return Err(ProviderError::RateLimited { retry_after });
        }

        if !status.is_success() {
            let error_body = response
                .json::<AnthropicError>()
                .await
                .map_err(|e| ProviderError::ParseError(e.to_string()))?;

            return Err(ProviderError::ApiError {
                status: status.as_u16(),
                message: error_body.error.message,
            });
        }

        let body: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        let content = body
            .content
            .into_iter()
            .filter_map(|block| block.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(CompletionResponse {
            content,
            usage: TokenUsage {
                prompt_tokens: body.usage.input_tokens,
                completion_tokens: body.usage.output_tokens,
                cache_read_tokens: body.usage.cache_read_input_tokens,
                cache_creation_tokens: body.usage.cache_creation_input_tokens,
            },
            model: body.model,
            stop_reason: body.stop_reason,
        })
    }

    #[cfg(not(feature = "anthropic"))]
    async fn complete(
        &self,
        _messages: Vec<ChatMessage>,
        _config: &CompletionConfig,
    ) -> Result<CompletionResponse, ProviderError> {
        Err(ProviderError::NotConfigured(
            "Anthropic provider requires 'anthropic' feature".to_string(),
        ))
    }

    async fn health_check(&self) -> bool {
        // Simple check - verify API key is set
        !self.api_key.is_empty()
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}

/// Factory for creating Anthropic providers from configuration.
///
/// ## Configuration Format
/// ```json
/// {
///   "api_key": "sk-ant-...",           // Optional, falls back to ANTHROPIC_API_KEY env
///   "base_url": "https://...",          // Optional, custom API endpoint
///   "model": "claude-sonnet-4-5-20250514",  // Optional, default model
///   "prompt_caching": true              // Optional, enable prompt caching
/// }
/// ```
pub struct AnthropicProviderFactory;

impl ProviderFactory for AnthropicProviderFactory {
    fn provider_type(&self) -> &'static str {
        "anthropic"
    }

    fn create(&self, config: &JsonValue) -> Result<Arc<dyn LlmProvider>, ProviderError> {
        // Get API key from config or environment
        let api_key = config["api_key"]
            .as_str()
            .map(String::from)
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| {
                ProviderError::NotConfigured(
                    "Anthropic API key required: set 'api_key' in config or ANTHROPIC_API_KEY env"
                        .to_string(),
                )
            })?;

        let mut provider = AnthropicProvider::new(api_key);

        // Apply optional base URL
        if let Some(base_url) = config["base_url"].as_str() {
            provider = provider.with_base_url(base_url);
        }

        Ok(Arc::new(provider))
    }

    fn validate_config(&self, config: &JsonValue) -> Result<(), ProviderError> {
        // API key is optional in config if env var is set
        let has_api_key = config["api_key"].as_str().is_some()
            || std::env::var("ANTHROPIC_API_KEY").is_ok();

        if !has_api_key {
            return Err(ProviderError::NotConfigured(
                "Anthropic API key required: set 'api_key' in config or ANTHROPIC_API_KEY env"
                    .to_string(),
            ));
        }

        // Validate base_url if present
        if let Some(url) = config["base_url"].as_str() {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(ProviderError::NotConfigured(
                    "base_url must start with http:// or https://".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn default_config(&self) -> JsonValue {
        serde_json::json!({
            "model": "claude-sonnet-4-5-20250514",
            "prompt_caching": true
        })
    }

    fn description(&self) -> &'static str {
        "Anthropic Claude provider with prompt caching support"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = AnthropicProvider::new("test-key");
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn test_token_estimation() {
        let provider = AnthropicProvider::new("test-key");
        let text = "Hello, world!"; // 13 chars
        let estimate = provider.estimate_tokens(text);
        assert!(estimate >= 2 && estimate <= 5);
    }

    #[test]
    fn test_factory_provider_type() {
        let factory = AnthropicProviderFactory;
        assert_eq!(factory.provider_type(), "anthropic");
    }

    #[test]
    fn test_factory_default_config() {
        let factory = AnthropicProviderFactory;
        let config = factory.default_config();
        assert_eq!(config["model"], "claude-sonnet-4-5-20250514");
        assert_eq!(config["prompt_caching"], true);
    }

    #[test]
    fn test_factory_description() {
        let factory = AnthropicProviderFactory;
        assert!(factory.description().contains("Anthropic"));
    }

    #[test]
    fn test_factory_create_with_api_key() {
        let factory = AnthropicProviderFactory;
        let config = serde_json::json!({
            "api_key": "test-api-key"
        });
        let provider = factory.create(&config);
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "anthropic");
    }

    #[test]
    fn test_factory_validate_invalid_base_url() {
        let factory = AnthropicProviderFactory;
        let config = serde_json::json!({
            "api_key": "test-key",
            "base_url": "invalid-url"
        });
        let result = factory.validate_config(&config);
        assert!(result.is_err());
    }
}
