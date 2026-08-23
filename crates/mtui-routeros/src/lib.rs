//! `RouterOS` classic TCP API client (v7) with TLS pin/custom-CA support.
//!
//! Transport is `api-ssl` (default port 8729). Reads use `/path/print`. Writes
//! use `/path/set`, `/path/add`, `/path/remove`, and `/path/command`. Streaming
//! uses `.listen`, `monitor-traffic`, and unterminated tool commands on a
//! dedicated session. See [`Client`] for the entry point, [`tls`] helpers for
//! certificate pinning, and [`secret`] for masking sensitive field values
//! before display.

mod client;
mod codec;
mod error;
mod mutate;
mod resource;
mod secret;
mod sentence;
mod session;
mod target;
mod tls;

pub use client::{ApiStream, Client, ClientOptions, DEFAULT_REQUEST_TIMEOUT};
pub use error::{Error, ErrorKind, Result};
pub use mutate::{changed_fields, encode_fields, is_command_name};
pub use resource::Resource;
pub use secret::{MASKED_VALUE, is_secret_key, mask_value};
pub use sentence::{Sentence, merge_listen_record};
pub use target::{
    ConnectionTarget, DEFAULT_API_SSL_PORT, header_host, migrate_connection_target,
    parse_connection_target,
};
pub use tls::{certificate_sha256, normalize_certificate_pin, probe_certificate};
