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
