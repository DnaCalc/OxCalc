# W016: Execution Sequence D - Witness Distillation and Retained Failure Packs

## Purpose
Operationalize the retained-witness lane after `W015` and `W017`.

This packet exists to:
1. turn the witness-distillation plan into a bounded execution sequence,
2. preserve OxCalc authority over `TraceCalc` scenario structure, engine-diff meaning, and acceptance-oracle behavior,
3. realize retained-failure artifacts in a way that stays replay-aware without overclaiming pack-grade status,
4. keep reduced witnesses grounded in declared scenario anchors, event groups, reject records, and view slices rather than in ad hoc minimization heuristics.

## Position and Dependencies
- **Depends on**: W015, W017
- **Blocks**: none
- **Cross-repo**: aligned to Foundation witness lifecycle and registry policy; any shared seam pressure discovered during reduced-witness rollout still routes through W005

## Scope
### In scope
1. Convert W016 into an execution packet with explicit sequence gates.
2. Realize the first deterministic reduction-unit and lifecycle artifact slice in Rust.
3. Use `TraceCalc` witness anchors as the authoritative reduction-unit source.
4. Emit OxCalc-local witness-seed artifacts under the declared replay-appliance artifact root.
5. Define the later execution path for explanatory-only witnesses, quarantined witnesses, replay-valid retained witnesses, and retained-failure baselines.

### Out of scope
1. Claiming `cap.C4.distill_valid` before replay-valid reduced witnesses exist.
2. Claiming `cap.C5.pack_valid`.
3. Replacing current `TraceCalc` scenario authoring with a new DSL or generic reducer abstraction.
4. Weakening engine-diff, reject-set, or view-surface semantics to fit a generic witness format.

## Deliverables
1. A concrete execution sequence for retained-witness rollout.
2. The first Rust witness-seed and reduction-unit artifact slice.
3. Deterministic lifecycle records and reduction manifests tied to declared `TraceCalc` anchors.
4. A later sequence plan for explanatory-only, quarantined, and replay-valid retained witnesses.

## Gate Model
### Entry gate
- W015 has established replay-facing coherence, normalized event-family doctrine, and adapter policy.
- W017 has established the Rust-first implementation lane and the active regenerable Rust baseline.
- `TraceCalc` replay-facing scenarios already carry `replay_projection` and `witness_anchors`.

### Exit gate
- Reduction units and closure rules are realized enough to emit deterministic witness-seed artifacts without reopening semantic authority questions.
- Lifecycle records are explicit enough to prevent explanatory-only or quarantined witnesses from being mistaken for pack-eligible evidence.
- The retained-witness execution path is broken into bounded later sequences rather than left as one broad planning bucket.

## Sequence Preconditions
Execution Sequence D assumes:
1. `w017-rust-parity-baseline` is the active regenerable Rust baseline,
2. the carried `w014-stage1-widening-baseline` remains the semantic anchor baseline,
3. the active replay-facing corpus through `R1..R8` is stable enough to seed reduction units,
4. the current Rust runner remains the artifact authority for local retained-witness rollout.

## Execution Packet Additions

### Environment Preconditions
- required tools:
  - `cargo`
  - `rustc`
  - `powershell`
- optional tools:
  - `lean`
  - `tlc`
- fallback rules:
  - W016 Sequence 1 may proceed on runner-oracle evidence without new formal-tool output
  - later replay-valid retained-witness claims must still bind back into W006 assurance lanes

### Evidence Layout
- canonical artifact root:
  - `docs/test-runs/core-engine/tracecalc-reference-machine/<run_id>/`
- retained-witness additive surfaces:
  - `replay-appliance/reductions/<reduction_id>/reduction_manifest.json`
  - `replay-appliance/witnesses/<witness_id>/lifecycle.json`
- later additive surfaces:
  - `replay-appliance/reductions/<reduction_id>/candidate_journal.jsonl`
  - `replay-appliance/reductions/<reduction_id>/witness_bundle/*`

### Replay-Corpus Readiness
- required carried replay classes:
  - `R1` through `R8`
- minimum carried scenarios for Sequence 1:
  - all replay-facing scenarios with `witness_anchors`
- witness-seed readiness rule:
  - a scenario may participate in W016 only if its reduction anchors are explicit enough to define scenario, phase, event-group, reject-record, and view-slice units where applicable

### Coupled Widening Rule
- engine surfaces widened in this packet:
  - none in semantic behavior
  - witness-seed artifact emission is additive over current runner behavior
- oracle/conformance surfaces widened in the same slice:
  - retained-witness seed manifests and lifecycle records are projected from the same run that emits conformance artifacts
- semantic-equivalence rule:
  - W016 Sequence 1 must not change existing result-state, trace, reject, view, or counter surfaces for the carried corpus

## Execution Sequence D

### Sequence 1. Reduction-Unit and Lifecycle Seed Slice
Primary work areas:
- reduction-unit model
- lifecycle record model
- runner emission for witness seeds

Entry gate:
- W016 is still planning-only and no reduced-witness artifacts exist yet.

Execution objective:
- realize the first deterministic witness-seed slice in Rust using declared `TraceCalc` anchors.

Exit gate:
- the runner emits deterministic `reduction_manifest.json` and `lifecycle.json` seed artifacts for replay-facing scenarios with witness anchors,
- scenario, phase-block, event-group, reject-record, and view-slice units are explicit in emitted manifests,
- lifecycle records use `wit.generated_local` and remain explicitly non-pack-eligible,
- any reduction status id not yet bound to a Foundation machine-readable registry is marked local-only rather than silently treated as canonical,
- existing carried conformance surfaces remain unchanged for the current corpus.

Sequence 1 realized scope:
1. deterministic witness ids and reduction ids,
2. reduction units derived from declared anchors,
3. closure rules grounded in scenario and event-group structure,
4. seed lifecycle records emitted under the replay-appliance root,
5. local-only seeded reduction status while registry binding remains open.

Sequence 1 explicit non-goals:
1. replay-valid reduced witnesses,
2. explanatory-only reduced witnesses,
3. quarantine flows,
4. candidate journals,
5. witness bundles.

### Sequence 2. Explanatory-Only and Quarantine Flow
Primary work areas:
- failed reduction handling
- explanatory-only lifecycle transition
- quarantine reason binding

Entry gate:
- Sequence 1 exit gate has passed.

Execution objective:
- realize the first explicit non-replay-valid witness outcomes without overclaiming retained validity.

Exit gate:
- explanatory-only reduced witnesses are explicit,
- quarantine reasons are explicit where replay-valid capture is missing,
- explanatory-only and quarantined outputs remain non-pack-eligible by emitted lifecycle state.

Sequence 2 realized scope:
1. lifecycle-state classification from scenario-result outcomes,
2. explicit `wit.explanatory_only` state for mismatch or assertion-driven non-replay-valid outcomes,
3. explicit `wit.quarantined` state with `capture_insufficient` for validation or capture failure,
4. surfaced witness lifecycle and reduction-manifest paths in scenario artifact paths,
5. additive runner output with unchanged carried conformance surfaces.

### Sequence 3. Replay-Valid Retained Witness Slice
Primary work areas:
- preserved mismatch predicates
- replay-valid reduced witness checks
- retained-local lifecycle transition

Entry gate:
- Sequence 2 exit gate has passed enough to make failure classes explicit.

Execution objective:
- realize the first replay-valid retained witness path for one bounded mismatch family.

Exit gate:
- at least one reduced witness family remains replay-valid after reduction,
- preservation predicates are explicit and exercised,
- retained-local witnesses are distinct from explanatory-only outputs.

Sequence 3 realized scope:
1. a dedicated retained-failure fixture manifest over existing replay-valid `TraceCalc` scenarios,
2. one bounded retained-local witness family for the publication-fence reject surface,
3. replay-validation artifacts that distinguish replay-valid, explanatory-only, and quarantined outcomes,
4. explicit retained-local lifecycle emission without pack promotion.

### Sequence 4. Retained-Failure Baseline and Bundle Binding
Primary work areas:
- retained-failure baseline run
- bundle structure
- workset-to-pack evidence binding

Entry gate:
- Sequence 3 exit gate has passed.

Execution objective:
- emit the first checked-in retained-failure baseline for OxCalc-local witness flows.

Exit gate:
- one checked-in retained-failure baseline exists,
- replay-appliance witness outputs are bound to concrete scenario ids and mismatch classes,
- pack-grade status remains unclaimed unless stronger evidence exists.

Sequence 4 realized scope:
1. CLI support for retained-failure fixture runs,
2. a checked-in retained-failure baseline at `w016-sequence4-retained-failure-baseline`,
3. emitted `case_summary.json`, `lifecycle.json`, `reduction_manifest.json`, `replay_validation.json`, and `witness_bundle/scenario.json` artifacts per retained-failure case,
4. explicit lifecycle variety across `wit.retained_local`, `wit.explanatory_only`, and `wit.quarantined`.

## OxCalc-Specific Distillation Rules
1. Reduction starts from declared scenario anchors rather than from generic AST or trace slicing.
2. Scenario units close over all finer reduction units.
3. Phase-block units close over event groups that share steps with the phase block.
4. Event-group units close over reject-record units whose triggering steps fall inside the event group.
5. View-slice units may close over event-group units that materially shape the retained view surface.
6. Local-only reduction status ids must be explicit until Foundation publishes a machine-readable reduction-status registry.
7. `wit.generated_local` is the only lifecycle state allowed for Sequence 1 seed artifacts.

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
| 10 | CURRENT_BLOCKERS.md updated (new/resolved)? | no |

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: []
- claim_confidence: validated
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
