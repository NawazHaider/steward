//! Synthesizer with metadata extensions for steward-runtime.
//!
//! # Blueprint Constraint (IMMUTABLE)
//! "State resolution rules (strict)... These rules are not configurable. They are the policy."
//!
//! Extensions may ONLY add metadata. They may NEVER:
//! - Change BLOCKED to anything else
//! - Change ESCALATE to PROCEED
//! - Change PROCEED to ESCALATE
//! - Override the `min(lens_confidences)` formula
//!
//! Domain-specific rules that affect verdicts MUST go in the CONTRACT:
//! - `boundaries.must_escalate_when[]` for escalation triggers
//! - `boundaries.invalidated_by[]` for blocking conditions
//! - `acceptance.fit_criteria[]` for acceptance requirements

use steward_core::{EvaluationResult, LensFindings};

/// Hook for synthesizer metadata ONLY.
///
/// # What Extensions CAN Do
/// - Add metadata (regulatory tags, audit info, domain annotations)
/// - Add timestamps, version info, compliance markers
/// - Log, trace, or emit metrics
///
/// # What Extensions CANNOT Do
/// - Change the verdict (PROCEED/ESCALATE/BLOCKED)
/// - Modify confidence scores
/// - Add or remove evidence
pub trait SynthesizerMetadataExtension: Send + Sync {
    /// Add domain-specific metadata to result.
    /// Called AFTER synthesis is complete.
    ///
    /// # Examples of allowed metadata:
    /// - regulatory_framework: "SEC_17a-4"
    /// - compliance_check: "financial_services"
    /// - audit_timestamp: "2025-12-20T14:32:00Z"
    /// - domain_version: "1.2.3"
    fn add_metadata(&self, result: &mut EvaluationResult, findings: &LensFindings);
}

/// Financial services extension (METADATA ONLY).
pub struct FinancialServicesExtension {
    /// Regulatory framework identifier
    pub regulatory_framework: String,

    /// Whether audit is enabled
    pub audit_enabled: bool,
}

impl SynthesizerMetadataExtension for FinancialServicesExtension {
    fn add_metadata(&self, result: &mut EvaluationResult, findings: &LensFindings) {
        // Get mutable reference to metadata using the core API
        let metadata = result.metadata_mut();

        // Add regulatory metadata
        metadata.insert(
            "regulatory_framework".to_string(),
            self.regulatory_framework.clone(),
        );
        metadata.insert(
            "compliance_domain".to_string(),
            "financial_services".to_string(),
        );
        metadata.insert(
            "audit_enabled".to_string(),
            self.audit_enabled.to_string(),
        );
        metadata.insert(
            "evaluated_confidence".to_string(),
            format!("{:.2}", min_confidence(findings)),
        );
    }
}

/// Healthcare compliance extension (METADATA ONLY).
pub struct HealthcareExtension {
    /// Whether PHI detection is strict
    pub phi_detection_strict: bool,
}

impl SynthesizerMetadataExtension for HealthcareExtension {
    fn add_metadata(&self, result: &mut EvaluationResult, _findings: &LensFindings) {
        let metadata = result.metadata_mut();

        metadata.insert(
            "compliance_domain".to_string(),
            "healthcare".to_string(),
        );
        metadata.insert(
            "phi_detection_mode".to_string(),
            if self.phi_detection_strict { "strict" } else { "standard" }.to_string(),
        );
        metadata.insert(
            "hipaa_applicable".to_string(),
            "true".to_string(),
        );
    }
}

/// Extension manager for collecting and applying metadata extensions.
pub struct ExtensionManager {
    extensions: Vec<Box<dyn SynthesizerMetadataExtension>>,
}

impl ExtensionManager {
    /// Create a new extension manager.
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }

    /// Add an extension.
    pub fn add(&mut self, extension: Box<dyn SynthesizerMetadataExtension>) {
        self.extensions.push(extension);
    }

    /// Apply all extensions to a result.
    ///
    /// This is called AFTER synthesis is complete.
    /// Extensions can ONLY add metadata - they cannot change the verdict.
    pub fn apply(&self, result: &mut EvaluationResult, findings: &LensFindings) {
        for ext in &self.extensions {
            ext.add_metadata(result, findings);
        }
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate minimum confidence from all lenses.
fn min_confidence(findings: &LensFindings) -> f64 {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_manager() {
        let mut manager = ExtensionManager::new();
        manager.add(Box::new(FinancialServicesExtension {
            regulatory_framework: "SEC_17a-4".to_string(),
            audit_enabled: true,
        }));

        assert_eq!(manager.extensions.len(), 1);
    }
}
