//! Contract parsing and validation.
//!
//! Stewardship contracts are structured data validated against JSON Schema.
//! This module handles parsing YAML/JSON contracts and validating them.

mod parser;
mod schema;

pub use parser::{Contract, ContractError, Rule};
