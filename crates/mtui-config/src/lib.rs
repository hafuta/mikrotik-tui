//! Profiles, credentials, environment overrides, and redacted file logging.
//!
//! This crate persists everything the TUI needs to remember between runs
//! except transient UI state: named router [`Profile`]s (never containing
//! secrets), a permission-hardened [`credentials::FileCredentialStore`] for
//! passwords, [`env::EnvOverrides`] for headless/CI use, and a file-only
//! tracing subscriber ([`logging::init_file_logging`]) that redacts
//! password-like values before they ever hit disk.
//!
//! Behavior is a Rust-idiomatic port of `internal/config/config.go` and
//! `internal/credentials/credentials.go`, not a line-for-line translation:
//! atomic writes, document versioning, and Unix permission hardening are
//! preserved; error and API shapes are adapted to Rust conventions.

mod credentials;
mod env;
mod error;
mod fsutil;
mod logging;
mod paths;
mod profile;
mod redact;

pub use credentials::{CREDENTIALS_FILE_NAME, Credential, CredentialStore, FileCredentialStore};
pub use env::{ENV_PREFIX, EnvOverrides};
pub use error::{ConfigError, Result};
pub use logging::{LOG_FILE_NAME, init_file_logging};
pub use paths::{APPLICATION, config_dir, state_dir};
pub use profile::{PROFILE_FILE_NAME, Preferences, Profile, ProfileStore, THEME_PREFERENCE_KEY};
pub use redact::redact;
