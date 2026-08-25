//! Tagged `RouterOS` API session over one TLS (or test) stream.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

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
    /// Test-only transcript. Tracing's thread-local subscriber misses events
    /// from `tokio::spawn` and from other cargo-test threads.
    #[cfg(test)]
    log_sink: LogSink,
}

#[cfg(test)]
#[derive(Clone, Default)]
struct LogSink(Arc<std::sync::Mutex<Vec<String>>>);

#[cfg(test)]
impl LogSink {
    fn push(&self, line: impl Into<String>) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(line.into());
    }

    fn text(&self) -> String {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .join("\n")
    }
}

impl SessionInner {
    #[cfg_attr(not(test), allow(clippy::unused_self))]
    fn record_log(&self, line: &str) {
        #[cfg(test)]
        self.log_sink.push(line);
        let _ = line;
    }
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
        #[cfg(test)]
        {
            Self::connect(
                stream,
                username,
                password,
                request_timeout,
                LogSink::default(),
            )
            .await
        }
        #[cfg(not(test))]
        {
            Self::connect(stream, username, password, request_timeout).await
        }
    }

    async fn connect<S>(
        stream: S,
        username: String,
        password: String,
        request_timeout: Duration,
        #[cfg(test)] log_sink: LogSink,
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
            #[cfg(test)]
            log_sink,
        });
        tokio::spawn(read_loop(reader, Arc::clone(&inner)));
        let session = Self { inner };
        session.login().await?;
        Ok(session)
    }

    #[cfg(test)]
    async fn from_stream_with_logs<S>(
        stream: S,
        username: String,
        password: String,
        request_timeout: Duration,
    ) -> (Result<Self>, LogSink)
    where
        S: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let log_sink = LogSink::default();
        let result = Self::connect(
            stream,
            username,
            password,
            request_timeout,
            log_sink.clone(),
        )
        .await;
        (result, log_sink)
    }

    async fn login(&self) -> Result<()> {
        let words = vec![
            "/login".to_string(),
            format!("=name={}", self.inner.username),
            format!("=password={}", self.inner.password),
        ];
        let replies = self.request_words("login", words).await?;
        if let Some(trap) = replies.iter().find(|sentence| sentence.is_trap()) {
            let message = trap
                .attr("message")
                .or_else(|| trap.attr("detail"))
                .unwrap_or("login failed");
            let sentence = trap.log_line();
            log_response_err(
                "login",
                "/login",
                trap.tag().unwrap_or(""),
                Instant::now(),
                message,
                Some(&sentence),
            );
            self.inner
                .record_log(&format!("ERROR response /login {message}"));
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
        let command = command_of(&words).to_string();
        let tag = self.next_tag();
        words.push(format!(".tag={tag}"));
        self.emit_outbound(operation, &words);
        let started = Instant::now();
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.inner.pending.lock().await.insert(tag.clone(), tx);
        if let Err(err) = self.write_sentence(&words).await {
            self.inner.pending.lock().await.remove(&tag);
            log_request_failed(operation, &command, &tag, &err);
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
            Ok(replies) if replies.is_empty() => {
                log_response_err(
                    operation,
                    &command,
                    &tag,
                    started,
                    "connection closed",
                    None,
                );
                self.inner
                    .record_log(&format!("ERROR response {command} connection closed"));
                Err(Error::new(
                    ErrorKind::Transport,
                    operation,
                    "connection closed",
                ))
            }
            Ok(replies) => {
                if let Some(fatal) = replies.iter().find(|sentence| sentence.is_fatal()) {
                    let message = fatal.attr("message").unwrap_or("fatal API error");
                    let sentence = fatal.log_line();
                    log_response_err(operation, &command, &tag, started, message, Some(&sentence));
                    self.inner
                        .record_log(&format!("ERROR response {command} {message}"));
                    return Err(Error::new(ErrorKind::Server, operation, message));
                }
                if let Some(trap) = replies.iter().find(|sentence| sentence.is_trap()) {
                    let message = trap
                        .attr("message")
                        .or_else(|| trap.attr("detail"))
                        .unwrap_or("request failed");
                    let sentence = trap.log_line();
                    log_response_err(operation, &command, &tag, started, message, Some(&sentence));
                    self.inner
                        .record_log(&format!("ERROR response {command} {message}"));
                    return Err(trap.trap_error(operation));
                }
                log_response_ok(operation, &command, &tag, started, Some(&replies));
                self.inner.record_log(&format!("INFO response {command}"));
                Ok(replies)
            }
            Err(_) => {
                log_response_err(
                    operation,
                    &command,
                    &tag,
                    started,
                    "request timed out",
                    None,
                );
                self.inner
                    .record_log(&format!("ERROR response {command} request timed out"));
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
        self.emit_outbound(operation, &words);
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
        self.emit_outbound("cancel", &words);
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

    fn emit_outbound(&self, operation: &str, words: &[String]) {
        log_outbound(operation, words);
        self.inner
            .record_log(&format!("INFO outbound {}", command_of(words)));
    }
}

fn command_of(words: &[String]) -> &str {
    words.first().map_or("?", String::as_str)
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn count_replies(replies: &[Sentence]) -> u64 {
    u64::try_from(replies.iter().filter(|sentence| sentence.is_re()).count()).unwrap_or(u64::MAX)
}

fn join_reply_logs(replies: &[Sentence]) -> String {
    replies
        .iter()
        .map(Sentence::log_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn log_outbound(operation: &str, words: &[String]) {
    let command = command_of(words);
    let sentence = Sentence::new(words.to_vec());
    let tag = sentence.tag().unwrap_or("");
    tracing::info!(
        operation,
        command,
        tag,
        sentence = sentence.log_line().as_str(),
        "outbound {command}"
    );
}

pub(crate) fn log_response_ok(
    operation: &str,
    command: &str,
    tag: &str,
    started: Instant,
    replies: Option<&[Sentence]>,
) {
    let elapsed_ms = elapsed_ms(started);
    match replies {
        Some(replies) => {
            let sentence = join_reply_logs(replies);
            tracing::info!(
                operation,
                command,
                tag,
                elapsed_ms,
                replies = count_replies(replies),
                sentence = sentence.as_str(),
                "response {command}"
            );
        }
        None => {
            tracing::info!(operation, command, tag, elapsed_ms, "response {command}");
        }
    }
}

pub(crate) fn log_response_err(
    operation: &str,
    command: &str,
    tag: &str,
    started: Instant,
    error: &str,
    sentence: Option<&str>,
) {
    let elapsed_ms = elapsed_ms(started);
    match sentence {
        Some(sentence) => {
            tracing::error!(
                operation,
                command,
                tag,
                elapsed_ms,
                error,
                sentence,
                "response {command}"
            );
        }
        None => {
            tracing::error!(
                operation,
                command,
                tag,
                elapsed_ms,
                error,
                "response {command}"
            );
        }
    }
}

fn log_request_failed(operation: &str, command: &str, tag: &str, err: &Error) {
    tracing::error!(
        operation,
        command,
        tag,
        error = err.message(),
        "request {command} failed"
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
                        // Tagged replies are logged with command context by
                        // `request` / `ApiStream`. Untagged `!fatal` still
                        // needs a console line here.
                        if sentence.tag().is_none() {
                            tracing::error!(
                                sentence = sentence.log_line().as_str(),
                                "api error reply"
                            );
                        }
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
    use crate::codec::{SentenceDecoder, encode_sentence};
    use std::pin::Pin;
    use std::task::{Context, Poll, Waker};
    use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};

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

    #[tokio::test]
    async fn successful_print_logs_response_at_info() {
        let (session, logs) = Session::from_stream_with_logs(
            ScriptIo::login_then_print(),
            "admin".into(),
            String::new(),
            Duration::from_secs(2),
        )
        .await;
        let session = session.expect("login");
        session
            .request("list", vec!["/interface/print".into()])
            .await
            .expect("print");

        let text = logs.text();
        assert!(
            text.contains("INFO outbound /login"),
            "missing login outbound: {text}"
        );
        assert!(
            text.contains("INFO response /login"),
            "missing login response: {text}"
        );
        assert!(
            text.contains("INFO outbound /interface/print"),
            "missing print outbound: {text}"
        );
        assert!(
            text.contains("INFO response /interface/print"),
            "missing print response: {text}"
        );
    }

    #[tokio::test]
    async fn login_trap_logs_response_at_error() {
        let (result, logs) = Session::from_stream_with_logs(
            ScriptIo::login_trap(),
            "admin".into(),
            String::new(),
            Duration::from_secs(2),
        )
        .await;
        let err = result.err().expect("trap");
        assert_eq!(err.kind(), ErrorKind::Auth);

        let text = logs.text();
        assert!(
            text.contains("INFO outbound /login"),
            "missing login outbound: {text}"
        );
        assert!(
            text.contains("ERROR response /login"),
            "missing login response: {text}"
        );
        assert!(
            text.contains("cannot log in"),
            "missing trap message: {text}"
        );
    }

    #[derive(Clone, Copy)]
    enum ScriptKind {
        LoginThenPrint,
        LoginTrap,
    }

    struct ScriptIo {
        inner: std::sync::Arc<std::sync::Mutex<ScriptState>>,
    }

    struct ScriptState {
        kind: ScriptKind,
        inbound: SentenceDecoder,
        outbound: Vec<u8>,
        read_at: usize,
        read_waker: Option<Waker>,
    }

    impl ScriptIo {
        fn login_then_print() -> Self {
            Self::new(ScriptKind::LoginThenPrint)
        }

        fn login_trap() -> Self {
            Self::new(ScriptKind::LoginTrap)
        }

        fn new(kind: ScriptKind) -> Self {
            Self {
                inner: std::sync::Arc::new(std::sync::Mutex::new(ScriptState {
                    kind,
                    inbound: SentenceDecoder::new(),
                    outbound: Vec::new(),
                    read_at: 0,
                    read_waker: None,
                })),
            }
        }
    }

    fn sentence_tag(words: &[String]) -> String {
        words
            .iter()
            .find_map(|word| word.strip_prefix(".tag="))
            .unwrap_or("1")
            .to_string()
    }

    fn queue_replies(state: &mut ScriptState, words: &[String]) {
        let tag = sentence_tag(words);
        let command = words.first().map_or("", String::as_str);
        match state.kind {
            ScriptKind::LoginTrap if command == "/login" => {
                state.outbound.extend(encode_sentence(&[
                    "!trap",
                    "=message=cannot log in",
                    &format!(".tag={tag}"),
                ]));
            }
            ScriptKind::LoginThenPrint if command == "/login" => {
                state
                    .outbound
                    .extend(encode_sentence(&["!done", &format!(".tag={tag}")]));
            }
            ScriptKind::LoginThenPrint if command == "/interface/print" => {
                state.outbound.extend(encode_sentence(&[
                    "!re",
                    "=.id=*1",
                    "=name=ether1",
                    &format!(".tag={tag}"),
                ]));
                state
                    .outbound
                    .extend(encode_sentence(&["!done", &format!(".tag={tag}")]));
            }
            _ => {}
        }
        if let Some(waker) = state.read_waker.take() {
            waker.wake();
        }
    }

    impl AsyncRead for ScriptIo {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            let mut state = self
                .inner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.read_at >= state.outbound.len() {
                state.read_waker = Some(cx.waker().clone());
                return Poll::Pending;
            }
            let avail = &state.outbound[state.read_at..];
            let n = avail.len().min(buf.remaining());
            buf.put_slice(&avail[..n]);
            state.read_at += n;
            Poll::Ready(Ok(()))
        }
    }

    impl AsyncWrite for ScriptIo {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            let mut state = self
                .inner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.inbound.push(buf);
            while let Ok(Some(words)) = state.inbound.take_sentence() {
                queue_replies(&mut state, &words);
            }
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }
}
