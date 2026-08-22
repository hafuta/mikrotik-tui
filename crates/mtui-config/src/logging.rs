//! File + in-memory structured logging.
//!
//! The TUI owns the terminal, so tracing must never write to stdout/stderr.
//! [`init_file_logging`] installs a JSON file subscriber and a memory layer
//! that feeds the in-app console. Every formatted line and captured field
//! passes through [`crate::redact::redact`] first.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

use crate::error::{ConfigError, Result};
use crate::log_layer::MemoryLogLayer;
use crate::log_store::{DEFAULT_LOG_CAPACITY, LogStore};
use crate::paths;
use crate::redact::redact;

/// Log file name, relative to the state (or cache) directory.
pub const LOG_FILE_NAME: &str = "mikrotik-tui.log";

static LOG_STORE: OnceLock<Arc<LogStore>> = OnceLock::new();

/// Process-wide console log buffer. Created on first use even before
/// [`init_file_logging`], so tests can inject records without a subscriber.
#[must_use]
pub fn shared_log_store() -> Arc<LogStore> {
    LOG_STORE
        .get_or_init(|| Arc::new(LogStore::with_capacity(DEFAULT_LOG_CAPACITY)))
        .clone()
}

/// Wraps a writer and redacts password-like values from every chunk before
/// forwarding it. Tracing's formatter writes one formatted event per `write`
/// call, so this is applied per log line.
struct RedactingWriter<W> {
    inner: W,
}

impl<W: Write> Write for RedactingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        self.inner.write_all(redact(&text).as_bytes())?;
        // Report the whole input as written: we transform rather than
        // truncate, and callers (tracing's formatter) do not retry partial
        // writes.
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Initializes file JSON tracing plus the in-memory console layer at
/// `<state-or-cache-dir>/mikrotik-tui.log` and installs it as the global
/// default subscriber. Call once at startup; returns the resolved log path.
pub fn init_file_logging() -> Result<PathBuf> {
    let dir = paths::state_dir()?;
    std::fs::create_dir_all(&dir).map_err(|source| ConfigError::Write {
        path: dir.clone(),
        source,
    })?;
    let log_path = dir.join(LOG_FILE_NAME);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|source| ConfigError::Write {
            path: log_path.clone(),
            source,
        })?;

    let env_filter = EnvFilter::try_from_env("MIKROTIK_TUI_LOG").unwrap_or_else(|_| {
        EnvFilter::new("info,mtui_app=trace,mtui_routeros=info,mtui_config=info")
    });

    let store = shared_log_store();
    let memory = MemoryLogLayer::new(store).with_filter(env_filter.clone());

    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false)
        .with_writer(move || RedactingWriter {
            inner: file.try_clone().expect("clone log file handle"),
        })
        .with_filter(env_filter);

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(file_layer).with(memory),
    )
    .map_err(|_| ConfigError::LoggingAlreadyInitialized)?;

    Ok(log_path)
}
