//! Connection target: host plus `api-ssl` or `api` port.

use crate::error::{Error, Result};

/// Default `RouterOS` plaintext `api` port.
pub const DEFAULT_API_PORT: u16 = 8728;

/// Default `RouterOS` `api-ssl` port.
pub const DEFAULT_API_SSL_PORT: u16 = 8729;

/// Default port for `use_tls` (`8729`) or plaintext `api` (`8728`).
#[must_use]
pub const fn default_api_port(use_tls: bool) -> u16 {
    if use_tls {
        DEFAULT_API_SSL_PORT
    } else {
        DEFAULT_API_PORT
    }
}

/// Parsed API endpoint.
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
/// Host-only values use the `api-ssl` default port.
pub fn parse_connection_target(raw: &str, operation: &'static str) -> Result<ConnectionTarget> {
    parse_connection_target_for(raw, operation, true)
}

/// Like [`parse_connection_target`], using `8729` when `use_tls` is true and
/// `8728` otherwise. An explicit port always wins. Legacy `https://` URLs
/// keep the `api-ssl` default.
pub fn parse_connection_target_for(
    raw: &str,
    operation: &'static str,
    use_tls: bool,
) -> Result<ConnectionTarget> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(Error::api(operation, "host is required"));
    }
    if trimmed.contains("://") {
        return parse_legacy_url(trimmed, operation);
    }
    parse_host_port(trimmed, operation, default_api_port(use_tls))
}

/// Normalize a typed or stored target to `host:8729` (legacy HTTPS URLs drop
/// the REST port and scheme).
#[must_use]
pub fn migrate_connection_target(raw: &str) -> String {
    migrate_connection_target_for(raw, true)
}

/// Normalize a target using the plaintext `api` default port when `use_tls`
/// is false.
#[must_use]
pub fn migrate_connection_target_for(raw: &str, use_tls: bool) -> String {
    parse_connection_target_for(raw, "migrate", use_tls)
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
    fn plaintext_api_defaults_to_8728() {
        let target = parse_connection_target_for("192.168.88.1", "test", false).unwrap();
        assert_eq!(target.port, 8728);
        assert_eq!(target.display(), "192.168.88.1:8728");
        assert_eq!(
            migrate_connection_target_for("router.lan", false),
            "router.lan:8728"
        );
        assert_eq!(default_api_port(true), 8729);
        assert_eq!(default_api_port(false), 8728);
    }

    #[test]
    fn explicit_port_wins_for_plaintext_and_tls() {
        let tls = parse_connection_target_for("router.lan:9991", "test", true).unwrap();
        let plain = parse_connection_target_for("router.lan:9991", "test", false).unwrap();
        assert_eq!(tls.port, 9991);
        assert_eq!(plain.port, 9991);
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
