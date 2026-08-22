//! In-memory ring buffer of structured log records for the TUI console.

use std::collections::{BTreeMap, VecDeque};
use std::sync::Mutex;
use std::time::SystemTime;

use chrono::{DateTime, Local};

/// Default number of records retained for the in-app console.
pub const DEFAULT_LOG_CAPACITY: usize = 2_000;

/// Tracing severity captured for console display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    #[must_use]
    pub fn from_tracing(level: &tracing::Level) -> Self {
        match *level {
            tracing::Level::TRACE => Self::Trace,
            tracing::Level::DEBUG => Self::Debug,
            tracing::Level::INFO => Self::Info,
            tracing::Level::WARN => Self::Warn,
            tracing::Level::ERROR => Self::Error,
        }
    }
}

/// One structured log event after redaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRecord {
    pub id: u64,
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub fields: BTreeMap<String, String>,
}

impl LogRecord {
    #[must_use]
    pub fn timestamp_label(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
    }
}

struct Inner {
    next_id: u64,
    capacity: usize,
    records: VecDeque<LogRecord>,
}

/// Thread-safe ring buffer written by the tracing layer and read by the TUI.
pub struct LogStore {
    inner: Mutex<Inner>,
}

impl LogStore {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(Inner {
                next_id: 1,
                capacity: capacity.max(1),
                records: VecDeque::new(),
            }),
        }
    }

    pub fn push(
        &self,
        timestamp: DateTime<Local>,
        level: LogLevel,
        target: String,
        message: String,
        fields: BTreeMap<String, String>,
    ) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let id = inner.next_id;
        inner.next_id = inner.next_id.saturating_add(1);
        inner.records.push_back(LogRecord {
            id,
            timestamp,
            level,
            target,
            message,
            fields,
        });
        while inner.records.len() > inner.capacity {
            inner.records.pop_front();
        }
    }

    #[must_use]
    pub fn records_after(&self, last_id: u64) -> Vec<LogRecord> {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner
            .records
            .iter()
            .filter(|record| record.id > last_id)
            .cloned()
            .collect()
    }

    /// Current buffer, oldest first. This is the console's source of truth
    /// whether or not the pane is visible.
    #[must_use]
    pub fn snapshot(&self) -> Vec<LogRecord> {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.records.iter().cloned().collect()
    }
}

impl Default for LogStore {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_LOG_CAPACITY)
    }
}

/// Local timestamp used when tracing does not supply one.
#[must_use]
pub fn now_local() -> DateTime<Local> {
    DateTime::<Local>::from(SystemTime::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_oldest_when_over_capacity() {
        let store = LogStore::with_capacity(2);
        store.push(
            now_local(),
            LogLevel::Info,
            "t".into(),
            "a".into(),
            BTreeMap::new(),
        );
        store.push(
            now_local(),
            LogLevel::Info,
            "t".into(),
            "b".into(),
            BTreeMap::new(),
        );
        store.push(
            now_local(),
            LogLevel::Info,
            "t".into(),
            "c".into(),
            BTreeMap::new(),
        );
        let records = store.records_after(0);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].message, "b");
        assert_eq!(records[1].message, "c");
        assert!(records[1].id > records[0].id);
    }

    #[test]
    fn snapshot_returns_every_buffered_record() {
        let store = LogStore::with_capacity(8);
        store.push(
            now_local(),
            LogLevel::Info,
            "t".into(),
            "a".into(),
            BTreeMap::new(),
        );
        store.push(
            now_local(),
            LogLevel::Warn,
            "t".into(),
            "b".into(),
            BTreeMap::new(),
        );
        let snap = store.snapshot();
        assert_eq!(snap.len(), 2);
        assert_eq!(snap[0].message, "a");
        assert_eq!(snap[1].message, "b");
    }

    #[test]
    fn records_after_skips_seen_ids() {
        let store = LogStore::with_capacity(8);
        store.push(
            now_local(),
            LogLevel::Info,
            "t".into(),
            "a".into(),
            BTreeMap::new(),
        );
        let first = store.records_after(0);
        assert_eq!(first.len(), 1);
        store.push(
            now_local(),
            LogLevel::Warn,
            "t".into(),
            "b".into(),
            BTreeMap::new(),
        );
        let rest = store.records_after(first[0].id);
        assert_eq!(rest.len(), 1);
        assert_eq!(rest[0].message, "b");
    }
}
