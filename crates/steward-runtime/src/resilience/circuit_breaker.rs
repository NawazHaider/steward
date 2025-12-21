//! Circuit breaker to prevent cascade failures.
//!
//! When LLM calls fail repeatedly, the circuit opens and
//! subsequent calls immediately fall back to deterministic evaluation.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use steward_core::LensType;

/// Circuit breaker configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failures before opening circuit
    pub failure_threshold: u32,

    /// Time before attempting recovery (in seconds)
    #[serde(with = "duration_secs")]
    pub recovery_timeout: Duration,

    /// Successes needed to close circuit
    pub success_threshold: u32,
}

mod duration_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 2,
        }
    }
}

/// State of a circuit.
#[derive(Debug, Clone)]
pub enum CircuitState {
    /// Normal operation
    Closed { failures: u32 },

    /// Circuit is open, all calls bypass
    Open { opened_at: Instant },

    /// Testing if circuit can close
    HalfOpen { successes: u32 },
}

/// Circuit breaker prevents cascade failures.
///
/// Each lens has its own circuit to allow independent recovery.
pub struct CircuitBreaker {
    states: RwLock<HashMap<LensType, CircuitState>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Check if circuit is open for a lens.
    ///
    /// Returns true if calls should bypass LLM and use fallback.
    pub fn is_open(&self, lens: LensType) -> bool {
        let states = self.states.read();
        match states.get(&lens) {
            Some(CircuitState::Open { opened_at }) => {
                // Check if recovery timeout has passed
                if opened_at.elapsed() >= self.config.recovery_timeout {
                    // Time to try half-open
                    drop(states);
                    self.transition_to_half_open(lens);
                    false
                } else {
                    true
                }
            }
            Some(CircuitState::HalfOpen { .. }) => false, // Allow test calls
            _ => false,
        }
    }

    /// Record a successful LLM call.
    pub fn record_success(&self, lens: LensType) {
        let mut states = self.states.write();
        match states.get(&lens).cloned() {
            Some(CircuitState::HalfOpen { successes }) => {
                if successes + 1 >= self.config.success_threshold {
                    // Close the circuit
                    states.insert(lens, CircuitState::Closed { failures: 0 });
                    tracing::info!(lens = ?lens, "Circuit closed after successful recovery");
                } else {
                    states.insert(lens, CircuitState::HalfOpen {
                        successes: successes + 1,
                    });
                }
            }
            Some(CircuitState::Closed { .. }) => {
                // Reset failures on success
                states.insert(lens, CircuitState::Closed { failures: 0 });
            }
            _ => {}
        }
    }

    /// Record a failed LLM call.
    pub fn record_failure(&self, lens: LensType) {
        let mut states = self.states.write();
        match states.get(&lens).cloned() {
            Some(CircuitState::Closed { failures }) => {
                if failures + 1 >= self.config.failure_threshold {
                    // Open the circuit
                    states.insert(lens, CircuitState::Open {
                        opened_at: Instant::now(),
                    });
                    tracing::warn!(
                        lens = ?lens,
                        failures = failures + 1,
                        "Circuit opened after repeated failures"
                    );
                } else {
                    states.insert(lens, CircuitState::Closed {
                        failures: failures + 1,
                    });
                }
            }
            Some(CircuitState::HalfOpen { .. }) => {
                // Failed during recovery, reopen
                states.insert(lens, CircuitState::Open {
                    opened_at: Instant::now(),
                });
                tracing::warn!(lens = ?lens, "Circuit reopened after failed recovery attempt");
            }
            None => {
                // First failure
                states.insert(lens, CircuitState::Closed { failures: 1 });
            }
            _ => {}
        }
    }

    /// Transition circuit to half-open state.
    fn transition_to_half_open(&self, lens: LensType) {
        let mut states = self.states.write();
        if matches!(states.get(&lens), Some(CircuitState::Open { .. })) {
            states.insert(lens, CircuitState::HalfOpen { successes: 0 });
            tracing::info!(lens = ?lens, "Circuit transitioning to half-open for recovery test");
        }
    }

    /// Get current state of a circuit.
    pub fn state(&self, lens: LensType) -> CircuitState {
        self.states
            .read()
            .get(&lens)
            .cloned()
            .unwrap_or(CircuitState::Closed { failures: 0 })
    }

    /// Reset all circuits to closed.
    pub fn reset(&self) {
        self.states.write().clear();
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_starts_closed() {
        let cb = CircuitBreaker::default();
        assert!(!cb.is_open(LensType::DignityInclusion));
    }

    #[test]
    fn test_circuit_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure(LensType::DignityInclusion);
        assert!(!cb.is_open(LensType::DignityInclusion));

        cb.record_failure(LensType::DignityInclusion);
        assert!(cb.is_open(LensType::DignityInclusion));
    }

    #[test]
    fn test_success_resets_failures() {
        let cb = CircuitBreaker::default();

        cb.record_failure(LensType::DignityInclusion);
        cb.record_failure(LensType::DignityInclusion);

        // Success should reset
        cb.record_success(LensType::DignityInclusion);

        // Need 3 more failures to open
        cb.record_failure(LensType::DignityInclusion);
        cb.record_failure(LensType::DignityInclusion);
        assert!(!cb.is_open(LensType::DignityInclusion));
    }

    #[test]
    fn test_lenses_are_independent() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        // Open Dignity circuit
        cb.record_failure(LensType::DignityInclusion);
        cb.record_failure(LensType::DignityInclusion);

        // Dignity is open, but Boundaries is closed
        assert!(cb.is_open(LensType::DignityInclusion));
        assert!(!cb.is_open(LensType::BoundariesSafety));
    }
}
