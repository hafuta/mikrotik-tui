//! TLS trust configuration: certificate pinning and custom CA support.
//!
//! `RouterOS` devices commonly present self-signed certificates. This module
//! lets callers either pin a known leaf certificate fingerprint (learned via
//! [`probe_certificate`]) or trust a custom CA bundle, without disabling
//! cryptographic signature verification of the TLS handshake itself.

use std::io::BufReader;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

/// Marker embedded in the TLS error surfaced when a certificate pin does not
/// match the presented leaf certificate. Used to classify transport errors
/// that bubble up through reqwest/hyper/rustls as [`crate::ErrorKind::Tls`].
pub(crate) const PIN_MISMATCH_MARKER: &str = "routeros: certificate pin mismatch";

const PROBE_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

fn crypto_provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

/// Returns the lowercase hex SHA-256 fingerprint of a DER-encoded
/// certificate.
#[must_use]
pub fn certificate_sha256(der: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(der);
    hex::encode(hasher.finalize())
}

/// Validates a SHA-256 certificate pin and normalizes it to lowercase hex
/// without separators, accepting `sha256:` and colon-delimited display
/// forms.
pub fn normalize_certificate_pin(pin: &str) -> Result<String> {
    let mut normalized = pin.trim().to_lowercase();
    if let Some(rest) = normalized.strip_prefix("sha256:") {
        normalized = rest.to_string();
    }
    let normalized: String = normalized.chars().filter(|c| *c != ':').collect();
    let decoded = hex::decode(&normalized).map_err(|_| {
        Error::tls(
            "normalize_certificate_pin",
            "invalid SHA-256 certificate pin",
        )
    })?;
    if decoded.len() != 32 {
        return Err(Error::tls(
            "normalize_certificate_pin",
            "invalid SHA-256 certificate pin",
        ));
    }
    Ok(hex::encode(decoded))
}

/// Builds a [`RootCertStore`] from a PEM-encoded certificate bundle.
pub(crate) fn root_store_from_pem(pem: &[u8]) -> Result<RootCertStore> {
    let mut store = RootCertStore::empty();
    let mut reader = BufReader::new(pem);
    let mut added = 0usize;
    for cert in rustls_pemfile::certs(&mut reader) {
        let cert = cert
            .map_err(|err| Error::tls("configure_tls", format!("invalid custom CA PEM: {err}")))?;
        store.add(cert).map_err(|err| {
            Error::tls(
                "configure_tls",
                format!("invalid custom CA certificate: {err}"),
            )
        })?;
        added += 1;
    }
    if added == 0 {
        return Err(Error::tls(
            "configure_tls",
            "custom CA contains no valid certificates",
        ));
    }
    Ok(store)
}

pub(crate) fn client_config_with_ca(pem: &[u8]) -> Result<ClientConfig> {
    let roots = root_store_from_pem(pem)?;
    ClientConfig::builder_with_provider(crypto_provider())
        .with_safe_default_protocol_versions()
        .map_err(|err| Error::tls("configure_tls", err.to_string()))
        .map(|builder| builder.with_root_certificates(roots).with_no_client_auth())
}

pub(crate) fn client_config_with_pin(pin: &str) -> Result<ClientConfig> {
    let verifier = Arc::new(PinnedCertVerifier {
        provider: crypto_provider(),
        pin: pin.to_string(),
    });
    ClientConfig::builder_with_provider(crypto_provider())
        .with_safe_default_protocol_versions()
        .map_err(|err| Error::tls("configure_tls", err.to_string()))
        .map(|builder| {
            builder
                .dangerous()
                .with_custom_certificate_verifier(verifier)
                .with_no_client_auth()
        })
}

/// Verifier that enforces an exact leaf certificate SHA-256 pin. Chain and
/// hostname validation are intentionally skipped (`RouterOS` self-signed
/// certificates rarely include a matching SAN), but the TLS handshake
/// signature is still cryptographically verified via the configured
/// provider.
#[derive(Debug)]
struct PinnedCertVerifier {
    provider: Arc<CryptoProvider>,
    pin: String,
}

impl ServerCertVerifier for PinnedCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        let actual = certificate_sha256(end_entity.as_ref());
        if actual != self.pin {
            return Err(rustls::Error::General(PIN_MISMATCH_MARKER.to_string()));
        }
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

/// Verifier that accepts any certificate (no chain/hostname/pin checks) but
/// still validates the handshake signature, recording the leaf fingerprint
/// as it is observed. Used only by [`probe_certificate`], which never sends
/// credentials or application data.
#[derive(Debug)]
struct RecordingCertVerifier {
    provider: Arc<CryptoProvider>,
    fingerprint: Mutex<Option<String>>,
}

impl ServerCertVerifier for RecordingCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        let mut slot = self
            .fingerprint
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *slot = Some(certificate_sha256(end_entity.as_ref()));
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

/// Connects to `base_url` and returns the leaf certificate's lowercase
/// SHA-256 fingerprint, without sending any credentials or HTTP request
/// data. The returned fingerprint is untrusted input: it must be reviewed
/// and approved (e.g. by a human) before being passed to
/// [`crate::ClientOptions`] as a certificate pin.
pub async fn probe_certificate(base_url: &str) -> Result<String> {
    let (host, port) = crate::client::probe_target(base_url)?;
    tokio::task::spawn_blocking(move || probe_certificate_blocking(&host, port))
        .await
        .map_err(|err| Error::transport("probe_certificate", err.to_string()))?
}

fn probe_certificate_blocking(host: &str, port: u16) -> Result<String> {
    let server_name = ServerName::try_from(host.to_string())
        .map_err(|err| Error::tls("probe_certificate", format!("invalid host name: {err}")))?;
    let provider = crypto_provider();
    let verifier = Arc::new(RecordingCertVerifier {
        provider: provider.clone(),
        fingerprint: Mutex::new(None),
    });
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|err| Error::tls("probe_certificate", err.to_string()))?
        .dangerous()
        .with_custom_certificate_verifier(verifier.clone())
        .with_no_client_auth();

    let mut connection = rustls::ClientConnection::new(Arc::new(config), server_name)
        .map_err(|err| Error::tls("probe_certificate", err.to_string()))?;

    let mut socket = TcpStream::connect((host, port))
        .map_err(|err| Error::transport("probe_certificate", err.to_string()))?;
    socket
        .set_read_timeout(Some(PROBE_CONNECT_TIMEOUT))
        .map_err(|err| Error::transport("probe_certificate", err.to_string()))?;
    socket
        .set_write_timeout(Some(PROBE_CONNECT_TIMEOUT))
        .map_err(|err| Error::transport("probe_certificate", err.to_string()))?;

    while connection.is_handshaking() {
        connection
            .complete_io(&mut socket)
            .map_err(|err| Error::tls("probe_certificate", err.to_string()))?;
    }

    let fingerprint = verifier
        .fingerprint
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    fingerprint.ok_or_else(|| Error::tls("probe_certificate", "TLS peer sent no certificate"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_plain_hex_pin() {
        let pin = "a".repeat(64);
        assert_eq!(normalize_certificate_pin(&pin).unwrap(), pin);
    }

    #[test]
    fn normalizes_sha256_prefixed_and_colon_delimited_pin() {
        let expected = "ab".repeat(32);
        let colonized = expected
            .as_bytes()
            .chunks(2)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect::<Vec<_>>()
            .join(":");
        let display = format!("sha256:{colonized}");
        assert_eq!(normalize_certificate_pin(&display).unwrap(), expected);
    }

    #[test]
    fn normalizes_mixed_case_and_whitespace() {
        let expected = "cd".repeat(32);
        let input = format!("  SHA256:{}  ", expected.to_uppercase());
        assert_eq!(normalize_certificate_pin(&input).unwrap(), expected);
    }

    #[test]
    fn rejects_invalid_pin_length_and_hex() {
        assert!(normalize_certificate_pin("not-hex").is_err());
        assert!(normalize_certificate_pin("ab").is_err());
        assert!(normalize_certificate_pin(&"a".repeat(63)).is_err());
    }
}
