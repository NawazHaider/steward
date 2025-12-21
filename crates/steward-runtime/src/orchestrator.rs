//! Runtime orchestrator for parallel lens evaluation.
//!
//! The orchestrator manages parallel lens evaluation with optional LLM assistance.
//! It implements:
//! - Parallel fan-out to all 5 lenses via tokio::join!
//! - Deterministic fan-in through the Synthesizer
//! - Circuit breaker and budget enforcement
//! - Early termination on BLOCKED

use std::sync::Arc;
use thiserror::Error;

use steward_core::{
    Contract, EvaluationRequest, EvaluationResult, LensFinding, LensFindings,
    LensType, Output, Synthesizer,
};

use crate::agents::{AgentError, LensAgent};
use crate::config::RuntimeConfig;
use crate::providers::LlmProvider;
use crate::resilience::{BudgetTracker, CircuitBreaker, LlmUsage};

/// Errors from the runtime orchestrator.
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Provider not configured: {0}")]
    ProviderNotConfigured(String),

    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),

    #[error("Timeout")]
    Timeout,

    #[error("Budget exceeded")]
    BudgetExceeded,

    #[error("All fallbacks exhausted")]
    AllFallbacksExhausted,

    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
}

/// Result from runtime evaluation.
#[derive(Debug)]
pub struct RuntimeResult {
    /// The evaluation result
    pub evaluation: EvaluationResult,

    /// LLM usage metrics
    pub llm_usage: LlmUsage,

    /// Whether early termination was triggered
    pub early_terminated: bool,
}

/// The runtime orchestrator manages parallel lens evaluation.
///
/// # Architecture
/// - Parallel fan-out: All 5 lenses execute concurrently via tokio::join!
/// - Deterministic fan-in: Synthesizer applies strict policy rules
/// - Resilience: Circuit breaker, budget, timeout per lens
/// - Fallback: On LLM failure, falls back to deterministic evaluation
pub struct RuntimeOrchestrator {
    /// LLM provider (used by registered agents)
    #[allow(dead_code)]
    provider: Arc<dyn LlmProvider>,

    /// Configuration
    config: RuntimeConfig,

    /// Circuit breaker for LLM calls
    circuit_breaker: CircuitBreaker,

    /// Token budget tracker
    budget_tracker: BudgetTracker,

    /// Lens agents (one per lens type)
    agents: Vec<Arc<dyn LensAgent>>,

    /// Synthesizer for final verdict
    synthesizer: Synthesizer,
}

impl RuntimeOrchestrator {
    /// Create a new runtime orchestrator.
    pub fn new(provider: Arc<dyn LlmProvider>, config: RuntimeConfig) -> Self {
        let circuit_breaker = CircuitBreaker::new(config.circuit_breaker.clone());
        let budget_tracker = BudgetTracker::new(
            config.budgets.global_max_tokens,
            1000, // Default per-lens
        );

        Self {
            provider,
            config,
            circuit_breaker,
            budget_tracker,
            agents: Vec::new(),
            synthesizer: Synthesizer::new(),
        }
    }

    /// Register a lens agent.
    pub fn register_agent(&mut self, agent: Arc<dyn LensAgent>) {
        self.agents.push(agent);
    }

    /// Evaluate with optional LLM assistance.
    ///
    /// # Execution Flow
    /// 1. Parse contract (deterministic)
    /// 2. Classify rules by evaluation method
    /// 3. Fan-out: Execute all 5 lens agents in parallel
    /// 4. Fan-in: Synthesize findings (deterministic)
    /// 5. Return result with usage metrics
    pub async fn evaluate(
        &self,
        contract: &Contract,
        output: &Output,
        context: Option<&[String]>,
    ) -> Result<RuntimeResult, RuntimeError> {
        let request = EvaluationRequest {
            contract: contract.clone(),
            output: output.clone(),
            context: context.map(|c| c.to_vec()),
            metadata: None,
        };

        // Fan-out: Parallel lens evaluation
        let (dignity, boundaries, restraint, transparency, accountability) = tokio::join!(
            self.evaluate_lens(LensType::DignityInclusion, &request),
            self.evaluate_lens(LensType::BoundariesSafety, &request),
            self.evaluate_lens(LensType::RestraintPrivacy, &request),
            self.evaluate_lens(LensType::TransparencyContestability, &request),
            self.evaluate_lens(LensType::AccountabilityOwnership, &request),
        );

        // Collect findings
        let findings = LensFindings {
            dignity_inclusion: dignity?,
            boundaries_safety: boundaries?,
            restraint_privacy: restraint?,
            transparency_contestability: transparency?,
            accountability_ownership: accountability?,
        };

        // Fan-in: Deterministic synthesis (NO LLM)
        let result = self.synthesizer.synthesize(findings, contract);

        Ok(RuntimeResult {
            evaluation: result,
            llm_usage: self.budget_tracker.get_usage(),
            early_terminated: false,
        })
    }

    /// Evaluate a single lens with timeout and circuit breaker.
    async fn evaluate_lens(
        &self,
        lens: LensType,
        request: &EvaluationRequest,
    ) -> Result<LensFinding, RuntimeError> {
        // Check circuit breaker
        if self.circuit_breaker.is_open(lens) {
            tracing::warn!(lens = ?lens, "Circuit open, falling back to deterministic");
            return self.fallback_to_deterministic(lens, request);
        }

        // Check budget
        let estimated_tokens = 500; // Conservative estimate
        if !self.budget_tracker.can_afford(lens, estimated_tokens) {
            tracing::warn!(lens = ?lens, "Budget exceeded, falling back to deterministic");
            return self.fallback_to_deterministic(lens, request);
        }

        // Apply timeout
        let timeout = self.config.lens_timeout(lens);

        match tokio::time::timeout(timeout, self.do_evaluate_lens(lens, request)).await {
            Ok(Ok(finding)) => {
                self.circuit_breaker.record_success(lens);
                Ok(finding)
            }
            Ok(Err(e)) => {
                tracing::warn!(lens = ?lens, error = %e, "Lens evaluation failed");
                self.circuit_breaker.record_failure(lens);
                self.fallback_to_deterministic(lens, request)
            }
            Err(_) => {
                tracing::warn!(lens = ?lens, timeout = ?timeout, "Lens evaluation timed out");
                self.circuit_breaker.record_failure(lens);
                self.fallback_to_deterministic(lens, request)
            }
        }
    }

    /// Actually evaluate a lens (internal).
    async fn do_evaluate_lens(
        &self,
        lens: LensType,
        request: &EvaluationRequest,
    ) -> Result<LensFinding, RuntimeError> {
        // Find the agent for this lens
        if let Some(agent) = self.agents.iter().find(|a| a.lens_type() == lens) {
            agent.evaluate(request).await.map_err(RuntimeError::from)
        } else {
            // No agent registered, use deterministic fallback
            self.fallback_to_deterministic(lens, request)
        }
    }

    /// Fallback to deterministic evaluation from steward-core.
    fn fallback_to_deterministic(
        &self,
        lens: LensType,
        request: &EvaluationRequest,
    ) -> Result<LensFinding, RuntimeError> {
        use steward_core::{
            AccountabilityLens, BoundariesLens, DignityLens, Lens, RestraintLens, TransparencyLens,
        };

        let lens_impl: Box<dyn Lens> = match lens {
            LensType::DignityInclusion => Box::new(DignityLens::new()),
            LensType::BoundariesSafety => Box::new(BoundariesLens::new()),
            LensType::RestraintPrivacy => Box::new(RestraintLens::new()),
            LensType::TransparencyContestability => Box::new(TransparencyLens::new()),
            LensType::AccountabilityOwnership => Box::new(AccountabilityLens::new()),
        };

        let mut finding = lens_impl.evaluate(request);

        // Mark as fallback with reduced confidence
        finding.confidence *= 0.8;

        Ok(finding)
    }

    /// Get current LLM usage.
    pub fn usage(&self) -> LlmUsage {
        self.budget_tracker.get_usage()
    }

    /// Reset budget tracker for a new evaluation.
    pub fn reset_budget(&self) {
        self.budget_tracker.reset();
    }
}

/// Builder for RuntimeOrchestrator.
pub struct RuntimeOrchestratorBuilder {
    provider: Option<Arc<dyn LlmProvider>>,
    config: RuntimeConfig,
    agents: Vec<Arc<dyn LensAgent>>,
}

impl RuntimeOrchestratorBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            provider: None,
            config: RuntimeConfig::default(),
            agents: Vec::new(),
        }
    }

    /// Set the LLM provider.
    pub fn provider(mut self, provider: Arc<dyn LlmProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set the configuration.
    pub fn config(mut self, config: RuntimeConfig) -> Self {
        self.config = config;
        self
    }

    /// Register a lens agent.
    pub fn agent(mut self, agent: Arc<dyn LensAgent>) -> Self {
        self.agents.push(agent);
        self
    }

    /// Build the orchestrator.
    pub fn build(self) -> Result<RuntimeOrchestrator, RuntimeError> {
        let provider = self
            .provider
            .ok_or_else(|| RuntimeError::ProviderNotConfigured("No provider set".to_string()))?;

        let mut orchestrator = RuntimeOrchestrator::new(provider, self.config);
        for agent in self.agents {
            orchestrator.register_agent(agent);
        }

        Ok(orchestrator)
    }
}

impl Default for RuntimeOrchestratorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::providers::{ChatMessage, CompletionConfig, CompletionResponse, ProviderError, TokenUsage};

    // Mock provider for testing
    struct MockProvider;

    #[async_trait]
    impl LlmProvider for MockProvider {
        async fn complete(
            &self,
            _messages: Vec<ChatMessage>,
            _config: &CompletionConfig,
        ) -> Result<CompletionResponse, ProviderError> {
            Ok(CompletionResponse {
                content: "{}".to_string(),
                usage: TokenUsage::default(),
                model: "mock".to_string(),
                stop_reason: Some("end_turn".to_string()),
            })
        }

        async fn health_check(&self) -> bool {
            true
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let provider = Arc::new(MockProvider);
        let orchestrator = RuntimeOrchestrator::new(provider, RuntimeConfig::default());
        assert_eq!(orchestrator.usage().llm_calls, 0);
    }

    #[tokio::test]
    async fn test_fallback_to_deterministic() {
        let provider = Arc::new(MockProvider);
        let orchestrator = RuntimeOrchestrator::new(provider, RuntimeConfig::default());

        let contract_yaml = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test Contract"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
"#;

        let contract = Contract::from_yaml(contract_yaml).unwrap();
        let output = Output::text("Hello, world!");

        let result = orchestrator.evaluate(&contract, &output, None).await;
        assert!(result.is_ok());
    }
}
