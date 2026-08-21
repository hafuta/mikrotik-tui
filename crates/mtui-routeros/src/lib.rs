//! Read-only `RouterOS` REST client (v7) with TLS pin/custom-CA support.
//!
//! This crate speaks only `GET /rest/...`; it never mutates router state.
//! See [`Client`] for the entry point, [`tls`] helpers for certificate
//! pinning, and [`secret`] for masking sensitive field values before
//! display.

mod client;
mod error;
mod resource;
mod secret;
mod tls;

pub use client::{Client, ClientOptions, DEFAULT_REQUEST_TIMEOUT};
pub use error::{Error, ErrorKind, Result};
pub use resource::Resource;
pub use secret::{MASKED_VALUE, is_secret_key, mask_value};
pub use tls::{certificate_sha256, normalize_certificate_pin, probe_certificate};
