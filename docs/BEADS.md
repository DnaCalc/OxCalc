# OxCalc Bead Pocket Reference

`OPERATIONS.md` owns the execution doctrine. This file is a short local reference for bead mechanics.

## Core Model

OxCalc executes through:

```
Spec -> Worksets -> Epic beads -> Child beads
```

1. `docs/SPEC.md` indexes spec and design truth.
2. `docs/WORKSET_REGISTER.md` owns large work areas, default sequence, and coarse history.
3. `.beads/` owns live execution truth through `br`.
4. Worksets are planning containers; beads are executable units.

## Tool Rule

`br` is the mutation tool. Do not edit `.beads/` files directly.

Useful commands:

```powershell
br ready
br list --status in_progress
br epic status
br show <id>
br create --title "..." --type epic --priority 2
br create --title "..." --type task --priority 2 --parent <epic-id>
br update <id> --status in_progress
br dep add <id> <depends-on-id>
br close <id> --reason "..."
```

`bv` is for graph-aware inspection. Agents should use only non-interactive inspection commands.

## Bead Quality

A useful bead states:

1. one reviewable outcome,
2. the smallest useful acceptance check or observation,
3. parent epic/workset,
4. real dependencies,
5. touched truth surfaces when relevant.

Good bead outputs are usually code, tests, replay/check artifacts, compact spec corrections, handoffs, or cleanup that reduces ambiguity.

Avoid beads that produce status tables, broad taxonomies, placeholder code, or duplicate docs without changing behavior, checks, or a decision.

## Bead Loop

1. Pick a ready bead with `br ready` and inspect it with `br show`.
2. Mark it in progress.
3. Do the one outcome.
4. Update the spec/design surfaces it touches.
5. Run the relevant useful check.
6. Review with fresh eyes: look for mistakes, hidden gaps, stale wording, brittle fixture-only behavior, or misplaced artifacts.
7. File out-of-scope follow-up work as new beads.
8. Commit the work when appropriate.
9. Close the bead with a reason that names the decisive check or observation.

## Workset Epics

Every active non-bootstrap workset should have an epic bead once it enters execution.

Some epics expand directly into child beads. Others start with a rollout bead that creates or refreshes the child set. Either pattern is acceptable when the graph stays explicit and `br` owns readiness, blockers, and child closure.

## Housekeeping

At workset boundaries, bring touched artifacts to a known state:

1. keep and place,
2. mark as parked/superseded/limited,
3. delete when genuinely dead.

The goal is clarity, not archival ceremony.
