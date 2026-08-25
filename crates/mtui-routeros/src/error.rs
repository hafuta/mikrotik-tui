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
    /// The router rejected the credentials.
    Auth,
    /// The requested record or endpoint does not exist.
    NotFound,
    /// The router is rate-limiting requests.
    RateLimited,
    /// The router reported an internal error.
    Server,
    /// A `RouterOS` `!trap` or other API-level failure.
    Api,
    /// The account lacks the group policy for this command.
    Permission,
    /// The reply could not be decoded.
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
            Self::Permission => "permission",
            Self::Decode => "decode",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Normalized `RouterOS` API client error.
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
        if let Some(status) = status.filter(|code| *code != 0) {
            parts.push(format!("status {status}"));
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

    /// True when the TCP session is gone (reset, closed, broken pipe).
    /// A single request timeout is not treated as a drop.
    #[must_use]
    pub fn is_link_loss(&self) -> bool {
        if self.kind == ErrorKind::Transport {
            return true;
        }
        let lower = self.message.to_ascii_lowercase();
        lower.contains("connection closed")
            || lower.contains("connection reset")
            || lower.contains("broken pipe")
            || lower.contains("not connected")
            || lower.contains("unexpected eof")
    }

    pub(crate) fn trap(
        kind: ErrorKind,
        operation: &'static str,
        api_code: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::build(kind, operation, None, api_code, message.into(), None)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_and_closed_are_link_loss() {
        let closed = Error::new(ErrorKind::Api, "list", "connection closed");
        assert!(closed.is_link_loss());
        let timeout = Error::new(ErrorKind::Timeout, "list", "request timed out");
        assert!(!timeout.is_link_loss());
        let transport = Error::new(ErrorKind::Transport, "list", "broken pipe");
        assert!(transport.is_link_loss());
        let canceled = Error::new(ErrorKind::Canceled, "list", "romon print canceled");
        assert!(!canceled.is_link_loss());
        assert_eq!(canceled.kind(), ErrorKind::Canceled);
        let trap = Error::new(ErrorKind::Api, "set", "no such command prefix (romon)");
        assert!(!trap.is_link_loss());
        assert!(trap.to_string().contains("romon"));
    }

    #[test]
    fn lte_apn_errors_keep_resource_operation_and_cause() {
        let cases = [
            (
                ErrorKind::Api,
                "list",
                "no such item",
                "routeros: list: api: no such item",
            ),
            (
                ErrorKind::Timeout,
                "list",
                "request timed out",
                "routeros: list: timeout: request timed out",
            ),
            (
                ErrorKind::Canceled,
                "list",
                "canceled",
                "routeros: list: canceled: canceled",
            ),
            (
                ErrorKind::Decode,
                "list",
                "field \"apn\" is not a string",
                "routeros: list: decode: field \"apn\" is not a string",
            ),
        ];
        for (kind, operation, message, display) in cases {
            let err = Error::new(kind, operation, message)
                .with_source(std::io::Error::other("fake transport"));
            assert_eq!(err.kind(), kind);
            assert_eq!(err.operation(), operation);
            assert_eq!(err.to_string(), display);
            assert!(std::error::Error::source(&err).is_some());
        }
    }
}
