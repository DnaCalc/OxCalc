# W036 Lean Theorem Coverage Expansion

Status: `calc-rqq.4_lean_theorem_coverage_expansion_validated`
Workset: `W036`
Parent epic: `calc-rqq`
Bead: `calc-rqq.4`

## 1. Purpose

This packet expands the W035 Lean proof inventory with W036 theorem-family classification rows for conformance match guards, harness first-fix rows, callable boundary inventory, external seam assumptions, opaque OxFunc kernel boundaries, and deferred TLA/conformance lanes.

The target is proof-inventory deepening, not full Lean verification. The new Lean artifacts make the W036 boundaries machine-checked while keeping full engine proof, full OxFunc callable semantics, and Stage 2/TLA-owned interleavings unpromoted.

## 2. Authority Inputs Reviewed

| Input | Role in this bead |
|---|---|
| `docs/spec/core-engine/w035-formalization/W035_LEAN_ASSUMPTION_DISCHARGE_AND_SEAM_PROOF_MAP.md` | W035 proof inventory source |
| `formal/lean/OxCalc/CoreEngine/W035AssumptionDischarge.lean` | predecessor checked Lean assumption inventory |
| `formal/lean/OxCalc/CoreEngine/W035SeamProofMap.lean` | predecessor checked seam proof map |
| `docs/spec/core-engine/w036-formalization/W036_RESIDUAL_COVERAGE_AND_PROMOTION_BLOCKER_LEDGER.md` | W036 obligations and owners |
| `docs/spec/core-engine/w036-formalization/W036_TRACECALC_COVERAGE_CLOSURE_CRITERIA_AND_MATRIX_EXPANSION.md` | W036 TraceCalc coverage/exclusion source |
| `docs/spec/core-engine/w036-formalization/W036_OPTIMIZED_CORE_ENGINE_CONFORMANCE_CLOSURE_PLAN_AND_FIRST_FIXES.md` | W036 conformance action and blocker source |
| `docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/` | W036 action, blocker, and match-promotion guard evidence |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current inbound OxFml observation ledger |

## 3. Lean Artifact Surface

| Artifact | Role |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W036LeanCoverageExpansion.lean` | checked W036 coverage inventory for match guards, harness-first-fix rows, TLA deferrals, external seams, opaque kernel rows, explicit-axiom count, and no full Lean promotion |
| `formal/lean/OxCalc/CoreEngine/W036CallableBoundaryInventory.lean` | checked callable boundary inventory separating OxCalc callable-carrier proof rows from callable metadata deferral, OxFml seam facts, and full OxFunc LAMBDA kernel opacity |

The W036 files introduce no Lean `axiom` declarations.

## 4. Coverage Expansion

| Family | W036 checked result | Promotion consequence |
|---|---|---|
| declared implementation gaps | `declaredGapNotMatchGuard_promotableAsLocalTheorem` records the guard that declared gaps are not matches | local guard proof, not optimized/core-engine verification |
| match-promotion guard | `matchPromotionGuard_promotableAsLocalTheorem` records zero W036 match-promoted rows | local guard proof, not full conformance |
| dynamic dependency bind | `dynamicBindHarness_notPromotableAsLocalTheorem` records harness evidence as non-promoting | remains optimized/conformance work |
| dynamic negative/shape update | `dynamicNegativeHarness_notPromotableAsLocalTheorem` records harness evidence as non-promoting | remains optimized/conformance work |
| snapshot and capability-view fences | `snapshotFence_deferredToFormalLane` and `capabilityViewFence_deferredToFormalLane` route the rows to the TLA/coordinator lane | owned by `calc-rqq.5` |
| multi-reader overlay release ordering | `multiReaderOverlay_deferredToFormalLane` routes the uncovered row to the TLA lane | owned by `calc-rqq.5` |
| W073 typed conditional-formatting metadata | `w073TypedFormatting_externalBoundary` records OxFml external ownership | no OxCalc-local proof promotion |
| full OxFunc callable kernel | `fullOxFuncCallableKernel_opaqueBoundary` records OxFunc opacity | no OxCalc-local proof promotion |

## 5. Callable Boundary Inventory

| Callable row | Checked result | Consequence |
|---|---|---|
| direct callable carrier | `directCallableCarrier_promotableAsCarrierProof` | OxCalc carrier fragment remains locally classified |
| runtime-effect visibility | `runtimeEffectVisibility_promotableAsCarrierProof` | OxCalc-visible carrier/effect row remains locally classified |
| callable metadata projection | `callableMetadataProjection_notPromotableAsCarrierProof`, `callableMetadataProjection_requiresConformanceFollowup`, `callableMetadataProjection_hasBlocker` | routes to `calc-rqq.4`; not a carrier proof |
| host-sensitive lambda effect | `hostSensitiveLambdaEffect_opaqueOrExternal` | OxFunc-opaque boundary |
| full OxFunc LAMBDA kernel | `fullOxFuncLambdaKernel_opaqueOrExternal`, `fullOxFuncLambdaKernel_notPromotableAsCarrierProof` | OxFunc-opaque boundary |
| OxFml callable-carrier seam | `oxfmlCallableCarrierSeam_requiresExternalAuthority` | OxFml external seam assumption |

`directCarrier_isSeparatedFromFullOxFuncKernel` checks that an OxCalc carrier-fragment proof can coexist with an opaque full OxFunc kernel boundary without promoting the kernel.

## 6. Summary Counts

| Surface | Local theorem/carrier rows | Explicit axiom rows | External seam rows | Opaque kernel rows | TLA deferred rows | Harness first-fix rows | Match-promoted rows | Full promotion |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| W036 coverage expansion | 2 | 0 | 2 | 1 | 3 | 2 | 0 | no |
| W036 callable boundary inventory | 2 | 0 | 1 | 2 | 0 | 0 | 0 | no |

The count rows are intentionally proof-inventory facts. They are not total proof coverage over the Rust engine.

## 7. OxFml Watch

No OxFml handoff is filed by this bead.

W073 typed conditional-formatting metadata remains an OxFml-owned external seam assumption. The W036 callable boundary inventory records the OxFml callable-carrier seam as external authority and the full OxFunc LAMBDA kernel as opaque.

## 8. Semantic-Equivalence Statement

This bead adds checked Lean files and spec text only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, pack-decision logic, continuous-assurance runner semantics, TLA model semantics, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The Lean artifacts classify proof ownership and promotion boundaries; they do not change executable calculator behavior.

## 9. Verification

| Command | Result |
|---|---|
| `lean formal\lean\OxCalc\CoreEngine\Stage1State.lean` | passed |
| `lean formal\lean\OxCalc\CoreEngine\W033FirstSlice.lean` | passed |
| `lean formal\lean\OxCalc\CoreEngine\W033PostSlice.lean` | passed |
| `lean formal\lean\OxCalc\CoreEngine\W034PublicationFences.lean` | passed |
| `lean formal\lean\OxCalc\CoreEngine\W034DependenciesOverlays.lean` | passed |
| `lean formal\lean\OxCalc\CoreEngine\W034LetLambdaReplay.lean` | passed |
| `lean formal\lean\OxCalc\CoreEngine\W034RefinementObligations.lean` | passed |
| `lean formal\lean\OxCalc\CoreEngine\W035AssumptionDischarge.lean` | passed |
| `lean formal\lean\OxCalc\CoreEngine\W035SeamProofMap.lean` | passed |
| `lean formal\lean\OxCalc\CoreEngine\W036LeanCoverageExpansion.lean` | passed |
| `lean formal\lean\OxCalc\CoreEngine\W036CallableBoundaryInventory.lean` | passed |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed; `worksets=14`, `beads total=89`, `open=6`, `in_progress=1`, `ready=0`, `blocked=5`, `closed=82` |
| `br dep cycles --json` | passed; `count=0` |
| `git diff --check` | passed; CRLF normalization warnings only |

Cargo tests are not required for this bead because it introduces no Rust behavior or Rust source changes.

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet and W036 status surfaces record the Lean expansion |
| 2 | Pack expectations updated for affected packs? | yes; pack remains unpromoted and `calc-rqq.8` owns reassessment after later W036 evidence |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for carried behavior evidence; this bead adds checked Lean classification artifacts and consumes W036 conformance/TraceCalc evidence |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml/FEC/F3E change or handoff trigger exists for this bead |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for this target; local theorem rows, external seams, opaque kernels, TLA deferrals, and conformance/harness rows are explicitly classified |
| 8 | Completion language audit passed? | yes; no full Lean verification, full OxFunc semantics, optimized/core-engine verification, pack, or Stage 2 promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 points at this W036 Lean coverage evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-rqq.4` execution state and later closure evidence |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-rqq.4` asks for expanded theorem families and explicit separation of proved local theorems, remaining external seam assumptions, opaque OxFunc facts, and deferred obligations |
| Gate criteria re-read | pass; the W036 Lean artifacts distinguish local theorem rows, external seam rows, opaque kernel rows, TLA deferrals, conformance/harness rows, explicit-axiom count, and promotion flags |
| Silent scope reduction check | pass; the callable metadata projection, host-sensitive lambda effect, full OxFunc kernel, snapshot/capability fences, and multi-reader overlay row are explicitly carried rather than silently omitted |
| "Looks done but is not" pattern check | pass; checked Lean inventory is not reported as full Lean verification or full engine proof |
| Result | pass for the `calc-rqq.4` Lean theorem coverage expansion target |

## 12. Three-Axis Report

- execution_state: `calc-rqq.4_lean_theorem_coverage_expansion_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-rqq.5` through `calc-rqq.9` remain open
  - full Lean verification remains partial because these files are checked proof-inventory slices, not total Rust-engine proof
  - TLA/coordinator rows for snapshot/capability fences and multi-reader overlays remain routed to `calc-rqq.5`
  - optimized/core-engine conformance remains partial because zero W035 declared gaps are promoted as matches
  - independent evaluator diversity, cross-engine differential service, continuous assurance operation/history, pack-grade replay, and Stage 2 policy remain partial
