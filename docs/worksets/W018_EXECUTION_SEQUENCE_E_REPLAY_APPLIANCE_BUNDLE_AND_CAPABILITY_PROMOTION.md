# W018: Execution Sequence E - Replay Appliance Bundle and Capability Promotion

## Purpose
Operationalize the next Rust-first replay lane after W015, W016, and W017.

This packet exists to:
1. realize normalized replay-appliance bundle emission in the active Rust runner,
2. promote OxCalc from local replay-aware artifacts to emitted replay-appliance bundle artifacts,
3. tighten capability evidence from the current conservative floor toward `cap.C2.diff_valid` and `cap.C3.explain_valid` without overclaiming,
4. keep OxCalc authority over `TraceCalc`, engine-diff meaning, and reference-machine semantics while projecting those surfaces into the replay-appliance bundle form.

## Position and Dependencies
- **Depends on**: W015, W016, W017
- **Blocks**: none
- **Cross-repo**: any shared seam pressure discovered while normalizing replay bundle surfaces still routes through W005

## Scope
### In scope
1. Emit normalized replay-appliance bundle roots from the active Rust runner.
2. Project normalized event-family, mismatch-kind, severity-class, lifecycle, and reduction-status surfaces into emitted bundle artifacts.
3. Realize the first bundle-validator path over emitted bundle artifacts.
4. Promote capability claims only to the highest level proven by emitted evidence.
5. Add explain-record and bundle-facing diff surfaces where required for `cap.C3.explain_valid`.

### Out of scope
1. Claiming `cap.C5.pack_valid`.
2. Replacing `TraceCalc` or the OxCalc reference-machine semantics with a generic replay abstraction.
3. New concurrency or Stage 2 execution widening.
4. Shared seam changes without a separate W005-routed handoff.

## Deliverables
1. An execution-sequenced replay-appliance realization packet.
2. Emitted normalized bundle roots under the active Rust run structure.
3. Bundle-validator realization and exercised evidence.
4. Updated capability claims backed by emitted local artifacts.

## Gate Model
### Entry gate
- W015 has established adapter doctrine and capability-manifest direction.
- W016 has established the first retained-witness and retained-failure baseline.
- W017 has established Rust as the active implementation lane for the declared scope.

### Exit gate
- The Rust runner emits normalized replay-appliance bundle artifacts for the declared current corpus.
- Bundle validation is exercised over emitted artifacts, not only spec text.
- Capability claims are promoted only to the highest level proven by current emitted artifacts.
- Any remaining pack-grade or broader replay-governance gaps are explicit successor lanes rather than hidden inside this packet.

## Sequence Preconditions
Execution Sequence E assumes:
1. `w017-rust-parity-baseline` remains the active regenerable Rust baseline for ordinary conformance surfaces,
2. `w016-sequence4-retained-failure-baseline` remains the first retained-failure baseline,
3. the replay-facing corpus through `R1..R8` is stable enough to serve as the first emitted bundle population,
4. the current capability floor remains conservative until emitted bundle evidence proves promotion.

## Execution Packet Additions

### Environment Preconditions
- required tools:
  - `cargo`
  - `powershell`
- optional tools:
  - `lean`
  - `tlc`
- fallback rules:
  - bundle-emission and validator evidence do not require fresh formal-tool output
  - no capability claim may imply proof or model-check evidence that has not been rerun

### Evidence Layout
- canonical ordinary run root:
  - `docs/test-runs/core-engine/tracecalc-reference-machine/<run_id>/`
- canonical retained-failure run root:
  - `docs/test-runs/core-engine/tracecalc-retained-failures/<run_id>/`
- canonical replay-appliance additive roots:
  - ordinary run: `.../<run_id>/replay-appliance/`
  - retained-failure run: `.../<run_id>/replay-appliance/`

### Replay-Corpus Readiness
- carried replay classes:
  - `R1` through `R8`
- carried retained-failure baseline:
  - `w016-sequence4-retained-failure-baseline`
- promotion note:
  - `cap.C2.diff_valid` and `cap.C3.explain_valid` remain the first realistic promotion targets for this packet

## Execution Sequence E

### Sequence 1. Bundle Artifact Layout and Projection Contract
Primary work areas:
- bundle root layout
- normalized projection of existing run surfaces
- artifact naming policy

Entry gate:
- W018 is still planning-only and the current runner emits replay-aware side artifacts but not a normalized emitted bundle root.

Execution objective:
- define and realize the emitted replay-appliance bundle root for ordinary and retained-failure runs.

Exit gate:
- emitted bundle root layout is realized in Rust,
- normalized event-family and mismatch projection rules are embodied in emitted artifacts,
- ordinary and retained-failure runs both expose bundle roots without changing carried conformance semantics.

### Sequence 2. Bundle Validator Realization
Primary work areas:
- bundle validator
- schema and artifact-shape checks
- exercised validator evidence

Entry gate:
- Sequence 1 exit gate has passed.

Execution objective:
- realize the first bundle validator over emitted local bundle artifacts.

Exit gate:
- validator artifacts exist and are exercised over ordinary and retained-failure bundle roots,
- unsupported or degraded bundle output is explicit rather than silently omitted.

### Sequence 3. Explain and Diff Promotion Slice
Primary work areas:
- explain record emission
- bundle-facing diff normalization
- capability-floor reassessment

Entry gate:
- Sequence 2 exit gate has passed.

Execution objective:
- realize the first emitted explain and normalized diff surfaces needed for capability promotion.

Exit gate:
- explain records are emitted for the declared current mismatch families,
- emitted diff surfaces are normalized enough to support an honest capability review,
- capability manifest is promoted only if the emitted evidence supports it.

### Sequence 4. Replay Bundle Baseline and Capability Claim Refresh
Primary work areas:
- checked-in replay-appliance baseline
- capability-manifest refresh
- run-ledger integration

Entry gate:
- Sequence 3 exit gate has passed.

Execution objective:
- check in one explicit replay-appliance-aware baseline and refresh capability claims against it.

Exit gate:
- one checked-in replay-appliance-aware baseline exists,
- capability claims are refreshed honestly against emitted evidence,
- any remaining `cap.C4+` or pack-grade gaps are explicit successor lanes.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes |
| 2 | Pack expectations updated for affected packs? | yes |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes |
| 6 | All required tests pass? | yes |
| 7 | No known semantic gaps remain in declared scope? | yes |
| 8 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 9 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 10 | CURRENT_BLOCKERS.md updated (new/resolved)? | yes |

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: []
- claim_confidence: high
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
