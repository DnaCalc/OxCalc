# W033 Pack Capability Binding

Status: `calc-uri.14_evidence_packet`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.14`
Created: 2026-05-04

## 1. Purpose

This packet binds the W033 claim matrix to current proof, model-check, replay, conformance, pack, and capability evidence.

The central rule is conservative: W033 does not promote any pack-grade or capability claim beyond current evidence. Where the current evidence is only local, first-slice, smoke-checked, source-only, or watch/deferred, the row says so and names the next evidence step.

## 2. Current Capability Ceiling

| Evidence lane | Artifact | Claimed capability | Target capability | W033 reading |
|---|---|---|---|---|
| TraceCalc replay adapter | `docs/test-runs/core-engine/tracecalc-reference-machine/w033-tracecalc-oracle-self-check-001/replay-appliance/adapter_capabilities/oxcalc.json` | `cap.C0.ingest_valid`, `cap.C1.replay_valid`, `cap.C2.diff_valid`, `cap.C3.explain_valid`, `cap.C4.distill_valid` | `cap.C5.pack_valid` | Local run snapshot reaches C4 for the covered TraceCalc family; C5 is explicitly not proven. |
| TreeCalc local replay adapter | `docs/test-runs/core-engine/treecalc-local/w033-treecalc-witness-bridge-001/replay-appliance/adapter_capabilities/oxcalc_treecalc.json` | `cap.C0.ingest_valid`, `cap.C1.replay_valid`, `cap.C2.diff_valid`, `cap.C3.explain_valid` | `cap.C4.distill_valid`, `cap.C5.pack_valid` | Local TreeCalc projection reaches C3 for the covered fixture family; C4/C5 remain targets. |
| OxFml retained witnesses | `../OxFml/crates/oxfml_core/tests/fixtures/witness_distillation/retained_witness_set_index.json` | retained local witness floor | pack-grade promotion blocked upstream | Read-only evidence input only; no OxCalc-owned promotion. |
| TLA+ | `formal/tla/CoreEngineStage1.tla` with `formal/tla/CoreEngineStage1.smoke.cfg` | smoke-checked safety model | broader model families | Useful model-check evidence for Stage 1 publication/reject/pin/overlay safety, not pack-grade by itself. |
| Lean | `formal/lean/OxCalc/CoreEngine/W033FirstSlice.lean` | first-slice checked transition lemmas | wider theorem family | Useful proof-backed first slice, not a complete formal proof suite. |

Capability limit statement: W033 may cite local C0-C4 TraceCalc evidence and local C0-C3 TreeCalc evidence for covered behavior. It must not claim `cap.C5.pack_valid`, cross-engine continuous assurance, broad production diversity, Stage 2 concurrency readiness, or LET/LAMBDA carrier coverage.

## 3. Evidence Classes Used

| Class | Evidence | Limit |
|---|---|---|
| `proof_backed_first_slice` | Lean theorems `applyReject_noPublish`, `applyPublish_atomic`, `conservativeAffectedSet_refl`, `emptyOverlays_safe` | covers crisp transition fragments only |
| `model_checked_smoke` | TLC smoke result: 4855 states generated, 908 distinct states, no error | bounded Stage 1 model only |
| `tracecalc_replay_local` | `w033-tracecalc-oracle-self-check-001`, 12 scenarios, 0 mismatches, bundle valid | covered TraceCalc family only |
| `treecalc_replay_local` | `w033-treecalc-witness-bridge-001`, 17 cases, 0 expectation mismatches, bundle valid | local TreeCalc fixture family only |
| `production_conformance_first_slice` | TraceCalc reference/engine diff has 12 rows and 0 mismatches | shared-executor independence caveat applies |
| `oxfml_source_readonly` | OxFml FEC and retained witness fixtures | imported upstream fact surface, not OxCalc-owned promotion |
| `measurement_only` | phase timing, overlay economics, measurement counters | cannot prove semantics without conformance binding |
| `deferred_or_watch` | explicit gap, handoff, or successor lane | no current promotion |

## 4. Pack Binding Matrix

| Pack row | W033 claim rows | Current evidence binding | Current capability reading | Blocked or deferred next evidence |
|---|---|---|---|---|
| `PACK.fec.commit_atomicity` | `W033-CLM-002`, `W033-CLM-004`, `W033-CLM-005` | Lean `applyPublish_atomic`; TLA `NoTornPublication` and `CandidateIsNotPublication`; TraceCalc `tc_accept_publish_001`, `tc_verify_clean_no_publish_001`; TreeCalc `tc_local_publish_001`; OxFml `fec_001_accept` read-only bridge | first-slice proof/model/replay backed; not pack-grade | direct OxFml fixture replay inside OxCalc; independent production engine comparison; broader stale-fence matrix |
| `PACK.fec.reject_detail_replay` | `W033-CLM-003`, `W033-CLM-005`, `W033-CLM-018` | Lean `applyReject_noPublish`; TLA `RejectIsNoPublish`; TraceCalc `tc_reject_no_publish_001`, `tc_publication_fence_reject_001`, `tc_artifact_token_reject_001`; TreeCalc reject cases; OxFml `fec_002_formula_token_reject` and `fec_003_capability_view_reject` read-only bridge | first-slice proof/model/replay backed; TraceCalc C4 local; TreeCalc C3 local | direct TreeCalc formula-token fixture; richer reject calculus and upstream taxonomy drift watch |
| `PACK.fec.overlay_lifecycle` | `W033-CLM-008`, `W033-CLM-009` | TLA `OverlayEvictionSafety` and `PinnedReaderStability`; TraceCalc `tc_overlay_retention_001`, `tc_pinned_view_stability_001`; TreeCalc post-edit overlay cases and retention guardrail | local model/replay backed; no concurrency promotion | Stage 2 contention model; overlay eviction proof beyond empty floor; broader retained overlay witness lifecycle |
| `PACK.concurrent.epochs` | `W033-CLM-004`, `W033-CLM-009`, `W033-CLM-016` | TLA smoke covers pinned-reader/publication compatibility; TraceCalc pinned view first slice | deferred for pack purposes | Stage 2 contention and epoch interleaving models; replay cases with concurrent publication/read pressure |
| `PACK.dag.dynamic_dependency_bind_semantics` | `W033-CLM-006`, `W033-CLM-007`, `W033-CLM-012` | Lean `ConservativeAffectedSet`; TraceCalc `tc_dynamic_dep_switch_001`, `tc_fallback_reentry_001`; TreeCalc dynamic, rebind, remove, move, and invalidation closure artifacts | local replay/model vocabulary backed; no LET/LAMBDA carrier coverage | LET/LAMBDA carrier scenarios; TLA dynamic-dependency model; under-invalidation negative tests |
| `PACK.overlay.fallback_economics` | `W033-CLM-008`, `W033-CLM-014` | TraceCalc `tc_scale_chain_seed_001` and `tc_fallback_reentry_001`; TreeCalc `overlay_economics_summary.json`, `phase_timing_summary.json`, and measurement counters | measurement-only plus local conformance context | semantic binding for any optimization claim; scale thresholds and retained performance baselines |
| `PACK.visibility.policy_equivalence` | `W033-CLM-011`, `W033-CLM-019` | metamorphic/differential family packet declares future families | deferred | semantic-equivalence statements for scheduling/visibility policy changes; differential runner widening |
| `PACK.trace.forensic_plane` | `W033-CLM-006`, `W033-CLM-017`, `W033-CLM-018` | TraceCalc replay bundle, witness lifecycles, explain records; TreeCalc trace/explain artifacts; OxFml retained witness inputs | local forensic plane evidence; C3/C4 local depending lane | direct cross-lane replay identity checks; LET/LAMBDA trace carrier visibility |
| `PACK.replay.appliance` | `W033-CLM-009`, `W033-CLM-010`, `W033-CLM-017` | TraceCalc and TreeCalc replay appliance bundle validations both `bundle_valid` with `missing_paths: []` | TraceCalc C4 local; TreeCalc C3 local; C5 not proven | pack-grade bundle governance; retained witness promotion; direct OxFml fixture replay |
| `PACK.diff.cross_engine.continuous` | `W033-CLM-010`, `W033-CLM-011`, `W033-CLM-019` | production conformance first slice: 12 rows, 0 mismatches | first-slice local comparison only | independent production-vs-oracle diversity; continuous differential suite; TreeCalc-to-TraceCalc comparator |
| `PACK.reject.calculus` | `W033-CLM-003`, `W033-CLM-005`, `W033-CLM-018` | TraceCalc reject set artifacts; TreeCalc `typed_reject_taxonomy.json`; OxFml FEC reject fixtures | local typed-reject evidence | formal reject taxonomy model; upstream handoff only if taxonomy insufficiency is found |
| `PACK.scaling.signature` | `W033-CLM-014` | TraceCalc scale-chain seed; TreeCalc phase timing and measurement summaries | measurement-only | bind large-scale runs to replay/conformance and semantic-equivalence criteria before using as assurance |

## 5. Claim Disposition Summary

| Claim rows | Disposition |
|---|---|
| `W033-CLM-001` | No pack promotion; structural/runtime separation remains source/proof vocabulary plus future spec-maintenance work. |
| `W033-CLM-002` to `W033-CLM-005` | Bound to FEC commit/reject pack rows at first-slice proof/model/replay level; no pack-grade promotion. |
| `W033-CLM-006` to `W033-CLM-007` | Bound to dynamic-dependency pack row with TreeCalc and TraceCalc local evidence; LET/LAMBDA and stronger invalidation proofs remain open. |
| `W033-CLM-008` to `W033-CLM-009` | Bound to overlay/concurrency/replay rows at smoke/local replay level; Stage 2 is deferred. |
| `W033-CLM-010` to `W033-CLM-011` | Bound to TraceCalc oracle and conformance first slice; independence caveat prevents broad production claim. |
| `W033-CLM-012` | Bound as watch/deferred under dynamic dependency and trace rows; no LET/LAMBDA carrier witness yet. |
| `W033-CLM-013` | No pack row; ordinary OxFunc semantics remain out of W033 scope. |
| `W033-CLM-014` | Scaling evidence is measurement-only until tied to semantic replay/conformance. |
| `W033-CLM-015` | Satisfied as a local governance rule by this packet: no capability row is promoted beyond current evidence. |
| `W033-CLM-016` | Deferred; no Stage 2 concurrency promotion. |
| `W033-CLM-017` to `W033-CLM-018` | Bound to replay appliance, trace forensic plane, reject replay, and OxFml read-only bridge rows with capability ceiling. |
| `W033-CLM-019` | Deferred beyond family packet; no continuous differential promotion. |
| `W033-CLM-020` | Historical rows remain governed by the no-loss crosswalk and do not create pack claims by themselves. |

## 6. Non-Promotion Decisions

1. `cap.C5.pack_valid` is not claimed for W033.
2. TreeCalc local replay is not promoted to `cap.C4.distill_valid` by this packet.
3. TraceCalc C4 local evidence is not treated as broad production independence.
4. OxFml retained-local witness state is not treated as OxCalc-owned witness promotion.
5. Stage 2 concurrency remains deferred.
6. `LET`/`LAMBDA` carrier facts remain watch/deferred until exercised carrier witnesses exist.
7. Performance and scaling artifacts remain measurement input unless bound to replay/conformance evidence.

## 7. Downstream Obligations

1. `calc-uri.15` must classify the LET/LAMBDA carrier, OxFml FEC/reject, and upstream trace/runtime-effect watch rows.
2. `calc-uri.16` must carry this capability ceiling into the W033 closure audit.
3. Successor work should decide whether to add direct OxFml fixture replay, TreeCalc-to-TraceCalc differential comparison, and pack-grade replay appliance governance.

## 8. Status

- execution_state: `pack_capability_binding_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - W033 parent epic remains open
  - OxFml handoff/watch packet consumes this binding and does not file a new handoff
  - closure audit has not yet consumed this binding
  - capability rows above current C3/C4 local evidence remain deferred

## 9. Post-W033 Successor Note

The successor packet `W033_PACK_CAPABILITY_POST_W033_DECISION.md` consumes post-W033 direct OxFml fixture projection, LET/LAMBDA carrier witnesses, and independent conformance widening evidence.

The resulting decision remains conservative:

1. target capability: `cap.C5.pack_valid`,
2. decision status: `capability_not_promoted`,
3. highest honest capability: `cap.C4.distill_valid`,
4. missing artifacts: 0,
5. no-promotion reasons: 8 machine-readable blocker ids.

The successor packet therefore preserves this binding document's non-promotion rule rather than overriding it.
