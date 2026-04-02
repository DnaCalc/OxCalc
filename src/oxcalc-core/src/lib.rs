#![forbid(unsafe_code)]

//! `OxCalc` core engine crate.
//!
//! This crate is the Rust-first home for `OxCalc` structural state,
//! coordinator/publication state, and recalc/overlay state.
//! The shape is intentionally semantic-first rather than a direct transfer
//! of any older non-Rust object model.

pub mod coordinator;
pub mod consumer;
pub mod dependency;
pub mod formula;
pub mod recalc;
pub mod structural;
pub mod treecalc;
pub mod treecalc_fixture;
pub mod treecalc_runner;
pub mod upstream_host;
pub mod upstream_host_fixture;
