//! JSON Schema validation for contracts.
//!
//! Contracts are validated against spec/contract.schema.json.
//! This module provides schema loading and validation utilities.
//!
//! Per spec section 7.3: "Every contract must validate against spec/contract.schema.json"

use std::sync::OnceLock;
use thiserror::Error;

/// Embedded contract schema (loaded at compile time).
const CONTRACT_SCHEMA_JSON: &str = include_str!("../../../../spec/contract.schema.json");

/// Compiled JSON Schema validator (initialized once, reused).
static COMPILED_SCHEMA: OnceLock<Result<jsonschema::Validator, String>> = OnceLock::new();

/// Errors from schema validation.
#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Failed to load schema: {0}")]
    LoadError(String),
}

/// Get or initialize the compiled schema validator.
fn get_validator() -> Result<&'static jsonschema::Validator, SchemaError> {
    let result = COMPILED_SCHEMA.get_or_init(|| {
        let schema_value: serde_json::Value = match serde_json::from_str(CONTRACT_SCHEMA_JSON) {
            Ok(v) => v,
            Err(e) => return Err(format!("Invalid schema JSON: {}", e)),
        };

        match jsonschema::options().build(&schema_value) {
            Ok(v) => Ok(v),
            Err(e) => Err(format!("Failed to compile schema: {}", e)),
        }
    });

    match result {
        Ok(v) => Ok(v),
        Err(e) => Err(SchemaError::LoadError(e.clone())),
    }
}

/// Validate a contract JSON value against the schema.
///
/// Returns Ok(()) if valid, or a list of validation error messages.
///
/// # Arguments
///
/// * `contract_json` - The contract as a JSON value
///
/// # Returns
///
/// * `Ok(())` - Contract is valid
/// * `Err(Vec<String>)` - List of validation errors
pub fn validate_contract_schema(contract_json: &serde_json::Value) -> Result<(), Vec<String>> {
    let validator = get_validator().map_err(|e| vec![e.to_string()])?;

    // Collect all validation errors
    let errors: Vec<String> = validator
        .iter_errors(contract_json)
        .map(|e| format!("{} at {}", e, e.instance_path))
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check if a contract JSON value is valid against the schema.
///
/// Returns true if valid, false otherwise. Use `validate_contract_schema`
/// for detailed error messages.
#[allow(dead_code)]
pub fn is_valid_contract(contract_json: &serde_json::Value) -> bool {
    get_validator()
        .map(|v| v.is_valid(contract_json))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_contract_passes_schema() {
        let value = serde_json::json!({
            "contract_version": "1.0",
            "schema_version": "2025-12-20",
            "name": "Test Contract",
            "intent": {
                "purpose": "Test automation"
            },
            "boundaries": {},
            "accountability": {
                "answerable_human": "test@example.com"
            },
            "acceptance": {}
        });
        assert!(validate_contract_schema(&value).is_ok());
    }

    #[test]
    fn test_missing_required_field_fails() {
        let value = serde_json::json!({
            "contract_version": "1.0",
            "schema_version": "2025-12-20",
            "name": "Test"
            // Missing: intent, boundaries, accountability, acceptance
        });
        let result = validate_contract_schema(&value);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_invalid_version_format_fails() {
        let value = serde_json::json!({
            "contract_version": "invalid",  // Should match pattern ^\d+\.\d+(\.\d+)?$
            "schema_version": "2025-12-20",
            "name": "Test",
            "intent": { "purpose": "Test" },
            "boundaries": {},
            "accountability": { "answerable_human": "test@example.com" },
            "acceptance": {}
        });
        let result = validate_contract_schema(&value);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_schema_version_format_fails() {
        let value = serde_json::json!({
            "contract_version": "1.0",
            "schema_version": "20251220",  // Should be YYYY-MM-DD format
            "name": "Test",
            "intent": { "purpose": "Test" },
            "boundaries": {},
            "accountability": { "answerable_human": "test@example.com" },
            "acceptance": {}
        });
        let result = validate_contract_schema(&value);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_rule_id_format_fails() {
        let value = serde_json::json!({
            "contract_version": "1.0",
            "schema_version": "2025-12-20",
            "name": "Test",
            "intent": { "purpose": "Test" },
            "boundaries": {
                "invalidated_by": [
                    { "id": "invalid_id", "rule": "Some rule" }  // Should match ^[A-Z][0-9]+$
                ]
            },
            "accountability": { "answerable_human": "test@example.com" },
            "acceptance": {}
        });
        let result = validate_contract_schema(&value);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_rule_id_format_passes() {
        let value = serde_json::json!({
            "contract_version": "1.0",
            "schema_version": "2025-12-20",
            "name": "Test",
            "intent": { "purpose": "Test" },
            "boundaries": {
                "invalidated_by": [
                    { "id": "B1", "rule": "PII exposed" },
                    { "id": "B2", "rule": "Credentials leaked" }
                ]
            },
            "accountability": { "answerable_human": "test@example.com" },
            "acceptance": {}
        });
        assert!(validate_contract_schema(&value).is_ok());
    }

    #[test]
    fn test_additional_properties_fail() {
        let value = serde_json::json!({
            "contract_version": "1.0",
            "schema_version": "2025-12-20",
            "name": "Test",
            "intent": { "purpose": "Test" },
            "boundaries": {},
            "accountability": { "answerable_human": "test@example.com" },
            "acceptance": {},
            "unknown_field": "should fail"  // additionalProperties: false
        });
        let result = validate_contract_schema(&value);
        assert!(result.is_err());
    }

    #[test]
    fn test_full_contract_with_all_sections() {
        let value = serde_json::json!({
            "contract_version": "1.0.0",
            "schema_version": "2025-12-20",
            "policy_pack": ["general", "healthcare"],
            "name": "Customer Support Bot",
            "description": "Handles tier-1 customer inquiries",
            "intent": {
                "purpose": "Provide helpful customer support",
                "optimizing_for": ["customer satisfaction", "response time"],
                "never_optimize_away": [
                    { "id": "N1", "rule": "Human escalation path must always be available" }
                ]
            },
            "boundaries": {
                "may_do_autonomously": [
                    { "id": "A1", "rule": "Answer factual questions about products" }
                ],
                "must_pause_when": [
                    { "id": "P1", "rule": "Customer expresses frustration" }
                ],
                "must_escalate_when": [
                    { "id": "E1", "rule": "Customer requests human agent" }
                ],
                "invalidated_by": [
                    { "id": "B1", "rule": "Customer PII exposed in response" }
                ]
            },
            "accountability": {
                "approved_by": "Product Manager",
                "answerable_human": "support-lead@company.com",
                "escalation_path": ["Tier 1", "Tier 2", "Manager"],
                "review_cadence": "monthly"
            },
            "acceptance": {
                "fit_criteria": [
                    { "id": "F1", "rule": "Addresses customer's actual question" }
                ],
                "dignity_check": [
                    { "id": "D1", "rule": "Does not dismiss customer concerns" }
                ]
            }
        });
        assert!(validate_contract_schema(&value).is_ok());
    }

    #[test]
    fn test_is_valid_helper() {
        let valid = serde_json::json!({
            "contract_version": "1.0",
            "schema_version": "2025-12-20",
            "name": "Test",
            "intent": { "purpose": "Test" },
            "boundaries": {},
            "accountability": { "answerable_human": "test@example.com" },
            "acceptance": {}
        });
        assert!(is_valid_contract(&valid));

        let invalid = serde_json::json!({ "name": "Only name" });
        assert!(!is_valid_contract(&invalid));
    }
}
