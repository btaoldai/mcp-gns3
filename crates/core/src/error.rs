//! Error types for the GNS3 MCP workspace.
//!
//! All public error enums are marked `#[non_exhaustive]` so that adding new
//! variants in future minor releases does not break downstream match expressions.
//!
//! # Error handling contract
//!
//! - `Gns3Error` is the canonical error type returned by [`crate::traits::Gns3Api`].
//! - `mcp-server` converts `Gns3Error` into actionable MCP error responses.
//! - Never expose raw HTTP status codes or internal GNS3 payloads in
//!   user-facing messages without sanitisation.

use thiserror::Error;

/// Errors that can occur when interacting with the GNS3 REST API.
///
/// This enum is `#[non_exhaustive]`: match with a wildcard arm (`_ => ...`)
/// to remain forward-compatible with new variants added in minor releases.
///
/// # Examples
///
/// ```rust
/// use gns3_mcp_core::error::Gns3Error;
///
/// fn handle(err: &Gns3Error) -> &'static str {
///     match err {
///         Gns3Error::Http { status, .. } if *status == 404 => "not found",
///         Gns3Error::Http { .. }    => "http error",
///         Gns3Error::Network(_)     => "network error",
///         Gns3Error::InvalidUuid(_) => "bad uuid",
///         Gns3Error::Server(_)      => "server error",
///         _ => "unknown error",  // required — enum is non_exhaustive
///     }
/// }
/// ```
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Gns3Error {
    /// An HTTP-level error returned by the GNS3 server.
    ///
    /// `status` is the HTTP status code (e.g. 404, 409, 500).
    /// `message` is the sanitised body or a default description.
    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },

    /// A network-level error (connection refused, timeout, DNS failure).
    ///
    /// The inner string is a human-readable description safe to surface to
    /// the LLM — it must not contain credentials or internal stack traces.
    #[error("Network error: {0}")]
    Network(String),

    /// A UUID supplied by the LLM failed validation.
    ///
    /// All UUIDs are validated by `gns3-client` before any HTTP call is made.
    #[error("Invalid UUID '{0}': must be a lowercase hyphenated UUIDv4")]
    InvalidUuid(String),

    /// An unexpected error returned by the GNS3 server.
    ///
    /// Used for 5xx responses not classified as transient (after retries
    /// are exhausted) and for malformed JSON responses.
    #[error("GNS3 server error: {0}")]
    Server(String),

    /// The circuit breaker is open — the GNS3 server is considered unavailable.
    ///
    /// The LLM should be instructed to wait before retrying.
    #[error("GNS3 server temporarily unavailable (circuit breaker open). Please retry in ~30 s.")]
    CircuitOpen,

    /// Failed to parse a response body.
    #[error("Deserialization failed: {0}")]
    Deserialize(String),

    /// Invalid or missing configuration.
    #[error("Invalid configuration: {0}")]
    Config(String),
}
