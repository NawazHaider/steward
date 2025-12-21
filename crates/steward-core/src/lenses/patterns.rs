//! Shared detection patterns for lenses.
//!
//! This module contains regex patterns used by multiple lenses to detect
//! PII (Personally Identifiable Information) and credentials in output content.
//!
//! ## SOLID Rationale
//!
//! These patterns are shared between `BoundariesLens` and `RestraintLens`:
//! - **DRY**: Single source of truth for detection patterns
//! - **OCP**: Add new patterns here without modifying lens logic
//! - **SRP**: Pattern definition is separate from pattern usage

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // =========================================================================
    // PII DETECTION PATTERNS
    // =========================================================================

    /// Email address pattern (RFC 5322 simplified)
    pub static ref EMAIL_PATTERN: Regex = Regex::new(
        r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"
    ).unwrap();

    /// US phone number pattern (with optional country code)
    pub static ref PHONE_PATTERN: Regex = Regex::new(
        r"(?:\+?1[-.\s]?)?(?:\([0-9]{3}\)|[0-9]{3})[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}"
    ).unwrap();

    /// Social Security Number pattern (XXX-XX-XXXX or XXXXXXXXX)
    pub static ref SSN_PATTERN: Regex = Regex::new(
        r"\b\d{3}[-\s]?\d{2}[-\s]?\d{4}\b"
    ).unwrap();

    /// Credit card number pattern (16 digits with optional separators)
    pub static ref CREDIT_CARD_PATTERN: Regex = Regex::new(
        r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b"
    ).unwrap();

    // =========================================================================
    // CREDENTIAL DETECTION PATTERNS
    // =========================================================================

    /// API key / secret / token pattern
    /// Matches: api_key=xxx, secret: xxx, token=xxx, etc.
    pub static ref API_KEY_PATTERN: Regex = Regex::new(
        r#"(?i)(api[_-]?key|secret[_-]?key|access[_-]?token|auth[_-]?token|bearer|password|secret|token)[\s:=]+['"]?[a-zA-Z0-9_-]{16,}['"]?"#
    ).unwrap();

    /// AWS access key pattern (all AWS key prefixes)
    /// Includes: AKIA (user), ABIA (STS), ACCA (catalog), AGPA (group),
    /// AIDA (IAM user), AIPA (EC2), ANPA (managed policy), ANVA (version),
    /// AROA (role), ASCA (cert), ASIA (temp STS)
    pub static ref AWS_KEY_PATTERN: Regex = Regex::new(
        r"(?i)(AKIA|ABIA|ACCA|AGPA|AIDA|AIPA|ANPA|ANVA|AROA|ASCA|ASIA)[A-Z0-9]{16}"
    ).unwrap();
}

/// Check if content contains any email addresses.
pub fn contains_email(content: &str) -> bool {
    EMAIL_PATTERN.is_match(content)
}

/// Check if content contains any phone numbers.
pub fn contains_phone(content: &str) -> bool {
    PHONE_PATTERN.is_match(content)
}

/// Check if content contains any SSN patterns.
pub fn contains_ssn(content: &str) -> bool {
    SSN_PATTERN.is_match(content)
}

/// Check if content contains any credit card numbers.
pub fn contains_credit_card(content: &str) -> bool {
    CREDIT_CARD_PATTERN.is_match(content)
}

/// Check if content contains any API keys or secrets.
pub fn contains_api_key(content: &str) -> bool {
    API_KEY_PATTERN.is_match(content)
}

/// Check if content contains any AWS access keys.
pub fn contains_aws_key(content: &str) -> bool {
    AWS_KEY_PATTERN.is_match(content)
}

/// Check if content contains any PII (email, phone, SSN, or credit card).
pub fn contains_pii(content: &str) -> bool {
    contains_email(content)
        || contains_phone(content)
        || contains_ssn(content)
        || contains_credit_card(content)
}

/// Check if content contains any credentials (API keys or AWS keys).
pub fn contains_credentials(content: &str) -> bool {
    contains_api_key(content) || contains_aws_key(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_detection() {
        assert!(contains_email("Contact john@example.com for help"));
        assert!(contains_email("Email: user.name+tag@domain.co.uk"));
        assert!(!contains_email("No email here"));
    }

    #[test]
    fn test_phone_detection() {
        assert!(contains_phone("Call us at 555-123-4567"));
        assert!(contains_phone("Phone: (555) 123-4567"));
        assert!(contains_phone("Reach us at +1 555 123 4567"));
        assert!(!contains_phone("No phone here"));
    }

    #[test]
    fn test_ssn_detection() {
        assert!(contains_ssn("SSN: 123-45-6789"));
        assert!(contains_ssn("Social: 123 45 6789"));
        assert!(!contains_ssn("Not an SSN: 12-345-6789"));
    }

    #[test]
    fn test_credit_card_detection() {
        assert!(contains_credit_card("Card: 4111-1111-1111-1111"));
        assert!(contains_credit_card("CC: 4111 1111 1111 1111"));
        assert!(!contains_credit_card("Not a card: 411111111111"));
    }

    #[test]
    fn test_api_key_detection() {
        assert!(contains_api_key("api_key: sk_live_abcdefghijklmnop"));
        assert!(contains_api_key("token=abc123def456ghi789"));
        assert!(contains_api_key("Authorization: Bearer eyJhbGciOiJIUzI1NiIs"));
        assert!(!contains_api_key("No key here"));
    }

    #[test]
    fn test_aws_key_detection() {
        assert!(contains_aws_key("AWS key: AKIAIOSFODNN7EXAMPLE"));
        assert!(contains_aws_key("Role ARN key: AROAIOSFODNN7EXAMPLE"));
        assert!(!contains_aws_key("Not AWS: BKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_contains_pii() {
        assert!(contains_pii("Email: test@example.com"));
        assert!(contains_pii("SSN: 123-45-6789"));
        assert!(!contains_pii("No PII in this text"));
    }

    #[test]
    fn test_contains_credentials() {
        assert!(contains_credentials("api_key=secret12345678901234"));
        assert!(contains_credentials("Key: AKIAIOSFODNN7EXAMPLE"));
        assert!(!contains_credentials("No credentials here"));
    }
}
