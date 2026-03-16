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

## Current Sequence Tail

The current planned continuation after `W015` is:
1. `W017_RUST_FIRST_REIMPLEMENTATION_OF_CURRENT_REALIZED_SCOPE.md`
2. `W016_WITNESS_DISTILLATION_AND_RETAINED_FAILURE_PACKS.md`

`W017` shifts OxCalc implementation execution to a Rust-first lane while preserving the existing exercised .NET code as parity/evidence reference.
`W016` remains the retained-witness follow-on lane after the Rust-first implementation direction is established.
