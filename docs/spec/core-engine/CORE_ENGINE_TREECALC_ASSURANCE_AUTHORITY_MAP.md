# Core Engine TreeCalc Assurance Authority Map

Status: `w031_inventory`
Owner workset: `W031_TREECALC_ASSURANCE_REFRESH_AND_RESIDUAL_PACKETIZATION.md`

## 1. Purpose

This map ties the older Stage 1 assurance-planning packets to the current TreeCalc local runtime evidence after `W030`.

It is repo-local authority inventory for `calc-ukb.1`; it does not change the OxFml seam, does not patch sibling repos, and does not claim broader `DNA TreeCalc` product readiness.

Terminology:
1. `DNA TreeCalc` is the future separate repo/product.
2. `OxCalcTree` is the OxCalc-owned host-facing tree runtime contract.
3. `TreeCalc` below means OxCalc-local tree-substrate/runtime evidence unless explicitly prefixed.

## 2. Current Evidence Anchors

| Anchor | Current authority role |
|---|---|
| `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` | Host-facing contract that preserves candidate/publication/reject, dependency/invalidation, runtime-effect, overlay, result, and diagnostic reachability. |
| `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md` | Planning companion that maps `W025` through `W031` to the first sequential TreeCalc-ready floor and names residual semantic breadth. |
| `docs/test-runs/core-engine/treecalc-local/w030-treecalc-oracle-baseline/` | Checked-in W030 baseline evidence for the current broader local TreeCalc corpus. |
| `docs/test-runs/core-engine/treecalc-local/w030-treecalc-oracle-baseline/run_summary.json` | Baseline summary: 17 cases, 9 published, 7 rejected, 1 verified clean, zero expectation mismatches. |
| `docs/test-runs/core-engine/treecalc-local/w030-treecalc-oracle-baseline/conformance/conformance_summary.json` | Conformance summary: 17/17 pass, zero mismatch cases. |
| `docs/test-runs/core-engine/treecalc-local/w030-treecalc-oracle-baseline/replay_artifact_manifest.json` | Replay artifact inventory for root, conformance, per-case, trace, explain, and post-edit artifacts. |

## 3. W008 Assurance Boundary Inventory

`W008` names the abstract coordinator/fence model surface: `structSnapshot`, `runtimeView`, `coordState`, `inFlight`, `acceptedCandidate`, `publishedView`, `pinnedReaders`, `overlayState`, `rejectLog`, `nodeCalcState`, `demandSet`, and `evictionEligibility`.

Current TreeCalc refresh state:

| W008 assumption or action | Current TreeCalc authority | Refreshed by W030? | Residual candidate |
|---|---|---:|---|
| Candidate result remains distinct from publication (`A5` versus `A7`) | `OxCalcTreeRecalcResult` exposes optional accepted candidate, optional publication bundle, run state, diagnostics, and emitted result artifacts. | yes, for W030 local sequential corpus | Broader evaluator-backed candidate identity breadth remains tied to later OxFml seam widening if new candidate families appear. |
| Reject preserves published view (`A6`, `S2`) | W030 rejected cases emit reject detail and no published-value mutation in per-case result/published-values artifacts. | yes, for covered reject families | Provider-failure/callable-publication remain watch lanes per OxFml notes until coordinator-visible evidence exists. |
| Atomic publication (`A7`, `S1`) | Published cases emit stable result and published-values artifacts through the coordinator-owned publication path. | yes, for single-run local sequential corpus | Multi-node atomic bundle stress beyond current corpus remains successor assurance work, not W031 inventory closure. |
| Verified clean without synthetic publication (`A3b`) | W030 includes one `verified_clean` case with no expectation mismatch. | yes, for first local equality surface | Equality breadth for future richer values/formats remains residual. |
| Pinned-reader stability and eviction eligibility (`A8`-`A10`, `S5`-`S6`) | Preserved as Stage 1 model authority and OxCalc guardrail; W030 local baseline does not add new pinned-reader corpus coverage. | no | Packetize as future assurance/corpus widening before concurrency or retention-policy promotion. |
| Overlay state is explicit and not hidden mutable truth | W030 runtime-effect overlay artifacts and replay manifest keep overlay sidecars visible. | yes, for covered runtime-effect overlay families | Wider overlay-retention/economics counters remain W010/W031 successor residuals. |

## 4. W009 Replay And Pack Binding Inventory

`W009` names replay classes `R1` through `R8` and first pack bindings. W030 provides TreeCalc local artifacts analogous to the older `TraceCalc` lane but does not by itself promote every pack to a broader program-grade claim.

| W009 replay class / pack | Current TreeCalc evidence | Refresh state | Residual candidate |
|---|---|---|---|
| `R1` Candidate-result versus publication | Published cases include result, trace, explain, candidate/publication fields, and published values. | refreshed for local TreeCalc corpus | Pack-grade replay appliance projection remains successor work if required beyond local artifacts. |
| `R2` Reject-is-no-publish | Seven rejected W030 cases with conformance pass and per-case reject artifacts. | refreshed for covered reject causes | Broader typed reject taxonomy remains residual as new OxFml/runtime families are exercised. |
| `R3` Fence-compatible accept/reject split | Compatibility/request fields are preserved through the `OxCalcTree` request/result model. | partial | Explicit incompatible-fence corpus breadth remains a candidate if later promotion needs it. |
| `R4` Pinned-reader stability | Existing Stage 1 authority remains; W030 does not widen it. | not refreshed by W030 | Future TreeCalc pinned-reader corpus bead before Stage 2 concurrency promotion. |
| `R5` Overlay retention and release safety | Runtime-effect overlay artifacts exist; retention/release economics are not newly widened. | partial | Future overlay-retention/release corpus and counters. |
| `R6` Typed reject taxonomy | Rejected W030 cases exercise current local reject/runtime-effect families. | refreshed for covered local families | Provider failure, callable publication, and broader seam rejects remain watch/residual lanes. |
| `R7` Verification-clean without publication | W030 has one verified-clean case. | refreshed for first local equality surface | Broader equality/value semantics remain future widening. |
| `R8` Fallback and overlay re-entry | W029/W030 runtime-effect overlay cases preserve reject/no-publish and overlay diagnostics. | partial | Conservative fallback re-entry economics remains W010 successor residual. |
| `PACK.concurrent.epochs` | No new concurrency evidence in W030; W031 treats concurrency as later. | not refreshed | Block Stage 2 promotion on explicit concurrency/epoch replay. |
| `PACK.dag.dynamic_dependency_bind_semantics` | Dependency graph, invalidation closure, dynamic/runtime-derived artifacts are emitted for covered cases. | refreshed for covered local families | Cross-run/cross-host stronger dependency identity remains later only if needed. |
| `PACK.overlay.fallback_economics` | Overlay artifacts exist, economics counters are not widened. | partial | Needs measurement/counter follow-up before economics-tuned optimization. |

## 5. W010 Measurement And Experiment Inventory

`W010` defines counters and experiments rather than current production instrumentation. W031 inventory keeps that distinction explicit.

| W010 family | Current TreeCalc authority | Refresh state | Residual candidate |
|---|---|---|---|
| `C1` Candidate and publication counters | W030 run summary and case index provide case-level result counts and per-case artifacts. | refreshed as coarse corpus counts | Production counter schema still future. |
| `C2` Pinned-reader and retention counters | No new W030 evidence. | not refreshed | Required before retention/concurrency promotion. |
| `C3` Invalidation and fallback counters | W030 artifacts include invalidation closure and post-edit artifacts for covered cases. | partial | Aggregate counter emission remains future. |
| `C4` Overlay economics counters | Runtime-effect overlay artifacts exist for covered cases. | partial | Overlay lookup/hit/miss/create/evict counters remain future. |
| `C5` Stage 2 promotion counters | Reserved only. | not refreshed | Must stay reserved until Stage 2 planning. |
| `E1`-`E5` experiments | W030 supplies deterministic corpus evidence, not experiment execution. | planning input only | Create successor measurement beads before promotion claims. |

## 6. W012 Reference-Machine And Oracle Inventory

`W012` established the `TraceCalc` reference machine and conformance-oracle doctrine. W030 does not replace that doctrine; it adds a TreeCalc local oracle/baseline lane beneath the `OxCalcTree` contract.

| W012 assumption | Current TreeCalc authority | Refresh state | Residual candidate |
|---|---|---|---|
| Deterministic conformance surface | W030 `conformance_summary.json`, `oracle_baseline.json`, `engine_diff.json`, `explain_index.json`, and per-case artifacts. | refreshed for TreeCalc local corpus | Broader replay-appliance projection remains successor work. |
| Engine outputs compared against oracle | W030 expectation mismatch count is zero across 17 cases. | refreshed for current corpus | Mismatch severity/reduced-witness continuation remains later if new mismatch families appear. |
| Candidate/publication boundary preservation | `OxCalcTree` result contract plus W030 artifacts preserve distinct result states. | refreshed for current corpus | More candidate family breadth remains later seam/corpus work. |
| Trace labels and counters | W030 emits trace and explain sidecars; counter schema remains coarse. | partial | Counter-specific conformance remains successor measurement work. |

## 7. TreeCalc Semantic-Plan Clause Inventory

| Semantic-plan clause | Current authority after W030 | Refresh state | Residual candidate |
|---|---|---|---|
| Structural/formula substrate (`TS-1`) | W025/W030 fixture corpus and `OxCalcTreeDocument` surface. | refreshed for local corpus | Broader node taxonomy remains future. |
| Bind/reference intake (`TS-2`) | W026 first consumed-seam floor and current local reference subset. | partial | Broader caller-context/table/host-sensitive breadth remains watch/residual. |
| Dependency graph/invalidation (`TS-3`) | W027 dependency graph, invalidation closure, and W030 emitted artifacts. | refreshed for covered families | Stronger cross-host dependency identity only if later evidence demands it. |
| Candidate-result intake (`TS-4`) | W028 candidate/reject/publication floor plus W030 baseline. | refreshed for local sequential floor | Broader evaluator-produced family breadth remains later. |
| Runtime-derived effects/overlay (`TS-5`) | W029/W030 runtime-effect and overlay artifacts. | refreshed for exercised families | Overlay economics and retained cleanup remain residual. |
| Corpus/oracle/baseline (`TS-6`-`TS-8`) | W030 checked-in baseline and replay artifact manifest. | refreshed for declared W030 scope | Later live/product breadth belongs to successor worksets. |
| Assurance/pack refresh (`TS-9`) | This map starts W031 inventory. | in progress | `calc-ukb.2` and `calc-ukb.3` should refresh notes and packetize residuals. |

## 8. OxCalcTree Contract Inventory

Current refreshed clauses from the host-facing contract:
1. `OxCalcTreeEnvironment` is a real context object and may affect diagnostics/overlay projection without changing candidate acceptance or publication authority.
2. `OxCalcTreeDocument` carries structural snapshot, formula catalog, and seeded published values.
3. `OxCalcTreeRecalcRequest` carries candidate/publication identity and compatibility/artifact-token basis.
4. `OxCalcTreeRecalcResult` directly exposes result state, dependency graph, invalidation closure, evaluation order, runtime effects, runtime-effect overlays, candidate result, publication bundle, reject detail, published values, node states, and diagnostics.
5. `OxCalcTreeRuntimeFacade` is the current ordinary one-shot execution service.

Residual contract candidates:
1. broader caller-context and address-mode carriage beyond the current subset,
2. table/structured-reference context if later TreeCalc scope admits it,
3. provider-failure and callable-publication if they become coordinator-visible,
4. pinned-reader/session lifecycle packaging beyond one-shot execution,
5. aggregate counter surfaces if W010 measurement moves from planning into runtime evidence.

## 9. W031 Residual Packetization Candidates

The following candidates should be considered by `calc-ukb.3`; they are not new implementation started by this inventory bead.

| Candidate | Why it remains residual | Suggested packet shape |
|---|---|---|
| TreeCalc pinned-reader and retention corpus | W008/W009 pinned-reader and overlay-retention safety were not widened by W030. | Future corpus/evidence bead before concurrency promotion. |
| TreeCalc measurement counter schema | W010 counters remain planning-level; W030 supplies artifacts but not runtime counter emission. | Future measurement/instrumentation bead. |
| Broader typed reject taxonomy | W030 covers current local reject families, not all future OxFml/provider/callable cases. | Watch lane; handoff only on concrete seam mismatch. |
| Replay-appliance projection for TreeCalc baseline | W030 emits local replay artifacts and manifest, not necessarily pack-grade appliance bundles. | Future replay projection bead if pack promotion requires it. |
| Broader caller/table/host context | OxFml notes identify table and caller-context watch lanes beyond current TreeCalc floor. | Future seam-review/handoff candidate only if admitted into TreeCalc scope. |
| Overlay economics/fallback re-entry | Overlay artifacts exist; economic counters and retention policy evidence do not. | Future W010-linked experiment/counter bead. |

## 10. Current W031 Inventory Reading

1. W030 refreshes the assurance authority for the declared local sequential TreeCalc corpus/oracle/baseline floor.
2. W030 does not refresh concurrency, pinned-reader retention, economics-tuned optimization, or broader host/product semantics.
3. The `OxCalcTree` contract is the host-facing authority; local replay artifacts are evidence sidecars, not a second host API.
4. No new OxFml handoff is triggered by this inventory alone.
5. `calc-ukb.2` should update affected replay/pack guardrails only where this map shows a concrete authority movement.
6. `calc-ukb.3` should convert remaining residuals into explicit successor beads or handoff candidates rather than prose ambiguity.
