//! The grid permanent-pair differential harness: GridEngineMode (run the
//! reference engine, the optimized engine, or both), the per-engine readout
//! and run reports, and compare_grid_engine_readouts which asserts the two
//! engines agree on value readout and committed effects. Internal to the
//! machine; shares the machine's types via `use super::*`.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridEngineMode {
    Reference,
    Optimized,
    Both,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridEngineCellReadout {
    pub address: ExcelGridCellAddress,
    pub computed: CalcValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GridEngineRecalcReport {
    Reference(GridCalcRefRecalcReport),
    Optimized(GridOptimizedRecalcReport),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridEngineRunReport {
    pub mode: GridEngineMode,
    pub recalc: GridEngineRecalcReport,
    pub readout: Vec<GridEngineCellReadout>,
    pub warm_noop: Option<GridEngineWarmNoOpReport>,
    /// The spill facts this recalc produced (anchor, extent, blocked), in anchor
    /// order. Unlike table/merged overlays (committed document state on the
    /// sheet), spills are a calc result, so they are surfaced from the run rather
    /// than from committed sheet state.
    pub spill_facts: Vec<GridSpillFact>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridEngineWarmNoOpReport {
    pub recalc: GridOptimizedWarmNoOpReport,
    pub readout: Vec<GridEngineCellReadout>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridDifferentialMismatch {
    pub address: ExcelGridCellAddress,
    pub reference: CalcValue,
    pub optimized: CalcValue,
}

/// A divergence between the two engines on overlay-driven spill blockage: a spill
/// anchor whose computed fact (extent / blocked) differs between engines, or is
/// present in only one. The committed table/merged/feature overlays are copied
/// identically to the reference, so the spill outcome (which spills the overlay
/// set blocks) is the only overlay-blockage dimension that can diverge - this is
/// the permanent-pair overlay invariant: the optimized overlay-set blockage probe
/// must agree with the reference brute-force blockage.
#[derive(Debug, Clone, PartialEq)]
pub struct GridOverlayBlockageMismatch {
    pub anchor: ExcelGridCellAddress,
    pub reference: Option<GridSpillFact>,
    pub optimized: Option<GridSpillFact>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridDifferentialRunReport {
    pub mode: GridEngineMode,
    pub reference: Option<GridEngineRunReport>,
    pub optimized: Option<GridEngineRunReport>,
    pub mismatches: Vec<GridDifferentialMismatch>,
    /// Overlay-blockage divergences (empty unless mode is `Both`); zero is the
    /// invariant.
    pub overlay_blockage_mismatches: Vec<GridOverlayBlockageMismatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridDirtyRecalcMismatch {
    pub address: ExcelGridCellAddress,
    pub dirty: CalcValue,
    pub mark_all: CalcValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridDirtyRecalcSpillFactMismatch {
    pub anchor: ExcelGridCellAddress,
    pub dirty: Option<GridSpillFact>,
    pub mark_all: Option<GridSpillFact>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridDynamicDefinedNameRuntimeSnapshot {
    pub keys: BTreeSet<String>,
    pub extents: BTreeMap<String, GridRect>,
    pub dependencies: GridDynamicDefinedNameDependencyState,
    pub volatile_names: BTreeSet<String>,
    pub external_pending_names: BTreeSet<String>,
}

impl GridDynamicDefinedNameRuntimeSnapshot {
    #[must_use]
    pub fn new(
        keys: BTreeSet<String>,
        extents: BTreeMap<String, GridRect>,
        dependencies: GridDynamicDefinedNameDependencyState,
        volatile_names: BTreeSet<String>,
        external_pending_names: BTreeSet<String>,
    ) -> Self {
        Self {
            keys,
            extents,
            dependencies,
            volatile_names,
            external_pending_names,
        }
    }
}

/// A divergence in the registry-effect comparison: after applying both runs'
/// `external_subscription_updates` to two independently seeded clones of a
/// `GridExternalAvailabilityTopicRegistry`, the resulting `roots_by_topic`
/// maps disagree for this topic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridRegistryEffectMismatch {
    pub topic_id: String,
    pub dirty_roots: BTreeSet<GridExternalAvailabilityRoot>,
    pub mark_all_roots: BTreeSet<GridExternalAvailabilityRoot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridDirtyRecalcDifferentialRunReport {
    pub mode: GridEngineMode,
    pub dirty_recalc: GridEngineRecalcReport,
    pub mark_all_recalc: GridEngineRecalcReport,
    pub dirty_readout: Vec<GridEngineCellReadout>,
    pub mark_all_readout: Vec<GridEngineCellReadout>,
    pub mismatches: Vec<GridDirtyRecalcMismatch>,
    pub spill_fact_mismatches: Vec<GridDirtyRecalcSpillFactMismatch>,
    pub dependency_graphs_equal: bool,
    pub dirty_structural_dependency_edges: usize,
    pub dirty_overlay_dependency_edges: usize,
    pub mark_all_structural_dependency_edges: usize,
    pub mark_all_overlay_dependency_edges: usize,
    pub dynamic_defined_name_state_equal: bool,
    pub dirty_dynamic_defined_names: GridDynamicDefinedNameRuntimeSnapshot,
    pub mark_all_dynamic_defined_names: GridDynamicDefinedNameRuntimeSnapshot,
    /// Spill-epoch-ledger snapshot equality between the two runs. This is a
    /// stricter check than `spill_fact_mismatches`: the ledger also carries
    /// the value-fingerprint/epoch bookkeeping used to gate spill-repair
    /// convergence, not just the anchor/extent/blocked triple.
    pub spill_epoch_ledger_equal: bool,
    /// Registry-effect comparison: both runs' `external_subscription_updates`
    /// applied to two clones of a caller-seeded
    /// `GridExternalAvailabilityTopicRegistry`, then compared by
    /// `roots_by_topic()`. Empty unless the caller opts in by passing
    /// `Some(seed_registry)` as `registry_effect_seed` to
    /// `build_grid_dirty_recalc_differential_report`.
    pub registry_effect_mismatches: Vec<GridRegistryEffectMismatch>,
    pub registry_effect_equal: bool,
}

impl GridDirtyRecalcDifferentialRunReport {
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.mismatches.is_empty()
            && self.spill_fact_mismatches.is_empty()
            && self.dependency_graphs_equal
            && self.dynamic_defined_name_state_equal
            && self.spill_epoch_ledger_equal
            && self.registry_effect_equal
    }
}

pub(super) fn compare_grid_engine_readouts(
    reference: &[GridEngineCellReadout],
    optimized: &[GridEngineCellReadout],
) -> Vec<GridDifferentialMismatch> {
    reference
        .iter()
        .zip(optimized.iter())
        .filter_map(|(reference, optimized)| {
            (reference.computed != optimized.computed).then(|| GridDifferentialMismatch {
                address: reference.address.clone(),
                reference: reference.computed.clone(),
                optimized: optimized.computed.clone(),
            })
        })
        .collect()
}

pub(super) fn compare_grid_dirty_recalc_readouts(
    dirty: &[GridEngineCellReadout],
    mark_all: &[GridEngineCellReadout],
) -> Vec<GridDirtyRecalcMismatch> {
    dirty
        .iter()
        .zip(mark_all.iter())
        .filter_map(|(dirty, mark_all)| {
            (dirty.computed != mark_all.computed).then(|| GridDirtyRecalcMismatch {
                address: dirty.address.clone(),
                dirty: dirty.computed.clone(),
                mark_all: mark_all.computed.clone(),
            })
        })
        .collect()
}

/// Compare the two engines' computed spill facts by anchor. A divergence (a spill
/// present in only one engine, or present in both with a different extent/blocked
/// flag) is an overlay-blockage mismatch - the optimized overlay-set probe and the
/// reference brute-force blockage disagreed.
pub(super) fn compare_grid_overlay_blockage(
    reference: &[GridSpillFact],
    optimized: &[GridSpillFact],
) -> Vec<GridOverlayBlockageMismatch> {
    let reference_by_anchor: BTreeMap<&ExcelGridCellAddress, &GridSpillFact> =
        reference.iter().map(|fact| (&fact.anchor, fact)).collect();
    let optimized_by_anchor: BTreeMap<&ExcelGridCellAddress, &GridSpillFact> =
        optimized.iter().map(|fact| (&fact.anchor, fact)).collect();
    let mut anchors: BTreeSet<&ExcelGridCellAddress> = BTreeSet::new();
    anchors.extend(reference_by_anchor.keys().copied());
    anchors.extend(optimized_by_anchor.keys().copied());
    anchors
        .into_iter()
        .filter_map(|anchor| {
            let reference_fact = reference_by_anchor.get(anchor).copied();
            let optimized_fact = optimized_by_anchor.get(anchor).copied();
            (reference_fact != optimized_fact).then(|| GridOverlayBlockageMismatch {
                anchor: anchor.clone(),
                reference: reference_fact.cloned(),
                optimized: optimized_fact.cloned(),
            })
        })
        .collect()
}

pub(super) fn compare_grid_dirty_recalc_spill_facts(
    dirty: &[GridSpillFact],
    mark_all: &[GridSpillFact],
) -> Vec<GridDirtyRecalcSpillFactMismatch> {
    let dirty_by_anchor: BTreeMap<&ExcelGridCellAddress, &GridSpillFact> =
        dirty.iter().map(|fact| (&fact.anchor, fact)).collect();
    let mark_all_by_anchor: BTreeMap<&ExcelGridCellAddress, &GridSpillFact> =
        mark_all.iter().map(|fact| (&fact.anchor, fact)).collect();
    let mut anchors: BTreeSet<&ExcelGridCellAddress> = BTreeSet::new();
    anchors.extend(dirty_by_anchor.keys().copied());
    anchors.extend(mark_all_by_anchor.keys().copied());
    anchors
        .into_iter()
        .filter_map(|anchor| {
            let dirty_fact = dirty_by_anchor.get(anchor).copied();
            let mark_all_fact = mark_all_by_anchor.get(anchor).copied();
            (dirty_fact != mark_all_fact).then(|| GridDirtyRecalcSpillFactMismatch {
                anchor: anchor.clone(),
                dirty: dirty_fact.cloned(),
                mark_all: mark_all_fact.cloned(),
            })
        })
        .collect()
}

/// Apply both runs' `external_subscription_updates` to two clones of a
/// caller-seeded `GridExternalAvailabilityTopicRegistry`, then compare the
/// resulting `roots_by_topic` maps per topic.
pub(super) fn compare_grid_dirty_recalc_registry_effect(
    seed_registry: &GridExternalAvailabilityTopicRegistry,
    dirty_updates: &[GridExternalAvailabilitySubscriptionUpdate],
    mark_all_updates: &[GridExternalAvailabilitySubscriptionUpdate],
) -> Vec<GridRegistryEffectMismatch> {
    let mut dirty_registry = seed_registry.clone();
    dirty_registry.apply_subscription_updates(dirty_updates);
    let mut mark_all_registry = seed_registry.clone();
    mark_all_registry.apply_subscription_updates(mark_all_updates);

    let mut topic_ids: BTreeSet<&String> = BTreeSet::new();
    topic_ids.extend(dirty_registry.roots_by_topic().keys());
    topic_ids.extend(mark_all_registry.roots_by_topic().keys());

    topic_ids
        .into_iter()
        .filter_map(|topic_id| {
            let dirty_roots = dirty_registry
                .roots_for_topic(topic_id)
                .cloned()
                .unwrap_or_default();
            let mark_all_roots = mark_all_registry
                .roots_for_topic(topic_id)
                .cloned()
                .unwrap_or_default();
            (dirty_roots != mark_all_roots).then(|| GridRegistryEffectMismatch {
                topic_id: topic_id.clone(),
                dirty_roots,
                mark_all_roots,
            })
        })
        .collect()
}

/// True when two spill-epoch ledgers agree on effective content - the same
/// anchors, each with the same extent/blocked/value_fingerprint - ignoring
/// the monotonic `value_epoch` counter, whose absolute value depends on how
/// many prior recalcs the seeding valuation carried, not on whether the two
/// runs computed the same spill state.
pub(super) fn spill_epoch_ledger_content_equal(
    dirty: &GridSpillEpochLedger,
    mark_all: &GridSpillEpochLedger,
) -> bool {
    let dirty_entries = dirty.entries();
    let mark_all_entries = mark_all.entries();
    if dirty_entries.len() != mark_all_entries.len() {
        return false;
    }
    dirty_entries.iter().all(|(anchor, entry)| {
        mark_all_entries.get(anchor).is_some_and(|other| {
            entry.snapshot.extent == other.snapshot.extent
                && entry.snapshot.blocked == other.snapshot.blocked
                && entry.value_fingerprint == other.value_fingerprint
        })
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_grid_dirty_recalc_differential_report(
    mode: GridEngineMode,
    dirty_recalc: GridEngineRecalcReport,
    mark_all_recalc: GridEngineRecalcReport,
    dirty_readout: Vec<GridEngineCellReadout>,
    mark_all_readout: Vec<GridEngineCellReadout>,
    dirty_spill_facts: Vec<GridSpillFact>,
    mark_all_spill_facts: Vec<GridSpillFact>,
    dirty_dependencies: &GridInvalidationRef,
    mark_all_dependencies: &GridInvalidationRef,
    dirty_dynamic_defined_names: GridDynamicDefinedNameRuntimeSnapshot,
    mark_all_dynamic_defined_names: GridDynamicDefinedNameRuntimeSnapshot,
    dirty_spill_epoch_ledger: &GridSpillEpochLedger,
    mark_all_spill_epoch_ledger: &GridSpillEpochLedger,
    registry_effect_seed: Option<&GridExternalAvailabilityTopicRegistry>,
    dirty_subscription_updates: &[GridExternalAvailabilitySubscriptionUpdate],
    mark_all_subscription_updates: &[GridExternalAvailabilitySubscriptionUpdate],
) -> GridDirtyRecalcDifferentialRunReport {
    let dynamic_defined_name_state_equal =
        dirty_dynamic_defined_names == mark_all_dynamic_defined_names;
    // Compare ledger *content* (anchor -> extent/blocked/value_fingerprint),
    // not the raw monotonic `value_epoch` counter: the epoch is carried
    // forward from whichever prior valuation seeded each run, so an
    // incremental dirty-recalc chain seeded from an already-once-recalced
    // baseline legitimately accumulates a higher epoch than a cold mark-all
    // rebuild seeded from an untouched sheet, even when both runs agree on
    // every effective spill fact. Comparing epoch numbers directly would
    // flag that provenance difference as a false divergence.
    let spill_epoch_ledger_equal =
        spill_epoch_ledger_content_equal(dirty_spill_epoch_ledger, mark_all_spill_epoch_ledger);
    let registry_effect_mismatches = match registry_effect_seed {
        Some(seed_registry) => compare_grid_dirty_recalc_registry_effect(
            seed_registry,
            dirty_subscription_updates,
            mark_all_subscription_updates,
        ),
        None => Vec::new(),
    };
    let registry_effect_equal = registry_effect_mismatches.is_empty();
    GridDirtyRecalcDifferentialRunReport {
        mode,
        dirty_recalc,
        mark_all_recalc,
        mismatches: compare_grid_dirty_recalc_readouts(&dirty_readout, &mark_all_readout),
        spill_fact_mismatches: compare_grid_dirty_recalc_spill_facts(
            &dirty_spill_facts,
            &mark_all_spill_facts,
        ),
        dependency_graphs_equal: dirty_dependencies == mark_all_dependencies,
        dirty_structural_dependency_edges: dirty_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::Structural),
        dirty_overlay_dependency_edges: dirty_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::CalcOverlay),
        mark_all_structural_dependency_edges: mark_all_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::Structural),
        mark_all_overlay_dependency_edges: mark_all_dependencies
            .semantic_dependency_count_for_layer(GridDependencyLayer::CalcOverlay),
        dynamic_defined_name_state_equal,
        dirty_dynamic_defined_names,
        mark_all_dynamic_defined_names,
        spill_epoch_ledger_equal,
        registry_effect_mismatches,
        registry_effect_equal,
        dirty_readout,
        mark_all_readout,
    }
}
