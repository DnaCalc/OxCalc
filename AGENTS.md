# AGENTS.md — OxCalc Agent Instructions

## 1. Context Loading Order

On session start, read in this order:

1. `README.md`
2. `CHARTER.md`
3. `OPERATIONS.md`
4. `docs/spec/README.md`
5. `CURRENT_BLOCKERS.md`
6. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
7. `docs/worksets/README.md`
8. Inbound observation ledgers from consumer repos (see OPERATIONS.md Section 12.2):
   - `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
9. Foundation doctrine docs (`../Foundation/CHARTER.md`, `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`, `../Foundation/OPERATIONS.md`)

## 2. Source-of-Truth Precedence

When guidance conflicts, precedence is:

1. `../Foundation/CHARTER.md`
2. `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`
3. `../Foundation/OPERATIONS.md`
4. this repo `CHARTER.md`
5. this repo `OPERATIONS.md`

For OxCalc-local work, treat `CHARTER.md` in this directory as the working charter.
For cross-program doctrine and architecture constraints, treat Foundation docs as authoritative.
For mutable core-engine and coordinator spec work, use `docs/spec/*` in this repo.
FEC/F3E files in this repo are mirrors, not canonical — canonical source is `../OxFml/docs/spec/`.

## 3. Anti-Premature-Completion Doctrine

This section is binding. Violations are doctrine failures, not style preferences.

### Rule 1: Restricted Completion Language
The words "implemented", "closed", "done", and "complete" are forbidden when describing:
- partial subsets of declared scope,
- scaffolding, stubs, or compile-only code,
- merely enabled paths without exercised evidence,
- spec text without replay/trace evidence.

Use "in-progress", "partial", or "scaffolded" instead.

### Rule 2: Self-Audit Required Before Completion Claims
Before ANY completion claim, the agent must:
1. Run the Pre-Closure Verification Checklist from OPERATIONS.md Section 7.
2. Run the Completion Claim Self-Audit from OPERATIONS.md Section 9.
3. Include the checklist and self-audit results in the completion report.

### Rule 3: Three-Axis Reporting Mandatory
Every status report must include:
- `scope_completeness` (`scope_complete` | `scope_partial`)
- `target_completeness` (`target_complete` | `target_partial`)
- `integration_completeness` (`integrated` | `partial`)
- explicit `open_lanes` list when any axis is partial

### Rule 4: Scaffolding Is Not Implementation
Stubs, empty traits, compile-only code, and placeholder implementations are scaffolding.
Scaffolding is never reported as implementation. Report it as `scaffolded`.

### Rule 5: Spec Text Without Evidence Is Not Done
Spec or contract text without at least one deterministic replay artifact proving intended behavior is not done. Report it as `spec_drafted`.

### Rule 6: Cross-Repo Handoff Is Not Completion
Filing a handoff packet to OxFml opens a dependency — it does not close work.
The originating item remains `in_progress` until the receiving repo acknowledges and integrates.

### Rule 7: Default to In-Progress
When uncertain whether work meets completion criteria, report `in_progress`.

### Rule 8: Semantic-Equivalence Under Strategy Change
A coordinator policy or scheduling change is not complete unless a semantic-equivalence statement is provided, demonstrating that observable results are invariant under the strategy change for all affected profiles.

## 4. Continuation Behavior

Mode: **AutoRun** for `W013 Execution Sequence A`; otherwise **checkpoint-at-gates** (conservative).

1. AutoRun is enabled for the declared scope `W013_EXECUTION_SEQUENCE_A_TREECALC_STAGE1_IMPLEMENTATION.md`.
2. The AutoRun exit gate is the final gate declared in `W013_EXECUTION_SEQUENCE_A_TREECALC_STAGE1_IMPLEMENTATION.md`.
3. While AutoRun is active for W013, the agent must continue across the numbered W013 sub-phases without pausing at intermediate workset gates unless:
   - the W013 final gate is reached,
   - a blocker requires `CURRENT_BLOCKERS.md` handling under Section 5,
   - a cross-repo or destructive decision requires user confirmation,
   - a direct conflict with user changes requires resolution.
4. While AutoRun is active, the agent must still report brief status updates during execution and must checkpoint at meaningful sequence milestones, but those checkpoints do not suspend execution.
5. Outside the declared W013 scope, the default mode remains checkpoint-at-gates.

Rationale: the user has explicitly authorized continuous execution for the first TreeCalc-first implementation wave, with W013 acting as the governing exit gate.

## 5. Blocker Handling

When a blocker is encountered:

1. Create or update `CURRENT_BLOCKERS.md` with a structured `BLK-CALC-NNN` entry.
2. Continue with other non-blocked work within scope.
3. If all paths are blocked, emit a structured summary:
   - blocked items with `BLK-*` identifiers,
   - current state of each,
   - exact unblock steps required,
   - recommendation (wait / escalate / workaround).

## 6. Public Attribution Doctrine

For any issue, pull request, email response, release note, discussion post, or any other external/public-facing message authored by an agent, the first line must be an italicized attribution line.

Required format:

*Posted by [Agent] agent on behalf of @govert*

Scope exclusions (do not add attribution by default):
- internal run artifacts,
- repository documentation drafts and working notes,
- local analysis files not being published externally.

## 7. Change Discipline

1. Keep changes minimal, explicit, and testable.
2. Changes to shared seam specs (FEC/F3E coordinator-facing clauses, publication fences, scheduling interaction, rejection policy) require cross-repo impact assessment before promotion.
3. When proposing changes that affect OxFml evaluator-facing clauses, file a handoff packet per OPERATIONS.md Section 5 and register it in `docs/handoffs/HANDOFF_REGISTER.csv`.
4. Neither repo marks a seam change as "complete" until both sides acknowledge.

