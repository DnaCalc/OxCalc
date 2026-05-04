# W034 TLA Model-Family And Contention Preconditions

Status: `calc-e77.5_tla_model_family`
Workset: `W034`
Parent epic: `calc-e77`
Bead: `calc-e77.5`
Created: 2026-05-05

## 1. Purpose

This packet records the W034 TLA+ model-family and contention-precondition slice.

The target is to add a checked interleaving model for:

1. stale publication-fence decisions across snapshot, compatibility-basis, and capability-view axes,
2. static, runtime, and dynamic-shape dependency update interleavings,
3. pinned-reader protected-overlay retention and release/eviction safety,
4. Stage 2 contention precondition blocking without promoting Stage 2 policy.

This slice does not claim full TLA+ verification of the core engine, full concurrency policy proof, pack-grade replay, continuous scale assurance, or Stage 2 promotion.

## 2. TLA Artifact Set

New checked artifacts:

| Artifact | Role |
|---|---|
| `formal/tla/CoreEngineW034Interleavings.tla` | W034 interleaving model for stale fences, dependency updates, pinned overlays, protected release/eviction, and Stage 2 contention-precondition blocking |
| `formal/tla/CoreEngineW034Interleavings.smoke.cfg` | routine smoke config with singleton node/reader/overlay, two snapshots, two compatibility bases, two capability views, and bounded transition depth |

Existing TLA smoke artifacts were rechecked as the base family:

1. `formal/tla/CoreEngineStage1.tla`
2. `formal/tla/CoreEngineStage1.smoke.cfg`
3. `formal/tla/CoreEnginePostW033.tla`
4. `formal/tla/CoreEnginePostW033.smoke.cfg`

## 3. Checked W034 Invariants

The W034 smoke config checks:

| Invariant | Meaning |
|---|---|
| `TypeInvariant` | all model variables remain inside declared state families |
| `RejectIsNoPublish` | reject decisions carry no publication id |
| `PublishRequiresCompatibleFence` | publish decisions require a compatible fence decision |
| `NoStaleFencePublication` | committed records are only created through the compatible-fence action |
| `StaticDependenciesAffected` | every static dependency target is included in the affected set |
| `RuntimeDependenciesAffected` | every runtime dependency target is included in the affected set |
| `DynamicShapeDependenciesAffected` | every dynamic-shape dependency target is included in the affected set |
| `ProtectedOverlayPinnedAndRetained` | protected overlays remain pinned, not eviction-eligible, and not evicted |
| `EvictedOverlayWasUnprotected` | eviction can only occur after protection has been released |
| `NoStage2ContentionPromotion` | Stage 2 remains unpromoted |
| `Stage2PreconditionsStillMissing` | required Stage 2 evidence is intentionally not all available in the checked model |

The Stage 2 evidence guard in the smoke config is:

1. `RequiredStage2Evidence = {stage1_tla, replay_equivalence, pack_gate}`
2. `AvailableStage2Evidence = {stage1_tla}`
3. Therefore `MissingStage2Evidence` remains non-empty and any contention attempt is recorded as `stage2_blocked`.

## 4. TLC Runs

Commands:

```powershell
scripts\run-tlc.ps1 formal\tla\CoreEngineStage1.tla formal\tla\CoreEngineStage1.smoke.cfg
scripts\run-tlc.ps1 formal\tla\CoreEnginePostW033.tla formal\tla\CoreEnginePostW033.smoke.cfg
scripts\run-tlc.ps1 formal\tla\CoreEngineW034Interleavings.tla formal\tla\CoreEngineW034Interleavings.smoke.cfg
```

Observed results:

| Model | TLC version | States generated | Distinct states | Queue left | Depth | Result |
|---|---|---:|---:|---:|---:|---|
| `CoreEngineStage1.smoke` | `2.19 of 08 August 2024` | 4,855 | 908 | 0 | 5 | no error found |
| `CoreEnginePostW033.smoke` | `2.19 of 08 August 2024` | 814,981 | 49,614 | 0 | 5 | no error found |
| `CoreEngineW034Interleavings.smoke` | `2.19 of 08 August 2024` | 247,984 | 19,373 | 0 | 5 | no error found |

The W034 smoke config is intentionally bounded for routine checking. A broader multi-node or deeper transition exploration is not claimed by this bead.

## 5. Obligation Mapping

| W034 obligation | Evidence in this bead | Carry after this bead |
|---|---|---|
| `W034-OBL-003` overlay retention and eviction pressure | W034 TLA model checks protected overlays are retained under active pins and can only be evicted after release | broader overlay economics and non-routine interleavings remain pack/scale work |
| `W034-OBL-008` TLA+ model-family depth | W034 interleaving model and smoke config check stale-fence, dependency-update, overlay, and contention-precondition invariants | full TLA+ verification and broader configs remain open beyond this bead |
| `W034-OBL-009` Stage 2 contention | `NoStage2ContentionPromotion` and `Stage2PreconditionsStillMissing` check the no-promotion/precondition posture | `calc-e77.6` still owns pack/capability and continuous scale gate binding before any future promotion talk |
| `W034-OBL-016` evidence non-mutation | new TLA artifacts are additive; existing Stage1/PostW033 smoke models were rerun, not rewritten | broader W034 closure audit remains `calc-e77.7` |

## 6. Semantic-Equivalence Statement

This bead adds checked TLA+ artifacts and documentation only. It does not change coordinator scheduling, invalidation strategy, publication semantics, reject policy, TraceCalc transition behavior, TreeCalc execution behavior, OxFml fixture content, Lean models, pack decisions, or formatting/display seam meaning.

Observable runtime behavior is invariant under this bead. The model-checking work states and checks bounded interleaving properties over abstract state shapes; it introduces no runtime producer and changes no execution path.

## 7. Limits

Known limits after this bead:

1. the W034 TLA evidence is a bounded smoke model, not exhaustive TLA+ verification of all core-engine behavior,
2. the routine config uses one node, one reader, and one overlay id, while widening fence axes through two snapshots, two compatibility bases, and two capability views,
3. no Stage 2 policy is promoted,
4. no pack-grade replay or continuous scale claim is promoted,
5. no production scheduler equivalence proof is added,
6. no direct OxFml TLA artifact is imported or rechecked by this bead.

## 8. Repository Validation

| Command | Result |
|---|---|
| `scripts\run-tlc.ps1 formal\tla\CoreEngineStage1.tla formal\tla\CoreEngineStage1.smoke.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEnginePostW033.tla formal\tla\CoreEnginePostW033.smoke.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW034Interleavings.tla formal\tla\CoreEngineW034Interleavings.smoke.cfg` | passed |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed with CRLF normalization warnings only |

No Rust tests are required for this documentation/TLA-only bead because it emits no Rust code, fixture, runner, or replay artifact.

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for in-scope items? | yes; this packet records artifacts, checked invariants, TLC output, limits, and obligation mapping |
| 2 | Pack expectations updated for affected packs? | yes; no pack promotion is made, and pack/capability remains mapped to `calc-e77.6` |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; this TLA slice links to the W034 TraceCalc/conformance surfaces already recorded and adds model-check evidence rather than new runtime behavior |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 6 records that no runtime policy or strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml-owned seam contradiction or handoff trigger appeared in this TLA bead |
| 6 | All required tests pass? | yes; see Section 8 |
| 7 | No known semantic gaps remain in declared target? | yes for this TLA model-family target; broader TLA verification, pack/capability, continuous scale, and Stage 2 promotion remain mapped to later W034 lanes |
| 8 | Completion language audit passed? | yes; this packet does not claim full TLA+ verification, Stage 2 promotion, pack-grade replay, or full concurrency proof |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth did not change |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; W034 current-state text now records this TLA model-family slice |
| 11 | execution-state blocker surface updated? | yes; `calc-e77.5` is represented in `.beads/` |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-e77.5` asks for widened TLA models/configs for stale-fence interleavings, overlay GC/pinned reader safety, dependency update interleavings, and Stage 2 contention preconditions without promoting Stage 2 policy |
| Gate criteria re-read | pass; TLC runs and model limits are recorded, and no broad proof or promotion is overclaimed |
| Silent scope reduction check | pass; routine config bounds and broader TLA/Stage 2/pack limits are explicit |
| "Looks done but is not" pattern check | pass; smoke model-check evidence is not represented as full formal verification or production scheduler proof |
| Result | pass for the `calc-e77.5` declared TLA model-family and contention-precondition target |

## 11. Three-Axis Report

- execution_state: `calc-e77.5_tla_model_family_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-e77.6` pack capability and continuous scale gate binding
  - `calc-e77.7` W034 closure audit and successor packetization
  - broader non-routine TLA exploration and full concurrency proof beyond this checked slice
