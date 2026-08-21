//! Normalized client errors.
//!
//! Mirrors the Go `routeros.APIError` shape (kind, HTTP status, `RouterOS` API
//! code, message, operation) so downstream UI code can classify failures the
//! same way regardless of which language a given surface is implemented in.

use std::fmt;

/// Coarse classification of a [`Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// The calling future/task was canceled before completion.
    Canceled,
    /// The request exceeded its configured timeout.
    Timeout,
    /// A network/transport-level failure (DNS, connect, I/O, ...).
    Transport,
    /// A TLS handshake, certificate pin, or trust failure.
    Tls,
    /// The router rejected the credentials (HTTP 401/403).
    Auth,
    /// The requested record or endpoint does not exist (HTTP 404).
    NotFound,
    /// The router is rate-limiting requests (HTTP 429).
    RateLimited,
    /// The router reported an internal error (HTTP 5xx).
    Server,
    /// Any other non-2xx REST API response.
    Api,
    /// The response body could not be decoded as the expected JSON shape.
    Decode,
}

impl ErrorKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Canceled => "canceled",
            Self::Timeout => "timeout",
            Self::Transport => "transport",
            Self::Tls => "tls",
            Self::Auth => "auth",
            Self::NotFound => "not_found",
            Self::RateLimited => "rate_limited",
            Self::Server => "server",
            Self::Api => "api",
            Self::Decode => "decode",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Normalized `RouterOS` REST client error.
#[derive(Debug, thiserror::Error)]
#[error("{display}")]
pub struct Error {
    kind: ErrorKind,
    operation: &'static str,
    status: Option<u16>,
    api_code: Option<String>,
    message: String,
    display: String,
    #[source]
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl Error {
    #[must_use]
    pub fn new(kind: ErrorKind, operation: &'static str, message: impl Into<String>) -> Self {
        Self::build(kind, operation, None, None, message.into(), None)
    }

    #[must_use]
    pub fn with_status(
        kind: ErrorKind,
        operation: &'static str,
        status: u16,
        api_code: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::build(
            kind,
            operation,
            Some(status),
            api_code,
            message.into(),
            None,
        )
    }

    /// Attach an underlying cause without changing the public display text.
    #[must_use]
    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    fn build(
        kind: ErrorKind,
        operation: &'static str,
        status: Option<u16>,
        api_code: Option<String>,
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    ) -> Self {
        let mut parts = vec!["routeros".to_string()];
        if !operation.is_empty() {
            parts.push(operation.to_string());
        }
        parts.push(kind.to_string());
        if let Some(status) = status {
            parts.push(format!("HTTP {status}"));
        }
        if !message.is_empty() {
            parts.push(message.clone());
        }
        Self {
            kind,
            operation,
            status,
            api_code,
            message,
            display: parts.join(": "),
            source,
        }
    }

    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    #[must_use]
    pub fn operation(&self) -> &'static str {
        self.operation
    }

    #[must_use]
    pub fn status(&self) -> Option<u16> {
        self.status
    }

    #[must_use]
    pub fn api_code(&self) -> Option<&str> {
        self.api_code.as_deref()
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn transport(operation: &'static str, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Transport, operation, message)
    }

    pub(crate) fn tls(operation: &'static str, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Tls, operation, message)
    }

    pub(crate) fn decode(operation: &'static str, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Decode, operation, message)
    }

    pub(crate) fn api(operation: &'static str, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Api, operation, message)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
