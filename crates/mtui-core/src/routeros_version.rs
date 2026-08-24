//! `RouterOS` version parse and the floor this TUI requires.

use std::fmt;

/// First `RouterOS` that exposes scriptable `/safe-mode`.
pub const MIN_ROUTEROS_VERSION: RouterOsVersion = RouterOsVersion {
    major: 7,
    minor: 18,
    patch: 0,
};

/// Numeric `RouterOS` release (`7.18.2` from `7.18.2 (stable)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RouterOsVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl fmt::Display for RouterOsVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Leading `major.minor.patch` from a `/system/resource` version string.
#[must_use]
pub fn parse_routeros_version(raw: &str) -> Option<RouterOsVersion> {
    let mut nums = raw
        .split(|ch: char| !ch.is_ascii_digit())
        .filter_map(|part| {
            if part.is_empty() {
                None
            } else {
                part.parse::<u16>().ok()
            }
        });
    let major = nums.next()?;
    Some(RouterOsVersion {
        major,
        minor: nums.next().unwrap_or(0),
        patch: nums.next().unwrap_or(0),
    })
}

/// `Ok` when `raw` is 7.18.0 or newer.
pub fn routeros_meets_minimum(raw: &str) -> Result<RouterOsVersion, String> {
    let Some(found) = parse_routeros_version(raw) else {
        return Err(format!(
            "Cannot read RouterOS version from {raw:?}. This app needs {MIN_ROUTEROS_VERSION} or newer.",
        ));
    };
    if found < MIN_ROUTEROS_VERSION {
        return Err(unsupported_routeros_copy(raw, found));
    }
    Ok(found)
}

#[must_use]
pub fn unsupported_routeros_copy(raw: &str, found: RouterOsVersion) -> String {
    format!("RouterOS {raw} ({found}) is too old. This app needs {MIN_ROUTEROS_VERSION} or newer.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_channel_suffix() {
        assert_eq!(
            parse_routeros_version("7.16.2 (long-term)"),
            Some(RouterOsVersion {
                major: 7,
                minor: 16,
                patch: 2
            })
        );
        assert_eq!(
            parse_routeros_version("7.18.2 (stable)"),
            Some(RouterOsVersion {
                major: 7,
                minor: 18,
                patch: 2
            })
        );
        assert_eq!(
            parse_routeros_version("7.18"),
            Some(RouterOsVersion {
                major: 7,
                minor: 18,
                patch: 0
            })
        );
    }

    #[test]
    fn floor_rejects_long_term_7_16() {
        let err = routeros_meets_minimum("7.16.2 (long-term)").expect_err("old");
        assert!(err.contains("7.18.0"));
        assert!(routeros_meets_minimum("7.18.2 (stable)").is_ok());
        assert!(routeros_meets_minimum("7.19.6").is_ok());
    }
}
