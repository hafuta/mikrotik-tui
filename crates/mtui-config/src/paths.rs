//! Platform config/state directory resolution.
//!
//! Profiles and credentials land in the same directories as the Go
//! implementation: `~/.config/mikrotik-tui` on Linux,
//! `~/Library/Application Support/mikrotik-tui` on macOS, and
//! `%APPDATA%\mikrotik-tui` on Windows. `XDG_CONFIG_HOME` /
//! `XDG_STATE_HOME` are honored explicitly so the override behaves
//! identically across platforms.
//!
//! On Windows the `directories` crate appends an extra `config` segment;
//! this module uses `%APPDATA%\mikrotik-tui` instead so Rust and Go share
//! `profiles.json` and `credentials.json`.

use std::path::PathBuf;

use directories::ProjectDirs;

use crate::error::{ConfigError, Result};

/// Application name used to namespace every on-disk path.
pub const APPLICATION: &str = "mikrotik-tui";

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", APPLICATION).ok_or(ConfigError::NoConfigDir)
}

fn env_dir(var: &str) -> Option<PathBuf> {
    let value = std::env::var(var).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(trimmed).join(APPLICATION))
}

/// Directory holding `profiles.json` and `credentials.json`.
pub fn config_dir() -> Result<PathBuf> {
    if let Some(dir) = env_dir("XDG_CONFIG_HOME") {
        return Ok(dir);
    }
    #[cfg(windows)]
    {
        let appdata = std::env::var_os("APPDATA")
            .filter(|value| !value.is_empty())
            .ok_or(ConfigError::NoConfigDir)?;
        Ok(PathBuf::from(appdata).join(APPLICATION))
    }
    #[cfg(not(windows))]
    {
        Ok(project_dirs()?.config_dir().to_path_buf())
    }
}

/// Directory for the file-only application log (`mikrotik-tui.log`).
/// Prefers the XDG state directory and falls back to the cache directory on
/// platforms without a distinct state location (macOS, Windows).
pub fn state_dir() -> Result<PathBuf> {
    if let Some(dir) = env_dir("XDG_STATE_HOME") {
        return Ok(dir);
    }
    let dirs = project_dirs()?;
    Ok(dirs.state_dir().map_or_else(
        || dirs.cache_dir().to_path_buf(),
        std::path::Path::to_path_buf,
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
        assert_eq!(
            dir.file_name().and_then(|name| name.to_str()),
            Some(APPLICATION)
        );
        assert_ne!(
            dir.file_name().and_then(|name| name.to_str()),
            Some("config")
        );
    }
}
