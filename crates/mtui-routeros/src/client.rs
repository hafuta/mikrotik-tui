//! `RouterOS` classic TCP API client (`api-ssl` or plaintext `api`).

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::pki_types::ServerName;
use serde_json::{Map, Value};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use crate::error::{Error, Result};
use crate::mutate::is_command_name;
use crate::resource::Resource;
use crate::sentence::Sentence;
use crate::session::Session;
use crate::target::{ConnectionTarget, parse_connection_target_for};
use crate::tls;

/// Default request timeout applied when [`ClientOptions::request_timeout`]
/// is left unset.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Configuration accepted by [`Client::connect`].
#[derive(Clone, Debug)]
pub struct ClientOptions {
    /// Router host or `host:port`. The default port is `8729` when
    /// [`use_tls`](Self::use_tls) is true and `8728` otherwise.
    pub target: String,
    pub username: String,
    pub password: String,
    /// Per-request timeout. Defaults to [`DEFAULT_REQUEST_TIMEOUT`] when
    /// `None`.
    pub request_timeout: Option<Duration>,
    /// Use `api-ssl` (TLS). When false, the plaintext `api` service is used
    /// and certificate options are ignored.
    pub use_tls: bool,
    /// PEM- or DER-encoded custom CA bundle. Ignored when
    /// [`certificate_pin`](Self::certificate_pin) is set or TLS is off.
    /// When unset, the OS trust store is used.
    pub ca_pem: Option<Vec<u8>>,
    /// SHA-256 leaf certificate fingerprint to pin (see
    /// [`crate::normalize_certificate_pin`] for accepted formats). When set,
    /// only this exact leaf certificate is trusted and [`ca_pem`](Self::ca_pem)
    /// is ignored.
    pub certificate_pin: Option<String>,
}

impl ClientOptions {
    #[must_use]
    pub fn new(
        target: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            target: target.into(),
            username: username.into(),
            password: password.into(),
            request_timeout: None,
            use_tls: true,
            ca_pem: None,
            certificate_pin: None,
        }
    }

    #[must_use]
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    #[must_use]
    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }

    #[must_use]
    pub fn with_ca_pem(mut self, pem: impl Into<Vec<u8>>) -> Self {
        self.ca_pem = Some(pem.into());
        self
    }

    #[must_use]
    pub fn with_certificate_pin(mut self, pin: impl Into<String>) -> Self {
        self.certificate_pin = Some(pin.into());
        self
    }
}

/// `RouterOS` classic API client (read, write, and streams).
///
/// Cloning a [`Client`] is cheap (it shares the control and stream sessions).
#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    control: Session,
    stream: Session,
    target: ConnectionTarget,
}

impl Client {
    /// Opens two API sessions (control + stream) and logs in.
    pub async fn connect(options: ClientOptions) -> Result<Self> {
        let timeout = options.request_timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT);
        let target = parse_connection_target_for(&options.target, "new_client", options.use_tls)?;
        let control = connect_session(&options, &target, timeout).await?;
        let stream = connect_session(&options, &target, timeout).await?;
        Ok(Self {
            inner: Arc::new(ClientInner {
                control,
                stream,
                target,
            }),
        })
    }

    #[must_use]
    pub fn target(&self) -> &ConnectionTarget {
        &self.inner.target
    }

    /// Fetches a list-like collection (`/path/print`).
    pub async fn list(&self, endpoint: &str) -> Result<Vec<Resource>> {
        let replies = self
            .inner
            .control
            .request("list", vec![command_path(endpoint, "print")?])
            .await?;
        Ok(replies
            .into_iter()
            .filter(Sentence::is_re)
            .map(Sentence::into_resource)
            .collect())
    }

    /// Fetches a single record by opaque `RouterOS` id (`print` + `?.id=`).
    pub async fn get(&self, endpoint: &str, id: &str) -> Result<Resource> {
        if id.trim().is_empty() {
            return Err(Error::api("get", "record id is required"));
        }
        let replies = self
            .inner
            .control
            .request("get", print_item_words(endpoint, id)?)
            .await?;
        replies
            .into_iter()
            .find(Sentence::is_re)
            .map(Sentence::into_resource)
            .ok_or_else(|| Error::new(crate::error::ErrorKind::NotFound, "get", "no such item"))
    }

    /// Fetches a singleton/system-scoped resource.
    pub async fn system(&self, endpoint: &str) -> Result<Resource> {
        let replies = self
            .inner
            .control
            .request("system", vec![command_path(endpoint, "print")?])
            .await?;
        Ok(replies
            .into_iter()
            .find(Sentence::is_re)
            .map(Sentence::into_resource)
            .unwrap_or_default())
    }

    /// Updates a single record (`set`).
    pub async fn patch(
        &self,
        endpoint: &str,
        id: &str,
        fields: &BTreeMap<String, String>,
    ) -> Result<()> {
        if id.trim().is_empty() {
            return Err(Error::api("patch", "record id is required"));
        }
        if fields.is_empty() {
            return Err(Error::api("patch", "no fields to update"));
        }
        let mut words = vec![command_path(endpoint, "set")?, format!("=.id={id}")];
        push_fields(&mut words, fields);
        self.inner.control.request("patch", words).await?;
        Ok(())
    }

    /// Updates a singleton resource (`set` without a record id).
    pub async fn patch_system(
        &self,
        endpoint: &str,
        fields: &BTreeMap<String, String>,
    ) -> Result<()> {
        if fields.is_empty() {
            return Err(Error::api("patch", "no fields to update"));
        }
        let mut words = vec![command_path(endpoint, "set")?];
        push_fields(&mut words, fields);
        self.inner.control.request("patch", words).await?;
        Ok(())
    }

    /// Creates a record (`add`).
    pub async fn put(&self, endpoint: &str, fields: &BTreeMap<String, String>) -> Result<()> {
        let mut words = vec![command_path(endpoint, "add")?];
        push_fields(&mut words, fields);
        self.inner.control.request("put", words).await?;
        Ok(())
    }

    /// Removes a record (`remove`).
    pub async fn delete(&self, endpoint: &str, id: &str) -> Result<()> {
        if id.trim().is_empty() {
            return Err(Error::api("delete", "record id is required"));
        }
        self.inner
            .control
            .request(
                "delete",
                vec![command_path(endpoint, "remove")?, format!("=.id={id}")],
            )
            .await?;
        Ok(())
    }

    /// Runs a console command (`enable`, `copy`, `fetch`, ...).
    pub async fn command(
        &self,
        endpoint: &str,
        command: &str,
        fields: &BTreeMap<String, String>,
    ) -> Result<Value> {
        if !is_command_name(command) {
            return Err(Error::api("command", "invalid command name"));
        }
        let mut words = vec![command_path(endpoint, command)?];
        push_fields(&mut words, fields);
        let replies = self.inner.control.request("command", words).await?;
        Ok(replies_to_json(replies))
    }

    /// Print once then `.listen` on the stream session.
    pub async fn listen(&self, endpoint: &str) -> Result<ApiStream> {
        self.open_stream("listen", vec![command_path(endpoint, "listen")?])
            .await
    }

    /// `/log/print` with `follow`.
    pub async fn follow_log(&self) -> Result<ApiStream> {
        self.open_stream("follow", vec!["/log/print".into(), "=follow=".into()])
            .await
    }

    /// `/interface/monitor-traffic` for one interface.
    pub async fn monitor_traffic(&self, interface: &str) -> Result<ApiStream> {
        if interface.trim().is_empty() {
            return Err(Error::api("monitor-traffic", "interface is required"));
        }
        self.open_stream(
            "monitor-traffic",
            vec![
                "/interface/monitor-traffic".into(),
                format!("=interface={interface}"),
            ],
        )
        .await
    }

    /// Unterminated tool command (`torch`, `ping`, `traceroute`).
    pub async fn stream_command(
        &self,
        endpoint: &str,
        command: &str,
        fields: &BTreeMap<String, String>,
    ) -> Result<ApiStream> {
        if !is_command_name(command) {
            return Err(Error::api("command", "invalid command name"));
        }
        let mut words = vec![command_path(endpoint, command)?];
        push_fields(&mut words, fields);
        self.open_stream("stream", words).await
    }

    async fn open_stream(&self, operation: &'static str, words: Vec<String>) -> Result<ApiStream> {
        let command = words.first().cloned().unwrap_or_else(|| "?".to_string());
        let started = Instant::now();
        let (tag, rx) = self.inner.stream.stream(operation, words).await?;
        Ok(ApiStream {
            tag,
            command,
            operation,
            started,
            rx,
            session: self.inner.stream.clone(),
        })
    }
}

/// Streaming `!re` replies until cancel or `!done`.
pub struct ApiStream {
    tag: String,
    command: String,
    operation: &'static str,
    started: Instant,
    rx: tokio::sync::mpsc::UnboundedReceiver<Sentence>,
    session: Session,
}

impl ApiStream {
    pub async fn recv(&mut self) -> Result<Option<Resource>> {
        loop {
            let Some(sentence) = self.rx.recv().await else {
                return Ok(None);
            };
            if sentence.is_fatal() {
                let message = sentence.attr("message").unwrap_or("fatal API error");
                let line = sentence.log_line();
                crate::session::log_response_err(
                    self.operation,
                    &self.command,
                    &self.tag,
                    self.started,
                    message,
                    Some(&line),
                );
                return Err(Error::new(
                    crate::error::ErrorKind::Server,
                    "stream",
                    message,
                ));
            }
            if sentence.is_trap() {
                let message = sentence
                    .attr("message")
                    .or_else(|| sentence.attr("detail"))
                    .unwrap_or("request failed");
                let line = sentence.log_line();
                crate::session::log_response_err(
                    self.operation,
                    &self.command,
                    &self.tag,
                    self.started,
                    message,
                    Some(&line),
                );
                return Err(sentence.trap_error("stream"));
            }
            if sentence.is_done() {
                crate::session::log_response_ok(
                    self.operation,
                    &self.command,
                    &self.tag,
                    self.started,
                    None,
                );
                return Ok(None);
            }
            if sentence.is_re() {
                let line = sentence.log_line();
                tracing::info!(
                    operation = self.operation,
                    command = self.command.as_str(),
                    tag = self.tag.as_str(),
                    sentence = line.as_str(),
                    "response {}",
                    self.command
                );
                return Ok(Some(sentence.into_resource()));
            }
        }
    }

    pub async fn cancel(self) -> Result<()> {
        self.session.finish_stream(&self.tag).await;
        Ok(())
    }
}

async fn connect_session(
    options: &ClientOptions,
    target: &ConnectionTarget,
    request_timeout: Duration,
) -> Result<Session> {
    let tcp = tokio::time::timeout(
        request_timeout,
        TcpStream::connect((target.host.as_str(), target.port)),
    )
    .await
    .map_err(|_| {
        Error::new(
            crate::error::ErrorKind::Timeout,
            "connect",
            "connect timed out",
        )
    })?
    .map_err(|err| classify_connect(&err.to_string()))?;
    if !options.use_tls {
        return Session::from_stream(
            tcp,
            options.username.clone(),
            options.password.clone(),
            request_timeout,
        )
        .await;
    }
    let config = if let Some(raw_pin) = &options.certificate_pin {
        let pin = tls::normalize_certificate_pin(raw_pin)?;
        tls::client_config_with_pin(&pin)?
    } else if let Some(pem) = &options.ca_pem {
        tls::client_config_with_ca(pem)?
    } else {
        tls::client_config_with_native_roots()?
    };
    let server_name = ServerName::try_from(target.host.clone())
        .map_err(|err| Error::tls("new_client", format!("invalid host name: {err}")))?;
    let connector = TlsConnector::from(Arc::new(config));
    let tls = connector
        .connect(server_name, tcp)
        .await
        .map_err(|err| classify_connect(&err.to_string()))?;
    Session::from_stream(
        tls,
        options.username.clone(),
        options.password.clone(),
        request_timeout,
    )
    .await
}

fn classify_connect(message: &str) -> Error {
    let lower = message.to_lowercase();
    if lower.contains(&tls::PIN_MISMATCH_MARKER.to_lowercase())
        || lower.contains("certificate")
        || lower.contains("invalid peer certificate")
        || lower.contains("unknownissuer")
        || lower.contains("tls")
        || lower.contains("ssl")
    {
        Error::tls("connect", message)
    } else {
        Error::transport("connect", message)
    }
}

fn command_path(endpoint: &str, command: &str) -> Result<String> {
    let path = api_path(endpoint, "command")?;
    Ok(format!("{path}/{command}"))
}

fn print_item_words(endpoint: &str, id: &str) -> Result<Vec<String>> {
    Ok(vec![command_path(endpoint, "print")?, format!("?.id={id}")])
}

fn api_path(endpoint: &str, operation: &'static str) -> Result<String> {
    let trimmed = endpoint.trim();
    let path = trimmed.strip_prefix("/rest").unwrap_or(trimmed);
    if !path.starts_with('/') || path.contains("..") || path.contains(' ') {
        return Err(Error::api(operation, "invalid API path"));
    }
    Ok(path.trim_end_matches('/').to_string())
}

fn push_fields(words: &mut Vec<String>, fields: &BTreeMap<String, String>) {
    for (key, value) in fields {
        if key == ".id" {
            words.push(format!("=.id={value}"));
        } else {
            words.push(format!("={key}={value}"));
        }
    }
}

fn replies_to_json(replies: Vec<Sentence>) -> Value {
    let rows: Vec<Value> = replies
        .into_iter()
        .filter(Sentence::is_re)
        .map(|sentence| {
            let mut map = Map::new();
            let resource = sentence.into_resource();
            if !resource.id.is_empty() {
                map.insert(".id".into(), Value::String(resource.id));
            }
            for (key, value) in resource.fields {
                map.insert(key, Value::String(value));
            }
            Value::Object(map)
        })
        .collect();
    Value::Array(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sentence::{Sentence, merge_listen_record};

    #[test]
    fn strips_rest_prefix() {
        assert_eq!(api_path("/rest/interface", "test").unwrap(), "/interface");
        assert_eq!(api_path("/interface", "test").unwrap(), "/interface");
        assert!(api_path("interface", "test").is_err());
        assert!(api_path("/interface/../system", "test").is_err());
    }

    #[test]
    fn print_by_id_uses_query_word_not_attribute() {
        assert_eq!(
            print_item_words("/rest/interface/ethernet", "*5").unwrap(),
            vec!["/interface/ethernet/print", "?.id=*5"]
        );
        assert_eq!(
            print_item_words("/rest/interface/vlan", "*3").unwrap(),
            vec!["/interface/vlan/print", "?.id=*3"]
        );
    }

    #[test]
    fn print_path() {
        assert_eq!(
            command_path("/rest/interface", "print").unwrap(),
            "/interface/print"
        );
        assert_eq!(command_path("/rest/tool", "fetch").unwrap(), "/tool/fetch");
        assert_eq!(
            command_path("/rest/ipv6/firewall/connection", "remove").unwrap(),
            "/ipv6/firewall/connection/remove"
        );
        assert_eq!(
            command_path("/rest/ipv6/firewall/connection", "print").unwrap(),
            "/ipv6/firewall/connection/print"
        );
        assert_eq!(
            command_path("/rest/tool/romon", "print").unwrap(),
            "/tool/romon/print"
        );
        assert_eq!(
            command_path("/rest/tool/romon/port", "add").unwrap(),
            "/tool/romon/port/add"
        );
        assert_eq!(
            command_path("/rest/tool/graphing/interface", "print").unwrap(),
            "/tool/graphing/interface/print"
        );
        assert_eq!(
            command_path("/rest/system/history", "undo").unwrap(),
            "/system/history/undo"
        );
        assert!(command_path("/rest/system/history/../file", "undo").is_err());
        assert!(is_command_name("undo"));
        assert!(!is_command_name("Undo"));
        assert!(!is_command_name("history/undo"));
    }

    #[test]
    fn empty_print_is_empty_list() {
        let rows: Vec<Resource> = Vec::new();
        assert!(rows.is_empty());
    }

    #[test]
    fn malformed_words_without_value_are_ignored() {
        let sentence = Sentence::new(vec!["!re".into(), "=broken".into(), "=name=ok".into()]);
        let resource = sentence.into_resource();
        assert_eq!(resource.field("name"), Some("ok"));
        assert!(resource.field("broken").is_none());
    }

    #[test]
    fn merge_listen_is_available() {
        let mut rows = Vec::new();
        merge_listen_record(
            &mut rows,
            Resource {
                id: "*1".into(),
                fields: BTreeMap::from([("name".into(), "ether1".into())])
                    .into_iter()
                    .collect(),
            },
        );
        assert_eq!(rows.len(), 1);
    }
}
