//! Secure credential handling for LLM providers.
//!
//! This module provides a centralized, type-safe way to handle API credentials
//! across all providers. Using this module ensures:
//!
//! - **No accidental logging**: Credentials cannot appear in Debug/Display output
//! - **Memory safety**: Credentials are zeroed on drop (defense in depth)
//! - **Consistent patterns**: All providers use the same secure handling
//! - **Compile-time safety**: Cannot accidentally pass credentials to format!
//!
//! ## Usage
//!
//! ```ignore
//! use crate::providers::secrets::{ApiCredential, CredentialSource};
//!
//! // Load from environment
//! let cred = ApiCredential::from_env("ANTHROPIC_API_KEY")?;
//!
//! // Load from config with env fallback
//! let cred = ApiCredential::from_config_or_env(&config, "api_key", "ANTHROPIC_API_KEY")?;
//!
//! // Use in HTTP header (explicit exposure)
//! request.header("x-api-key", cred.expose());
//! ```

use secrecy::{ExposeSecret, SecretString};
use serde_json::Value as JsonValue;
use std::fmt;

use super::ProviderError;

/// Where a credential was loaded from.
///
/// This is useful for debugging configuration issues without
/// exposing the actual credential value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialSource {
    /// Loaded from configuration file/JSON
    Config,
    /// Loaded from environment variable
    Environment,
    /// Provided programmatically
    Programmatic,
}

impl fmt::Display for CredentialSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialSource::Config => write!(f, "config"),
            CredentialSource::Environment => write!(f, "environment"),
            CredentialSource::Programmatic => write!(f, "programmatic"),
        }
    }
}

/// A securely-stored API credential.
///
/// This wrapper provides:
/// - Safe Debug implementation that shows `[REDACTED]`
/// - Memory zeroing on drop via `secrecy` crate
/// - Explicit exposure via `.expose()` method
/// - Source tracking for debugging
///
/// # Example
///
/// ```ignore
/// let cred = ApiCredential::new("sk-secret-key", CredentialSource::Environment);
///
/// // Safe to log/debug - shows [REDACTED]
/// println!("Provider credential: {:?}", cred);
///
/// // Explicit exposure for API calls
/// let key = cred.expose();
/// ```
pub struct ApiCredential {
    value: SecretString,
    source: CredentialSource,
    name: &'static str,
}

impl ApiCredential {
    /// Create a new credential from a string value.
    ///
    /// The value is immediately wrapped in SecretString and cannot
    /// be accidentally logged after this point.
    pub fn new(
        value: impl Into<String>,
        source: CredentialSource,
        name: &'static str,
    ) -> Self {
        Self {
            value: SecretString::from(value.into()),
            source,
            name,
        }
    }

    /// Load credential from an environment variable.
    ///
    /// # Arguments
    /// * `env_var` - Name of the environment variable
    /// * `name` - Human-readable name for error messages (e.g., "Anthropic API key")
    ///
    /// # Example
    ///
    /// ```ignore
    /// let cred = ApiCredential::from_env("ANTHROPIC_API_KEY", "Anthropic API key")?;
    /// ```
    pub fn from_env(env_var: &str, name: &'static str) -> Result<Self, ProviderError> {
        std::env::var(env_var)
            .map(|v| Self::new(v, CredentialSource::Environment, name))
            .map_err(|_| {
                ProviderError::NotConfigured(format!(
                    "{} not set: configure '{}' environment variable",
                    name, env_var
                ))
            })
    }

    /// Load credential from JSON config, falling back to environment variable.
    ///
    /// This is the recommended way to load credentials in provider factories:
    /// 1. Check if `config_key` exists in the JSON config
    /// 2. If not, fall back to `env_var` environment variable
    /// 3. Return error if neither is set
    ///
    /// # Arguments
    /// * `config` - JSON configuration object
    /// * `config_key` - Key to look for in config (e.g., "api_key")
    /// * `env_var` - Fallback environment variable (e.g., "ANTHROPIC_API_KEY")
    /// * `name` - Human-readable name for error messages
    ///
    /// # Example
    ///
    /// ```ignore
    /// let cred = ApiCredential::from_config_or_env(
    ///     &config,
    ///     "api_key",
    ///     "ANTHROPIC_API_KEY",
    ///     "Anthropic API key"
    /// )?;
    /// ```
    pub fn from_config_or_env(
        config: &JsonValue,
        config_key: &str,
        env_var: &str,
        name: &'static str,
    ) -> Result<Self, ProviderError> {
        // Try config first
        if let Some(value) = config[config_key].as_str() {
            return Ok(Self::new(value, CredentialSource::Config, name));
        }

        // Fall back to environment
        if let Ok(value) = std::env::var(env_var) {
            return Ok(Self::new(value, CredentialSource::Environment, name));
        }

        Err(ProviderError::NotConfigured(format!(
            "{} required: set '{}' in config or {} environment variable",
            name, config_key, env_var
        )))
    }

    /// Check if a credential is available (without loading it).
    ///
    /// Useful for validation without creating the credential.
    pub fn is_available(config: &JsonValue, config_key: &str, env_var: &str) -> bool {
        config[config_key].as_str().is_some() || std::env::var(env_var).is_ok()
    }

    /// Expose the credential value for use in API calls.
    ///
    /// # Security
    ///
    /// Only call this at the point where the credential is actually needed
    /// (e.g., setting an HTTP header). Never store the exposed value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // GOOD: Expose at point of use
    /// request.header("x-api-key", cred.expose());
    ///
    /// // BAD: Don't store the exposed value
    /// let key = cred.expose().to_string(); // Don't do this!
    /// ```
    pub fn expose(&self) -> &str {
        self.value.expose_secret()
    }

    /// Check if the credential is empty.
    pub fn is_empty(&self) -> bool {
        self.value.expose_secret().is_empty()
    }

    /// Get the source of this credential.
    pub fn source(&self) -> CredentialSource {
        self.source
    }

    /// Get the human-readable name of this credential.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl fmt::Debug for ApiCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiCredential")
            .field("value", &"[REDACTED]")
            .field("source", &self.source)
            .field("name", &self.name)
            .finish()
    }
}

impl fmt::Display for ApiCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} from {} [REDACTED]", self.name, self.source)
    }
}

/// Builder for providers that need multiple credentials.
///
/// Some providers (like Azure OpenAI) need both an API key and endpoint.
/// This builder provides a fluent interface for loading multiple credentials.
///
/// # Example
///
/// ```ignore
/// let creds = CredentialBuilder::new()
///     .require("api_key", "AZURE_API_KEY", "Azure API key")
///     .optional("endpoint", "AZURE_ENDPOINT", "Azure endpoint")
///     .build(&config)?;
///
/// let api_key = creds.get("api_key")?;
/// let endpoint = creds.get_optional("endpoint");
/// ```
pub struct CredentialBuilder {
    required: Vec<CredentialSpec>,
    optional: Vec<CredentialSpec>,
}

struct CredentialSpec {
    config_key: &'static str,
    env_var: &'static str,
    name: &'static str,
}

impl CredentialBuilder {
    /// Create a new credential builder.
    pub fn new() -> Self {
        Self {
            required: Vec::new(),
            optional: Vec::new(),
        }
    }

    /// Add a required credential.
    pub fn require(
        mut self,
        config_key: &'static str,
        env_var: &'static str,
        name: &'static str,
    ) -> Self {
        self.required.push(CredentialSpec {
            config_key,
            env_var,
            name,
        });
        self
    }

    /// Add an optional credential.
    pub fn optional(
        mut self,
        config_key: &'static str,
        env_var: &'static str,
        name: &'static str,
    ) -> Self {
        self.optional.push(CredentialSpec {
            config_key,
            env_var,
            name,
        });
        self
    }

    /// Build the credential set from config.
    pub fn build(self, config: &JsonValue) -> Result<CredentialSet, ProviderError> {
        let mut credentials = std::collections::BTreeMap::new();

        // Load required credentials
        for spec in self.required {
            let cred =
                ApiCredential::from_config_or_env(config, spec.config_key, spec.env_var, spec.name)?;
            credentials.insert(spec.config_key, cred);
        }

        // Load optional credentials
        for spec in self.optional {
            if ApiCredential::is_available(config, spec.config_key, spec.env_var) {
                let cred = ApiCredential::from_config_or_env(
                    config,
                    spec.config_key,
                    spec.env_var,
                    spec.name,
                )?;
                credentials.insert(spec.config_key, cred);
            }
        }

        Ok(CredentialSet { credentials })
    }
}

impl Default for CredentialBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A set of loaded credentials.
pub struct CredentialSet {
    credentials: std::collections::BTreeMap<&'static str, ApiCredential>,
}

impl CredentialSet {
    /// Get a required credential by key.
    pub fn get(&self, key: &str) -> Result<&ApiCredential, ProviderError> {
        self.credentials.get(key).ok_or_else(|| {
            ProviderError::NotConfigured(format!("Credential '{}' not found", key))
        })
    }

    /// Get an optional credential by key.
    pub fn get_optional(&self, key: &str) -> Option<&ApiCredential> {
        self.credentials.get(key)
    }

    /// Check if a credential exists.
    pub fn has(&self, key: &str) -> bool {
        self.credentials.contains_key(key)
    }
}

impl fmt::Debug for CredentialSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CredentialSet")
            .field("keys", &self.credentials.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_redacted_in_debug() {
        let secret = "sk-super-secret-key-12345";
        let cred = ApiCredential::new(secret, CredentialSource::Programmatic, "Test API key");

        let debug = format!("{:?}", cred);
        assert!(!debug.contains(secret), "Secret exposed in Debug!");
        assert!(debug.contains("[REDACTED]"));
    }

    #[test]
    fn test_credential_redacted_in_display() {
        let secret = "sk-super-secret-key-12345";
        let cred = ApiCredential::new(secret, CredentialSource::Config, "Test API key");

        let display = format!("{}", cred);
        assert!(!display.contains(secret), "Secret exposed in Display!");
        assert!(display.contains("[REDACTED]"));
        assert!(display.contains("Test API key"));
        assert!(display.contains("config"));
    }

    #[test]
    fn test_credential_expose() {
        let secret = "sk-super-secret-key-12345";
        let cred = ApiCredential::new(secret, CredentialSource::Programmatic, "Test API key");

        assert_eq!(cred.expose(), secret);
    }

    #[test]
    fn test_credential_source_tracking() {
        let cred = ApiCredential::new("key", CredentialSource::Environment, "Test");
        assert_eq!(cred.source(), CredentialSource::Environment);
    }

    #[test]
    fn test_from_config_or_env_prefers_config() {
        let config = serde_json::json!({
            "api_key": "config-key"
        });

        // Even if env var exists, config takes precedence
        std::env::set_var("TEST_API_KEY_PRIORITY", "env-key");
        let cred = ApiCredential::from_config_or_env(
            &config,
            "api_key",
            "TEST_API_KEY_PRIORITY",
            "Test key",
        )
        .unwrap();

        assert_eq!(cred.expose(), "config-key");
        assert_eq!(cred.source(), CredentialSource::Config);

        std::env::remove_var("TEST_API_KEY_PRIORITY");
    }

    #[test]
    fn test_from_config_or_env_falls_back_to_env() {
        let config = serde_json::json!({});

        std::env::set_var("TEST_API_KEY_FALLBACK", "env-key");
        let cred = ApiCredential::from_config_or_env(
            &config,
            "api_key",
            "TEST_API_KEY_FALLBACK",
            "Test key",
        )
        .unwrap();

        assert_eq!(cred.expose(), "env-key");
        assert_eq!(cred.source(), CredentialSource::Environment);

        std::env::remove_var("TEST_API_KEY_FALLBACK");
    }

    #[test]
    fn test_from_config_or_env_error_when_missing() {
        let config = serde_json::json!({});

        let result = ApiCredential::from_config_or_env(
            &config,
            "api_key",
            "NONEXISTENT_VAR_12345",
            "Test key",
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Test key"));
        assert!(err.to_string().contains("api_key"));
        assert!(err.to_string().contains("NONEXISTENT_VAR_12345"));
    }

    #[test]
    fn test_is_available() {
        let config = serde_json::json!({
            "api_key": "value"
        });

        assert!(ApiCredential::is_available(&config, "api_key", "NONEXISTENT"));
        assert!(!ApiCredential::is_available(
            &serde_json::json!({}),
            "api_key",
            "NONEXISTENT"
        ));
    }

    #[test]
    fn test_credential_builder() {
        let config = serde_json::json!({
            "api_key": "my-key"
        });

        let creds = CredentialBuilder::new()
            .require("api_key", "TEST_KEY", "API key")
            .optional("endpoint", "TEST_ENDPOINT", "Endpoint")
            .build(&config)
            .unwrap();

        assert!(creds.has("api_key"));
        assert!(!creds.has("endpoint"));
        assert_eq!(creds.get("api_key").unwrap().expose(), "my-key");
    }
}
