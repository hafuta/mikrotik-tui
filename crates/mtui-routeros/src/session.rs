//! Tagged `RouterOS` API session over one TLS (or test) stream.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::{Mutex, mpsc};
use tokio::time::timeout;

use crate::codec::{SentenceDecoder, encode_sentence};
use crate::error::{Error, ErrorKind, Result};
use crate::sentence::Sentence;
use crate::tls::PIN_MISMATCH_MARKER;

const MAX_BUFFER: usize = 8 << 20;
const READ_CHUNK: usize = 8 << 10;

/// Live tagged multiplexer: one writer, one reader task, replies by `.tag`.
#[derive(Clone)]
pub struct Session {
    inner: Arc<SessionInner>,
}

struct SessionInner {
    writer: Mutex<WriteHalf<BoxStream>>,
    next_tag: AtomicU64,
    pending: Mutex<HashMap<String, mpsc::UnboundedSender<Sentence>>>,
    timeout: Duration,
    username: String,
    password: String,
}

/// Type-erased TLS or test stream.
type BoxStream = PinBox;

struct PinBox {
    inner: Box<dyn StreamIo>,
}

trait StreamIo: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T> StreamIo for T where T: AsyncRead + AsyncWrite + Send + Unpin {}

impl AsyncRead for PinBox {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut *self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for PinBox {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut *self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut *self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut *self.inner).poll_shutdown(cx)
    }
}

impl Session {
    pub async fn from_stream<S>(
        stream: S,
        username: String,
        password: String,
        request_timeout: Duration,
    ) -> Result<Self>
    where
        S: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let boxed = PinBox {
            inner: Box::new(stream),
        };
        let (reader, writer) = tokio::io::split(boxed);
        let inner = Arc::new(SessionInner {
            writer: Mutex::new(writer),
            next_tag: AtomicU64::new(1),
            pending: Mutex::new(HashMap::new()),
            timeout: request_timeout,
            username,
            password,
        });
        tokio::spawn(read_loop(reader, Arc::clone(&inner)));
        let session = Self { inner };
        session.login().await?;
        Ok(session)
    }

    async fn login(&self) -> Result<()> {
        let words = vec![
            "/login".to_string(),
            format!("=name={}", self.inner.username),
            format!("=password={}", self.inner.password),
        ];
        tracing::info!(command = "/login", "outbound /login");
        let replies = self.request_words("login", words).await?;
        if let Some(trap) = replies.iter().find(|sentence| sentence.is_trap()) {
            return Err(trap.trap_error("login"));
        }
        if replies.iter().any(Sentence::is_fatal) {
            return Err(Error::new(ErrorKind::Auth, "login", "login failed"));
        }
        Ok(())
    }

    fn next_tag(&self) -> String {
        self.inner
            .next_tag
            .fetch_add(1, Ordering::Relaxed)
            .to_string()
    }

    fn redact(&self, text: &str) -> String {
        let mut redacted = text.to_string();
        for secret in [self.inner.password.as_str(), self.inner.username.as_str()] {
            if !secret.is_empty() {
                redacted = redacted.replace(secret, "[redacted]");
            }
        }
        redacted
    }

    /// Send a tagged sentence and collect replies until `!done` / `!trap` / `!fatal`.
    pub async fn request(
        &self,
        operation: &'static str,
        words: Vec<String>,
    ) -> Result<Vec<Sentence>> {
        self.request_words(operation, words).await
    }

    async fn request_words(
        &self,
        operation: &'static str,
        mut words: Vec<String>,
    ) -> Result<Vec<Sentence>> {
        let tag = self.next_tag();
        words.push(format!(".tag={tag}"));
        log_outbound(operation, &words);
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.inner.pending.lock().await.insert(tag.clone(), tx);
        if let Err(err) = self.write_sentence(&words).await {
            self.inner.pending.lock().await.remove(&tag);
            return Err(err);
        }
        let collected = timeout(self.inner.timeout, async {
            let mut replies = Vec::new();
            while let Some(sentence) = rx.recv().await {
                let terminal = sentence.is_done() || sentence.is_trap() || sentence.is_fatal();
                replies.push(sentence);
                if terminal {
                    break;
                }
            }
            replies
        })
        .await;
        self.inner.pending.lock().await.remove(&tag);
        match collected {
            Ok(replies) if replies.is_empty() => Err(Error::new(
                ErrorKind::Transport,
                operation,
                "connection closed",
            )),
            Ok(replies) => {
                if let Some(fatal) = replies.iter().find(|sentence| sentence.is_fatal()) {
                    return Err(Error::new(
                        ErrorKind::Server,
                        operation,
                        fatal.attr("message").unwrap_or("fatal API error"),
                    ));
                }
                if let Some(trap) = replies.iter().find(|sentence| sentence.is_trap()) {
                    return Err(trap.trap_error(operation));
                }
                Ok(replies)
            }
            Err(_) => {
                let _ = self.write_cancel(tag.as_str()).await;
                Err(Error::new(
                    ErrorKind::Timeout,
                    operation,
                    "request timed out",
                ))
            }
        }
    }

    /// Start an unterminated command (listen / monitor / follow / ping).
    pub async fn stream(
        &self,
        operation: &'static str,
        mut words: Vec<String>,
    ) -> Result<(String, mpsc::UnboundedReceiver<Sentence>)> {
        let tag = self.next_tag();
        words.push(format!(".tag={tag}"));
        log_outbound(operation, &words);
        let (tx, rx) = mpsc::unbounded_channel();
        self.inner.pending.lock().await.insert(tag.clone(), tx);
        if let Err(err) = self.write_sentence(&words).await {
            self.inner.pending.lock().await.remove(&tag);
            return Err(err);
        }
        Ok((tag, rx))
    }

    pub async fn cancel_tag(&self, tag: &str) -> Result<()> {
        self.write_cancel(tag).await
    }

    async fn write_cancel(&self, tag: &str) -> Result<()> {
        let words = vec!["/cancel".to_string(), format!("=.tag={tag}")];
        log_outbound("cancel", &words);
        self.write_sentence(&words).await
    }

    pub async fn finish_stream(&self, tag: &str) {
        self.inner.pending.lock().await.remove(tag);
        let _ = self.cancel_tag(tag).await;
        self.inner.pending.lock().await.remove(tag);
    }

    async fn write_sentence(&self, words: &[String]) -> Result<()> {
        let bytes = encode_sentence(words);
        let mut writer = self.inner.writer.lock().await;
        writer
            .write_all(&bytes)
            .await
            .map_err(|err| classify_io("write", &self.redact(&err.to_string())))?;
        writer
            .flush()
            .await
            .map_err(|err| classify_io("write", &self.redact(&err.to_string())))
    }
}

fn log_outbound(operation: &str, words: &[String]) {
    let command = words.first().map_or("?", String::as_str);
    let sentence = Sentence::new(words.to_vec());
    tracing::info!(operation, command, "outbound {command}");
    tracing::debug!(
        operation,
        sentence = sentence.log_line().as_str(),
        "api sentence"
    );
}

fn classify_io(operation: &'static str, message: &str) -> Error {
    let lower = message.to_lowercase();
    if lower.contains(&PIN_MISMATCH_MARKER.to_lowercase())
        || lower.contains("certificate")
        || lower.contains("invalid peer certificate")
        || lower.contains("unknownissuer")
        || lower.contains("tls")
        || lower.contains("ssl")
    {
        Error::tls(operation, message)
    } else if lower.contains("timed out") || lower.contains("timeout") {
        Error::new(ErrorKind::Timeout, operation, message)
    } else if lower.contains("canceled") || lower.contains("cancelled") {
        Error::new(ErrorKind::Canceled, operation, message)
    } else {
        Error::transport(operation, message)
    }
}

async fn read_loop(mut reader: ReadHalf<PinBox>, inner: Arc<SessionInner>) {
    let mut decoder = SentenceDecoder::new();
    let mut buf = vec![0_u8; READ_CHUNK];
    loop {
        let read = match reader.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        decoder.push(&buf[..read]);
        if decoder.buffered_bytes() > MAX_BUFFER {
            tracing::error!("API read buffer exceeded");
            break;
        }
        loop {
            match decoder.take_sentence() {
                Ok(Some(words)) => {
                    let sentence = Sentence::new(words);
                    if sentence.is_trap() || sentence.is_fatal() {
                        tracing::error!(sentence = sentence.log_line().as_str(), "api error reply");
                    } else if sentence.is_done() {
                        tracing::debug!(
                            tag = sentence.tag().unwrap_or(""),
                            sentence = sentence.log_line().as_str(),
                            "api done"
                        );
                    }
                    if let Some(tag) = sentence.tag() {
                        let pending = inner.pending.lock().await;
                        if let Some(tx) = pending.get(tag) {
                            let _ = tx.send(sentence);
                        }
                    } else if sentence.is_fatal() {
                        let pending = inner.pending.lock().await;
                        for tx in pending.values() {
                            let _ = tx.send(sentence.clone());
                        }
                    }
                }
                Ok(None) => break,
                Err(err) => {
                    tracing::error!(error = %err, "api decode failed");
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode_sentence;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn tagged_print_collects_re_then_done() {
        let (client, mut server) = tokio::io::duplex(64 * 1024);
        let server_task = tokio::spawn(async move {
            let mut buf = vec![0_u8; 4096];
            let _ = server.read(&mut buf).await;
            let login_ok = encode_sentence(&["!done", ".tag=1"]);
            server.write_all(&login_ok).await.unwrap();
            let _ = server.read(&mut buf).await;
            let body = [
                encode_sentence(&["!re", "=.id=*1", "=name=ether1", ".tag=2"]),
                encode_sentence(&["!done", ".tag=2"]),
            ]
            .concat();
            server.write_all(&body).await.unwrap();
        });

        let session = Session::from_stream(
            client,
            "admin".into(),
            String::new(),
            Duration::from_secs(2),
        )
        .await
        .expect("login");
        let replies = session
            .request("list", vec!["/interface/print".into()])
            .await
            .expect("print");
        assert!(replies.iter().any(Sentence::is_re));
        assert!(replies.iter().any(Sentence::is_done));
        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn login_trap_is_returned() {
        let (client, mut server) = tokio::io::duplex(64 * 1024);
        tokio::spawn(async move {
            let mut buf = vec![0_u8; 4096];
            let _ = server.read(&mut buf).await;
            let trap = encode_sentence(&["!trap", "=message=cannot log in", ".tag=1"]);
            let _ = server.write_all(&trap).await;
        });
        let err = Session::from_stream(
            client,
            "admin".into(),
            String::new(),
            Duration::from_secs(2),
        )
        .await
        .err()
        .expect("trap");
        assert_eq!(err.kind(), ErrorKind::Auth);
    }
}
