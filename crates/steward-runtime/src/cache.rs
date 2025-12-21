//! Caching layer for steward-runtime.
//!
//! Provides in-memory caching of evaluation results to reduce LLM costs
//! for repeated evaluations with identical inputs.

use moka::future::Cache;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use steward_core::{Contract, LensFinding, LensType, Output};

/// Cache key for evaluation results.
#[derive(Clone, Debug)]
pub struct CacheKey {
    contract_hash: u64,
    output_hash: u64,
    context_hash: u64,
    lens: LensType,
}

impl CacheKey {
    /// Create a cache key from evaluation inputs.
    pub fn new(
        contract: &Contract,
        output: &Output,
        context: Option<&[String]>,
        lens: LensType,
    ) -> Self {
        Self {
            contract_hash: hash_contract(contract),
            output_hash: hash_output(output),
            context_hash: hash_context(context),
            lens,
        }
    }
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.contract_hash.hash(state);
        self.output_hash.hash(state);
        self.context_hash.hash(state);
        self.lens.hash(state);
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.contract_hash == other.contract_hash
            && self.output_hash == other.output_hash
            && self.context_hash == other.context_hash
            && self.lens == other.lens
    }
}

impl Eq for CacheKey {}

/// Evaluation cache using moka.
pub struct EvaluationCache {
    cache: Cache<CacheKey, LensFinding>,
}

impl EvaluationCache {
    /// Create a new cache with the given configuration.
    pub fn new(max_entries: u64, ttl: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_entries)
            .time_to_live(ttl)
            .build();

        Self { cache }
    }

    /// Get a cached finding.
    pub async fn get(&self, key: &CacheKey) -> Option<LensFinding> {
        self.cache.get(key).await
    }

    /// Store a finding in the cache.
    pub async fn insert(&self, key: CacheKey, finding: LensFinding) {
        self.cache.insert(key, finding).await;
    }

    /// Clear the cache.
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }

    /// Get cache statistics.
    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }
}

impl Default for EvaluationCache {
    fn default() -> Self {
        Self::new(10_000, Duration::from_secs(3600))
    }
}

// Hash helpers

fn hash_contract(contract: &Contract) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    // Hash the contract name and version as a simple approach
    contract.name.hash(&mut hasher);
    contract.contract_version.hash(&mut hasher);
    hasher.finish()
}

fn hash_output(output: &Output) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    output.content.hash(&mut hasher);
    hasher.finish()
}

fn hash_context(context: Option<&[String]>) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    if let Some(ctx) = context {
        for item in ctx {
            item.hash(&mut hasher);
        }
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use steward_core::LensState;

    #[tokio::test]
    async fn test_cache_operations() {
        let cache = EvaluationCache::default();

        // Create a mock contract and output
        let contract_yaml = r#"
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test"
intent:
  purpose: "Test"
accountability:
  answerable_human: "test@example.com"
"#;
        let contract = Contract::from_yaml(contract_yaml).unwrap();
        let output = Output::text("Hello");

        let key = CacheKey::new(&contract, &output, None, LensType::DignityInclusion);

        // Cache miss
        assert!(cache.get(&key).await.is_none());

        // Insert
        let finding = LensFinding {
            lens: Some(LensType::DignityInclusion),
            question_asked: None,
            state: LensState::Pass,
            rules_evaluated: vec![],
            confidence: 0.9,
        };
        cache.insert(key.clone(), finding.clone()).await;

        // Cache hit
        let cached = cache.get(&key).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().confidence, 0.9);
    }
}
