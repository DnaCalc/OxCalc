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
3. `W019_EXECUTION_SEQUENCE_F_REPLAY_DISTILL_AND_PACK_PROMOTION.md`

`W017` moved the active implementation fully into the Rust workspace under `src/`, with historical carried runs replacing the old parallel-code reference role.
`W016` has now reached its declared gate for the first retained-witness and retained-failure baseline.
`W018` has now reached its declared gate for replay-appliance bundle realization and capability promotion through `cap.C3.explain_valid`.
`W019` is the explicit successor lane for `cap.C4.distill_valid`, `cap.C5.pack_valid`, and broader replay-appliance widening.
Later widening must use successor worksets rather than silently reopening `W016` or `W018`.
