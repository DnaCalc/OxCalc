#![forbid(unsafe_code)]

//! Transitional facade for the relocated strict-excel-grid reference engine.
//!
//! The implementation now lives in [`crate::grid::reference_engine`]; this
//! re-export preserves the historical `excel_grid_reference::` paths until call
//! sites are repointed to `crate::grid::...` and the shim is removed.

pub use crate::grid::reference_engine::*;
