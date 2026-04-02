# W029: TreeCalc Runtime-Derived Effects and Overlay Closure

## Purpose
Make dynamic dependency, capability-sensitive, execution-restriction-sensitive, and shape-sensitive runtime effects real in the live TreeCalc engine path rather than leaving them as proving-lane-only constructs.
This packet widens runtime-derived truth beneath the existing `OxCalcTree` host-facing consumer contract.

## Position and Dependencies
- **Depends on**: W027, W028
- **Blocks**: W030, W031
- **Cross-repo**: may justify a narrower handoff only if execution-restriction or runtime-derived effect transport is still too narrow for the live TreeCalc path

## Boundary With W026
W026 is the consumed-seam intake packet for the current TreeCalc-first runtime-derived floor.
It owns:
1. the current emitted family subset consumed now,
2. the ownership split between canonical OxFml families and OxCalc-local projection labels for that subset,
3. the direct host-facing and replay-facing reachability rule for that subset.

W029 begins after that floor.
It owns:
1. realizing additional emitted runtime-derived families,
2. widening overlay closure beneath the existing `OxCalcTree` contract,
3. hardening runtime-derived behavior across more live runtime states than the current W026 minimum.

This means W029 does not renegotiate the current W026 transport truth unless live implementation evidence proves the W026 floor insufficient or false.

## Scope
### In scope
1. dynamic dependency activation and release over the live TreeCalc path
2. capability-sensitive runtime-derived effects
3. execution-restriction-sensitive runtime-derived effects
4. shape-sensitive or topology-sensitive runtime-derived effects required by first-phase TreeCalc semantics
5. overlay closure so runtime-derived facts are explicit, replay-visible, and not hidden mutable truth

### Out of scope
1. broader display semantics
2. async or concurrent overlay strategy
3. broader grid or host program semantics outside first-phase TreeCalc scope
4. reopening the current W026 semantic minimum family split or direct reachability rule unless live evidence proves them insufficient

## Deliverables
1. runtime-derived effect handling in the Rust TreeCalc path with replay-visible state
2. explicit overlay rules for dynamic dependency and execution-sensitive facts
3. deterministic diagnostics or artifacts showing runtime-derived dependency changes and fallback behavior
4. narrowed decision on whether execution-restriction transport remains a seam blocker

## Gate Model
### Entry gate
- W028 has established real evaluator-backed candidate intake
- W027 has established the structural dependency and invalidation substrate
- W026 has locked the current consumed-now transport and reachability floor for the emitted TreeCalc-first runtime-derived subset
- the `OxCalcTree` consumer contract remains the host-facing contract, with runtime-derived effect closure still below that surface

### Exit gate
- runtime-derived facts that affect recalc or publication are explicit, replay-visible, and no longer proving-lane-only constructs
- overlay truth for in-scope runtime effects is explicit and deterministic
- any still-narrow execution-restriction seam issue is packetized explicitly rather than left implicit

## Pre-Closure Verification Checklist
1. Spec text and realization notes updated for all in-scope items: no
2. Pack expectations updated for affected packs: no
3. At least one deterministic replay artifact exists per in-scope behavior: no
4. Semantic-equivalence statement provided for policy or strategy changes: no
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: no
6. All required tests pass: no
7. No known semantic gaps remain in declared scope: no
8. Completion language audit passed: no
9. `IN_PROGRESS_FEATURE_WORKLIST.md` updated: no
10. `CURRENT_BLOCKERS.md` updated if needed: no

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the first live implementation slice now exists beneath the landed `OxCalcTree` host-facing contract: runtime effects carry an explicit family classification and the runtime-effect overlay projection now preserves dynamic-dependency versus execution-restriction distinction
  - W026 now owns the consumed-now transport and reachability floor for that subset; W029 still owns any wider family realization or broader overlay closure beneath it
  - host-facing replay or session widening has not been executed here yet
  - runtime-derived effect closure over the live TreeCalc path remains partial beyond the current dynamic-dependency versus execution-restriction subset
  - capability-sensitive and shape/topology-sensitive runtime handling are not realized yet
  - runtime-derived overlays on the published-success path are still narrower than the reject/fallback-side local floor
- claim_confidence: draft
- reviewed_inbound_observations: current OxFml seam baseline consumed; execution-restriction transport remains a watch lane, but the local TreeCalc floor now preserves the current dynamic-dependency versus execution-restriction distinction explicitly in runtime-effect and overlay projection
