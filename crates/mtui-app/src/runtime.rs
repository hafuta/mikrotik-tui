//! Event loop: terminal + tokio worker bridge.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use mtui_core::FetchKind;
use mtui_routeros::{Client, ClientOptions, ErrorKind, probe_certificate};
use mtui_ui::ColorDepth;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::{mpsc, watch};

use crate::app::{App, AppCommand};
use crate::event::{AppEvent, WorkerMsg};
use crate::render;
use crate::telemetry::select_wan_interface;
use crate::write::MutationOp;

pub fn run(alt_screen: bool, demo: bool) -> anyhow::Result<()> {
    tracing::info!(color_depth = ?ColorDepth::detect(), "terminal color depth");
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let mut terminal = setup_terminal(alt_screen)?;
    let result = rt.block_on(run_ui(&rt, &mut terminal, alt_screen, demo));
    restore_terminal(&mut terminal, alt_screen)?;
    result
}

fn setup_terminal(alt_screen: bool) -> anyhow::Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    if alt_screen {
        execute!(stdout, EnterAlternateScreen)?;
    }
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    alt_screen: bool,
) -> anyhow::Result<()> {
    disable_raw_mode()?;
    if alt_screen {
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    }
    Ok(())
}

async fn run_ui(
    rt: &tokio::runtime::Runtime,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    alt_screen: bool,
    demo: bool,
) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<WorkerMsg>();
    let (view_tx, _) = watch::channel(0_u64);
    let (torch_tx, _) = watch::channel(0_u64);
    let (probe_tx, _) = watch::channel(0_u64);
    let listen_gate = StreamGate::default();
    let wan_gate = StreamGate::default();
    let mut app = App::new(alt_screen)?;
    let size = terminal.size()?;
    let _ = app.update(AppEvent::Resize {
        width: size.width,
        height: size.height,
    });
    let startup = if demo {
        app.enter_demo()
    } else {
        app.startup_commands()
    };
    dispatch_commands(
        rt,
        &tx,
        &mut app,
        startup,
        &view_tx,
        &torch_tx,
        &probe_tx,
        &listen_gate,
        &wan_gate,
    );
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
                dispatch_commands(
                    rt,
                    &tx,
                    &mut app,
                    cmds,
                    &view_tx,
                    &torch_tx,
                    &probe_tx,
                    &listen_gate,
                    &wan_gate,
                );
            }
            msg = rx.recv() => {
                if let Some(msg) = msg {
                    let cmds = app.update(AppEvent::Worker(msg));
                    dispatch_commands(
                        rt,
                        &tx,
                        &mut app,
                        cmds,
                        &view_tx,
                        &torch_tx,
                        &probe_tx,
                        &listen_gate,
                        &wan_gate,
                    );
                }
            }
            () = tokio::time::sleep(Duration::from_millis(16)) => {
                while event::poll(Duration::from_millis(0))? {
                    match event::read()? {
                        Event::Key(key) if key.kind == KeyEventKind::Press => {
                            let cmds = app.update(AppEvent::Input(key));
                            dispatch_commands(
                                rt,
                                &tx,
                                &mut app,
                                cmds,
                                &view_tx,
                                &torch_tx,
                                &probe_tx,
                                &listen_gate,
                                &wan_gate,
                            );
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
    Ok(())
}

#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn dispatch_commands(
    rt: &tokio::runtime::Runtime,
    tx: &mpsc::UnboundedSender<WorkerMsg>,
    app: &mut App,
    cmds: Vec<AppCommand>,
    view_tx: &watch::Sender<u64>,
    torch_tx: &watch::Sender<u64>,
    probe_tx: &watch::Sender<u64>,
    listen_gate: &StreamGate,
    wan_gate: &StreamGate,
) {
    let _ = view_tx.send_replace(app.poll_generation);
    let _ = torch_tx.send_replace(app.torch_generation);
    let _ = probe_tx.send_replace(app.probe_generation);
    for cmd in cmds {
        if let Some(store) = app.demo.as_mut()
            && let Some(msgs) = crate::demo::handle(store, &cmd)
        {
            for msg in msgs {
                let _ = tx.send(msg);
            }
            continue;
        }
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
                let mut view_rx = view_tx.subscribe();
                let gate = listen_gate.clone();
                rt.spawn(async move {
                    fetch_resource(
                        client,
                        request_id,
                        generation,
                        resource_id,
                        tx,
                        &mut view_rx,
                        &gate,
                    )
                    .await;
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
                let mut view_rx = view_tx.subscribe();
                let gate = wan_gate.clone();
                rt.spawn(async move {
                    fetch_dashboard(client, request_id, generation, tx, &mut view_rx, &gate).await;
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
                let mut view_rx = view_tx.subscribe();
                let gate = wan_gate.clone();
                rt.spawn(async move {
                    fetch_header(client, request_id, generation, tx, &mut view_rx, &gate).await;
                });
            }
            AppCommand::ForgetProfile { name } => app.forget_profile(&name),
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
                        Err(err) => {
                            send_if_auth(&tx, &err);
                            Some(err.to_string())
                        }
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
                let mut torch_rx = torch_tx.subscribe();
                rt.spawn(async move {
                    stream_torch(
                        client,
                        generation,
                        interface,
                        src,
                        dst,
                        protocol,
                        port,
                        tx,
                        &mut torch_rx,
                    )
                    .await;
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
                let mut probe_rx = probe_tx.subscribe();
                let mut fields = std::collections::BTreeMap::new();
                fields.insert("address".into(), address);
                fields.insert("count".into(), count);
                if !src.trim().is_empty() {
                    fields.insert("src-address".into(), src);
                }
                rt.spawn(async move {
                    stream_probe(
                        client,
                        generation,
                        "/rest/tool".into(),
                        "ping".into(),
                        fields,
                        tx,
                        &mut probe_rx,
                    )
                    .await;
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
                let mut probe_rx = probe_tx.subscribe();
                let mut fields = std::collections::BTreeMap::new();
                fields.insert("address".into(), address);
                fields.insert("count".into(), count);
                if !src.trim().is_empty() {
                    fields.insert("src-address".into(), src);
                }
                if !protocol.trim().is_empty() {
                    fields.insert("protocol".into(), protocol);
                }
                rt.spawn(async move {
                    stream_probe(
                        client,
                        generation,
                        "/rest/tool".into(),
                        "traceroute".into(),
                        fields,
                        tx,
                        &mut probe_rx,
                    )
                    .await;
                });
            }
            AppCommand::FetchProbe {
                generation,
                endpoint,
                command,
                fields,
                ..
            } => {
                let Some(client) = app.client.clone() else {
                    continue;
                };
                let tx = tx.clone();
                let mut probe_rx = probe_tx.subscribe();
                rt.spawn(async move {
                    stream_probe(
                        client,
                        generation,
                        endpoint,
                        command,
                        fields,
                        tx,
                        &mut probe_rx,
                    )
                    .await;
                });
            }
            AppCommand::CopyToClipboard { text } => match copy_to_clipboard(&text) {
                Ok(()) => {
                    tracing::info!("copied to clipboard");
                    app.status = "Copied".into();
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
    match Client::connect(options).await {
        Ok(client) => match client.system("/rest/system/resource").await {
            Ok(router) => WorkerMsg::Connected {
                client: Some(Arc::new(client)),
                router: Some(router),
                error: None,
                error_kind: None,
            },
            Err(err) => tls_or_connect_error(url, had_pin, err).await,
        },
        Err(err) => tls_or_connect_error(url, had_pin, err).await,
    }
}

#[derive(Clone, Default)]
struct StreamGate(Arc<Mutex<Option<(u64, String)>>>);

impl StreamGate {
    fn try_own(&self, generation: u64, id: &str) -> bool {
        let Ok(mut slot) = self.0.lock() else {
            return false;
        };
        if slot.as_ref() == Some(&(generation, id.to_string())) {
            return false;
        }
        *slot = Some((generation, id.to_string()));
        true
    }
}

async fn until_stale(rx: &mut watch::Receiver<u64>, generation: u64) {
    loop {
        if *rx.borrow() != generation {
            return;
        }
        if rx.changed().await.is_err() {
            return;
        }
    }
}

async fn fetch_resource(
    client: Arc<Client>,
    request_id: u64,
    generation: u64,
    resource_id: String,
    tx: mpsc::UnboundedSender<WorkerMsg>,
    view_rx: &mut watch::Receiver<u64>,
    gate: &StreamGate,
) {
    let Some(spec) = mtui_core::resource_by_id(&resource_id) else {
        let _ = tx.send(WorkerMsg::ResourceResult {
            request_id,
            generation,
            resource_id,
            rows: Vec::new(),
            error: Some("unknown resource".into()),
        });
        return;
    };
    let result = match spec.fetch {
        FetchKind::Local => Ok(Vec::new()),
        FetchKind::List { endpoint } => client.list(endpoint).await,
        FetchKind::System { endpoint } => client.system(endpoint).await.map(|r| vec![r]),
    };
    match result {
        Ok(rows) => {
            let _ = tx.send(WorkerMsg::ResourceResult {
                request_id,
                generation,
                resource_id: resource_id.clone(),
                rows,
                error: None,
            });
        }
        Err(err) => {
            send_if_auth(&tx, &err);
            let _ = tx.send(WorkerMsg::ResourceResult {
                request_id,
                generation,
                resource_id,
                rows: Vec::new(),
                error: Some(err.to_string()),
            });
            return;
        }
    }
    if !gate.try_own(generation, spec.id) {
        return;
    }
    let stream = match spec.fetch {
        FetchKind::List { .. } if spec.id == "logs" => client.follow_log().await,
        FetchKind::List { endpoint } => client.listen(endpoint).await,
        _ => return,
    };
    let Ok(mut stream) = stream else {
        return;
    };
    loop {
        tokio::select! {
            () = until_stale(view_rx, generation) => {
                let _ = stream.cancel().await;
                return;
            }
            sample = stream.recv() => {
                match sample {
                    Ok(Some(row)) => {
                        let _ = tx.send(WorkerMsg::ListenDelta {
                            generation,
                            resource_id: resource_id.clone(),
                            row,
                        });
                    }
                    Ok(None) | Err(_) => return,
                }
            }
        }
    }
}

fn is_auth_failure(err: &mtui_routeros::Error) -> bool {
    err.kind() == ErrorKind::Auth || matches!(err.status(), Some(401 | 403))
}

fn send_if_auth(tx: &mpsc::UnboundedSender<WorkerMsg>, err: &mtui_routeros::Error) {
    if is_auth_failure(err) {
        let _ = tx.send(WorkerMsg::AuthRequired {
            message: crate::app::classify_connect_error(ErrorKind::Auth, err.message()),
        });
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
        error_kind: Some(err.kind()),
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

async fn fetch_header(
    client: Arc<Client>,
    request_id: u64,
    generation: u64,
    tx: mpsc::UnboundedSender<WorkerMsg>,
    view_rx: &mut watch::Receiver<u64>,
    gate: &StreamGate,
) {
    let (sys, interfaces) = tokio::join!(
        client.system("/rest/system/resource"),
        client.list("/rest/interface"),
    );

    let (system, system_error) = match sys {
        Ok(record) => (Some(record), None),
        Err(err) => {
            send_if_auth(&tx, &err);
            (None, Some(err.to_string()))
        }
    };
    let (interfaces, interface_error) = match interfaces {
        Ok(rows) => (rows, None),
        Err(err) => {
            send_if_auth(&tx, &err);
            (Vec::new(), Some(err.to_string()))
        }
    };
    let wan_name = select_wan_interface(&interfaces)
        .ok()
        .and_then(|iface| iface.field("name").map(ToOwned::to_owned));
    let _ = tx.send(WorkerMsg::HeaderResult {
        request_id,
        generation,
        system,
        system_error,
        interfaces,
        interface_error,
    });
    if let Some(name) = wan_name {
        stream_wan(client, generation, name, tx, view_rx, gate).await;
    }
}

async fn fetch_dashboard(
    client: Arc<Client>,
    request_id: u64,
    generation: u64,
    tx: mpsc::UnboundedSender<WorkerMsg>,
    view_rx: &mut watch::Receiver<u64>,
    gate: &StreamGate,
) {
    let (cpu, sys, interfaces, firewall) = tokio::join!(
        client.list("/rest/system/resource/cpu"),
        client.system("/rest/system/resource"),
        client.list("/rest/interface"),
        client.list("/rest/ip/firewall/filter"),
    );

    let (cpu, cpu_error) = match cpu {
        Ok(rows) => (rows, None),
        Err(err) => {
            send_if_auth(&tx, &err);
            (Vec::new(), Some(err.to_string()))
        }
    };
    let (system, system_error) = match sys {
        Ok(record) => (Some(record), None),
        Err(err) => {
            send_if_auth(&tx, &err);
            (None, Some(err.to_string()))
        }
    };
    let (interfaces, interface_error) = match interfaces {
        Ok(rows) => (rows, None),
        Err(err) => {
            send_if_auth(&tx, &err);
            (Vec::new(), Some(err.to_string()))
        }
    };
    let (firewall, firewall_error) = match firewall {
        Ok(rows) => (rows, None),
        Err(err) => {
            send_if_auth(&tx, &err);
            (Vec::new(), Some(err.to_string()))
        }
    };
    let wan_name = select_wan_interface(&interfaces)
        .ok()
        .and_then(|iface| iface.field("name").map(ToOwned::to_owned));
    let _ = tx.send(WorkerMsg::DashboardResult {
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
    });
    if let Some(name) = wan_name {
        stream_wan(client, generation, name, tx, view_rx, gate).await;
    }
}

async fn stream_wan(
    client: Arc<Client>,
    generation: u64,
    interface: String,
    tx: mpsc::UnboundedSender<WorkerMsg>,
    view_rx: &mut watch::Receiver<u64>,
    gate: &StreamGate,
) {
    if !gate.try_own(generation, "wan") {
        return;
    }
    let Ok(mut stream) = client.monitor_traffic(&interface).await else {
        return;
    };
    loop {
        tokio::select! {
            () = until_stale(view_rx, generation) => {
                let _ = stream.cancel().await;
                return;
            }
            sample = stream.recv() => {
                match sample {
                    Ok(Some(sample)) => {
                        let _ = tx.send(WorkerMsg::WanSample {
                            generation,
                            interface: interface.clone(),
                            sample,
                        });
                    }
                    Ok(None) | Err(_) => return,
                }
            }
        }
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
        MutationOp::Batch { ops } => {
            for op in ops {
                Box::pin(run_mutation(client, op)).await?;
            }
            Ok(())
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn stream_torch(
    client: Arc<Client>,
    generation: u64,
    interface: String,
    src: String,
    dst: String,
    protocol: String,
    port: String,
    tx: mpsc::UnboundedSender<WorkerMsg>,
    gen_rx: &mut watch::Receiver<u64>,
) {
    let mut fields = std::collections::BTreeMap::new();
    fields.insert("interface".into(), interface);
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
    let mut stream = match client.stream_command("/rest/tool", "torch", &fields).await {
        Ok(stream) => stream,
        Err(err) => {
            let _ = tx.send(WorkerMsg::TorchResult {
                generation,
                rows: Vec::new(),
                error: Some(err.to_string()),
                done: true,
            });
            return;
        }
    };
    loop {
        tokio::select! {
            () = until_stale(gen_rx, generation) => {
                let _ = stream.cancel().await;
                return;
            }
            sample = stream.recv() => {
                match sample {
                    Ok(Some(row)) => {
                        let _ = tx.send(WorkerMsg::TorchResult {
                            generation,
                            rows: vec![row.display_row()],
                            error: None,
                            done: false,
                        });
                    }
                    Ok(None) => {
                        let _ = tx.send(WorkerMsg::TorchResult {
                            generation,
                            rows: Vec::new(),
                            error: None,
                            done: true,
                        });
                        return;
                    }
                    Err(err) => {
                        let _ = tx.send(WorkerMsg::TorchResult {
                            generation,
                            rows: Vec::new(),
                            error: Some(err.to_string()),
                            done: true,
                        });
                        return;
                    }
                }
            }
        }
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

#[allow(clippy::too_many_arguments)]
async fn stream_probe(
    client: Arc<Client>,
    generation: u64,
    endpoint: String,
    command: String,
    fields: std::collections::BTreeMap<String, String>,
    tx: mpsc::UnboundedSender<WorkerMsg>,
    gen_rx: &mut watch::Receiver<u64>,
) {
    let mut stream = match client.stream_command(&endpoint, &command, &fields).await {
        Ok(stream) => stream,
        Err(err) => {
            let _ = tx.send(WorkerMsg::PingTraceResult {
                generation,
                rows: Vec::new(),
                error: Some(err.to_string()),
                done: true,
            });
            return;
        }
    };
    loop {
        tokio::select! {
            () = until_stale(gen_rx, generation) => {
                let _ = stream.cancel().await;
                return;
            }
            sample = stream.recv() => {
                match sample {
                    Ok(Some(row)) => {
                        let _ = tx.send(WorkerMsg::PingTraceResult {
                            generation,
                            rows: vec![row.display_row()],
                            error: None,
                            done: false,
                        });
                    }
                    Ok(None) => {
                        let _ = tx.send(WorkerMsg::PingTraceResult {
                            generation,
                            rows: Vec::new(),
                            error: None,
                            done: true,
                        });
                        return;
                    }
                    Err(err) => {
                        let _ = tx.send(WorkerMsg::PingTraceResult {
                            generation,
                            rows: Vec::new(),
                            error: Some(err.to_string()),
                            done: true,
                        });
                        return;
                    }
                }
            }
        }
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
