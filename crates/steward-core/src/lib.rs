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

/// Cross-lens integration tests verifying the fan-out/fan-in architecture
#[cfg(test)]
mod cross_lens_tests {
    use super::*;

    /// Full contract with rules for all five lenses
    const FULL_CONTRACT: &str = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Full Contract"
intent:
  purpose: "Customer support with comprehensive rules"
  never_optimize_away:
    - id: "N1"
      rule: "Human escalation path must always be available"
boundaries:
  may_do_autonomously:
    - id: "A1"
      rule: "Answer factual questions about products"
    - id: "A2"
      rule: "Provide order status from verified data"
  must_escalate_when:
    - id: "E1"
      rule: "Customer requests human agent"
  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed in response"
    - id: "B5"
      rule: "Internal system credentials or secrets exposed"
accountability:
  approved_by: "Manager"
  answerable_human: "support@example.com"
  escalation_path:
    - "Tier 1 Support"
    - "Manager"
acceptance:
  dignity_check:
    - id: "D1"
      rule: "Does not dismiss or minimize customer concerns"
    - id: "D2"
      rule: "Does not pressure customer toward automated resolution"
    - id: "D3"
      rule: "Preserves clear path to human help"
  fit_criteria:
    - id: "F1"
      rule: "Addresses the customer's actual question"
    - id: "F4"
      rule: "Cites sources when making factual claims"
"#;

    #[test]
    fn test_all_lenses_pass_yields_proceed() {
        let contract = Contract::from_yaml(FULL_CONTRACT).unwrap();
        let output = Output::text(
            "Regarding your question about the order: According to our records, \
             your order shipped yesterday. Here's what you can do: track your package \
             at the link in your email. If you need further assistance, please contact us \
             or speak to a human agent."
        );
        let result = evaluate(&contract, &output).unwrap();

        // All lenses should pass with this well-crafted response
        assert!(matches!(result.state, State::Proceed { .. }));
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_blocked_takes_priority_over_escalate() {
        let contract = Contract::from_yaml(FULL_CONTRACT).unwrap();

        // This output has BOTH:
        // - PII exposure (BLOCKED by Boundaries/Restraint)
        // - Missing escalation path (ESCALATE by Dignity)
        let output = Output::text(
            "Your order shipped to john.doe@email.com. \
             That's all I can tell you."
        );
        let result = evaluate(&contract, &output).unwrap();

        // BLOCKED should take priority
        assert!(matches!(result.state, State::Blocked { .. }));

        if let State::Blocked { violation } = &result.state {
            // Should cite the PII violation, not the dignity concern
            assert!(violation.rule_id == "B1" || violation.lens == LensType::BoundariesSafety || violation.lens == LensType::RestraintPrivacy);
        }
    }

    #[test]
    fn test_multiple_escalate_conditions() {
        let contract = Contract::from_yaml(FULL_CONTRACT).unwrap();

        // This output has multiple ESCALATE conditions:
        // - Missing human escalation path (Dignity)
        // - Uncited claims (Transparency)
        let output = Output::text(
            "Studies show that 99% of customers are satisfied."
        );
        let result = evaluate(&contract, &output).unwrap();

        // Should be ESCALATE (not BLOCKED, not PROCEED)
        assert!(matches!(result.state, State::Escalate { .. }));
    }

    #[test]
    fn test_lens_independence_pii_detected_by_both() {
        let contract_yaml = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test Contract"
intent:
  purpose: "Test"
boundaries:
  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed in response"
    - id: "B5"
      rule: "Internal system credentials or secrets exposed"
accountability:
  answerable_human: "test@example.com"
"#;

        let contract = Contract::from_yaml(contract_yaml).unwrap();
        let output = Output::text(
            "The customer's phone is (555) 123-4567."
        );
        let result = evaluate(&contract, &output).unwrap();

        // Both Boundaries and Restraint should detect the PII
        // But the synthesizer should only report one BLOCKED
        assert!(matches!(result.state, State::Blocked { .. }));

        // Verify both lenses flagged it
        assert!(result.lens_findings.boundaries_safety.state.is_blocked());
        assert!(result.lens_findings.restraint_privacy.state.is_blocked());
    }

    #[test]
    fn test_confidence_is_minimum_of_lenses() {
        let contract = Contract::from_yaml(FULL_CONTRACT).unwrap();
        let output = Output::text(
            "I understand and I'm happy to help. According to our policy, \
             you can return items within 30 days. If you need more help, \
             please contact us or speak to a human agent."
        );
        let result = evaluate(&contract, &output).unwrap();

        // Overall confidence should be minimum of all lens confidences
        let min_lens_confidence = [
            result.lens_findings.dignity_inclusion.confidence,
            result.lens_findings.boundaries_safety.confidence,
            result.lens_findings.restraint_privacy.confidence,
            result.lens_findings.transparency_contestability.confidence,
            result.lens_findings.accountability_ownership.confidence,
        ].iter().cloned().fold(f64::INFINITY, f64::min);

        assert_eq!(result.confidence, min_lens_confidence);
    }

    #[test]
    fn test_dismissive_language_blocked() {
        let contract = Contract::from_yaml(FULL_CONTRACT).unwrap();
        let output = Output::text(
            "That's not how it works. You should have read the documentation. \
             There's nothing I can do about that."
        );
        let result = evaluate(&contract, &output).unwrap();

        // Dismissive language should trigger BLOCKED from Dignity lens
        assert!(matches!(result.state, State::Blocked { .. }));
        assert!(result.lens_findings.dignity_inclusion.state.is_blocked());
    }

    #[test]
    fn test_pressure_without_recourse_blocked() {
        let contract = Contract::from_yaml(FULL_CONTRACT).unwrap();
        let output = Output::text(
            "You must accept this offer immediately. This is your final chance. \
             No exceptions can be made."
        );
        let result = evaluate(&contract, &output).unwrap();

        // Pressure without escalation path should trigger BLOCKED
        assert!(matches!(result.state, State::Blocked { .. }));
    }

    #[test]
    fn test_api_key_exposure_blocked() {
        let contract = Contract::from_yaml(FULL_CONTRACT).unwrap();
        let output = Output::text(
            "Here's your API key: api_key: sk-prod-verylongsecretkeythatshouldbehidden123"
        );
        let result = evaluate(&contract, &output).unwrap();

        // API key should trigger BLOCKED from either Boundaries or Restraint
        assert!(matches!(result.state, State::Blocked { .. }));
        assert!(
            result.lens_findings.boundaries_safety.state.is_blocked() ||
            result.lens_findings.restraint_privacy.state.is_blocked()
        );
    }

    #[test]
    fn test_determinism_same_input_same_output() {
        let contract = Contract::from_yaml(FULL_CONTRACT).unwrap();
        let output = Output::text("Your order shipped yesterday. Contact us if needed.");

        // Run evaluation 3 times
        let result1 = evaluate(&contract, &output).unwrap();
        let result2 = evaluate(&contract, &output).unwrap();
        let result3 = evaluate(&contract, &output).unwrap();

        // All should produce the same state
        assert_eq!(
            std::mem::discriminant(&result1.state),
            std::mem::discriminant(&result2.state)
        );
        assert_eq!(
            std::mem::discriminant(&result2.state),
            std::mem::discriminant(&result3.state)
        );

        // All should produce the same confidence
        assert_eq!(result1.confidence, result2.confidence);
        assert_eq!(result2.confidence, result3.confidence);
    }
}
