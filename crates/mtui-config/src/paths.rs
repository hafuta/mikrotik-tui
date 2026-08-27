//! Platform config/state directory resolution.
//!
//! Profiles and credentials land in `~/.config/routeros-tui` on Linux,
//! `~/Library/Application Support/routeros-tui` on macOS, and
//! `%APPDATA%\routeros-tui` on Windows. `XDG_CONFIG_HOME` /
//! `XDG_STATE_HOME` are honored explicitly so the override behaves
//! identically across platforms.
//!
//! On Windows the `directories` crate appends an extra `config` segment;
//! this module uses `%APPDATA%\routeros-tui` instead so Rust and Go share
//! `profiles.json` and `credentials.json`.
//!
//! If the new directory does not exist yet, an existing `mikrotik-tui`
//! directory from the previous product name is used so profiles stay
//! reachable.

use std::path::PathBuf;

use directories::ProjectDirs;

use crate::error::{ConfigError, Result};

/// Application name used to namespace every on-disk path.
pub const APPLICATION: &str = "routeros-tui";

/// Previous product directory name. Used only when the current directory
/// has not been created yet.
pub const LEGACY_APPLICATION: &str = "mikrotik-tui";

/// Expands a leading `~/` or `~\` using `HOME` or `USERPROFILE`. Other paths
/// are returned unchanged so Windows drive paths and Unix absolute paths work.
#[must_use]
pub fn expand_user_path(raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    if trimmed == "~" {
        return home_dir().unwrap_or_else(|| PathBuf::from("~"));
    }
    let rest = trimmed
        .strip_prefix("~/")
        .or_else(|| trimmed.strip_prefix("~\\"));
    if let Some(rest) = rest
        && let Some(home) = home_dir()
    {
        return home.join(rest);
    }
    PathBuf::from(trimmed)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn project_dirs(app: &str) -> Result<ProjectDirs> {
    ProjectDirs::from("", "", app).ok_or(ConfigError::NoConfigDir)
}

fn env_dir(var: &str, app: &str) -> Option<PathBuf> {
    let value = std::env::var(var).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(trimmed).join(app))
}

fn prefer_existing(preferred: PathBuf, legacy: PathBuf) -> PathBuf {
    if preferred.exists() || !legacy.exists() {
        preferred
    } else {
        legacy
    }
}

fn config_dir_for(app: &str) -> Result<PathBuf> {
    if let Some(dir) = env_dir("XDG_CONFIG_HOME", app) {
        return Ok(dir);
    }
    #[cfg(windows)]
    {
        let appdata = std::env::var_os("APPDATA")
            .filter(|value| !value.is_empty())
            .ok_or(ConfigError::NoConfigDir)?;
        Ok(PathBuf::from(appdata).join(app))
    }
    #[cfg(not(windows))]
    {
        Ok(project_dirs(app)?.config_dir().to_path_buf())
    }
}

fn state_dir_for(app: &str) -> Result<PathBuf> {
    if let Some(dir) = env_dir("XDG_STATE_HOME", app) {
        return Ok(dir);
    }
    let dirs = project_dirs(app)?;
    Ok(dirs.state_dir().map_or_else(
        || dirs.cache_dir().to_path_buf(),
        std::path::Path::to_path_buf,
    ))
}

/// Directory holding `profiles.json` and `credentials.json`.
pub fn config_dir() -> Result<PathBuf> {
    Ok(prefer_existing(
        config_dir_for(APPLICATION)?,
        config_dir_for(LEGACY_APPLICATION)?,
    ))
}

/// Directory for the file-only application log (`routeros-tui.log`).
/// Prefers the XDG state directory and falls back to the cache directory on
/// platforms without a distinct state location (macOS, Windows).
pub fn state_dir() -> Result<PathBuf> {
    Ok(prefer_existing(
        state_dir_for(APPLICATION)?,
        state_dir_for(LEGACY_APPLICATION)?,
    ))
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn windows_config_dir_matches_go_appdata_layout() {
        if std::env::var_os("XDG_CONFIG_HOME").is_some() {
            return;
        }
        let dir = config_dir().expect("config dir");
        let name = dir.file_name().and_then(|name| name.to_str());
        assert!(
            name == Some(APPLICATION) || name == Some(LEGACY_APPLICATION),
            "{dir:?}"
        );
        assert_ne!(
            dir.file_name().and_then(|name| name.to_str()),
            Some("config")
        );
    }
}
