//! The Five Lenses for stewardship evaluation.
//!
//! Each lens asks one stewardship question and evaluates independently.
//! Lenses cannot access other lenses' findings - synthesis is policy, not intelligence.
//!
//! ## Lens Independence
//!
//! - Lenses evaluate in parallel (simulated sequential in current implementation)
//! - No lens may access another lens's findings
//! - No shared mutable state between lenses
//! - Deterministic ordering guaranteed via BTreeMap

mod accountability;
mod boundaries;
mod dignity;
pub mod domain_patterns;
pub mod patterns;
mod restraint;
mod transparency;

pub use accountability::AccountabilityLens;
pub use boundaries::BoundariesLens;
pub use dignity::DignityLens;
pub use domain_patterns::{check_domain_patterns, DomainMatch, PatternSeverity};
pub use restraint::RestraintLens;
pub use transparency::TransparencyLens;

use crate::types::{EvaluationRequest, LensType};
pub use crate::types::{LensFinding, LensState};

/// Trait implemented by all lenses.
pub trait Lens {
    /// The type of this lens.
    fn lens_type(&self) -> LensType;

    /// The stewardship question this lens answers.
    fn question(&self) -> &'static str {
        self.lens_type().question()
    }

    /// Evaluate the request against this lens's rules.
    ///
    /// # Arguments
    ///
    /// * `request` - The evaluation request containing contract, output, and context
    ///
    /// # Returns
    ///
    /// A `LensFinding` with the state, rules evaluated, and confidence.
    fn evaluate(&self, request: &EvaluationRequest) -> LensFinding;
}

/// Create a default PASS finding for a lens.
#[allow(dead_code)] // Utility function for lens implementations
pub(crate) fn default_pass_finding(lens_type: LensType) -> LensFinding {
    LensFinding {
        lens: Some(lens_type),
        question_asked: Some(lens_type.question().to_string()),
        state: LensState::Pass,
        rules_evaluated: vec![],
        confidence: 0.5, // Default confidence when no rules apply
    }
}

