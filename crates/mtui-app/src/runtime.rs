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
            AppCommand::FetchSystem => {
                let Some(client) = app.client.clone() else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg = match client.system("/rest/system/resource").await {
                        Ok(system) => WorkerMsg::SystemResult {
                            system: Some(system),
                            error: None,
                        },
                        Err(err) => WorkerMsg::SystemResult {
                            system: None,
                            error: Some(err.to_string()),
                        },
                    };
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
