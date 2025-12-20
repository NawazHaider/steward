//! Contract parsing from YAML/JSON.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur when parsing contracts.
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Failed to read contract file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Contract validation failed: {0}")]
    ValidationError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// A single rule with ID and text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    /// Unique identifier (e.g., "B1", "D2")
    pub id: String,

    /// The rule text
    pub rule: String,
}

/// Intent section of a contract.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Intent {
    /// Primary purpose of the automation
    pub purpose: String,

    /// What to optimize for (in priority order)
    #[serde(default)]
    pub optimizing_for: Vec<String>,

    /// Constraints that must never be traded off
    #[serde(default)]
    pub never_optimize_away: Vec<Rule>,
}

/// Boundaries section of a contract.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Boundaries {
    /// Actions allowed without human intervention
    #[serde(default)]
    pub may_do_autonomously: Vec<Rule>,

    /// Conditions that trigger a pause
    #[serde(default)]
    pub must_pause_when: Vec<Rule>,

    /// Conditions that require escalation
    #[serde(default)]
    pub must_escalate_when: Vec<Rule>,

    /// Conditions that invalidate automation entirely
    #[serde(default)]
    pub invalidated_by: Vec<Rule>,
}

/// Accountability section of a contract.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Accountability {
    /// Who approved this contract
    #[serde(default)]
    pub approved_by: Option<String>,

    /// Contact for the accountable human
    pub answerable_human: String,

    /// Ordered escalation path
    #[serde(default)]
    pub escalation_path: Vec<String>,

    /// How often to review
    #[serde(default)]
    pub review_cadence: Option<String>,
}

/// Acceptance section of a contract.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Acceptance {
    /// Criteria for fit-for-purpose output
    #[serde(default)]
    pub fit_criteria: Vec<Rule>,

    /// Dignity preservation criteria
    #[serde(default)]
    pub dignity_check: Vec<Rule>,
}

/// A stewardship contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Version of this contract (semver)
    pub contract_version: String,

    /// Version of the contract schema (date-based)
    pub schema_version: String,

    /// Policy packs this contract extends
    #[serde(default)]
    pub policy_pack: Vec<String>,

    /// Human-readable name
    pub name: String,

    /// Detailed description
    #[serde(default)]
    pub description: Option<String>,

    /// Intent section
    pub intent: Intent,

    /// Boundaries section
    #[serde(default)]
    pub boundaries: Boundaries,

    /// Accountability section
    pub accountability: Accountability,

    /// Acceptance section
    #[serde(default)]
    pub acceptance: Acceptance,
}

impl Contract {
    /// Parse a contract from YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self, ContractError> {
        let contract: Contract = serde_yaml::from_str(yaml)?;
        contract.validate()?;
        Ok(contract)
    }

    /// Parse a contract from JSON string.
    pub fn from_json(json: &str) -> Result<Self, ContractError> {
        let contract: Contract = serde_json::from_str(json)?;
        contract.validate()?;
        Ok(contract)
    }

    /// Parse a contract from a YAML file.
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self, ContractError> {
        let contents = fs::read_to_string(path)?;
        Self::from_yaml(&contents)
    }

    /// Parse a contract from a JSON file.
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, ContractError> {
        let contents = fs::read_to_string(path)?;
        Self::from_json(&contents)
    }

    /// Validate the contract structure.
    fn validate(&self) -> Result<(), ContractError> {
        // Check required fields
        if self.name.is_empty() {
            return Err(ContractError::MissingField("name".to_string()));
        }

        if self.intent.purpose.is_empty() {
            return Err(ContractError::MissingField("intent.purpose".to_string()));
        }

        if self.accountability.answerable_human.is_empty() {
            return Err(ContractError::MissingField(
                "accountability.answerable_human".to_string(),
            ));
        }

        // Validate rule IDs are unique within sections
        self.validate_unique_rule_ids()?;

        Ok(())
    }

    /// Ensure rule IDs are unique within their sections.
    fn validate_unique_rule_ids(&self) -> Result<(), ContractError> {
        let mut seen = std::collections::HashSet::new();

        let all_rules = self
            .intent
            .never_optimize_away
            .iter()
            .chain(self.boundaries.may_do_autonomously.iter())
            .chain(self.boundaries.must_pause_when.iter())
            .chain(self.boundaries.must_escalate_when.iter())
            .chain(self.boundaries.invalidated_by.iter())
            .chain(self.acceptance.fit_criteria.iter())
            .chain(self.acceptance.dignity_check.iter());

        for rule in all_rules {
            if !seen.insert(&rule.id) {
                return Err(ContractError::ValidationError(format!(
                    "Duplicate rule ID: {}",
                    rule.id
                )));
            }
        }

        Ok(())
    }

    /// Get all rules that should be evaluated by the Boundaries lens.
    pub fn boundaries_rules(&self) -> Vec<&Rule> {
        let mut rules = Vec::new();
        rules.extend(self.boundaries.may_do_autonomously.iter());
        rules.extend(self.boundaries.must_pause_when.iter());
        rules.extend(self.boundaries.must_escalate_when.iter());
        rules.extend(self.boundaries.invalidated_by.iter());
        rules
    }

    /// Get all rules that should be evaluated by the Restraint lens.
    pub fn restraint_rules(&self) -> Vec<&Rule> {
        // Privacy-related rules from invalidated_by and never_optimize_away
        self.boundaries
            .invalidated_by
            .iter()
            .filter(|r| {
                let text = r.rule.to_lowercase();
                text.contains("pii")
                    || text.contains("privacy")
                    || text.contains("credential")
                    || text.contains("secret")
                    || text.contains("expose")
            })
            .chain(self.intent.never_optimize_away.iter().filter(|r| {
                let text = r.rule.to_lowercase();
                text.contains("privacy") || text.contains("data")
            }))
            .collect()
    }

    /// Get all rules that should be evaluated by the Dignity lens.
    pub fn dignity_rules(&self) -> Vec<&Rule> {
        let mut rules: Vec<&Rule> = self.acceptance.dignity_check.iter().collect();

        // Also include dignity-related rules from never_optimize_away
        rules.extend(self.intent.never_optimize_away.iter().filter(|r| {
            let text = r.rule.to_lowercase();
            text.contains("dignity")
                || text.contains("respect")
                || text.contains("human")
                || text.contains("escalation")
        }));

        rules
    }

    /// Get all rules that should be evaluated by the Transparency lens.
    pub fn transparency_rules(&self) -> Vec<&Rule> {
        self.acceptance.fit_criteria.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_CONTRACT: &str = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test Contract"
intent:
  purpose: "Test purpose"
boundaries:
  invalidated_by:
    - id: "B1"
      rule: "PII exposed"
accountability:
  answerable_human: "test@example.com"
acceptance: {}
"#;

    #[test]
    fn test_parse_valid_contract() {
        let contract = Contract::from_yaml(VALID_CONTRACT).unwrap();
        assert_eq!(contract.name, "Test Contract");
        assert_eq!(contract.intent.purpose, "Test purpose");
        assert_eq!(contract.boundaries.invalidated_by.len(), 1);
    }

    #[test]
    fn test_missing_answerable_human() {
        let yaml = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability: {}
"#;
        // This should fail due to missing answerable_human
        let result = Contract::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_rule_ids() {
        let yaml = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
boundaries:
  invalidated_by:
    - id: "B1"
      rule: "Rule 1"
    - id: "B1"
      rule: "Rule 2"
accountability:
  answerable_human: "test@example.com"
"#;
        let result = Contract::from_yaml(yaml);
        assert!(matches!(
            result,
            Err(ContractError::ValidationError(_))
        ));
    }
}
