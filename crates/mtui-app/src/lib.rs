//! Application state machine: connects business logic to pure UI widgets.

mod app;
mod demo;
mod event;
mod files_io;
mod health;
mod help;
mod keys;
mod menu_paths;
mod render;
mod runtime;
mod safe_mode;
mod session;
mod telemetry;
mod write;

pub use app::{App, AppCommand, Screen};
pub use runtime::run;
pub use session::{MAX_SESSIONS, Session, SessionId};

#[cfg(test)]
mod isolation;
