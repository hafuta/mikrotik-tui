//! Read-only `RouterOS` REST client.

use std::fmt::Write as _;
use std::time::Duration;

use serde::Deserialize;

use crate::error::{Error, ErrorKind, Result};
use crate::resource::Resource;
use crate::tls;

/// Default request timeout applied when [`ClientOptions::request_timeout`]
/// is left unset.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Upper bound on the number of response bytes read for a single request.
const MAX_RESPONSE_BYTES: usize = 8 << 20;

/// Configuration accepted by [`Client::new`].
#[derive(Clone, Debug)]
pub struct ClientOptions {
    /// Router REST base URL, e.g. `https://192.0.2.1`. Must be `https`, and
    /// must not contain userinfo, a query string, or a fragment.
    pub base_url: String,
    pub username: String,
    pub password: String,
    /// Per-request timeout. Defaults to [`DEFAULT_REQUEST_TIMEOUT`] when
    /// `None`.
    pub request_timeout: Option<Duration>,
    /// PEM-encoded custom CA bundle to trust instead of the system/webpki
    /// trust store. Ignored when [`certificate_pin`](Self::certificate_pin)
    /// is set.
    pub ca_pem: Option<Vec<u8>>,
    /// SHA-256 leaf certificate fingerprint to pin (see
    /// [`crate::normalize_certificate_pin`] for accepted formats). When set,
    /// only this exact leaf certificate is trusted and [`ca_pem`](Self::ca_pem)
    /// is ignored.
    pub certificate_pin: Option<String>,
}

impl ClientOptions {
    #[must_use]
    pub fn new(
        base_url: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            username: username.into(),
            password: password.into(),
            request_timeout: None,
            ca_pem: None,
            certificate_pin: None,
        }
    }

    #[must_use]
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    #[must_use]
    pub fn with_ca_pem(mut self, pem: impl Into<Vec<u8>>) -> Self {
        self.ca_pem = Some(pem.into());
        self
    }

    #[must_use]
    pub fn with_certificate_pin(mut self, pin: impl Into<String>) -> Self {
        self.certificate_pin = Some(pin.into());
        self
    }
}

/// Read-only `RouterOS` REST client.
///
/// Cloning a [`Client`] is cheap (it shares the underlying `reqwest::Client`
/// connection pool).
#[derive(Clone)]
pub struct Client {
    base_url: String,
    username: String,
    password: String,
    http: reqwest::Client,
}

impl Client {
    /// Builds a client from `options`, validating the base URL and TLS
    /// configuration eagerly.
    pub fn new(options: ClientOptions) -> Result<Self> {
        let parsed = parse_base_url(&options.base_url, "new_client")?;
        let base_url = parsed.as_str().trim_end_matches('/').to_string();

        let timeout = options.request_timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT);
        let mut builder = reqwest::Client::builder().timeout(timeout);

        if let Some(raw_pin) = &options.certificate_pin {
            let pin = tls::normalize_certificate_pin(raw_pin)?;
            builder = builder.use_preconfigured_tls(tls::client_config_with_pin(&pin)?);
        } else if let Some(pem) = &options.ca_pem {
            builder = builder.use_preconfigured_tls(tls::client_config_with_ca(pem)?);
        }

        let http = builder
            .build()
            .map_err(|err| Error::transport("new_client", err.to_string()))?;

        Ok(Self {
            base_url,
            username: options.username,
            password: options.password,
            http,
        })
    }

    /// Fetches a list-like `/rest/...` collection.
    pub async fn list(&self, endpoint: &str) -> Result<Vec<Resource>> {
        self.get_json(endpoint, "list").await
    }

    /// Fetches a single record by opaque `RouterOS` id.
    pub async fn get(&self, endpoint: &str, id: &str) -> Result<Resource> {
        let path = record_endpoint(endpoint, id);
        self.get_json(&path, "get").await
    }

    /// Fetches a singleton/system-scoped resource.
    pub async fn system(&self, endpoint: &str) -> Result<Resource> {
        self.get_json(endpoint, "system").await
    }

    async fn get_json<T>(&self, endpoint: &str, operation: &'static str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        validate_endpoint(endpoint, operation)?;
        let url = format!("{}{endpoint}", self.base_url);

        let response = self
            .http
            .get(&url)
            .header(reqwest::header::ACCEPT, "application/json")
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|err| self.classify_request_error(operation, &err))?;

        let status = response.status();
        if !status.is_success() {
            return Err(self
                .build_status_error(operation, status.as_u16(), response)
                .await);
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|err| self.classify_request_error(operation, &err))?;
        if bytes.len() > MAX_RESPONSE_BYTES {
            return Err(Error::decode(operation, "response body too large"));
        }

        serde_json::from_slice(&bytes)
            .map_err(|err| Error::decode(operation, format!("invalid API response: {err}")))
    }

    async fn build_status_error(
        &self,
        operation: &'static str,
        status: u16,
        response: reqwest::Response,
    ) -> Error {
        let body = response.bytes().await.unwrap_or_default();
        let wire: WireApiError = serde_json::from_slice(&body).unwrap_or_default();

        let mut message = wire
            .message
            .filter(|value| !value.trim().is_empty())
            .or(wire.detail.filter(|value| !value.trim().is_empty()))
            .unwrap_or_else(|| {
                reqwest::StatusCode::from_u16(status)
                    .ok()
                    .and_then(|code| code.canonical_reason())
                    .unwrap_or("request failed")
                    .to_string()
            });
        message = self.redact(&message);

        let api_code = wire.error.and_then(|value| match value {
            serde_json::Value::String(text) => Some(text),
            serde_json::Value::Number(number) => Some(number.to_string()),
            _ => None,
        });

        Error::with_status(
            kind_for_status(status),
            operation,
            status,
            api_code,
            message,
        )
    }

    fn classify_request_error(&self, operation: &'static str, err: &reqwest::Error) -> Error {
        let chain = self.redact(&error_chain_text(err)).to_lowercase();
        let kind = if err.is_timeout() {
            ErrorKind::Timeout
        } else if chain.contains(&tls::PIN_MISMATCH_MARKER.to_lowercase())
            || chain.contains("certificate")
            || chain.contains("invalid peer certificate")
            || chain.contains("unknownissuer")
            || (err.is_connect() && (chain.contains("tls") || chain.contains("ssl")))
        {
            ErrorKind::Tls
        } else if chain.contains("operation canceled") || chain.contains("operation cancelled") {
            ErrorKind::Canceled
        } else if err.is_decode() {
            ErrorKind::Decode
        } else {
            ErrorKind::Transport
        };
        Error::new(kind, operation, self.redact(&err.to_string()))
    }

    fn redact(&self, text: &str) -> String {
        let mut redacted = text.to_string();
        for secret in [self.password.as_str(), self.username.as_str()] {
            if !secret.is_empty() {
                redacted = redacted.replace(secret, "[redacted]");
            }
        }
        redacted
    }
}

#[derive(Debug, Default, Deserialize)]
struct WireApiError {
    #[serde(default)]
    error: Option<serde_json::Value>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    detail: Option<String>,
}

fn kind_for_status(status: u16) -> ErrorKind {
    match status {
        401 | 403 => ErrorKind::Auth,
        404 => ErrorKind::NotFound,
        429 => ErrorKind::RateLimited,
        500..=599 => ErrorKind::Server,
        _ => ErrorKind::Api,
    }
}

fn error_chain_text(err: &(dyn std::error::Error + 'static)) -> String {
    let mut text = err.to_string();
    let mut source = err.source();
    while let Some(inner) = source {
        text.push_str(": ");
        text.push_str(&inner.to_string());
        source = inner.source();
    }
    text
}

/// Validates that `raw` is an `https` URL with no userinfo, query, or
/// fragment, returning the parsed form.
pub(crate) fn parse_base_url(raw: &str, operation: &'static str) -> Result<reqwest::Url> {
    let trimmed = raw.trim().trim_end_matches('/');
    let parsed = reqwest::Url::parse(trimmed)
        .map_err(|err| Error::api(operation, format!("base URL must be a valid URL: {err}")))?;

    if parsed.scheme() != "https" {
        return Err(Error::api(operation, "base URL must use https"));
    }
    if parsed.host_str().is_none_or(str::is_empty) {
        return Err(Error::api(operation, "base URL must include a host"));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(Error::api(
            operation,
            "base URL must not contain userinfo credentials",
        ));
    }
    if parsed.query().is_some() {
        return Err(Error::api(operation, "base URL must not contain a query"));
    }
    if parsed.fragment().is_some() {
        return Err(Error::api(
            operation,
            "base URL must not contain a fragment",
        ));
    }
    Ok(parsed)
}

/// Resolves the `(host, port)` pair used by [`crate::probe_certificate`],
/// defaulting to port 443 when unspecified.
pub(crate) fn probe_target(base_url: &str) -> Result<(String, u16)> {
    let parsed = parse_base_url(base_url, "probe_certificate")?;
    let host = parsed
        .host_str()
        .ok_or_else(|| Error::api("probe_certificate", "base URL must include a host"))?
        .to_string();
    let port = parsed.port_or_known_default().unwrap_or(443);
    Ok((host, port))
}

fn validate_endpoint(endpoint: &str, operation: &'static str) -> Result<()> {
    if endpoint == "/rest" || endpoint.starts_with("/rest/") {
        Ok(())
    } else {
        Err(Error::api(operation, "invalid REST endpoint"))
    }
}

fn record_endpoint(endpoint: &str, id: &str) -> String {
    format!(
        "{}/{}",
        endpoint.trim_end_matches('/'),
        escape_path_segment(id)
    )
}

/// Percent-encodes `value` for use as a single opaque URL path segment,
/// escaping every byte outside the RFC 3986 "unreserved" set (including
/// `/`), so a `RouterOS` id can never be split across path segments.
fn escape_path_segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(*byte as char);
            }
            other => {
                let _ = write!(out, "%{other:02X}");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_https_base_url() {
        let parsed = parse_base_url("https://192.0.2.1/", "test").unwrap();
        assert_eq!(parsed.scheme(), "https");
    }

    #[test]
    fn rejects_non_https_base_url() {
        assert!(parse_base_url("http://192.0.2.1", "test").is_err());
    }

    #[test]
    fn rejects_userinfo_query_and_fragment() {
        assert!(parse_base_url("https://user:pass@192.0.2.1", "test").is_err());
        assert!(parse_base_url("https://192.0.2.1?x=1", "test").is_err());
        assert!(parse_base_url("https://192.0.2.1#frag", "test").is_err());
    }

    #[test]
    fn validates_rest_endpoint_prefix() {
        assert!(validate_endpoint("/rest/interface", "test").is_ok());
        assert!(validate_endpoint("/rest", "test").is_ok());
        assert!(validate_endpoint("/interface", "test").is_err());
        assert!(validate_endpoint("rest/interface", "test").is_err());
    }

    #[test]
    fn escapes_reserved_characters_in_ids() {
        assert_eq!(escape_path_segment("*1/unsafe id"), "%2A1%2Funsafe%20id");
        assert_eq!(
            record_endpoint("/rest/interface", "*1/unsafe id"),
            "/rest/interface/%2A1%2Funsafe%20id"
        );
    }

    #[test]
    fn classifies_status_codes() {
        assert_eq!(kind_for_status(401), ErrorKind::Auth);
        assert_eq!(kind_for_status(403), ErrorKind::Auth);
        assert_eq!(kind_for_status(404), ErrorKind::NotFound);
        assert_eq!(kind_for_status(429), ErrorKind::RateLimited);
        assert_eq!(kind_for_status(500), ErrorKind::Server);
        assert_eq!(kind_for_status(400), ErrorKind::Api);
    }
}
