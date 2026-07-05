//! Workbook calc-settings: typed values, wire encoding, and change seeds.
//!
//! W062 R2.5 (D1 §5). Workbook calculation settings — date system, calc mode,
//! and iteration settings — live in a `#workbook-settings` meta-child of the
//! workbook root, one meta grandchild per setting whose value is an ordinary
//! [`NodeInputRecord`](crate::workspace_revision::NodeInputRecord) literal.
//! Because the storage is node inputs, revision-identity participation is
//! automatic: a settings change is a node-input edit that changes the
//! node-input snapshot id and therefore the workspace revision id (D1 §4/§5).
//!
//! This module owns only the *typed value model* and its *wire text encoding*.
//! The read/write accessors and change-seed emission live on the context
//! (`consumer.rs`); the meta-node storage layout (group symbol, per-setting
//! symbols) is documented alongside them. Seed emission is contract-only: no
//! recalc mechanics live here or in the accessor (D3/R4 owns those).

use serde::{Deserialize, Serialize};

/// Date system for serial-date interpretation. Excel's two epochs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub enum DateSystem {
    /// 1900 date system (Excel default): day 1 is 1900-01-01.
    #[default]
    Excel1900,
    /// 1904 date system (legacy Mac): day 0 is 1904-01-01.
    Excel1904,
}

impl DateSystem {
    /// Wire text for the setting node's literal input record.
    #[must_use]
    pub fn as_wire_text(self) -> &'static str {
        match self {
            Self::Excel1900 => "1900",
            Self::Excel1904 => "1904",
        }
    }

    /// Parse a stored wire text back to the typed value. Unknown text (e.g. a
    /// hand-corrupted snapshot) falls back to the default, keeping reads total.
    #[must_use]
    pub fn from_wire_text(text: &str) -> Self {
        match text {
            "1904" => Self::Excel1904,
            _ => Self::Excel1900,
        }
    }
}

/// Recalculation scheduling mode. A scheduling fact, never a value fact:
/// changing it never invalidates any computed value (D1 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub enum CalcMode {
    /// Automatic recalculation (Excel default).
    #[default]
    Automatic,
    /// Manual recalculation.
    Manual,
}

impl CalcMode {
    /// Wire text for the setting node's literal input record.
    #[must_use]
    pub fn as_wire_text(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Manual => "manual",
        }
    }

    /// Parse a stored wire text back to the typed value. Unknown text falls
    /// back to the default, keeping reads total.
    #[must_use]
    pub fn from_wire_text(text: &str) -> Self {
        match text {
            "manual" => Self::Manual,
            _ => Self::Automatic,
        }
    }
}

/// Provenance of a published grid-cell value: *how* the value now on the
/// published readout came to be there (W062 R5.6, D4 §6/§8). Distinct from the
/// value itself — two cells reading `21` are not interchangeable if one was
/// engine-computed this tick and the other is a file cache the engine has never
/// touched. The readout carries this so a host (and the save path, R5.7/R6) can
/// tell a fresh value from a stale-but-honest one, and the differential harness
/// can exclude pre-engine values by construction (contract C15).
///
/// Ordering (`Calculated` < `Stale` < `FileCached`) is arbitrary but total, so
/// provenance can key a `BTreeMap`; it carries no semantic weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PublishedValueProvenance {
    /// Genuinely evaluated by the engine on the recalc identified by `tick_id`
    /// (the [`WorkbookRecalcTick::tick_id`](crate::grid::machine::WorkbookRecalcTick) of
    /// the transaction that produced it). This is the only provenance the
    /// differential harness compares — an engine value is the engine's word.
    Calculated { tick_id: u64 },
    /// A value that *was* `Calculated` at `since_tick_id` but whose sheet has
    /// since accumulated authored edits that have not been drained through a
    /// recalc (Manual calc mode, or between an edit and the next
    /// `recalculate_workbook`). The value on the readout is the pre-edit value:
    /// honestly stale, explicitly marked, never silently presented as fresh.
    Stale { since_tick_id: u64 },
    /// A cached value read from a loaded file, never evaluated by this engine
    /// (D4 §8: the ingest seat). Renders instantly on load and — under Manual
    /// mode — indefinitely, without pretending the engine computed it; the first
    /// genuine evaluation replaces it with `Calculated`. **Invisible to the
    /// oracle/optimized differential by construction** (C15): a `FileCached`
    /// value is pre-engine, not an engine disagreement. The mint + plumbing land
    /// here (R5.6); ingest population lands in R6.
    FileCached,
}

impl PublishedValueProvenance {
    /// Whether this value is an engine computation the differential harness must
    /// compare. `FileCached` values are pre-engine and excluded by construction
    /// (contract C15); `Stale` values are prior-tick engine values retained
    /// verbatim across a suppressed edit and equally not a live disagreement.
    #[must_use]
    pub fn is_engine_calculated(self) -> bool {
        matches!(self, Self::Calculated { .. })
    }
}

/// Iterative-calculation settings for cycle groups.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IterationSettings {
    /// Whether iterative calculation is enabled. Default `false` (Excel default).
    pub enabled: bool,
    /// Maximum iterations per recalc. Default `100` (Excel default).
    pub max_iterations: u32,
    /// Convergence threshold. Default `0.001` (Excel default).
    ///
    /// Note: change detection compares with `PartialEq`, so a NaN here always
    /// registers as "changed" and its wire text (`"NaN"`) reads back as the
    /// default (failed parse ⇒ default). Excel never supplies NaN; if
    /// iteration inputs ever become externally sourced, sanitize upstream.
    pub max_change: f64,
}

impl Default for IterationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            max_iterations: 100,
            max_change: 0.001,
        }
    }
}

/// Typed workbook calculation settings. Read through
/// `OxCalcTreeContext::workbook_calc_settings` (defaults on absence) and
/// written through `set_workbook_calc_settings`.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct WorkbookCalcSettings {
    pub date_system: DateSystem,
    pub calc_mode: CalcMode,
    pub iteration: IterationSettings,
}

/// A typed workbook-setting change, delivered as a seed with old and new
/// values (D1 §5, C4). Carried on its own channel so old/new values survive
/// (the value-invalidation [`InvalidationSeed`](crate::dependency::InvalidationSeed)
/// carries only `{node_id, reason}`). `CalcMode` changes emit *only* this seed
/// and no value invalidation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkbookSettingChanged {
    DateSystem {
        old: DateSystem,
        new: DateSystem,
    },
    CalcMode {
        old: CalcMode,
        new: CalcMode,
    },
    Iteration {
        old: IterationSettings,
        new: IterationSettings,
    },
}
