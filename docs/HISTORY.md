# OxCalc History Pointers

Status: `active_history_pointer`
Last updated: 2026-04-03

## 1. Purpose
This file is a compact pointer for historical execution-surface changes that are
no longer supposed to live as active doctrine.

It does not replace git history.
It exists only so the active tree can stay light while still giving readers a
clear path back to retired execution surfaces.

## 2. Current Doctrine Shift
`W032` moved OxCalc to:
1. [docs/WORKSET_REGISTER.md](WORKSET_REGISTER.md) for ordered workset truth,
2. [docs/BEADS.md](BEADS.md) for the local bead method,
3. `.beads/` for live execution-state truth.

## 3. Retired Active Surfaces
The following surfaces no longer own ordinary live execution truth:
1. `CURRENT_BLOCKERS.md`
2. status narratives inside `docs/worksets/README.md`
3. detailed execution-state tracking inside `docs/IN_PROGRESS_FEATURE_WORKLIST.md`

These surfaces remain only as:
1. historical pointers,
2. compact maps,
3. provenance packets,
4. git-history-backed records.

## 4. Use Rule
Use:
1. [docs/WORKSET_REGISTER.md](WORKSET_REGISTER.md) for current workset order,
2. [docs/BEADS.md](BEADS.md) for execution method,
3. `.beads/` for live execution state.

Do not reopen retired active surfaces as a second tracker.
