//! Configuration for connecting to a GNS3 server.

use gns3_mcp_core::Gns3Error;
use gns3_mcp_core::circuit_breaker::CircuitBreakerConfig;
use std::time::Duration;

/// Default GNS3 server URL.
const DEFAULT_GNS3_URL: &str = "http://localhost:3080";

/// Default HTTP timeout in seconds.
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Environment variable name for overriding the HTTP timeout.
const TIMEOUT_ENV_VAR: &str = "GNS3_TIMEOUT_SECS";

/// Environment variable name for overriding the circuit breaker failure threshold.
const CB_FAILURE_THRESHOLD_ENV_VAR: &str = "GNS3_CB_FAILURE_THRESHOLD";

/// Environment variable name for overriding the circuit breaker recovery timeout (seconds).
const CB_RECOVERY_TIMEOUT_SECS_ENV_VAR: &str = "GNS3_CB_RECOVERY_TIMEOUT_SECS";

/// Configuration for the GNS3 HTTP client.
///
/// Built from environment variables via [`Gns3ClientConfig::from_env`].
/// Credentials are never logged or included in error messages.
#[derive(Clone)]
pub struct Gns3ClientConfig {
    /// Base URL of the GNS3 server (e.g. `http://localhost:3080`).
    pub base_url: String,
    /// Optional username for HTTP basic auth.
    pub username: Option<String>,
    /// Optional password for HTTP basic auth.
    pub password: Option<String>,
    /// HTTP request timeout in seconds.
    pub timeout_secs: u64,
}

/// Parse `GNS3_TIMEOUT_SECS` from the environment.
///
/// Returns [`DEFAULT_TIMEOUT_SECS`] if the variable is absent.
/// Returns an error if the value is present but is not a positive integer.
fn parse_timeout_secs() -> Result<u64, Gns3Error> {
    match std::env::var(TIMEOUT_ENV_VAR) {
        Err(_) => Ok(DEFAULT_TIMEOUT_SECS),
        Ok(raw) => {
            let value: u64 = raw.trim().parse().map_err(|_| {
                Gns3Error::Config(format!(
                    "{TIMEOUT_ENV_VAR} must be a positive integer (got: {raw:?})"
                ))
            })?;
            if value == 0 {
                return Err(Gns3Error::Config(format!(
                    "{TIMEOUT_ENV_VAR} must be greater than 0"
                )));
            }
            Ok(value)
        }
    }
}

/// Parse `GNS3_CB_FAILURE_THRESHOLD` from the environment.
///
/// Returns `CircuitBreakerConfig::default().failure_threshold` if the variable is absent.
/// Returns an error if the value is present but is not a positive integer.
fn parse_cb_failure_threshold() -> Result<u32, Gns3Error> {
    match std::env::var(CB_FAILURE_THRESHOLD_ENV_VAR) {
        Err(_) => Ok(CircuitBreakerConfig::default().failure_threshold),
        Ok(raw) => {
            let value: u32 = raw.trim().parse().map_err(|_| {
                Gns3Error::Config(format!(
                    "{CB_FAILURE_THRESHOLD_ENV_VAR} must be a positive integer (got: {raw:?})"
                ))
            })?;
            if value == 0 {
                return Err(Gns3Error::Config(format!(
                    "{CB_FAILURE_THRESHOLD_ENV_VAR} must be greater than 0"
                )));
            }
            Ok(value)
        }
    }
}

/// Parse `GNS3_CB_RECOVERY_TIMEOUT_SECS` from the environment.
///
/// Returns `CircuitBreakerConfig::default().recovery_timeout` if the variable is absent.
/// Returns an error if the value is present but is not a positive integer.
fn parse_cb_recovery_timeout() -> Result<Duration, Gns3Error> {
    match std::env::var(CB_RECOVERY_TIMEOUT_SECS_ENV_VAR) {
        Err(_) => Ok(CircuitBreakerConfig::default().recovery_timeout),
        Ok(raw) => {
            let secs: u64 = raw.trim().parse().map_err(|_| {
                Gns3Error::Config(format!(
                    "{CB_RECOVERY_TIMEOUT_SECS_ENV_VAR} must be a positive integer (got: {raw:?})"
                ))
            })?;
            if secs == 0 {
                return Err(Gns3Error::Config(format!(
                    "{CB_RECOVERY_TIMEOUT_SECS_ENV_VAR} must be greater than 0"
                )));
            }
            Ok(Duration::from_secs(secs))
        }
    }
}

impl Gns3ClientConfig {
    /// Build configuration from environment variables.
    ///
    /// Reads:
    /// - `GNS3_URL` (optional, defaults to `http://localhost:3080`)
    /// - `GNS3_USER` (optional)
    /// - `GNS3_PASSWORD` (optional)
    /// - `GNS3_TIMEOUT_SECS` (optional, defaults to [`DEFAULT_TIMEOUT_SECS`])
    /// - `GNS3_CB_FAILURE_THRESHOLD` (optional, defaults to 5)
    /// - `GNS3_CB_RECOVERY_TIMEOUT_SECS` (optional, defaults to 30)
    ///
    /// # Errors
    ///
    /// Returns [`Gns3Error::Config`] if:
    /// - `GNS3_URL` is present but empty or has an invalid scheme.
    /// - `GNS3_TIMEOUT_SECS` is present but is not a positive integer.
    /// - `GNS3_CB_FAILURE_THRESHOLD` is present but is not a positive integer.
    /// - `GNS3_CB_RECOVERY_TIMEOUT_SECS` is present but is not a positive integer.
    pub fn from_env() -> Result<Self, Gns3Error> {
        let base_url = std::env::var("GNS3_URL").unwrap_or_else(|_| DEFAULT_GNS3_URL.to_string());

        if base_url.is_empty() {
            return Err(Gns3Error::Config(
                "GNS3_URL is set but empty — provide a valid URL or unset it".to_string(),
            ));
        }

        // Strip trailing slash for consistent URL building
        let base_url = base_url.trim_end_matches('/').to_string();

        let username = std::env::var("GNS3_USER").ok().filter(|s| !s.is_empty());
        let password = std::env::var("GNS3_PASSWORD")
            .ok()
            .filter(|s| !s.is_empty());

        // Reject non-HTTP schemes (ftp://, javascript:, etc.)
        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            return Err(Gns3Error::Config(
                "GNS3_URL must start with http:// or https://".to_string(),
            ));
        }

        let timeout_secs = parse_timeout_secs()?;

        Ok(Self {
            base_url,
            username,
            password,
            timeout_secs,
        })
    }
}

impl Gns3ClientConfig {
    /// Build a circuit breaker configuration from environment variables.
    ///
    /// Reads:
    /// - `GNS3_CB_FAILURE_THRESHOLD` (optional, defaults to 5)
    /// - `GNS3_CB_RECOVERY_TIMEOUT_SECS` (optional, defaults to 30)
    ///
    /// # Errors
    ///
    /// Returns [`Gns3Error::Config`] if either variable is present but invalid.
    pub fn circuit_breaker_config_from_env() -> Result<CircuitBreakerConfig, Gns3Error> {
        let failure_threshold = parse_cb_failure_threshold()?;
        let recovery_timeout = parse_cb_recovery_timeout()?;
        Ok(CircuitBreakerConfig {
            failure_threshold,
            recovery_timeout,
        })
    }
}

impl std::fmt::Debug for Gns3ClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gns3ClientConfig")
            .field("base_url", &self.base_url)
            .field("username", &self.username.as_ref().map(|_| "<redacted>"))
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .field("timeout_secs", &self.timeout_secs)
            .finish()
    }
}
