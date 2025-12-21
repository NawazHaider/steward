//! Fallback strategies when LLM calls fail.

use serde::{Deserialize, Serialize};

/// Fallback strategy when LLM fails or budget exceeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FallbackStrategy {
    /// Use deterministic evaluation from steward-core
    Deterministic,

    /// Return cached result if available
    Cache,

    /// Use a simpler/cheaper model
    SimplerModel { model: String },

    /// Return ESCALATE with low confidence
    EscalateWithUncertainty,

    /// Fail the evaluation
    Fail,
}

impl Default for FallbackStrategy {
    fn default() -> Self {
        Self::Deterministic
    }
}

/// Fallback chain - tried in order.
///
/// Note: Currently the orchestrator uses `Vec<FallbackStrategy>` from config directly.
/// This struct provides a builder pattern for programmatic chain construction.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Builder pattern reserved for programmatic use
pub struct FallbackChain {
    strategies: Vec<FallbackStrategy>,
}

#[allow(dead_code)] // Builder pattern reserved for programmatic use
impl FallbackChain {
    /// Create a new fallback chain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a strategy to the chain.
    pub fn add(mut self, strategy: FallbackStrategy) -> Self {
        self.strategies.push(strategy);
        self
    }

    /// Get the default fallback chain.
    ///
    /// Order: Cache -> Simpler Model -> Deterministic -> Escalate
    pub fn default_chain() -> Self {
        Self::new()
            .add(FallbackStrategy::Cache)
            .add(FallbackStrategy::SimplerModel {
                model: "claude-haiku-4-5".to_string(),
            })
            .add(FallbackStrategy::Deterministic)
            .add(FallbackStrategy::EscalateWithUncertainty)
    }

    /// Get strategies in order.
    pub fn strategies(&self) -> &[FallbackStrategy] {
        &self.strategies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_chain() {
        let chain = FallbackChain::default_chain();
        assert_eq!(chain.strategies().len(), 4);

        assert!(matches!(chain.strategies()[0], FallbackStrategy::Cache));
        assert!(matches!(
            chain.strategies()[1],
            FallbackStrategy::SimplerModel { .. }
        ));
    }
}
