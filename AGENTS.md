# AGENTS.md — OxCalc Agent Instructions

## 1. Context Loading Order

On session start, read in this order:

1. `README.md`
2. `CHARTER.md`
3. `OPERATIONS.md`
4. `docs/WORKSET_REGISTER.md`
5. `docs/BEADS.md`
6. `docs/worksets/README.md`
7. `docs/spec/README.md`
8. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
9. Inbound observation ledgers from consumer repos (see OPERATIONS.md Section 12.2):
   - `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
10. Foundation doctrine docs (`../Foundation/CHARTER.md`, `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`, `../Foundation/OPERATIONS.md`)

## 2. Source-of-Truth Precedence

When guidance conflicts, precedence is:

1. `../Foundation/CHARTER.md`
2. `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`
3. `../Foundation/OPERATIONS.md`
4. this repo `CHARTER.md`
5. this repo `OPERATIONS.md`

For OxCalc-local work, treat `CHARTER.md` in this directory as the working charter.
For cross-program doctrine and architecture constraints, treat Foundation docs as authoritative.
Treat `docs/WORKSET_REGISTER.md` as the canonical ordered workset surface.
Treat `docs/BEADS.md` as the canonical local bead-method surface.
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

## 3A. Product-Taste Doctrine

OxCalc is a high-quality, high-velocity engineering repo. The working rules exist to help build reliable, performant, useful software; they are not the work product.

1. Start from the calculation engine's behavior. Prefer code, tests, traces, proof models, and compact specs that explain or improve how calculation actually works.
2. Artifacts earn their place by being consumed. A document, ledger, model, or generated output should improve a decision, guide code, support a proof, serve as replay evidence, or be archived.
3. Prefer distillation over accumulation. When a wave emits many local artifacts, first extract the durable insight, update the active spec, and move low-signal surfaces out of the active path.
4. Do not add doctrine to compensate for weak direction. Prune the active surface, make the next engineering move concrete, and keep the command path short.
5. Make specifications executable where possible. A good spec names state, transitions, invariants, pre/post conditions, reference semantics, and the checks that can falsify it.
6. Keep status language subordinate to substance. Reports should say what changed in behavior, proof coverage, model coverage, test coverage, or repo shape.
7. Cleanup is real engineering work when it reduces ambiguity, lowers maintenance cost, or clears the path for better implementation and formalization.
8. When in doubt, choose the smaller active truth surface that future agents can read quickly and trust.

## 4. Continuation Behavior

Mode: **checkpoint-at-natural-boundaries** with light bead-doctrine execution.

1. Agent must pause and report status at material boundaries: new workset activation, dependency handoff, irreversible cleanup move, or user-requested checkpoint.
2. AutoRun is disabled by default.
3. AutoRun may only be enabled when explicitly requested by the user for a specific declared scope.
4. When AutoRun is enabled for a declared scope, the governing workset and exit gate must be updated here before execution continues under AutoRun.
5. Outside an explicitly declared AutoRun scope, the default mode remains checkpoint-at-natural-boundaries.

### Temporary AutoRun Scope
1. Current temporary AutoRun scope: none.
2. AutoRun remains unavailable unless the user explicitly declares a scope and exit condition.
3. When a temporary AutoRun scope ends, reset this section to none.

Transition note:
1. OxCalc now uses `docs/WORKSET_REGISTER.md` plus `.beads/` as the ordinary execution-state model.
2. `.beads/` is now bootstrapped as the ordinary blocker surface.
3. `CURRENT_BLOCKERS.md` no longer owns live blocker truth.

## 5. Blocker Handling

When a blocker is encountered:

1. Record the blocker in `.beads/` through the ordinary bead graph.
2. Continue with other non-blocked work within scope.
3. If all paths are blocked, emit a structured summary:
   - blocked items with `BLK-*` identifiers,
   - current state of each,
   - exact unblock steps required,
   - recommendation (wait / escalate / workaround).

Post-bootstrap rule:
1. ordinary blockers belong in the bead graph rather than in new prose blocker notes.

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

