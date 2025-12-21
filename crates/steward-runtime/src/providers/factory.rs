//! Provider factory pattern for dynamic LLM provider registration.
//!
//! This module enables adding new LLM providers without modifying enums.
//! Providers register factories that create instances from configuration.
//!
//! ## Usage
//!
//! ```ignore
//! let mut registry = ProviderRegistry::new();
//! registry.register(Arc::new(AnthropicProviderFactory));
//!
//! let provider = registry.create("anthropic", &config)?;
//! ```

use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value as JsonValue;

use super::{LlmProvider, ProviderError};

/// Factory for creating LLM providers from configuration.
///
/// Implement this trait to add a new provider type without modifying
/// the ProviderType enum. Each factory is responsible for:
/// 1. Validating its configuration format
/// 2. Creating provider instances
/// 3. Providing a unique type identifier
pub trait ProviderFactory: Send + Sync {
    /// Unique identifier for this provider type.
    ///
    /// Examples: "anthropic", "openai", "local", "azure-openai"
    fn provider_type(&self) -> &'static str;

    /// Create a provider instance from JSON configuration.
    ///
    /// # Arguments
    /// * `config` - Provider-specific configuration as JSON
    ///
    /// # Returns
    /// A configured provider instance or an error
    fn create(&self, config: &JsonValue) -> Result<Arc<dyn LlmProvider>, ProviderError>;

    /// Validate configuration without creating a provider.
    ///
    /// Use this for fast config validation during startup.
    fn validate_config(&self, config: &JsonValue) -> Result<(), ProviderError>;

    /// Get default configuration for this provider.
    ///
    /// Returns sensible defaults for optional fields.
    fn default_config(&self) -> JsonValue {
        serde_json::json!({})
    }

    /// Human-readable description of this provider.
    fn description(&self) -> &'static str {
        "LLM Provider"
    }
}

/// Registry of available provider factories.
///
/// The registry maintains a mapping of provider type names to their factories.
/// Use this to dynamically create providers from configuration.
#[derive(Default)]
pub struct ProviderRegistry {
    factories: BTreeMap<String, Arc<dyn ProviderFactory>>,
}

impl ProviderRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a provider factory.
    ///
    /// If a factory with the same type already exists, it will be replaced.
    pub fn register(&mut self, factory: Arc<dyn ProviderFactory>) {
        self.factories
            .insert(factory.provider_type().to_string(), factory);
    }

    /// Create a provider from type name and configuration.
    ///
    /// # Arguments
    /// * `provider_type` - The provider type (e.g., "anthropic", "openai")
    /// * `config` - Provider-specific configuration as JSON
    ///
    /// # Returns
    /// A configured provider instance or an error
    pub fn create(
        &self,
        provider_type: &str,
        config: &JsonValue,
    ) -> Result<Arc<dyn LlmProvider>, ProviderError> {
        self.factories
            .get(provider_type)
            .ok_or_else(|| ProviderError::NotConfigured(format!(
                "Unknown provider type: '{}'. Available: {:?}",
                provider_type,
                self.available_types()
            )))?
            .create(config)
    }

    /// Validate configuration for a provider type.
    pub fn validate(
        &self,
        provider_type: &str,
        config: &JsonValue,
    ) -> Result<(), ProviderError> {
        self.factories
            .get(provider_type)
            .ok_or_else(|| ProviderError::NotConfigured(format!(
                "Unknown provider type: '{}'",
                provider_type
            )))?
            .validate_config(config)
    }

    /// List available provider types.
    pub fn available_types(&self) -> Vec<&str> {
        self.factories.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a provider type is registered.
    pub fn has_provider(&self, provider_type: &str) -> bool {
        self.factories.contains_key(provider_type)
    }

    /// Get the factory for a provider type.
    pub fn get_factory(&self, provider_type: &str) -> Option<&Arc<dyn ProviderFactory>> {
        self.factories.get(provider_type)
    }

    /// Get default configuration for a provider type.
    pub fn default_config(&self, provider_type: &str) -> Option<JsonValue> {
        self.factories
            .get(provider_type)
            .map(|f| f.default_config())
    }

    /// Create a registry with all built-in providers registered.
    #[cfg(feature = "anthropic")]
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Arc::new(super::AnthropicProviderFactory));
        registry
    }

    /// Create a registry with all built-in providers registered.
    #[cfg(not(feature = "anthropic"))]
    pub fn with_defaults() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ProviderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderRegistry")
            .field("providers", &self.available_types())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::providers::{ChatMessage, CompletionConfig, CompletionResponse, TokenUsage};

    // Mock provider for testing
    struct MockProvider {
        name: String,
    }

    #[async_trait]
    impl LlmProvider for MockProvider {
        async fn complete(
            &self,
            _messages: Vec<ChatMessage>,
            _config: &CompletionConfig,
        ) -> Result<CompletionResponse, ProviderError> {
            Ok(CompletionResponse {
                content: "mock response".to_string(),
                usage: TokenUsage::default(),
                model: "mock".to_string(),
                stop_reason: Some("end_turn".to_string()),
            })
        }

        async fn health_check(&self) -> bool {
            true
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    // Mock factory for testing
    struct MockProviderFactory;

    impl ProviderFactory for MockProviderFactory {
        fn provider_type(&self) -> &'static str {
            "mock"
        }

        fn create(&self, config: &JsonValue) -> Result<Arc<dyn LlmProvider>, ProviderError> {
            let name = config["name"]
                .as_str()
                .unwrap_or("mock-provider")
                .to_string();
            Ok(Arc::new(MockProvider { name }))
        }

        fn validate_config(&self, _config: &JsonValue) -> Result<(), ProviderError> {
            Ok(())
        }

        fn description(&self) -> &'static str {
            "Mock provider for testing"
        }
    }

    #[test]
    fn test_registry_register_and_create() {
        let mut registry = ProviderRegistry::new();
        registry.register(Arc::new(MockProviderFactory));

        assert!(registry.has_provider("mock"));
        assert!(!registry.has_provider("unknown"));

        let config = serde_json::json!({"name": "test-mock"});
        let provider = registry.create("mock", &config);
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "test-mock");
    }

    #[test]
    fn test_registry_unknown_provider() {
        let registry = ProviderRegistry::new();
        let config = serde_json::json!({});

        let result = registry.create("unknown", &config);
        assert!(result.is_err());

        match result {
            Err(ProviderError::NotConfigured(msg)) => {
                assert!(msg.contains("Unknown provider type"));
            }
            _ => panic!("Expected NotConfigured error"),
        }
    }

    #[test]
    fn test_registry_available_types() {
        let mut registry = ProviderRegistry::new();
        assert!(registry.available_types().is_empty());

        registry.register(Arc::new(MockProviderFactory));
        assert_eq!(registry.available_types(), vec!["mock"]);
    }

    #[test]
    fn test_registry_validate() {
        let mut registry = ProviderRegistry::new();
        registry.register(Arc::new(MockProviderFactory));

        let config = serde_json::json!({});
        assert!(registry.validate("mock", &config).is_ok());
        assert!(registry.validate("unknown", &config).is_err());
    }
}
