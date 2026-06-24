//! Spill ledger and spill-publication bookkeeping for the strict-excel-grid
//! engines: committed spill-fact records, the epoch ledger tracking anchor
//! extent and value changes across recalcs, and helpers that compute spill
//! extents and value fingerprints. Internal to the machine; shares the
//! machine's types via `use super::*`.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillFact {
    pub anchor: ExcelGridCellAddress,
    pub extent: GridRect,
    pub blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillEpochSnapshot {
    pub anchor: ExcelGridCellAddress,
    pub extent: GridRect,
    pub blocked: bool,
    pub value_epoch: u64,
}

impl GridSpillEpochSnapshot {
    #[must_use]
    pub fn new(fact: GridSpillFact, value_epoch: u64) -> Self {
        Self {
            anchor: fact.anchor,
            extent: fact.extent,
            blocked: fact.blocked,
            value_epoch,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillEpochLedgerEntry {
    pub snapshot: GridSpillEpochSnapshot,
    pub value_fingerprint: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridSpillEpochLedger {
    entries: BTreeMap<ExcelGridCellAddress, GridSpillEpochLedgerEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillEpochLedgerUpdateReport {
    pub anchors_before: usize,
    pub anchors_after: usize,
    pub anchors_added: usize,
    pub anchors_removed: usize,
    pub anchors_changed: usize,
    pub extent_changed_anchors: usize,
    pub value_changed_anchors: usize,
    pub blocked_changed_anchors: usize,
    pub epochs_preserved: usize,
}

impl GridSpillEpochLedger {
    #[must_use]
    pub fn entries(&self) -> &BTreeMap<ExcelGridCellAddress, GridSpillEpochLedgerEntry> {
        &self.entries
    }

    #[must_use]
    pub fn snapshots(&self) -> BTreeMap<ExcelGridCellAddress, GridSpillEpochSnapshot> {
        self.entries
            .iter()
            .map(|(anchor, entry)| (anchor.clone(), entry.snapshot.clone()))
            .collect()
    }

    #[must_use]
    pub fn snapshot_for(&self, anchor: &ExcelGridCellAddress) -> Option<&GridSpillEpochSnapshot> {
        self.entries.get(anchor).map(|entry| &entry.snapshot)
    }

    pub fn update_from_spill_facts<F>(
        &mut self,
        spill_facts: &BTreeMap<ExcelGridCellAddress, GridSpillFact>,
        mut value_fingerprint_for: F,
    ) -> GridSpillEpochLedgerUpdateReport
    where
        F: FnMut(&GridSpillFact) -> String,
    {
        let previous = std::mem::take(&mut self.entries);
        let mut next = BTreeMap::new();
        let mut report = GridSpillEpochLedgerUpdateReport {
            anchors_before: previous.len(),
            anchors_after: spill_facts.len(),
            anchors_added: 0,
            anchors_removed: previous
                .keys()
                .filter(|anchor| !spill_facts.contains_key(*anchor))
                .count(),
            anchors_changed: 0,
            extent_changed_anchors: 0,
            value_changed_anchors: 0,
            blocked_changed_anchors: 0,
            epochs_preserved: 0,
        };

        for (anchor, fact) in spill_facts {
            let value_fingerprint = value_fingerprint_for(fact);
            let (value_epoch, changed) = match previous.get(anchor) {
                Some(entry) => {
                    let extent_changed = entry.snapshot.extent != fact.extent;
                    let blocked_changed = entry.snapshot.blocked != fact.blocked;
                    let value_changed = entry.value_fingerprint != value_fingerprint;
                    if extent_changed {
                        report.extent_changed_anchors += 1;
                    }
                    if blocked_changed {
                        report.blocked_changed_anchors += 1;
                    }
                    if value_changed {
                        report.value_changed_anchors += 1;
                    }
                    if extent_changed || blocked_changed || value_changed {
                        (entry.snapshot.value_epoch.saturating_add(1), true)
                    } else {
                        report.epochs_preserved += 1;
                        (entry.snapshot.value_epoch, false)
                    }
                }
                None => {
                    report.anchors_added += 1;
                    (1, true)
                }
            };
            if changed {
                report.anchors_changed += 1;
            }
            next.insert(
                anchor.clone(),
                GridSpillEpochLedgerEntry {
                    snapshot: GridSpillEpochSnapshot::new(fact.clone(), value_epoch),
                    value_fingerprint,
                },
            );
        }

        self.entries = next;
        report
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridSpillEpochChangeKind {
    Added,
    Removed,
    ExtentChanged,
    ValueChanged,
    BlockedChanged,
    ExtentAndValueChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillEpochAnchorChange {
    pub anchor: ExcelGridCellAddress,
    pub kind: GridSpillEpochChangeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSpillEpochInvalidationReport {
    pub anchors_compared: usize,
    pub changed_anchors: Vec<GridSpillEpochAnchorChange>,
    pub unchanged_anchors: usize,
    pub extent_epoch_changed_anchors: usize,
    pub value_epoch_changed_anchors: usize,
    pub blocked_epoch_changed_anchors: usize,
    pub dirty_closure: BTreeSet<ExcelGridCellAddress>,
}

pub(super) fn anchor_cell_rect(
    address: &ExcelGridCellAddress,
    bounds: ExcelGridBounds,
) -> GridRect {
    GridRect::new(
        address.workbook_id.clone(),
        address.sheet_id.clone(),
        address.row,
        address.col,
        address.row,
        address.col,
        bounds,
    )
    .expect("anchor address was already checked against grid bounds")
}

pub(super) fn spill_extent_for_array(
    anchor: &ExcelGridCellAddress,
    shape: ArrayShape,
    bounds: ExcelGridBounds,
) -> Option<GridRect> {
    let rows = u32::try_from(shape.rows).ok()?;
    let cols = u32::try_from(shape.cols).ok()?;
    let bottom_row = anchor.row.checked_add(rows.checked_sub(1)?)?;
    let right_col = anchor.col.checked_add(cols.checked_sub(1)?)?;
    if !bounds.contains_row(bottom_row) || !bounds.contains_col(right_col) {
        return None;
    }
    GridRect::new(
        anchor.workbook_id.clone(),
        anchor.sheet_id.clone(),
        anchor.row,
        anchor.col,
        bottom_row,
        right_col,
        bounds,
    )
    .ok()
}

pub(super) fn array_cell_address(
    anchor: &ExcelGridCellAddress,
    row_offset: usize,
    col_offset: usize,
) -> Option<ExcelGridCellAddress> {
    let row_offset = u32::try_from(row_offset).ok()?;
    let col_offset = u32::try_from(col_offset).ok()?;
    Some(ExcelGridCellAddress::new(
        anchor.workbook_id.clone(),
        anchor.sheet_id.clone(),
        anchor.row.checked_add(row_offset)?,
        anchor.col.checked_add(col_offset)?,
    ))
}

pub(super) fn formula_count(authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>) -> usize {
    authored
        .values()
        .filter(|cell| matches!(cell, GridAuthoredCell::Formula(_)))
        .count()
}

pub(super) fn formula_contains_grid_spill_reference(
    formula: &GridFormulaCell,
    address: &ExcelGridCellAddress,
    profile: &StrictExcelGridReferenceProfile,
    bounds: ExcelGridBounds,
) -> bool {
    let bound = bind_grid_formula_for_transform(formula, address, profile, bounds);
    if bound.normalized_references.iter().any(|normalized| {
        let NormalizedReference::ProfileSymbolic(record) = normalized else {
            return false;
        };
        if record.profile_id != EXCEL_GRID_PROFILE_ID {
            return false;
        }
        matches!(
            decode_excel_grid_reference_payload(&record.profile_payload),
            Some(ExcelGridReference::SpillAnchor { .. })
        )
    }) {
        return true;
    }

    formula.source_text.contains('#')
}

pub(super) fn authored_contains_grid_spill_reference(
    authored: &BTreeMap<ExcelGridCellAddress, GridAuthoredCell>,
    profile: &StrictExcelGridReferenceProfile,
    bounds: ExcelGridBounds,
) -> bool {
    authored.iter().any(|(address, cell)| {
        let GridAuthoredCell::Formula(formula) = cell else {
            return false;
        };
        formula_contains_grid_spill_reference(formula, address, profile, bounds)
    })
}

pub(super) fn count_formula_spill_publications<F>(
    spill_facts: &BTreeMap<ExcelGridCellAddress, GridSpillFact>,
    mut is_formula_anchor: F,
) -> GridSpillPublicationCounters
where
    F: FnMut(&ExcelGridCellAddress) -> bool,
{
    let mut counters = GridSpillPublicationCounters::default();
    for fact in spill_facts.values() {
        if !is_formula_anchor(&fact.anchor) {
            continue;
        }
        if fact.blocked {
            counters.facts_blocked += 1;
            continue;
        }
        counters.facts_published += 1;
        counters.ghost_cells_published +=
            usize::try_from(fact.extent.cell_count().saturating_sub(1)).unwrap_or(usize::MAX);
    }
    counters
}

pub(super) fn calc_value_fingerprint(value: &CalcValue) -> String {
    format!("{value:?}")
}

pub(super) fn calc_array_value_fingerprint(array: &CalcArray) -> String {
    let shape = array.shape();
    let mut fingerprint = format!("array:{}x{}:", shape.rows, shape.cols);
    for row in 0..shape.rows {
        for col in 0..shape.cols {
            match array.get(row, col) {
                Some(value) => fingerprint.push_str(&calc_value_fingerprint(value)),
                None => fingerprint.push_str("<missing>"),
            }
            fingerprint.push('|');
        }
    }
    fingerprint
}

pub(super) fn blocked_spill_value_fingerprint(array: &CalcArray) -> String {
    format!("blocked:{}", calc_array_value_fingerprint(array))
}

pub(super) fn manual_spill_fact_value_fingerprint(fact: &GridSpillFact) -> String {
    format!(
        "manual:{}:{}:{}:{}:{}:{}:{}",
        fact.anchor.workbook_id,
        fact.anchor.sheet_id,
        fact.anchor.row,
        fact.anchor.col,
        fact.extent.cell_count(),
        fact.blocked,
        if fact.blocked { "blocked" } else { "published" }
    )
}

pub(super) fn blocked_formula_spill_extent_contains_anchor<F>(
    anchor: &ExcelGridCellAddress,
    spill_facts: &BTreeMap<ExcelGridCellAddress, GridSpillFact>,
    mut is_formula_anchor: F,
) -> bool
where
    F: FnMut(&ExcelGridCellAddress) -> bool,
{
    spill_facts.values().any(|fact| {
        fact.blocked
            && fact.anchor != *anchor
            && is_formula_anchor(&fact.anchor)
            && fact.extent.contains(anchor)
    })
}
