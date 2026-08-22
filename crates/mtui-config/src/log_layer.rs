//! Tracing layer that records redacted events into [`super::LogStore`].

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer};

use crate::log_store::{LogLevel, LogStore, now_local};
use crate::redact::redact;

/// Forwards tracing events into an in-memory [`LogStore`].
#[derive(Clone)]
pub struct MemoryLogLayer {
    store: Arc<LogStore>,
}

impl MemoryLogLayer {
    #[must_use]
    pub fn new(store: Arc<LogStore>) -> Self {
        Self { store }
    }
}

impl<S: Subscriber> Layer<S> for MemoryLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);
        let meta = event.metadata();
        let mut fields = BTreeMap::new();
        for (key, value) in visitor.fields {
            let combined = redact(&format!("{key}={value}"));
            let redacted_value = combined
                .split_once('=')
                .map(|(_, rest)| rest.to_string())
                .unwrap_or(combined);
            fields.insert(key, redacted_value);
        }
        self.store.push(
            now_local(),
            LogLevel::from_tracing(meta.level()),
            redact(meta.target()),
            redact(&visitor.message),
            fields,
        );
    }
}

#[derive(Default)]
struct FieldVisitor {
    message: String,
    fields: BTreeMap<String, String>,
}

impl FieldVisitor {
    fn record_value(&mut self, field: &Field, value: String) {
        if field.name() == "message" {
            self.message = value;
        } else if !field.name().starts_with("log.") {
            self.fields.insert(field.name().to_string(), value);
        }
    }
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.record_value(field, format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.record_value(field, value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record_value(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record_value(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.record_value(field, value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.record_value(field, value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::layer::SubscriberExt;

    #[test]
    fn captures_message_and_fields() {
        let store = Arc::new(LogStore::with_capacity(8));
        let subscriber = tracing_subscriber::registry().with(MemoryLogLayer::new(store.clone()));
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(
                endpoint = "/rest/interface",
                method = "GET",
                "outbound GET /rest/interface"
            );
        });
        let records = store.records_after(0);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].message, "outbound GET /rest/interface");
        assert_eq!(
            records[0].fields.get("endpoint").map(String::as_str),
            Some("/rest/interface")
        );
        assert_eq!(records[0].level, LogLevel::Info);
    }

    #[test]
    fn redacts_password_fields() {
        let store = Arc::new(LogStore::with_capacity(8));
        let subscriber = tracing_subscriber::registry().with(MemoryLogLayer::new(store.clone()));
        tracing::subscriber::with_default(subscriber, || {
            tracing::warn!(password = "hunter2", "login failed");
        });
        let records = store.records_after(0);
        assert_eq!(
            records[0].fields.get("password").map(String::as_str),
            Some("[REDACTED]")
        );
    }
}
