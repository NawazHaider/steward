//! Accountability & Ownership Lens
//!
//! **Question**: Who approved this, who can stop it, and who answers for it?
//!
//! This lens validates the accountability section of the contract itself,
//! not the output content.

use crate::evidence::Evidence;
use crate::types::{
    EvaluationRequest, LensFinding, LensState, LensType, RuleEvaluation, RuleResult,
};

use super::Lens;

/// The Accountability & Ownership lens.
pub struct AccountabilityLens;

impl AccountabilityLens {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AccountabilityLens {
    fn default() -> Self {
        Self::new()
    }
}

impl Lens for AccountabilityLens {
    fn lens_type(&self) -> LensType {
        LensType::AccountabilityOwnership
    }

    fn evaluate(&self, request: &EvaluationRequest) -> LensFinding {
        let contract = &request.contract;
        let mut rules_evaluated = Vec::new();
        let mut issues = Vec::new();

        // Check answerable_human is present and non-empty
        // (This should already be validated by contract parser, but double-check)
        if contract.accountability.answerable_human.is_empty() {
            rules_evaluated.push(RuleEvaluation {
                rule_id: "ACC1".to_string(),
                rule_text: Some("Contract must have answerable_human".to_string()),
                result: RuleResult::Violated,
                evidence: vec![Evidence::from_contract(
                    "Missing answerable_human",
                    "accountability.answerable_human",
                )],
                rationale: Some("No accountable human defined".to_string()),
            });

            return LensFinding {
                lens: Some(self.lens_type()),
                question_asked: Some(self.question().to_string()),
                state: LensState::Blocked {
                    violation: "No accountable human defined in contract".to_string(),
                },
                rules_evaluated,
                confidence: 0.99,
            };
        }

        // Mark answerable_human as satisfied
        rules_evaluated.push(RuleEvaluation {
            rule_id: "ACC1".to_string(),
            rule_text: Some("Contract must have answerable_human".to_string()),
            result: RuleResult::Satisfied,
            evidence: vec![],
            rationale: Some(format!(
                "Accountable human: {}",
                contract.accountability.answerable_human
            )),
        });

        // Check escalation path exists
        if contract.accountability.escalation_path.is_empty() {
            rules_evaluated.push(RuleEvaluation {
                rule_id: "ACC2".to_string(),
                rule_text: Some("Contract should have escalation_path".to_string()),
                result: RuleResult::Uncertain,
                evidence: vec![],
                rationale: Some("No escalation path defined".to_string()),
            });

            issues.push("No escalation path defined");
        } else {
            rules_evaluated.push(RuleEvaluation {
                rule_id: "ACC2".to_string(),
                rule_text: Some("Contract should have escalation_path".to_string()),
                result: RuleResult::Satisfied,
                evidence: vec![],
                rationale: Some(format!(
                    "Escalation path has {} levels",
                    contract.accountability.escalation_path.len()
                )),
            });
        }

        // Check approval (recommended but not required)
        if contract.accountability.approved_by.is_none() {
            rules_evaluated.push(RuleEvaluation {
                rule_id: "ACC3".to_string(),
                rule_text: Some("Contract should have approved_by".to_string()),
                result: RuleResult::Uncertain,
                evidence: vec![],
                rationale: Some("No approval on record".to_string()),
            });

            issues.push("No approval on record");
        } else {
            rules_evaluated.push(RuleEvaluation {
                rule_id: "ACC3".to_string(),
                rule_text: Some("Contract should have approved_by".to_string()),
                result: RuleResult::Satisfied,
                evidence: vec![],
                rationale: None,
            });
        }

        // Determine final state
        let state = if !issues.is_empty() {
            LensState::Escalate {
                reason: issues.join("; "),
            }
        } else {
            LensState::Pass
        };

        // Calculate confidence
        let confidence = if issues.is_empty() {
            0.95
        } else {
            0.75
        };

        LensFinding {
            lens: Some(self.lens_type()),
            question_asked: Some(self.question().to_string()),
            state,
            rules_evaluated,
            confidence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Contract;
    use crate::types::Output;

    #[test]
    fn test_full_accountability_passes() {
        let contract = Contract::from_yaml(r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  approved_by: "Manager"
  answerable_human: "test@example.com"
  escalation_path:
    - "Tier 1"
    - "Manager"
"#).unwrap();

        let request = EvaluationRequest {
            contract,
            output: Output::text("Test output"),
            context: None,
            metadata: None,
        };

        let lens = AccountabilityLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass());
        assert!(finding.confidence > 0.9);
    }

    #[test]
    fn test_missing_escalation_path_escalates() {
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
            output: Output::text("Test output"),
            context: None,
            metadata: None,
        };

        let lens = AccountabilityLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_escalate());
    }

    #[test]
    fn test_missing_approved_by_escalates() {
        let contract = Contract::from_yaml(r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
  escalation_path:
    - "Tier 1"
"#).unwrap();

        let request = EvaluationRequest {
            contract,
            output: Output::text("Test output"),
            context: None,
            metadata: None,
        };

        let lens = AccountabilityLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_escalate());
        if let LensState::Escalate { reason } = &finding.state {
            assert!(reason.contains("approval"), "Should mention missing approval");
        }
    }

    #[test]
    fn test_both_missing_escalates_with_combined_reason() {
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
            output: Output::text("Test output"),
            context: None,
            metadata: None,
        };

        let lens = AccountabilityLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_escalate());
        if let LensState::Escalate { reason } = &finding.state {
            // Should contain both issues
            assert!(reason.contains("escalation"), "Should mention missing escalation path");
            assert!(reason.contains("approval"), "Should mention missing approval");
        }
    }

    #[test]
    fn test_single_level_escalation_path_passes() {
        let contract = Contract::from_yaml(r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  approved_by: "Product Owner"
  answerable_human: "support@example.com"
  escalation_path:
    - "Manager"
"#).unwrap();

        let request = EvaluationRequest {
            contract,
            output: Output::text("Test output"),
            context: None,
            metadata: None,
        };

        let lens = AccountabilityLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass(), "Single-level escalation path should be valid");
        assert!(finding.confidence > 0.9);
    }

    #[test]
    fn test_review_cadence_included() {
        let contract = Contract::from_yaml(r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  approved_by: "Manager"
  answerable_human: "test@example.com"
  escalation_path:
    - "Tier 1"
    - "Tier 2"
  review_cadence: "monthly"
"#).unwrap();

        let request = EvaluationRequest {
            contract,
            output: Output::text("Test output"),
            context: None,
            metadata: None,
        };

        let lens = AccountabilityLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass());
        // Verify review_cadence is parsed (it's metadata, doesn't affect pass/fail)
        assert_eq!(
            request.contract.accountability.review_cadence,
            Some("monthly".to_string())
        );
    }

    #[test]
    fn test_all_rules_evaluated() {
        let contract = Contract::from_yaml(r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  approved_by: "Manager"
  answerable_human: "test@example.com"
  escalation_path:
    - "Tier 1"
"#).unwrap();

        let request = EvaluationRequest {
            contract,
            output: Output::text("Test output"),
            context: None,
            metadata: None,
        };

        let lens = AccountabilityLens::new();
        let finding = lens.evaluate(&request);

        // Should evaluate all 3 accountability rules
        assert_eq!(finding.rules_evaluated.len(), 3, "Should evaluate ACC1, ACC2, ACC3");

        let rule_ids: Vec<&str> = finding.rules_evaluated.iter()
            .map(|r| r.rule_id.as_str())
            .collect();
        assert!(rule_ids.contains(&"ACC1"), "Should evaluate answerable_human");
        assert!(rule_ids.contains(&"ACC2"), "Should evaluate escalation_path");
        assert!(rule_ids.contains(&"ACC3"), "Should evaluate approved_by");
    }

    #[test]
    fn test_deep_escalation_path() {
        let contract = Contract::from_yaml(r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Enterprise Support"
intent:
  purpose: "Handle enterprise customer requests"
accountability:
  approved_by: "VP of Engineering"
  answerable_human: "enterprise-support@company.com"
  escalation_path:
    - "Tier 1 Support"
    - "Tier 2 Support"
    - "Engineering Lead"
    - "Director of Engineering"
    - "VP of Engineering"
"#).unwrap();

        let request = EvaluationRequest {
            contract,
            output: Output::text("Test output"),
            context: None,
            metadata: None,
        };

        let lens = AccountabilityLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass());

        // Check the rationale mentions the number of levels
        let acc2_rule = finding.rules_evaluated.iter()
            .find(|r| r.rule_id == "ACC2")
            .expect("Should have ACC2 rule");
        assert!(
            acc2_rule.rationale.as_ref().unwrap().contains("5 levels"),
            "Should note 5 escalation levels"
        );
    }
}
