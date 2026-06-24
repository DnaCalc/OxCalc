#![forbid(unsafe_code)]

//! OxCalc's `strict-excel-grid` reference system.
//!
//! This module is the home of the grid reference *permanent pair*: a
//! simple-correct reference engine (GridCalc-Ref) and the compact optimized
//! engine (GridOptimizedSheet), the profile-pure shared core they both build
//! on, and the differential harness that proves their observational
//! equivalence. The doctrine these implementations refine is specified in
//! `docs/spec/core-engine/CORE_ENGINE_GRID_MODEL.md`,
//! `CORE_ENGINE_GRID_REFERENCE_MACHINE.md`, `CORE_ENGINE_GRID_PERF_REGISTER.md`,
//! and `CORE_ENGINE_GRID_REFINEMENT_AND_EQUIVALENCE.md`.
//!
//! The submodules are layered:
//! - **shared core** (`coords`, ...): profile-pure coordinate and reference
//!   types plus parsing/geometry, consumed by both engines as a stateless
//!   library so the differential cross-check keeps its teeth.

pub mod ast;
pub mod authored;
pub mod coords;
pub mod error;
pub mod geometry;
pub mod machine;
pub mod reference_engine;
pub mod runner;
pub mod scale;
