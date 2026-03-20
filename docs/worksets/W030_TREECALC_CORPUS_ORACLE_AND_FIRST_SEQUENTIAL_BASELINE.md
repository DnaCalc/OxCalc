# W030: TreeCalc Corpus, Oracle, and First Sequential Baseline

## Purpose
Turn the real TreeCalc engine path into an exercised and comparable runtime by widening corpus, oracle, replay, and baseline evidence until the first sequential TreeCalc-ready engine can be observed directly.

## Position and Dependencies
- **Depends on**: W028, W029
- **Blocks**: W031
- **Cross-repo**: none by default; narrower handoff only if the widened evidence surfaces expose a real new seam insufficiency

## Scope
### In scope
1. first TreeCalc corpus using real nodes, formulas, references, and bind/evaluator flows
2. oracle and conformance widening so ordinary TreeCalc runs can be compared deterministically
3. replay, diff, explain, and retained-witness continuation for the live TreeCalc path
4. first checked-in sequential TreeCalc-ready baseline run
5. semantic-equivalence statement for any strategy substitutions used during the transition from proving substrate to live TreeCalc path

### Out of scope
1. concurrency or async realization
2. broad pack-grade replay promotion beyond the current declared replay capability floor
3. grid or workbook-sheet semantics

## Deliverables
1. TreeCalc corpus scenarios or fixtures that exercise the real engine path
2. widened oracle and conformance artifacts for those TreeCalc runs
3. one checked-in sequential TreeCalc-ready baseline
4. replay, diff, and explain surfaces analogous to the current `TraceCalc` lane

## Gate Model
### Entry gate
- W028 and W029 have made the real TreeCalc engine pipeline executable
- the TreeCalc corpus boundary for first-phase scope is explicit

### Exit gate
- the live engine path can run the first TreeCalc corpus without `TraceCalc` scripted semantics standing in for real formula execution
- ordinary TreeCalc runs have conformance and replay artifacts analogous to the current `TraceCalc` lane
- one checked-in TreeCalc-ready baseline exists

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
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - real TreeCalc corpus is not authored yet
  - TreeCalc oracle/conformance widening is not realized yet
  - first sequential TreeCalc-ready baseline does not exist yet
- claim_confidence: draft
- reviewed_inbound_observations: current OxFml seam baseline remains sufficient for planning; narrower handoff only if exercised TreeCalc evidence reveals a new insufficiency
