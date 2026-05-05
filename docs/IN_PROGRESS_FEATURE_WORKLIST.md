# IN_PROGRESS_FEATURE_WORKLIST.md — OxCalc

Status: `active_feature_map`
Last updated: 2026-05-05

Purpose:
1. provide a compact repo-level map of the major OxCalc lanes that remain live,
2. point readers at the current owning workset or contract surface,
3. avoid duplicating execution-state, blocker, or status detail now owned by `.beads/`.

Use rule:
1. use this file as a high-level feature map only,
2. use [WORKSET_REGISTER.md](C:\Work\DnaCalc\OxCalc\docs\WORKSET_REGISTER.md) for ordered workset truth,
3. use `.beads/` for ready, blocked, in-progress, and closed execution state,
4. use [docs/worksets/README.md](C:\Work\DnaCalc\OxCalc\docs\worksets\README.md) for compact workset/provenance indexing,
5. use the canonical spec docs for actual semantic truth.

Supersession note:
1. this file no longer acts as a second execution tracker,
2. detailed doctrine migration truth belongs to [W032_OXCALC_BEADS_MIGRATION_AND_LIGHT_DOCTRINE_REORIENTATION.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W032_OXCALC_BEADS_MIGRATION_AND_LIGHT_DOCTRINE_REORIENTATION.md),
3. detailed TreeCalc packet sequencing belongs to [WORKSET_REGISTER.md](C:\Work\DnaCalc\OxCalc\docs\WORKSET_REGISTER.md) plus the owning worksets.

## Active Feature Map

### IP-01 Stage 1 Engine And Replay Baseline
- Current state: active; the Rust-first Stage 1 structural, coordinator, recalc, replay, and retained-witness floors exist and remain the basis for later widening.
- Canonical owner: [W024_EXECUTION_SEQUENCE_J_BROADER_PROGRAM_SCOPE_PACK_PROMOTION.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W024_EXECUTION_SEQUENCE_J_BROADER_PROGRAM_SCOPE_PACK_PROMOTION.md), the canonical core-engine spec set in `docs/spec/core-engine/`, and the replay/evidence docs under `docs/spec/core-engine/`.

### IP-02 OxFml Downstream Integration
- Current state: active; OxFml V1 runtime/replay consumer uptake is landed, and broader downstream rounds now reopen only on later real pressure.
- Canonical owner: [W020_OXFML_DOWNSTREAM_INTEGRATION_ROUND_01.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W020_OXFML_DOWNSTREAM_INTEGRATION_ROUND_01.md) and [CORE_ENGINE_OXFML_SEAM.md](C:\Work\DnaCalc\OxCalc\docs\spec\core-engine\CORE_ENGINE_OXFML_SEAM.md).

### IP-03 TreeCalc Semantic Completion
- Current state: active; `W025` and `W026` reached gate for the first TreeCalc structural and consumed-seam packet floors, `W027` now has a first exercised dependency/invalidation substrate floor, and `W028` through `W031` remain the active widening line.
- Canonical owner: [CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md](C:\Work\DnaCalc\OxCalc\docs\spec\core-engine\CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md), [W025_TREECALC_STRUCTURAL_AND_FORMULA_SUBSTRATE_WIDENING.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W025_TREECALC_STRUCTURAL_AND_FORMULA_SUBSTRATE_WIDENING.md), [W026_TREECALC_OXFML_BIND_REFERENCE_AND_SEAM_INTAKE.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W026_TREECALC_OXFML_BIND_REFERENCE_AND_SEAM_INTAKE.md), and the active successor worksets in `W027` through `W031`.

### IP-04 Formalization And Assurance
- Current state: active; formalization direction, replay/pack binding, and measurement/assurance planning exist. `W033` established the OxCalc + OxFml first-pass formalization/spec-evolution packet and post-W033 successor evidence slices. `W034` records bounded successor evidence, not full formalization. `W035` now has closure-audit and successor-packet evidence: its TraceCalc oracle matrix records 30 scenarios, 17 matrix rows, 15 covered rows, 2 classified uncovered rows, and 0 failed/missing rows; its implementation-conformance evidence records 6 W034 gap dispositions, 5 implementation-work deferrals, 1 spec-evolution deferral, and 0 failed rows; its Lean evidence distinguishes local proof rows, zero explicit axiom rows, OxFml external seam assumptions including W073, OxFunc opaque kernel boundaries, and TLA-owned deferrals; its TLA evidence includes scheduler-gate, partition-gap, and multi-reader overlay configs with no Stage 2 promotion; its continuous-assurance evidence records 5 source rows, 3 scheduled lanes, 4 differential rows, 0 missing artifacts, 0 unexpected mismatches, and 9 no-promotion reasons; its pack/Stage 2 reassessment records 10 evidence rows, 10 satisfied inputs, 19 no-promotion blockers, 0 missing artifacts, and `cap.C4.distill_valid` as the highest honest capability. `W036` is the next ordered successor for TraceCalc coverage closure criteria, optimized/core-engine conformance closure, Lean theorem coverage expansion, concrete TLA Stage 2 partition modeling, independent evaluator diversity/cross-engine differential evidence, continuous-assurance operation/history, and pack-grade reassessment.
- Canonical owner: [W006_CORE_FORMALIZATION_AND_GATE_BINDING.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W006_CORE_FORMALIZATION_AND_GATE_BINDING.md) through [W010_EXPERIMENT_REGISTER_AND_MEASUREMENT_SCHEMA_PLANNING.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W010_EXPERIMENT_REGISTER_AND_MEASUREMENT_SCHEMA_PLANNING.md), plus [W031_TREECALC_ASSURANCE_REFRESH_AND_RESIDUAL_PACKETIZATION.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W031_TREECALC_ASSURANCE_REFRESH_AND_RESIDUAL_PACKETIZATION.md), [W033_OXCALC_OXFML_CORE_FORMALIZATION_PASS.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W033_OXCALC_OXFML_CORE_FORMALIZATION_PASS.md), [W034_CORE_FORMALIZATION_DEEPENING_AND_IMPLEMENTATION_VERIFICATION.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W034_CORE_FORMALIZATION_DEEPENING_AND_IMPLEMENTATION_VERIFICATION.md), [W035_CORE_FORMALIZATION_PROOF_AND_ASSURANCE_HARDENING.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W035_CORE_FORMALIZATION_PROOF_AND_ASSURANCE_HARDENING.md), [W036_CORE_FORMALIZATION_VERIFICATION_CLOSURE_EXPANSION.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W036_CORE_FORMALIZATION_VERIFICATION_CLOSURE_EXPANSION.md), [CORE_ENGINE_OXCALC_OXFML_FORMALIZATION_PASS_PLAN.md](C:\Work\DnaCalc\OxCalc\docs\spec\core-engine\CORE_ENGINE_OXCALC_OXFML_FORMALIZATION_PASS_PLAN.md), [W034_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md](C:\Work\DnaCalc\OxCalc\docs\spec\core-engine\w034-formalization\W034_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md), and [W035_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md](C:\Work\DnaCalc\OxCalc\docs\spec\core-engine\w035-formalization\W035_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md).

### IP-05 Execution Doctrine Migration
- Current state: active; the workset register, bead method, validator, and `.beads/` bootstrap now exist, and active repo-level docs now defer live execution truth to the bead graph.
- Canonical owner: [W032_OXCALC_BEADS_MIGRATION_AND_LIGHT_DOCTRINE_REORIENTATION.md](C:\Work\DnaCalc\OxCalc\docs\worksets\W032_OXCALC_BEADS_MIGRATION_AND_LIGHT_DOCTRINE_REORIENTATION.md), [WORKSET_REGISTER.md](C:\Work\DnaCalc\OxCalc\docs\WORKSET_REGISTER.md), and [BEADS.md](C:\Work\DnaCalc\OxCalc\docs\BEADS.md).

### IP-06 Operation Model, Undo/Redo, And Collaboration Positioning
- Current state: planned; OxCalc now has a single architecture/design/work-plan packet that positions the `OpLog` realization, undo/redo, live-editing substrate, and OxReplay replay-export relation as OxCalc-owned execution concerns rather than Foundation code or OxReplay scope.
- Canonical owner: [CORE_ENGINE_OPLOG_UNDO_REDO_AND_COLLAB_ARCHITECTURE_PLAN.md](C:\Work\DnaCalc\OxCalc\docs\spec\core-engine\CORE_ENGINE_OPLOG_UNDO_REDO_AND_COLLAB_ARCHITECTURE_PLAN.md).

## Status Vocabulary
- `planned`: accepted lane, no active execution claim implied here.
- `active`: live lane with current owner surfaces.
- `parked`: current baseline or parked authority exists; reopen only by explicit future work.

Current reading:
1. `IP-01` through `IP-05` are `active`.
2. `IP-06` is `planned`.
