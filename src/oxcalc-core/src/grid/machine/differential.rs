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
