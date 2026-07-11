//! Wall-clock phase timers for the grid recalc lane (roadmap item M-2).
//!
//! Observation only, by doctrine (CORE_ENGINE_GRID_PERF_REGISTER.md §1.3):
//! wall-clock readings must NEVER enter a counter gate, a register assertion,
//! or a checked-in JSON artifact. Timings ride the recalc report structs
//! purely for interactive inspection (`summary_line`) and are deliberately
//! invisible to report equality so counter-gated tests stay deterministic.
//!
//! wasm32-safe: mirrors the `LocalTreeCalcInstant` pattern in `treecalc.rs` —
//! a real `std::time::Instant` off-wasm, a zero duration on wasm32.

use std::collections::BTreeMap;
#[cfg(target_arch = "wasm32")]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

use serde::{Serialize, Serializer};

/// Stable-keyed recalc phases for the grid lane. `stable_id` values are the
/// serialization contract; variant names may be refactored, ids may not.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GridRecalcPhaseKey {
    SeedClosure,
    Schedule,
    StructuralInstall,
    FastPathEvaluate,
    OxfmlEvaluate,
    SpillRepair,
    Publish,
    Total,
    Other(String),
}

impl GridRecalcPhaseKey {
    #[must_use]
    pub fn stable_id(&self) -> &str {
        match self {
            Self::SeedClosure => "seed_closure",
            Self::Schedule => "schedule",
            Self::StructuralInstall => "structural_install",
            Self::FastPathEvaluate => "fast_path_evaluate",
            Self::OxfmlEvaluate => "oxfml_evaluate",
            Self::SpillRepair => "spill_repair",
            Self::Publish => "publish",
            Self::Total => "total",
            Self::Other(value) => value.as_str(),
        }
    }
}

impl Serialize for GridRecalcPhaseKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.stable_id())
    }
}

/// wasm-safe instant: real `Instant` off-wasm, zero elapsed on wasm32.
#[derive(Debug, Clone, Copy)]
struct GridPhaseInstant {
    #[cfg(not(target_arch = "wasm32"))]
    instant: Instant,
}

impl GridPhaseInstant {
    fn now() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            instant: Instant::now(),
        }
    }

    fn elapsed(&self) -> Duration {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.instant.elapsed()
        }

        #[cfg(target_arch = "wasm32")]
        {
            Duration::ZERO
        }
    }
}

/// Opaque token marking the start of one phase measurement; obtained from
/// [`GridRecalcPhaseTimer::phase_start`] and spent in
/// [`GridRecalcPhaseTimer::accumulate`].
#[derive(Debug, Clone, Copy)]
pub struct GridPhaseStart(GridPhaseInstant);

/// Elapsed wall-clock micros per recalc phase.
///
/// Equality is deliberately always-true (see the `PartialEq` impl below):
/// these timings ride report structs that tests compare with `assert_eq!`,
/// and two semantically identical runs must stay equal regardless of the
/// wall-clock they happened to consume.
#[derive(Debug, Clone, Default)]
pub struct GridRecalcPhaseTimings(pub BTreeMap<GridRecalcPhaseKey, u128>);

/// Always-true equality: timings are observation-only and must never make
/// two otherwise-identical recalc reports compare unequal (reports derive
/// `PartialEq`/`Eq` and are asserted with `assert_eq!` in counter-gated
/// tests, and the warm-no-op cache compares baseline reports).
impl PartialEq for GridRecalcPhaseTimings {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for GridRecalcPhaseTimings {}

impl GridRecalcPhaseTimings {
    /// Const-context constructor (used by `GridOptimizedRecalcReport::empty`).
    #[must_use]
    pub const fn empty() -> Self {
        Self(BTreeMap::new())
    }

    /// Compact one-line display, e.g.
    /// `seed_closure=12us oxfml_evaluate=9000us total=12345us`, in key order.
    #[must_use]
    pub fn summary_line(&self) -> String {
        self.0
            .iter()
            .map(|(key, micros)| format!("{}={micros}us", key.stable_id()))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Accumulating wall-clock timer for one recalc pass. Per-formula phases are
/// summed across the evaluation loop; `finish` records `Total` from
/// construction time.
#[derive(Debug)]
pub struct GridRecalcPhaseTimer {
    started_at: GridPhaseInstant,
    timings_micros: BTreeMap<GridRecalcPhaseKey, u128>,
}

impl GridRecalcPhaseTimer {
    #[must_use]
    pub fn start() -> Self {
        Self {
            started_at: GridPhaseInstant::now(),
            timings_micros: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn phase_start(&self) -> GridPhaseStart {
        GridPhaseStart(GridPhaseInstant::now())
    }

    /// Adds the elapsed micros since `start` to `key` (accumulating: the
    /// same phase may be entered once per formula in the worklist loop).
    pub fn accumulate(&mut self, key: GridRecalcPhaseKey, start: GridPhaseStart) {
        self.add_micros(key, start.0.elapsed().as_micros());
    }

    fn add_micros(&mut self, key: GridRecalcPhaseKey, micros: u128) {
        *self.timings_micros.entry(key).or_insert(0) += micros;
    }

    #[must_use]
    pub fn finish(mut self) -> GridRecalcPhaseTimings {
        let total = self.started_at.elapsed().as_micros();
        self.timings_micros.insert(GridRecalcPhaseKey::Total, total);
        GridRecalcPhaseTimings(self.timings_micros)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_recalc_phase_keys_have_stable_snake_case_ids() {
        let expected = [
            (GridRecalcPhaseKey::SeedClosure, "seed_closure"),
            (GridRecalcPhaseKey::Schedule, "schedule"),
            (GridRecalcPhaseKey::StructuralInstall, "structural_install"),
            (GridRecalcPhaseKey::FastPathEvaluate, "fast_path_evaluate"),
            (GridRecalcPhaseKey::OxfmlEvaluate, "oxfml_evaluate"),
            (GridRecalcPhaseKey::SpillRepair, "spill_repair"),
            (GridRecalcPhaseKey::Publish, "publish"),
            (GridRecalcPhaseKey::Total, "total"),
            (
                GridRecalcPhaseKey::Other("custom_phase".to_string()),
                "custom_phase",
            ),
        ];
        for (key, id) in expected {
            assert_eq!(key.stable_id(), id);
            assert_eq!(
                serde_json::to_string(&key).expect("phase key serializes"),
                format!("\"{id}\"")
            );
        }
    }

    #[test]
    fn phase_timings_equality_ignores_measured_values() {
        let mut measured = GridRecalcPhaseTimings::default();
        measured.0.insert(GridRecalcPhaseKey::Total, 12_345);
        measured.0.insert(GridRecalcPhaseKey::OxfmlEvaluate, 9_000);
        let empty = GridRecalcPhaseTimings::empty();
        assert_eq!(measured, empty);
        assert_eq!(empty, GridRecalcPhaseTimings::default());
        assert_eq!(
            measured.summary_line(),
            "oxfml_evaluate=9000us total=12345us"
        );
    }

    #[test]
    fn accumulate_sums_micros_per_key_and_finish_records_total() {
        let mut timer = GridRecalcPhaseTimer::start();
        timer.add_micros(GridRecalcPhaseKey::OxfmlEvaluate, 5);
        timer.add_micros(GridRecalcPhaseKey::OxfmlEvaluate, 7);
        timer.add_micros(GridRecalcPhaseKey::Schedule, 3);
        // The public token path must land in the same accumulating slot.
        let started = timer.phase_start();
        timer.accumulate(GridRecalcPhaseKey::Schedule, started);
        let timings = timer.finish();
        assert_eq!(timings.0.get(&GridRecalcPhaseKey::OxfmlEvaluate), Some(&12));
        assert!(timings.0.get(&GridRecalcPhaseKey::Schedule).copied() >= Some(3));
        assert!(timings.0.contains_key(&GridRecalcPhaseKey::Total));
    }
}
