//! Python bindings for Steward using PyO3.
//!
//! This module exposes the core Steward types and evaluation functions
//! to Python, enabling `pip install steward` usage.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use std::collections::HashMap;

// Re-use core types
use steward_core::{
    self as core, EvidenceSource as CoreEvidenceSource, LensState as CoreLensState,
    LensType as CoreLensType, RuleResult as CoreRuleResult, State as CoreState,
};

/// Python-compatible wrapper for Contract.
#[pyclass(name = "Contract")]
#[derive(Clone)]
pub struct PyContract {
    inner: core::Contract,
}

#[pymethods]
impl PyContract {
    /// Create a Contract from a YAML string.
    #[staticmethod]
    pub fn from_yaml(yaml: &str) -> PyResult<Self> {
        core::Contract::from_yaml(yaml)
            .map(|c| PyContract { inner: c })
            .map_err(|e| PyValueError::new_err(format!("Failed to parse contract: {}", e)))
    }

    /// Create a Contract from a YAML file.
    #[staticmethod]
    pub fn from_yaml_file(path: &str) -> PyResult<Self> {
        core::Contract::from_yaml_file(path)
            .map(|c| PyContract { inner: c })
            .map_err(|e| PyValueError::new_err(format!("Failed to load contract: {}", e)))
    }

    /// Get the contract name.
    #[getter]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get the contract version.
    #[getter]
    pub fn version(&self) -> String {
        self.inner.contract_version.clone()
    }

    /// Get the contract purpose.
    #[getter]
    pub fn purpose(&self) -> String {
        self.inner.intent.purpose.clone()
    }

    /// Get the accountable human email.
    #[getter]
    pub fn accountable_human(&self) -> String {
        self.inner.accountability.answerable_human.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "Contract(name='{}', version='{}')",
            self.name(),
            self.version()
        )
    }
}

/// Python-compatible wrapper for Output.
#[pyclass(name = "Output")]
#[derive(Clone)]
pub struct PyOutput {
    inner: core::Output,
}

#[pymethods]
impl PyOutput {
    /// Create a text output.
    #[staticmethod]
    pub fn text(content: &str) -> Self {
        PyOutput {
            inner: core::Output::text(content),
        }
    }

    /// Get the output content.
    #[getter]
    pub fn content(&self) -> String {
        self.inner.content.clone()
    }

    fn __repr__(&self) -> String {
        let preview = if self.inner.content.len() > 50 {
            format!("{}...", &self.inner.content[..50])
        } else {
            self.inner.content.clone()
        };
        format!("Output(content='{}')", preview)
    }
}

/// Python-compatible enum for LensType.
/// Ordered alphabetically to match steward-core's Ord implementation.
#[pyclass(name = "LensType", eq)]
#[derive(Clone, PartialEq, Debug)]
pub enum PyLensType {
    AccountabilityOwnership,
    BoundariesSafety,
    DignityInclusion,
    RestraintPrivacy,
    TransparencyContestability,
}

impl From<CoreLensType> for PyLensType {
    fn from(lt: CoreLensType) -> Self {
        match lt {
            CoreLensType::DignityInclusion => PyLensType::DignityInclusion,
            CoreLensType::BoundariesSafety => PyLensType::BoundariesSafety,
            CoreLensType::RestraintPrivacy => PyLensType::RestraintPrivacy,
            CoreLensType::TransparencyContestability => PyLensType::TransparencyContestability,
            CoreLensType::AccountabilityOwnership => PyLensType::AccountabilityOwnership,
        }
    }
}

#[pymethods]
impl PyLensType {
    fn __repr__(&self) -> String {
        match self {
            PyLensType::DignityInclusion => "LensType.DignityInclusion".to_string(),
            PyLensType::BoundariesSafety => "LensType.BoundariesSafety".to_string(),
            PyLensType::RestraintPrivacy => "LensType.RestraintPrivacy".to_string(),
            PyLensType::TransparencyContestability => {
                "LensType.TransparencyContestability".to_string()
            }
            PyLensType::AccountabilityOwnership => "LensType.AccountabilityOwnership".to_string(),
        }
    }
}

/// Python-compatible enum for RuleResult.
#[pyclass(name = "RuleResult", eq)]
#[derive(Clone, PartialEq, Debug)]
pub enum PyRuleResult {
    Satisfied,
    Violated,
    Uncertain,
    NotApplicable,
}

impl From<CoreRuleResult> for PyRuleResult {
    fn from(rr: CoreRuleResult) -> Self {
        match rr {
            CoreRuleResult::Satisfied => PyRuleResult::Satisfied,
            CoreRuleResult::Violated => PyRuleResult::Violated,
            CoreRuleResult::Uncertain => PyRuleResult::Uncertain,
            CoreRuleResult::NotApplicable => PyRuleResult::NotApplicable,
        }
    }
}

#[pymethods]
impl PyRuleResult {
    fn __repr__(&self) -> String {
        match self {
            PyRuleResult::Satisfied => "RuleResult.Satisfied".to_string(),
            PyRuleResult::Violated => "RuleResult.Violated".to_string(),
            PyRuleResult::Uncertain => "RuleResult.Uncertain".to_string(),
            PyRuleResult::NotApplicable => "RuleResult.NotApplicable".to_string(),
        }
    }
}

/// Python-compatible enum for EvidenceSource.
#[pyclass(name = "EvidenceSource", eq)]
#[derive(Clone, PartialEq, Debug)]
pub enum PyEvidenceSource {
    Contract,
    Output,
    Context,
    Metadata,
}

impl From<CoreEvidenceSource> for PyEvidenceSource {
    fn from(es: CoreEvidenceSource) -> Self {
        match es {
            CoreEvidenceSource::Contract => PyEvidenceSource::Contract,
            CoreEvidenceSource::Output => PyEvidenceSource::Output,
            CoreEvidenceSource::Context => PyEvidenceSource::Context,
            CoreEvidenceSource::Metadata => PyEvidenceSource::Metadata,
        }
    }
}

#[pymethods]
impl PyEvidenceSource {
    fn __repr__(&self) -> String {
        match self {
            PyEvidenceSource::Contract => "EvidenceSource.Contract".to_string(),
            PyEvidenceSource::Output => "EvidenceSource.Output".to_string(),
            PyEvidenceSource::Context => "EvidenceSource.Context".to_string(),
            PyEvidenceSource::Metadata => "EvidenceSource.Metadata".to_string(),
        }
    }
}

/// Evidence supporting a rule evaluation.
#[pyclass(name = "Evidence")]
#[derive(Clone)]
pub struct PyEvidence {
    #[pyo3(get)]
    pub source: PyEvidenceSource,
    #[pyo3(get)]
    pub claim: String,
    #[pyo3(get)]
    pub pointer: String,
}

impl From<core::Evidence> for PyEvidence {
    fn from(e: core::Evidence) -> Self {
        PyEvidence {
            source: e.source.into(),
            claim: e.claim,
            pointer: e.pointer,
        }
    }
}

#[pymethods]
impl PyEvidence {
    fn __repr__(&self) -> String {
        format!(
            "Evidence(source={:?}, claim='{}', pointer='{}')",
            self.source, self.claim, self.pointer
        )
    }
}

/// Result of evaluating a single rule.
#[pyclass(name = "RuleEvaluation")]
#[derive(Clone)]
pub struct PyRuleEvaluation {
    #[pyo3(get)]
    pub rule_id: String,
    #[pyo3(get)]
    pub rule_text: Option<String>,
    #[pyo3(get)]
    pub result: PyRuleResult,
    #[pyo3(get)]
    pub evidence: Vec<PyEvidence>,
    #[pyo3(get)]
    pub rationale: Option<String>,
}

impl From<core::RuleEvaluation> for PyRuleEvaluation {
    fn from(re: core::RuleEvaluation) -> Self {
        PyRuleEvaluation {
            rule_id: re.rule_id,
            rule_text: re.rule_text,
            result: re.result.into(),
            evidence: re.evidence.into_iter().map(|e| e.into()).collect(),
            rationale: re.rationale,
        }
    }
}

#[pymethods]
impl PyRuleEvaluation {
    fn __repr__(&self) -> String {
        format!(
            "RuleEvaluation(rule_id='{}', result={:?})",
            self.rule_id, self.result
        )
    }
}

/// State of a single lens after evaluation.
#[pyclass(name = "LensState")]
#[derive(Clone)]
pub struct PyLensState {
    state_type: String,
    reason: Option<String>,
    violation: Option<String>,
}

impl From<CoreLensState> for PyLensState {
    fn from(ls: CoreLensState) -> Self {
        match ls {
            CoreLensState::Pass => PyLensState {
                state_type: "Pass".to_string(),
                reason: None,
                violation: None,
            },
            CoreLensState::Escalate { reason } => PyLensState {
                state_type: "Escalate".to_string(),
                reason: Some(reason),
                violation: None,
            },
            CoreLensState::Blocked { violation } => PyLensState {
                state_type: "Blocked".to_string(),
                reason: None,
                violation: Some(violation),
            },
        }
    }
}

#[pymethods]
impl PyLensState {
    /// Check if this lens passed.
    pub fn is_pass(&self) -> bool {
        self.state_type == "Pass"
    }

    /// Check if this lens triggered escalation.
    pub fn is_escalate(&self) -> bool {
        self.state_type == "Escalate"
    }

    /// Check if this lens blocked.
    pub fn is_blocked(&self) -> bool {
        self.state_type == "Blocked"
    }

    /// Get the escalation reason (if escalate state).
    #[getter]
    pub fn reason(&self) -> Option<String> {
        self.reason.clone()
    }

    /// Get the violation description (if blocked state).
    #[getter]
    pub fn violation(&self) -> Option<String> {
        self.violation.clone()
    }

    fn __repr__(&self) -> String {
        match self.state_type.as_str() {
            "Pass" => "LensState.Pass".to_string(),
            "Escalate" => format!(
                "LensState.Escalate(reason='{}')",
                self.reason.as_deref().unwrap_or("")
            ),
            "Blocked" => format!(
                "LensState.Blocked(violation='{}')",
                self.violation.as_deref().unwrap_or("")
            ),
            _ => "LensState.Unknown".to_string(),
        }
    }
}

/// Finding from a single lens evaluation.
#[pyclass(name = "LensFinding")]
#[derive(Clone)]
pub struct PyLensFinding {
    #[pyo3(get)]
    pub lens: Option<PyLensType>,
    #[pyo3(get)]
    pub question_asked: Option<String>,
    #[pyo3(get)]
    pub state: PyLensState,
    #[pyo3(get)]
    pub rules_evaluated: Vec<PyRuleEvaluation>,
    #[pyo3(get)]
    pub confidence: f64,
}

impl From<core::LensFinding> for PyLensFinding {
    fn from(lf: core::LensFinding) -> Self {
        PyLensFinding {
            lens: lf.lens.map(|l| l.into()),
            question_asked: lf.question_asked,
            state: lf.state.into(),
            rules_evaluated: lf.rules_evaluated.into_iter().map(|r| r.into()).collect(),
            confidence: lf.confidence,
        }
    }
}

#[pymethods]
impl PyLensFinding {
    fn __repr__(&self) -> String {
        let lens_str = self
            .lens
            .as_ref()
            .map(|l| format!("{:?}", l))
            .unwrap_or_else(|| "None".to_string());
        format!(
            "LensFinding(lens={}, state='{}', confidence={:.2})",
            lens_str, self.state.state_type, self.confidence
        )
    }
}

/// Findings from all five lenses.
#[pyclass(name = "LensFindings")]
#[derive(Clone)]
pub struct PyLensFindings {
    #[pyo3(get)]
    pub dignity_inclusion: PyLensFinding,
    #[pyo3(get)]
    pub boundaries_safety: PyLensFinding,
    #[pyo3(get)]
    pub restraint_privacy: PyLensFinding,
    #[pyo3(get)]
    pub transparency_contestability: PyLensFinding,
    #[pyo3(get)]
    pub accountability_ownership: PyLensFinding,
}

impl From<core::LensFindings> for PyLensFindings {
    fn from(lf: core::LensFindings) -> Self {
        PyLensFindings {
            dignity_inclusion: lf.dignity_inclusion.into(),
            boundaries_safety: lf.boundaries_safety.into(),
            restraint_privacy: lf.restraint_privacy.into(),
            transparency_contestability: lf.transparency_contestability.into(),
            accountability_ownership: lf.accountability_ownership.into(),
        }
    }
}

#[pymethods]
impl PyLensFindings {
    /// Get all lens findings as a list.
    pub fn all(&self) -> Vec<PyLensFinding> {
        vec![
            self.dignity_inclusion.clone(),
            self.boundaries_safety.clone(),
            self.restraint_privacy.clone(),
            self.transparency_contestability.clone(),
            self.accountability_ownership.clone(),
        ]
    }

    fn __repr__(&self) -> String {
        "LensFindings(5 lenses)".to_string()
    }
}

/// Boundary violation details.
#[pyclass(name = "BoundaryViolation")]
#[derive(Clone)]
pub struct PyBoundaryViolation {
    #[pyo3(get)]
    pub lens: PyLensType,
    #[pyo3(get)]
    pub rule_id: String,
    #[pyo3(get)]
    pub rule_text: String,
    #[pyo3(get)]
    pub evidence: Vec<PyEvidence>,
    #[pyo3(get)]
    pub accountable_human: String,
}

impl From<core::BoundaryViolation> for PyBoundaryViolation {
    fn from(bv: core::BoundaryViolation) -> Self {
        PyBoundaryViolation {
            lens: bv.lens.into(),
            rule_id: bv.rule_id,
            rule_text: bv.rule_text,
            evidence: bv.evidence.into_iter().map(|e| e.into()).collect(),
            accountable_human: bv.accountable_human,
        }
    }
}

#[pymethods]
impl PyBoundaryViolation {
    fn __repr__(&self) -> String {
        format!(
            "BoundaryViolation(rule_id='{}', lens={:?})",
            self.rule_id, self.lens
        )
    }
}

/// The evaluation state: Proceed, Escalate, or Blocked.
#[pyclass(name = "State")]
#[derive(Clone)]
pub struct PyState {
    state_type: String,
    summary: Option<String>,
    uncertainty: Option<String>,
    decision_point: Option<String>,
    options: Option<Vec<String>>,
    violation: Option<PyBoundaryViolation>,
}

impl From<CoreState> for PyState {
    fn from(state: CoreState) -> Self {
        match state {
            CoreState::Proceed { summary } => PyState {
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
            } => PyState {
                state_type: "Escalate".to_string(),
                summary: None,
                uncertainty: Some(uncertainty),
                decision_point: Some(decision_point),
                options: Some(options),
                violation: None,
            },
            CoreState::Blocked { violation } => PyState {
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

#[pymethods]
impl PyState {
    /// Check if state is Proceed.
    pub fn is_proceed(&self) -> bool {
        self.state_type == "Proceed"
    }

    /// Check if state is Escalate.
    pub fn is_escalate(&self) -> bool {
        self.state_type == "Escalate"
    }

    /// Check if state is Blocked.
    pub fn is_blocked(&self) -> bool {
        self.state_type == "Blocked"
    }

    /// Get the summary (for Proceed state).
    #[getter]
    pub fn summary(&self) -> Option<String> {
        self.summary.clone()
    }

    /// Get the uncertainty description (for Escalate state).
    #[getter]
    pub fn uncertainty(&self) -> Option<String> {
        self.uncertainty.clone()
    }

    /// Get the decision point (for Escalate state).
    #[getter]
    pub fn decision_point(&self) -> Option<String> {
        self.decision_point.clone()
    }

    /// Get the options for human decision (for Escalate state).
    #[getter]
    pub fn options(&self) -> Option<Vec<String>> {
        self.options.clone()
    }

    /// Get the boundary violation (for Blocked state).
    #[getter]
    pub fn violation(&self) -> Option<PyBoundaryViolation> {
        self.violation.clone()
    }

    fn __repr__(&self) -> String {
        match self.state_type.as_str() {
            "Proceed" => format!(
                "State.Proceed(summary='{}')",
                self.summary.as_deref().unwrap_or("")
            ),
            "Escalate" => format!(
                "State.Escalate(decision_point='{}')",
                self.decision_point.as_deref().unwrap_or("")
            ),
            "Blocked" => format!(
                "State.Blocked(rule_id='{}')",
                self.violation
                    .as_ref()
                    .map(|v| v.rule_id.as_str())
                    .unwrap_or("")
            ),
            _ => "State.Unknown".to_string(),
        }
    }
}

/// The complete result of evaluating an output against a contract.
#[pyclass(name = "EvaluationResult")]
#[derive(Clone)]
pub struct PyEvaluationResult {
    #[pyo3(get)]
    pub state: PyState,
    #[pyo3(get)]
    pub lens_findings: PyLensFindings,
    #[pyo3(get)]
    pub confidence: f64,
    #[pyo3(get)]
    pub evaluated_at: String,
}

impl From<core::EvaluationResult> for PyEvaluationResult {
    fn from(er: core::EvaluationResult) -> Self {
        PyEvaluationResult {
            state: er.state.into(),
            lens_findings: er.lens_findings.into(),
            confidence: er.confidence,
            evaluated_at: er.evaluated_at.to_rfc3339(),
        }
    }
}

#[pymethods]
impl PyEvaluationResult {
    /// Check if the result is Proceed.
    pub fn is_proceed(&self) -> bool {
        self.state.is_proceed()
    }

    /// Check if the result is Escalate.
    pub fn is_escalate(&self) -> bool {
        self.state.is_escalate()
    }

    /// Check if the result is Blocked.
    pub fn is_blocked(&self) -> bool {
        self.state.is_blocked()
    }

    /// Get the summary (for Proceed state).
    #[getter]
    pub fn summary(&self) -> Option<String> {
        self.state.summary()
    }

    /// Get the decision point (for Escalate state).
    #[getter]
    pub fn decision_point(&self) -> Option<String> {
        self.state.decision_point()
    }

    /// Get the violation (for Blocked state).
    #[getter]
    pub fn violation(&self) -> Option<PyBoundaryViolation> {
        self.state.violation()
    }

    /// Convert the result to a JSON string.
    pub fn to_json(&self) -> PyResult<String> {
        // Serialize using a simplified representation
        let json = serde_json::json!({
            "state": {
                "type": if self.is_proceed() { "Proceed" }
                        else if self.is_escalate() { "Escalate" }
                        else { "Blocked" },
                "summary": self.state.summary,
                "decision_point": self.state.decision_point,
                "options": self.state.options,
                "violation": self.state.violation.as_ref().map(|v| {
                    serde_json::json!({
                        "lens": format!("{:?}", v.lens),
                        "rule_id": v.rule_id,
                        "rule_text": v.rule_text,
                        "accountable_human": v.accountable_human
                    })
                })
            },
            "confidence": self.confidence,
            "evaluated_at": self.evaluated_at
        });
        serde_json::to_string_pretty(&json)
            .map_err(|e| PyRuntimeError::new_err(format!("JSON serialization failed: {}", e)))
    }

    fn __repr__(&self) -> String {
        format!(
            "EvaluationResult(state='{}', confidence={:.2})",
            self.state.state_type, self.confidence
        )
    }
}

/// Custom error type for evaluation errors.
#[pyclass(name = "EvaluationError", extends = pyo3::exceptions::PyException)]
pub struct PyEvaluationError;

/// Custom error type for contract parsing errors.
#[pyclass(name = "ContractError", extends = pyo3::exceptions::PyException)]
pub struct PyContractError;

/// Evaluate an output against a stewardship contract.
///
/// This is the main entry point for Steward evaluation.
///
/// Args:
///     contract: The stewardship contract defining rules
///     output: The AI-generated output to evaluate
///
/// Returns:
///     An EvaluationResult containing the state, lens findings, and confidence.
///
/// Raises:
///     EvaluationError: If evaluation fails
#[pyfunction]
pub fn evaluate(contract: &PyContract, output: &PyOutput) -> PyResult<PyEvaluationResult> {
    core::evaluate(&contract.inner, &output.inner)
        .map(|r| r.into())
        .map_err(|e| PyRuntimeError::new_err(format!("Evaluation failed: {}", e)))
}

/// Evaluate with optional context and metadata.
///
/// Args:
///     contract: The stewardship contract
///     output: The AI-generated output
///     context: Optional list of context strings the AI had access to
///     metadata: Optional dictionary of metadata for the evaluation
///
/// Returns:
///     An EvaluationResult containing the state, lens findings, and confidence.
#[pyfunction]
#[pyo3(signature = (contract, output, context=None, metadata=None))]
pub fn evaluate_with_context(
    contract: &PyContract,
    output: &PyOutput,
    context: Option<Vec<String>>,
    metadata: Option<HashMap<String, String>>,
) -> PyResult<PyEvaluationResult> {
    core::evaluate_with_context(
        &contract.inner,
        &output.inner,
        context.as_deref(),
        metadata.as_ref(),
    )
    .map(|r| r.into())
    .map_err(|e| PyRuntimeError::new_err(format!("Evaluation failed: {}", e)))
}

/// Steward Python module.
#[pymodule]
fn steward(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyContract>()?;
    m.add_class::<PyOutput>()?;
    m.add_class::<PyEvaluationResult>()?;
    m.add_class::<PyState>()?;
    m.add_class::<PyBoundaryViolation>()?;
    m.add_class::<PyLensFindings>()?;
    m.add_class::<PyLensFinding>()?;
    m.add_class::<PyLensState>()?;
    m.add_class::<PyLensType>()?;
    m.add_class::<PyRuleEvaluation>()?;
    m.add_class::<PyRuleResult>()?;
    m.add_class::<PyEvidence>()?;
    m.add_class::<PyEvidenceSource>()?;
    m.add_class::<PyEvaluationError>()?;
    m.add_class::<PyContractError>()?;
    m.add_function(wrap_pyfunction!(evaluate, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_with_context, m)?)?;
    Ok(())
}
