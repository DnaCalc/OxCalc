# W029: TreeCalc Runtime-Derived Effects and Overlay Closure

## Purpose
Make dynamic dependency, capability-sensitive, execution-restriction-sensitive, and shape-sensitive runtime effects real in the live TreeCalc engine path rather than leaving them as proving-lane-only constructs.
This packet widens runtime-derived truth beneath the existing `OxCalcTree` host-facing consumer contract.

Terminology note:
1. `TreeCalc` in this workset means OxCalc's internal tree-substrate/runtime/reference preparation lane beneath the `OxCalcTree` contract.
2. It does not name the future separate `DNA TreeCalc` repo/product, although that product is the intended primary consumer of this lane.

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
Audit bead: `calc-k5i.7`.

1. Spec text and realization notes updated for all in-scope items: yes — this packet records the W029 realized floor beneath the `OxCalcTree` host-facing contract.
2. Pack expectations updated for affected packs: yes — TreeCalc fixture expectations now assert runtime-effect overlay kinds where in scope, and the local TreeCalc baseline was regenerated to 15 cases.
3. At least one deterministic replay artifact exists per in-scope behavior: yes — checked-in artifacts under `docs/test-runs/core-engine/treecalc-local/w025-treecalc-local-baseline/` cover dynamic dependency, execution restriction, capability-sensitive, shape/topology, publish, verified-clean, reject/no-publish, and post-edit overlay paths.
4. Semantic-equivalence statement provided for policy or strategy changes: yes — environment context and overlay-projection policy affect diagnostics and overlay sidecars only; candidate acceptance, reject/no-publish, and coordinator publication authority remain invariant for existing published and verified-clean paths.
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: yes — no new OxFml canonical seam change is required; W029 uses OxCalc-local TreeCalc projection labels while preserving OxFml ownership of formula-language semantics.
6. All required tests pass: yes — `cargo run -p oxcalc-tracecalc-cli -- treecalc w025-treecalc-local-baseline`, scoped OxCalc `cargo fmt -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `scripts/check-worksets.ps1` pass.
7. No known semantic gaps remain in declared scope: yes — remaining broader corpus/oracle work belongs to W030 and assurance/residual packetization belongs to W031.
8. Completion language audit passed: yes — closure language is limited to W029 declared runtime-derived effects/environment widening scope and does not claim W030/W031 closure.
9. `IN_PROGRESS_FEATURE_WORKLIST.md` updated: yes / not applicable — feature-map truth did not change for this W029 closure audit.
10. `CURRENT_BLOCKERS.md` updated if needed: yes / not applicable — ordinary execution state is in `.beads/`; no prose blocker update is needed.

## Status
- execution_state: closure_recommended
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: []
- closure_audit_result: pass for declared W029 phase scope
- next_ready_if_closed: W030 first corpus/oracle bead under `calc-lb1`
- non_scope_successors:
  - broader TreeCalc corpus/oracle and first sequential baseline belongs to `W030`
  - assurance refresh and residual packetization belongs to `W031`
- claim_confidence: high for W029 declared phase scope
- reviewed_inbound_observations: current OxFml seam baseline consumed; OxFml W066 resolved the historical W028 baseline quarantine guard, and normal clean `../OxFml` HEAD `487a5cfedc342f3983576d553cfc798941ab96bd` is the current validated baseline; execution-restriction, capability-sensitive, dynamic-dependency, and shape/topology runtime-derived facts now have OxCalc-local live TreeCalc evidence and deterministic overlay/replay projection without a new OxFml handoff trigger.
