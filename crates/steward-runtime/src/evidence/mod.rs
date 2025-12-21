//! Evidence validation for LLM output.
//!
//! # Core Principle
//! "LLMs assist; they do not decide."
//!
//! LLM agents produce **evidence**, not **verdicts**. The runtime MUST validate
//! all LLM output before accepting it. Invalid evidence triggers fallback to
//! deterministic evaluation — never "best-effort parse".

mod validator;

pub use validator::{EvidenceValidator, EvidenceValidationError};
