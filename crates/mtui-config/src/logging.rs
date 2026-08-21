//! File-only structured logging.
//!
//! The TUI owns the terminal, so tracing must never write to stdout/stderr.
//! [`init_file_logging`] installs a global JSON subscriber that writes to
//! `<state-or-cache-dir>/mikrotik-tui.log`, passing every formatted line
//! through [`crate::redact::redact`] first.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use tracing_subscriber::EnvFilter;

use crate::error::{ConfigError, Result};
use crate::paths;
use crate::redact::redact;

/// Log file name, relative to the state (or cache) directory.
pub const LOG_FILE_NAME: &str = "mikrotik-tui.log";

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

/// Initializes file-only JSON tracing output at
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

    let env_filter =
        EnvFilter::try_from_env("MIKROTIK_TUI_LOG").unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_ansi(false)
        .with_env_filter(env_filter)
        .with_writer(move || RedactingWriter {
            inner: file.try_clone().expect("clone log file handle"),
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_| ConfigError::LoggingAlreadyInitialized)?;

    Ok(log_path)
}
