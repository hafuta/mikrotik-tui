//! TLS trust configuration: pinning, custom CA, and OS trust store.
//!
//! `RouterOS` devices commonly present self-signed certificates. Callers can
//! pin a known leaf fingerprint (learned via [`probe_certificate`]), trust a
//! custom CA file, or use the operating system trust store (Windows
//! certificate store, macOS keychain, Linux CA bundle). Cryptographic
//! signature verification of the TLS handshake stays enabled.

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
/// that bubble up through rustls as [`crate::ErrorKind::Tls`].
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

/// Builds a [`RootCertStore`] from a PEM bundle or a single DER certificate.
/// Windows `.cer` exports are often DER; macOS and Linux CA files are usually
/// PEM.
pub(crate) fn root_store_from_bytes(bytes: &[u8]) -> Result<RootCertStore> {
    let mut store = RootCertStore::empty();
    let mut reader = BufReader::new(bytes);
    let mut added = 0usize;
    let mut pem_error = None;
    for cert in rustls_pemfile::certs(&mut reader) {
        match cert {
            Ok(cert) => {
                store.add(cert).map_err(|err| {
                    Error::tls(
                        "configure_tls",
                        format!("invalid custom CA certificate: {err}"),
                    )
                })?;
                added += 1;
            }
            Err(err) => pem_error = Some(err),
        }
    }
    if added == 0 && looks_like_der(bytes) {
        store
            .add(CertificateDer::from(bytes.to_vec()))
            .map_err(|err| {
                Error::tls(
                    "configure_tls",
                    format!("invalid custom CA certificate: {err}"),
                )
            })?;
        added = 1;
    }
    if added == 0 {
        return Err(Error::tls(
            "configure_tls",
            pem_error.map_or_else(
                || "custom CA contains no valid certificates".to_string(),
                |err| format!("invalid custom CA PEM: {err}"),
            ),
        ));
    }
    Ok(store)
}

fn looks_like_der(bytes: &[u8]) -> bool {
    bytes.first().is_some_and(|tag| *tag == 0x30)
}

pub(crate) fn client_config_with_ca(pem: &[u8]) -> Result<ClientConfig> {
    let roots = root_store_from_bytes(pem)?;
    config_with_roots(roots)
}

pub(crate) fn client_config_with_native_roots() -> Result<ClientConfig> {
    let loaded = rustls_native_certs::load_native_certs();
    if loaded.certs.is_empty() {
        let detail = loaded
            .errors
            .into_iter()
            .map(|err| err.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(Error::tls(
            "configure_tls",
            if detail.is_empty() {
                "OS trust store contains no certificates".to_string()
            } else {
                format!("failed to load OS trust store: {detail}")
            },
        ));
    }
    let mut roots = RootCertStore::empty();
    let mut added = 0usize;
    for cert in loaded.certs {
        if roots.add(cert).is_ok() {
            added += 1;
        }
    }
    if added == 0 {
        return Err(Error::tls(
            "configure_tls",
            "OS trust store contains no usable certificates",
        ));
    }
    config_with_roots(roots)
}

fn config_with_roots(roots: RootCertStore) -> Result<ClientConfig> {
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
pub async fn probe_certificate(target: &str) -> Result<String> {
    let parsed = crate::target::parse_connection_target(target, "probe_certificate")?;
    let host = parsed.host;
    let port = parsed.port;
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

    #[test]
    fn custom_ca_rejects_empty_and_garbage() {
        assert!(root_store_from_bytes(b"").is_err());
        assert!(root_store_from_bytes(b"not-a-certificate").is_err());
    }
}
