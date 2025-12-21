//! Core types for Steward evaluation.
//!
//! These types are the data structures used throughout Steward for
//! contracts, outputs, and evaluation results.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::contract::Contract;
use crate::evidence::Evidence;

/// The type of content being evaluated.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Text,
    // Future: Image, Audio, Code
}

/// Output from an AI system to be evaluated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    /// Type of content
    pub content_type: ContentType,

    /// The actual content
    pub content: String,

    /// Optional metadata about the output
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Output {
    /// Create a text output.
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content_type: ContentType::Text,
            content: content.into(),
            metadata: HashMap::new(),
        }
    }

    /// Create a text output with metadata.
    pub fn text_with_metadata(content: impl Into<String>, metadata: HashMap<String, String>) -> Self {
        Self {
            content_type: ContentType::Text,
            content: content.into(),
            metadata,
        }
    }
}

/// Request for evaluation.
#[derive(Debug, Clone)]
pub struct EvaluationRequest {
    /// The contract to evaluate against
    pub contract: Contract,

    /// The output to evaluate
    pub output: Output,

    /// Optional context the AI had access to
    pub context: Option<Vec<String>>,

    /// Optional metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Result of evaluating an output against a contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// The final state: PROCEED, ESCALATE, or BLOCKED
    pub state: State,

    /// Findings from each lens
    pub lens_findings: LensFindings,

    /// Overall confidence (minimum of lens confidences)
    pub confidence: f64,

    /// When the evaluation occurred
    pub evaluated_at: DateTime<Utc>,

    /// Optional runtime metadata (added by extensions)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl EvaluationResult {
    /// Get mutable reference to metadata for runtime extensions.
    pub fn metadata_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.metadata
    }
}

/// The three possible states from evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum State {
    /// All conditions met. Automation may continue.
    Proceed {
        /// Summary of why it passed
        summary: String,
    },

    /// Uncertainty detected. Human judgment required.
    Escalate {
        /// What triggered the escalation
        uncertainty: String,

        /// The decision that needs human judgment
        decision_point: String,

        /// Options for the human (no ranking)
        options: Vec<String>,
    },

    /// Boundary violated. Automation must halt.
    Blocked {
        /// Details of the violation
        violation: BoundaryViolation,
    },
}

/// Details of a boundary violation that triggered BLOCKED.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundaryViolation {
    /// Which lens detected the violation
    pub lens: LensType,

    /// ID of the violated rule (e.g., "B1")
    pub rule_id: String,

    /// Full text of the violated rule
    pub rule_text: String,

    /// Evidence supporting the violation
    pub evidence: Vec<Evidence>,

    /// Contact for the accountable human
    pub accountable_human: String,
}

/// The five lens types.
///
/// Ordered alphabetically for deterministic iteration in BTreeMap.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum LensType {
    AccountabilityOwnership,
    BoundariesSafety,
    DignityInclusion,
    RestraintPrivacy,
    TransparencyContestability,
}

impl LensType {
    /// Get the stewardship question this lens answers.
    pub fn question(&self) -> &'static str {
        match self {
            LensType::DignityInclusion => "Does this disempower people or exclude them from relevance?",
            LensType::BoundariesSafety => "Does this respect defined scope and stop conditions?",
            LensType::RestraintPrivacy => "Does this expose what should be protected?",
            LensType::TransparencyContestability => "Can the human understand why this happened and contest it?",
            LensType::AccountabilityOwnership => "Who approved this, who can stop it, and who answers for it?",
        }
    }
}

/// Findings from all five lenses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensFindings {
    pub dignity_inclusion: LensFinding,
    pub boundaries_safety: LensFinding,
    pub restraint_privacy: LensFinding,
    pub transparency_contestability: LensFinding,
    pub accountability_ownership: LensFinding,
}

/// Finding from a single lens evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensFinding {
    /// Which lens produced this finding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lens: Option<LensType>,

    /// The stewardship question asked
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question_asked: Option<String>,

    /// The state determined by this lens
    pub state: LensState,

    /// Rules that were evaluated
    #[serde(default)]
    pub rules_evaluated: Vec<RuleEvaluation>,

    /// Confidence in the finding
    pub confidence: f64,
}

/// State of a single lens.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum LensState {
    /// All rules passed
    Pass,

    /// Escalation needed
    Escalate {
        /// Reason for escalation
        reason: String,
    },

    /// Boundary violated
    Blocked {
        /// What was violated
        violation: String,
    },
}

impl LensState {
    /// Check if this is a Pass state.
    pub fn is_pass(&self) -> bool {
        matches!(self, LensState::Pass)
    }

    /// Check if this is an Escalate state.
    pub fn is_escalate(&self) -> bool {
        matches!(self, LensState::Escalate { .. })
    }

    /// Check if this is a Blocked state.
    pub fn is_blocked(&self) -> bool {
        matches!(self, LensState::Blocked { .. })
    }
}

/// Evaluation of a single rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvaluation {
    /// Rule ID (e.g., "B1", "D2")
    pub rule_id: String,

    /// Full rule text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_text: Option<String>,

    /// Result of evaluation
    pub result: RuleResult,

    /// Evidence supporting the result
    #[serde(default)]
    pub evidence: Vec<Evidence>,

    /// Explanation of how the rule was evaluated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

/// Result of evaluating a single rule.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleResult {
    /// Rule condition is satisfied
    Satisfied,

    /// Rule condition is violated
    Violated,

    /// Cannot determine with certainty
    Uncertain,

    /// Rule does not apply to this output
    NotApplicable,
}

/// Source of evidence.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvidenceSource {
    /// Evidence from the contract
    Contract,

    /// Evidence from the output being evaluated
    Output,

    /// Evidence from the context
    Context,

    /// Evidence from metadata
    Metadata,
}
