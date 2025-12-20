//! Evidence linking for Steward evaluations.
//!
//! Every finding must be supported by evidence that points to specific
//! locations in the contract, output, or context.

use serde::{Deserialize, Serialize};

use crate::types::EvidenceSource;

/// A piece of evidence supporting an evaluation finding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    /// What this evidence supports
    pub claim: String,

    /// Where the evidence comes from
    pub source: EvidenceSource,

    /// Pointer to the location (e.g., "output.content[47:72]")
    pub pointer: String,
}

impl Evidence {
    /// Create evidence from output content.
    pub fn from_output(claim: impl Into<String>, start: usize, end: usize) -> Self {
        Self {
            claim: claim.into(),
            source: EvidenceSource::Output,
            pointer: format!("output.content[{}:{}]", start, end),
        }
    }

    /// Create evidence from context.
    pub fn from_context(claim: impl Into<String>, index: usize, start: usize, end: usize) -> Self {
        Self {
            claim: claim.into(),
            source: EvidenceSource::Context,
            pointer: format!("context[{}][{}:{}]", index, start, end),
        }
    }

    /// Create evidence from the contract.
    pub fn from_contract(claim: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            claim: claim.into(),
            source: EvidenceSource::Contract,
            pointer: path.into(),
        }
    }

    /// Create evidence from metadata.
    pub fn from_metadata(claim: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            claim: claim.into(),
            source: EvidenceSource::Metadata,
            pointer: format!("metadata.{}", key.into()),
        }
    }
}

/// Builder for creating evidence with fluent API.
pub struct EvidenceBuilder {
    claim: String,
    source: EvidenceSource,
    pointer: String,
}

impl EvidenceBuilder {
    /// Start building evidence with a claim.
    pub fn new(claim: impl Into<String>) -> Self {
        Self {
            claim: claim.into(),
            source: EvidenceSource::Output,
            pointer: String::new(),
        }
    }

    /// Set the source to output.
    pub fn from_output(mut self, start: usize, end: usize) -> Self {
        self.source = EvidenceSource::Output;
        self.pointer = format!("output.content[{}:{}]", start, end);
        self
    }

    /// Set the source to context.
    pub fn from_context(mut self, index: usize, start: usize, end: usize) -> Self {
        self.source = EvidenceSource::Context;
        self.pointer = format!("context[{}][{}:{}]", index, start, end);
        self
    }

    /// Build the evidence.
    pub fn build(self) -> Evidence {
        Evidence {
            claim: self.claim,
            source: self.source,
            pointer: self.pointer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_from_output() {
        let evidence = Evidence::from_output("Email found", 42, 68);
        assert_eq!(evidence.source, EvidenceSource::Output);
        assert_eq!(evidence.pointer, "output.content[42:68]");
    }

    #[test]
    fn test_evidence_from_context() {
        let evidence = Evidence::from_context("Frustration detected", 0, 0, 24);
        assert_eq!(evidence.source, EvidenceSource::Context);
        assert_eq!(evidence.pointer, "context[0][0:24]");
    }

    #[test]
    fn test_evidence_builder() {
        let evidence = EvidenceBuilder::new("PII detected")
            .from_output(100, 150)
            .build();

        assert_eq!(evidence.claim, "PII detected");
        assert_eq!(evidence.pointer, "output.content[100:150]");
    }
}
