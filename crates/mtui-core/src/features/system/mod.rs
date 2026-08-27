//! System feature boundary: catalog, forms, guides, and tests.
//!
//! Covers `RouterOS` `/system`, `/user`, `/disk`, `/certificate`, `/snmp`,
//! `/port`, `/special-login`, and `/log`.
#![allow(dead_code)]

pub(crate) mod forms;
pub(crate) mod guides;
pub(crate) mod resources;
pub(crate) mod rules;

#[cfg(test)]
mod tests;
