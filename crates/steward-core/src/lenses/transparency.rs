//! Transparency & Contestability Lens
//!
//! **Question**: Can the human understand why this happened and contest it?
//!
//! This lens evaluates:
//! - `acceptance.fit_criteria[]`
//! - `intent.purpose` (for alignment check)
//!
//! ## State Conditions
//!
//! | State | Condition |
//! |-------|-----------|
//! | **PASS** | Output addresses question, sources cited, assumptions visible |
//! | **ESCALATE** | Uncited claims, hidden assumptions, no contestability path |
//! | **BLOCKED** | Mandatory fit criterion violated |
//!
//! ## Key Checks
//!
//! 1. **Citation presence**: When factual claims are made, are sources cited?
//! 2. **Assumption visibility**: Are assumptions stated explicitly?
//! 3. **Contestability path**: Is there a way to challenge the output?
//! 4. **AI disclosure**: Is it clear this is AI-generated (when required)?

use lazy_static::lazy_static;
use regex::Regex;

use crate::evidence::Evidence;
use crate::types::{
    EvaluationRequest, LensFinding, LensState, LensType, RuleEvaluation, RuleResult,
};

use super::Lens;

lazy_static! {
    // Citation indicators - phrases that indicate sourcing
    static ref CITATION_PATTERNS: Regex = Regex::new(
        r"(?i)\b(according to|based on|per|as (stated|noted|documented) in|source:|reference:|from (our|the) (records|database|system|policy)|policy \d+|section \d+)\b"
    ).unwrap();

    // Uncited claim patterns - definitive statements that should have sources
    static ref CLAIM_PATTERNS: Vec<(&'static str, Regex)> = vec![
        ("factual claim", Regex::new(r"(?i)\b(studies show|research (shows|indicates|proves)|statistics (show|indicate)|data (shows|suggests)|it is (known|proven|established) that)\b").unwrap()),
        ("absolute claim", Regex::new(r"(?i)\b(always|never|everyone|no one|all|none|every|best|worst|only|guaranteed|definitely|certainly)\b").unwrap()),
        ("numeric claim", Regex::new(r"\b\d+(\.\d+)?%|\$\d+|\d+ (days|hours|minutes|years|people|users|customers)\b").unwrap()),
    ];

    // Assumption indicators - phrases that make assumptions explicit
    static ref ASSUMPTION_PATTERNS: Regex = Regex::new(
        r"(?i)\b(assuming|I assume|based on the assumption|if I understand correctly|it appears that|it seems|I believe|in my understanding)\b"
    ).unwrap();

    // Hidden assumption patterns - implicit assumptions that should be stated
    static ref HIDDEN_ASSUMPTION_PATTERNS: Vec<(&'static str, Regex)> = vec![
        ("implied capability", Regex::new(r"(?i)\b(you (can|should|will) (easily|quickly|simply)|just|simply|obviously)\b").unwrap()),
        ("implied knowledge", Regex::new(r"(?i)\b(as you (know|understand)|you're (aware|familiar)|naturally|of course)\b").unwrap()),
        ("implied preference", Regex::new(r"(?i)\b(you (want|prefer|need)|what you're looking for|the best (option|choice) for you)\b").unwrap()),
    ];

    // Contestability indicators - ways to challenge or get more info
    static ref CONTESTABILITY_PATTERNS: Regex = Regex::new(
        r"(?i)\b(if (you|this) (disagree|isn't correct|is wrong)|please (let us know|contact|reach out)|for (more|further) (information|details|clarification)|appeal|dispute|review|question|feedback)\b"
    ).unwrap();

    // AI disclosure patterns
    static ref AI_DISCLOSURE_PATTERNS: Regex = Regex::new(
        r"(?i)\b(I am (an AI|a virtual|an automated)|AI (assistant|system|generated)|automated (response|system|message)|bot|virtual (assistant|agent))\b"
    ).unwrap();

    // Next steps indicators - clear actionable guidance
    static ref NEXT_STEPS_PATTERNS: Regex = Regex::new(
        r"(?i)\b(next (steps?|actions?)|to (proceed|continue)|you (can|should|may) (now|next)|here's (what|how))\b"
    ).unwrap();

    // Question addressing indicators
    static ref ADDRESSES_QUESTION_PATTERNS: Regex = Regex::new(
        r"(?i)\b(regarding (your|the) (question|inquiry|concern)|to (answer|address) your|in response to|about your)\b"
    ).unwrap();
}

/// The Transparency & Contestability lens.
pub struct TransparencyLens;

impl TransparencyLens {
    pub fn new() -> Self {
        Self
    }

    /// Check if output contains citations.
    fn has_citations(&self, content: &str) -> bool {
        CITATION_PATTERNS.is_match(content)
    }

    /// Check for uncited claims that should have sources.
    fn check_uncited_claims(&self, content: &str) -> Vec<(String, usize, usize)> {
        let mut findings = Vec::new();

        // Only flag if there are claims but no citations
        if self.has_citations(content) {
            return findings;
        }

        for (pattern_name, regex) in CLAIM_PATTERNS.iter() {
            for m in regex.find_iter(content) {
                findings.push((pattern_name.to_string(), m.start(), m.end()));
            }
        }

        findings
    }

    /// Check if assumptions are made explicit.
    fn has_explicit_assumptions(&self, content: &str) -> bool {
        ASSUMPTION_PATTERNS.is_match(content)
    }

    /// Check for hidden assumptions.
    fn check_hidden_assumptions(&self, content: &str) -> Vec<(String, usize, usize)> {
        let mut findings = Vec::new();

        for (pattern_name, regex) in HIDDEN_ASSUMPTION_PATTERNS.iter() {
            for m in regex.find_iter(content) {
                findings.push((pattern_name.to_string(), m.start(), m.end()));
            }
        }

        findings
    }

    /// Check if contestability path exists.
    fn has_contestability(&self, content: &str) -> bool {
        CONTESTABILITY_PATTERNS.is_match(content)
    }

    /// Check for AI disclosure.
    #[allow(dead_code)] // Reserved for future transparency checks
    fn has_ai_disclosure(&self, content: &str) -> bool {
        AI_DISCLOSURE_PATTERNS.is_match(content)
    }

    /// Check for next steps guidance.
    fn has_next_steps(&self, content: &str) -> bool {
        NEXT_STEPS_PATTERNS.is_match(content)
    }

    /// Check if output addresses the question.
    fn addresses_question(&self, content: &str) -> bool {
        ADDRESSES_QUESTION_PATTERNS.is_match(content)
    }

    /// Categorize a fit criterion rule.
    fn categorize_rule(&self, rule_text: &str) -> FitCategory {
        let lower = rule_text.to_lowercase();

        if lower.contains("cite") || lower.contains("source") || lower.contains("reference") {
            FitCategory::Citation
        } else if lower.contains("address") || lower.contains("question") || lower.contains("answer") {
            FitCategory::AddressesQuestion
        } else if lower.contains("next step") || lower.contains("action") || lower.contains("clear") {
            FitCategory::NextSteps
        } else if lower.contains("accurat") || lower.contains("correct") || lower.contains("true") {
            FitCategory::Accuracy
        } else if lower.contains("languag") || lower.contains("audience") || lower.contains("appropriate") {
            FitCategory::Language
        } else {
            FitCategory::General
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum FitCategory {
    Citation,
    AddressesQuestion,
    NextSteps,
    Accuracy,
    Language,
    General,
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
        let content = &request.output.content;

        let mut rules_evaluated = Vec::new();
        let blocked_violation: Option<(String, String, Vec<Evidence>)> = None;
        let mut escalate_reason: Option<String> = None;

        // Get transparency rules (fit_criteria)
        let fit_criteria = contract.transparency_rules();

        if fit_criteria.is_empty() {
            // No explicit fit criteria - do basic transparency checks
            let uncited_claims = self.check_uncited_claims(content);
            let hidden_assumptions = self.check_hidden_assumptions(content);

            if !uncited_claims.is_empty() && !hidden_assumptions.is_empty() {
                // Multiple transparency concerns
                return LensFinding {
                    lens: Some(self.lens_type()),
                    question_asked: Some(self.question().to_string()),
                    state: LensState::Escalate {
                        reason: "Multiple transparency concerns: uncited claims and hidden assumptions".to_string(),
                    },
                    rules_evaluated: vec![RuleEvaluation {
                        rule_id: "IMPLICIT".to_string(),
                        rule_text: Some("Implicit transparency check".to_string()),
                        result: RuleResult::Uncertain,
                        evidence: vec![],
                        rationale: Some("Output may lack sufficient transparency".to_string()),
                    }],
                    confidence: 0.55,
                };
            }

            return LensFinding {
                lens: Some(self.lens_type()),
                question_asked: Some(self.question().to_string()),
                state: LensState::Pass,
                rules_evaluated: vec![],
                confidence: 0.6,
            };
        }

        // Evaluate each fit criterion
        for rule in fit_criteria {
            let category = self.categorize_rule(&rule.rule);

            match category {
                FitCategory::Citation => {
                    let uncited_claims = self.check_uncited_claims(content);
                    if !uncited_claims.is_empty() {
                        let (claim_type, start, end) = &uncited_claims[0];
                        let evidence = vec![Evidence::from_output(
                            format!("Uncited {}", claim_type),
                            *start,
                            *end,
                        )];

                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Uncertain,
                            evidence,
                            rationale: Some(format!(
                                "Found {} without citation at position {}:{}",
                                claim_type, start, end
                            )),
                        });

                        if escalate_reason.is_none() {
                            escalate_reason = Some(format!(
                                "Uncited claims detected (rule {})",
                                rule.id
                            ));
                        }
                    } else if self.has_citations(content) {
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Satisfied,
                            evidence: vec![],
                            rationale: Some("Citations present in output".to_string()),
                        });
                    } else {
                        // No claims requiring citation, no citations - neutral
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Satisfied,
                            evidence: vec![],
                            rationale: Some("No claims requiring citation detected".to_string()),
                        });
                    }
                }

                FitCategory::AddressesQuestion => {
                    if self.addresses_question(content) {
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Satisfied,
                            evidence: vec![],
                            rationale: Some("Output appears to address the question".to_string()),
                        });
                    } else {
                        // Can't definitively determine without seeing the question
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Uncertain,
                            evidence: vec![],
                            rationale: Some("Cannot confirm output addresses original question".to_string()),
                        });

                        if escalate_reason.is_none() {
                            escalate_reason = Some(format!(
                                "Cannot confirm question is addressed (rule {})",
                                rule.id
                            ));
                        }
                    }
                }

                FitCategory::NextSteps => {
                    if self.has_next_steps(content) {
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Satisfied,
                            evidence: vec![],
                            rationale: Some("Next steps guidance present".to_string()),
                        });
                    } else {
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Uncertain,
                            evidence: vec![],
                            rationale: Some("No clear next steps provided".to_string()),
                        });

                        if escalate_reason.is_none() {
                            escalate_reason = Some(format!(
                                "No clear next steps provided (rule {})",
                                rule.id
                            ));
                        }
                    }
                }

                FitCategory::Accuracy => {
                    // Accuracy is hard to verify deterministically
                    // Check for hedging language that might indicate uncertainty
                    let has_claims = !self.check_uncited_claims(content).is_empty();
                    let has_hedging = self.has_explicit_assumptions(content);

                    if has_claims && !has_hedging && !self.has_citations(content) {
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Uncertain,
                            evidence: vec![],
                            rationale: Some("Claims made without citations or hedging".to_string()),
                        });

                        if escalate_reason.is_none() {
                            escalate_reason = Some(format!(
                                "Accuracy cannot be verified (rule {})",
                                rule.id
                            ));
                        }
                    } else {
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Satisfied,
                            evidence: vec![],
                            rationale: Some("No obvious accuracy concerns".to_string()),
                        });
                    }
                }

                FitCategory::Language => {
                    // Check for inappropriate language patterns
                    // This is a basic check - could be expanded
                    let has_jargon = content.contains("TL;DR")
                        || content.contains("IMHO")
                        || content.contains("FYI");
                    let has_informal = content.contains("gonna")
                        || content.contains("wanna")
                        || content.contains("kinda");

                    if has_jargon || has_informal {
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Uncertain,
                            evidence: vec![],
                            rationale: Some("Informal language detected".to_string()),
                        });

                        if escalate_reason.is_none() {
                            escalate_reason = Some(format!(
                                "Language may not be appropriate (rule {})",
                                rule.id
                            ));
                        }
                    } else {
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Satisfied,
                            evidence: vec![],
                            rationale: Some("Language appears appropriate".to_string()),
                        });
                    }
                }

                FitCategory::General => {
                    // For general rules, check overall transparency indicators
                    let has_contestability = self.has_contestability(content);
                    let hidden_assumptions = self.check_hidden_assumptions(content);

                    if !hidden_assumptions.is_empty() && !has_contestability {
                        let (pattern, start, end) = &hidden_assumptions[0];
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Uncertain,
                            evidence: vec![Evidence::from_output(
                                format!("Hidden assumption: {}", pattern),
                                *start,
                                *end,
                            )],
                            rationale: Some("Hidden assumptions without contestability path".to_string()),
                        });

                        if escalate_reason.is_none() {
                            escalate_reason = Some(format!(
                                "Transparency concern (rule {})",
                                rule.id
                            ));
                        }
                    } else {
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Satisfied,
                            evidence: vec![],
                            rationale: Some("No transparency concerns detected".to_string()),
                        });
                    }
                }
            }
        }

        // If blocked (rare for transparency - usually escalate)
        if let Some((rule_id, rule_text, _evidence)) = blocked_violation {
            return LensFinding {
                lens: Some(self.lens_type()),
                question_asked: Some(self.question().to_string()),
                state: LensState::Blocked {
                    violation: format!("{}: {}", rule_id, rule_text),
                },
                rules_evaluated,
                confidence: 0.90,
            };
        }

        // Check for escalation
        let state = if let Some(reason) = escalate_reason {
            LensState::Escalate { reason }
        } else {
            LensState::Pass
        };

        // Calculate confidence
        let confidence = calculate_confidence(&rules_evaluated);

        LensFinding {
            lens: Some(self.lens_type()),
            question_asked: Some(self.question().to_string()),
            state,
            rules_evaluated,
            confidence,
        }
    }
}

/// Calculate confidence based on rule evaluations.
fn calculate_confidence(rules: &[RuleEvaluation]) -> f64 {
    if rules.is_empty() {
        return 0.5;
    }

    let mut confidence: f64 = 1.0;

    for rule in rules {
        match rule.result {
            RuleResult::Satisfied => {
                let penalty: f64 = match rule.evidence.len() {
                    0 => 0.04, // Slightly higher penalty - transparency is harder to verify
                    1 => 0.02,
                    _ => 0.01,
                };
                confidence -= penalty;
            }
            RuleResult::Uncertain => {
                confidence -= 0.12;
            }
            RuleResult::Violated | RuleResult::NotApplicable => {
                // BLOCKED handled separately
            }
        }
    }

    confidence.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Contract;
    use crate::types::Output;

    fn create_test_request(contract_yaml: &str, content: &str) -> EvaluationRequest {
        EvaluationRequest {
            contract: Contract::from_yaml(contract_yaml).unwrap(),
            output: Output::text(content),
            context: None,
            metadata: None,
        }
    }

    #[allow(dead_code)] // Test helper for future use
    const BASE_CONTRACT: &str = r#"
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
      rule: "Addresses the customer's actual question"
    - id: "F2"
      rule: "Information is accurate based on provided context"
    - id: "F3"
      rule: "Provides clear next steps when action is required"
    - id: "F4"
      rule: "Cites sources when making factual claims"
    - id: "F5"
      rule: "Uses language appropriate for general audience"
"#;

    #[test]
    fn test_cited_output_passes() {
        // Use a focused contract with just citation rules
        let contract = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
acceptance:
  fit_criteria:
    - id: "F4"
      rule: "Cites sources when making factual claims"
"#;

        let request = create_test_request(
            contract,
            "According to our records, your order shipped on December 18th.",
        );

        let lens = TransparencyLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass());
        assert!(finding.confidence > 0.8);
    }

    #[test]
    fn test_uncited_claims_escalates() {
        let contract = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
acceptance:
  fit_criteria:
    - id: "F4"
      rule: "Cites sources when making factual claims"
"#;

        let request = create_test_request(
            contract,
            "Studies show that 95% of customers prefer faster shipping. \
             Everyone wants their packages to arrive quickly.",
        );

        let lens = TransparencyLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_escalate());
    }

    #[test]
    fn test_next_steps_satisfied() {
        let contract = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
acceptance:
  fit_criteria:
    - id: "F3"
      rule: "Provides clear next steps"
"#;

        let request = create_test_request(
            contract,
            "Your order has been processed. Here's what you can do next: \
             track your package using the link we sent, or contact support if needed.",
        );

        let lens = TransparencyLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass());
    }

    #[test]
    fn test_no_next_steps_escalates() {
        let contract = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
acceptance:
  fit_criteria:
    - id: "F3"
      rule: "Provides clear next steps when action is required"
"#;

        let request = create_test_request(
            contract,
            "Your payment was declined.",
        );

        let lens = TransparencyLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_escalate());
    }

    #[test]
    fn test_informal_language_escalates() {
        let contract = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
acceptance:
  fit_criteria:
    - id: "F5"
      rule: "Uses language appropriate for general audience"
"#;

        let request = create_test_request(
            contract,
            "Hey! TL;DR - your order is gonna arrive soon, kinda like tomorrow or something.",
        );

        let lens = TransparencyLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_escalate());
    }

    #[test]
    fn test_addresses_question_passes() {
        // Use a focused contract for testing question addressing
        let contract = r#"
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
      rule: "Addresses the customer's actual question"
"#;

        let request = create_test_request(
            contract,
            "To answer your question about the return policy: \
             You have 30 days from the date of purchase to return items.",
        );

        let lens = TransparencyLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass());
    }

    #[test]
    fn test_hidden_assumptions_escalates() {
        let contract = r#"
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
      rule: "General transparency"
"#;

        let request = create_test_request(
            contract,
            "You can easily just go to the settings and obviously find what you're looking for. \
             As you know, this is the best option for you.",
        );

        let lens = TransparencyLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_escalate());
    }

    #[test]
    fn test_no_fit_criteria_moderate_confidence() {
        let contract = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
"#;

        let request = create_test_request(
            contract,
            "Your order will arrive tomorrow.",
        );

        let lens = TransparencyLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass());
        assert!(finding.confidence < 0.7); // Lower confidence without explicit rules
    }
}
