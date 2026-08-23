//! Profiles, credentials, environment overrides, and redacted file logging.
//!
//! This crate persists everything the TUI needs to remember between runs
//! except transient UI state: named router [`Profile`]s (never containing
//! secrets), a replaceable [`credentials::CredentialStore`] that prefers the
//! OS keychain and falls back to a permission-hardened JSON file, [`env::EnvOverrides`]
//! for headless/CI use, and a tracing subscriber ([`logging::init_file_logging`])
//! that redacts password-like values before they hit disk or the in-app console.
//!
//! Behavior is a Rust-idiomatic port of `internal/config/config.go` and
//! `internal/credentials/credentials.go`, not a line-for-line translation:
//! atomic writes, document versioning, and Unix permission hardening are
//! preserved; error and API shapes are adapted to Rust conventions.

mod credentials;
mod env;
mod error;
mod fsutil;
mod log_layer;
mod log_store;
mod logging;
mod paths;
mod profile;
mod redact;

pub use credentials::{
    CREDENTIALS_FILE_NAME, Credential, CredentialStore, FileCredentialStore,
    PlatformCredentialStore,
};
pub use env::{ENV_PREFIX, EnvOverrides, read_ca_file};
pub use error::{ConfigError, Result};
pub use log_store::{DEFAULT_LOG_CAPACITY, LogLevel, LogRecord, LogStore};
pub use logging::{LOG_FILE_NAME, init_file_logging, shared_log_store};
pub use paths::expand_user_path;
pub use paths::{APPLICATION, config_dir, state_dir};
pub use profile::{
    HIDDEN_NAV_PREFERENCE_KEY, PROFILE_FILE_NAME, Preferences, Profile, ProfileStore,
    THEME_PREFERENCE_KEY,
};
pub use redact::redact;
