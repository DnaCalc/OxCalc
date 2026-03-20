# OxCalc Worksets

Worksets are sequence-based execution packets for OxCalc core-engine and coordinator work.

## Naming Convention

Sequential numbering: `W001`, `W002`, `W003`, ...

File pattern: `W<NNN>_<SLUG>.md`

Sequential numbering makes dependency ordering visible and avoids ambiguity.

## Status Vocabulary

| Status | Meaning |
|--------|---------|
| `planned` | Accepted into sequence, not yet started |
| `in_progress` | Active work underway |
| `blocked` | In-progress with active blocker (see CURRENT_BLOCKERS.md) |
| `complete` | All gate criteria met, pre-closure checklist passed, three-axis report attached |

## Claim Confidence

| Level | Meaning |
|-------|---------|
| `draft` | Initial structure, known gaps |
| `provisional` | Substantive content, pending final evidence |
| `validated` | All evidence present and verified |

## Workset Template

Each workset file must include:

```markdown
# W<NNN>: <Title>

## Purpose
<Why this workset exists and what it delivers>

## Position and Dependencies
- **Depends on**: <W-NNN references or "none">
- **Blocks**: <W-NNN references or "none">
- **Cross-repo**: <handoff dependencies if any>

## Scope
### In scope
1. <item>

### Out of scope
1. <item>

## Deliverables
1. <deliverable with verifiable criteria>

## Gate Model
### Entry gate
- <precondition>

### Exit gate
- <criteria — binary yes/no>

## Execution Packet Additions
(Required when the workset is an execution packet rather than a planning-only packet)

### Environment Preconditions
- <required tools>
- <optional tools and fallback rules>

### Evidence Layout
- canonical artifact root: <path>
- checked-in or ephemeral: <policy>
- baseline run naming: <policy>

### Replay-Corpus Readiness
- required replay classes with scenario ids: <mapping>
- reserve or later replay classes: <mapping>

### Coupled Widening Rule
- engine surfaces widened in this packet: <list or "none">
- oracle/conformance surfaces widened in the same slice: <list or "none">
- widened comparison artifact: <path or "none">

## Pre-Closure Verification Checklist
(Copy from OPERATIONS.md Section 7, fill in yes/no for each item)

## Status
- execution_state: planned | in_progress | blocked | complete
- scope_completeness: scope_complete | scope_partial
- target_completeness: target_complete | target_partial
- integration_completeness: integrated | partial
- open_lanes: <list or "none">
- claim_confidence: draft | provisional | validated
- reviewed_inbound_observations: <summary or "none">
```

## Rules

1. Worksets are sequence or gate driven, never date driven.
2. Each workset must declare dependencies, deliverables, and gate criteria.
3. Completion requires passing the Pre-Closure Verification Checklist (OPERATIONS.md Section 7).
4. Completion requires a three-axis status report (AGENTS.md Section 3, Rule 3).
5. Completion requires the Completion Claim Self-Audit (OPERATIONS.md Section 9).
6. Claim confidence and status must be stated separately.
7. Workset status should record reviewed inbound observations when interface or design inputs from sibling repos are relevant.
8. Execution packets must declare environment preconditions, evidence layout, and replay-corpus readiness before implementation begins.
9. If a workset expects emitted evidence, it must declare the canonical artifact root and whether the emitted artifacts are checked in or ephemeral.
10. If a workset widens semantic behavior, it must state how engine, oracle, and conformance surfaces widen together.
11. Workset closure and broader feature-area continuation are distinct; later widening should use successor worksets or explicit extension lanes rather than silently reopening a completed workset.
12. When a workset changes implementation language or runtime direction, it must state:
    - the authoritative semantic and evidence references,
    - what existing implementation remains as executable comparison surface,
    - and the rules preventing mechanical cross-language pattern transfer.
13. Replay-facing capability-promotion packets must name:
    - emitted bundle root,
    - emitted validator artifact,
    - emitted explain artifact,
    - and the highest capability level that may move if the evidence is checked in.
14. Replay-facing emitted artifacts are additive sidecars unless a spec explicitly says otherwise; worksets must not silently replace the native OxCalc artifact root as the semantic authority.
15. If a capability ladder is expected to continue beyond the current packet, the successor packet should be authored before closure of the current packet.

## Current Sequence Tail

The current realized continuation after `W017` is:
1. `W016_WITNESS_DISTILLATION_AND_RETAINED_FAILURE_PACKS.md`
2. `W018_EXECUTION_SEQUENCE_E_REPLAY_APPLIANCE_BUNDLE_AND_CAPABILITY_PROMOTION.md`
3. `W020_OXFML_DOWNSTREAM_INTEGRATION_ROUND_01.md`
4. `W019_EXECUTION_SEQUENCE_F_REPLAY_DISTILL_AND_PACK_PROMOTION.md`
5. `W021_EXECUTION_SEQUENCE_G_PACK_GRADE_REPLAY_PROMOTION.md`
6. `W022_EXECUTION_SEQUENCE_H_SHARED_PACK_FAMILY_AND_DIRECT_BINDING_WIDENING.md`
7. `W023_EXECUTION_SEQUENCE_I_PROGRAM_GRADE_PACK_PROMOTION.md`
8. `W024_EXECUTION_SEQUENCE_J_BROADER_PROGRAM_SCOPE_PACK_PROMOTION.md`
9. `W025_TREECALC_STRUCTURAL_AND_FORMULA_SUBSTRATE_WIDENING.md`
10. `W026_TREECALC_OXFML_BIND_REFERENCE_AND_SEAM_INTAKE.md`
11. `W027_TREECALC_DEPENDENCY_GRAPH_AND_INVALIDATION_CLOSURE.md`
12. `W028_TREECALC_EVALUATOR_BACKED_CANDIDATE_RESULT_INTEGRATION.md`
13. `W029_TREECALC_RUNTIME_DERIVED_EFFECTS_AND_OVERLAY_CLOSURE.md`
14. `W030_TREECALC_CORPUS_ORACLE_AND_FIRST_SEQUENTIAL_BASELINE.md`
15. `W031_TREECALC_ASSURANCE_REFRESH_AND_RESIDUAL_PACKETIZATION.md`

`W017` moved the active implementation fully into the Rust workspace under `src/`, with historical carried runs replacing the old parallel-code reference role.
`W016` has now reached its declared gate for the first retained-witness and retained-failure baseline.
`W018` has now reached its declared gate for replay-appliance bundle realization and capability promotion through `cap.C3.explain_valid`.
`W020` is the first post-W018 OxFml downstream integration round: it processes the current OxFml note and inbound handoff without reopening W005. That round is now materially processed and does not currently justify a narrower follow-on handoff.
`W019` has now reached its declared gate: it proves `cap.C4.distill_valid`, adds dependency-projection-sensitive reduced witnesses, and defines rehearsal-only pack-candidate separation without claiming `cap.C5.pack_valid`.
`W021` has now reached its declared gate: it declares a semantic-only pack-grade contract, emits a pack-grade validation artifact, and leaves `cap.C5.pack_valid` explicitly blocked rather than implicit.
`W022` is the explicit successor lane for retained-shared or pack-promoted witness families, direct-binding-sensitive pack evidence, and any later pack-grade promotion. It now reaches a bounded decision point without claiming `cap.C5.pack_valid`.
`W023` is the next residual lane for broader program-grade pack promotion beyond the current semantic-only `TraceCalc` scope. Its first slice emits explicit program-grade contract and validation sidecars, its second slice widens broader host-sensitive family evidence, and its third slice makes the capability and handoff decision explicit.

`W024` is now the next residual lane after W023 for broader program-grade pack promotion beyond the currently exercised host-sensitive `TraceCalc` family.
`W025` through `W031` are the now-packetized TreeCalc semantic-completion sequence. Together they are the line-of-sight plan from the current proving substrate to the first semantically-complete sequential TreeCalc engine.
Later widening must use successor worksets rather than silently reopening `W016`, `W018`, `W019`, or W005.
