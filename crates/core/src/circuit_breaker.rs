//! Circuit breaker for outbound GNS3 API calls.
//!
//! Implements a simple three-state circuit breaker that prevents cascading
//! failures when the GNS3 server is unavailable.
//!
//! # States
//!
//! ```text
//! CLOSED ──(N consecutive failures)──► OPEN
//!   ▲                                     │
//!   └──(probe succeeds)── HALF_OPEN ◄────┘
//!                              │           (after recovery_timeout)
//!                              └─(probe fails)──► OPEN
//! ```
//!
//! # Examples
//!
//! ```rust
//! use gns3_mcp_core::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
//! use std::time::Duration;
//!
//! let cb = CircuitBreaker::new(CircuitBreakerConfig {
//!     failure_threshold: 5,
//!     recovery_timeout: Duration::from_secs(30),
//! });
//!
//! // Wrap any async operation:
//! // cb.call(|| async { my_api_call().await }).await
//! ```

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Configuration for a [`CircuitBreaker`].
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before the circuit opens.
    pub failure_threshold: u32,
    /// How long to wait in the OPEN state before attempting a probe.
    pub recovery_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
        }
    }
}

/// Internal state of the circuit breaker.
#[derive(Debug)]
enum State {
    /// Normal operation. Failures are counted.
    Closed { consecutive_failures: u32 },
    /// Circuit is open. All calls fail fast.
    Open { opened_at: Instant },
    /// One probe call is allowed to test recovery.
    HalfOpen,
}

/// Error type returned when the circuit is open.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CircuitBreakerError<E> {
    /// The underlying operation failed.
    #[error("operation failed: {0}")]
    Inner(E),
    /// The circuit is open — the GNS3 server is considered unavailable.
    #[error("circuit breaker open: GNS3 server unavailable, retry after back-off")]
    Open,
}

/// A three-state async circuit breaker.
///
/// Wrap outbound API calls with [`CircuitBreaker::call`] to prevent
/// hammering a down GNS3 server.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<State>>,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker with the given configuration.
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(State::Closed {
                consecutive_failures: 0,
            })),
        }
    }

    /// Creates a circuit breaker with default configuration.
    ///
    /// Threshold: 5 consecutive failures. Recovery timeout: 30 s.
    pub fn with_defaults() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Executes `operation`, tracking success/failure to drive state transitions.
    ///
    /// Returns [`CircuitBreakerError::Open`] immediately if the circuit is open
    /// and the recovery timeout has not yet elapsed.
    pub async fn call<F, Fut, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        // --- Pre-call: check state ---
        {
            let mut state = self.state.lock().await;
            match &*state {
                State::Open { opened_at } => {
                    if opened_at.elapsed() < self.config.recovery_timeout {
                        tracing::warn!(
                            recovery_in_secs = (self.config.recovery_timeout
                                .saturating_sub(opened_at.elapsed()))
                            .as_secs(),
                            "Circuit breaker OPEN — call rejected"
                        );
                        return Err(CircuitBreakerError::Open);
                    }
                    // Timeout elapsed: attempt a probe
                    tracing::info!("Circuit breaker → HALF-OPEN, sending probe");
                    *state = State::HalfOpen;
                }
                State::Closed { .. } | State::HalfOpen => {}
            }
        }

        // --- Execute operation ---
        let result = operation().await;

        // --- Post-call: update state ---
        {
            let mut state = self.state.lock().await;
            match &result {
                Ok(_) => {
                    if !matches!(*state, State::Closed { consecutive_failures: 0 }) {
                        tracing::info!("Circuit breaker → CLOSED (recovered)");
                    }
                    *state = State::Closed {
                        consecutive_failures: 0,
                    };
                }
                Err(_) => {
                    match &*state {
                        State::HalfOpen => {
                            tracing::warn!("Circuit breaker probe failed → OPEN");
                            *state = State::Open {
                                opened_at: Instant::now(),
                            };
                        }
                        State::Closed {
                            consecutive_failures,
                        } => {
                            let new_count = consecutive_failures + 1;
                            if new_count >= self.config.failure_threshold {
                                tracing::error!(
                                    threshold = self.config.failure_threshold,
                                    "Circuit breaker threshold reached → OPEN"
                                );
                                *state = State::Open {
                                    opened_at: Instant::now(),
                                };
                            } else {
                                tracing::warn!(
                                    failures = new_count,
                                    threshold = self.config.failure_threshold,
                                    "Circuit breaker failure count increased"
                                );
                                *state = State::Closed {
                                    consecutive_failures: new_count,
                                };
                            }
                        }
                        State::Open { .. } => {
                            // Shouldn't happen, but stay open
                        }
                    }
                }
            }
        }

        result.map_err(CircuitBreakerError::Inner)
    }

    /// Returns `true` if the circuit is currently open (calls will fail fast).
    pub async fn is_open(&self) -> bool {
        let state = self.state.lock().await;
        matches!(&*state, State::Open { opened_at } if opened_at.elapsed() < self.config.recovery_timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    async fn ok_op() -> Result<&'static str, &'static str> {
        Ok("ok")
    }

    async fn fail_op() -> Result<&'static str, &'static str> {
        Err("err")
    }

    #[tokio::test]
    async fn closed_on_success() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(60),
        });
        let result = cb.call(ok_op).await;
        assert!(result.is_ok());
        assert!(!cb.is_open().await);
    }

    #[tokio::test]
    async fn opens_after_threshold() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(60),
        });
        // 3 consecutive failures → circuit opens
        for _ in 0..3 {
            let _ = cb.call(fail_op).await;
        }
        assert!(cb.is_open().await);
    }

    #[tokio::test]
    async fn rejects_fast_when_open() {
        let counter = Arc::new(AtomicU32::new(0));
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_secs(60),
        });
        // Open the circuit
        let _ = cb.call(fail_op).await;

        // This call must be rejected without calling the operation
        let c = counter.clone();
        let result = cb
            .call(|| async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok::<_, &str>("should not reach here")
            })
            .await;

        assert!(matches!(result, Err(CircuitBreakerError::Open)));
        assert_eq!(counter.load(Ordering::SeqCst), 0, "operation must not be called when open");
    }

    #[tokio::test]
    async fn resets_on_recovery() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(10), // very short for tests
        });
        // Open
        for _ in 0..2 {
            let _ = cb.call(fail_op).await;
        }
        assert!(cb.is_open().await);

        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Probe succeeds → circuit closes
        let result = cb.call(ok_op).await;
        assert!(result.is_ok());
        assert!(!cb.is_open().await);
    }

    #[tokio::test]
    async fn stays_open_if_probe_fails() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(10),
        });
        let _ = cb.call(fail_op).await;
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Probe fails → stays open
        let _ = cb.call(fail_op).await;
        assert!(cb.is_open().await);
    }
}
