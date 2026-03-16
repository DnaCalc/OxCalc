#![forbid(unsafe_code)]

//! `OxCalc` core engine crate.
//!
//! This crate is the Rust-first home for `OxCalc` structural state,
//! coordinator/publication state, and recalc/overlay state.
//! The shape is intentionally semantic-first rather than a direct transfer
//! of the current .NET object model.

pub mod coordinator;
pub mod recalc;
pub mod structural;
