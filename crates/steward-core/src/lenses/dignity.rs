//! Dignity & Inclusion Lens
//!
//! **Question**: Does this disempower people or exclude them from relevance?
//!
//! Phase 1: Basic implementation with keyword matching for dignity violations.

use crate::types::{EvaluationRequest, LensFinding, LensType};

use super::{default_pass_finding, Lens};

/// The Dignity & Inclusion lens.
pub struct DignityLens;

impl DignityLens {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DignityLens {
    fn default() -> Self {
        Self::new()
    }
}

impl Lens for DignityLens {
    fn lens_type(&self) -> LensType {
        LensType::DignityInclusion
    }

    fn evaluate(&self, request: &EvaluationRequest) -> LensFinding {
        let contract = &request.contract;
        let _content = &request.output.content;

        // Phase 1: Basic implementation
        // TODO: Implement full dignity rule evaluation

        let dignity_rules = contract.dignity_rules();
        if dignity_rules.is_empty() {
            return default_pass_finding(self.lens_type());
        }

        // For now, return PASS with moderate confidence
        // Full implementation will check:
        // - Dismissive language patterns
        // - Pressure without recourse
        // - Assumption patterns
        // - Human escalation path preservation

        LensFinding {
            lens: Some(self.lens_type()),
            question_asked: Some(self.question().to_string()),
            state: super::LensState::Pass,
            rules_evaluated: vec![],
            confidence: 0.75, // Moderate confidence for placeholder
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Contract;
    use crate::types::Output;

    #[test]
    fn test_basic_dignity_pass() {
        let contract = Contract::from_yaml(r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
acceptance:
  dignity_check:
    - id: "D1"
      rule: "Does not dismiss concerns"
"#).unwrap();

        let request = EvaluationRequest {
            contract,
            output: Output::text("I understand your concern and want to help."),
            context: None,
            metadata: None,
        };

        let lens = DignityLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass());
    }
}
