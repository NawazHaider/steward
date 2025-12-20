//! Restraint & Privacy Lens
//!
//! **Question**: Does this expose what should be protected?
//!
//! Phase 1: Delegates PII detection to Boundaries lens.
//! This lens focuses on scope creep and data minimization.

use crate::types::{EvaluationRequest, LensFinding, LensType};

use super::{default_pass_finding, Lens};

/// The Restraint & Privacy lens.
pub struct RestraintLens;

impl RestraintLens {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RestraintLens {
    fn default() -> Self {
        Self::new()
    }
}

impl Lens for RestraintLens {
    fn lens_type(&self) -> LensType {
        LensType::RestraintPrivacy
    }

    fn evaluate(&self, request: &EvaluationRequest) -> LensFinding {
        let contract = &request.contract;
        let _content = &request.output.content;

        // Phase 1: Basic implementation
        // PII detection is handled by Boundaries lens
        // This lens focuses on:
        // - Scope creep beyond stated purpose
        // - Data minimization violations
        // - Unnecessary data retention

        let restraint_rules = contract.restraint_rules();
        if restraint_rules.is_empty() {
            return default_pass_finding(self.lens_type());
        }

        // TODO: Implement scope and data minimization checks

        LensFinding {
            lens: Some(self.lens_type()),
            question_asked: Some(self.question().to_string()),
            state: super::LensState::Pass,
            rules_evaluated: vec![],
            confidence: 0.70, // Lower confidence for placeholder
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Contract;
    use crate::types::Output;

    #[test]
    fn test_basic_restraint_pass() {
        let contract = Contract::from_yaml(r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
"#).unwrap();

        let request = EvaluationRequest {
            contract,
            output: Output::text("Your order is on the way."),
            context: None,
            metadata: None,
        };

        let lens = RestraintLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass());
    }
}
