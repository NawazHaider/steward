//! Boundaries & Safety Lens
//!
//! **Question**: Does this respect defined scope and stop conditions?
//!
//! This is the primary lens for Phase 1, implementing full pattern matching
//! for boundary violations including PII detection.

use lazy_static::lazy_static;
use regex::Regex;

use crate::evidence::Evidence;
use crate::types::{
    EvaluationRequest, LensFinding, LensState, LensType, RuleEvaluation, RuleResult,
};

use super::Lens;

lazy_static! {
    // PII Detection Patterns
    static ref EMAIL_PATTERN: Regex = Regex::new(
        r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"
    ).unwrap();

    static ref PHONE_PATTERN: Regex = Regex::new(
        r"(?:\+?1[-.\s]?)?(?:\([0-9]{3}\)|[0-9]{3})[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}"
    ).unwrap();

    static ref SSN_PATTERN: Regex = Regex::new(
        r"\b\d{3}[-\s]?\d{2}[-\s]?\d{4}\b"
    ).unwrap();

    static ref CREDIT_CARD_PATTERN: Regex = Regex::new(
        r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b"
    ).unwrap();

    // API Key / Secret Patterns
    static ref API_KEY_PATTERN: Regex = Regex::new(
        r#"(?i)(api[_-]?key|secret|token|password|auth)[\s:=]+['"]?[a-zA-Z0-9_-]{20,}['"]?"#
    ).unwrap();

    // Keywords for escalation conditions
    static ref FRUSTRATION_KEYWORDS: Vec<&'static str> = vec![
        "frustrated", "angry", "furious", "annoyed", "upset",
        "terrible", "awful", "horrible", "worst", "unacceptable",
        "ridiculous", "outrageous", "disgusted", "fed up"
    ];

    static ref LEGAL_KEYWORDS: Vec<&'static str> = vec![
        "lawyer", "attorney", "legal", "lawsuit", "sue",
        "court", "litigation", "regulatory", "complaint", "law"
    ];

    static ref MEDICAL_KEYWORDS: Vec<&'static str> = vec![
        "symptom", "doctor", "hospital", "medicine", "medical",
        "diagnosis", "treatment", "prescription", "health", "sick"
    ];

    static ref FINANCIAL_ADVICE_KEYWORDS: Vec<&'static str> = vec![
        "invest", "investment", "stock", "portfolio", "trading",
        "financial advice", "should i buy", "financial plan"
    ];
}

/// The Boundaries & Safety lens.
pub struct BoundariesLens;

impl BoundariesLens {
    pub fn new() -> Self {
        Self
    }

    /// Check if output contains PII.
    fn check_pii(&self, content: &str) -> Vec<(String, usize, usize)> {
        let mut findings = Vec::new();

        // Check for emails
        for m in EMAIL_PATTERN.find_iter(content) {
            findings.push(("Email address".to_string(), m.start(), m.end()));
        }

        // Check for phone numbers
        for m in PHONE_PATTERN.find_iter(content) {
            findings.push(("Phone number".to_string(), m.start(), m.end()));
        }

        // Check for SSNs
        for m in SSN_PATTERN.find_iter(content) {
            findings.push(("Social Security Number".to_string(), m.start(), m.end()));
        }

        // Check for credit cards
        for m in CREDIT_CARD_PATTERN.find_iter(content) {
            findings.push(("Credit card number".to_string(), m.start(), m.end()));
        }

        findings
    }

    /// Check if output contains API keys or secrets.
    fn check_secrets(&self, content: &str) -> Vec<(String, usize, usize)> {
        let mut findings = Vec::new();

        for m in API_KEY_PATTERN.find_iter(content) {
            findings.push(("API key or secret".to_string(), m.start(), m.end()));
        }

        findings
    }

    /// Check for keyword matches in content.
    fn check_keywords(&self, content: &str, keywords: &[&str]) -> bool {
        let lower = content.to_lowercase();
        keywords.iter().any(|kw| lower.contains(kw))
    }

    /// Check for specific phrase matches.
    fn check_phrase(&self, content: &str, phrase: &str) -> Option<(usize, usize)> {
        let lower_content = content.to_lowercase();
        let lower_phrase = phrase.to_lowercase();

        lower_content.find(&lower_phrase).map(|start| {
            (start, start + phrase.len())
        })
    }
}

impl Default for BoundariesLens {
    fn default() -> Self {
        Self::new()
    }
}

impl Lens for BoundariesLens {
    fn lens_type(&self) -> LensType {
        LensType::BoundariesSafety
    }

    fn evaluate(&self, request: &EvaluationRequest) -> LensFinding {
        let contract = &request.contract;
        let content = &request.output.content;
        let context = request.context.as_deref().unwrap_or(&[]);

        let mut rules_evaluated = Vec::new();
        let mut blocked_violation: Option<(String, String, Vec<Evidence>)> = None;
        let mut escalate_reason: Option<String> = None;

        // 1. Check invalidated_by rules (most severe - triggers BLOCKED)
        for rule in &contract.boundaries.invalidated_by {
            let rule_lower = rule.rule.to_lowercase();

            // PII exposure check
            if rule_lower.contains("pii") || rule_lower.contains("personal") {
                let pii_found = self.check_pii(content);
                if !pii_found.is_empty() {
                    let (pii_type, start, end) = &pii_found[0];
                    let evidence = vec![Evidence::from_output(
                        format!("{} exposed in response", pii_type),
                        *start,
                        *end,
                    )];

                    rules_evaluated.push(RuleEvaluation {
                        rule_id: rule.id.clone(),
                        rule_text: Some(rule.rule.clone()),
                        result: RuleResult::Violated,
                        evidence: evidence.clone(),
                        rationale: Some(format!("{} found at position {}:{}", pii_type, start, end)),
                    });

                    blocked_violation = Some((rule.id.clone(), rule.rule.clone(), evidence));
                    break;
                }
            }

            // Credential/secret exposure check
            if rule_lower.contains("credential") || rule_lower.contains("secret") {
                let secrets_found = self.check_secrets(content);
                if !secrets_found.is_empty() {
                    let (secret_type, start, end) = &secrets_found[0];
                    let evidence = vec![Evidence::from_output(
                        format!("{} exposed", secret_type),
                        *start,
                        *end,
                    )];

                    rules_evaluated.push(RuleEvaluation {
                        rule_id: rule.id.clone(),
                        rule_text: Some(rule.rule.clone()),
                        result: RuleResult::Violated,
                        evidence: evidence.clone(),
                        rationale: Some(format!("{} found at position {}:{}", secret_type, start, end)),
                    });

                    blocked_violation = Some((rule.id.clone(), rule.rule.clone(), evidence));
                    break;
                }
            }

            // Medical/legal/financial advice check
            if rule_lower.contains("medical") && rule_lower.contains("advice") {
                if self.check_keywords(content, &MEDICAL_KEYWORDS) {
                    let evidence = vec![Evidence::from_output(
                        "Medical-related content detected",
                        0,
                        content.len().min(100),
                    )];

                    rules_evaluated.push(RuleEvaluation {
                        rule_id: rule.id.clone(),
                        rule_text: Some(rule.rule.clone()),
                        result: RuleResult::Uncertain,
                        evidence: evidence.clone(),
                        rationale: Some("Content may contain medical advice".to_string()),
                    });

                    // This is uncertain, so escalate rather than block
                    if escalate_reason.is_none() {
                        escalate_reason = Some(format!(
                            "Possible medical content detected (rule {})",
                            rule.id
                        ));
                    }
                }
            }

            // If not matched, mark as satisfied
            if blocked_violation.is_none() && escalate_reason.is_none() {
                rules_evaluated.push(RuleEvaluation {
                    rule_id: rule.id.clone(),
                    rule_text: Some(rule.rule.clone()),
                    result: RuleResult::Satisfied,
                    evidence: vec![],
                    rationale: None,
                });
            }
        }

        // If already blocked, return immediately
        if let Some((rule_id, rule_text, _evidence)) = blocked_violation {
            return LensFinding {
                lens: Some(LensType::BoundariesSafety),
                question_asked: Some(self.question().to_string()),
                state: LensState::Blocked {
                    violation: format!("{}: {}", rule_id, rule_text),
                },
                rules_evaluated,
                confidence: 0.98, // High confidence for pattern-matched violations
            };
        }

        // 2. Check must_escalate_when rules
        for rule in &contract.boundaries.must_escalate_when {
            let rule_lower = rule.rule.to_lowercase();

            // Legal/compliance topic
            if rule_lower.contains("legal") || rule_lower.contains("compliance") {
                // Check both output and context
                let has_legal = self.check_keywords(content, &LEGAL_KEYWORDS)
                    || context.iter().any(|c| self.check_keywords(c, &LEGAL_KEYWORDS));

                if has_legal {
                    rules_evaluated.push(RuleEvaluation {
                        rule_id: rule.id.clone(),
                        rule_text: Some(rule.rule.clone()),
                        result: RuleResult::Uncertain,
                        evidence: vec![],
                        rationale: Some("Legal/compliance topic detected".to_string()),
                    });

                    if escalate_reason.is_none() {
                        escalate_reason = Some(format!(
                            "Legal/compliance topic detected (rule {})",
                            rule.id
                        ));
                    }
                }
            }

            // Customer requests human
            if rule_lower.contains("human") && rule_lower.contains("request") {
                let human_phrases = ["talk to a human", "speak to a person", "real person",
                                     "human agent", "customer service", "speak to someone"];

                for phrase in human_phrases {
                    // Check context for human request
                    for (idx, ctx) in context.iter().enumerate() {
                        if let Some((start, end)) = self.check_phrase(ctx, phrase) {
                            rules_evaluated.push(RuleEvaluation {
                                rule_id: rule.id.clone(),
                                rule_text: Some(rule.rule.clone()),
                                result: RuleResult::Uncertain,
                                evidence: vec![Evidence::from_context(
                                    "Customer requested human agent",
                                    idx,
                                    start,
                                    end,
                                )],
                                rationale: Some("Customer explicitly requested human".to_string()),
                            });

                            if escalate_reason.is_none() {
                                escalate_reason = Some(format!(
                                    "Customer explicitly requested human agent (rule {})",
                                    rule.id
                                ));
                            }
                            break;
                        }
                    }
                }
            }
        }

        // 3. Check must_pause_when rules
        for rule in &contract.boundaries.must_pause_when {
            let rule_lower = rule.rule.to_lowercase();

            // Customer frustration
            if rule_lower.contains("frustrat") || rule_lower.contains("anger") {
                for (idx, ctx) in context.iter().enumerate() {
                    if self.check_keywords(ctx, &FRUSTRATION_KEYWORDS) {
                        rules_evaluated.push(RuleEvaluation {
                            rule_id: rule.id.clone(),
                            rule_text: Some(rule.rule.clone()),
                            result: RuleResult::Uncertain,
                            evidence: vec![Evidence::from_context(
                                "Customer frustration detected",
                                idx,
                                0,
                                ctx.len().min(50),
                            )],
                            rationale: Some("Frustration keywords detected in context".to_string()),
                        });

                        if escalate_reason.is_none() {
                            escalate_reason = Some(format!(
                                "Customer frustration detected (rule {})",
                                rule.id
                            ));
                        }
                        break;
                    }
                }
            }
        }

        // Build final finding
        let state = if let Some(reason) = escalate_reason {
            LensState::Escalate { reason }
        } else {
            LensState::Pass
        };

        // Calculate confidence based on evidence quality
        let confidence = calculate_confidence(&rules_evaluated);

        LensFinding {
            lens: Some(LensType::BoundariesSafety),
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
        return 0.5; // Default when no rules apply
    }

    let mut confidence: f64 = 1.0;

    for rule in rules {
        match rule.result {
            RuleResult::Satisfied => {
                let penalty: f64 = match rule.evidence.len() {
                    0 => 0.05,
                    1 => 0.02,
                    _ => 0.01,
                };
                confidence -= penalty;
            }
            RuleResult::Uncertain => {
                confidence -= 0.15;
            }
            RuleResult::Violated | RuleResult::NotApplicable => {
                // No penalty for these - BLOCKED handled separately
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

    #[test]
    fn test_email_detection() {
        let contract = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
boundaries:
  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed in response"
accountability:
  answerable_human: "test@example.com"
"#;

        let request = create_test_request(
            contract,
            "Contact john.doe@email.com for more info.",
        );

        let lens = BoundariesLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_blocked());
        assert!(finding.rules_evaluated.iter().any(|r| r.rule_id == "B1"));
    }

    #[test]
    fn test_phone_detection() {
        let contract = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
boundaries:
  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed"
accountability:
  answerable_human: "test@example.com"
"#;

        let request = create_test_request(
            contract,
            "Call us at (555) 123-4567.",
        );

        let lens = BoundariesLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_blocked());
    }

    #[test]
    fn test_no_pii_passes() {
        let contract = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
boundaries:
  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed"
accountability:
  answerable_human: "test@example.com"
"#;

        let request = create_test_request(
            contract,
            "Your order will arrive tomorrow.",
        );

        let lens = BoundariesLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass());
    }

    #[test]
    fn test_frustration_escalation() {
        let contract = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
boundaries:
  must_pause_when:
    - id: "P1"
      rule: "Customer expresses frustration"
accountability:
  answerable_human: "test@example.com"
"#;

        let mut request = create_test_request(
            contract,
            "I understand your concern.",
        );
        request.context = Some(vec!["I'm so frustrated with this service!".to_string()]);

        let lens = BoundariesLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_escalate());
    }
}
