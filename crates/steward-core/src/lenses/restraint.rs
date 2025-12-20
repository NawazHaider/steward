//! Restraint & Privacy Lens
//!
//! **Question**: Does this expose what should be protected?
//!
//! This lens evaluates:
//! - `boundaries.invalidated_by[]` (privacy-related rules only)
//! - `intent.never_optimize_away[]` (privacy-related rules only)
//!
//! ## State Conditions
//!
//! | State | Condition |
//! |-------|-----------|
//! | **PASS** | No protected data exposed, scope respected |
//! | **ESCALATE** | Potential scope creep, data minimization concern |
//! | **BLOCKED** | PII exposed, credentials exposed, explicit privacy violation |
//!
//! ## Built-in Patterns
//!
//! | Category | Pattern |
//! |----------|---------|
//! | Email | RFC 5322 compliant patterns |
//! | Phone | International and US formats |
//! | SSN | US Social Security Number format |
//! | Credit Card | Major card formats (Visa, MC, Amex, etc.) |
//! | API Key | Common API key and secret patterns |

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

    // Address pattern (basic - street address)
    static ref ADDRESS_PATTERN: Regex = Regex::new(
        r"\b\d{1,5}\s+[A-Za-z]+\s+(Street|St|Avenue|Ave|Boulevard|Blvd|Road|Rd|Drive|Dr|Lane|Ln|Way|Court|Ct)\b"
    ).unwrap();

    // Date of birth pattern (common formats)
    static ref DOB_PATTERN: Regex = Regex::new(
        r"(?i)\b(date of birth|dob|born on|birthday)[:\s]+\d{1,2}[/-]\d{1,2}[/-]\d{2,4}\b"
    ).unwrap();

    // API Key / Secret Patterns
    static ref API_KEY_PATTERN: Regex = Regex::new(
        r#"(?i)(api[_-]?key|secret[_-]?key|access[_-]?token|auth[_-]?token|bearer|password)[\s:=]+['"]?[a-zA-Z0-9_-]{16,}['"]?"#
    ).unwrap();

    // AWS-style credentials
    static ref AWS_KEY_PATTERN: Regex = Regex::new(
        r"(?i)(AKIA|ABIA|ACCA|AGPA|AIDA|AIPA|ANPA|ANVA|AROA|ASCA|ASIA)[A-Z0-9]{16}"
    ).unwrap();

    // Private key patterns
    static ref PRIVATE_KEY_PATTERN: Regex = Regex::new(
        r"-----BEGIN (RSA |DSA |EC |OPENSSH )?PRIVATE KEY-----"
    ).unwrap();

    // Database connection strings
    static ref DB_CONNECTION_PATTERN: Regex = Regex::new(
        r"(?i)(mongodb|mysql|postgres|postgresql|redis)://[^\s]+"
    ).unwrap();

    // Scope creep indicators - accessing data beyond stated purpose
    static ref SCOPE_CREEP_PATTERNS: Vec<(&'static str, Regex)> = vec![
        ("external API call", Regex::new(r"(?i)(fetched from|retrieved from|called|accessed)\s+(external|third[- ]party|api\.|\w+\.com)").unwrap()),
        ("database query", Regex::new(r"(?i)(SELECT|INSERT|UPDATE|DELETE)\s+.*(FROM|INTO|SET)").unwrap()),
        ("file access", Regex::new(r"(?i)(read|wrote|opened|accessed)\s+(file|document|record)").unwrap()),
    ];

    // Data minimization violations - unnecessary data included
    static ref EXCESS_DATA_PATTERNS: Vec<(&'static str, Regex)> = vec![
        ("full records", Regex::new(r"(?i)(all|complete|full|entire)\s+(records?|data|information|history)").unwrap()),
        ("detailed logs", Regex::new(r"(?i)(detailed|complete|full)\s+(logs?|audit trail|history)").unwrap()),
    ];
}

/// The Restraint & Privacy lens.
pub struct RestraintLens;

impl RestraintLens {
    pub fn new() -> Self {
        Self
    }

    /// Check if output contains PII.
    fn check_pii(&self, content: &str) -> Vec<PiiMatch> {
        let mut findings = Vec::new();

        // Check for emails
        for m in EMAIL_PATTERN.find_iter(content) {
            findings.push(PiiMatch {
                pii_type: PiiType::Email,
                value: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
            });
        }

        // Check for phone numbers
        for m in PHONE_PATTERN.find_iter(content) {
            findings.push(PiiMatch {
                pii_type: PiiType::Phone,
                value: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
            });
        }

        // Check for SSNs
        for m in SSN_PATTERN.find_iter(content) {
            findings.push(PiiMatch {
                pii_type: PiiType::Ssn,
                value: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
            });
        }

        // Check for credit cards
        for m in CREDIT_CARD_PATTERN.find_iter(content) {
            // Basic Luhn check could go here in production
            findings.push(PiiMatch {
                pii_type: PiiType::CreditCard,
                value: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
            });
        }

        // Check for addresses
        for m in ADDRESS_PATTERN.find_iter(content) {
            findings.push(PiiMatch {
                pii_type: PiiType::Address,
                value: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
            });
        }

        // Check for DOB
        for m in DOB_PATTERN.find_iter(content) {
            findings.push(PiiMatch {
                pii_type: PiiType::DateOfBirth,
                value: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
            });
        }

        findings
    }

    /// Check if output contains secrets or credentials.
    fn check_secrets(&self, content: &str) -> Vec<SecretMatch> {
        let mut findings = Vec::new();

        // Check for API keys
        for m in API_KEY_PATTERN.find_iter(content) {
            findings.push(SecretMatch {
                secret_type: SecretType::ApiKey,
                start: m.start(),
                end: m.end(),
            });
        }

        // Check for AWS credentials
        for m in AWS_KEY_PATTERN.find_iter(content) {
            findings.push(SecretMatch {
                secret_type: SecretType::AwsKey,
                start: m.start(),
                end: m.end(),
            });
        }

        // Check for private keys
        for m in PRIVATE_KEY_PATTERN.find_iter(content) {
            findings.push(SecretMatch {
                secret_type: SecretType::PrivateKey,
                start: m.start(),
                end: m.end(),
            });
        }

        // Check for database connection strings
        for m in DB_CONNECTION_PATTERN.find_iter(content) {
            findings.push(SecretMatch {
                secret_type: SecretType::DbConnection,
                start: m.start(),
                end: m.end(),
            });
        }

        findings
    }

    /// Check for scope creep indicators.
    fn check_scope_creep(&self, content: &str) -> Vec<(String, usize, usize)> {
        let mut findings = Vec::new();

        for (pattern_name, regex) in SCOPE_CREEP_PATTERNS.iter() {
            for m in regex.find_iter(content) {
                findings.push((pattern_name.to_string(), m.start(), m.end()));
            }
        }

        findings
    }

    /// Check for data minimization violations.
    fn check_excess_data(&self, content: &str) -> Vec<(String, usize, usize)> {
        let mut findings = Vec::new();

        for (pattern_name, regex) in EXCESS_DATA_PATTERNS.iter() {
            for m in regex.find_iter(content) {
                findings.push((pattern_name.to_string(), m.start(), m.end()));
            }
        }

        findings
    }

    /// Determine if a rule is PII-related.
    fn is_pii_rule(&self, rule_text: &str) -> bool {
        let lower = rule_text.to_lowercase();
        lower.contains("pii")
            || lower.contains("personal")
            || lower.contains("email")
            || lower.contains("phone")
            || lower.contains("ssn")
            || lower.contains("social security")
            || lower.contains("credit card")
            || lower.contains("address")
    }

    /// Determine if a rule is credential-related.
    fn is_credential_rule(&self, rule_text: &str) -> bool {
        let lower = rule_text.to_lowercase();
        lower.contains("credential")
            || lower.contains("secret")
            || lower.contains("api key")
            || lower.contains("password")
            || lower.contains("token")
            || lower.contains("private key")
    }

    /// Determine if a rule is privacy-related.
    fn is_privacy_rule(&self, rule_text: &str) -> bool {
        let lower = rule_text.to_lowercase();
        lower.contains("privacy")
            || lower.contains("data")
            || lower.contains("scope")
            || lower.contains("expose")
            || lower.contains("protect")
    }
}

#[derive(Debug, Clone)]
struct PiiMatch {
    pii_type: PiiType,
    value: String,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Copy)]
enum PiiType {
    Email,
    Phone,
    Ssn,
    CreditCard,
    Address,
    DateOfBirth,
}

impl PiiType {
    fn as_str(&self) -> &'static str {
        match self {
            PiiType::Email => "Email address",
            PiiType::Phone => "Phone number",
            PiiType::Ssn => "Social Security Number",
            PiiType::CreditCard => "Credit card number",
            PiiType::Address => "Street address",
            PiiType::DateOfBirth => "Date of birth",
        }
    }
}

#[derive(Debug, Clone)]
struct SecretMatch {
    secret_type: SecretType,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Copy)]
enum SecretType {
    ApiKey,
    AwsKey,
    PrivateKey,
    DbConnection,
}

impl SecretType {
    fn as_str(&self) -> &'static str {
        match self {
            SecretType::ApiKey => "API key or token",
            SecretType::AwsKey => "AWS access key",
            SecretType::PrivateKey => "Private key",
            SecretType::DbConnection => "Database connection string",
        }
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
        let content = &request.output.content;

        let mut rules_evaluated = Vec::new();
        let mut blocked_violation: Option<(String, String, Vec<Evidence>)> = None;
        let mut escalate_reason: Option<String> = None;

        // Get restraint/privacy rules
        let restraint_rules = contract.restraint_rules();

        if restraint_rules.is_empty() {
            // No explicit privacy rules - still check for obvious violations
            let pii_found = self.check_pii(content);
            let secrets_found = self.check_secrets(content);

            if !pii_found.is_empty() || !secrets_found.is_empty() {
                // Even without explicit rules, PII/secrets exposure should escalate
                let evidence = if !pii_found.is_empty() {
                    let pii = &pii_found[0];
                    vec![Evidence::from_output(
                        format!("{} detected", pii.pii_type.as_str()),
                        pii.start,
                        pii.end,
                    )]
                } else {
                    let secret = &secrets_found[0];
                    vec![Evidence::from_output(
                        format!("{} detected", secret.secret_type.as_str()),
                        secret.start,
                        secret.end,
                    )]
                };

                return LensFinding {
                    lens: Some(self.lens_type()),
                    question_asked: Some(self.question().to_string()),
                    state: LensState::Escalate {
                        reason: "Potential sensitive data detected without explicit privacy rules".to_string(),
                    },
                    rules_evaluated: vec![RuleEvaluation {
                        rule_id: "IMPLICIT".to_string(),
                        rule_text: Some("Implicit privacy check".to_string()),
                        result: RuleResult::Uncertain,
                        evidence,
                        rationale: Some("Sensitive data pattern detected".to_string()),
                    }],
                    confidence: 0.65,
                };
            }

            return LensFinding {
                lens: Some(self.lens_type()),
                question_asked: Some(self.question().to_string()),
                state: LensState::Pass,
                rules_evaluated: vec![],
                confidence: 0.6, // Lower confidence when no explicit rules
            };
        }

        // Evaluate each restraint rule
        for rule in restraint_rules {
            // PII-related rules
            if self.is_pii_rule(&rule.rule) {
                let pii_found = self.check_pii(content);
                if !pii_found.is_empty() {
                    let pii = &pii_found[0];
                    let evidence = vec![Evidence::from_output(
                        format!("{} exposed in response", pii.pii_type.as_str()),
                        pii.start,
                        pii.end,
                    )];

                    rules_evaluated.push(RuleEvaluation {
                        rule_id: rule.id.clone(),
                        rule_text: Some(rule.rule.clone()),
                        result: RuleResult::Violated,
                        evidence: evidence.clone(),
                        rationale: Some(format!(
                            "{} found at position {}:{}",
                            pii.pii_type.as_str(),
                            pii.start,
                            pii.end
                        )),
                    });

                    if blocked_violation.is_none() {
                        blocked_violation = Some((rule.id.clone(), rule.rule.clone(), evidence));
                    }
                } else {
                    rules_evaluated.push(RuleEvaluation {
                        rule_id: rule.id.clone(),
                        rule_text: Some(rule.rule.clone()),
                        result: RuleResult::Satisfied,
                        evidence: vec![],
                        rationale: Some("No PII detected in output".to_string()),
                    });
                }
            }
            // Credential-related rules
            else if self.is_credential_rule(&rule.rule) {
                let secrets_found = self.check_secrets(content);
                if !secrets_found.is_empty() {
                    let secret = &secrets_found[0];
                    let evidence = vec![Evidence::from_output(
                        format!("{} exposed", secret.secret_type.as_str()),
                        secret.start,
                        secret.end,
                    )];

                    rules_evaluated.push(RuleEvaluation {
                        rule_id: rule.id.clone(),
                        rule_text: Some(rule.rule.clone()),
                        result: RuleResult::Violated,
                        evidence: evidence.clone(),
                        rationale: Some(format!(
                            "{} found at position {}:{}",
                            secret.secret_type.as_str(),
                            secret.start,
                            secret.end
                        )),
                    });

                    if blocked_violation.is_none() {
                        blocked_violation = Some((rule.id.clone(), rule.rule.clone(), evidence));
                    }
                } else {
                    rules_evaluated.push(RuleEvaluation {
                        rule_id: rule.id.clone(),
                        rule_text: Some(rule.rule.clone()),
                        result: RuleResult::Satisfied,
                        evidence: vec![],
                        rationale: Some("No credentials detected in output".to_string()),
                    });
                }
            }
            // General privacy rules
            else if self.is_privacy_rule(&rule.rule) {
                // Check for scope creep
                let scope_creep = self.check_scope_creep(content);
                let excess_data = self.check_excess_data(content);

                if !scope_creep.is_empty() {
                    let (pattern, start, end) = &scope_creep[0];
                    let evidence = vec![Evidence::from_output(
                        format!("Potential scope creep: {}", pattern),
                        *start,
                        *end,
                    )];

                    rules_evaluated.push(RuleEvaluation {
                        rule_id: rule.id.clone(),
                        rule_text: Some(rule.rule.clone()),
                        result: RuleResult::Uncertain,
                        evidence,
                        rationale: Some(format!("Scope creep pattern '{}' detected", pattern)),
                    });

                    if escalate_reason.is_none() {
                        escalate_reason = Some(format!(
                            "Potential scope creep detected (rule {})",
                            rule.id
                        ));
                    }
                } else if !excess_data.is_empty() {
                    let (pattern, start, end) = &excess_data[0];
                    let evidence = vec![Evidence::from_output(
                        format!("Potential data minimization issue: {}", pattern),
                        *start,
                        *end,
                    )];

                    rules_evaluated.push(RuleEvaluation {
                        rule_id: rule.id.clone(),
                        rule_text: Some(rule.rule.clone()),
                        result: RuleResult::Uncertain,
                        evidence,
                        rationale: Some(format!(
                            "Data minimization concern '{}' detected",
                            pattern
                        )),
                    });

                    if escalate_reason.is_none() {
                        escalate_reason = Some(format!(
                            "Data minimization concern (rule {})",
                            rule.id
                        ));
                    }
                } else {
                    rules_evaluated.push(RuleEvaluation {
                        rule_id: rule.id.clone(),
                        rule_text: Some(rule.rule.clone()),
                        result: RuleResult::Satisfied,
                        evidence: vec![],
                        rationale: Some("No privacy concerns detected".to_string()),
                    });
                }
            } else {
                // Rule type not recognized - mark as satisfied with uncertainty
                rules_evaluated.push(RuleEvaluation {
                    rule_id: rule.id.clone(),
                    rule_text: Some(rule.rule.clone()),
                    result: RuleResult::Satisfied,
                    evidence: vec![],
                    rationale: Some("Rule type not matched to specific check".to_string()),
                });
            }
        }

        // If blocked, return immediately
        if let Some((rule_id, rule_text, _evidence)) = blocked_violation {
            return LensFinding {
                lens: Some(self.lens_type()),
                question_asked: Some(self.question().to_string()),
                state: LensState::Blocked {
                    violation: format!("{}: {}", rule_id, rule_text),
                },
                rules_evaluated,
                confidence: 0.98, // Very high confidence for pattern-matched PII/secrets
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
                    0 => 0.03, // Lower penalty for privacy - absence of evidence is good
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

    const BASE_CONTRACT: &str = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
  never_optimize_away:
    - id: "N1"
      rule: "Privacy of customer data"
boundaries:
  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed in response"
    - id: "B5"
      rule: "Internal system credentials or secrets exposed"
accountability:
  answerable_human: "test@example.com"
"#;

    #[test]
    fn test_email_pii_blocked() {
        let request = create_test_request(
            BASE_CONTRACT,
            "The customer's email is john.doe@example.com. Please contact them.",
        );

        let lens = RestraintLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_blocked());
        assert!(finding.confidence > 0.9);
    }

    #[test]
    fn test_phone_pii_blocked() {
        let request = create_test_request(
            BASE_CONTRACT,
            "You can reach them at (555) 123-4567.",
        );

        let lens = RestraintLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_blocked());
    }

    #[test]
    fn test_ssn_pii_blocked() {
        let request = create_test_request(
            BASE_CONTRACT,
            "Their SSN is 123-45-6789.",
        );

        let lens = RestraintLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_blocked());
    }

    #[test]
    fn test_api_key_blocked() {
        let request = create_test_request(
            BASE_CONTRACT,
            "Use this api_key: sk-abcdefghijklmnopqrstuvwxyz123456",
        );

        let lens = RestraintLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_blocked());
    }

    #[test]
    fn test_no_pii_passes() {
        let request = create_test_request(
            BASE_CONTRACT,
            "Your order has shipped and will arrive by Thursday.",
        );

        let lens = RestraintLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_pass());
        assert!(finding.confidence > 0.8);
    }

    #[test]
    fn test_scope_creep_escalates() {
        let contract = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
  never_optimize_away:
    - id: "N1"
      rule: "Data privacy and scope limits"
accountability:
  answerable_human: "test@example.com"
"#;

        let request = create_test_request(
            contract,
            "I retrieved from external api.thirdparty.com to get additional information.",
        );

        let lens = RestraintLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_escalate());
    }

    #[test]
    fn test_implicit_pii_check_escalates() {
        // Contract without explicit PII rules should still flag PII
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
            "Contact john@example.com for details.",
        );

        let lens = RestraintLens::new();
        let finding = lens.evaluate(&request);

        // Should escalate even without explicit rules
        assert!(finding.state.is_escalate());
    }

    #[test]
    fn test_credit_card_blocked() {
        let request = create_test_request(
            BASE_CONTRACT,
            "The card number is 4111-1111-1111-1111.",
        );

        let lens = RestraintLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_blocked());
    }

    #[test]
    fn test_aws_key_blocked() {
        let request = create_test_request(
            BASE_CONTRACT,
            "The access key is AKIAIOSFODNN7EXAMPLE.",
        );

        let lens = RestraintLens::new();
        let finding = lens.evaluate(&request);

        assert!(finding.state.is_blocked());
    }
}
