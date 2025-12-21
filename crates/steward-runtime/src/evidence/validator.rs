//! Evidence validation ensures LLM output has referential integrity.
//!
//! LLMs produce EVIDENCE, not VERDICTS.
//! If evidence is invalid, we fallback — we never "best-effort parse".

use regex::Regex;
use steward_core::{Evidence, Output};
use thiserror::Error;

/// Errors from evidence validation.
#[derive(Error, Debug, Clone)]
pub enum EvidenceValidationError {
    #[error("Pointer out of bounds: {pointer} (requested end {requested_end}, actual length {actual_length})")]
    PointerOutOfBounds {
        pointer: String,
        actual_length: usize,
        requested_end: usize,
    },

    #[error("Context index out of bounds: {pointer} (index {index}, context length {context_length})")]
    ContextIndexOutOfBounds {
        pointer: String,
        index: usize,
        context_length: usize,
    },

    #[error("Unknown evidence source: {pointer}")]
    UnknownSource { pointer: String },

    #[error("Quote mismatch at {pointer}: expected '{expected}', found '{actual}'")]
    QuoteMismatch {
        pointer: String,
        expected: String,
        actual: String,
    },

    #[error("Invalid pointer format: {pointer}")]
    InvalidPointerFormat { pointer: String },

    #[error("Missing required evidence for rule {rule_id}")]
    MissingEvidence { rule_id: String },
}

/// Evidence validator ensures LLM output has referential integrity.
///
/// # Validation Steps
/// 1. JSON schema compliance (structure is valid)
/// 2. Pointer ranges are in-bounds
/// 3. Quotes match referenced slices exactly
///
/// # On Failure
/// Caller MUST fallback to deterministic evaluation.
pub struct EvidenceValidator<'a> {
    output: &'a Output,
    context: &'a [String],
    pointer_regex: Regex,
}

impl<'a> EvidenceValidator<'a> {
    /// Create a new evidence validator.
    pub fn new(output: &'a Output, context: &'a [String]) -> Self {
        // Pattern: source[start:end] or source[index][start:end]
        let pointer_regex = Regex::new(
            r"^(?P<source>\w+(?:\.\w+)*|\w+\[\d+\])\[(?P<start>\d+):(?P<end>\d+)\]$"
        ).expect("Invalid regex");

        Self {
            output,
            context,
            pointer_regex,
        }
    }

    /// Validate all evidence in a list.
    pub fn validate_all(&self, evidence: &[Evidence]) -> Result<(), EvidenceValidationError> {
        for e in evidence {
            self.validate(e)?;
        }
        Ok(())
    }

    /// Validate a single evidence item.
    pub fn validate(&self, evidence: &Evidence) -> Result<(), EvidenceValidationError> {
        self.validate_pointer(&evidence.pointer)?;
        self.validate_quote(&evidence.pointer, &evidence.claim)?;
        Ok(())
    }

    /// Validate that a pointer is in-bounds.
    fn validate_pointer(&self, pointer: &str) -> Result<(), EvidenceValidationError> {
        let (source, range) = self.parse_pointer(pointer)?;

        match source.as_str() {
            "output.content" | "output" => {
                if range.end > self.output.content.len() {
                    return Err(EvidenceValidationError::PointerOutOfBounds {
                        pointer: pointer.to_string(),
                        actual_length: self.output.content.len(),
                        requested_end: range.end,
                    });
                }
            }
            s if s.starts_with("context[") => {
                let index = self.parse_context_index(s)?;
                if index >= self.context.len() {
                    return Err(EvidenceValidationError::ContextIndexOutOfBounds {
                        pointer: pointer.to_string(),
                        index,
                        context_length: self.context.len(),
                    });
                }
                if range.end > self.context[index].len() {
                    return Err(EvidenceValidationError::PointerOutOfBounds {
                        pointer: pointer.to_string(),
                        actual_length: self.context[index].len(),
                        requested_end: range.end,
                    });
                }
            }
            _ => {
                return Err(EvidenceValidationError::UnknownSource {
                    pointer: pointer.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate that a quote matches the referenced slice.
    fn validate_quote(&self, pointer: &str, quote: &str) -> Result<(), EvidenceValidationError> {
        let actual_slice = self.extract_slice(pointer)?;

        // Normalize whitespace for comparison
        let normalized_actual = normalize_whitespace(&actual_slice);
        let normalized_quote = normalize_whitespace(quote);

        if normalized_actual != normalized_quote {
            return Err(EvidenceValidationError::QuoteMismatch {
                pointer: pointer.to_string(),
                expected: quote.to_string(),
                actual: actual_slice,
            });
        }

        Ok(())
    }

    /// Parse a pointer into source and range.
    fn parse_pointer(&self, pointer: &str) -> Result<(String, std::ops::Range<usize>), EvidenceValidationError> {
        // Try regex match first
        if let Some(caps) = self.pointer_regex.captures(pointer) {
            let source = caps.name("source").unwrap().as_str().to_string();
            let start: usize = caps.name("start").unwrap().as_str().parse()
                .map_err(|_| EvidenceValidationError::InvalidPointerFormat {
                    pointer: pointer.to_string(),
                })?;
            let end: usize = caps.name("end").unwrap().as_str().parse()
                .map_err(|_| EvidenceValidationError::InvalidPointerFormat {
                    pointer: pointer.to_string(),
                })?;

            if start > end {
                return Err(EvidenceValidationError::InvalidPointerFormat {
                    pointer: pointer.to_string(),
                });
            }

            return Ok((source, start..end));
        }

        Err(EvidenceValidationError::InvalidPointerFormat {
            pointer: pointer.to_string(),
        })
    }

    /// Parse context index from a source like "context[0]".
    fn parse_context_index(&self, source: &str) -> Result<usize, EvidenceValidationError> {
        let re = Regex::new(r"context\[(\d+)\]").expect("Invalid regex");
        if let Some(caps) = re.captures(source) {
            let index: usize = caps.get(1).unwrap().as_str().parse()
                .map_err(|_| EvidenceValidationError::InvalidPointerFormat {
                    pointer: source.to_string(),
                })?;
            return Ok(index);
        }
        Err(EvidenceValidationError::InvalidPointerFormat {
            pointer: source.to_string(),
        })
    }

    /// Extract the slice at a pointer location.
    fn extract_slice(&self, pointer: &str) -> Result<String, EvidenceValidationError> {
        let (source, range) = self.parse_pointer(pointer)?;

        match source.as_str() {
            "output.content" | "output" => {
                if range.end > self.output.content.len() {
                    return Err(EvidenceValidationError::PointerOutOfBounds {
                        pointer: pointer.to_string(),
                        actual_length: self.output.content.len(),
                        requested_end: range.end,
                    });
                }
                Ok(self.output.content[range].to_string())
            }
            s if s.starts_with("context[") => {
                let index = self.parse_context_index(s)?;
                if index >= self.context.len() {
                    return Err(EvidenceValidationError::ContextIndexOutOfBounds {
                        pointer: pointer.to_string(),
                        index,
                        context_length: self.context.len(),
                    });
                }
                if range.end > self.context[index].len() {
                    return Err(EvidenceValidationError::PointerOutOfBounds {
                        pointer: pointer.to_string(),
                        actual_length: self.context[index].len(),
                        requested_end: range.end,
                    });
                }
                Ok(self.context[index][range].to_string())
            }
            _ => Err(EvidenceValidationError::UnknownSource {
                pointer: pointer.to_string(),
            }),
        }
    }
}

/// Normalize whitespace for quote comparison.
fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use steward_core::EvidenceSource;

    fn make_output(content: &str) -> Output {
        Output::text(content)
    }

    fn make_evidence(pointer: &str, claim: &str) -> Evidence {
        Evidence {
            claim: claim.to_string(),
            source: EvidenceSource::Output,
            pointer: pointer.to_string(),
        }
    }

    #[test]
    fn test_valid_pointer() {
        let output = make_output("Hello, world!");
        let context: Vec<String> = vec![];
        let validator = EvidenceValidator::new(&output, &context);

        let evidence = make_evidence("output.content[0:5]", "Hello");
        assert!(validator.validate(&evidence).is_ok());
    }

    #[test]
    fn test_pointer_out_of_bounds() {
        let output = make_output("Hello");
        let context: Vec<String> = vec![];
        let validator = EvidenceValidator::new(&output, &context);

        let evidence = make_evidence("output.content[0:100]", "Hello");
        let result = validator.validate(&evidence);

        assert!(matches!(
            result,
            Err(EvidenceValidationError::PointerOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_quote_mismatch() {
        let output = make_output("Hello, world!");
        let context: Vec<String> = vec![];
        let validator = EvidenceValidator::new(&output, &context);

        let evidence = make_evidence("output.content[0:5]", "Goodbye");
        let result = validator.validate(&evidence);

        assert!(matches!(
            result,
            Err(EvidenceValidationError::QuoteMismatch { .. })
        ));
    }

    #[test]
    fn test_context_pointer() {
        let output = make_output("Output text");
        let context = vec!["Context item 0".to_string(), "Context item 1".to_string()];
        let validator = EvidenceValidator::new(&output, &context);

        let evidence = make_evidence("context[0][0:7]", "Context");
        assert!(validator.validate(&evidence).is_ok());
    }

    #[test]
    fn test_context_index_out_of_bounds() {
        let output = make_output("Output text");
        let context = vec!["Only one item".to_string()];
        let validator = EvidenceValidator::new(&output, &context);

        let evidence = make_evidence("context[5][0:5]", "Hello");
        let result = validator.validate(&evidence);

        assert!(matches!(
            result,
            Err(EvidenceValidationError::ContextIndexOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_unknown_source() {
        let output = make_output("Hello");
        let context: Vec<String> = vec![];
        let validator = EvidenceValidator::new(&output, &context);

        let evidence = make_evidence("unknown[0:5]", "Hello");
        let result = validator.validate(&evidence);

        assert!(matches!(
            result,
            Err(EvidenceValidationError::UnknownSource { .. })
        ));
    }

    #[test]
    fn test_invalid_pointer_format() {
        let output = make_output("Hello");
        let context: Vec<String> = vec![];
        let validator = EvidenceValidator::new(&output, &context);

        let evidence = make_evidence("not-a-pointer", "Hello");
        let result = validator.validate(&evidence);

        assert!(matches!(
            result,
            Err(EvidenceValidationError::InvalidPointerFormat { .. })
        ));
    }

    #[test]
    fn test_whitespace_normalization() {
        let output = make_output("Hello   world");
        let context: Vec<String> = vec![];
        let validator = EvidenceValidator::new(&output, &context);

        // Quote has different whitespace but should match after normalization
        let evidence = make_evidence("output.content[0:13]", "Hello world");
        assert!(validator.validate(&evidence).is_ok());
    }
}
