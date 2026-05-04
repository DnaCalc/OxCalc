# W033 Formal Model Family Widening

Status: `calc-rcr_evidence_packet`
Workset: `W033`
Successor bead: `calc-rcr`
Created: 2026-05-04

## 1. Purpose

This packet records the post-W033 Lean and TLA widening slice for `calc-rcr`.

The target is an additive formal model-family slice over the surfaces named by the successor bead:

1. FEC bridge and publication fence compatibility,
2. reject/no-publish behavior,
3. dynamic and static dependency closure,
4. protected overlay retention,
5. narrow `LET`/`LAMBDA` carrier visibility,
6. replay-equivalent observable histories,
7. explicit non-promotion of Stage 2 contention.

The slice does not promote pack-grade replay, broad production diversity, full OxFunc semantics, or Stage 2 concurrency policy.

## 2. New Formal Artifacts

| Artifact | Role | Check command | Result |
|---|---|---|---|
| `formal/lean/OxCalc/CoreEngine/W033PostSlice.lean` | Checked Lean vocabulary and theorem slice for post-W033 bridge, fence, dependency, overlay, carrier, replay, and no-Stage-2-promotion facts. | `lean formal\lean\OxCalc\CoreEngine\W033PostSlice.lean` | passed |
| `formal/tla/CoreEnginePostW033.tla` | Bounded TLA smoke model for imported candidates, dependency closure, carrier visibility, protected overlays, publish/reject decisions, and no Stage 2 promotion. | `.\scripts\run-tlc.ps1 formal\tla\CoreEnginePostW033.tla formal\tla\CoreEnginePostW033.smoke.cfg` | passed |
| `formal/tla/CoreEnginePostW033.smoke.cfg` | Small routine TLC config for the post-W033 model. | same as above | passed |

The previous first-slice artifacts remain the floor:

1. `formal/lean/OxCalc/CoreEngine/Stage1State.lean`
2. `formal/lean/OxCalc/CoreEngine/W033FirstSlice.lean`
3. `formal/tla/CoreEngineStage1.tla`
4. `formal/tla/CoreEngineStage1.smoke.cfg`

## 3. Lean Coverage

The Lean artifact adds checked theorem families:

| Theorem or family | Meaning |
|---|---|
| `snapshotMismatch_cannotPublish` | A snapshot-mismatched commit/fence pair cannot satisfy the publish-compatibility predicate. |
| `compatibilityMismatch_cannotPublish` | A compatibility-basis mismatch cannot satisfy the publish-compatibility predicate. |
| `capabilityMismatch_cannotPublish` | A capability-token mismatch cannot satisfy the publish-compatibility predicate. |
| `applyPublishWithFence_atomic` | Publishing through a compatible fence updates public view and commit history atomically in the abstract model. |
| `applyReject_noPublish` | Reject intake does not mutate public view or commit history. |
| `dependencyClosure_containsStatic` | The declared closure contains all static dependencies. |
| `dependencyClosure_containsRuntime` | The declared closure contains all runtime dependencies. |
| `protectedOverlayRetained_refl` | Unchanged overlay state retains every protected overlay. |
| `protectedOverlaySafe_nil` | Empty overlay state is safe. |
| `visibleCarrier_requiresInvocationContract` | A dependency-visible or runtime-effect-visible callable carrier must have an invocation contract under the honest-bridge predicate. |
| `replayEquivalent_refl` and `replayEquivalent_symm` | Observable replay equivalence is reflexive and symmetric for the modeled history projection. |
| `rejectStep_noPublication` | A modeled reject observation has no publication id. |
| `postW033_noStage2ContentionPromotion` | The post-W033 formal envelope records Stage 2 contention as not promoted. |

## 4. TLA Coverage

The TLA model introduces a small state machine with these actions:

1. import candidate fact,
2. add static dependency and mark affected target,
3. add runtime dependency and mark affected target,
4. import a visible callable carrier with invocation contract,
5. reject candidate with no publication,
6. publish only through the compatible default fence,
7. retain a protected non-evictable overlay,
8. add an unprotected eviction-eligible overlay.

The smoke config checks these invariants:

1. `TypeInvariant`,
2. `RejectIsNoPublish`,
3. `RuntimeDependenciesAffected`,
4. `StaticDependenciesAffected`,
5. `CarrierVisibilityHasInvocationContract`,
6. `ProtectedOverlaySafety`,
7. `PublishRequiresCompatibleFence`,
8. `CandidateIsNotPublication`,
9. `NoStage2ContentionPromotion`.

TLC result:

| Metric | Value |
|---|---:|
| TLC version | `2.19 of 08 August 2024` |
| Distinct initial states | 1 |
| States generated | 814981 |
| Distinct states found | 49614 |
| States left on queue | 0 |
| Complete state graph depth | 5 |
| Result | no error found |

The smoke command uses the local TLA runner and `tla2tools.jar` resolved from the existing OxFml tool location when needed.

## 5. OxFml Formatting Baseline

The current OxFml working state contains upstream formatting and W073 documentation edits. OxCalc does not own those edits and this bead does not commit them.

The earlier workspace-wide formatter issue has cleared in the current dependency state:

```powershell
cargo fmt --all -- --check
```

Result: passed.

Future OxCalc validations can use the full workspace formatter gate again while the current OxFml formatting state remains available.

## 6. Replay And Projection Evidence Linkage

This bead does not add a new runtime behavior. It formalizes surfaces already exercised by deterministic replay/projection artifacts from the preceding post-W033 successor beads:

| Surface | Existing evidence root |
|---|---|
| Direct OxFml fixture projection for FEC accept/reject/fence facts | `docs/test-runs/core-engine/oxfml-fixture-bridge/post-w033-direct-oxfml-fixture-bridge-001/` |
| `LET`/`LAMBDA` carrier publication, invocation reject, dependency visibility, runtime-effect visibility, and replay identity | `docs/test-runs/core-engine/tracecalc-reference-machine/post-w033-let-lambda-carrier-witness-001/` and `docs/test-runs/core-engine/treecalc-local/post-w033-let-lambda-treecalc-witness-001/` |
| TreeCalc/CoreEngine independent projection comparison | `docs/test-runs/core-engine/independent-conformance/post-w033-independent-conformance-001/` |
| Pack-capability no-promotion decision | `docs/test-runs/core-engine/pack-capability/post-w033-pack-capability-decision-001/` |

The new Lean and TLA artifacts are therefore a formalization pass over replay-backed surfaces, not standalone prose specification.

## 7. Limits And Successor Carry

This bead does not claim:

1. a full Lean module-family split,
2. a full TLA model family for every FEC/F3E interleaving,
3. Stage 2 contention readiness,
4. pack-grade replay,
5. final callable transport authority,
6. full OxFunc semantic coverage,
7. direct OxFml evaluator re-execution inside OxCalc.

Open successor pressure remains:

1. deeper stale-fence matrices,
2. overlay retention beyond reflexive retention and smoke interleavings,
3. richer dynamic-dependency negative cases,
4. broader callable-carrier provenance and invocation policy,
5. Stage 2 contention model design,
6. scale/metamorphic semantic binding under `calc-8lg`.

## 8. Semantic-Equivalence Statement

This bead adds formal artifacts and documentation only.

It does not change coordinator scheduling, invalidation strategy, publication semantics, reject policy, TraceCalc behavior, TreeCalc behavior, OxFml fixture content, or pack capability decision logic.

Observable runtime behavior is invariant under this bead because no runtime producer, evaluator, coordinator transition, replay runner, or fixture expectation is changed.

## 9. Verification

Commands run:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `lean formal\lean\OxCalc\CoreEngine\Stage1State.lean` | passed |
| `lean formal\lean\OxCalc\CoreEngine\W033FirstSlice.lean` | passed |
| `lean formal\lean\OxCalc\CoreEngine\W033PostSlice.lean` | passed |
| `.\scripts\run-tlc.ps1 formal\tla\CoreEngineStage1.tla formal\tla\CoreEngineStage1.smoke.cfg` | passed |
| `.\scripts\run-tlc.ps1 formal\tla\CoreEnginePostW033.tla formal\tla\CoreEnginePostW033.smoke.cfg` | passed |
| `cargo test --workspace` | passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |
| `git diff --check` | passed; CRLF normalization warnings only |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed with `count: 0` |

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for in-scope items? | yes; this packet records the formal artifacts, coverage, limits, and evidence commands |
| 2 | Pack expectations updated for affected packs? | yes; this packet makes no pack-grade promotion and preserves `calc-lwh` no-promotion posture |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; Section 6 links the formal slice to existing deterministic replay/projection artifacts for FEC, LET/LAMBDA carriers, independent conformance, and pack-capability decisions |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 records that no runtime policy or strategy change was made |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no upstream mismatch was found, and no new handoff row is required |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared target? | yes for this widening slice; broader model-family gaps are explicit successor pressure |
| 8 | Completion language audit passed? | yes; this packet reports checked formal coverage without promoting broader proof, pack, or Stage 2 claims |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth did not change |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | not applicable; feature-map truth did not change |
| 11 | execution-state blocker surface updated? | yes; `calc-rcr` is represented in `.beads/` |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-rcr` asks for Lean/TLA widening over FEC bridge, fences, dynamic dependency, overlays, LET/LAMBDA carriers, replay histories, and deferred Stage 2 contention |
| Gate criteria re-read | pass; the target is a checked formal widening slice and evidence packet, not a broad proof-suite or Stage 2 promotion |
| Silent scope reduction check | pass; broader stale-fence, overlay, dynamic-dependency, callable, and contention work remains visible as successor pressure |
| "Looks done but is not" pattern check | pass; smoke-checked and theorem-backed slices are not represented as pack-grade, broad production, or full concurrency proof |
| Result | pass for the `calc-rcr` declared formal model-family widening target |

## 12. Three-Axis Report

- execution_state: `calc-rcr_evidence_packet_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - broader Lean module-family split remains open
  - deeper TLA model families remain open
  - Stage 2 contention remains unpromoted
  - pack-grade replay remains unpromoted
  - scale/metamorphic semantic binding remains successor-scoped to `calc-8lg`
