//! Error type shared by every module in this crate.

use std::path::PathBuf;

use thiserror::Error;

/// Errors produced while loading, saving, or resolving configuration,
/// credentials, or logging state.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("locate user config directory")]
    NoConfigDir,

    #[error("locate user state or cache directory")]
    NoStateDir,

    #[error("read {}: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("write {}: {source}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("decode {}: {source}", path.display())]
    Decode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("encode {what}: {source}")]
    Encode {
        what: &'static str,
        #[source]
        source: serde_json::Error,
    },

    #[error("unsupported {what} format version {version}")]
    UnsupportedVersion { what: &'static str, version: u32 },

    #[error("profile name is required")]
    ProfileNameRequired,

    #[error("profile {0:?} URL is required")]
    ProfileUrlRequired(String),

    #[error("profile {0:?} username is required")]
    ProfileUsernameRequired(String),

    #[error("duplicate profile {0:?}")]
    DuplicateProfile(String),

    #[error(
        "credential store {} has insecure permissions {mode:#o}; want 0600",
        path.display()
    )]
    InsecurePermissions { path: PathBuf, mode: u32 },

    #[error("credential profile name is required")]
    CredentialProfileNameRequired,

    #[error("credentials not found for profile {0:?}")]
    CredentialsNotFound(String),

    #[error("read secret file {}: {source}", path.display())]
    SecretFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("tracing subscriber already initialized")]
    LoggingAlreadyInitialized,
}

/// Convenience alias used throughout this crate.
pub type Result<T> = std::result::Result<T, ConfigError>;
