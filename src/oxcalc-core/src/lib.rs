#![forbid(unsafe_code)]

//! `OxCalc` core engine crate.
//!
//! This crate is the Rust-first home for `OxCalc` structural state,
//! coordinator/publication state, and recalc/overlay state.
//! The shape is intentionally semantic-first rather than a direct transfer
//! of any older non-Rust object model.

pub mod consumer;
pub mod coordinator;
pub mod correctness_floor;
pub mod dependency;
pub mod error_algebra;
pub mod formula;
pub mod numerical_reduction;
pub mod oxfml_session;
pub mod recalc;
pub mod recalc_wave;
pub mod repository;
pub mod rich_value_capability;
pub mod stream_semantics;
pub mod structural;
pub mod treecalc;
pub mod treecalc_fixture;
pub mod treecalc_runner;
pub mod treecalc_scale;
pub mod upstream_host;
pub mod upstream_host_fixture;
pub mod upstream_host_runner;
pub mod value_cache;
