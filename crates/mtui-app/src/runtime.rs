//! Event loop: terminal + tokio worker bridge.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use mtui_core::{FetchKind, routeros_meets_minimum, truthy};
use mtui_routeros::{Client, ClientOptions, ErrorKind, probe_certificate};
use mtui_ssh::{SshConnectOptions, SshInput, SshPty};
use mtui_ui::ColorDepth;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::{mpsc, watch};

use crate::app::{App, AppCommand};
use crate::event::{AppEvent, WorkerMsg};
use crate::render;
use crate::session::SessionId;
use crate::telemetry::select_wan_interface;
use crate::write::MutationOp;

struct SessionIo {
    view_tx: watch::Sender<u64>,
    torch_tx: watch::Sender<u64>,
    probe_tx: watch::Sender<u64>,
    listen_gate: StreamGate,
    wan_gate: StreamGate,
    ssh_tx: Option<mpsc::UnboundedSender<SshInput>>,
    ssh_cancel: Option<tokio::sync::oneshot::Sender<()>>,
    ssh_gen: u64,
}

impl SessionIo {
    fn new() -> Self {
        let (view_tx, _) = watch::channel(0_u64);
        let (torch_tx, _) = watch::channel(0_u64);
        let (probe_tx, _) = watch::channel(0_u64);
        Self {
            view_tx,
            torch_tx,
            probe_tx,
            listen_gate: StreamGate::default(),
            wan_gate: StreamGate::default(),
            ssh_tx: None,
            ssh_cancel: None,
            ssh_gen: 0,
        }
    }
}

fn ensure_io(ios: &mut HashMap<SessionId, SessionIo>, id: SessionId) -> &SessionIo {
    ios.entry(id).or_insert_with(SessionIo::new)
}

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
    let (ssh_tx, mut ssh_rx) = mpsc::unbounded_channel::<WorkerMsg>();
    let mut ios: HashMap<SessionId, SessionIo> = HashMap::new();
    let mut app = App::new(alt_screen)?;
    ensure_io(&mut ios, app.active);
    let size = terminal.size()?;
    let _ = app.update(AppEvent::Resize {
        width: size.width,
        height: size.height,
    });
    let startup = if demo {
        let cmds = app.enter_demo();
        app.stamp(cmds)
    } else {
        let cmds = app.startup_commands();
        app.stamp(cmds)
    };
    dispatch_commands(rt, &tx, &ssh_tx, &mut app, &mut ios, startup);
    let mut tick = tokio::time::interval(Duration::from_secs(2));
    loop {
        app.pull_console_logs();
        terminal.draw(|f| render::draw(f, &app))?;
        if app.should_quit {
            break;
        }
        tokio::select! {
            biased;
            msg = ssh_rx.recv() => {
                if let Some(msg) = msg {
                    apply_worker_frame(rt, &tx, &ssh_tx, &mut app, &mut ios, &mut ssh_rx, msg);
                }
            }
            _ = tick.tick() => {
                let cmds = app.update(AppEvent::Tick);
                dispatch_commands(rt, &tx, &ssh_tx, &mut app, &mut ios, cmds);
            }
            msg = rx.recv() => {
                if let Some(msg) = msg {
                    apply_worker_frame(rt, &tx, &ssh_tx, &mut app, &mut ios, &mut rx, msg);
                }
            }
            () = tokio::time::sleep(Duration::from_millis(16)) => {}
        }
        drain_input(rt, &tx, &ssh_tx, &mut app, &mut ios)?;
    }
    Ok(())
}

/// Cap so a listen/follow dump cannot redraw once per row and starve keys.
const WORKER_MSGS_PER_FRAME: usize = 32;

fn apply_worker_frame(
    rt: &tokio::runtime::Runtime,
    tx: &mpsc::UnboundedSender<WorkerMsg>,
    ssh_tx: &mpsc::UnboundedSender<WorkerMsg>,
    app: &mut App,
    ios: &mut HashMap<SessionId, SessionIo>,
    rx: &mut mpsc::UnboundedReceiver<WorkerMsg>,
    first: WorkerMsg,
) {
    for msg in take_worker_batch(rx, first) {
        let cmds = app.update(AppEvent::Worker(msg));
        dispatch_commands(rt, tx, ssh_tx, app, ios, cmds);
    }
}

fn take_worker_batch(
    rx: &mut mpsc::UnboundedReceiver<WorkerMsg>,
    first: WorkerMsg,
) -> Vec<WorkerMsg> {
    let mut batch = Vec::with_capacity(WORKER_MSGS_PER_FRAME);
    batch.push(first);
    while batch.len() < WORKER_MSGS_PER_FRAME {
        match rx.try_recv() {
            Ok(msg) => batch.push(msg),
            Err(_) => break,
        }
    }
    let mut batch = coalesce_ssh_data(batch);
    drain_follow_on_ssh_data(rx, &mut batch);
    batch
}

fn coalesce_ssh_data(batch: Vec<WorkerMsg>) -> Vec<WorkerMsg> {
    let mut out = Vec::with_capacity(batch.len());
    for msg in batch {
        if let WorkerMsg::SshData {
            session,
            generation,
            bytes,
        } = msg
        {
            if let Some(WorkerMsg::SshData {
                session: prev_session,
                generation: prev_generation,
                bytes: buf,
            }) = out.last_mut()
                && *prev_session == session
                && *prev_generation == generation
            {
                buf.extend(bytes);
                continue;
            }
            out.push(WorkerMsg::SshData {
                session,
                generation,
                bytes,
            });
        } else {
            out.push(msg);
        }
    }
    out
}

fn drain_follow_on_ssh_data(
    rx: &mut mpsc::UnboundedReceiver<WorkerMsg>,
    batch: &mut Vec<WorkerMsg>,
) {
    let Some((session, generation)) = batch.last().and_then(|msg| match msg {
        WorkerMsg::SshData {
            session,
            generation,
            ..
        } => Some((*session, *generation)),
        _ => None,
    }) else {
        return;
    };
    loop {
        match rx.try_recv() {
            Ok(WorkerMsg::SshData {
                session: next_session,
                generation: next_generation,
                bytes,
            }) if next_session == session && next_generation == generation => {
                if let Some(WorkerMsg::SshData { bytes: buf, .. }) = batch.last_mut() {
                    buf.extend(bytes);
                }
            }
            Ok(other) => {
                batch.push(other);
                break;
            }
            Err(_) => break,
        }
    }
}

fn drain_input(
    rt: &tokio::runtime::Runtime,
    tx: &mpsc::UnboundedSender<WorkerMsg>,
    ssh_tx: &mpsc::UnboundedSender<WorkerMsg>,
    app: &mut App,
    ios: &mut HashMap<SessionId, SessionIo>,
) -> anyhow::Result<()> {
    while event::poll(Duration::from_millis(0))? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let cmds = app.update(AppEvent::Input(key));
                dispatch_commands(rt, tx, ssh_tx, app, ios, cmds);
            }
            Event::Resize(width, height) => {
                let cmds = app.update(AppEvent::Resize { width, height });
                dispatch_commands(rt, tx, ssh_tx, app, ios, cmds);
            }
            _ => {}
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn dispatch_commands(
    rt: &tokio::runtime::Runtime,
    tx: &mpsc::UnboundedSender<WorkerMsg>,
    ssh_tx: &mpsc::UnboundedSender<WorkerMsg>,
    app: &mut App,
    ios: &mut HashMap<SessionId, SessionIo>,
    cmds: Vec<AppCommand>,
) {
    ios.retain(|id, _| app.session(*id).is_some());
    for cmd in cmds {
        if let AppCommand::CloseSession { session } = cmd {
            ios.remove(&session);
            continue;
        }
        if let AppCommand::CloseSsh { session, .. } = cmd {
            if let Some(io) = ios.get_mut(&session) {
                io.ssh_tx = None;
                if let Some(cancel) = io.ssh_cancel.take() {
                    let _ = cancel.send(());
                }
            }
            continue;
        }
        if let AppCommand::Quit = cmd {
            app.should_quit = true;
            continue;
        }
        let Some(session) = cmd.session() else {
            continue;
        };
        if app.session(session).is_none() {
            continue;
        }
        let io = ensure_io(ios, session);
        if let Some(target) = app.session(session) {
            let _ = io.view_tx.send_replace(target.poll_generation);
            let _ = io.torch_tx.send_replace(target.torch_generation);
            let _ = io.probe_tx.send_replace(target.probe_generation);
        }
        let demo_msgs = app.session_mut(session).and_then(|target| {
            target
                .demo
                .as_mut()
                .and_then(|store| crate::demo::handle(store, &cmd))
        });
        if let Some(msgs) = demo_msgs {
            for msg in msgs {
                if matches!(
                    msg,
                    WorkerMsg::SshReady { .. }
                        | WorkerMsg::SshData { .. }
                        | WorkerMsg::SshClosed { .. }
                        | WorkerMsg::SshService { .. }
                ) {
                    let _ = ssh_tx.send(msg);
                } else {
                    let _ = tx.send(msg);
                }
            }
            continue;
        }
        let client = app
            .session(session)
            .and_then(|target| target.client.clone());
        if matches!(
            &cmd,
            AppCommand::ProbeSshService { .. }
                | AppCommand::OpenSsh { .. }
                | AppCommand::SshWrite { .. }
                | AppCommand::SshResize { .. }
        ) {
            dispatch_ssh(rt, ssh_tx, ios, client, cmd);
            continue;
        }
        let io = ios
            .get(&session)
            .expect("session I/O created before dispatch");
        match cmd {
            AppCommand::Quit
            | AppCommand::CloseSession { .. }
            | AppCommand::ProbeSshService { .. }
            | AppCommand::OpenSsh { .. }
            | AppCommand::SshWrite { .. }
            | AppCommand::SshResize { .. }
            | AppCommand::CloseSsh { .. } => {}
            AppCommand::Connect {
                session,
                url,
                username,
                password,
                pin,
                ca_pem,
                use_tls,
            } => {
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg =
                        connect_worker(session, url, username, password, pin, ca_pem, use_tls)
                            .await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::FetchResource {
                session,
                request_id,
                generation,
                resource_id,
            } => {
                let Some(client) = client else {
                    continue;
                };
                let tx = tx.clone();
                let mut view_rx = io.view_tx.subscribe();
                let gate = io.listen_gate.clone();
                rt.spawn(async move {
                    fetch_resource(
                        session,
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
                session,
                request_id,
                generation,
            } => {
                let Some(client) = client else {
                    continue;
                };
                let tx = tx.clone();
                let mut view_rx = io.view_tx.subscribe();
                let gate = io.wan_gate.clone();
                rt.spawn(async move {
                    fetch_dashboard(
                        session,
                        client,
                        request_id,
                        generation,
                        tx,
                        &mut view_rx,
                        &gate,
                    )
                    .await;
                });
            }
            AppCommand::FetchAccess {
                session,
                request_id,
                generation,
            } => {
                let Some(client) = client else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg = fetch_access(session, client, request_id, generation).await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::ProbeMenuPaths {
                session,
                generation,
            } => {
                let Some(client) = client else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg = probe_menu_paths(session, client, generation).await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::FetchHeader {
                session,
                request_id,
                generation,
            } => {
                let Some(client) = client else {
                    continue;
                };
                let tx = tx.clone();
                let mut view_rx = io.view_tx.subscribe();
                let gate = io.wan_gate.clone();
                rt.spawn(async move {
                    fetch_header(
                        session,
                        client,
                        request_id,
                        generation,
                        tx,
                        &mut view_rx,
                        &gate,
                    )
                    .await;
                });
            }
            AppCommand::ForgetProfile { name, .. } => app.forget_profile(&name),
            AppCommand::Mutate {
                session,
                request_id,
                generation,
                op,
            } => {
                let Some(client) = client else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let error = match run_mutation(&client, op).await {
                        Ok(()) => None,
                        Err(err) => {
                            send_if_session_event(&tx, session, generation, &err);
                            Some(err.to_string())
                        }
                    };
                    let _ = tx.send(WorkerMsg::MutateResult {
                        session,
                        request_id,
                        generation,
                        error,
                    });
                });
            }
            AppCommand::FetchTorch {
                session,
                generation,
                interface,
                src,
                dst,
                protocol,
                port,
                ..
            } => {
                let Some(client) = client else {
                    continue;
                };
                let tx = tx.clone();
                let mut torch_rx = io.torch_tx.subscribe();
                rt.spawn(async move {
                    stream_torch(
                        session,
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
                session,
                generation,
                address,
                count,
                src,
                ..
            } => {
                let Some(client) = client else {
                    continue;
                };
                let tx = tx.clone();
                let mut probe_rx = io.probe_tx.subscribe();
                let mut fields = std::collections::BTreeMap::new();
                fields.insert("address".into(), address);
                fields.insert("count".into(), count);
                if !src.trim().is_empty() {
                    fields.insert("src-address".into(), src);
                }
                rt.spawn(async move {
                    stream_probe(
                        session,
                        client,
                        generation,
                        "/tool".into(),
                        "ping".into(),
                        fields,
                        tx,
                        &mut probe_rx,
                    )
                    .await;
                });
            }
            AppCommand::FetchTraceroute {
                session,
                generation,
                address,
                count,
                src,
                protocol,
                ..
            } => {
                let Some(client) = client else {
                    continue;
                };
                let tx = tx.clone();
                let mut probe_rx = io.probe_tx.subscribe();
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
                        session,
                        client,
                        generation,
                        "/tool".into(),
                        "traceroute".into(),
                        fields,
                        tx,
                        &mut probe_rx,
                    )
                    .await;
                });
            }
            AppCommand::FetchProbe {
                session,
                generation,
                endpoint,
                command,
                fields,
                ..
            } => {
                let Some(client) = client else {
                    continue;
                };
                let tx = tx.clone();
                let mut probe_rx = io.probe_tx.subscribe();
                rt.spawn(async move {
                    stream_probe(
                        session,
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
            AppCommand::CopyToClipboard { session, text } => {
                let status = match copy_to_clipboard(&text) {
                    Ok(()) => {
                        tracing::info!("copied to clipboard");
                        "Copied".into()
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "clipboard copy failed");
                        format!("Clipboard copy failed: {err}")
                    }
                };
                if let Some(target) = app.session_mut(session) {
                    target.status = status;
                }
            }
            AppCommand::ReadLocalFile {
                session,
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
                        session,
                        request_id,
                        generation,
                        remote_name,
                        contents,
                        error,
                    });
                });
            }
            AppCommand::WriteLocalFile {
                session,
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
                        session,
                        request_id,
                        generation,
                        error,
                    });
                });
            }
            AppCommand::FetchRecord {
                session,
                request_id,
                generation,
                id,
                name,
                local_path,
            } => {
                let Some(client) = client else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg = fetch_file_record(
                        session, client, request_id, generation, id, name, local_path,
                    )
                    .await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::FetchLookup {
                session,
                request_id,
                generation,
                resource_id,
                value_key,
            } => {
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg = fetch_lookup(
                        session,
                        client,
                        request_id,
                        generation,
                        resource_id,
                        value_key,
                    )
                    .await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::FetchFormRecord {
                session,
                request_id,
                generation,
                resource_id,
                endpoint,
                id,
            } => {
                let tx = tx.clone();
                rt.spawn(async move {
                    let msg = fetch_form_record(
                        session,
                        client,
                        request_id,
                        generation,
                        resource_id,
                        endpoint,
                        id,
                    )
                    .await;
                    let _ = tx.send(msg);
                });
            }
            AppCommand::ListLocalDir {
                session,
                generation,
                path,
            } => {
                let tx = tx.clone();
                rt.spawn(async move {
                    let result =
                        tokio::task::spawn_blocking(move || crate::files_io::list_local_dir(&path))
                            .await;
                    let (dir, entries, error) = match result {
                        Ok(Ok((dir, entries))) => (dir, entries, None),
                        Ok(Err(err)) => (String::new(), Vec::new(), Some(err)),
                        Err(err) => (String::new(), Vec::new(), Some(err.to_string())),
                    };
                    let _ = tx.send(WorkerMsg::ListLocalDirResult {
                        session,
                        generation,
                        dir,
                        entries,
                        error,
                    });
                });
            }
            AppCommand::FetchSafeMode {
                session,
                generation,
            } => {
                let Some(client) = client else {
                    continue;
                };
                let tx = tx.clone();
                rt.spawn(async move {
                    let result = client.system("/safe-mode").await;
                    let (row, error) = match result {
                        Ok(row) => (Some(row), None),
                        Err(err) => {
                            send_if_session_event(&tx, session, generation, &err);
                            (None, Some(err.to_string()))
                        }
                    };
                    let _ = tx.send(WorkerMsg::SafeModeResult {
                        session,
                        generation,
                        row,
                        error,
                    });
                });
            }
        }
    }
}

fn dispatch_ssh(
    rt: &tokio::runtime::Runtime,
    tx: &mpsc::UnboundedSender<WorkerMsg>,
    ios: &mut HashMap<SessionId, SessionIo>,
    client: Option<Arc<Client>>,
    cmd: AppCommand,
) {
    match cmd {
        AppCommand::ProbeSshService {
            session,
            generation,
        } => {
            let Some(client) = client else {
                let _ = tx.send(WorkerMsg::SshService {
                    session,
                    generation,
                    port: mtui_ssh::DEFAULT_SSH_PORT,
                    disabled: true,
                    id: String::new(),
                    error: Some("Connect to the router first".into()),
                });
                return;
            };
            let tx = tx.clone();
            rt.spawn(async move {
                let msg = probe_ssh_service(session, generation, client).await;
                let _ = tx.send(msg);
            });
        }
        AppCommand::OpenSsh {
            session,
            generation,
            host,
            port,
            username,
            password,
            expected_fingerprint,
            cols,
            rows,
        } => {
            let (stdin_tx, stdin_rx) = mpsc::unbounded_channel();
            let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
            if let Some(io) = ios.get_mut(&session) {
                if let Some(prev) = io.ssh_cancel.take() {
                    let _ = prev.send(());
                }
                io.ssh_tx = Some(stdin_tx);
                io.ssh_cancel = Some(cancel_tx);
                io.ssh_gen = generation;
            }
            let tx = tx.clone();
            rt.spawn(async move {
                run_ssh_session(
                    session,
                    generation,
                    SshConnectOptions {
                        host,
                        port,
                        username,
                        password,
                        expected_fingerprint,
                        cols,
                        rows,
                    },
                    stdin_rx,
                    cancel_rx,
                    tx,
                )
                .await;
            });
        }
        AppCommand::SshWrite {
            session,
            generation,
            bytes,
        } => {
            if let Some(io) = ios.get(&session)
                && io.ssh_gen == generation
                && let Some(stdin) = &io.ssh_tx
            {
                let _ = stdin.send(SshInput::Data(bytes));
            }
        }
        AppCommand::SshResize {
            session,
            generation,
            cols,
            rows,
        } => {
            if let Some(io) = ios.get(&session)
                && io.ssh_gen == generation
                && let Some(stdin) = &io.ssh_tx
            {
                let _ = stdin.send(SshInput::Resize { cols, rows });
            }
        }
        _ => {}
    }
}

async fn probe_ssh_service(session: SessionId, generation: u64, client: Arc<Client>) -> WorkerMsg {
    match client.list("/ip/service").await {
        Ok(rows) => {
            let Some(row) = rows.iter().find(|row| {
                row.field("name")
                    .is_some_and(|name| name.eq_ignore_ascii_case("ssh"))
            }) else {
                return WorkerMsg::SshService {
                    session,
                    generation,
                    port: mtui_ssh::DEFAULT_SSH_PORT,
                    disabled: true,
                    id: String::new(),
                    error: Some("SSH service is not present on this router".into()),
                };
            };
            let port = row
                .field("port")
                .and_then(|value| value.parse().ok())
                .unwrap_or(mtui_ssh::DEFAULT_SSH_PORT);
            WorkerMsg::SshService {
                session,
                generation,
                port,
                disabled: truthy(row.field("disabled")),
                id: row.id.clone(),
                error: None,
            }
        }
        Err(err) => WorkerMsg::SshService {
            session,
            generation,
            port: mtui_ssh::DEFAULT_SSH_PORT,
            disabled: true,
            id: String::new(),
            error: Some(err.to_string()),
        },
    }
}

async fn run_ssh_session(
    session: SessionId,
    generation: u64,
    options: SshConnectOptions,
    stdin: mpsc::UnboundedReceiver<SshInput>,
    mut cancel: tokio::sync::oneshot::Receiver<()>,
    tx: mpsc::UnboundedSender<WorkerMsg>,
) {
    let pty = tokio::select! {
        biased;
        _ = &mut cancel => return,
        result = SshPty::connect(options) => match result {
            Ok(pty) => pty,
            Err(err) => {
                let _ = tx.send(WorkerMsg::SshClosed {
                    session,
                    generation,
                    error: Some(err.to_string()),
                });
                return;
            }
        }
    };
    let fingerprint = pty.fingerprint.clone();
    let stages_ms = pty.stages_ms.clone();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel();
    let runner = tokio::spawn(async move {
        pty.run(stdin, out_tx).await;
    });
    let _ = tx.send(WorkerMsg::SshReady {
        session,
        generation,
        fingerprint,
        stages_ms,
    });
    loop {
        tokio::select! {
            biased;
            _ = &mut cancel => break,
            bytes = out_rx.recv() => {
                let Some(bytes) = bytes else {
                    break;
                };
                if tx
                    .send(WorkerMsg::SshData {
                        session,
                        generation,
                        bytes,
                    })
                    .is_err()
                {
                    break;
                }
            }
        }
    }
    drop(out_rx);
    let _ = runner.await;
    let _ = tx.send(WorkerMsg::SshClosed {
        session,
        generation,
        error: None,
    });
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(text.to_string()))
        .map_err(|err| err.to_string())
}

async fn connect_worker(
    session: SessionId,
    url: String,
    username: String,
    password: String,
    pin: Option<String>,
    ca_pem: Option<Vec<u8>>,
    use_tls: bool,
) -> WorkerMsg {
    let had_pin = pin.is_some();
    let mut options = ClientOptions::new(url.clone(), username, password).with_tls(use_tls);
    if use_tls {
        if let Some(pin) = pin {
            options = options.with_certificate_pin(pin);
        }
        if let Some(pem) = ca_pem {
            options = options.with_ca_pem(pem);
        }
    }
    match Client::connect(options).await {
        Ok(client) => match client.system("/system/resource").await {
            Ok(router) => {
                let version = router.field("version").unwrap_or("");
                if let Err(message) = routeros_meets_minimum(version) {
                    return WorkerMsg::Connected {
                        session,
                        client: None,
                        router: None,
                        error: Some(message),
                        error_kind: Some(ErrorKind::Api),
                    };
                }
                WorkerMsg::Connected {
                    session,
                    client: Some(Arc::new(client)),
                    router: Some(router),
                    error: None,
                    error_kind: None,
                }
            }
            Err(err) => tls_or_connect_error(session, url, had_pin, err).await,
        },
        Err(err) => tls_or_connect_error(session, url, had_pin, err).await,
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

#[allow(clippy::too_many_arguments)]
async fn fetch_resource(
    session: SessionId,
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
            session,
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
                session,
                request_id,
                generation,
                resource_id: resource_id.clone(),
                rows,
                error: None,
            });
        }
        Err(err) => {
            send_if_session_event(&tx, session, generation, &err);
            let _ = tx.send(WorkerMsg::ResourceResult {
                session,
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
                            session,
                            generation,
                            resource_id: resource_id.clone(),
                            row,
                        });
                    }
                    Ok(None) => {
                        let _ = tx.send(WorkerMsg::SessionLost {
                            session,
                            generation,
                            reason: "connection closed".into(),
                        });
                        return;
                    }
                    Err(err) => {
                        send_if_session_event(&tx, session, generation, &err);
                        return;
                    }
                }
            }
        }
    }
}

fn is_auth_failure(err: &mtui_routeros::Error) -> bool {
    err.kind() == ErrorKind::Auth || matches!(err.status(), Some(401 | 403))
}

fn send_if_auth(
    tx: &mpsc::UnboundedSender<WorkerMsg>,
    session: SessionId,
    err: &mtui_routeros::Error,
) {
    if is_auth_failure(err) {
        let _ = tx.send(WorkerMsg::AuthRequired {
            session,
            message: crate::app::classify_connect_error(ErrorKind::Auth, err.message()),
        });
    }
}

fn send_if_session_event(
    tx: &mpsc::UnboundedSender<WorkerMsg>,
    session: SessionId,
    generation: u64,
    err: &mtui_routeros::Error,
) {
    send_if_auth(tx, session, err);
    if err.is_link_loss() {
        let _ = tx.send(WorkerMsg::SessionLost {
            session,
            generation,
            reason: err.message().to_string(),
        });
    }
}

async fn probe_menu_paths(session: SessionId, client: Arc<Client>, generation: u64) -> WorkerMsg {
    match crate::menu_paths::probe_missing_resource_ids(&client).await {
        Ok(missing_ids) => WorkerMsg::MenuPathsResult {
            session,
            generation,
            missing_ids,
            error: None,
        },
        Err(err) => {
            if err.is_link_loss() {
                return WorkerMsg::SessionLost {
                    session,
                    generation,
                    reason: err.message().to_string(),
                };
            }
            WorkerMsg::MenuPathsResult {
                session,
                generation,
                missing_ids: HashSet::new(),
                error: Some(err.to_string()),
            }
        }
    }
}

async fn fetch_access(
    session: SessionId,
    client: Arc<Client>,
    request_id: u64,
    generation: u64,
) -> WorkerMsg {
    let _ = request_id;
    let (users, groups) = tokio::join!(client.list("/user"), client.list("/user/group"),);
    match (users, groups) {
        (Ok(users), Ok(groups)) => WorkerMsg::AccessResult {
            session,
            generation,
            users,
            groups,
            error: None,
        },
        (Err(err), _) | (_, Err(err)) => {
            if err.is_link_loss() {
                return WorkerMsg::SessionLost {
                    session,
                    generation,
                    reason: err.message().to_string(),
                };
            }
            WorkerMsg::AccessResult {
                session,
                generation,
                users: Vec::new(),
                groups: Vec::new(),
                error: Some(err.to_string()),
            }
        }
    }
}

async fn tls_or_connect_error(
    session: SessionId,
    url: String,
    had_pin: bool,
    err: mtui_routeros::Error,
) -> WorkerMsg {
    if !had_pin && err.kind() == ErrorKind::Tls {
        return match probe_certificate(&url).await {
            Ok(fingerprint) => WorkerMsg::ProbeResult {
                session,
                fingerprint: Some(fingerprint),
                error: None,
            },
            Err(probe_err) => WorkerMsg::ProbeResult {
                session,
                fingerprint: None,
                error: Some(probe_err.to_string()),
            },
        };
    }
    WorkerMsg::Connected {
        session,
        client: None,
        router: None,
        error: Some(err.to_string()),
        error_kind: Some(err.kind()),
    }
}

async fn fetch_lookup(
    session: SessionId,
    client: Option<Arc<Client>>,
    request_id: u64,
    generation: u64,
    resource_id: String,
    value_key: String,
) -> WorkerMsg {
    let Some(spec) = mtui_core::resource_by_id(&resource_id) else {
        return WorkerMsg::LookupResult {
            session,
            request_id,
            generation,
            options: Vec::new(),
            error: Some("unknown resource".into()),
        };
    };
    let Some(client) = client else {
        return WorkerMsg::LookupResult {
            session,
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
            session,
            request_id,
            generation,
            options: lookup_options(&rows, &value_key),
            error: None,
        },
        Err(err) => WorkerMsg::LookupResult {
            session,
            request_id,
            generation,
            options: Vec::new(),
            error: Some(err.to_string()),
        },
    }
}

async fn fetch_form_record(
    session: SessionId,
    client: Option<Arc<Client>>,
    request_id: u64,
    generation: u64,
    resource_id: String,
    endpoint: String,
    id: String,
) -> WorkerMsg {
    let Some(client) = client else {
        return WorkerMsg::FormRecordResult {
            session,
            request_id,
            generation,
            resource_id,
            id,
            fields: None,
            error: Some("not connected".into()),
        };
    };
    match client.get(&endpoint, &id).await {
        Ok(row) => WorkerMsg::FormRecordResult {
            session,
            request_id,
            generation,
            resource_id,
            id,
            fields: Some(row.fields),
            error: None,
        },
        Err(err) => WorkerMsg::FormRecordResult {
            session,
            request_id,
            generation,
            resource_id,
            id,
            fields: None,
            error: Some(err.to_string()),
        },
    }
}

fn lookup_options(
    rows: &[mtui_routeros::Resource],
    value_key: &str,
) -> Vec<mtui_core::LookupOption> {
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
        if out
            .iter()
            .any(|item: &mtui_core::LookupOption| item.value == value)
        {
            continue;
        }
        out.push(mtui_core::LookupOption::from_fields(value, &row.fields));
    }
    mtui_core::sort_lookup_options(&mut out);
    out
}

async fn fetch_header(
    session: SessionId,
    client: Arc<Client>,
    request_id: u64,
    generation: u64,
    tx: mpsc::UnboundedSender<WorkerMsg>,
    view_rx: &mut watch::Receiver<u64>,
    gate: &StreamGate,
) {
    let (sys, interfaces) =
        tokio::join!(client.system("/system/resource"), client.list("/interface"),);

    let (system, system_error) = match sys {
        Ok(record) => (Some(record), None),
        Err(err) => {
            send_if_session_event(&tx, session, generation, &err);
            (None, Some(err.to_string()))
        }
    };
    let (interfaces, interface_error) = match interfaces {
        Ok(rows) => (rows, None),
        Err(err) => {
            send_if_session_event(&tx, session, generation, &err);
            (Vec::new(), Some(err.to_string()))
        }
    };
    let wan_name = select_wan_interface(&interfaces)
        .ok()
        .and_then(|iface| iface.field("name").map(ToOwned::to_owned));
    let _ = tx.send(WorkerMsg::HeaderResult {
        session,
        request_id,
        generation,
        system,
        system_error,
        interfaces,
        interface_error,
    });
    if let Some(name) = wan_name {
        stream_wan(session, client, generation, name, tx, view_rx, gate).await;
    }
}

async fn fetch_dashboard(
    session: SessionId,
    client: Arc<Client>,
    request_id: u64,
    generation: u64,
    tx: mpsc::UnboundedSender<WorkerMsg>,
    view_rx: &mut watch::Receiver<u64>,
    gate: &StreamGate,
) {
    let (cpu, sys, interfaces, firewall) = tokio::join!(
        client.list("/system/resource/cpu"),
        client.system("/system/resource"),
        client.list("/interface"),
        client.list("/ip/firewall/filter"),
    );

    let (cpu, cpu_error) = match cpu {
        Ok(rows) => (rows, None),
        Err(err) => {
            send_if_session_event(&tx, session, generation, &err);
            (Vec::new(), Some(err.to_string()))
        }
    };
    let (system, system_error) = match sys {
        Ok(record) => (Some(record), None),
        Err(err) => {
            send_if_session_event(&tx, session, generation, &err);
            (None, Some(err.to_string()))
        }
    };
    let (interfaces, interface_error) = match interfaces {
        Ok(rows) => (rows, None),
        Err(err) => {
            send_if_session_event(&tx, session, generation, &err);
            (Vec::new(), Some(err.to_string()))
        }
    };
    let (firewall, firewall_error) = match firewall {
        Ok(rows) => (rows, None),
        Err(err) => {
            send_if_session_event(&tx, session, generation, &err);
            (Vec::new(), Some(err.to_string()))
        }
    };
    let wan_name = select_wan_interface(&interfaces)
        .ok()
        .and_then(|iface| iface.field("name").map(ToOwned::to_owned));
    let _ = tx.send(WorkerMsg::DashboardResult {
        session,
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
        stream_wan(session, client, generation, name, tx, view_rx, gate).await;
    }
}

async fn stream_wan(
    session: SessionId,
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
                            session,
                            generation,
                            interface: interface.clone(),
                            sample,
                        });
                    }
                    Ok(None) => {
                        let _ = tx.send(WorkerMsg::SessionLost {
                            session,
                            generation,
                            reason: "connection closed".into(),
                        });
                        return;
                    }
                    Err(err) => {
                        send_if_session_event(&tx, session, generation, &err);
                        return;
                    }
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
    session: SessionId,
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
    let mut stream = match client.stream_command("/tool", "torch", &fields).await {
        Ok(stream) => stream,
        Err(err) => {
            let _ = tx.send(WorkerMsg::TorchResult {
                session,
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
                            session,
                            generation,
                            rows: vec![row.display_row()],
                            error: None,
                            done: false,
                        });
                    }
                    Ok(None) => {
                        let _ = tx.send(WorkerMsg::TorchResult {
                            session,
                            generation,
                            rows: Vec::new(),
                            error: None,
                            done: true,
                        });
                        return;
                    }
                    Err(err) => {
                        send_if_session_event(&tx, session, generation, &err);
                        let _ = tx.send(WorkerMsg::TorchResult {
                            session,
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
    session: SessionId,
    client: std::sync::Arc<Client>,
    request_id: u64,
    generation: u64,
    id: String,
    name: String,
    local_path: String,
) -> WorkerMsg {
    match client.file_contents(&id, &name).await {
        Ok(contents) => WorkerMsg::RecordResult {
            session,
            request_id,
            generation,
            local_path,
            contents: Some(contents),
            error: None,
        },
        Err(err) => WorkerMsg::RecordResult {
            session,
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
    session: SessionId,
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
                session,
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
                            session,
                            generation,
                            rows: vec![row.display_row()],
                            error: None,
                            done: false,
                        });
                    }
                    Ok(None) => {
                        let _ = tx.send(WorkerMsg::PingTraceResult {
                            session,
                            generation,
                            rows: Vec::new(),
                            error: None,
                            done: true,
                        });
                        return;
                    }
                    Err(err) => {
                        send_if_session_event(&tx, session, generation, &err);
                        let _ = tx.send(WorkerMsg::PingTraceResult {
                            session,
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
    use super::{WORKER_MSGS_PER_FRAME, coalesce_ssh_data, lookup_options, take_worker_batch};
    use crate::event::WorkerMsg;
    use crate::session::SessionId;
    use mtui_routeros::Resource;
    use std::collections::HashMap;
    use tokio::sync::mpsc;

    #[test]
    fn worker_batch_stops_at_frame_cap() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        for i in 0..100_u32 {
            tx.send(WorkerMsg::ProbeResult {
                session: SessionId::raw(1),
                fingerprint: Some(i.to_string()),
                error: None,
            })
            .expect("send");
        }
        let first = rx.try_recv().expect("first");
        let batch = take_worker_batch(&mut rx, first);
        assert_eq!(batch.len(), WORKER_MSGS_PER_FRAME);
        assert!(rx.try_recv().is_ok());
    }

    #[test]
    fn adjacent_ssh_data_is_merged_in_order() {
        let session = SessionId::raw(1);
        let batch = vec![
            WorkerMsg::SshReady {
                session,
                generation: 3,
                fingerprint: String::new(),
                stages_ms: String::new(),
            },
            WorkerMsg::SshData {
                session,
                generation: 3,
                bytes: b"he".to_vec(),
            },
            WorkerMsg::SshData {
                session,
                generation: 3,
                bytes: b"llo".to_vec(),
            },
            WorkerMsg::SshClosed {
                session,
                generation: 3,
                error: None,
            },
        ];
        let merged = coalesce_ssh_data(batch);
        assert_eq!(merged.len(), 3);
        match &merged[1] {
            WorkerMsg::SshData { bytes, .. } => assert_eq!(bytes, b"hello"),
            _ => panic!("expected merged PTY bytes"),
        }
    }

    #[test]
    fn ssh_ready_and_banner_share_a_batch_without_listen_deltas() {
        let session = SessionId::raw(1);
        let (tx, mut rx) = mpsc::unbounded_channel();
        tx.send(WorkerMsg::SshReady {
            session,
            generation: 1,
            fingerprint: String::new(),
            stages_ms: String::new(),
        })
        .expect("send");
        tx.send(WorkerMsg::SshData {
            session,
            generation: 1,
            bytes: b"[admin@MikroTik] > ".to_vec(),
        })
        .expect("send");
        let first = rx.try_recv().expect("first");
        let batch = take_worker_batch(&mut rx, first);
        assert!(matches!(batch[0], WorkerMsg::SshReady { .. }));
        match &batch[1] {
            WorkerMsg::SshData { bytes, .. } => {
                assert_eq!(bytes.as_slice(), b"[admin@MikroTik] > ");
            }
            _ => panic!("expected PTY bytes after SshReady"),
        }
        assert_eq!(batch.len(), 2);
    }

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
        second.insert("disabled".into(), "true".into());
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
        let options = lookup_options(&rows, "name");
        assert_eq!(
            options
                .iter()
                .map(|option| option.value.as_str())
                .collect::<Vec<_>>(),
            ["ether1", "ether2"]
        );
        assert!(!options[0].disabled);
        assert!(options[1].disabled);
    }
}
