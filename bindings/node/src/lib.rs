//! Node.js bindings for Steward using napi-rs.
//!
//! This module exposes the core Steward types and evaluation functions
//! to Node.js, enabling `npm install @steward/core` usage.

#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;

use steward_core::{
    self as core, EvidenceSource as CoreEvidenceSource, LensState as CoreLensState,
    LensType as CoreLensType, RuleResult as CoreRuleResult, State as CoreState,
};

/// Lens types for categorizing findings.
/// Ordered alphabetically to match steward-core's Ord implementation.
#[napi]
pub enum LensType {
    AccountabilityOwnership,
    BoundariesSafety,
    DignityInclusion,
    RestraintPrivacy,
    TransparencyContestability,
}

impl From<CoreLensType> for LensType {
    fn from(lt: CoreLensType) -> Self {
        match lt {
            CoreLensType::DignityInclusion => LensType::DignityInclusion,
            CoreLensType::BoundariesSafety => LensType::BoundariesSafety,
            CoreLensType::RestraintPrivacy => LensType::RestraintPrivacy,
            CoreLensType::TransparencyContestability => LensType::TransparencyContestability,
            CoreLensType::AccountabilityOwnership => LensType::AccountabilityOwnership,
        }
    }
}

/// Rule evaluation result types.
#[napi]
pub enum RuleResult {
    Satisfied,
    Violated,
    Uncertain,
    NotApplicable,
}

impl From<CoreRuleResult> for RuleResult {
    fn from(rr: CoreRuleResult) -> Self {
        match rr {
            CoreRuleResult::Satisfied => RuleResult::Satisfied,
            CoreRuleResult::Violated => RuleResult::Violated,
            CoreRuleResult::Uncertain => RuleResult::Uncertain,
            CoreRuleResult::NotApplicable => RuleResult::NotApplicable,
        }
    }
}

/// Source of evidence for findings.
#[napi]
pub enum EvidenceSource {
    Contract,
    Output,
    Context,
    Metadata,
}

impl From<CoreEvidenceSource> for EvidenceSource {
    fn from(es: CoreEvidenceSource) -> Self {
        match es {
            CoreEvidenceSource::Contract => EvidenceSource::Contract,
            CoreEvidenceSource::Output => EvidenceSource::Output,
            CoreEvidenceSource::Context => EvidenceSource::Context,
            CoreEvidenceSource::Metadata => EvidenceSource::Metadata,
        }
    }
}

/// Evidence supporting a rule evaluation.
#[napi(object)]
pub struct Evidence {
    pub source: EvidenceSource,
    pub claim: String,
    pub pointer: String,
}

impl From<core::Evidence> for Evidence {
    fn from(e: core::Evidence) -> Self {
        Evidence {
            source: e.source.into(),
            claim: e.claim,
            pointer: e.pointer,
        }
    }
}

/// Result of evaluating a single rule.
#[napi(object)]
pub struct RuleEvaluation {
    pub rule_id: String,
    pub rule_text: Option<String>,
    pub result: RuleResult,
    pub evidence: Vec<Evidence>,
    pub rationale: Option<String>,
}

impl From<core::RuleEvaluation> for RuleEvaluation {
    fn from(re: core::RuleEvaluation) -> Self {
        RuleEvaluation {
            rule_id: re.rule_id,
            rule_text: re.rule_text,
            result: re.result.into(),
            evidence: re.evidence.into_iter().map(|e| e.into()).collect(),
            rationale: re.rationale,
        }
    }
}

/// State of a single lens after evaluation.
#[napi(object)]
pub struct LensState {
    pub state_type: String,
    pub reason: Option<String>,
    pub violation: Option<String>,
}

impl From<CoreLensState> for LensState {
    fn from(ls: CoreLensState) -> Self {
        match ls {
            CoreLensState::Pass => LensState {
                state_type: "Pass".to_string(),
                reason: None,
                violation: None,
            },
            CoreLensState::Escalate { reason } => LensState {
                state_type: "Escalate".to_string(),
                reason: Some(reason),
                violation: None,
            },
            CoreLensState::Blocked { violation } => LensState {
                state_type: "Blocked".to_string(),
                reason: None,
                violation: Some(violation),
            },
        }
    }
}

/// Finding from a single lens evaluation.
#[napi(object)]
pub struct LensFinding {
    pub lens: Option<LensType>,
    pub question_asked: Option<String>,
    pub state: LensState,
    pub rules_evaluated: Vec<RuleEvaluation>,
    pub confidence: f64,
}

impl From<core::LensFinding> for LensFinding {
    fn from(lf: core::LensFinding) -> Self {
        LensFinding {
            lens: lf.lens.map(|l| l.into()),
            question_asked: lf.question_asked,
            state: lf.state.into(),
            rules_evaluated: lf.rules_evaluated.into_iter().map(|r| r.into()).collect(),
            confidence: lf.confidence,
        }
    }
}

/// Findings from all five lenses.
#[napi(object)]
pub struct LensFindings {
    pub dignity_inclusion: LensFinding,
    pub boundaries_safety: LensFinding,
    pub restraint_privacy: LensFinding,
    pub transparency_contestability: LensFinding,
    pub accountability_ownership: LensFinding,
}

impl From<core::LensFindings> for LensFindings {
    fn from(lf: core::LensFindings) -> Self {
        LensFindings {
            dignity_inclusion: lf.dignity_inclusion.into(),
            boundaries_safety: lf.boundaries_safety.into(),
            restraint_privacy: lf.restraint_privacy.into(),
            transparency_contestability: lf.transparency_contestability.into(),
            accountability_ownership: lf.accountability_ownership.into(),
        }
    }
}

/// Boundary violation details.
#[napi(object)]
pub struct BoundaryViolation {
    pub lens: LensType,
    pub rule_id: String,
    pub rule_text: String,
    pub evidence: Vec<Evidence>,
    pub accountable_human: String,
}

impl From<core::BoundaryViolation> for BoundaryViolation {
    fn from(bv: core::BoundaryViolation) -> Self {
        BoundaryViolation {
            lens: bv.lens.into(),
            rule_id: bv.rule_id,
            rule_text: bv.rule_text,
            evidence: bv.evidence.into_iter().map(|e| e.into()).collect(),
            accountable_human: bv.accountable_human,
        }
    }
}

/// The evaluation state: Proceed, Escalate, or Blocked.
#[napi(object)]
pub struct State {
    pub state_type: String,
    pub summary: Option<String>,
    pub uncertainty: Option<String>,
    pub decision_point: Option<String>,
    pub options: Option<Vec<String>>,
    pub violation: Option<BoundaryViolation>,
}

impl From<CoreState> for State {
    fn from(state: CoreState) -> Self {
        match state {
            CoreState::Proceed { summary } => State {
                state_type: "Proceed".to_string(),
                summary: Some(summary),
                uncertainty: None,
                decision_point: None,
                options: None,
                violation: None,
            },
            CoreState::Escalate {
                uncertainty,
                decision_point,
                options,
            } => State {
                state_type: "Escalate".to_string(),
                summary: None,
                uncertainty: Some(uncertainty),
                decision_point: Some(decision_point),
                options: Some(options),
                violation: None,
            },
            CoreState::Blocked { violation } => State {
                state_type: "Blocked".to_string(),
                summary: None,
                uncertainty: None,
                decision_point: None,
                options: None,
                violation: Some(violation.into()),
            },
        }
    }
}

/// The complete result of evaluating an output against a contract.
#[napi(object)]
pub struct EvaluationResult {
    pub state: State,
    pub lens_findings: LensFindings,
    pub confidence: f64,
    pub evaluated_at: String,
}

impl From<core::EvaluationResult> for EvaluationResult {
    fn from(er: core::EvaluationResult) -> Self {
        EvaluationResult {
            state: er.state.into(),
            lens_findings: er.lens_findings.into(),
            confidence: er.confidence,
            evaluated_at: er.evaluated_at.to_rfc3339(),
        }
    }
}

/// A stewardship contract defining rules for AI evaluation.
#[napi]
pub struct Contract {
    inner: core::Contract,
}

#[napi]
impl Contract {
    /// Create a Contract from a YAML string.
    #[napi(factory)]
    pub fn from_yaml(yaml: String) -> Result<Self> {
        core::Contract::from_yaml(&yaml)
            .map(|c| Contract { inner: c })
            .map_err(|e| Error::new(Status::InvalidArg, format!("Failed to parse contract: {}", e)))
    }

    /// Create a Contract from a YAML file.
    #[napi(factory)]
    pub fn from_yaml_file(path: String) -> Result<Self> {
        core::Contract::from_yaml_file(&path)
            .map(|c| Contract { inner: c })
            .map_err(|e| Error::new(Status::InvalidArg, format!("Failed to load contract: {}", e)))
    }

    /// Get the contract name.
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get the contract version.
    #[napi(getter)]
    pub fn version(&self) -> String {
        self.inner.contract_version.clone()
    }

    /// Get the contract purpose.
    #[napi(getter)]
    pub fn purpose(&self) -> String {
        self.inner.intent.purpose.clone()
    }

    /// Get the accountable human email.
    #[napi(getter)]
    pub fn accountable_human(&self) -> String {
        self.inner.accountability.answerable_human.clone()
    }
}

/// AI-generated output to be evaluated.
#[napi]
pub struct Output {
    inner: core::Output,
}

#[napi]
impl Output {
    /// Create a text output.
    #[napi(factory)]
    pub fn text(content: String) -> Self {
        Output {
            inner: core::Output::text(content),
        }
    }

    /// Get the output content.
    #[napi(getter)]
    pub fn content(&self) -> String {
        self.inner.content.clone()
    }
}

/// Evaluate an output against a stewardship contract.
///
/// This is the main entry point for Steward evaluation.
#[napi]
pub fn evaluate(contract: &Contract, output: &Output) -> Result<EvaluationResult> {
    core::evaluate(&contract.inner, &output.inner)
        .map(|r| r.into())
        .map_err(|e| Error::new(Status::GenericFailure, format!("Evaluation failed: {}", e)))
}

/// Evaluate with optional context and metadata.
#[napi]
pub fn evaluate_with_context(
    contract: &Contract,
    output: &Output,
    context: Option<Vec<String>>,
    metadata: Option<HashMap<String, String>>,
) -> Result<EvaluationResult> {
    core::evaluate_with_context(
        &contract.inner,
        &output.inner,
        context.as_deref(),
        metadata.as_ref(),
    )
    .map(|r| r.into())
    .map_err(|e| Error::new(Status::GenericFailure, format!("Evaluation failed: {}", e)))
}

/// Check if an evaluation result is Proceed.
#[napi]
pub fn is_proceed(state_type: String) -> bool {
    state_type == "Proceed"
}

/// Check if an evaluation result is Escalate.
#[napi]
pub fn is_escalate(state_type: String) -> bool {
    state_type == "Escalate"
}

/// Check if an evaluation result is Blocked.
#[napi]
pub fn is_blocked(state_type: String) -> bool {
    state_type == "Blocked"
}
