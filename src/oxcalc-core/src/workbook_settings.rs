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
