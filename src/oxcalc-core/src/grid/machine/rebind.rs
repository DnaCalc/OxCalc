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
    TreeCalcDynamicTableRebindRequest, TreeCalcDynamicTableReferenceTargetKind,
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

/// Resolve a dynamic-rebind claim into its consequence by dispatching on family.
///
/// The structured-table feeder is wired in CTRO-2; until then a structured-table
/// claim is rejected rather than silently mis-resolved.
pub fn resolve_dynamic_rebind_claim(
    refs: &GridInvalidationRef,
    claim: &DynamicRebindClaim,
) -> Result<DynamicRebindConsequence, GridRefError> {
    match claim.family {
        DynamicRebindFamily::CellDynamicRequest => resolve_cell_dynamic_request_claim(refs, claim),
        DynamicRebindFamily::SpillAnchorRef => resolve_spill_anchor_claim(refs, claim),
        DynamicRebindFamily::StructuredTableRebind => {
            unimplemented!("CTRO-2: structured-table feeder")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
