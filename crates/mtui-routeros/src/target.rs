//! Connection target: host plus api-ssl port (default 8729).

use crate::error::{Error, Result};

/// Default `RouterOS` `api-ssl` port.
pub const DEFAULT_API_SSL_PORT: u16 = 8729;

/// Parsed api-ssl endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionTarget {
    pub host: String,
    pub port: u16,
}

impl ConnectionTarget {
    #[must_use]
    pub fn display(&self) -> String {
        if self.host.contains(':') && !self.host.starts_with('[') {
            format!("[{}]:{}", self.host, self.port)
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }
}

/// Parse `host`, `host:port`, `[ipv6]:port`, or a legacy `https://` REST URL.
pub fn parse_connection_target(raw: &str, operation: &'static str) -> Result<ConnectionTarget> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(Error::api(operation, "host is required"));
    }
    if trimmed.contains("://") {
        return parse_legacy_url(trimmed, operation);
    }
    parse_host_port(trimmed, operation, DEFAULT_API_SSL_PORT)
}

/// Normalize a typed or stored target to `host:8729` (legacy HTTPS URLs drop
/// the REST port and scheme).
#[must_use]
pub fn migrate_connection_target(raw: &str) -> String {
    parse_connection_target(raw, "migrate")
        .map_or_else(|_| raw.trim().to_string(), |target| target.display())
}

fn parse_legacy_url(raw: &str, operation: &'static str) -> Result<ConnectionTarget> {
    let rest = raw
        .split_once("://")
        .map_or(raw, |(_, rest)| rest)
        .trim_end_matches('/');
    if rest.contains('@') {
        return Err(Error::api(
            operation,
            "target must not contain userinfo credentials",
        ));
    }
    if rest.contains('?') {
        return Err(Error::api(operation, "target must not contain a query"));
    }
    if rest.contains('#') {
        return Err(Error::api(operation, "target must not contain a fragment"));
    }
    let hostport = rest.split('/').next().unwrap_or(rest);
    let host_only = if let Some(inner) = hostport.strip_prefix('[') {
        inner.split(']').next().unwrap_or(inner)
    } else {
        hostport.rsplit_once(':').map_or(hostport, |(host, port)| {
            if port.chars().all(|c| c.is_ascii_digit()) {
                host
            } else {
                hostport
            }
        })
    };
    parse_host_port(host_only, operation, DEFAULT_API_SSL_PORT)
}

fn parse_host_port(
    raw: &str,
    operation: &'static str,
    default_port: u16,
) -> Result<ConnectionTarget> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(Error::api(operation, "host is required"));
    }
    if let Some(inner) = raw.strip_prefix('[') {
        let Some((host, rest)) = inner.split_once(']') else {
            return Err(Error::api(operation, "invalid IPv6 host"));
        };
        if host.is_empty() {
            return Err(Error::api(operation, "host is required"));
        }
        let port = if rest.is_empty() {
            default_port
        } else {
            let Some(port) = rest.strip_prefix(':') else {
                return Err(Error::api(operation, "invalid host:port"));
            };
            parse_port(port, operation)?
        };
        return Ok(ConnectionTarget {
            host: host.to_string(),
            port,
        });
    }
    if raw.chars().filter(|c| *c == ':').count() == 1
        && let Some((host, port)) = raw.rsplit_once(':')
    {
        if host.is_empty() {
            return Err(Error::api(operation, "host is required"));
        }
        return Ok(ConnectionTarget {
            host: host.to_string(),
            port: parse_port(port, operation)?,
        });
    }
    if raw.contains(' ') {
        return Err(Error::api(operation, "host must not contain spaces"));
    }
    Ok(ConnectionTarget {
        host: raw.to_string(),
        port: default_port,
    })
}

fn parse_port(raw: &str, operation: &'static str) -> Result<u16> {
    raw.parse::<u16>()
        .map_err(|_| Error::api(operation, "invalid port"))
        .and_then(|port| {
            if port == 0 {
                Err(Error::api(operation, "invalid port"))
            } else {
                Ok(port)
            }
        })
}

/// Host shown in the session header (no port).
#[must_use]
pub fn header_host(raw: &str) -> String {
    parse_connection_target(raw, "header").map_or_else(
        |_| {
            let rest = raw
                .trim()
                .split_once("://")
                .map_or(raw.trim(), |(_, rest)| rest);
            let hostport = rest.split('/').next().unwrap_or(rest);
            if let Some(inner) = hostport.strip_prefix('[') {
                inner.split(']').next().unwrap_or(inner).to_string()
            } else {
                hostport.split(':').next().unwrap_or(hostport).to_string()
            }
        },
        |target| target.host,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_host_and_default_port() {
        let target = parse_connection_target("192.168.88.1", "test").unwrap();
        assert_eq!(target.host, "192.168.88.1");
        assert_eq!(target.port, 8729);
        assert_eq!(target.display(), "192.168.88.1:8729");
    }

    #[test]
    fn parses_explicit_port() {
        let target = parse_connection_target("router.lan:9991", "test").unwrap();
        assert_eq!(target.port, 9991);
    }

    #[test]
    fn parses_ipv6() {
        let target = parse_connection_target("[2001:db8::1]:8729", "test").unwrap();
        assert_eq!(target.host, "2001:db8::1");
        assert_eq!(target.display(), "[2001:db8::1]:8729");
    }

    #[test]
    fn migrates_https_rest_urls_to_api_ssl() {
        assert_eq!(
            migrate_connection_target("https://192.168.88.1/"),
            "192.168.88.1:8729"
        );
        assert_eq!(
            migrate_connection_target("https://192.168.88.1:8443"),
            "192.168.88.1:8729"
        );
        assert_eq!(
            migrate_connection_target("https://[2001:db8::1]:443"),
            "[2001:db8::1]:8729"
        );
    }

    #[test]
    fn rejects_userinfo_and_empty() {
        assert!(parse_connection_target("", "test").is_err());
        assert!(parse_connection_target("https://user:pass@router", "test").is_err());
    }
}
