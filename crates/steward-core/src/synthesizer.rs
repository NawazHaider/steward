//! Synthesizer: Aggregates lens findings into final state.
//!
//! The synthesizer applies strict, non-configurable policy rules:
//! 1. If ANY lens returns BLOCKED → final state is BLOCKED
//! 2. Else if ANY lens returns ESCALATE → final state is ESCALATE
//! 3. Else → final state is PROCEED
//!
//! These rules are governance machinery, not a tuning toy.

use chrono::Utc;

use crate::contract::Contract;
use crate::types::{
    BoundaryViolation, EvaluationResult, LensFindings, LensState, LensType, State,
};

/// The Synthesizer aggregates lens findings into a final result.
pub struct Synthesizer;

impl Synthesizer {
    pub fn new() -> Self {
        Self
    }

    /// Synthesize lens findings into a final evaluation result.
    ///
    /// # Arguments
    ///
    /// * `findings` - Findings from all five lenses
    /// * `contract` - The contract (for accountable_human in violations)
    ///
    /// # Returns
    ///
    /// An `EvaluationResult` with the final state and confidence.
    pub fn synthesize(&self, findings: LensFindings, contract: &Contract) -> EvaluationResult {
        let accountable_human = contract.accountability.answerable_human.clone();

        // Calculate confidence first before moving findings
        let confidence = self.calculate_confidence_from_findings(&findings);

        // Check for BLOCKED state (Rule 1: Any BLOCKED -> BLOCKED)
        if let Some((lens_type, rule_id, rule_text, evidence)) = self.find_blocked(&findings) {
            return EvaluationResult {
                state: State::Blocked {
                    violation: BoundaryViolation {
                        lens: lens_type,
                        rule_id,
                        rule_text,
                        evidence,
                        accountable_human,
                    },
                },
                lens_findings: findings,
                confidence,
                evaluated_at: Utc::now(),
            };
        }

        // Check for ESCALATE state (Rule 2: Any ESCALATE -> ESCALATE)
        if let Some((lens_type, reason)) = self.find_escalate(&findings) {
            return EvaluationResult {
                state: State::Escalate {
                    uncertainty: reason.clone(),
                    decision_point: self.build_decision_point(lens_type, &reason),
                    options: self.build_options(lens_type, &reason),
                },
                lens_findings: findings,
                confidence,
                evaluated_at: Utc::now(),
            };
        }

        // Rule 3: Otherwise -> PROCEED
        let summary = self.build_summary(&findings);
        EvaluationResult {
            state: State::Proceed { summary },
            lens_findings: findings,
            confidence,
            evaluated_at: Utc::now(),
        }
    }

    /// Find the first BLOCKED lens and extract violation details.
    fn find_blocked(&self, findings: &LensFindings) -> Option<(LensType, String, String, Vec<crate::evidence::Evidence>)> {
        let checks = [
            (LensType::DignityInclusion, &findings.dignity_inclusion),
            (LensType::BoundariesSafety, &findings.boundaries_safety),
            (LensType::RestraintPrivacy, &findings.restraint_privacy),
            (LensType::TransparencyContestability, &findings.transparency_contestability),
            (LensType::AccountabilityOwnership, &findings.accountability_ownership),
        ];

        for (lens_type, finding) in &checks {
            if let LensState::Blocked { violation } = &finding.state {
                let violated_rule = finding
                    .rules_evaluated
                    .iter()
                    .find(|r| matches!(r.result, crate::types::RuleResult::Violated));

                let (rule_id, rule_text, evidence) = if let Some(rule) = violated_rule {
                    (
                        rule.rule_id.clone(),
                        rule.rule_text.clone().unwrap_or_else(|| violation.clone()),
                        rule.evidence.clone(),
                    )
                } else {
                    (
                        "UNKNOWN".to_string(),
                        violation.clone(),
                        vec![],
                    )
                };

                return Some((*lens_type, rule_id, rule_text, evidence));
            }
        }

        None
    }

    /// Find the first ESCALATE lens and extract the reason.
    fn find_escalate(&self, findings: &LensFindings) -> Option<(LensType, String)> {
        let checks = [
            (LensType::DignityInclusion, &findings.dignity_inclusion),
            (LensType::BoundariesSafety, &findings.boundaries_safety),
            (LensType::RestraintPrivacy, &findings.restraint_privacy),
            (LensType::TransparencyContestability, &findings.transparency_contestability),
            (LensType::AccountabilityOwnership, &findings.accountability_ownership),
        ];

        for (lens_type, finding) in &checks {
            if let LensState::Escalate { reason } = &finding.state {
                return Some((*lens_type, reason.clone()));
            }
        }

        None
    }

    /// Calculate overall confidence from LensFindings.
    fn calculate_confidence_from_findings(&self, findings: &LensFindings) -> f64 {
        [
            findings.dignity_inclusion.confidence,
            findings.boundaries_safety.confidence,
            findings.restraint_privacy.confidence,
            findings.transparency_contestability.confidence,
            findings.accountability_ownership.confidence,
        ]
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min)
        .min(1.0)
        .max(0.0)
    }

    /// Build a human-readable summary for PROCEED state.
    fn build_summary(&self, findings: &LensFindings) -> String {
        let mut summary = String::from("All contract conditions satisfied. ");

        let total_rules: usize = [
            &findings.dignity_inclusion,
            &findings.boundaries_safety,
            &findings.restraint_privacy,
            &findings.transparency_contestability,
            &findings.accountability_ownership,
        ]
        .iter()
        .map(|f| f.rules_evaluated.len())
        .sum();

        if total_rules > 0 {
            summary.push_str(&format!("{} rules evaluated. ", total_rules));
        }

        summary.push_str("Output may proceed.");
        summary
    }

    /// Build decision point description for ESCALATE state.
    fn build_decision_point(&self, lens: LensType, reason: &str) -> String {
        match lens {
            LensType::BoundariesSafety => {
                format!(
                    "Should automation continue or should a human take over? Trigger: {}",
                    reason
                )
            }
            LensType::DignityInclusion => {
                format!(
                    "Does this output preserve human dignity? Concern: {}",
                    reason
                )
            }
            LensType::RestraintPrivacy => {
                format!(
                    "Is this data exposure appropriate? Concern: {}",
                    reason
                )
            }
            LensType::TransparencyContestability => {
                format!(
                    "Can the recipient understand and challenge this? Issue: {}",
                    reason
                )
            }
            LensType::AccountabilityOwnership => {
                format!(
                    "Is accountability clear for this automation? Issue: {}",
                    reason
                )
            }
        }
    }

    /// Build options for ESCALATE state (no ranking - options presented equally).
    fn build_options(&self, lens: LensType, _reason: &str) -> Vec<String> {
        match lens {
            LensType::BoundariesSafety => vec![
                "Continue with automated response - condition is minor".to_string(),
                "Transfer to human agent - honor the trigger condition".to_string(),
                "Acknowledge the trigger, then offer human transfer option".to_string(),
            ],
            LensType::DignityInclusion => vec![
                "Proceed - output preserves dignity adequately".to_string(),
                "Revise output to address dignity concern".to_string(),
                "Escalate to human for judgment".to_string(),
            ],
            LensType::RestraintPrivacy => vec![
                "Proceed - exposure is acceptable for this context".to_string(),
                "Redact sensitive information before proceeding".to_string(),
                "Block and notify privacy team".to_string(),
            ],
            LensType::TransparencyContestability => vec![
                "Proceed - transparency is sufficient".to_string(),
                "Add clarifying information before proceeding".to_string(),
                "Escalate for human review".to_string(),
            ],
            LensType::AccountabilityOwnership => vec![
                "Proceed - accountability is clear enough".to_string(),
                "Add accountability information to output".to_string(),
                "Update contract with missing accountability".to_string(),
            ],
        }
    }
}

impl Default for Synthesizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LensFinding, LensState};

    fn test_contract() -> Contract {
        Contract::from_yaml(r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
"#).unwrap()
    }

    fn pass_finding(lens: LensType) -> LensFinding {
        LensFinding {
            lens: Some(lens),
            question_asked: None,
            state: LensState::Pass,
            rules_evaluated: vec![],
            confidence: 0.9,
        }
    }

    fn blocked_finding(lens: LensType) -> LensFinding {
        LensFinding {
            lens: Some(lens),
            question_asked: None,
            state: LensState::Blocked {
                violation: "Test violation".to_string(),
            },
            rules_evaluated: vec![],
            confidence: 0.95,
        }
    }

    fn escalate_finding(lens: LensType) -> LensFinding {
        LensFinding {
            lens: Some(lens),
            question_asked: None,
            state: LensState::Escalate {
                reason: "Test escalation".to_string(),
            },
            rules_evaluated: vec![],
            confidence: 0.7,
        }
    }

    #[test]
    fn test_all_pass_yields_proceed() {
        let findings = LensFindings {
            dignity_inclusion: pass_finding(LensType::DignityInclusion),
            boundaries_safety: pass_finding(LensType::BoundariesSafety),
            restraint_privacy: pass_finding(LensType::RestraintPrivacy),
            transparency_contestability: pass_finding(LensType::TransparencyContestability),
            accountability_ownership: pass_finding(LensType::AccountabilityOwnership),
        };

        let synthesizer = Synthesizer::new();
        let result = synthesizer.synthesize(findings, &test_contract());

        assert!(matches!(result.state, State::Proceed { .. }));
    }

    #[test]
    fn test_one_blocked_yields_blocked() {
        let findings = LensFindings {
            dignity_inclusion: pass_finding(LensType::DignityInclusion),
            boundaries_safety: blocked_finding(LensType::BoundariesSafety),
            restraint_privacy: pass_finding(LensType::RestraintPrivacy),
            transparency_contestability: pass_finding(LensType::TransparencyContestability),
            accountability_ownership: pass_finding(LensType::AccountabilityOwnership),
        };

        let synthesizer = Synthesizer::new();
        let result = synthesizer.synthesize(findings, &test_contract());

        assert!(matches!(result.state, State::Blocked { .. }));
    }

    #[test]
    fn test_blocked_takes_priority_over_escalate() {
        let findings = LensFindings {
            dignity_inclusion: escalate_finding(LensType::DignityInclusion),
            boundaries_safety: blocked_finding(LensType::BoundariesSafety),
            restraint_privacy: pass_finding(LensType::RestraintPrivacy),
            transparency_contestability: pass_finding(LensType::TransparencyContestability),
            accountability_ownership: pass_finding(LensType::AccountabilityOwnership),
        };

        let synthesizer = Synthesizer::new();
        let result = synthesizer.synthesize(findings, &test_contract());

        // BLOCKED should take priority
        assert!(matches!(result.state, State::Blocked { .. }));
    }

    #[test]
    fn test_escalate_when_no_blocked() {
        let findings = LensFindings {
            dignity_inclusion: escalate_finding(LensType::DignityInclusion),
            boundaries_safety: pass_finding(LensType::BoundariesSafety),
            restraint_privacy: pass_finding(LensType::RestraintPrivacy),
            transparency_contestability: pass_finding(LensType::TransparencyContestability),
            accountability_ownership: pass_finding(LensType::AccountabilityOwnership),
        };

        let synthesizer = Synthesizer::new();
        let result = synthesizer.synthesize(findings, &test_contract());

        assert!(matches!(result.state, State::Escalate { .. }));
    }

    #[test]
    fn test_confidence_is_minimum() {
        let mut findings = LensFindings {
            dignity_inclusion: pass_finding(LensType::DignityInclusion),
            boundaries_safety: pass_finding(LensType::BoundariesSafety),
            restraint_privacy: pass_finding(LensType::RestraintPrivacy),
            transparency_contestability: pass_finding(LensType::TransparencyContestability),
            accountability_ownership: pass_finding(LensType::AccountabilityOwnership),
        };

        // Set one lens to low confidence
        findings.boundaries_safety.confidence = 0.5;

        let synthesizer = Synthesizer::new();
        let result = synthesizer.synthesize(findings, &test_contract());

        assert_eq!(result.confidence, 0.5);
    }
}
