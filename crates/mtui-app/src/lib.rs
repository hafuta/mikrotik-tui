//! Application state machine: connects business logic to pure UI widgets.

mod app;
mod event;
mod keys;
mod render;
mod runtime;
mod telemetry;

pub use app::{App, AppCommand, Screen};
pub use runtime::run;
