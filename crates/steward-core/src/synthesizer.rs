//! Synthesizer: Aggregates lens findings into final state.
//!
//! The synthesizer applies strict, non-configurable policy rules:
//! 1. If ANY lens returns BLOCKED → final state is BLOCKED
//! 2. Else if ANY lens returns ESCALATE → final state is ESCALATE
//! 3. Else if confidence < 0.4 → force ESCALATE (the honesty rule)
//! 4. Else → final state is PROCEED
//!
//! The honesty rule: uncertainty is a governance signal, not an error to hide.
//! Low confidence means the system cannot reliably assess the output.
//!
//! These rules are governance machinery, not a tuning toy.

use chrono::{DateTime, Utc};

use crate::contract::Contract;
use crate::types::{
    BoundaryViolation, EvaluationResult, LensFindings, LensState, LensType, State,
};

/// The Synthesizer aggregates lens findings into a final result.
pub struct Synthesizer;

impl Synthesizer {
    /// The honesty rule threshold: confidence below this triggers ESCALATE.
    ///
    /// Per spec section 2.4: "< 0.4: Low confidence (triggers ESCALATE if no BLOCKED)"
    ///
    /// # Security Note
    ///
    /// This is a compile-time constant, not a runtime configuration.
    /// The threshold cannot be modified after compilation, ensuring governance
    /// policy cannot be bypassed at runtime. Public visibility enables:
    /// - Transparent policy (aligns with Steward's transparency principle)
    /// - Clean test assertions without magic numbers
    /// - External documentation/tooling can reference the authoritative value
    pub const LOW_CONFIDENCE_THRESHOLD: f64 = 0.4;

    pub fn new() -> Self {
        Self
    }

    /// Synthesize lens findings into a final evaluation result.
    ///
    /// This is the convenience wrapper that uses the current time.
    /// For deterministic evaluation (testing, reproducibility), use `synthesize_at()`.
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
        self.synthesize_at(findings, contract, Utc::now())
    }

    /// Synthesize lens findings into a final evaluation result with explicit timestamp.
    ///
    /// This function is fully deterministic: same inputs always produce same output.
    /// Use this for testing, golden tests, and reproducible evaluation.
    ///
    /// # Arguments
    ///
    /// * `findings` - Findings from all five lenses
    /// * `contract` - The contract (for accountable_human in violations)
    /// * `evaluated_at` - Timestamp for the evaluation (caller-provided for determinism)
    ///
    /// # Returns
    ///
    /// An `EvaluationResult` with the final state and confidence.
    pub fn synthesize_at(
        &self,
        findings: LensFindings,
        contract: &Contract,
        evaluated_at: DateTime<Utc>,
    ) -> EvaluationResult {
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
                evaluated_at,
                metadata: std::collections::HashMap::new(),
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
                evaluated_at,
                metadata: std::collections::HashMap::new(),
            };
        }

        // Rule 3: The Honesty Rule - Low confidence forces ESCALATE
        // If confidence < 0.4 and no lens is BLOCKED, the system admits uncertainty
        // rather than guessing. Uncertainty is a governance signal, not an error.
        if confidence < Self::LOW_CONFIDENCE_THRESHOLD {
            return EvaluationResult {
                state: State::Escalate {
                    uncertainty: format!(
                        "Low confidence ({:.0}%) - system cannot reliably assess this output",
                        confidence * 100.0
                    ),
                    decision_point: "Confidence is too low for automated decision. Human judgment required.".to_string(),
                    options: vec![
                        "Review output manually and approve if appropriate".to_string(),
                        "Request additional context to improve confidence".to_string(),
                        "Reject output and request regeneration".to_string(),
                    ],
                },
                lens_findings: findings,
                confidence,
                evaluated_at,
                metadata: std::collections::HashMap::new(),
            };
        }

        // Rule 4: Otherwise -> PROCEED
        let summary = self.build_summary(&findings);
        EvaluationResult {
            state: State::Proceed { summary },
            lens_findings: findings,
            confidence,
            evaluated_at,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Find the first BLOCKED lens and extract violation details.
    ///
    /// Per spec: "Every BLOCKED has at least one evidence citation"
    /// This method enforces the evidence invariant by synthesizing minimal
    /// evidence when a lens fails to provide it (which is a lens bug, but
    /// we must still produce valid output).
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

                let (rule_id, rule_text, evidence) = match violated_rule {
                    Some(rule) if !rule.evidence.is_empty() => {
                        // Valid BLOCKED with evidence
                        (
                            rule.rule_id.clone(),
                            rule.rule_text.clone().unwrap_or_else(|| violation.clone()),
                            rule.evidence.clone(),
                        )
                    }
                    Some(rule) => {
                        // BLOCKED with violated rule but no evidence - synthesize minimal evidence
                        // This is a lens implementation bug, but we must still meet the invariant
                        let synthetic_evidence = vec![crate::evidence::Evidence {
                            claim: format!("Rule {} was violated: {}", rule.rule_id, violation),
                            source: crate::types::EvidenceSource::Contract,
                            pointer: format!("contract.rule[{}]", rule.rule_id),
                        }];
                        (
                            rule.rule_id.clone(),
                            rule.rule_text.clone().unwrap_or_else(|| violation.clone()),
                            synthetic_evidence,
                        )
                    }
                    None => {
                        // Lens returned BLOCKED without violated rule - this is a lens bug
                        // Synthesize minimal evidence to meet the invariant
                        let synthetic_evidence = vec![crate::evidence::Evidence {
                            claim: violation.clone(),
                            source: crate::types::EvidenceSource::Output,
                            pointer: "output.content[0:0]".to_string(),
                        }];
                        (
                            format!("{:?}_VIOLATION", lens_type),
                            violation.clone(),
                            synthetic_evidence,
                        )
                    }
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
        .clamp(0.0, 1.0)
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

    // =========================================================================
    // Honesty Rule Tests (Spec Section 2.4)
    // =========================================================================

    fn low_confidence_pass_finding(lens: LensType, confidence: f64) -> LensFinding {
        LensFinding {
            lens: Some(lens),
            question_asked: None,
            state: LensState::Pass,
            rules_evaluated: vec![],
            confidence,
        }
    }

    #[test]
    fn test_honesty_rule_low_confidence_forces_escalate() {
        // All lenses pass but with confidence below threshold
        let low_conf = Synthesizer::LOW_CONFIDENCE_THRESHOLD - 0.01;
        let findings = LensFindings {
            dignity_inclusion: low_confidence_pass_finding(LensType::DignityInclusion, low_conf),
            boundaries_safety: low_confidence_pass_finding(LensType::BoundariesSafety, low_conf),
            restraint_privacy: low_confidence_pass_finding(LensType::RestraintPrivacy, low_conf),
            transparency_contestability: low_confidence_pass_finding(LensType::TransparencyContestability, low_conf),
            accountability_ownership: low_confidence_pass_finding(LensType::AccountabilityOwnership, low_conf),
        };

        let synthesizer = Synthesizer::new();
        let result = synthesizer.synthesize(findings, &test_contract());

        // Should be ESCALATE due to honesty rule, not PROCEED
        assert!(matches!(result.state, State::Escalate { .. }));

        if let State::Escalate { uncertainty, .. } = &result.state {
            assert!(uncertainty.contains("Low confidence"));
        }
    }

    #[test]
    fn test_honesty_rule_boundary_exactly_at_threshold_proceeds() {
        // Confidence exactly at 0.4 should PROCEED (threshold is strictly < 0.4)
        let at_threshold = Synthesizer::LOW_CONFIDENCE_THRESHOLD;
        let findings = LensFindings {
            dignity_inclusion: low_confidence_pass_finding(LensType::DignityInclusion, at_threshold),
            boundaries_safety: low_confidence_pass_finding(LensType::BoundariesSafety, at_threshold),
            restraint_privacy: low_confidence_pass_finding(LensType::RestraintPrivacy, at_threshold),
            transparency_contestability: low_confidence_pass_finding(LensType::TransparencyContestability, at_threshold),
            accountability_ownership: low_confidence_pass_finding(LensType::AccountabilityOwnership, at_threshold),
        };

        let synthesizer = Synthesizer::new();
        let result = synthesizer.synthesize(findings, &test_contract());

        // Exactly 0.4 should PROCEED (spec says "< 0.4" triggers escalate)
        assert!(matches!(result.state, State::Proceed { .. }));
    }

    #[test]
    fn test_honesty_rule_one_low_confidence_lens_triggers_escalate() {
        // Four lenses high confidence, one below threshold
        let mut findings = LensFindings {
            dignity_inclusion: pass_finding(LensType::DignityInclusion),
            boundaries_safety: pass_finding(LensType::BoundariesSafety),
            restraint_privacy: pass_finding(LensType::RestraintPrivacy),
            transparency_contestability: pass_finding(LensType::TransparencyContestability),
            accountability_ownership: pass_finding(LensType::AccountabilityOwnership),
        };

        // One lens with very low confidence
        findings.restraint_privacy.confidence = 0.2;

        let synthesizer = Synthesizer::new();
        let result = synthesizer.synthesize(findings, &test_contract());

        // min() confidence is 0.2 < 0.4, should ESCALATE
        assert!(matches!(result.state, State::Escalate { .. }));
        assert_eq!(result.confidence, 0.2);
    }

    #[test]
    fn test_honesty_rule_blocked_takes_priority_over_low_confidence() {
        // BLOCKED should take priority even with low confidence
        let low_conf = 0.1;
        let mut findings = LensFindings {
            dignity_inclusion: low_confidence_pass_finding(LensType::DignityInclusion, low_conf),
            boundaries_safety: blocked_finding(LensType::BoundariesSafety),
            restraint_privacy: low_confidence_pass_finding(LensType::RestraintPrivacy, low_conf),
            transparency_contestability: low_confidence_pass_finding(LensType::TransparencyContestability, low_conf),
            accountability_ownership: low_confidence_pass_finding(LensType::AccountabilityOwnership, low_conf),
        };
        findings.boundaries_safety.confidence = low_conf;

        let synthesizer = Synthesizer::new();
        let result = synthesizer.synthesize(findings, &test_contract());

        // BLOCKED takes priority over honesty rule
        assert!(matches!(result.state, State::Blocked { .. }));
    }

    #[test]
    fn test_honesty_rule_lens_escalate_takes_priority_over_low_confidence() {
        // Lens ESCALATE should take priority over honesty rule ESCALATE
        // (Both result in ESCALATE, but the reason should be from the lens)
        let low_conf = 0.2;
        let mut findings = LensFindings {
            dignity_inclusion: escalate_finding(LensType::DignityInclusion),
            boundaries_safety: low_confidence_pass_finding(LensType::BoundariesSafety, low_conf),
            restraint_privacy: low_confidence_pass_finding(LensType::RestraintPrivacy, low_conf),
            transparency_contestability: low_confidence_pass_finding(LensType::TransparencyContestability, low_conf),
            accountability_ownership: low_confidence_pass_finding(LensType::AccountabilityOwnership, low_conf),
        };
        findings.dignity_inclusion.confidence = low_conf;

        let synthesizer = Synthesizer::new();
        let result = synthesizer.synthesize(findings, &test_contract());

        assert!(matches!(result.state, State::Escalate { .. }));

        // Should cite the lens reason, not the honesty rule
        if let State::Escalate { uncertainty, .. } = &result.state {
            assert!(uncertainty.contains("Test escalation"));
            assert!(!uncertainty.contains("Low confidence"));
        }
    }

    // FIX-005 Tests: Evidence invariant for BLOCKED states

    #[test]
    fn test_blocked_always_has_evidence() {
        // Even when a lens provides BLOCKED without evidence, the synthesizer
        // must synthesize minimal evidence to meet the invariant
        let mut findings = LensFindings {
            dignity_inclusion: pass_finding(LensType::DignityInclusion),
            boundaries_safety: blocked_finding(LensType::BoundariesSafety),
            restraint_privacy: pass_finding(LensType::RestraintPrivacy),
            transparency_contestability: pass_finding(LensType::TransparencyContestability),
            accountability_ownership: pass_finding(LensType::AccountabilityOwnership),
        };

        // Simulate a lens bug: BLOCKED state but no violated rule with evidence
        findings.boundaries_safety.rules_evaluated.clear();

        let synthesizer = Synthesizer::new();
        let result = synthesizer.synthesize(findings, &test_contract());

        assert!(matches!(result.state, State::Blocked { .. }));

        if let State::Blocked { violation } = &result.state {
            // Must have evidence even when lens fails to provide it
            assert!(!violation.evidence.is_empty(), "BLOCKED must have evidence");
            // Rule ID should not be "UNKNOWN"
            assert!(!violation.rule_id.is_empty());
        }
    }

    #[test]
    fn test_blocked_with_proper_evidence_preserved() {
        use crate::evidence::Evidence;
        use crate::types::{RuleEvaluation, RuleResult, EvidenceSource};

        let findings = LensFindings {
            dignity_inclusion: pass_finding(LensType::DignityInclusion),
            boundaries_safety: LensFinding {
                lens: Some(LensType::BoundariesSafety),
                question_asked: None,
                state: LensState::Blocked {
                    violation: "PII exposed".to_string(),
                },
                rules_evaluated: vec![RuleEvaluation {
                    rule_id: "B1".to_string(),
                    rule_text: Some("PII exposure rule".to_string()),
                    result: RuleResult::Violated,
                    evidence: vec![Evidence {
                        claim: "Email found in output".to_string(),
                        source: EvidenceSource::Output,
                        pointer: "output.content[10:30]".to_string(),
                    }],
                    rationale: Some("Email detected".to_string()),
                }],
                confidence: 0.98,
            },
            restraint_privacy: pass_finding(LensType::RestraintPrivacy),
            transparency_contestability: pass_finding(LensType::TransparencyContestability),
            accountability_ownership: pass_finding(LensType::AccountabilityOwnership),
        };

        let synthesizer = Synthesizer::new();
        let result = synthesizer.synthesize(findings, &test_contract());

        if let State::Blocked { violation } = &result.state {
            assert_eq!(violation.rule_id, "B1");
            assert_eq!(violation.evidence.len(), 1);
            assert_eq!(violation.evidence[0].claim, "Email found in output");
        } else {
            panic!("Expected BLOCKED state");
        }
    }
}
