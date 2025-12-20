//! # steward-runtime
//!
//! Optional LLM-assisted evaluation for Steward.
//!
//! This crate provides LLM-based evaluation for rules that require
//! interpretation beyond deterministic pattern matching.
//!
//! ## Important
//!
//! This crate is OPTIONAL. The core evaluation in `steward-core` is
//! fully deterministic and never makes LLM calls.
//!
//! Use this crate when:
//! - Rules are ambiguous and need semantic understanding
//! - Pattern matching produces too many false positives/negatives
//! - You need natural language interpretation of rules
//!
//! ## Example
//!
//! ```rust,ignore
//! use steward_runtime::{LlmEvaluator, LlmConfig};
//!
//! let config = LlmConfig::new("claude-3-5-sonnet");
//! let evaluator = LlmEvaluator::new(config)?;
//!
//! // Evaluate a rule that needs interpretation
//! let result = evaluator.evaluate_rule(
//!     "Customer expresses frustration",
//!     "I've been waiting for an hour!",
//! ).await?;
//! ```

use thiserror::Error;

/// Errors from the runtime.
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("LLM provider not configured")]
    NotConfigured,

    #[error("LLM call failed: {0}")]
    LlmError(String),

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Budget exceeded")]
    BudgetExceeded,
}

/// Configuration for LLM-assisted evaluation.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Model to use (e.g., "claude-3-5-sonnet")
    pub model: String,

    /// Maximum tokens per call
    pub max_tokens: u32,

    /// Temperature (0.0 for deterministic)
    pub temperature: f32,

    /// API endpoint (optional, uses default for provider)
    pub endpoint: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model: "claude-3-5-sonnet".to_string(),
            max_tokens: 500,
            temperature: 0.0, // Deterministic
            endpoint: None,
        }
    }
}

impl LlmConfig {
    /// Create a new config with the specified model.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }
}

/// Placeholder for LLM evaluator.
///
/// Full implementation will be added in Phase 5.
pub struct LlmEvaluator {
    #[allow(dead_code)]
    config: LlmConfig,
}

impl LlmEvaluator {
    /// Create a new LLM evaluator.
    pub fn new(config: LlmConfig) -> Result<Self, RuntimeError> {
        // TODO: Validate config, initialize provider client
        Ok(Self { config })
    }

    /// Evaluate a rule using LLM.
    ///
    /// This is a placeholder - full implementation in Phase 5.
    pub async fn evaluate_rule(
        &self,
        _rule: &str,
        _content: &str,
    ) -> Result<RuleInterpretation, RuntimeError> {
        Err(RuntimeError::NotConfigured)
    }
}

/// Result of LLM-based rule interpretation.
#[derive(Debug, Clone)]
pub struct RuleInterpretation {
    /// Whether the rule is satisfied
    pub satisfied: bool,

    /// Confidence in the interpretation (0.0 - 1.0)
    pub confidence: f64,

    /// Reasoning for the interpretation
    pub reasoning: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = LlmConfig::default();
        assert_eq!(config.temperature, 0.0);
        assert_eq!(config.max_tokens, 500);
    }

    #[test]
    fn test_evaluator_not_implemented() {
        let config = LlmConfig::new("test-model");
        let evaluator = LlmEvaluator::new(config).unwrap();

        // Run async test
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(evaluator.evaluate_rule("test", "content"));

        assert!(matches!(result, Err(RuntimeError::NotConfigured)));
    }
}
