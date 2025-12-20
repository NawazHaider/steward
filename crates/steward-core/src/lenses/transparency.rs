//! Transparency & Contestability Lens
//!
//! **Question**: Can the human understand why this happened and contest it?
//!
//! Phase 1: Basic implementation checking for fit criteria.

use crate::types::{EvaluationRequest, LensFinding, LensType};

use super::{default_pass_finding, Lens};

/// The Transparency & Contestability lens.
pub struct TransparencyLens;

impl TransparencyLens {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TransparencyLens {
    fn default() -> Self {
        Self::new()
    }
}

impl Lens for TransparencyLens {
    fn lens_type(&self) -> LensType {
        LensType::TransparencyContestability
    }

    fn evaluate(&self, request: &EvaluationRequest) -> LensFinding {
        let contract = &request.contract;
        let _content = &request.output.content;

        // Phase 1: Basic implementation
        // TODO: Implement full transparency checks:
        // - Uncited claims detection
        // - Assumption visibility
        // - Contestability path presence
        // - AI disclosure

        let fit_criteria = contract.transparency_rules();
        if fit_criteria.is_empty() {
            return default_pass_finding(self.lens_type());
        }

        LensFinding {
            lens: Some(self.lens_type()),
            question_asked: Some(self.question().to_string()),
            state: super::LensState::Pass,
            rules_evaluated: vec![],
            confidence: 0.70,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Contract;
    use crate::types::Output;

    #[test]
    fn test_basic_transparency_pass() {
        let contract = Contract::from_yaml(r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
acceptance:
  fit_criteria:
    - id: "F1"
      rule: "Cites sources when making claims"
"#).unwrap();

        let request = EvaluationRequest {
            contract,
            output: Output::text("According to our records, your order shipped."),
            context: None,
            metadata: None,
        };

        let lens = TransparencyLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass());
    }
}
