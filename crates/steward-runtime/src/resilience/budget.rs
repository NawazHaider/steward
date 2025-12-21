//! Token budget management for LLM calls.
//!
//! Enforces per-lens and global token budgets to control costs.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use steward_core::LensType;

use crate::providers::TokenUsage;

/// Token budget for a scope (lens or global).
pub struct TokenBudget {
    /// Maximum tokens allowed
    pub max_tokens: u32,

    /// Currently used tokens
    used: AtomicU32,
}

impl TokenBudget {
    /// Create a new token budget.
    pub fn new(max_tokens: u32) -> Self {
        Self {
            max_tokens,
            used: AtomicU32::new(0),
        }
    }

    /// Check if we can afford to use tokens.
    pub fn can_afford(&self, tokens: u32) -> bool {
        self.remaining() >= tokens
    }

    /// Record token usage.
    pub fn record(&self, tokens: u32) {
        self.used.fetch_add(tokens, Ordering::SeqCst);
    }

    /// Get remaining tokens.
    pub fn remaining(&self) -> u32 {
        self.max_tokens.saturating_sub(self.used.load(Ordering::SeqCst))
    }

    /// Get used tokens.
    pub fn used(&self) -> u32 {
        self.used.load(Ordering::SeqCst)
    }

    /// Reset the budget.
    pub fn reset(&self) {
        self.used.store(0, Ordering::SeqCst);
    }
}

/// Accumulated LLM usage for an evaluation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmUsage {
    /// Total tokens used
    pub total_tokens: u32,

    /// Prompt/input tokens
    pub prompt_tokens: u32,

    /// Completion/output tokens
    pub completion_tokens: u32,

    /// Number of LLM calls made
    pub llm_calls: u32,

    /// Estimated cost in USD
    pub estimated_cost: f64,

    /// Cache hits (Anthropic)
    pub cache_hits: u32,

    /// Tokens written to cache
    pub cache_creation_tokens: u32,

    /// Tokens read from cache
    pub cache_read_tokens: u32,
}

impl LlmUsage {
    /// Add token usage from a provider response.
    pub fn add(&mut self, usage: &TokenUsage, model: &str) {
        self.prompt_tokens += usage.prompt_tokens;
        self.completion_tokens += usage.completion_tokens;
        self.total_tokens += usage.total();
        self.llm_calls += 1;
        self.cache_creation_tokens += usage.cache_creation_tokens;
        self.cache_read_tokens += usage.cache_read_tokens;

        if usage.cache_read_tokens > 0 {
            self.cache_hits += 1;
        }

        // Estimate cost based on model
        self.estimated_cost += Self::estimate_cost(usage, model);
    }

    /// Estimate cost for a usage entry.
    fn estimate_cost(usage: &TokenUsage, model: &str) -> f64 {
        // Pricing per million tokens (as of Dec 2025)
        let (input_rate, output_rate, cache_write_rate, cache_read_rate) = match model {
            m if m.contains("sonnet-4-5") => (3.0, 15.0, 3.75, 0.3),
            m if m.contains("opus-4-5") => (5.0, 25.0, 6.25, 0.5),
            m if m.contains("haiku-4-5") => (1.0, 5.0, 1.25, 0.1),
            m if m.contains("gpt-4o-mini") => (0.15, 0.6, 0.0, 0.0),
            m if m.contains("gpt-4o") => (2.5, 10.0, 0.0, 0.0),
            _ => (3.0, 15.0, 3.75, 0.3), // Default to Sonnet pricing
        };

        let input_cost = (usage.prompt_tokens as f64 / 1_000_000.0) * input_rate;
        let output_cost = (usage.completion_tokens as f64 / 1_000_000.0) * output_rate;
        let cache_write_cost = (usage.cache_creation_tokens as f64 / 1_000_000.0) * cache_write_rate;
        let cache_read_cost = (usage.cache_read_tokens as f64 / 1_000_000.0) * cache_read_rate;

        input_cost + output_cost + cache_write_cost + cache_read_cost
    }
}

/// Budget tracker for the entire evaluation.
pub struct BudgetTracker {
    /// Per-lens budgets
    lens_budgets: HashMap<LensType, TokenBudget>,

    /// Global budget for entire evaluation
    global_budget: TokenBudget,

    /// Accumulated usage
    usage: RwLock<LlmUsage>,
}

impl BudgetTracker {
    /// Create a new budget tracker with default budgets.
    pub fn new(global_max: u32, per_lens_max: u32) -> Self {
        let mut lens_budgets = HashMap::new();

        for lens in [
            LensType::DignityInclusion,
            LensType::BoundariesSafety,
            LensType::RestraintPrivacy,
            LensType::TransparencyContestability,
            LensType::AccountabilityOwnership,
        ] {
            lens_budgets.insert(lens, TokenBudget::new(per_lens_max));
        }

        Self {
            lens_budgets,
            global_budget: TokenBudget::new(global_max),
            usage: RwLock::new(LlmUsage::default()),
        }
    }

    /// Create with custom per-lens budgets.
    pub fn with_lens_budgets(global_max: u32, budgets: HashMap<LensType, u32>) -> Self {
        let lens_budgets = budgets
            .into_iter()
            .map(|(lens, max)| (lens, TokenBudget::new(max)))
            .collect();

        Self {
            lens_budgets,
            global_budget: TokenBudget::new(global_max),
            usage: RwLock::new(LlmUsage::default()),
        }
    }

    /// Check if we can afford a call for a lens.
    pub fn can_afford(&self, lens: LensType, estimated_tokens: u32) -> bool {
        let lens_ok = self
            .lens_budgets
            .get(&lens)
            .map(|b| b.can_afford(estimated_tokens))
            .unwrap_or(true);

        let global_ok = self.global_budget.can_afford(estimated_tokens);

        lens_ok && global_ok
    }

    /// Record usage after a call.
    pub fn record_usage(&self, lens: LensType, usage: &TokenUsage, model: &str) {
        let total = usage.total();

        // Record to lens budget
        if let Some(budget) = self.lens_budgets.get(&lens) {
            budget.record(total);
        }

        // Record to global budget
        self.global_budget.record(total);

        // Update accumulated usage
        self.usage.write().add(usage, model);
    }

    /// Get current usage.
    pub fn get_usage(&self) -> LlmUsage {
        self.usage.read().clone()
    }

    /// Get remaining global budget.
    pub fn remaining_global(&self) -> u32 {
        self.global_budget.remaining()
    }

    /// Get remaining budget for a lens.
    pub fn remaining_lens(&self, lens: LensType) -> u32 {
        self.lens_budgets
            .get(&lens)
            .map(|b| b.remaining())
            .unwrap_or(0)
    }

    /// Reset all budgets.
    pub fn reset(&self) {
        for budget in self.lens_budgets.values() {
            budget.reset();
        }
        self.global_budget.reset();
        *self.usage.write() = LlmUsage::default();
    }
}

impl Default for BudgetTracker {
    fn default() -> Self {
        Self::new(5000, 1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_enforcement() {
        let budget = TokenBudget::new(100);

        assert!(budget.can_afford(50));
        assert!(budget.can_afford(100));
        assert!(!budget.can_afford(101));

        budget.record(60);
        assert_eq!(budget.remaining(), 40);
        assert!(!budget.can_afford(50));
        assert!(budget.can_afford(40));
    }

    #[test]
    fn test_budget_tracker() {
        let tracker = BudgetTracker::new(500, 100);

        // Should be able to afford initial calls
        assert!(tracker.can_afford(LensType::DignityInclusion, 50));

        // Record usage
        let usage = TokenUsage {
            prompt_tokens: 30,
            completion_tokens: 20,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
        };
        tracker.record_usage(LensType::DignityInclusion, &usage, "claude-sonnet-4-5");

        // Check remaining
        assert_eq!(tracker.remaining_lens(LensType::DignityInclusion), 50);
        assert_eq!(tracker.remaining_global(), 450);

        // Should not be able to afford more than remaining
        assert!(!tracker.can_afford(LensType::DignityInclusion, 60));
    }

    #[test]
    fn test_cost_estimation() {
        let mut usage = LlmUsage::default();

        let token_usage = TokenUsage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
        };

        usage.add(&token_usage, "claude-sonnet-4-5");

        // 1000 input tokens * $3/MTok = $0.003
        // 500 output tokens * $15/MTok = $0.0075
        // Total: ~$0.0105
        assert!(usage.estimated_cost > 0.01 && usage.estimated_cost < 0.02);
    }
}
