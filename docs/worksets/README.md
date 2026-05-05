# OxCalc Worksets

This directory holds OxCalc workset packets.

Worksets remain the scoped planning and provenance units for substantive repo work. They are not the live execution-state surface.

## Active Ownership

Use these surfaces together:

1. [WORKSET_REGISTER.md](/C:/Work/DnaCalc/OxCalc/docs/WORKSET_REGISTER.md) for ordered workset truth.
2. [BEADS.md](/C:/Work/DnaCalc/OxCalc/docs/BEADS.md) for the `workset -> epic -> bead` execution method.
3. `.beads/` for live readiness, in-progress state, blockers, and closure flow.

This directory is now a compact index plus the workset files themselves.

## Naming Rule

Sequential numbering: `W001`, `W002`, `W003`, ...

File pattern: `W<NNN>_<SLUG>.md`

Sequential numbering keeps dependency order visible and avoids ambiguity.

## What A Workset Owns

Each workset file should still define:

1. purpose and scope,
2. dependency position,
3. exit gate or closure condition,
4. evidence expectations when applicable,
5. any cross-repo seam or handoff implications.

Execution state should not be maintained here once the workset has been registered and broken into bead-level execution.

## Current Register-Tracked Line

The current ordered TreeCalc and doctrine line is tracked in [WORKSET_REGISTER.md](/C:/Work/DnaCalc/OxCalc/docs/WORKSET_REGISTER.md), including:

1. `W025` TreeCalc structural and formula substrate widening,
2. `W026` TreeCalc OxFml bind/reference and seam intake,
3. `W027` TreeCalc dependency graph and invalidation closure,
4. `W028` TreeCalc evaluator-backed candidate result integration,
5. `W029` TreeCalc runtime-derived effects and overlay closure,
6. `W030` TreeCalc corpus/oracle and first sequential baseline,
7. `W031` TreeCalc assurance refresh and residual packetization,
8. `W032` OxCalc beads migration and light doctrine reorientation,
9. `W033` OxCalc + OxFml core formalization pass,
10. `W034` core formalization deepening and implementation verification,
11. `W035` core formalization proof and assurance hardening,
12. `W036` core formalization verification closure expansion,
13. `W037` core formalization full-verification promotion gates,
14. `W038` core formalization release-grade closure hardening,
15. `W039` core formalization release-grade successor closure,
16. `W040` core formalization release-grade direct verification,
17. `W041` core formalization release-grade successor verification,
18. `W042` core formalization release-grade evidence closure expansion.

## Historical Note

Older worksets may still contain status/checklist wording that predates the bead-model migration. Treat those passages as historical provenance unless a newer active doctrine surface explicitly promotes them.
