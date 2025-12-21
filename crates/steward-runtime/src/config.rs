//! Configuration for steward-runtime.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;
use steward_core::LensType;

use crate::resilience::{CircuitBreakerConfig, FallbackStrategy};
use serde_json::Value as JsonValue;

/// Runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Whether runtime is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Default LLM provider
    #[serde(default = "default_provider")]
    pub default_provider: String,

    /// Timeout configuration
    #[serde(default)]
    pub timeouts: TimeoutConfig,

    /// Token budget configuration
    #[serde(default)]
    pub budgets: BudgetConfig,

    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,

    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,

    /// Early termination configuration
    #[serde(default)]
    pub early_termination: EarlyTerminationConfig,

    /// Prompt caching configuration
    #[serde(default)]
    pub prompt_caching: PromptCachingConfig,

    /// Verification pass configuration
    #[serde(default)]
    pub verification: VerificationConfig,

    /// Provider configurations (BTreeMap for deterministic iteration)
    #[serde(default)]
    pub providers: BTreeMap<String, ProviderConfig>,

    /// Fallback chain
    #[serde(default)]
    pub fallback: Vec<FallbackStrategy>,
}

fn default_true() -> bool {
    true
}

fn default_provider() -> String {
    "anthropic".to_string()
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_provider: "anthropic".to_string(),
            timeouts: TimeoutConfig::default(),
            budgets: BudgetConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            retry: RetryConfig::default(),
            cache: CacheConfig::default(),
            early_termination: EarlyTerminationConfig::default(),
            prompt_caching: PromptCachingConfig::default(),
            verification: VerificationConfig::default(),
            providers: BTreeMap::new(),
            fallback: vec![
                FallbackStrategy::Cache,
                FallbackStrategy::SimplerModel {
                    model: "claude-haiku-4-5".to_string(),
                },
                FallbackStrategy::Deterministic,
                FallbackStrategy::EscalateWithUncertainty,
            ],
        }
    }
}

impl RuntimeConfig {
    /// Get timeout for a specific lens.
    pub fn lens_timeout(&self, lens: LensType) -> Duration {
        self.timeouts
            .per_lens
            .get(&lens)
            .copied()
            .unwrap_or(self.timeouts.lens_default)
    }

    /// Get token budget for a specific lens.
    pub fn lens_budget(&self, lens: LensType) -> u32 {
        self.budgets
            .per_lens
            .get(&lens)
            .copied()
            .unwrap_or(1000)
    }
}

/// Timeout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Global evaluation timeout
    #[serde(with = "humantime_serde", default = "default_global_timeout")]
    pub global: Duration,

    /// Default timeout for lens evaluation
    #[serde(with = "humantime_serde", default = "default_lens_timeout")]
    pub lens_default: Duration,

    /// LLM call timeout
    #[serde(with = "humantime_serde", default = "default_llm_timeout")]
    pub llm_call: Duration,

    /// Per-lens timeout overrides (BTreeMap for deterministic iteration)
    #[serde(default)]
    pub per_lens: BTreeMap<LensType, Duration>,
}

fn default_global_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_lens_timeout() -> Duration {
    Duration::from_secs(10)
}

fn default_llm_timeout() -> Duration {
    Duration::from_secs(15)
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            global: Duration::from_secs(30),
            lens_default: Duration::from_secs(10),
            llm_call: Duration::from_secs(15),
            per_lens: BTreeMap::new(),
        }
    }
}

/// Token budget configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Global maximum tokens
    #[serde(default = "default_global_budget")]
    pub global_max_tokens: u32,

    /// Per-lens budgets (BTreeMap for deterministic iteration)
    #[serde(default)]
    pub per_lens: BTreeMap<LensType, u32>,
}

fn default_global_budget() -> u32 {
    5000
}

impl Default for BudgetConfig {
    fn default() -> Self {
        let mut per_lens = BTreeMap::new();
        // Alphabetical order to match LensType's Ord implementation
        per_lens.insert(LensType::AccountabilityOwnership, 500);
        per_lens.insert(LensType::BoundariesSafety, 1000);
        per_lens.insert(LensType::DignityInclusion, 1000);
        per_lens.insert(LensType::RestraintPrivacy, 500);
        per_lens.insert(LensType::TransparencyContestability, 1500);

        Self {
            global_max_tokens: 5000,
            per_lens,
        }
    }
}

/// Retry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Initial backoff
    #[serde(with = "humantime_serde", default = "default_initial_backoff")]
    pub initial_backoff: Duration,

    /// Maximum backoff
    #[serde(with = "humantime_serde", default = "default_max_backoff")]
    pub max_backoff: Duration,

    /// Backoff multiplier
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
}

fn default_max_retries() -> u32 {
    2
}

fn default_initial_backoff() -> Duration {
    Duration::from_millis(100)
}

fn default_max_backoff() -> Duration {
    Duration::from_secs(2)
}

fn default_multiplier() -> f64 {
    2.0
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 2,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(2),
            multiplier: 2.0,
        }
    }
}

/// Cache configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Cache TTL
    #[serde(with = "humantime_serde", default = "default_cache_ttl")]
    pub ttl: Duration,

    /// Maximum cache entries
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
}

fn default_cache_ttl() -> Duration {
    Duration::from_secs(3600)
}

fn default_max_entries() -> usize {
    10000
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl: Duration::from_secs(3600),
            max_entries: 10000,
        }
    }
}

/// Early termination configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarlyTerminationConfig {
    /// Enable early termination on BLOCKED
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Complete remaining lenses with fallback
    #[serde(default = "default_true")]
    pub complete_remaining: bool,
}

impl Default for EarlyTerminationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            complete_remaining: true,
        }
    }
}

/// Prompt caching configuration (Anthropic-specific).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCachingConfig {
    /// Enable prompt caching
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Minimum tokens to enable caching
    #[serde(default = "default_min_tokens")]
    pub min_tokens: u32,

    /// Cache type: "ephemeral" or "persistent"
    #[serde(default = "default_cache_type")]
    pub cache_type: String,
}

fn default_min_tokens() -> u32 {
    1024
}

fn default_cache_type() -> String {
    "ephemeral".to_string()
}

impl Default for PromptCachingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_tokens: 1024,
            cache_type: "ephemeral".to_string(),
        }
    }
}

/// Verification pass configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    /// Enable verification pass
    #[serde(default)]
    pub enabled: bool,

    /// Confidence threshold for verification
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f64,

    /// Model for verification
    #[serde(default = "default_verification_model")]
    pub verification_model: String,

    /// Maximum lenses to verify
    #[serde(default = "default_max_verify")]
    pub max_lenses_to_verify: usize,

    /// Disagreement strategy
    #[serde(default)]
    pub disagreement_strategy: DisagreementStrategy,

    /// Only verify on ESCALATE
    #[serde(default = "default_true")]
    pub only_on_escalate: bool,
}

fn default_confidence_threshold() -> f64 {
    0.6
}

fn default_verification_model() -> String {
    "claude-opus-4-5".to_string()
}

fn default_max_verify() -> usize {
    2
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            confidence_threshold: 0.6,
            verification_model: "claude-opus-4-5".to_string(),
            max_lenses_to_verify: 2,
            disagreement_strategy: DisagreementStrategy::FlagForHumanReview,
            only_on_escalate: true,
        }
    }
}

/// Strategy when verification disagrees with initial result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisagreementStrategy {
    /// Trust the verification result
    TrustVerification,

    /// Flag for human review
    #[default]
    FlagForHumanReview,

    /// Use weighted average of confidences
    WeightedAverage { verification_weight: f64 },

    /// Always escalate on disagreement
    EscalateOnDisagreement,
}

/// Provider configuration.
///
/// Uses JSON value for provider-specific settings, validated by ProviderFactory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type identifier (e.g., "anthropic", "openai", "local")
    #[serde(rename = "type")]
    pub provider_type: String,

    /// Provider-specific configuration as JSON
    /// Validated by the corresponding ProviderFactory
    #[serde(flatten)]
    pub settings: JsonValue,
}

// Custom serialization for Duration using humantime format
mod humantime_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&humantime::format_duration(*duration).to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        humantime::parse_duration(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RuntimeConfig::default();
        assert!(config.enabled);
        assert_eq!(config.default_provider, "anthropic");
        assert_eq!(config.budgets.global_max_tokens, 5000);
    }

    #[test]
    fn test_lens_timeout() {
        let mut config = RuntimeConfig::default();
        config.timeouts.per_lens.insert(
            LensType::TransparencyContestability,
            Duration::from_secs(15),
        );

        // Custom timeout
        assert_eq!(
            config.lens_timeout(LensType::TransparencyContestability),
            Duration::from_secs(15)
        );

        // Default timeout
        assert_eq!(
            config.lens_timeout(LensType::DignityInclusion),
            Duration::from_secs(10)
        );
    }
}
