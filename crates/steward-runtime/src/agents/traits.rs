//! Lens agent trait and common types.

use async_trait::async_trait;
use std::time::Duration;
use steward_core::{EvaluationRequest, LensFinding, LensType};
use thiserror::Error;

/// Errors from lens agents.
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("LLM call failed: {0}")]
    LlmError(String),

    #[error("Evidence validation failed: {0}")]
    EvidenceInvalid(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Budget exceeded")]
    BudgetExceeded,

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Trait for lens agents that can use LLM assistance.
///
/// # Isolation Contract
/// Each lens agent operates in isolation:
/// - No shared mutable state between agents
/// - No access to other agents' findings during evaluation
/// - Deterministic ordering via BTreeMap (not HashMap)
/// - Same input always produces same output for deterministic rules
#[async_trait]
pub trait LensAgent: Send + Sync {
    /// The lens type this agent evaluates.
    fn lens_type(&self) -> LensType;

    /// Evaluate rules for this lens.
    ///
    /// # Arguments
    /// * `request` - The evaluation request containing contract, output, and context
    ///
    /// # Returns
    /// A `LensFinding` with the lens state, rule evaluations, and confidence
    ///
    /// # Isolation Contract
    /// - MUST NOT access other lens findings
    /// - MUST NOT share state with other agents
    /// - MUST return deterministic results for deterministic rules
    async fn evaluate(&self, request: &EvaluationRequest) -> Result<LensFinding, AgentError>;

    /// Token budget for this agent (if using LLM).
    fn token_budget(&self) -> u32 {
        1000 // Default 1000 tokens per lens
    }

    /// Timeout for this agent.
    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }

    /// Whether this lens needs LLM for the given request.
    ///
    /// Returns true if any rules require semantic interpretation.
    fn needs_llm(&self, _request: &EvaluationRequest) -> bool {
        false // Default: no LLM needed
    }
}
