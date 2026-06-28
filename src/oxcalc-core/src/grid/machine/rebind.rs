//! Dynamic-rebind consequence model for the strict-excel-grid engines: the
//! claim/consequence vocabulary describing what a dynamic reference (an
//! `INDIRECT`/`OFFSET`-style cell request, a spill anchor, or a structured-table
//! feeder) resolved to before and after a recalc, and the cause-driven
//! resolution that folds each claim into its exact dirty closure and
//! invalidation reasons. Internal to the machine; shares the machine's types
//! via `use super::*`, and pulls in the crate-root dependency/structured-table
//! vocabulary it classifies against.

use super::*;

use crate::dependency::{DependencyDescriptorKind, InvalidationReasonKind};
use crate::structured_table::{
    TreeCalcDynamicTableRebindCause, TreeCalcDynamicTableRebindReport,
    TreeCalcDynamicTableRebindRequest, TreeCalcDynamicTableRebindStatus,
    TreeCalcDynamicTableReferenceTargetKind, TreeCalcTableUpdateScenarioKind,
    classify_treecalc_dynamic_table_rebind,
};

/// The concrete identity a dynamic reference resolved to in one recalc epoch.
///
/// Two identities comparing unequal is the signal that a dynamic reference
/// changed what it points at; equal identities mean the resolution was
/// reference-preserving and nothing downstream needs to be dirtied.
///
/// Only `Eq` is derived: `GridSpillEpochSnapshot` implements neither `Ord` nor
/// `Hash` (and `GridRect` lacks `Hash`), so the spec's `PartialOrd, Ord, Hash`
/// would require widening those existing types. Identity equality is all this
/// model needs — the identity is only ever held in an `Option` and compared
/// with `==`/`!=` by the transition classifier, never used as a map/set key —
/// so the deviation is sound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedReferenceIdentity {
    /// A cell-shaped dynamic request keyed by its request string, resolving to a
    /// rectangular target.
    Cell {
        request_key: String,
        target: GridRect,
    },
    /// A spill anchor, carried as the full epoch snapshot so extent/value/blocked
    /// transitions are part of the identity.
    Spill(GridSpillEpochSnapshot),
    /// A structured-table feeder keyed by table identity (CTRO-2 populates this).
    Table {
        table_key: String,
        resolved_identity: String,
    },
}

/// Which family of dynamic reference a claim belongs to. Selects the resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicRebindFamily {
    CellDynamicRequest,
    SpillAnchorRef,
    StructuredTableRebind,
}

/// Why a dynamic reference's resolution changed across a recalc. Drives the
/// invalidation-reason and changed-dependency-kind sets via
/// [`cause_to_reason_table`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynamicRebindCause {
    /// A previously unresolved dynamic reference now resolves to a target.
    IdentityActivated,
    /// The reference resolves to a different target than before.
    IdentityReclassified,
    /// The reference no longer resolves to a target.
    IdentityReleased,
    /// The reference resolves to the same target it did before.
    IdentityPreserved,
    /// A spill anchor's epoch changed in the indicated way.
    SpillEpochChanged(GridSpillEpochChangeKind),
    /// A structured-table feeder rebind (CTRO-2 territory; copied verbatim).
    Table(TreeCalcDynamicTableRebindCause),
    /// An axis edit landed concurrently with a rebind.
    AxisEditDuringRebind,
}

/// The outcome classification recorded on a consequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicRebindStatus {
    Activated,
    Reclassified,
    Released,
    ReferencePreserving,
    Changed,
    Error,
    Excluded,
}

/// The worksheet-error effect a rebind imposes on the owning cell, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicRebindErrorEffect {
    None,
    Ref,
    Spill,
    NameOrValue,
}

/// The structural-graph change a rebind implies (e.g. an anchor appearing or a
/// table extent shifting). Cell and spill families only ever need a small slice
/// of these in CTRO-1; the rest are reserved for the structured-table feeder.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicRebindStructuralChange {
    None,
    AnchorAddressTransformed,
    AnchorAdded,
    AnchorRemoved,
    TableExtentChanged,
    TableKeyChanged,
    WorkspaceAvailabilityChanged,
    ReverseIndexRebuilt,
}

/// A single dynamic-reference rebind to resolve: who owns it, which request key
/// indexes its dependents, and what it resolved to before and after.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicRebindClaim {
    pub claim_id: String,
    pub family: DynamicRebindFamily,
    pub owner: ExcelGridCellAddress,
    pub request_key: String,
    pub target_kind: Option<TreeCalcDynamicTableReferenceTargetKind>,
    pub before_identity: Option<ResolvedReferenceIdentity>,
    pub after_identity: Option<ResolvedReferenceIdentity>,
    pub cause: DynamicRebindCause,
    pub table_request: Option<TreeCalcDynamicTableRebindRequest>,
}

/// The resolved consequence of a claim: the exact dirty closure plus the
/// invalidation reasons, changed dependency kinds, error effect, and structural
/// change it implies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicRebindConsequence {
    pub claim_id: String,
    pub family: DynamicRebindFamily,
    pub status: DynamicRebindStatus,
    pub dirty_closure: BTreeSet<ExcelGridCellAddress>,
    pub invalidation_reasons: BTreeSet<InvalidationReasonKind>,
    pub changed_dependency_kinds: BTreeSet<DependencyDescriptorKind>,
    pub error_effect: DynamicRebindErrorEffect,
    pub structural_change: DynamicRebindStructuralChange,
    pub table_report: Option<TreeCalcDynamicTableRebindReport>,
}

/// Classify a before/after identity transition into its status and cause.
///
/// `(None, None)` is unreachable for a live claim but is folded into the
/// reference-preserving arm so the function is total.
#[must_use]
pub fn classify_identity_transition(
    before: Option<&ResolvedReferenceIdentity>,
    after: Option<&ResolvedReferenceIdentity>,
) -> (DynamicRebindStatus, DynamicRebindCause) {
    match (before, after) {
        (None, Some(_)) => (
            DynamicRebindStatus::Activated,
            DynamicRebindCause::IdentityActivated,
        ),
        (Some(_), None) => (
            DynamicRebindStatus::Released,
            DynamicRebindCause::IdentityReleased,
        ),
        (Some(before), Some(after)) if before != after => (
            DynamicRebindStatus::Reclassified,
            DynamicRebindCause::IdentityReclassified,
        ),
        (Some(_), Some(_)) | (None, None) => (
            DynamicRebindStatus::ReferencePreserving,
            DynamicRebindCause::IdentityPreserved,
        ),
    }
}

/// Map a cell/spill cause onto its `(invalidation_reasons, changed_dependency_kinds)`
/// pair. Table causes are resolved by the structured-table feeder and copy their
/// report verbatim, so they never reach this table.
#[must_use]
pub fn cause_to_reason_table(
    _family: DynamicRebindFamily,
    cause: &DynamicRebindCause,
) -> (
    BTreeSet<InvalidationReasonKind>,
    BTreeSet<DependencyDescriptorKind>,
) {
    use DependencyDescriptorKind as Kind;
    use InvalidationReasonKind as Reason;

    let (reasons, kinds): (&[Reason], &[Kind]) = match cause {
        DynamicRebindCause::IdentityActivated => (
            &[Reason::DynamicDependencyActivated],
            &[Kind::DynamicPotential],
        ),
        DynamicRebindCause::IdentityReclassified => (
            &[Reason::DynamicDependencyReclassified],
            &[Kind::DynamicPotential],
        ),
        DynamicRebindCause::IdentityReleased => (
            &[Reason::DynamicDependencyReleased],
            &[Kind::DynamicPotential],
        ),
        DynamicRebindCause::IdentityPreserved => (&[], &[]),
        DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::Added) => (
            &[Reason::DependencyAdded, Reason::DynamicDependencyActivated],
            &[Kind::DynamicPotential],
        ),
        DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::Removed) => (
            &[Reason::DependencyRemoved, Reason::DynamicDependencyReleased],
            &[Kind::DynamicPotential],
        ),
        DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::ExtentChanged)
        | DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::ExtentAndValueChanged) => {
            (
                &[
                    Reason::StructuralRecalcOnly,
                    Reason::DynamicDependencyReclassified,
                ],
                &[Kind::DynamicPotential, Kind::ShapeTopology],
            )
        }
        DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::ValueChanged) => {
            (&[Reason::UpstreamPublication], &[Kind::DynamicPotential])
        }
        DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::BlockedChanged) => (
            &[
                Reason::StructuralRebindRequired,
                Reason::DynamicDependencyReclassified,
            ],
            &[Kind::DynamicPotential],
        ),
        DynamicRebindCause::AxisEditDuringRebind => (
            &[Reason::StructuralRebindRequired],
            &[Kind::DynamicPotential],
        ),
        DynamicRebindCause::Table(_) => {
            unreachable!(
                "table causes are resolved by the structured-table feeder, which copies its \
                 report verbatim; cause_to_reason_table handles cell and spill families only"
            )
        }
    };

    (
        reasons.iter().copied().collect(),
        kinds.iter().copied().collect(),
    )
}

/// Resolve a cell dynamic-request claim into its consequence.
///
/// A non-reference-preserving transition is dirtied through the request key's
/// pre-existing dependent closure: the same key holds the union of the old and
/// new dependents because a reclassification is recorded in the resolved
/// identity, not in the request key, so the key (and therefore its closure)
/// is stable across the rebind.
fn resolve_cell_dynamic_request_claim(
    refs: &GridInvalidationRef,
    claim: &DynamicRebindClaim,
) -> Result<DynamicRebindConsequence, GridRefError> {
    let (status, cause) = classify_identity_transition(
        claim.before_identity.as_ref(),
        claim.after_identity.as_ref(),
    );

    if status == DynamicRebindStatus::ReferencePreserving {
        return Ok(DynamicRebindConsequence {
            claim_id: claim.claim_id.clone(),
            family: claim.family,
            status,
            dirty_closure: BTreeSet::new(),
            invalidation_reasons: BTreeSet::new(),
            changed_dependency_kinds: BTreeSet::new(),
            error_effect: DynamicRebindErrorEffect::None,
            structural_change: DynamicRebindStructuralChange::None,
            table_report: None,
        });
    }

    let dirty_closure = refs.dirty_closure_for_dynamic_request(&claim.request_key);
    let (invalidation_reasons, changed_dependency_kinds) =
        cause_to_reason_table(DynamicRebindFamily::CellDynamicRequest, &cause);
    let error_effect = if status == DynamicRebindStatus::Released {
        DynamicRebindErrorEffect::Ref
    } else {
        DynamicRebindErrorEffect::None
    };

    Ok(DynamicRebindConsequence {
        claim_id: claim.claim_id.clone(),
        family: claim.family,
        status,
        dirty_closure,
        invalidation_reasons,
        changed_dependency_kinds,
        error_effect,
        structural_change: DynamicRebindStructuralChange::None,
        table_report: None,
    })
}

/// Pull the spill epoch snapshot out of a resolved identity, if present.
fn spill_snapshot_of(
    identity: Option<&ResolvedReferenceIdentity>,
) -> Option<GridSpillEpochSnapshot> {
    match identity {
        Some(ResolvedReferenceIdentity::Spill(snapshot)) => Some(snapshot.clone()),
        _ => None,
    }
}

/// Resolve a spill-anchor claim into its consequence.
///
/// The dirty closure is seeded strictly by the anchor cell(s) — the union over
/// the before-anchor and after-anchor — never by the (possibly grown or shrunk)
/// spill extent. New or vacated extent cells therefore never enter the closure
/// here: only cells that declared a spill-fact dependency on the anchor do.
fn resolve_spill_anchor_claim(
    refs: &GridInvalidationRef,
    claim: &DynamicRebindClaim,
) -> Result<DynamicRebindConsequence, GridRefError> {
    let before = spill_snapshot_of(claim.before_identity.as_ref());
    let after = spill_snapshot_of(claim.after_identity.as_ref());

    let Some(kind) = spill_epoch_change_kind(before.as_ref(), after.as_ref()) else {
        return Ok(DynamicRebindConsequence {
            claim_id: claim.claim_id.clone(),
            family: claim.family,
            status: DynamicRebindStatus::ReferencePreserving,
            dirty_closure: BTreeSet::new(),
            invalidation_reasons: BTreeSet::new(),
            changed_dependency_kinds: BTreeSet::new(),
            error_effect: DynamicRebindErrorEffect::None,
            structural_change: DynamicRebindStructuralChange::None,
            table_report: None,
        });
    };

    let cause = DynamicRebindCause::SpillEpochChanged(kind);

    let mut anchors = BTreeSet::new();
    if let Some(before) = &before {
        anchors.insert(before.anchor.clone());
    }
    if let Some(after) = &after {
        anchors.insert(after.anchor.clone());
    }
    let mut dirty_closure = BTreeSet::new();
    for anchor in anchors {
        dirty_closure
            .extend(refs.dirty_closure_for_spill_fact(GridSpillDependency::anchor(anchor))?);
    }

    let status = match kind {
        GridSpillEpochChangeKind::Added => DynamicRebindStatus::Changed,
        GridSpillEpochChangeKind::Removed => DynamicRebindStatus::Released,
        _ => DynamicRebindStatus::Reclassified,
    };

    let (invalidation_reasons, changed_dependency_kinds) =
        cause_to_reason_table(DynamicRebindFamily::SpillAnchorRef, &cause);

    let error_effect = if after.as_ref().is_some_and(|snapshot| snapshot.blocked) {
        DynamicRebindErrorEffect::Spill
    } else if status == DynamicRebindStatus::Released {
        DynamicRebindErrorEffect::Ref
    } else {
        DynamicRebindErrorEffect::None
    };

    let structural_change = match kind {
        GridSpillEpochChangeKind::Added => DynamicRebindStructuralChange::AnchorAdded,
        GridSpillEpochChangeKind::Removed => DynamicRebindStructuralChange::AnchorRemoved,
        _ => DynamicRebindStructuralChange::None,
    };

    Ok(DynamicRebindConsequence {
        claim_id: claim.claim_id.clone(),
        family: claim.family,
        status,
        dirty_closure,
        invalidation_reasons,
        changed_dependency_kinds,
        error_effect,
        structural_change,
        table_report: None,
    })
}

/// Map a classifier table status onto the rebind status this model records.
///
/// `RebindRequired` is the live-rebind case (a resolved table changed and its
/// dependents must be re-derived) so it folds onto `Reclassified`; both
/// "target gone" outcomes (`DeletedTarget`/`UnavailableTarget`) fold onto
/// `Released`; a `TypedExclusion` (the classifier admitting no table lowering)
/// folds onto `Excluded`.
#[must_use]
pub fn table_status_to_rebind_status(
    status: TreeCalcDynamicTableRebindStatus,
) -> DynamicRebindStatus {
    match status {
        TreeCalcDynamicTableRebindStatus::ReferencePreserving => {
            DynamicRebindStatus::ReferencePreserving
        }
        TreeCalcDynamicTableRebindStatus::RebindRequired => DynamicRebindStatus::Reclassified,
        TreeCalcDynamicTableRebindStatus::DeletedTarget
        | TreeCalcDynamicTableRebindStatus::UnavailableTarget => DynamicRebindStatus::Released,
        TreeCalcDynamicTableRebindStatus::TypedExclusion => DynamicRebindStatus::Excluded,
    }
}

/// Pull the human table name carried by a `Table` resolved identity, if present.
///
/// **Table-name-for-closure resolution (the CTRO-2 open question):** the
/// classifier report carries opaque *identity* strings
/// (`*_resolved_table_identity`), but `dirty_closure_for_table` keys on the
/// human table *name* (it derives the table key via `excel_grid_table_name_key`,
/// the same derivation `GridTableDependency::new` performs when a dependent is
/// seeded). Rather than widen any type or try to invert an identity string back
/// to a name, the claim conveys the name on the existing
/// `ResolvedReferenceIdentity::Table { table_key, resolved_identity }`: its
/// `table_key` field holds the human table name (fed verbatim to
/// `dirty_closure_for_table`) and `resolved_identity` mirrors the report's
/// identity string. A rename is read from the before/after identities' names —
/// exactly parallel to how the cell and spill feeders read their before/after
/// state from the claim, not from the classifier report — so no new field is
/// needed and the closure round-trips a seeded
/// `GridDependency::Table(GridTableDependency::new(name, ..))`.
fn table_name_of(identity: Option<&ResolvedReferenceIdentity>) -> Option<&str> {
    match identity {
        Some(ResolvedReferenceIdentity::Table { table_key, .. }) => Some(table_key.as_str()),
        _ => None,
    }
}

/// Resolve a structured-table feeder claim into its consequence.
///
/// The status, invalidation reasons, and changed dependency kinds round-trip the
/// classifier report verbatim (the CTRO-2 invariant): they are copied straight
/// off the report rather than routed through [`cause_to_reason_table`], whose
/// `Table(_)` arm is unreachable. The dirty closure seeds the dependents of the
/// affected table(s) via `dirty_closure_for_table`: the new resolved table (if
/// any) plus the old resolved table whenever it differs — including a delete,
/// where `after_name` is `None` and the OLD table's referrers must still
/// recompute to `#REF!` (this mirrors `delete_table`, which dirties the old
/// table key before purging it). Only a reference-preserving rebind or a typed
/// exclusion dirties nothing — the selector still resolves to the same live
/// name, with no retarget to fan out from. A table that grows into a spill is
/// modelled as a separate `SpillAnchorRef` claim, never folded into this
/// consequence.
fn resolve_structured_table_claim(
    refs: &GridInvalidationRef,
    claim: &DynamicRebindClaim,
) -> Result<DynamicRebindConsequence, GridRefError> {
    let request = claim
        .table_request
        .as_ref()
        .expect("structured-table claim carries a table_request");
    let report = classify_treecalc_dynamic_table_rebind(request);
    let status = table_status_to_rebind_status(report.status);

    // VERBATIM round-trip: the classifier already cleared reasons/kinds for a
    // reference-preserving rebind and RETAINED them (clearing only its own
    // dependency_fact_kinds) for a typed exclusion, so copying them as-is is the
    // whole point of the feeder.
    let invalidation_reasons = report.invalidation_reasons.clone();
    let changed_dependency_kinds = report.changed_dependency_kinds.clone();

    let before_name = table_name_of(claim.before_identity.as_ref());
    let after_name = table_name_of(claim.after_identity.as_ref());

    let dirty_closure = match status {
        // No live table body to dirty downstream of: an unchanged resolution
        // (ReferencePreserving) or an excluded selector still resolving to the
        // same live name (TypedExclusion - the classifier cleared its fact kinds
        // and there is no retarget to fan out from).
        DynamicRebindStatus::ReferencePreserving | DynamicRebindStatus::Excluded => BTreeSet::new(),
        // RebindRequired (Reclassified) and a deleted/unavailable target (Released)
        // both dirty the dependents of the NEW resolved table (if any) UNION the
        // OLD resolved table whenever it differs - including a delete, where
        // after_name is None so the old name's referrers must recompute to #REF!
        // (mirrors delete_table dirtying the old key before purging it).
        _ => {
            let mut closure = BTreeSet::new();
            if let Some(after_name) = after_name {
                closure.extend(refs.dirty_closure_for_table(after_name)?);
            }
            if let Some(before_name) = before_name {
                if after_name != Some(before_name) {
                    closure.extend(refs.dirty_closure_for_table(before_name)?);
                }
            }
            closure
        }
    };

    let error_effect = match status {
        DynamicRebindStatus::Excluded | DynamicRebindStatus::Released => {
            DynamicRebindErrorEffect::Ref
        }
        _ => DynamicRebindErrorEffect::None,
    };

    let structural_change =
        structural_change_for_table_cause(&request.cause, before_name, after_name);

    Ok(DynamicRebindConsequence {
        claim_id: claim.claim_id.clone(),
        family: claim.family,
        status,
        dirty_closure,
        invalidation_reasons,
        changed_dependency_kinds,
        error_effect,
        structural_change,
        table_report: Some(report),
    })
}

/// Derive the structural-graph change a table rebind implies from its cause.
///
/// A rename (the cause names a rename, or before/after resolved names differ)
/// is a key change; row/column/resize lifecycle is an extent change; opening or
/// closing a workspace is an availability change; everything else is structurally
/// inert from the graph's point of view.
fn structural_change_for_table_cause(
    cause: &TreeCalcDynamicTableRebindCause,
    before_name: Option<&str>,
    after_name: Option<&str>,
) -> DynamicRebindStructuralChange {
    let renamed = matches!(
        (before_name, after_name),
        (Some(before), Some(after)) if before != after
    );
    match cause {
        TreeCalcDynamicTableRebindCause::TableLifecycle(scenario) => match scenario {
            TreeCalcTableUpdateScenarioKind::TableRename
            | TreeCalcTableUpdateScenarioKind::NodeRename
            | TreeCalcTableUpdateScenarioKind::ColumnRename => {
                DynamicRebindStructuralChange::TableKeyChanged
            }
            TreeCalcTableUpdateScenarioKind::TableResize
            | TreeCalcTableUpdateScenarioKind::TableMove
            | TreeCalcTableUpdateScenarioKind::NodeMove
            | TreeCalcTableUpdateScenarioKind::RowInsert
            | TreeCalcTableUpdateScenarioKind::RowDelete
            | TreeCalcTableUpdateScenarioKind::RowReorder
            | TreeCalcTableUpdateScenarioKind::ColumnInsert
            | TreeCalcTableUpdateScenarioKind::ColumnDelete
            | TreeCalcTableUpdateScenarioKind::ColumnReorder => {
                DynamicRebindStructuralChange::TableExtentChanged
            }
            TreeCalcTableUpdateScenarioKind::WorkspaceOpen
            | TreeCalcTableUpdateScenarioKind::WorkspaceClose => {
                DynamicRebindStructuralChange::WorkspaceAvailabilityChanged
            }
            _ if renamed => DynamicRebindStructuralChange::TableKeyChanged,
            _ => DynamicRebindStructuralChange::None,
        },
        // Selector / dynamic-fn retargets carry no lifecycle scenario; a change of
        // resolved name is still a key change.
        _ if renamed => DynamicRebindStructuralChange::TableKeyChanged,
        _ => DynamicRebindStructuralChange::None,
    }
}

/// Resolve a dynamic-rebind claim into its consequence by dispatching on family.
pub fn resolve_dynamic_rebind_claim(
    refs: &GridInvalidationRef,
    claim: &DynamicRebindClaim,
) -> Result<DynamicRebindConsequence, GridRefError> {
    match claim.family {
        DynamicRebindFamily::CellDynamicRequest => resolve_cell_dynamic_request_claim(refs, claim),
        DynamicRebindFamily::SpillAnchorRef => resolve_spill_anchor_claim(refs, claim),
        DynamicRebindFamily::StructuredTableRebind => resolve_structured_table_claim(refs, claim),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structured_table::TreeCalcTableLifecycleContextVersions;

    fn bounds() -> ExcelGridBounds {
        ExcelGridBounds {
            max_rows: 10,
            max_cols: 5,
        }
    }

    fn address(row: u32, col: u32) -> ExcelGridCellAddress {
        ExcelGridCellAddress::new("book:default", "sheet:default", row, col)
    }

    fn set(
        addresses: impl IntoIterator<Item = ExcelGridCellAddress>,
    ) -> BTreeSet<ExcelGridCellAddress> {
        addresses.into_iter().collect()
    }

    fn rect(row_a: u32, col_a: u32, row_b: u32, col_b: u32) -> GridRect {
        GridRect::new(
            "book:default",
            "sheet:default",
            row_a,
            col_a,
            row_b,
            col_b,
            bounds(),
        )
        .unwrap()
    }

    fn cell_identity(request_key: &str, target: GridRect) -> ResolvedReferenceIdentity {
        ResolvedReferenceIdentity::Cell {
            request_key: request_key.to_string(),
            target,
        }
    }

    fn spill_snapshot(
        anchor: ExcelGridCellAddress,
        extent: GridRect,
        blocked: bool,
        value_epoch: u64,
    ) -> GridSpillEpochSnapshot {
        GridSpillEpochSnapshot {
            anchor,
            extent,
            blocked,
            value_epoch,
        }
    }

    fn cell_claim(
        request_key: &str,
        before: Option<ResolvedReferenceIdentity>,
        after: Option<ResolvedReferenceIdentity>,
    ) -> DynamicRebindClaim {
        let (_, cause) = classify_identity_transition(before.as_ref(), after.as_ref());
        DynamicRebindClaim {
            claim_id: format!("claim:{request_key}"),
            family: DynamicRebindFamily::CellDynamicRequest,
            owner: address(1, 1),
            request_key: request_key.to_string(),
            target_kind: None,
            before_identity: before,
            after_identity: after,
            cause,
            table_request: None,
        }
    }

    fn spill_claim(
        request_key: &str,
        before: Option<GridSpillEpochSnapshot>,
        after: Option<GridSpillEpochSnapshot>,
    ) -> DynamicRebindClaim {
        let cause = match spill_epoch_change_kind(before.as_ref(), after.as_ref()) {
            Some(kind) => DynamicRebindCause::SpillEpochChanged(kind),
            None => DynamicRebindCause::IdentityPreserved,
        };
        DynamicRebindClaim {
            claim_id: format!("claim:{request_key}"),
            family: DynamicRebindFamily::SpillAnchorRef,
            owner: address(1, 1),
            request_key: request_key.to_string(),
            target_kind: None,
            before_identity: before.map(ResolvedReferenceIdentity::Spill),
            after_identity: after.map(ResolvedReferenceIdentity::Spill),
            cause,
            table_request: None,
        }
    }

    /// A `Table` resolved identity carrying the human table NAME in `table_key`
    /// (the closure key) and the classifier identity string in `resolved_identity`.
    fn table_identity(name: &str, resolved_identity: &str) -> ResolvedReferenceIdentity {
        ResolvedReferenceIdentity::Table {
            table_key: name.to_string(),
            resolved_identity: resolved_identity.to_string(),
        }
    }

    /// A structured-table claim with the given cause, before/after resolved table
    /// names (driving the closure key + rename detection) and identity strings
    /// (mirrored into the `table_request`'s `*_resolved_table_identity`).
    fn table_claim(
        cause: TreeCalcDynamicTableRebindCause,
        target_kind: TreeCalcDynamicTableReferenceTargetKind,
        before: Option<(&str, &str)>,
        after: Option<(&str, &str)>,
    ) -> DynamicRebindClaim {
        let request = TreeCalcDynamicTableRebindRequest {
            selector_handle: "dynamic-table-selector:1".to_string(),
            selector_identity: "dynamic-selector:Sales[#Data]".to_string(),
            source_reference_handle: Some("structured-ref:dynamic-table".to_string()),
            target_kind,
            cause: cause.clone(),
            before_resolved_table_identity: before.map(|(_, identity)| identity.to_string()),
            after_resolved_table_identity: after.map(|(_, identity)| identity.to_string()),
            caller_context_id: None,
            context_versions: TreeCalcTableLifecycleContextVersions::default(),
            oxfml_structured_bind_packet_available: true,
        };
        DynamicRebindClaim {
            claim_id: "claim:table".to_string(),
            family: DynamicRebindFamily::StructuredTableRebind,
            owner: address(1, 1),
            request_key: "table-request".to_string(),
            target_kind: Some(target_kind),
            before_identity: before.map(|(name, identity)| table_identity(name, identity)),
            after_identity: after.map(|(name, identity)| table_identity(name, identity)),
            cause: DynamicRebindCause::Table(cause),
            table_request: Some(request),
        }
    }

    /// Seed an invalidation ref with a table dependent on `name` plus a chained
    /// scalar dependent, returning both for closure asserts.
    fn refs_with_table_dependent(
        name: &str,
    ) -> (
        GridInvalidationRef,
        ExcelGridCellAddress,
        ExcelGridCellAddress,
    ) {
        let mut refs = GridInvalidationRef::new(bounds());
        let consumer = address(2, 1);
        let downstream = address(3, 1);
        refs.set_cell_dependencies(
            consumer.clone(),
            [GridDependency::Table(
                GridTableDependency::new(name, rect(1, 1, 2, 2), bounds()).unwrap(),
            )],
        )
        .unwrap();
        refs.set_cell_dependencies(downstream.clone(), [GridDependency::Cell(consumer.clone())])
            .unwrap();
        (refs, consumer, downstream)
    }

    fn reasons(
        items: impl IntoIterator<Item = InvalidationReasonKind>,
    ) -> BTreeSet<InvalidationReasonKind> {
        items.into_iter().collect()
    }

    fn kinds(
        items: impl IntoIterator<Item = DependencyDescriptorKind>,
    ) -> BTreeSet<DependencyDescriptorKind> {
        items.into_iter().collect()
    }

    #[test]
    fn classify_identity_transition_covers_all_five_transitions() {
        let target = rect(5, 5, 5, 5);
        let x = cell_identity("k", rect(1, 1, 1, 1));
        let y = cell_identity("k", target);

        assert_eq!(
            classify_identity_transition(None, Some(&x)),
            (
                DynamicRebindStatus::Activated,
                DynamicRebindCause::IdentityActivated
            )
        );
        assert_eq!(
            classify_identity_transition(Some(&x), None),
            (
                DynamicRebindStatus::Released,
                DynamicRebindCause::IdentityReleased
            )
        );
        assert_eq!(
            classify_identity_transition(Some(&x), Some(&y)),
            (
                DynamicRebindStatus::Reclassified,
                DynamicRebindCause::IdentityReclassified
            )
        );
        assert_eq!(
            classify_identity_transition(Some(&x), Some(&x)),
            (
                DynamicRebindStatus::ReferencePreserving,
                DynamicRebindCause::IdentityPreserved
            )
        );
        assert_eq!(
            classify_identity_transition(None, None),
            (
                DynamicRebindStatus::ReferencePreserving,
                DynamicRebindCause::IdentityPreserved
            )
        );
    }

    #[test]
    fn cause_to_reason_table_matches_cell_and_spill_causes() {
        let family = DynamicRebindFamily::CellDynamicRequest;

        assert_eq!(
            cause_to_reason_table(family, &DynamicRebindCause::IdentityActivated),
            (
                reasons([InvalidationReasonKind::DynamicDependencyActivated]),
                kinds([DependencyDescriptorKind::DynamicPotential])
            )
        );
        assert_eq!(
            cause_to_reason_table(family, &DynamicRebindCause::IdentityReclassified),
            (
                reasons([InvalidationReasonKind::DynamicDependencyReclassified]),
                kinds([DependencyDescriptorKind::DynamicPotential])
            )
        );
        assert_eq!(
            cause_to_reason_table(family, &DynamicRebindCause::IdentityReleased),
            (
                reasons([InvalidationReasonKind::DynamicDependencyReleased]),
                kinds([DependencyDescriptorKind::DynamicPotential])
            )
        );
        assert_eq!(
            cause_to_reason_table(family, &DynamicRebindCause::IdentityPreserved),
            (reasons([]), kinds([]))
        );

        let spill = DynamicRebindFamily::SpillAnchorRef;
        assert_eq!(
            cause_to_reason_table(
                spill,
                &DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::Added)
            ),
            (
                reasons([
                    InvalidationReasonKind::DependencyAdded,
                    InvalidationReasonKind::DynamicDependencyActivated
                ]),
                kinds([DependencyDescriptorKind::DynamicPotential])
            )
        );
        assert_eq!(
            cause_to_reason_table(
                spill,
                &DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::Removed)
            ),
            (
                reasons([
                    InvalidationReasonKind::DependencyRemoved,
                    InvalidationReasonKind::DynamicDependencyReleased
                ]),
                kinds([DependencyDescriptorKind::DynamicPotential])
            )
        );
        let extent_reasons = (
            reasons([
                InvalidationReasonKind::StructuralRecalcOnly,
                InvalidationReasonKind::DynamicDependencyReclassified,
            ]),
            kinds([
                DependencyDescriptorKind::DynamicPotential,
                DependencyDescriptorKind::ShapeTopology,
            ]),
        );
        assert_eq!(
            cause_to_reason_table(
                spill,
                &DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::ExtentChanged)
            ),
            extent_reasons
        );
        assert_eq!(
            cause_to_reason_table(
                spill,
                &DynamicRebindCause::SpillEpochChanged(
                    GridSpillEpochChangeKind::ExtentAndValueChanged
                )
            ),
            extent_reasons
        );
        assert_eq!(
            cause_to_reason_table(
                spill,
                &DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::ValueChanged)
            ),
            (
                reasons([InvalidationReasonKind::UpstreamPublication]),
                kinds([DependencyDescriptorKind::DynamicPotential])
            )
        );
        assert_eq!(
            cause_to_reason_table(
                spill,
                &DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::BlockedChanged)
            ),
            (
                reasons([
                    InvalidationReasonKind::StructuralRebindRequired,
                    InvalidationReasonKind::DynamicDependencyReclassified
                ]),
                kinds([DependencyDescriptorKind::DynamicPotential])
            )
        );
        // The cross-cutting axis-edit cause (produced by the CTRO-3 differential,
        // not by the CTRO-1 feeders) still maps through this table.
        assert_eq!(
            cause_to_reason_table(spill, &DynamicRebindCause::AxisEditDuringRebind),
            (
                reasons([InvalidationReasonKind::StructuralRebindRequired]),
                kinds([DependencyDescriptorKind::DynamicPotential])
            )
        );
    }

    #[test]
    fn cell_activated_dirties_request_closure_with_chained_dependent() {
        let mut refs = GridInvalidationRef::new(bounds());
        let b1 = address(1, 2);
        let c1 = address(1, 3);
        refs.set_cell_dependencies(
            b1.clone(),
            [GridDependency::DynamicRequest("k".to_string())],
        )
        .unwrap();
        refs.set_cell_dependencies(c1.clone(), [GridDependency::Cell(b1.clone())])
            .unwrap();

        let claim = cell_claim("k", None, Some(cell_identity("k", rect(5, 5, 5, 5))));
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(consequence.status, DynamicRebindStatus::Activated);
        assert_eq!(
            consequence.dirty_closure,
            refs.dirty_closure_for_dynamic_request("k")
        );
        assert_eq!(consequence.dirty_closure, set([b1, c1]));
        assert_eq!(
            consequence.invalidation_reasons,
            reasons([InvalidationReasonKind::DynamicDependencyActivated])
        );
        assert_eq!(
            consequence.changed_dependency_kinds,
            kinds([DependencyDescriptorKind::DynamicPotential])
        );
        assert_eq!(consequence.error_effect, DynamicRebindErrorEffect::None);
        assert_eq!(
            consequence.structural_change,
            DynamicRebindStructuralChange::None
        );
    }

    #[test]
    fn cell_reference_preserving_is_empty() {
        let mut refs = GridInvalidationRef::new(bounds());
        let b1 = address(1, 2);
        refs.set_cell_dependencies(b1, [GridDependency::DynamicRequest("k".to_string())])
            .unwrap();

        let identity = cell_identity("k", rect(5, 5, 5, 5));
        let claim = cell_claim("k", Some(identity.clone()), Some(identity));
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(consequence.status, DynamicRebindStatus::ReferencePreserving);
        assert!(consequence.dirty_closure.is_empty());
        assert!(consequence.invalidation_reasons.is_empty());
        assert!(consequence.changed_dependency_kinds.is_empty());
        assert_eq!(consequence.error_effect, DynamicRebindErrorEffect::None);
    }

    #[test]
    fn cell_released_reports_ref_and_request_closure() {
        let mut refs = GridInvalidationRef::new(bounds());
        let b1 = address(1, 2);
        let c1 = address(1, 3);
        refs.set_cell_dependencies(
            b1.clone(),
            [GridDependency::DynamicRequest("k".to_string())],
        )
        .unwrap();
        refs.set_cell_dependencies(c1.clone(), [GridDependency::Cell(b1.clone())])
            .unwrap();

        let claim = cell_claim("k", Some(cell_identity("k", rect(5, 5, 5, 5))), None);
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(consequence.status, DynamicRebindStatus::Released);
        assert_eq!(consequence.error_effect, DynamicRebindErrorEffect::Ref);
        assert_eq!(
            consequence.dirty_closure,
            refs.dirty_closure_for_dynamic_request("k")
        );
        assert_eq!(consequence.dirty_closure, set([b1, c1]));
        assert_eq!(
            consequence.invalidation_reasons,
            reasons([InvalidationReasonKind::DynamicDependencyReleased])
        );
    }

    /// Build an invalidation ref with a spill-fact dependent on `anchor` plus a
    /// chained scalar dependent, and return both dependents for closure asserts.
    fn refs_with_spill_dependent(
        anchor: &ExcelGridCellAddress,
    ) -> (
        GridInvalidationRef,
        ExcelGridCellAddress,
        ExcelGridCellAddress,
    ) {
        let mut refs = GridInvalidationRef::new(bounds());
        let consumer = address(2, 1);
        let downstream = address(3, 1);
        refs.set_cell_dependencies(
            consumer.clone(),
            [GridDependency::SpillFact(GridSpillDependency::anchor(
                anchor.clone(),
            ))],
        )
        .unwrap();
        refs.set_cell_dependencies(downstream.clone(), [GridDependency::Cell(consumer.clone())])
            .unwrap();
        (refs, consumer, downstream)
    }

    #[test]
    fn spill_added_changes_with_anchor_closure_excluding_extent() {
        let anchor = address(1, 1);
        let (refs, consumer, downstream) = refs_with_spill_dependent(&anchor);
        // Extent cell that is NOT a spill-fact dependent of the anchor.
        let extent_cell = address(1, 2);

        let after = spill_snapshot(anchor.clone(), rect(1, 1, 1, 2), false, 1);
        let claim = spill_claim("k", None, Some(after));
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(
            claim.cause,
            DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::Added)
        );
        assert_eq!(consequence.status, DynamicRebindStatus::Changed);
        assert_eq!(
            consequence.structural_change,
            DynamicRebindStructuralChange::AnchorAdded
        );
        assert_eq!(
            consequence.dirty_closure,
            refs.dirty_closure_for_spill_fact(GridSpillDependency::anchor(anchor.clone()))
                .unwrap()
        );
        assert_eq!(consequence.dirty_closure, set([consumer, downstream]));
        assert!(!consequence.dirty_closure.contains(&extent_cell));
        assert_eq!(
            consequence.invalidation_reasons,
            reasons([
                InvalidationReasonKind::DependencyAdded,
                InvalidationReasonKind::DynamicDependencyActivated
            ])
        );
    }

    #[test]
    fn spill_removed_releases_with_anchor_closure() {
        let anchor = address(1, 1);
        let (refs, consumer, downstream) = refs_with_spill_dependent(&anchor);

        let before = spill_snapshot(anchor.clone(), rect(1, 1, 1, 2), false, 1);
        let claim = spill_claim("k", Some(before), None);
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(
            claim.cause,
            DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::Removed)
        );
        assert_eq!(consequence.status, DynamicRebindStatus::Released);
        assert_eq!(consequence.error_effect, DynamicRebindErrorEffect::Ref);
        assert_eq!(
            consequence.structural_change,
            DynamicRebindStructuralChange::AnchorRemoved
        );
        assert_eq!(consequence.dirty_closure, set([consumer, downstream]));
    }

    #[test]
    fn spill_extent_changed_reclassifies_with_anchor_closure_excluding_new_extent() {
        let anchor = address(1, 1);
        let (refs, consumer, downstream) = refs_with_spill_dependent(&anchor);
        // Grown-into extent cell; not a dependent, so it must stay out of closure.
        let grown_cell = address(1, 3);

        let before = spill_snapshot(anchor.clone(), rect(1, 1, 1, 2), false, 1);
        let after = spill_snapshot(anchor.clone(), rect(1, 1, 1, 3), false, 1);
        let claim = spill_claim("k", Some(before), Some(after));
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(
            claim.cause,
            DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::ExtentChanged)
        );
        assert_eq!(consequence.status, DynamicRebindStatus::Reclassified);
        assert_eq!(consequence.dirty_closure, set([consumer, downstream]));
        assert!(!consequence.dirty_closure.contains(&grown_cell));
        assert_eq!(
            consequence.invalidation_reasons,
            reasons([
                InvalidationReasonKind::StructuralRecalcOnly,
                InvalidationReasonKind::DynamicDependencyReclassified
            ])
        );
    }

    #[test]
    fn spill_value_changed_reclassifies_with_anchor_closure() {
        let anchor = address(1, 1);
        let (refs, consumer, downstream) = refs_with_spill_dependent(&anchor);

        let before = spill_snapshot(anchor.clone(), rect(1, 1, 1, 2), false, 1);
        let after = spill_snapshot(anchor.clone(), rect(1, 1, 1, 2), false, 2);
        let claim = spill_claim("k", Some(before), Some(after));
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(
            claim.cause,
            DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::ValueChanged)
        );
        assert_eq!(consequence.status, DynamicRebindStatus::Reclassified);
        assert_eq!(consequence.dirty_closure, set([consumer, downstream]));
        assert_eq!(
            consequence.invalidation_reasons,
            reasons([InvalidationReasonKind::UpstreamPublication])
        );
    }

    #[test]
    fn spill_blocked_changed_reports_spill_effect() {
        let anchor = address(1, 1);
        let (refs, consumer, downstream) = refs_with_spill_dependent(&anchor);

        let before = spill_snapshot(anchor.clone(), rect(1, 1, 1, 2), false, 1);
        let after = spill_snapshot(anchor.clone(), rect(1, 1, 1, 2), true, 1);
        let claim = spill_claim("k", Some(before), Some(after));
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(
            claim.cause,
            DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::BlockedChanged)
        );
        assert_eq!(consequence.status, DynamicRebindStatus::Reclassified);
        assert_eq!(consequence.error_effect, DynamicRebindErrorEffect::Spill);
        assert_eq!(consequence.dirty_closure, set([consumer, downstream]));
        assert_eq!(
            consequence.invalidation_reasons,
            reasons([
                InvalidationReasonKind::StructuralRebindRequired,
                InvalidationReasonKind::DynamicDependencyReclassified
            ])
        );
    }

    #[test]
    fn spill_reference_preserving_is_empty() {
        let anchor = address(1, 1);
        let (refs, _consumer, _downstream) = refs_with_spill_dependent(&anchor);

        let snapshot = spill_snapshot(anchor.clone(), rect(1, 1, 1, 2), false, 1);
        let claim = spill_claim("k", Some(snapshot.clone()), Some(snapshot));
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(consequence.status, DynamicRebindStatus::ReferencePreserving);
        assert!(consequence.dirty_closure.is_empty());
        assert!(consequence.invalidation_reasons.is_empty());
        assert!(consequence.changed_dependency_kinds.is_empty());
    }

    #[test]
    fn spill_extent_and_value_changed_reclassifies_with_full_reasons() {
        let anchor = address(1, 1);
        let (refs, consumer, downstream) = refs_with_spill_dependent(&anchor);

        // Extent R1C1:R1C2 -> R1C1:R1C3 AND value_epoch 1 -> 2 in one recalc.
        let before = spill_snapshot(anchor.clone(), rect(1, 1, 1, 2), false, 1);
        let after = spill_snapshot(anchor.clone(), rect(1, 1, 1, 3), false, 2);
        let claim = spill_claim("k", Some(before), Some(after));
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(
            claim.cause,
            DynamicRebindCause::SpillEpochChanged(GridSpillEpochChangeKind::ExtentAndValueChanged)
        );
        assert_eq!(consequence.status, DynamicRebindStatus::Reclassified);
        assert_eq!(
            consequence.structural_change,
            DynamicRebindStructuralChange::None
        );
        assert_eq!(consequence.dirty_closure, set([consumer, downstream]));
        assert_eq!(
            consequence.invalidation_reasons,
            reasons([
                InvalidationReasonKind::StructuralRecalcOnly,
                InvalidationReasonKind::DynamicDependencyReclassified
            ])
        );
        assert_eq!(
            consequence.changed_dependency_kinds,
            kinds([
                DependencyDescriptorKind::DynamicPotential,
                DependencyDescriptorKind::ShapeTopology
            ])
        );
    }

    #[test]
    fn spill_distinct_before_after_anchors_union_both_closures() {
        // before anchor A1 (R1C1), after anchor A2 (R1C2): the resolver must seed
        // the closure from BOTH anchors' spill-fact dependents (the genuine
        // old-union-new over two distinct anchors). The extent also changes, so
        // spill_epoch_change_kind (which ignores the anchor) yields ExtentChanged,
        // keeping the change non-reference-preserving and exercising the union loop.
        let before_anchor = address(1, 1);
        let after_anchor = address(1, 2);
        let mut refs = GridInvalidationRef::new(bounds());
        let before_consumer = address(4, 1);
        let after_consumer = address(4, 2);
        refs.set_cell_dependencies(
            before_consumer.clone(),
            [GridDependency::SpillFact(GridSpillDependency::anchor(
                before_anchor.clone(),
            ))],
        )
        .unwrap();
        refs.set_cell_dependencies(
            after_consumer.clone(),
            [GridDependency::SpillFact(GridSpillDependency::anchor(
                after_anchor.clone(),
            ))],
        )
        .unwrap();

        let before = spill_snapshot(before_anchor.clone(), rect(1, 1, 1, 2), false, 1);
        let after = spill_snapshot(after_anchor.clone(), rect(1, 2, 1, 4), false, 1);
        let claim = spill_claim("k", Some(before), Some(after));
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        let mut expected = refs
            .dirty_closure_for_spill_fact(GridSpillDependency::anchor(before_anchor))
            .unwrap();
        expected.extend(
            refs.dirty_closure_for_spill_fact(GridSpillDependency::anchor(after_anchor))
                .unwrap(),
        );
        assert_eq!(consequence.dirty_closure, expected);
        assert_eq!(
            consequence.dirty_closure,
            set([before_consumer, after_consumer])
        );
    }

    #[test]
    fn table_status_maps_classifier_status() {
        assert_eq!(
            table_status_to_rebind_status(TreeCalcDynamicTableRebindStatus::ReferencePreserving),
            DynamicRebindStatus::ReferencePreserving
        );
        assert_eq!(
            table_status_to_rebind_status(TreeCalcDynamicTableRebindStatus::RebindRequired),
            DynamicRebindStatus::Reclassified
        );
        assert_eq!(
            table_status_to_rebind_status(TreeCalcDynamicTableRebindStatus::DeletedTarget),
            DynamicRebindStatus::Released
        );
        assert_eq!(
            table_status_to_rebind_status(TreeCalcDynamicTableRebindStatus::UnavailableTarget),
            DynamicRebindStatus::Released
        );
        assert_eq!(
            table_status_to_rebind_status(TreeCalcDynamicTableRebindStatus::TypedExclusion),
            DynamicRebindStatus::Excluded
        );
    }

    /// The reasons/changed-kinds round-trip the classifier report verbatim across
    /// every representative cause — this is the whole CTRO-2 invariant.
    #[test]
    fn table_lifecycle_round_trips_classifier_reasons_and_kinds() {
        let cases = [
            (
                TreeCalcDynamicTableRebindCause::TableLifecycle(
                    TreeCalcTableUpdateScenarioKind::TableRename,
                ),
                TreeCalcDynamicTableReferenceTargetKind::Table,
                Some(("Sales", "tree-table:sales:v1")),
                Some(("SalesRenamed", "tree-table:sales:v2")),
                DynamicRebindStatus::Reclassified,
            ),
            (
                TreeCalcDynamicTableRebindCause::TableLifecycle(
                    TreeCalcTableUpdateScenarioKind::TableDelete,
                ),
                TreeCalcDynamicTableReferenceTargetKind::Table,
                Some(("Sales", "tree-table:sales:v1")),
                None,
                DynamicRebindStatus::Released,
            ),
            (
                TreeCalcDynamicTableRebindCause::TableLifecycle(
                    TreeCalcTableUpdateScenarioKind::WorkspaceClose,
                ),
                TreeCalcDynamicTableReferenceTargetKind::CrossWorkspaceTable,
                Some(("Sales", "tree-table:sales:v1")),
                None,
                DynamicRebindStatus::Released,
            ),
            (
                TreeCalcDynamicTableRebindCause::DynamicTargetNotTable,
                TreeCalcDynamicTableReferenceTargetKind::Table,
                Some(("Sales", "tree-table:sales:v1")),
                Some(("Sales", "tree-table:sales:v1")),
                DynamicRebindStatus::Excluded,
            ),
            (
                TreeCalcDynamicTableRebindCause::TableLifecycle(
                    TreeCalcTableUpdateScenarioKind::SaveReopen,
                ),
                TreeCalcDynamicTableReferenceTargetKind::Table,
                Some(("Sales", "tree-table:sales:v1")),
                Some(("Sales", "tree-table:sales:v1")),
                DynamicRebindStatus::ReferencePreserving,
            ),
        ];

        for (cause, target_kind, before, after, expected_status) in cases {
            let refs = GridInvalidationRef::new(bounds());
            let claim = table_claim(cause.clone(), target_kind, before, after);
            let report =
                classify_treecalc_dynamic_table_rebind(claim.table_request.as_ref().unwrap());
            let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

            assert_eq!(consequence.status, expected_status, "{cause:?}");
            assert_eq!(
                consequence.invalidation_reasons, report.invalidation_reasons,
                "reasons round-trip for {cause:?}"
            );
            assert_eq!(
                consequence.changed_dependency_kinds, report.changed_dependency_kinds,
                "kinds round-trip for {cause:?}"
            );
            assert_eq!(
                consequence.table_report.as_ref(),
                Some(&report),
                "{cause:?}"
            );
            // A non-preserving rebind must carry non-empty reasons/kinds (the
            // classifier clears them only for ReferencePreserving), so an
            // empty-where-non-empty classifier regression cannot slip through as
            // empty == empty.
            if expected_status != DynamicRebindStatus::ReferencePreserving {
                assert!(
                    !consequence.invalidation_reasons.is_empty(),
                    "non-empty reasons for {cause:?}"
                );
                assert!(
                    !consequence.changed_dependency_kinds.is_empty(),
                    "non-empty kinds for {cause:?}"
                );
            }
        }
    }

    #[test]
    fn table_rebind_required_closure_parity_with_dirty_closure_for_table() {
        // Non-rename RebindRequired (selector retarget that lands on the same
        // resolved name): the closure is exactly dirty_closure_for_table(name).
        let (refs, consumer, downstream) = refs_with_table_dependent("Sales");
        let claim = table_claim(
            TreeCalcDynamicTableRebindCause::SelectorTextChanged,
            TreeCalcDynamicTableReferenceTargetKind::Table,
            Some(("Sales", "tree-table:sales:v1")),
            Some(("Sales", "tree-table:sales:v2")),
        );
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(consequence.status, DynamicRebindStatus::Reclassified);
        assert_eq!(consequence.error_effect, DynamicRebindErrorEffect::None);
        assert_eq!(
            consequence.dirty_closure,
            refs.dirty_closure_for_table("Sales").unwrap()
        );
        assert_eq!(consequence.dirty_closure, set([consumer, downstream]));
    }

    #[test]
    fn table_rename_unions_old_and_new_closures_and_changes_key() {
        // Rename Sales -> SalesRenamed: both old and new resolved tables have
        // dependents that must be dirtied (old ∪ new), and the structural change
        // is a key change.
        let mut refs = GridInvalidationRef::new(bounds());
        let old_consumer = address(2, 1);
        let new_consumer = address(4, 1);
        refs.set_cell_dependencies(
            old_consumer.clone(),
            [GridDependency::Table(
                GridTableDependency::new("Sales", rect(1, 1, 2, 2), bounds()).unwrap(),
            )],
        )
        .unwrap();
        refs.set_cell_dependencies(
            new_consumer.clone(),
            [GridDependency::Table(
                GridTableDependency::new("SalesRenamed", rect(3, 1, 4, 2), bounds()).unwrap(),
            )],
        )
        .unwrap();

        let claim = table_claim(
            TreeCalcDynamicTableRebindCause::TableLifecycle(
                TreeCalcTableUpdateScenarioKind::TableRename,
            ),
            TreeCalcDynamicTableReferenceTargetKind::Table,
            Some(("Sales", "tree-table:sales:v1")),
            Some(("SalesRenamed", "tree-table:sales:v2")),
        );
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        let mut expected = refs.dirty_closure_for_table("SalesRenamed").unwrap();
        expected.extend(refs.dirty_closure_for_table("Sales").unwrap());
        assert_eq!(consequence.dirty_closure, expected);
        assert_eq!(consequence.dirty_closure, set([old_consumer, new_consumer]));
        assert_eq!(
            consequence.structural_change,
            DynamicRebindStructuralChange::TableKeyChanged
        );
    }

    #[test]
    fn table_reference_preserving_is_empty_with_empty_reasons() {
        // SaveReopen with an unchanged resolved identity preserves the reference:
        // the classifier clears its own reasons/kinds, and the feeder dirties
        // nothing even though a table dependent exists.
        let (refs, _consumer, _downstream) = refs_with_table_dependent("Sales");
        let claim = table_claim(
            TreeCalcDynamicTableRebindCause::TableLifecycle(
                TreeCalcTableUpdateScenarioKind::SaveReopen,
            ),
            TreeCalcDynamicTableReferenceTargetKind::Table,
            Some(("Sales", "tree-table:sales:v1")),
            Some(("Sales", "tree-table:sales:v1")),
        );
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(consequence.status, DynamicRebindStatus::ReferencePreserving);
        assert!(consequence.dirty_closure.is_empty());
        assert!(consequence.invalidation_reasons.is_empty());
        assert!(consequence.changed_dependency_kinds.is_empty());
        assert_eq!(consequence.error_effect, DynamicRebindErrorEffect::None);
    }

    #[test]
    fn table_deleted_and_unavailable_dirty_old_dependents_with_ref() {
        // A deleted / workspace-unavailable target releases the selector to #REF!,
        // and the OLD resolved table's dependents must recompute - exactly the set
        // delete_table dirties (the old table key, computed before its edges are
        // purged). The closure is dirty_closure_for_table(old_name), NOT empty.
        let (refs, consumer, downstream) = refs_with_table_dependent("Sales");
        let expected = refs.dirty_closure_for_table("Sales").unwrap();
        assert_eq!(expected, set([consumer, downstream]));

        let deleted = resolve_dynamic_rebind_claim(
            &refs,
            &table_claim(
                TreeCalcDynamicTableRebindCause::TableLifecycle(
                    TreeCalcTableUpdateScenarioKind::TableDelete,
                ),
                TreeCalcDynamicTableReferenceTargetKind::Table,
                Some(("Sales", "tree-table:sales:v1")),
                None,
            ),
        )
        .unwrap();
        assert_eq!(deleted.status, DynamicRebindStatus::Released);
        assert_eq!(deleted.error_effect, DynamicRebindErrorEffect::Ref);
        assert_eq!(deleted.dirty_closure, expected);

        let unavailable = resolve_dynamic_rebind_claim(
            &refs,
            &table_claim(
                TreeCalcDynamicTableRebindCause::TableLifecycle(
                    TreeCalcTableUpdateScenarioKind::WorkspaceClose,
                ),
                TreeCalcDynamicTableReferenceTargetKind::CrossWorkspaceTable,
                Some(("Sales", "tree-table:sales:v1")),
                None,
            ),
        )
        .unwrap();
        assert_eq!(unavailable.status, DynamicRebindStatus::Released);
        assert_eq!(unavailable.error_effect, DynamicRebindErrorEffect::Ref);
        assert_eq!(unavailable.dirty_closure, expected);
    }

    /// TypedExclusion vs ReferencePreserving: BOTH dirty nothing and both flag a
    /// distinct error/status, but a typed exclusion RETAINS the classifier's
    /// invalidation reasons/changed-kinds (it only clears the report-local
    /// `dependency_fact_kinds`), whereas a reference-preserving rebind yields
    /// empty reasons/kinds. Pin that difference plus the #REF! effect.
    #[test]
    fn table_typed_exclusion_retains_reasons_but_flags_ref() {
        let refs = GridInvalidationRef::new(bounds());
        let claim = table_claim(
            TreeCalcDynamicTableRebindCause::DynamicTargetNotTable,
            TreeCalcDynamicTableReferenceTargetKind::Table,
            Some(("Sales", "tree-table:sales:v1")),
            Some(("Sales", "tree-table:sales:v1")),
        );
        let report = classify_treecalc_dynamic_table_rebind(claim.table_request.as_ref().unwrap());
        let consequence = resolve_dynamic_rebind_claim(&refs, &claim).unwrap();

        assert_eq!(consequence.status, DynamicRebindStatus::Excluded);
        assert_eq!(consequence.error_effect, DynamicRebindErrorEffect::Ref);
        assert!(consequence.dirty_closure.is_empty());
        // RETAINED verbatim from the report (NOT cleared like ReferencePreserving).
        assert_eq!(
            consequence.invalidation_reasons,
            report.invalidation_reasons
        );
        assert_eq!(
            consequence.changed_dependency_kinds,
            report.changed_dependency_kinds
        );
        assert!(!consequence.invalidation_reasons.is_empty());
        assert!(!consequence.changed_dependency_kinds.is_empty());
        // The classifier did clear its report-local dependency_fact_kinds.
        assert!(
            consequence
                .table_report
                .as_ref()
                .unwrap()
                .dependency_fact_kinds
                .is_empty()
        );
    }
}
