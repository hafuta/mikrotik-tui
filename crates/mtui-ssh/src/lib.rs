//! Interactive SSH client that requests a `RouterOS` PTY.
//!
//! This crate does not parse CLI. It authenticates, requests a terminal, and
//! forwards bytes. Host-key checks are TOFU: a stored SHA-256 fingerprint must
//! match; a first connect accepts and returns the fingerprint to persist.

use std::borrow::Cow;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use russh::client::{self, AuthResult, Handle};
use russh::keys::{HashAlg, PublicKey};
use russh::{Channel, ChannelMsg, Preferred, cipher, client::Config, compression, kex, mac};
use sha2::{Digest, Sha256};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

/// Default inbound SSH port on `RouterOS`.
pub const DEFAULT_SSH_PORT: u16 = 22;

/// Bytes the UI wants to send, or a window-change.
#[derive(Debug, Clone)]
pub enum SshInput {
    Data(Vec<u8>),
    Resize { cols: u32, rows: u32 },
}

/// Options for an interactive session. Password is used only here.
pub struct SshConnectOptions {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    /// Lowercase hex SHA-256 of the server host key. Empty means first trust.
    pub expected_fingerprint: Option<String>,
    pub cols: u32,
    pub rows: u32,
}

/// A live PTY plus the host-key fingerprint that was accepted.
pub struct SshPty {
    pub fingerprint: String,
    /// `tcp/kex/auth/chan/pty` milliseconds for the dock status line.
    pub stages_ms: String,
    cols: u32,
    rows: u32,
    session: Handle<ClientHandler>,
    channel: Channel<client::Msg>,
}

#[derive(Debug, thiserror::Error)]
pub enum SshError {
    #[error("{0}")]
    Connect(String),
    #[error("wrong username or password for SSH")]
    Auth,
    #[error("SSH host key mismatch (stored {stored}, saw {seen})")]
    HostKeyMismatch { stored: String, seen: String },
    #[error("SSH host key was rejected")]
    HostKeyRejected,
}

pub type Result<T> = std::result::Result<T, SshError>;

/// SHA-256 fingerprint as lowercase hex (same encoding as API TLS pins).
#[must_use]
pub fn fingerprint_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

struct ClientHandler {
    expected: Option<String>,
    seen: Arc<Mutex<Option<String>>>,
}

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    #[allow(clippy::unused_async_trait_impl)]
    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> std::result::Result<bool, Self::Error> {
        let fp = host_key_fingerprint(server_public_key);
        if let Ok(mut seen) = self.seen.lock() {
            *seen = Some(fp.clone());
        }
        match self.expected.as_deref() {
            None | Some("") => Ok(true),
            Some(stored) if stored.eq_ignore_ascii_case(&fp) => Ok(true),
            Some(_) => Ok(false),
        }
    }
}

fn host_key_fingerprint(key: &PublicKey) -> String {
    let ssh = key.fingerprint(HashAlg::Sha256);
    fingerprint_hex(ssh.as_bytes())
}

impl SshPty {
    /// Dial, authenticate with a password, and request a PTY + shell.
    pub async fn connect(options: SshConnectOptions) -> Result<Self> {
        let started = std::time::Instant::now();
        let seen = Arc::new(Mutex::new(None));
        let handler = ClientHandler {
            expected: options.expected_fingerprint.clone(),
            seen: Arc::clone(&seen),
        };
        let config = Config {
            inactivity_timeout: None,
            nodelay: true,
            preferred: routeros_preferred(),
            ..Config::default()
        };
        let socket = tcp_connect(&options.host, options.port)
            .await
            .map_err(|err| SshError::Connect(err.to_string()))?;
        let mut session = client::connect_stream(Arc::new(config), socket, handler)
            .await
            .map_err(|err| {
                classify_connect(&err, &seen, options.expected_fingerprint.as_deref())
            })?;

        let auth = session
            .authenticate_password(&options.username, options.password)
            .await
            .map_err(|err| SshError::Connect(err.to_string()))?;
        if !auth_ok(&auth) {
            return Err(SshError::Auth);
        }

        let channel = session
            .channel_open_session()
            .await
            .map_err(|err| SshError::Connect(err.to_string()))?;
        channel
            .request_pty(
                false,
                "xterm",
                options.cols.max(1),
                options.rows.max(1),
                0,
                0,
                &[],
            )
            .await
            .map_err(|err| SshError::Connect(err.to_string()))?;
        channel
            .request_shell(false)
            .await
            .map_err(|err| SshError::Connect(err.to_string()))?;

        let fingerprint = seen
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
            .unwrap_or_default();

        Ok(Self {
            fingerprint,
            stages_ms: format!("{}ms", started.elapsed().as_millis()),
            cols: options.cols.max(1),
            rows: options.rows.max(1),
            session,
            channel,
        })
    }

    /// Read PTY output and apply input until `stdin` closes or the channel ends.
    pub async fn run(
        mut self,
        mut stdin: mpsc::UnboundedReceiver<SshInput>,
        out: mpsc::UnboundedSender<Vec<u8>>,
    ) {
        let mut cols = self.cols;
        let mut rows = self.rows;
        let mut queries = QueryScanner::new();
        loop {
            tokio::select! {
                msg = self.channel.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { ref data } | ChannelMsg::ExtendedData { ref data, .. }) => {
                            let replies = queries.push(data, rows, cols);
                            if out.send(data.to_vec()).is_err() {
                                break;
                            }
                            if !replies.is_empty() && self.channel.data(replies.as_slice()).await.is_err() {
                                break;
                            }
                        }
                        Some(ChannelMsg::Eof | ChannelMsg::Close) | None => break,
                        _ => {}
                    }
                }
                cmd = stdin.recv() => {
                    match cmd {
                        Some(SshInput::Data(bytes)) => {
                            if self.channel.data(&bytes[..]).await.is_err() {
                                break;
                            }
                        }
                        Some(SshInput::Resize { cols: next_cols, rows: next_rows }) => {
                            cols = next_cols.max(1);
                            rows = next_rows.max(1);
                            let _ = self.channel.window_change(cols, rows, 0, 0).await;
                        }
                        None => break,
                    }
                }
            }
        }
        let _ = self.channel.eof().await;
        drop(self.session);
    }
}

fn routeros_preferred() -> Preferred {
    Preferred {
        kex: Cow::Borrowed(&[
            kex::CURVE25519,
            kex::CURVE25519_PRE_RFC_8731,
            kex::ECDH_SHA2_NISTP256,
            kex::DH_G14_SHA256,
            kex::DH_G14_SHA1,
            kex::EXTENSION_SUPPORT_AS_CLIENT,
            kex::EXTENSION_OPENSSH_STRICT_KEX_AS_CLIENT,
        ]),
        cipher: Cow::Borrowed(&[
            cipher::AES_128_CTR,
            cipher::AES_256_CTR,
            cipher::AES_256_GCM,
            cipher::CHACHA20_POLY1305,
        ]),
        mac: Cow::Borrowed(&[
            mac::HMAC_SHA256,
            mac::HMAC_SHA1,
            mac::HMAC_SHA256_ETM,
            mac::HMAC_SHA1_ETM,
        ]),
        compression: Cow::Borrowed(&[compression::NONE]),
        ..Preferred::DEFAULT
    }
}

/// Dial `RouterOS` SSH without waiting on Windows DNS.
///
/// `getaddrinfo` on Windows often spends ~10s on a doomed AAAA lookup even
/// when `host` is already an IPv4 literal. OpenSSH skips that for IP
/// addresses; we do the same, then prefer IPv4 for names.
async fn tcp_connect(host: &str, port: u16) -> std::io::Result<TcpStream> {
    if let Some(addr) = literal_socket_addr(host, port) {
        let stream = TcpStream::connect(addr).await?;
        let _ = stream.set_nodelay(true);
        return Ok(stream);
    }
    let mut addrs: Vec<SocketAddr> = tokio::net::lookup_host((host, port)).await?.collect();
    sort_ipv4_first(&mut addrs);
    if addrs.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("no addresses for {host}:{port}"),
        ));
    }
    if addrs.len() == 1 {
        let stream = TcpStream::connect(addrs[0]).await?;
        let _ = stream.set_nodelay(true);
        return Ok(stream);
    }
    let (tx, mut rx) = mpsc::unbounded_channel();
    for (index, addr) in addrs.into_iter().enumerate() {
        let tx = tx.clone();
        tokio::spawn(async move {
            if index > 0 {
                tokio::time::sleep(Duration::from_millis(150)).await;
            }
            if let Ok(stream) = TcpStream::connect(addr).await {
                let _ = stream.set_nodelay(true);
                let _ = tx.send(stream);
            }
        });
    }
    drop(tx);
    rx.recv().await.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("SSH connect to {host}:{port} failed"),
        )
    })
}

fn sort_ipv4_first(addrs: &mut [SocketAddr]) {
    addrs.sort_by_key(|addr| u8::from(!addr.is_ipv4()));
}

fn literal_socket_addr(host: &str, port: u16) -> Option<SocketAddr> {
    let host = host.trim();
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Some(SocketAddr::new(ip, port));
    }
    let inner = host.strip_prefix('[')?.strip_suffix(']')?;
    let ip_part = inner.split_once('%').map_or(inner, |(ip, _)| ip);
    ip_part
        .parse::<IpAddr>()
        .ok()
        .map(|ip| SocketAddr::new(ip, port))
}

fn auth_ok(auth: &AuthResult) -> bool {
    auth.success()
}

/// `RouterOS` probes the PTY with `ESC Z` and `CSI 6n` and waits ~10s for a
/// reply before printing the banner. OpenSSH answers these; we must too.
struct QueryScanner {
    pending: Vec<u8>,
    row: u32,
    col: u32,
}

impl QueryScanner {
    fn new() -> Self {
        Self {
            pending: Vec::new(),
            row: 1,
            col: 1,
        }
    }

    fn push(&mut self, data: &[u8], rows: u32, cols: u32) -> Vec<u8> {
        self.pending.extend_from_slice(data);
        let (replies, consumed) =
            extract_query_replies(&self.pending, rows, cols, &mut self.row, &mut self.col);
        self.pending.drain(..consumed);
        replies
    }
}

const DA_VT100: &[u8] = b"\x1b[?1;2c";

fn extract_query_replies(
    buf: &[u8],
    rows: u32,
    cols: u32,
    row: &mut u32,
    col: &mut u32,
) -> (Vec<u8>, usize) {
    let mut replies = Vec::new();
    let mut i = 0;
    while i < buf.len() {
        match buf[i] {
            b'\r' => {
                *col = 1;
                i += 1;
            }
            b'\n' | 0x0b => {
                *row = (*row + 1).min(rows);
                i += 1;
            }
            0x1b => {
                if i + 1 >= buf.len() {
                    return (replies, i);
                }
                match buf[i + 1] {
                    b'Z' => {
                        replies.extend_from_slice(DA_VT100);
                        i += 2;
                    }
                    b'D' => {
                        *row = (*row + 1).min(rows);
                        i += 2;
                    }
                    b'M' => {
                        *row = (*row).saturating_sub(1).max(1);
                        i += 2;
                    }
                    b'[' => {
                        let mut j = i + 2;
                        while j < buf.len() && is_csi_param_byte(buf[j]) {
                            j += 1;
                        }
                        if j >= buf.len() {
                            return (replies, i);
                        }
                        let params = &buf[i + 2..j];
                        apply_csi(params, buf[j], rows, cols, row, col, &mut replies);
                        i = j + 1;
                    }
                    _ => i += 1,
                }
            }
            byte if byte.is_ascii_graphic() || byte == b' ' => {
                *col = (*col + 1).min(cols);
                i += 1;
            }
            _ => i += 1,
        }
    }
    (replies, buf.len())
}

fn apply_csi(
    params: &[u8],
    final_byte: u8,
    rows: u32,
    cols: u32,
    row: &mut u32,
    col: &mut u32,
    replies: &mut Vec<u8>,
) {
    let nums = csi_nums(params);
    let n = |idx: usize, default: u32| nums.get(idx).copied().filter(|v| *v > 0).unwrap_or(default);
    match final_byte {
        b'n' if params == b"6" => {
            replies.extend(format!("\x1b[{row};{col}R").into_bytes());
        }
        b'c' if params.is_empty() || params == b"0" => replies.extend_from_slice(DA_VT100),
        b'A' => *row = (*row).saturating_sub(n(0, 1)).max(1),
        b'B' => *row = (*row + n(0, 1)).min(rows),
        b'C' => *col = (*col + n(0, 1)).min(cols),
        b'D' => *col = (*col).saturating_sub(n(0, 1)).max(1),
        b'H' | b'f' => {
            *row = n(0, 1).min(rows);
            *col = n(1, 1).min(cols);
        }
        _ => {}
    }
}

fn csi_nums(params: &[u8]) -> Vec<u32> {
    params
        .split(|byte| *byte == b';')
        .map(|part| {
            let digits: String = part
                .iter()
                .copied()
                .filter(u8::is_ascii_digit)
                .map(char::from)
                .collect();
            digits.parse().unwrap_or(0)
        })
        .collect()
}

fn is_csi_param_byte(byte: u8) -> bool {
    matches!(byte, 0x20..=0x3f)
}

fn classify_connect(
    err: &russh::Error,
    seen: &Mutex<Option<String>>,
    expected: Option<&str>,
) -> SshError {
    let seen_fp = seen
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_default();
    if let Some(stored) = expected.filter(|value| !value.is_empty())
        && !seen_fp.is_empty()
        && !stored.eq_ignore_ascii_case(&seen_fp)
    {
        return SshError::HostKeyMismatch {
            stored: stored.to_string(),
            seen: seen_fp,
        };
    }
    if !seen_fp.is_empty() && expected.is_some_and(|value| !value.is_empty()) {
        return SshError::HostKeyRejected;
    }
    SshError::Connect(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        QueryScanner, extract_query_replies, fingerprint_hex, literal_socket_addr, sort_ipv4_first,
    };

    #[test]
    fn routeros_banner_probe_gets_da_and_cursor_report() {
        let chunk = b"\r\x1b[9999B\r\x1b[9999B\x1bZ  \x1b[6n";
        let mut row = 1;
        let mut col = 1;
        let (replies, consumed) = extract_query_replies(chunk, 24, 80, &mut row, &mut col);
        assert_eq!(consumed, chunk.len());
        assert_eq!(replies, b"\x1b[?1;2c\x1b[24;3R");
        assert_eq!((row, col), (24, 3));
    }

    #[test]
    fn width_probe_reports_last_column() {
        let mut scanner = QueryScanner::new();
        assert_eq!(
            scanner.push(b"\x1b[H\x1b[9999C\x1b[6n", 24, 80),
            b"\x1b[1;80R"
        );
    }

    #[test]
    fn split_csi_6n_is_answered_on_the_second_chunk() {
        let mut scanner = QueryScanner::new();
        assert!(scanner.push(b"\x1b[6", 24, 80).is_empty());
        assert_eq!(scanner.push(b"n", 24, 80), b"\x1b[1;1R");
    }

    #[test]
    fn fingerprint_hex_is_lowercase_sha256() {
        let hex = fingerprint_hex(b"abc");
        assert_eq!(hex.len(), 64);
        assert!(
            hex.chars()
                .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
        );
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn ipv4_addresses_are_tried_before_ipv6() {
        use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

        let v4 = SocketAddr::from((Ipv4Addr::LOCALHOST, 22));
        let v6 = SocketAddr::from((Ipv6Addr::LOCALHOST, 22));
        let mut addrs = vec![v6, v4];
        sort_ipv4_first(&mut addrs);
        assert!(addrs[0].is_ipv4());
        assert!(addrs[1].is_ipv6());
    }

    #[test]
    fn ipv4_literals_do_not_go_through_dns() {
        let addr = literal_socket_addr("192.168.88.1", 22).expect("ipv4");
        assert_eq!(addr.to_string(), "192.168.88.1:22");
        assert!(literal_socket_addr("router.lan", 22).is_none());
        let v6 = literal_socket_addr("[fe80::1%12]", 22).expect("ipv6");
        assert!(v6.is_ipv6());
        assert_eq!(v6.port(), 22);
    }
}
