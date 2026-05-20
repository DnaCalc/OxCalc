# AGENTS.md — OxCalc Agent Instructions

## 1. Context Loading Order

On session start, read in this order:

1. `README.md`
2. `CHARTER.md`
3. `OPERATIONS.md`
4. `docs/WORKSET_REGISTER.md`
5. `docs/SPEC.md`
6. `docs/worksets/README.md`
7. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
8. Inbound observation ledgers from consumer repos (see OPERATIONS.md Section 12.2):
   - `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
9. Foundation doctrine docs (`../Foundation/CHARTER.md`, `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`, `../Foundation/OPERATIONS.md`)

`docs/BEADS.md` is now a pocket reference for the local bead method. Read it only when bead mechanics are directly relevant or when `OPERATIONS.md` points there.

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
Treat `.beads/` as the only live execution-state surface.
Treat `docs/SPEC.md` as the spec/design entrypoint; for mutable core-engine and coordinator spec work, use the `docs/spec/*` set it indexes.
FEC/F3E files in this repo are mirrors, not canonical — canonical source is `../OxFml/docs/spec/`.

## 3. Completion And Reporting Doctrine

This section is binding. It replaces the older checklist-heavy completion ritual with product-accountable reporting.

### Rule 1: Product Scope First
When reporting a feature area, start with what works for users or downstream hosts:
1. the supported scope,
2. the decisive evidence or check,
3. known exclusions or gaps,
4. formal/proof status separately.

Do not hide behind generic `partial` language when a concrete supported scope can be named.

### Rule 2: Completion Language Is Allowed When Scoped
It is valid to say a bead, workset, or feature scope is implemented, closed, or complete when:
1. the declared scope is explicit,
2. the implementation exists beyond scaffolding,
3. the relevant checks/evidence for that scope passed,
4. exclusions are explicit.

Do not use completion language for scaffolding, stubs, compile-only paths, unexercised enabled paths, or spec text without an implementation/evidence path.

### Rule 3: Scaffolding Is Not Implementation
Stubs, empty traits, compile-only code, and placeholder implementations are scaffolding.
Scaffolding is never reported as implementation. Report it as `scaffolded`.

### Rule 4: Spec Text Is Design Work Until Exercised
Spec or contract text without implementation, tests, replay, model checks, or a named downstream consumer is design/spec work. Call it `spec_drafted`, `specified`, or `planned`, not implemented.

### Rule 5: Cross-Repo Handoffs Are Dependencies
Filing a handoff packet to another repo does not close behavior that depends on the receiving repo. Report the local handoff as sent; report the product or seam behavior as open until the receiving side acknowledges and the integration path is exercised.

### Rule 6: Strategy Changes Need Equivalence Evidence
A coordinator policy, scheduler, concurrency, cache, or invalidation strategy change can be claimed for a profile only when observable results are shown invariant for that profile, or the non-equivalence is explicitly declared as a profile change.

### Rule 7: Status Format
Use the shortest status shape that answers the question. Prefer:
1. `Product status:` what is supported.
2. `Evidence:` decisive checks, runs, or observations.
3. `Still open:` concrete gaps.
4. `Formal status:` proof/model status if relevant.

Historical packet-local status fields may remain in older artifacts. Do not copy them into ordinary reports unless the artifact being edited explicitly owns that format.

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
1. OxCalc now uses `docs/WORKSET_REGISTER.md` plus `.beads/` as the ordinary planning/execution model.
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
3. When proposing changes that affect OxFml evaluator-facing clauses, file a handoff packet per OPERATIONS.md Section 5.
4. Neither repo claims a seam change for product use until both sides acknowledge the contract and the integration path is exercised or explicitly scoped.

