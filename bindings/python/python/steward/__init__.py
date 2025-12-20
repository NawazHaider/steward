"""
Steward - Deterministic stewardship contract evaluation engine.

This package provides Python bindings for Steward, a tool that evaluates
AI outputs against human-defined stewardship contracts.

Example:
    >>> from steward import Contract, Output, evaluate
    >>> contract = Contract.from_yaml_file("contract.yaml")
    >>> output = Output.text("The AI generated this response")
    >>> result = evaluate(contract, output)
    >>> if result.is_blocked():
    ...     print(f"BLOCKED: {result.violation.rule_id}")
    ... elif result.is_escalate():
    ...     print(f"ESCALATE: {result.decision_point}")
    ... else:
    ...     print(f"PROCEED: {result.summary}")
"""

# Import from the native Rust extension
from steward.steward import (
    Contract,
    Output,
    EvaluationResult,
    State,
    BoundaryViolation,
    LensFindings,
    LensFinding,
    LensState,
    LensType,
    RuleEvaluation,
    RuleResult,
    Evidence,
    EvidenceSource,
    evaluate,
    evaluate_with_context,
    EvaluationError,
    ContractError,
)

__version__ = "0.1.0"
__author__ = "Patrick Peña - Agenisea AI"
__email__ = "patrick@agenisea.ai"

__all__ = [
    # Core types
    "Contract",
    "Output",
    "EvaluationResult",
    "State",
    "BoundaryViolation",
    # Lens types
    "LensFindings",
    "LensFinding",
    "LensState",
    "LensType",
    # Rule types
    "RuleEvaluation",
    "RuleResult",
    # Evidence types
    "Evidence",
    "EvidenceSource",
    # Functions
    "evaluate",
    "evaluate_with_context",
    # Errors
    "EvaluationError",
    "ContractError",
]
