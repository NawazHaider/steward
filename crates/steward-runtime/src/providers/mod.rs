//! LLM Provider abstractions for steward-runtime.
//!
//! This module defines the trait for LLM providers and includes
//! implementations for Anthropic, OpenAI, and local models.
//!
//! ## Security
//!
//! All providers use the [`secrets`] module for secure credential handling.
//! See [`ApiCredential`] for the recommended patterns.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

mod factory;
pub mod secrets;

#[cfg(feature = "anthropic")]
mod anthropic;

pub use factory::{ProviderFactory, ProviderRegistry};
pub use secrets::{ApiCredential, CredentialBuilder, CredentialSet, CredentialSource};

#[cfg(feature = "anthropic")]
pub use anthropic::{AnthropicProvider, AnthropicProviderFactory};

/// Errors from LLM providers.
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("Rate limit exceeded, retry after {retry_after:?}")]
    RateLimited { retry_after: Option<Duration> },

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("JSON parse error: {0}")]
    ParseError(String),

    #[error("Authentication failed")]
    AuthError,

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Provider not configured: {0}")]
    NotConfigured(String),
}

/// Configuration for a completion request.
#[derive(Debug, Clone)]
pub struct CompletionConfig {
    /// Model to use
    pub model: String,

    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// Temperature (0.0 for deterministic)
    pub temperature: f32,

    /// Request timeout
    pub timeout: Duration,

    /// Enable prompt caching (Anthropic-specific)
    pub prompt_caching: bool,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-5-20250514".to_string(),
            max_tokens: 500,
            temperature: 0.0,
            timeout: Duration::from_secs(15),
            prompt_caching: true,
        }
    }
}

/// A chat message for LLM completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role: "system", "user", or "assistant"
    pub role: String,

    /// Message content
    pub content: String,
}

impl ChatMessage {
    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// Response from an LLM completion.
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    /// Generated content
    pub content: String,

    /// Token usage
    pub usage: TokenUsage,

    /// Model used
    pub model: String,

    /// Stop reason
    pub stop_reason: Option<String>,
}

/// Token usage from a completion.
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    /// Tokens in the prompt
    pub prompt_tokens: u32,

    /// Tokens in the completion
    pub completion_tokens: u32,

    /// Tokens read from cache (Anthropic)
    pub cache_read_tokens: u32,

    /// Tokens written to cache (Anthropic)
    pub cache_creation_tokens: u32,
}

impl TokenUsage {
    /// Total tokens used.
    pub fn total(&self) -> u32 {
        self.prompt_tokens + self.completion_tokens
    }
}

/// Provider abstraction allows swapping LLM backends.
///
/// # Blueprint Constraint
/// This is the ONLY place where LLM calls are made.
/// The Synthesizer NEVER calls this - only lens agents do.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Execute a chat completion.
    async fn complete(
        &self,
        messages: Vec<ChatMessage>,
        config: &CompletionConfig,
    ) -> Result<CompletionResponse, ProviderError>;

    /// Check if provider is healthy.
    async fn health_check(&self) -> bool;

    /// Get provider name for metrics.
    fn name(&self) -> &str;

    /// Estimate tokens for a prompt.
    fn estimate_tokens(&self, text: &str) -> u32 {
        // Simple estimate: ~4 chars per token
        (text.len() / 4) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_creation() {
        let system = ChatMessage::system("You are a helpful assistant.");
        assert_eq!(system.role, "system");

        let user = ChatMessage::user("Hello!");
        assert_eq!(user.role, "user");

        let assistant = ChatMessage::assistant("Hi there!");
        assert_eq!(assistant.role, "assistant");
    }

    #[test]
    fn test_token_usage_total() {
        let usage = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
        };
        assert_eq!(usage.total(), 150);
    }
}
