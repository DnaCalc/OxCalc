# W012: TraceCalc Reference Machine and Conformance Oracle

## Purpose
Define the `TraceCalc Reference Machine` as the first executable conformance oracle for OxCalc, so production implementation can be measured against a deterministic semantic reference rather than against prose alone.

## Position and Dependencies
- **Depends on**: W007, W008, W009, W010, W011
- **Blocks**: none
- **Cross-repo**: must remain compatible with accepted OxFml seam direction for candidate-result versus publication and typed reject consequences, but is otherwise self-contained in first scope

## Scope
### In scope
1. Reference-machine purpose, role, and oracle doctrine.
2. Reference-machine state model aligned to W007 and W008 vocabulary.
3. Reference-machine transition set for self-contained `TraceCalc` scenarios.
4. Canonical observed artifact surface emitted by the machine.
5. Conformance-comparison rules for later production-engine matching.
6. Workset and roadmap binding so the oracle becomes the explicit pre-implementation semantic baseline.

### Out of scope
1. Immediate implementation of the reference machine.
2. OxFml-integrated evaluator execution.
3. Staged concurrency realization inside the oracle.
4. Full replay-pack export.
5. Grid or later substrate semantics.

## Deliverables
1. Canonical spec companion at `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`.
2. Explicit reference-machine state and transition inventory sufficient to implement a first deterministic interpreter.
3. Explicit conformance surface naming what later engines must match.
4. Roadmap and feature-register integration for the reference-machine lane.
5. Closed first-pass policy for artifact layout and oracle-to-engine diff behavior.

## Gate Model
### Entry gate
- W007 defines the Lean-facing state-object vocabulary.
- W008 defines the TLA+-oriented coordinator action boundary.
- W009 defines replay and pack pressure.
- W010 defines counters and decisive experiment pressure.
- W011 defines the harness, corpus, and validator-runner boundary.

### Exit gate
- The reference-machine semantic core is explicit enough to implement without reopening baseline semantic questions.
- The observed artifact surface is explicit enough to compare later engine outputs against the oracle.
- The conformance rule is explicit enough to state when a later engine matches or diverges from the oracle.
- The lane is integrated into the active roadmap and workset sequence.
- The first-pass artifact layout and diff policy are explicit enough to implement directly.

## State-Model Direction
The reference machine should align with:
1. immutable structural truth,
2. published-view state,
3. pinned-view state,
4. runtime overlay state,
5. candidate-result store,
6. reject log,
7. trace log,
8. counter state,
9. run context.

## Transition Direction
The first transition set should include:
1. load and validation transitions,
2. scenario-step transitions for pin, stale, admit, candidate emit, reject, publish, and reset,
3. completion transitions for evaluation and artifact emission.

## Conformance Direction
The first conformance surface should require matching at least:
1. final published view,
2. pinned-view observations,
3. typed reject outcomes,
4. trace label counts,
5. declared counters,
6. candidate-versus-publication boundary preservation.

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
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the reference-machine and engine conformance path is now widened through the 12-scenario `w014-stage1-widening-baseline` run
  - replay-appliance bundle projection, richer mismatch severity output, and reduced-witness flows remain later lanes
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
