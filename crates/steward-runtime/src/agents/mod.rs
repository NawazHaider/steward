//! Lens agents for LLM-assisted evaluation.
//!
//! Each lens has an agent that can use LLM for semantic interpretation
//! when deterministic pattern matching is insufficient.

mod traits;

pub use traits::{LensAgent, AgentError};
