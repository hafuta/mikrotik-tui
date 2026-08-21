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
                _ = tokio::time::sleep(Duration::from_millis(16)) => {
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
        }
    }
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
