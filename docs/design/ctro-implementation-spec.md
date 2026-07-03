# Historical CTRO implementation spec (removed rebind.rs claim lifecycle)

Source: ultracode investigation/taxonomy/design workflow. This document is retained as historical
claim-lifecycle design material. It no longer describes live grid-machine code: `grid/machine/rebind.rs`
has been removed, and the active CTRO direction is the ideal graph model documented in
`ctro-recalc-integration-design.md`. For the current parked product status, readiness checklist, and
remaining refinements, start with `ctro-parked-rework-status.md`.

The useful residue here is conceptual: resolved identity comparison, dirty-closure thinking,
error-effect routing, and the CTRO-3 differential question. The active implementation is now
structural dependencies plus calc-overlay dependencies plus runtime dependency traces plus publication
deltas consumed by effective-graph recalc. **Do not use the term "production ready" anywhere.**

## Module location (architectural correction to the plan)
The plan names `src/rebind.rs` (crate root). That **cannot** work: every load-bearing closure path is
`pub(super)`/private inside `grid/machine/*` and the modules share types via `use super::*`. So:

- File: **`grid/machine/rebind.rs`**, opening with `use super::*;`.
- In `grid/machine.rs`: add `mod rebind;` after `mod spill_ledger;`, and `pub use rebind::*;` after
  `pub use spill_ledger::*;` (keep alphabetical-ish ordering consistent with the file's existing style).
- rebind.rs additionally imports the crate-root classifier + dependency enums:
  ```rust
  use crate::dependency::{DependencyDescriptorKind, InvalidationReasonKind};
  use crate::structured_table::{
      classify_treecalc_dynamic_table_rebind, TreeCalcDynamicTableReferenceTargetKind,
      TreeCalcDynamicTableRebindCause, TreeCalcDynamicTableRebindReport,
      TreeCalcDynamicTableRebindRequest, TreeCalcDynamicTableRebindStatus,
  };
  ```
  (Verify the exact exported names in structured_table.rs before importing; adjust if drifted.)

## Verified fold-target signatures (already confirmed in the tree)
- `GridInvalidationRef::dirty_closure_for_dynamic_request(&self, request_key: &str) -> BTreeSet<ExcelGridCellAddress>` (invalidation.rs:1068)
- `GridInvalidationRef::dirty_closure_for_spill_fact(&self, dependency: GridSpillDependency) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError>` (invalidation.rs:1080; checks address, seeds from `spill_dependents_by_anchor[anchor]`)
- `GridInvalidationRef::dirty_closure_for_table(&self, ...)` (invalidation.rs:1210 — verify exact signature for CTRO-2)
- `GridSpillDependency { pub anchor: ExcelGridCellAddress }` + `GridSpillDependency::anchor(anchor) -> Self` (invalidation.rs:90/96)
- `GridSpillEpochSnapshot { pub anchor: ExcelGridCellAddress, pub extent: GridRect, pub blocked: bool, pub value_epoch: u64 }` + `::new(fact: GridSpillFact, value_epoch: u64)` (spill_ledger.rs:17)
- `GridSpillEpochChangeKind { Added, Removed, ExtentChanged, ValueChanged, BlockedChanged, ExtentAndValueChanged }` (spill_ledger.rs:150)
- `spill_epoch_change_kind(old: Option<&GridSpillEpochSnapshot>, new: Option<&GridSpillEpochSnapshot>) -> Option<GridSpillEpochChangeKind>` (optimized_sheet.rs:4148, `pub(super)`, reachable via `use super::*`)
- `GridDependency` enum (invalidation.rs:156): `Cell, Range, Name, Table(GridTableDependency), SpillFact(GridSpillDependency), SpillBlocker, AxisVisibility, AxisValue, DynamicRequest(String)`
- `InvalidationReasonKind` (dependency.rs:257): `StructuralRebindRequired, StructuralRecalcOnly, UpstreamPublication, ExternallyInvalidated, TreeReference*..., StructuredTable*..., DependencyAdded, DependencyRemoved, DependencyReclassified, DynamicDependencyActivated, DynamicDependencyReleased, DynamicDependencyReclassified` (Copy, Ord, Hash)
- `DependencyDescriptorKind` (dependency.rs:13): `StaticDirect, ..., DynamicPotential, HostSensitive, CapabilitySensitive, ShapeTopology, Unresolved` (Copy, Ord, Hash)

## Types (all declared in CTRO-1; table Option fields stay None until CTRO-2)
```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResolvedReferenceIdentity {
    Cell { request_key: String, target: GridRect },
    Spill(GridSpillEpochSnapshot),
    Table { table_key: String, resolved_identity: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicRebindFamily { CellDynamicRequest, SpillAnchorRef, StructuredTableRebind }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynamicRebindCause {
    IdentityActivated, IdentityReclassified, IdentityReleased, IdentityPreserved,
    SpillEpochChanged(GridSpillEpochChangeKind),
    Table(TreeCalcDynamicTableRebindCause),
    AxisEditDuringRebind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicRebindStatus {
    Activated, Reclassified, Released, ReferencePreserving, Changed, Error, Excluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicRebindErrorEffect { None, Ref, Spill, NameOrValue }

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicRebindStructuralChange {
    None, AnchorAddressTransformed, AnchorAdded, AnchorRemoved,
    TableExtentChanged, TableKeyChanged, WorkspaceAvailabilityChanged, ReverseIndexRebuilt,
}

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

// CTRO-3 (declare in CTRO-3, not CTRO-1):
pub struct GridDynamicRebindMismatch { claim_id, family, dirty_closure_equal, status_equal,
    invalidation_reasons_equal, changed_dependency_kinds_equal, error_effect_equal,
    structural_change_equal, incremental: DynamicRebindConsequence, brute_force: DynamicRebindConsequence }
pub struct DynamicRebindReport { claims_compared: usize, consequences: Vec<DynamicRebindConsequence>,
    mismatches: Vec<GridDynamicRebindMismatch> }
```

## cause → reason table (verified variant names; cell+spill only; table copies report verbatim)
| cause | invalidation_reasons | changed_dependency_kinds |
|---|---|---|
| IdentityActivated | {DynamicDependencyActivated} | {DynamicPotential} |
| IdentityReclassified | {DynamicDependencyReclassified} | {DynamicPotential} |
| IdentityReleased | {DynamicDependencyReleased} | {DynamicPotential} |
| IdentityPreserved | {} | {} |
| SpillEpochChanged(Added) | {DependencyAdded, DynamicDependencyActivated} | {DynamicPotential} |
| SpillEpochChanged(Removed) | {DependencyRemoved, DynamicDependencyReleased} | {DynamicPotential} |
| SpillEpochChanged(ExtentChanged) and (ExtentAndValueChanged) | {StructuralRecalcOnly, DynamicDependencyReclassified} | {DynamicPotential, ShapeTopology} |
| SpillEpochChanged(ValueChanged) | {UpstreamPublication} | {DynamicPotential} |
| SpillEpochChanged(BlockedChanged) | {StructuralRebindRequired, DynamicDependencyReclassified} | {DynamicPotential} |
| AxisEditDuringRebind | {StructuralRebindRequired} | {DynamicPotential} |
| Table(_) | unreachable in this table — assert; table feeder copies report fields verbatim | — |

## Functions
### classify_identity_transition(before: Option<&RRI>, after: Option<&RRI>) -> (DynamicRebindStatus, DynamicRebindCause)
- (None, Some) → (Activated, IdentityActivated)
- (Some, None) → (Released, IdentityReleased)
- (Some(X), Some(Y)) X!=Y → (Reclassified, IdentityReclassified)
- (Some(X), Some(X)) → (ReferencePreserving, IdentityPreserved)
- (None, None) → (ReferencePreserving, IdentityPreserved) [unreachable for a live claim]

### resolve_cell_dynamic_request_claim(refs, claim) -> Result<Consequence>
1. (status, cause) = classify_identity_transition(before, after).
2. If IdentityPreserved/ReferencePreserving: empty closure, empty reasons/kinds, error_effect None, structural_change None.
3. Else dirty_closure = refs.dirty_closure_for_dynamic_request(&claim.request_key) (the EXACT pre-existing closure; one key holds old∪new because reclassification is captured in identity not key).
4. (reasons, kinds) = cause_to_reason_table(CellDynamicRequest, &cause).
5. error_effect = Ref if status==Released else None. structural_change = None.

### resolve_spill_anchor_claim(refs, claim) -> Result<Consequence>
1. before/after = extract GridSpillEpochSnapshot from ResolvedReferenceIdentity::Spill (Option).
2. kind = spill_epoch_change_kind(before.as_ref(), after.as_ref()); if None → ReferencePreserving (empty closure, empty reasons), done.
3. cause = SpillEpochChanged(kind).
4. dirty_closure = union over anchor in {before.anchor} ∪ {after.anchor} of refs.dirty_closure_for_spill_fact(GridSpillDependency::anchor(anchor))? — seed by ANCHOR, never extent cells.
5. status: Added→Changed, Removed→Released, else Reclassified.
6. (reasons,kinds)=cause_to_reason_table(SpillAnchorRef,&cause).
7. error_effect: after.blocked==true → Spill; status==Released → Ref; else None.
8. structural_change: Added→AnchorAdded, Removed→AnchorRemoved, else None.
- Propagate GridRefError from dirty_closure_for_spill_fact (out-of-bounds anchor).

### resolve_dynamic_rebind_claim(refs, claim) -> Result<Consequence>  [pub]
match claim.family { CellDynamicRequest → resolve_cell..., SpillAnchorRef → resolve_spill...,
StructuredTableRebind → resolve_structured_table_claim (CTRO-2; CTRO-1 stub: `unimplemented!("CTRO-2")`) }

### CTRO-2: resolve_structured_table_claim, table_status_to_rebind_status (see design)
### CTRO-3: reference_resolve_dynamic_rebind, compare_dynamic_rebind_consequences, run_dynamic_rebind_differential,
   GridDifferentialRunReport.dynamic_rebind field, run_engine_mode_with_oxfml Both-arm hook.

## CTRO-1 test requirements (Tier-B unit tests in rebind.rs `#[cfg(test)] mod tests`)
Model construction on the existing test `grid_invalidation_ref_classifies_dynamic_dependencies_separately`
in invalidation.rs (find it; reuse its `GridInvalidationRef::new(bounds)` + `set_cell_dependencies` idiom).
Cover:
1. classify_identity_transition: all 5 transitions → correct (status, cause).
2. cause_to_reason_table: each cell + spill cause → exactly the table above (assert the BTreeSets).
3. Cell Activated: seed DynamicRequest("k") on B1, chain Cell(B1) on C1; claim before None after Some;
   assert consequence.dirty_closure == refs.dirty_closure_for_dynamic_request("k") (set ==), status Activated,
   reasons {DynamicDependencyActivated}, kinds {DynamicPotential}, error None.
4. Cell ReferencePreserving: before==after → empty closure + empty reasons/kinds, error None.
5. Cell Released: before Some after None → status Released, error_effect Ref, closure == dirty_closure_for_dynamic_request.
6. Spill Added/Removed/ExtentChanged/ValueChanged/BlockedChanged: build before/after snapshots that produce
   each kind via spill_epoch_change_kind; seed a SpillFact dependent on the anchor; assert cause SpillEpochChanged(kind),
   status (Added→Changed, Removed→Released, else Reclassified), closure == dirty_closure_for_spill_fact(anchor),
   blocked after → error_effect Spill; assert NEW/SHRUNK extent cells are NOT in the closure (seed-by-anchor).
7. Spill ReferencePreserving: before==after snapshot → kind None → empty closure.

## CTRO-2 — structured-table feeder (verified API)
Replace the `unimplemented!("CTRO-2…")` arm in `resolve_dynamic_rebind_claim` with `resolve_structured_table_claim`.

Verified types (structured_table.rs):
- `classify_treecalc_dynamic_table_rebind(request: &TreeCalcDynamicTableRebindRequest) -> TreeCalcDynamicTableRebindReport` (:6646)
- `TreeCalcDynamicTableRebindStatus { ReferencePreserving, RebindRequired, DeletedTarget, UnavailableTarget, TypedExclusion }` (:6592)
- `TreeCalcDynamicTableRebindRequest { selector_handle, selector_identity, source_reference_handle: Option<String>, target_kind, cause, before_resolved_table_identity: Option<String>, after_resolved_table_identity: Option<String>, caller_context_id: Option<String>, context_versions, oxfml_structured_bind_packet_available }` (:6613)
- `TreeCalcDynamicTableRebindReport { ..., status, dependency_fact_kinds, changed_dependency_kinds: BTreeSet<DependencyDescriptorKind>, invalidation_reasons: BTreeSet<InvalidationReasonKind>, diagnostics, oxfunc_opaque_reference_admitted, ... }` (:6627)
- `GridInvalidationRef::dirty_closure_for_table(&self, table_name: impl AsRef<str>) -> Result<BTreeSet<ExcelGridCellAddress>, GridRefError>` (invalidation.rs:1210) — takes the table NAME, derives the key via `excel_grid_table_name_key`, seeds `table_dependents_by_key`.
- Find and study the existing test `dynamic_table_rebind_covers_lifecycle_workspace_and_typed_exclusions` (structured_table.rs) for how to construct a `TreeCalcDynamicTableRebindRequest` (incl. `TreeCalcDynamicTableRebindCause`, `TreeCalcDynamicTableReferenceTargetKind`, `TreeCalcTableLifecycleContextVersions`) — reuse its idiom.

`table_status_to_rebind_status(TreeCalcDynamicTableRebindStatus) -> DynamicRebindStatus`:
ReferencePreserving→ReferencePreserving; RebindRequired→Reclassified; DeletedTarget→Released; UnavailableTarget→Released; TypedExclusion→Excluded.

`resolve_structured_table_claim(refs, claim)`:
1. `report = classify_treecalc_dynamic_table_rebind(claim.table_request.as_ref().expect("table claim carries table_request"))`.
2. `status = table_status_to_rebind_status(report.status)`.
3. `invalidation_reasons = report.invalidation_reasons.clone()`, `changed_dependency_kinds = report.changed_dependency_kinds.clone()` — VERBATIM (CTRO-2 round-trip invariant). Do NOT route through cause_to_reason_table.
4. dirty_closure:
   - `ReferencePreserving` → empty.
   - `Excluded` (TypedExclusion) or `Released` (Deleted/Unavailable, i.e. resolved target gone) → empty (#REF!, no body to dirty downstream of a missing table).
   - else (RebindRequired) → seed `dirty_closure_for_table(after_name)?`, and when the resolved table changed (rename / selector / dynamic-fn result, i.e. before_name != after_name and both Some) UNION `dirty_closure_for_table(before_name)?` (old∪new).
   - **OPEN QUESTION to resolve by reading the code:** how the claim conveys the table NAME(s) for `dirty_closure_for_table`. The report's `*_resolved_table_identity` are identity strings; `dirty_closure_for_table` wants the human table name. Decide cleanly (e.g. carry before/after table names on the claim, or derive them) and document the choice; the tests must seed a `GridDependency::Table(GridTableDependency::new(name, extent, bounds))` and assert the feeder's closure equals `dirty_closure_for_table(name)`.
5. error_effect: `Excluded` / `Released` (Deleted/Unavailable/TypedExclusion/DynamicTargetNotTable) → `Ref`; else `None`. **Never `Spill`** — table-grows-into-spill is modelled as a SEPARATE `SpillAnchorRef(BlockedChanged)` claim, not merged into the table consequence.
6. structural_change: rename → `TableKeyChanged`; resize/row/column lifecycle → `TableExtentChanged`; workspace open/close → `WorkspaceAvailabilityChanged`; else `None`. (Derive from `claim.table_request.cause` / report.)
7. `table_report = Some(report)`.

CTRO-2 invariant: consequence.{status-source}/invalidation_reasons/changed_dependency_kinds round-trip the
classifier exactly (assert consequence.invalidation_reasons == report.invalidation_reasons etc.); dirty_closure
== dirty_closure_for_table closure (old∪new on rename); ReferencePreserving → empty closure AND the classifier
already yields its reasons/kinds; TypedExclusion → empty closure + error Ref but reasons/kinds RETAINED (it only
clears dependency_fact_kinds, which lives on the report, not on the consequence's changed_dependency_kinds).
CTRO-2 tests: a table-lifecycle round-trip across representative causes (RebindRequired, DeletedTarget,
UnavailableTarget, TypedExclusion, ReferencePreserving) asserting reasons/kinds == report; closure parity vs
dirty_closure_for_table; rename → union(old,new) + TableKeyChanged; ReferencePreserving/Deleted/Unavailable/
TypedExclusion empty-closure + correct error_effect.

## Per-bead invariant + gate
- CTRO-1: cell closure == dirty_closure_for_dynamic_request; spill closure == dirty_closure_for_spill_fact over old∪new anchor;
  identity-changed→Reclassified else Activated/Released; ReferencePreserving→empty closure+reasons.
- Gate each bead: `cargo build -p oxcalc-core` clean (no warnings), `cargo test -p oxcalc-core` green,
  `cargo fmt -p oxcalc-core`. Trunk-based commit on main; trailer `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.
