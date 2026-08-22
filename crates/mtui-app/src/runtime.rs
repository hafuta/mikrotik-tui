//! Event loop: terminal + tokio worker bridge.

use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use mtui_core::FetchKind;
use mtui_routeros::{Client, ClientOptions, ErrorKind, probe_certificate};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;

use crate::app::{App, AppCommand};
use crate::event::{AppEvent, WorkerMsg};
use crate::render;
use crate::write::{MutationOp, json_rows};

pub fn run(alt_screen: bool) -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    if alt_screen {
        execute!(stdout, EnterAlternateScreen)?;
    }
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = mpsc::unbounded_channel::<WorkerMsg>();
    let mut app = App::new(alt_screen)?;
    let size = terminal.size()?;
    let _ = app.update(AppEvent::Resize {
        width: size.width,
        height: size.height,
    });
    let startup = app.startup_commands();

    let result = rt.block_on(async {
        dispatch_commands(&rt, &tx, &mut app, startup);
        let mut tick = tokio::time::interval(Duration::from_secs(2));
        loop {
            app.pull_console_logs();
            terminal.draw(|f| render::draw(f, &app))?;

            if app.should_quit {
                break;
            }

            tokio::select! {
                _ = tick.tick() => {
                    let cmds = app.update(AppEvent::Tick);
                    dispatch_commands(&rt, &tx, &mut app, cmds);
                }
                msg = rx.recv() => {
                    if let Some(msg) = msg {
                        let cmds = app.update(AppEvent::Worker(msg));
                        dispatch_commands(&rt, &tx, &mut app, cmds);
                    }
                }
                () = tokio::time::sleep(Duration::from_millis(16)) => {
                    while event::poll(Duration::from_millis(0))? {
                        match event::read()? {
                            Event::Key(key) if key.kind == KeyEventKind::Press => {
                                let cmds = app.update(AppEvent::Input(key));
                                dispatch_commands(&rt, &tx, &mut app, cmds);
                            }
                            Event::Resize(width, height) => {
                                let _ = app.update(AppEvent::Resize { width, height });
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    });

    disable_raw_mode()?;
    if alt_screen {
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    }
    result
}

#[allow(clippy::too_many_lines)]
fn dispatch_commands(
    rt: &tokio::runtime::Runtime,
    tx: &mpsc::UnboundedSender<WorkerMsg>,
    app: &mut App,
    cmds: Vec<AppCommand>,
) {
    for cmd in cmds {
        match cmd {
            AppCommand::Quit => app.should_quit = true,
            AppCommand::Connect {
                url,
                username,
                password,
                pin,
                ca_pem,
            } => {
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg = connect_worker(url, username, password, pin, ca_pem).await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::FetchResource {
                request_id,
                generation,
                resource_id,
            } => {
                let Some(client) = app.client.clone() else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg = fetch_resource(client, request_id, generation, &resource_id).await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::FetchDashboard {
                request_id,
                generation,
            } => {
                let Some(client) = app.client.clone() else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg = fetch_dashboard(client, request_id, generation).await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::FetchHeader {
                request_id,
                generation,
            } => {
                let Some(client) = app.client.clone() else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg = fetch_header(client, request_id, generation).await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::ClearSession => app.clear_saved_session(),
            AppCommand::Mutate {
                request_id,
                generation,
                op,
            } => {
                let Some(client) = app.client.clone() else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let error = match run_mutation(&client, op).await {
                        Ok(()) => None,
                        Err(err) => Some(err.to_string()),
                    };
                    let _ = tx.send(WorkerMsg::MutateResult {
                        request_id,
                        generation,
                        error,
                    });
                });
            }
            AppCommand::FetchTorch {
                generation,
                interface,
                src,
                dst,
                protocol,
                port,
                ..
            } => {
                let Some(client) = app.client.clone() else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg =
                        fetch_torch(client, generation, interface, src, dst, protocol, port).await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::FetchPing {
                generation,
                address,
                count,
                src,
                ..
            } => {
                let Some(client) = app.client.clone() else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg =
                        fetch_probe(client, generation, "ping", address, count, src, None).await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::FetchTraceroute {
                generation,
                address,
                count,
                src,
                protocol,
                ..
            } => {
                let Some(client) = app.client.clone() else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg = fetch_probe(
                        client,
                        generation,
                        "traceroute",
                        address,
                        count,
                        src,
                        Some(protocol),
                    )
                    .await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::CopyToClipboard { text } => match copy_to_clipboard(&text) {
                Ok(()) => {
                    tracing::info!("copied log to clipboard");
                    app.status = "Copied log to clipboard".into();
                }
                Err(err) => {
                    tracing::warn!(error = %err, "clipboard copy failed");
                    app.status = format!("Clipboard copy failed: {err}");
                }
            },
            AppCommand::ReadLocalFile {
                request_id,
                generation,
                path,
                remote_name,
            } => {
                let tx = tx.clone();
                rt.spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        crate::files_io::read_utf8_upload(std::path::Path::new(&path))
                    })
                    .await;
                    let (contents, error) = match result {
                        Ok(Ok(contents)) => (Some(contents), None),
                        Ok(Err(err)) => (None, Some(err)),
                        Err(err) => (None, Some(err.to_string())),
                    };
                    let _ = tx.send(WorkerMsg::ReadLocalFileResult {
                        request_id,
                        generation,
                        remote_name,
                        contents,
                        error,
                    });
                });
            }
            AppCommand::WriteLocalFile {
                request_id,
                generation,
                path,
                contents,
            } => {
                let tx = tx.clone();
                rt.spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        crate::files_io::write_download(std::path::Path::new(&path), &contents)
                    })
                    .await;
                    let error = match result {
                        Ok(Ok(())) => None,
                        Ok(Err(err)) => Some(err),
                        Err(err) => Some(err.to_string()),
                    };
                    let _ = tx.send(WorkerMsg::WriteLocalFileResult {
                        request_id,
                        generation,
                        error,
                    });
                });
            }
            AppCommand::FetchRecord {
                request_id,
                generation,
                endpoint,
                id,
                local_path,
            } => {
                let Some(client) = app.client.clone() else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg =
                        fetch_file_record(client, request_id, generation, endpoint, id, local_path)
                            .await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::FetchLookup {
                request_id,
                generation,
                resource_id,
                value_key,
            } => {
                let tx = tx.clone();
                let client = app.client.clone();
                rt.spawn(async move {
                    let msg =
                        fetch_lookup(client, request_id, generation, resource_id, value_key).await;
                    let _ = tx.send(msg);
                });
            }
        }
    }
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(text.to_string()))
        .map_err(|err| err.to_string())
}

async fn connect_worker(
    url: String,
    username: String,
    password: String,
    pin: Option<String>,
    ca_pem: Option<Vec<u8>>,
) -> WorkerMsg {
    let had_pin = pin.is_some();
    let mut options = ClientOptions::new(url.clone(), username, password);
    if let Some(pin) = pin {
        options = options.with_certificate_pin(pin);
    }
    if let Some(pem) = ca_pem {
        options = options.with_ca_pem(pem);
    }
    match Client::new(options) {
        Ok(client) => match client.system("/rest/system/resource").await {
            Ok(router) => WorkerMsg::Connected {
                client: Some(Arc::new(client)),
                router: Some(router),
                error: None,
            },
            Err(err) => tls_or_connect_error(url, had_pin, err).await,
        },
        Err(err) => tls_or_connect_error(url, had_pin, err).await,
    }
}

async fn tls_or_connect_error(url: String, had_pin: bool, err: mtui_routeros::Error) -> WorkerMsg {
    if !had_pin && err.kind() == ErrorKind::Tls {
        return match probe_certificate(&url).await {
            Ok(fingerprint) => WorkerMsg::ProbeResult {
                fingerprint: Some(fingerprint),
                error: None,
            },
            Err(probe_err) => WorkerMsg::ProbeResult {
                fingerprint: None,
                error: Some(probe_err.to_string()),
            },
        };
    }
    WorkerMsg::Connected {
        client: None,
        router: None,
        error: Some(err.to_string()),
    }
}

async fn fetch_resource(
    client: Arc<Client>,
    request_id: u64,
    generation: u64,
    resource_id: &str,
) -> WorkerMsg {
    let Some(spec) = mtui_core::resource_by_id(resource_id) else {
        return WorkerMsg::ResourceResult {
            request_id,
            generation,
            resource_id: resource_id.to_string(),
            rows: Vec::new(),
            error: Some("unknown resource".into()),
        };
    };
    let result = match spec.fetch {
        FetchKind::Local => Ok(Vec::new()),
        FetchKind::List { endpoint } => client.list(endpoint).await,
        FetchKind::System { endpoint } => client.system(endpoint).await.map(|r| vec![r]),
    };
    match result {
        Ok(rows) => WorkerMsg::ResourceResult {
            request_id,
            generation,
            resource_id: resource_id.to_string(),
            rows,
            error: None,
        },
        Err(err) => WorkerMsg::ResourceResult {
            request_id,
            generation,
            resource_id: resource_id.to_string(),
            rows: Vec::new(),
            error: Some(err.to_string()),
        },
    }
}

async fn fetch_lookup(
    client: Option<Arc<Client>>,
    request_id: u64,
    generation: u64,
    resource_id: String,
    value_key: String,
) -> WorkerMsg {
    let Some(spec) = mtui_core::resource_by_id(&resource_id) else {
        return WorkerMsg::LookupResult {
            request_id,
            generation,
            options: Vec::new(),
            error: Some("unknown resource".into()),
        };
    };
    let Some(client) = client else {
        return WorkerMsg::LookupResult {
            request_id,
            generation,
            options: Vec::new(),
            error: Some("not connected".into()),
        };
    };
    let result = match spec.fetch {
        FetchKind::Local => Ok(Vec::new()),
        FetchKind::List { endpoint } => client.list(endpoint).await,
        FetchKind::System { endpoint } => client.system(endpoint).await.map(|r| vec![r]),
    };
    match result {
        Ok(rows) => WorkerMsg::LookupResult {
            request_id,
            generation,
            options: lookup_option_values(&rows, &value_key),
            error: None,
        },
        Err(err) => WorkerMsg::LookupResult {
            request_id,
            generation,
            options: Vec::new(),
            error: Some(err.to_string()),
        },
    }
}

fn lookup_option_values(rows: &[mtui_routeros::Resource], value_key: &str) -> Vec<String> {
    let mut out = Vec::new();
    for row in rows {
        let value = if value_key == ".id" {
            Some(row.id.as_str())
        } else {
            row.field(value_key)
        };
        let Some(value) = value.filter(|item| !item.is_empty()) else {
            continue;
        };
        if !out.iter().any(|item| item == value) {
            out.push(value.to_string());
        }
    }
    out
}

async fn fetch_header(client: Arc<Client>, request_id: u64, generation: u64) -> WorkerMsg {
    let (sys, interfaces) = tokio::join!(
        client.system("/rest/system/resource"),
        client.list("/rest/interface"),
    );

    let (system, system_error) = match sys {
        Ok(record) => (Some(record), None),
        Err(err) => (None, Some(err.to_string())),
    };
    let (interfaces, interface_error) = match interfaces {
        Ok(rows) => (rows, None),
        Err(err) => (Vec::new(), Some(err.to_string())),
    };

    WorkerMsg::HeaderResult {
        request_id,
        generation,
        system,
        system_error,
        interfaces,
        interface_error,
    }
}

async fn fetch_dashboard(client: Arc<Client>, request_id: u64, generation: u64) -> WorkerMsg {
    let (cpu, sys, interfaces, firewall) = tokio::join!(
        client.list("/rest/system/resource/cpu"),
        client.system("/rest/system/resource"),
        client.list("/rest/interface"),
        client.list("/rest/ip/firewall/filter"),
    );

    let (cpu, cpu_error) = match cpu {
        Ok(rows) => (rows, None),
        Err(err) => (Vec::new(), Some(err.to_string())),
    };
    let (system, system_error) = match sys {
        Ok(record) => (Some(record), None),
        Err(err) => (None, Some(err.to_string())),
    };
    let (interfaces, interface_error) = match interfaces {
        Ok(rows) => (rows, None),
        Err(err) => (Vec::new(), Some(err.to_string())),
    };
    let (firewall, firewall_error) = match firewall {
        Ok(rows) => (rows, None),
        Err(err) => (Vec::new(), Some(err.to_string())),
    };

    WorkerMsg::DashboardResult {
        request_id,
        generation,
        cpu,
        cpu_error,
        system,
        system_error,
        interfaces,
        interface_error,
        firewall,
        firewall_error,
    }
}

async fn run_mutation(client: &Client, op: MutationOp) -> mtui_routeros::Result<()> {
    match op {
        MutationOp::Patch {
            endpoint,
            id,
            fields,
        } => {
            if let Some(id) = id {
                client.patch(&endpoint, &id, &fields).await
            } else {
                client.patch_system(&endpoint, &fields).await
            }
        }
        MutationOp::Put { endpoint, fields } => client.put(&endpoint, &fields).await,
        MutationOp::Delete { endpoint, id } => client.delete(&endpoint, &id).await,
        MutationOp::Command {
            endpoint,
            command,
            fields,
        } => client
            .command(&endpoint, &command, &fields)
            .await
            .map(|_| ()),
    }
}

async fn fetch_torch(
    client: std::sync::Arc<Client>,
    generation: u64,
    interface: String,
    src: String,
    dst: String,
    protocol: String,
    port: String,
) -> WorkerMsg {
    let mut fields = std::collections::BTreeMap::new();
    fields.insert("interface".into(), interface);
    fields.insert("duration".into(), "2s".into());
    if !src.trim().is_empty() {
        fields.insert("src-address".into(), src);
    }
    if !dst.trim().is_empty() {
        fields.insert("dst-address".into(), dst);
    }
    if !protocol.trim().is_empty() {
        fields.insert("ip-protocol".into(), protocol);
    }
    if !port.trim().is_empty() {
        fields.insert("port".into(), port);
    }
    match client.command("/rest/tool", "torch", &fields).await {
        Ok(value) => WorkerMsg::TorchResult {
            generation,
            rows: json_rows(value),
            error: None,
        },
        Err(err) => WorkerMsg::TorchResult {
            generation,
            rows: Vec::new(),
            error: Some(err.to_string()),
        },
    }
}

async fn fetch_file_record(
    client: std::sync::Arc<Client>,
    request_id: u64,
    generation: u64,
    endpoint: String,
    id: String,
    local_path: String,
) -> WorkerMsg {
    match client.get(&endpoint, &id).await {
        Ok(resource) => WorkerMsg::RecordResult {
            request_id,
            generation,
            local_path,
            contents: resource.fields.get("contents").cloned(),
            error: None,
        },
        Err(err) => WorkerMsg::RecordResult {
            request_id,
            generation,
            local_path,
            contents: None,
            error: Some(err.to_string()),
        },
    }
}

async fn fetch_probe(
    client: std::sync::Arc<Client>,
    generation: u64,
    command: &str,
    address: String,
    count: String,
    src: String,
    protocol: Option<String>,
) -> WorkerMsg {
    let mut fields = std::collections::BTreeMap::new();
    fields.insert("address".into(), address);
    fields.insert("count".into(), count);
    if !src.trim().is_empty() {
        fields.insert("src-address".into(), src);
    }
    if let Some(protocol) = protocol
        && !protocol.trim().is_empty()
    {
        fields.insert("protocol".into(), protocol);
    }
    match client.command("/rest/tool", command, &fields).await {
        Ok(value) => WorkerMsg::PingTraceResult {
            generation,
            rows: json_rows(value),
            error: None,
        },
        Err(err) => WorkerMsg::PingTraceResult {
            generation,
            rows: Vec::new(),
            error: Some(err.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::lookup_option_values;
    use mtui_routeros::Resource;
    use std::collections::HashMap;

    #[test]
    fn lookup_skips_empty_values_and_dedupes() {
        let mut first = HashMap::new();
        first.insert("name".into(), "ether1".into());
        let mut empty = HashMap::new();
        empty.insert("name".into(), String::new());
        let mut again = HashMap::new();
        again.insert("name".into(), "ether1".into());
        let mut second = HashMap::new();
        second.insert("name".into(), "ether2".into());
        let rows = vec![
            Resource {
                id: "*1".into(),
                fields: first,
            },
            Resource {
                id: "*2".into(),
                fields: empty,
            },
            Resource {
                id: "*3".into(),
                fields: again,
            },
            Resource {
                id: "*4".into(),
                fields: second,
            },
        ];
        assert_eq!(lookup_option_values(&rows, "name"), ["ether1", "ether2"]);
    }
}
