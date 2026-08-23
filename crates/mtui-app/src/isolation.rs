//! Session isolation: commands and worker results never leak across tabs.

use mtui_routeros::Resource;

use crate::app::{App, AppCommand, Screen};
use crate::event::{AppEvent, WorkerMsg};
use crate::session::{LinkState, SessionId};

fn test_app() -> App {
    App::new(false).expect("app")
}

#[test]
fn new_session_does_not_copy_password_or_client() {
    let mut app = test_app();
    let a = app.test_session();
    app.login.password = "secret-a".into();
    app.pending_password = Some("pending-a".into());
    app.poll_generation = 7;
    let b = app.new_session().expect("second tab");
    assert_ne!(a, b);
    assert_eq!(app.session(a).expect("a").login.password, "secret-a");
    assert_eq!(
        app.session(a).expect("a").pending_password.as_deref(),
        Some("pending-a")
    );
    assert!(app.session(b).expect("b").login.password.is_empty());
    assert!(app.session(b).expect("b").pending_password.is_none());
    assert!(app.session(b).expect("b").client.is_none());
    assert_eq!(app.session(b).expect("b").poll_generation, 0);
    assert_eq!(app.session(a).expect("a").poll_generation, 7);
}

#[test]
fn stamp_uses_active_session() {
    let mut app = test_app();
    let a = app.test_session();
    let b = app.new_session().expect("second tab");
    assert_eq!(app.active, b);
    let cmds = app.stamp(vec![AppCommand::FetchHeader {
        session: SessionId::raw(0),
        request_id: 1,
        generation: 1,
    }]);
    assert!(matches!(
        cmds.as_slice(),
        [AppCommand::FetchHeader { session, .. }] if *session == b
    ));
    app.active = a;
    let cmds = app.stamp(vec![AppCommand::FetchDashboard {
        session: SessionId::raw(0),
        request_id: 1,
        generation: 1,
    }]);
    assert!(matches!(
        cmds.as_slice(),
        [AppCommand::FetchDashboard { session, .. }] if *session == a
    ));
}

#[test]
fn stale_worker_for_a_does_not_touch_b() {
    let mut app = test_app();
    let a = app.test_session();
    app.screen = Screen::Main;
    app.current_resource = "interfaces".into();
    app.poll_generation = 3;
    app.status = "A ready".into();
    let b = app.new_session().expect("second tab");
    app.screen = Screen::Main;
    app.current_resource = "interfaces".into();
    app.poll_generation = 1;
    app.status = "B ready".into();

    let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
        session: a,
        request_id: 1,
        generation: 2,
        resource_id: "interfaces".into(),
        rows: vec![Resource::default()],
        error: None,
    }));

    assert_eq!(app.session(a).expect("a").status, "A ready");
    assert_eq!(app.session(b).expect("b").status, "B ready");
    assert_eq!(app.session(a).expect("a").poll_generation, 3);
}

#[test]
fn worker_for_closed_session_does_not_set_b_client() {
    let mut app = test_app();
    let a = app.test_session();
    let b = app.new_session().expect("second tab");
    app.close_session(a);
    assert_eq!(app.sessions.len(), 1);
    assert_eq!(app.active, b);
    assert!(app.session(a).is_none());

    let _ = app.update(AppEvent::Worker(WorkerMsg::Connected {
        session: a,
        client: None,
        router: None,
        error: None,
        error_kind: None,
    }));

    assert!(app.session(b).expect("b").client.is_none());
    assert_eq!(app.session(b).expect("b").screen, Screen::Login);
}

#[test]
fn never_leave_zero_tabs_and_cap_at_eight() {
    let mut app = test_app();
    let first = app.test_session();
    app.close_session(first);
    assert_eq!(app.sessions.len(), 1);

    for _ in 0..7 {
        assert!(app.new_session().is_some());
    }
    assert_eq!(app.sessions.len(), 8);
    assert!(app.new_session().is_none());
    assert_eq!(app.sessions.len(), 8);
}

#[test]
fn close_session_returns_addressed_command() {
    let mut app = test_app();
    let a = app.test_session();
    let b = app.new_session().expect("second tab");
    app.active = a;
    let cmds = app.stamp(vec![AppCommand::CloseSession { session: a }]);
    assert!(matches!(
        cmds.as_slice(),
        [AppCommand::CloseSession { session }] if *session == a
    ));
    assert_eq!(b.get(), 2);
}

#[test]
fn drop_on_a_does_not_mark_b_down() {
    let mut app = test_app();
    let a = app.test_session();
    app.screen = Screen::Main;
    app.link = LinkState::Live;
    app.poll_generation = 4;
    app.status = "A live".into();
    let b = app.new_session().expect("second tab");
    app.screen = Screen::Main;
    app.link = LinkState::Live;
    app.poll_generation = 2;
    app.status = "B live".into();

    let _ = app.update(AppEvent::Worker(WorkerMsg::SessionLost {
        session: a,
        generation: 4,
        reason: "connection closed".into(),
    }));

    assert_eq!(app.session(a).expect("a").link, LinkState::Dropped);
    assert!(!app.session(a).expect("a").session_ready());
    assert_eq!(app.session(b).expect("b").link, LinkState::Live);
    assert!(app.session(b).expect("b").session_ready());
    assert_eq!(app.session(b).expect("b").status, "B live");
}
