//! # steward-core
//!
//! Deterministic stewardship contract evaluation engine.
//!
//! This crate provides the core evaluation logic for Steward, answering:
//! - Should this automation proceed?
//! - Where must it stop?
//! - Who answers for it?
//!
//! ## Key Guarantees
//!
//! 1. **Deterministic**: Same input always produces same output
//! 2. **No LLM calls**: All evaluation is rule-based
//! 3. **Traceable**: Every BLOCKED cites rule_id and evidence
//! 4. **Parallel-safe**: Lenses evaluate independently
//!
//! ## Example
//!
//! ```rust,ignore
//! use steward_core::{Contract, Output, evaluate};
//!
//! let contract = Contract::from_yaml_file("contract.yaml")?;
//! let output = Output::text("Your order shipped yesterday.");
//! let result = evaluate(&contract, &output)?;
//!
//! match result.state {
//!     State::Proceed { summary } => println!("OK: {}", summary),
//!     State::Escalate { decision_point, .. } => println!("ESCALATE: {}", decision_point),
//!     State::Blocked { violation } => println!("BLOCKED: {}", violation.rule_id),
//! }
//! ```

pub mod contract;
pub mod evidence;
pub mod lenses;
pub mod synthesizer;
pub mod types;

// Re-export main types at crate root
pub use contract::{Contract, ContractError};
pub use evidence::Evidence;
pub use lenses::{
    AccountabilityLens, BoundariesLens, DignityLens, Lens, LensFinding, LensState,
    RestraintLens, TransparencyLens,
};
pub use synthesizer::Synthesizer;
pub use types::{
    BoundaryViolation, ContentType, EvaluationRequest, EvaluationResult, EvidenceSource,
    LensFindings, LensType, Output, RuleEvaluation, RuleResult, State,
};

use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during evaluation
#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("Contract error: {0}")]
    Contract(#[from] ContractError),

    #[error("Invalid output: {0}")]
    InvalidOutput(String),

    #[error("Lens evaluation failed: {0}")]
    LensError(String),
}

/// Evaluate an output against a stewardship contract.
///
/// This is the main entry point for Steward evaluation.
///
/// # Arguments
///
/// * `contract` - The stewardship contract defining rules
/// * `output` - The AI-generated output to evaluate
///
/// # Returns
///
/// An `EvaluationResult` containing:
/// - `state`: PROCEED, ESCALATE, or BLOCKED
/// - `lens_findings`: What each lens observed
/// - `confidence`: How well-supported the findings are
/// - `evaluated_at`: Timestamp of evaluation
pub fn evaluate(contract: &Contract, output: &Output) -> Result<EvaluationResult, EvaluationError> {
    evaluate_with_context(contract, output, None, None)
}

/// Evaluate with optional context and metadata.
///
/// # Arguments
///
/// * `contract` - The stewardship contract
/// * `output` - The AI-generated output
/// * `context` - Optional context the AI had access to
/// * `metadata` - Optional metadata for the evaluation
pub fn evaluate_with_context(
    contract: &Contract,
    output: &Output,
    context: Option<&[String]>,
    metadata: Option<&HashMap<String, String>>,
) -> Result<EvaluationResult, EvaluationError> {
    // Create evaluation request
    let request = EvaluationRequest {
        contract: contract.clone(),
        output: output.clone(),
        context: context.map(|c| c.to_vec()),
        metadata: metadata.cloned(),
    };

    // Fan-out: Run all lenses in parallel (simulated with sequential for now)
    // In production, this would use rayon or tokio for true parallelism
    let dignity_finding = DignityLens::new().evaluate(&request);
    let boundaries_finding = BoundariesLens::new().evaluate(&request);
    let restraint_finding = RestraintLens::new().evaluate(&request);
    let transparency_finding = TransparencyLens::new().evaluate(&request);
    let accountability_finding = AccountabilityLens::new().evaluate(&request);

    // Fan-in: Collect findings
    let findings = LensFindings {
        dignity_inclusion: dignity_finding,
        boundaries_safety: boundaries_finding,
        restraint_privacy: restraint_finding,
        transparency_contestability: transparency_finding,
        accountability_ownership: accountability_finding,
    };

    // Synthesize final result
    let synthesizer = Synthesizer::new();
    let result = synthesizer.synthesize(findings, contract);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_evaluation() {
        let contract_yaml = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test Contract"
intent:
  purpose: "Test evaluation"
boundaries:
  may_do_autonomously:
    - id: "A1"
      rule: "Answer questions"
accountability:
  approved_by: "Manager"
  answerable_human: "support@example.com"
  escalation_path:
    - "Tier 1 Support"
    - "Manager"
acceptance: {}
"#;

        let contract = Contract::from_yaml(contract_yaml).unwrap();
        let output = Output::text("This is a test response.");
        let result = evaluate(&contract, &output).unwrap();

        // Should proceed since no violations and full accountability
        assert!(matches!(result.state, State::Proceed { .. }));
    }

    #[test]
    fn test_pii_blocked() {
        let contract_yaml = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test Contract"
intent:
  purpose: "Test evaluation"
boundaries:
  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed in response"
accountability:
  answerable_human: "test@example.com"
acceptance: {}
"#;

        let contract = Contract::from_yaml(contract_yaml).unwrap();
        let output = Output::text("Contact john.doe@email.com for help.");
        let result = evaluate(&contract, &output).unwrap();

        // Should be blocked due to PII
        assert!(matches!(result.state, State::Blocked { .. }));
    }
}
