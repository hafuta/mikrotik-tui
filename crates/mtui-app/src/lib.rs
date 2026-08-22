//! Application state machine: connects business logic to pure UI widgets.

mod app;
mod event;
mod files_io;
mod keys;
mod render;
mod runtime;
mod telemetry;
mod write;

pub use app::{App, AppCommand, Screen};
pub use runtime::run;
