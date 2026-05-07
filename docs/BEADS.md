# OxCalc Beads Working Method

## 1. Purpose
This file defines the local bead method for OxCalc.

It covers:
1. the local execution model,
2. the `br` / `bv` tool split,
3. the bead mutation rule,
4. OxCalc-specific bead quality expectations,
5. the `workset -> epic -> bead` rollout pattern,
6. the transition rule for the lighter post-W026 doctrine.

## 2. Core Model
Execution in OxCalc now moves through:
1. [docs/WORKSET_REGISTER.md](WORKSET_REGISTER.md)
2. `workset -> epic -> bead`
3. `.beads/` as the detailed execution truth

Interpretation rule:
1. worksets are high-level planning and scope-partition units,
2. epics are the main execution lanes under a chosen workset,
3. beads are the unit of executable progress,
4. worksets do not carry ready/in-progress/blocked/closed execution state,
5. `.beads/` is the sole owner of execution-state truth,
6. replay, trace, oracle, and seam artifacts remain evidence/provenance surfaces and are not replaced by beads.

## 3. Transition Status
Current migration status:
1. `W032` established the OxCalc bead-doctrine reset and light active-tree reorientation.
2. `docs/WORKSET_REGISTER.md` and this file now exist as active doctrine surfaces.
3. `.beads/` is now bootstrapped as the live execution-state surface.
4. `CURRENT_BLOCKERS.md` no longer owns ordinary blocker truth.

Transition rule:
1. use this file and [docs/WORKSET_REGISTER.md](WORKSET_REGISTER.md) to shape live execution now,
2. keep ordinary execution-state truth in `.beads/`,
3. do not reintroduce ad hoc execution-state notes now that `.beads/` exists.

## 4. Tool Split
`br` is the mutation tool.

Use it to:
1. inspect ready work,
2. create beads,
3. update bead status,
4. add dependencies,
5. close completed beads.

Typical commands:

```powershell
br ready
br show <id>
br create --title "..." --type task --priority 2
br update <id> --status in_progress
br close <id> --reason "Completed"
br dep add <issue> <depends-on>
```

`bv` is the graph-aware triage and analysis tool.

Use it to:
1. inspect the ready path,
2. identify blockers,
3. inspect graph shape and pressure.

Agent rule:
1. use only non-interactive robot-style inspection calls from agent sessions,
2. prefer machine-readable or robot output modes where available,
3. do not launch blocking interactive views from unattended sessions.

## 5. Mutation Rule
Do not edit `.beads/` files directly.

After `.beads/` bootstrap:
1. use `br` for issue creation and mutation,
2. use `bv` and read-only `br` for graph inspection,
3. keep execution-state truth out of ad hoc notes.

Bootstrap exception:
1. the initial `W032` bootstrap may create the first `.beads/` files directly,
2. later ordinary execution should treat that bootstrap as legacy initialization only.

## 6. OxCalc Bead Quality Bar
Every executable OxCalc bead should state:
1. one reviewable implementation or doctrine outcome,
2. the evidence needed for closure,
3. its parent epic,
4. real dependency relationships,
5. the truth surfaces touched when that matters.

For OxCalc, closure evidence normally means some combination of:
1. implementation code,
2. test code,
3. deterministic replay, trace, explain, or oracle evidence,
4. active spec or contract updates,
5. required upstream or downstream seam handoffs when a boundary changes,
6. current host-facing or seam-facing truth-surface updates where behavior claims changed.

Bad beads:
1. vague activity without a reviewable outcome,
2. ongoing themes disguised as one issue,
3. mini-worksets hidden inside one bead,
4. local-document-only output unless the bead is making a narrow doctrine correction, truth correction, or required handoff,
5. claims of closure without evidence where doctrine requires evidence.

## 6A. High-Signal Beads And Cleanup Bias
Beads should make the repo easier to change correctly.

A good bead produces one of:
1. engine behavior,
2. test or replay coverage,
3. a proof/model/check that can fail usefully,
4. a durable spec correction,
5. a boundary handoff with exact semantics,
6. an archive or cleanup move that reduces active-surface noise.

Low-value bead shapes:
1. broad classifications with no code, proof, replay, or spec consumer,
2. status tables that restate known limits without changing the next engineering move,
3. generated evidence kept active after its baseline role has expired,
4. duplicate documents that describe the same lane at different levels without adding precision,
5. workset-specific runner branches that remain in active source after their evidence role has expired.

Cleanup beads are first-class work when they:
1. move stale material out of active paths,
2. leave a shallow archive manifest,
3. update live references,
4. create explicit follow-up beads for any source or validation cleanup that would be unsafe to do in the same pass.

## 7. OxCalc Epic Shapes
Typical OxCalc epics:
1. engine/runtime implementation lane,
2. replay/evidence lane,
3. seam/handoff lane,
4. assurance/formal lane,
5. consumer/host contract lane,
6. cleanup/archive lane when the workset itself is migration or reduction work.

Not every workset needs every epic.
But any real OxCalc rollout should make the intended lane split explicit.

## 8. Rollout Rule
Any workset chosen for execution should be rolled out into one or more epics.

Rollout pattern:
1. some epics should be expanded into child beads immediately,
2. some epics may begin with a rollout bead when the child set still needs to be created or refreshed,
3. obvious implementation work should be expanded directly instead of hidden behind narrative placeholders,
4. both patterns are normal as long as the graph stays explicit.

A rollout bead is complete only when:
1. the epic has a believable ready path,
2. the next child beads exist explicitly,
3. the work no longer depends on narrative memory alone.

## 9. Closure Rule
A bead closes only when:
1. the stated outcome exists,
2. the stated evidence exists,
3. any newly discovered required work has already been added back into the graph,
4. the current truth surfaces touched by the bead are updated.

Do not close a bead because "enough progress happened."

Capability-bearing or seam-bearing OxCalc beads must normally close on meaningful code plus verification, or on a narrow doctrine change with the declared evidence and truth-surface updates.
Stub code, placeholder artifacts, and descriptive notes are not sufficient.

## 10. Documentation Rule
After planning is in place, default bead outputs should be:
1. implementation code,
2. test code,
3. narrowly-scoped spec or contract corrections,
4. narrowly-scoped upstream seam handoffs,
5. necessary evidence or reference notes for behavior that now exists in code.

Do not use beads as a reason to multiply local status documents.

## 11. Compact Rollout Template
When a workset is chosen for rollout, capture:

1. Workset:
   - id
   - title
   - scope
   - terminal condition
2. Execution epics:
   - implementation lane
   - evidence lane
   - seam/integration lane where needed
   - assurance lane where needed
   - cleanup/archive lane where needed
3. First rollout bead per epic:
   - title
   - one reviewable outcome
   - completion evidence
4. First execution child beads:
   - one clear outcome each
   - explicit dependencies
   - explicit evidence

## 12. Validator
Validator:
1. `scripts/check-worksets.ps1`

Planned minimum checks:
1. the workset register exists,
2. workset ids are unique,
3. the register exposes a coherent active sequence,
4. the bead workspace exists,
5. basic bead summary can be reported from `br` or the bootstrapped export.
