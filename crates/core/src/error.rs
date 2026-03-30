//! Error types for GNS3 API interactions.

/// Errors originating from GNS3 API operations.
#[derive(Debug, thiserror::Error)]
pub enum Gns3Error {
    /// HTTP transport failure (network, timeout, TLS).
    #[error("HTTP request failed: {0}")]
    Http(String),

    /// GNS3 server returned a non-2xx status code.
    #[error("GNS3 API error ({status}): {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Error message from the server.
        message: String,
    },

    /// Failed to parse a response body.
    #[error("Deserialization failed: {0}")]
    Deserialize(String),

    /// Invalid or missing configuration.
    #[error("Invalid configuration: {0}")]
    Config(String),

    /// A UUID received from the caller was malformed.
    #[error("Invalid UUID '{value}': {reason}")]
    InvalidUuid {
        /// The raw value that failed parsing.
        value: String,
        /// Why it failed.
        reason: String,
    },
}
