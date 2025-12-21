//! # steward-runtime
//!
//! Optional LLM-assisted evaluation for Steward.
//!
//! This crate provides LLM-based evaluation for rules that require
//! semantic interpretation beyond deterministic pattern matching.
//!
//! ## Key Principle
//!
//! **The core remains deterministic. LLMs assist; they do not decide.**
//!
//! - `steward-core` handles pattern matching deterministically
//! - `steward-runtime` provides optional LLM orchestration
//! - The Synthesizer NEVER makes LLM calls - it applies strict policy rules
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    STEWARD RUNTIME PIPELINE                      │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │  Contract + Output                                               │
//! │       │                                                          │
//! │       ▼                                                          │
//! │  ┌─────────────────────────────────────────────┐                │
//! │  │            PARALLEL FAN-OUT                  │                │
//! │  │         (5 lens agents via tokio::join!)    │                │
//! │  └─────────────────────────────────────────────┘                │
//! │       │                                                          │
//! │       ▼                                                          │
//! │  ┌─────────────────────────────────────────────┐                │
//! │  │            SYNTHESIZER                       │                │
//! │  │     (STRICT POLICY MACHINE - NO LLM)        │                │
//! │  │                                              │                │
//! │  │  1. ANY BLOCKED → BLOCKED                   │                │
//! │  │  2. ELSE ANY ESCALATE → ESCALATE            │                │
//! │  │  3. ELSE confidence < 0.4 → ESCALATE        │                │
//! │  │  4. ELSE → PROCEED                          │                │
//! │  └─────────────────────────────────────────────┘                │
//! │       │                                                          │
//! │       ▼                                                          │
//! │  EvaluationResult + LlmUsage                                    │
//! │                                                                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Evidence Validation
//!
//! LLM agents produce **evidence**, not **verdicts**. The runtime validates:
//! 1. JSON schema compliance
//! 2. Pointer ranges are in-bounds
//! 3. Quotes match referenced slices exactly
//!
//! On validation failure: fallback to deterministic (never "best-effort parse").
//!
//! ## Example
//!
//! ```rust,ignore
//! use steward_runtime::{RuntimeOrchestrator, RuntimeConfig};
//! use steward_runtime::providers::AnthropicProvider;
//! use std::sync::Arc;
//!
//! // Create provider
//! let provider = Arc::new(AnthropicProvider::from_env()?);
//!
//! // Create orchestrator
//! let orchestrator = RuntimeOrchestrator::new(provider, RuntimeConfig::default());
//!
//! // Evaluate
//! let result = orchestrator.evaluate(&contract, &output, None).await?;
//! println!("State: {:?}", result.evaluation.state);
//! println!("LLM calls: {}", result.llm_usage.llm_calls);
//! ```

pub mod agents;
pub mod cache;
pub mod config;
pub mod evidence;
pub mod orchestrator;
pub mod prompts;
pub mod providers;
pub mod resilience;
pub mod synthesizer;

// Re-exports
pub use agents::{AgentError, LensAgent};
pub use cache::{CacheKey, EvaluationCache};
pub use config::RuntimeConfig;
pub use evidence::{EvidenceValidationError, EvidenceValidator};
pub use orchestrator::{RuntimeError, RuntimeOrchestrator, RuntimeOrchestratorBuilder, RuntimeResult};
pub use prompts::{get_lens_prompt, BASE_SYSTEM_PROMPT};
#[cfg(feature = "anthropic")]
pub use providers::AnthropicProvider;
pub use providers::{
    ChatMessage, CompletionConfig, CompletionResponse, LlmProvider,
    ProviderError, ProviderType, TokenUsage,
};
pub use resilience::{BudgetTracker, CircuitBreaker, CircuitBreakerConfig, FallbackStrategy, LlmUsage};
pub use synthesizer::{ExtensionManager, SynthesizerMetadataExtension};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_compiles() {
        // Just verify the crate compiles with all modules
        let _ = RuntimeConfig::default();
    }
}
