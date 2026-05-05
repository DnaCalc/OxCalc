# W035 Lean Assumption Discharge And Seam Proof Map

Status: `calc-tkq.4_lean_assumption_discharge_validated`
Workset: `W035`
Parent epic: `calc-tkq`
Bead: `calc-tkq.4`

## 1. Purpose

This packet deepens the W034 Lean proof slices by separating:

1. local OxCalc theorems that are represented as checked Lean facts,
2. explicit axiom slots that would have to be visible if introduced,
3. OxFml-owned seam assumptions,
4. OxFunc-opaque callable-kernel facts,
5. later formal lanes such as non-routine scheduler/interleaving exploration.

The goal is proof hygiene, not broad promotion. W035 keeps the formalization path evolvable: current implementation behavior, current specs, TraceCalc evidence, and seam documents remain evidence surfaces that can change as understanding improves.

## 2. Authority Inputs Reviewed

| Input | Role in this bead |
|---|---|
| `docs/worksets/W035_CORE_FORMALIZATION_PROOF_AND_ASSURANCE_HARDENING.md` | W035 scope and `calc-tkq.4` gate |
| `docs/spec/core-engine/w035-formalization/W035_RESIDUAL_PROOF_OBLIGATION_AND_SPEC_EVOLUTION_LEDGER.md` | W035 obligations and W073 watch row |
| `docs/spec/core-engine/w035-formalization/W035_TRACECALC_ORACLE_MATRIX_EXPANSION.md` | callable/full-OxFunc and overlay/TLA classifications |
| `docs/spec/core-engine/w035-formalization/W035_IMPLEMENTATION_CONFORMANCE_HARDENING.md` | W034 gap dispositions carried into proof mapping |
| `docs/spec/core-engine/w034-formalization/W034_LEAN_PROOF_FAMILY_DEEPENING.md` | predecessor Lean proof family |
| `formal/lean/OxCalc/CoreEngine/W034*.lean` | current checked W034 proof slices |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | inbound OxFml seam observation ledger |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current typed-only conditional-formatting input contract |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | downstream-facing W073 typed-rule uptake note |

## 3. Lean Artifact Surface

| Artifact | Role |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W035AssumptionDischarge.lean` | checked inventory of local proof rows, explicit-axiom count, external seam assumptions, opaque kernel boundaries, and later-lane deferrals |
| `formal/lean/OxCalc/CoreEngine/W035SeamProofMap.lean` | checked seam map that prevents external OxFml/OxFunc facts from being promoted as OxCalc-local Lean proofs |

The W035 files intentionally introduce no Lean `axiom` declarations. They include an explicit-axiom classification bucket and prove that the current W035 Lean assumption summary has zero explicit axiom rows.

## 4. Local Discharge Rows

| Row | Lean fact | Disposition |
|---|---|---|
| snapshot-fence reject/no-publish | `snapshotFenceObligation_isDischargedLocal` and `snapshotFenceRejectNoPublish_promotableAsLocalProof` | local checked proof row |
| dependency no-under-invalidation | `dependencyClosureObligation_isDischargedLocal` and `DependencyNoUnderInvalidationFact` | local checked proof row |
| protected overlay retention | `overlayRetentionObligation_isDischargedLocal` and `OverlayProtectedRetentionFact` | local checked proof row |
| callable carrier identity | `callableCarrierObligation_isDischargedLocal` and `callableCarrierIdentity_promotableAsLocalCarrierProof` | local checked carrier proof row, not full OxFunc semantics |

These rows deepen W034 by making the local proof classifications machine-checked. They do not replace the broader W034 theorem files and do not claim full proof coverage of the production Rust engine.

## 5. External And Opaque Rows

| Row | Owner | Lean classification | Consequence |
|---|---|---|---|
| OxFml fence artifact meaning | OxFml | `externalSeamAssumption` | OxCalc can reference the seam fact but cannot promote it as a local proof |
| OxFml W073 typed conditional-formatting metadata | OxFml | `externalSeamAssumption` | `typed_rule` is required for W073 aggregate/visualization families if OxCalc later constructs those payloads |
| full callable semantic kernel | OxFunc | `opaqueKernelBoundary` | W035 covers the carrier fragment only; general callable evaluation remains outside OxCalc |
| multi-reader overlay release ordering | W035 TLA lane | `deferredToLaterLane` | owned by `calc-tkq.5` non-routine exploration |

The seam-map theorems prove that the W073 row and full OxFunc callable-kernel row are not promotable as OxCalc-local proofs.

## 6. W073 Formatting Intake

The current OxFml formatting update is incorporated as an external seam/input-contract assumption:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage` options.
2. W072 bounded `thresholds` strings are intentionally ignored for those families.
3. `thresholds` remains available only for scalar/operator/expression rule families where threshold text is the real rule input.

OxCalc still has no current W035 runtime path constructing those conditional-formatting payloads. This bead therefore records the W073 fact in the formal seam map and does not patch OxCalc runtime code or file an OxFml handoff.

## 7. Summary Counts

| Surface | Local proof rows | Explicit axiom rows | External seam rows | Opaque kernel rows | Deferred rows | Promotion result |
|---|---:|---:|---:|---:|---:|---|
| assumption discharge | 4 | 0 | 1 | 1 | 1 | full Lean verification not promoted |
| seam proof map | 4 | 0 | 2 | 1 | 1 | external/opaque rows not promoted as OxCalc proofs |

The seam proof map has two external rows because it tracks both the general OxFml fence-artifact meaning and the W073 typed conditional-formatting input contract.

## 8. Obligation Disposition

| Obligation | `calc-tkq.4` disposition |
|---|---|
| `W035-OBL-004` | callable carrier identity is locally classified; full OxFunc callable semantics remain an opaque kernel boundary |
| `W035-OBL-007` | local proof rows for fences, dependency closure, overlay retention, and callable carrier identity are represented in checked Lean artifacts |
| `W035-OBL-008` | imported OxFml/OxFunc facts are split into external seam assumptions and opaque kernel boundaries |
| `W035-OBL-013` | W073 typed-only conditional-formatting metadata is carried as an OxFml-owned external seam assumption |

## 9. Semantic-Equivalence Statement

This bead adds Lean classification artifacts and spec text. It does not change coordinator scheduling, dependency graph construction, soft-reference resolution, recalc semantics, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc semantics, TLA actions, pack-decision logic, or OxFml evaluator behavior.

Observable runtime behavior is invariant under this bead. The checked Lean files classify proof ownership and assumption boundaries; they do not alter executable calculator behavior.

## 10. Verification

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
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed; `count=0` |
| `git diff --check` | passed; CRLF normalization warnings only |

Cargo tests were not run for this bead because it introduces no Rust behavior or Rust source changes.

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W035 workset status, W035 ledger, spec index, and feature-map surfaces record the Lean proof-map evidence |
| 2 | Pack expectations updated for affected packs? | yes; no pack promotion is claimed and `calc-tkq.7` still owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for carried W035 evidence; this bead adds checked Lean classification artifacts rather than new runtime behavior |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 9 states no runtime policy or strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 is recorded as an OxFml-owned external seam assumption and no concrete OxCalc handoff trigger exists |
| 6 | All required tests pass? | yes; Lean and hygiene validation commands passed |
| 7 | No known semantic gaps remain in declared scope? | yes for this target; external and opaque facts are explicitly classified rather than hidden |
| 8 | Completion language audit passed? | yes; full Lean verification and full OxFunc callable semantics remain partial |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 points at this Lean proof-map evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-tkq.4` execution state and later closure evidence |

## 12. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-tkq.4` asks for assumption inventory, local discharge where practical, and imported seam proof mapping |
| Gate criteria re-read | pass; checked Lean artifacts distinguish local proofs, explicit-axiom slots, external seam assumptions, opaque kernel facts, and later-lane deferrals |
| Silent scope reduction check | pass; full OxFunc callable semantics and multi-reader overlay interleavings are explicitly classified rather than omitted |
| "Looks done but is not" pattern check | pass; no full Lean verification, full engine proof, Stage 2 promotion, or OxFml formatting implementation claim is made |
| Result | pass for the `calc-tkq.4` Lean proof-map target |

## 13. Three-Axis Report

- execution_state: `calc-tkq.4_lean_assumption_discharge_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-tkq.5` through `calc-tkq.8` remain open
  - full Lean/TLA verification remains open
  - full TraceCalc oracle coverage remains open
  - full optimized/core-engine verification and fully independent evaluator diversity remain open beyond W035 conformance-hardening dispositions
  - pack-grade replay, continuous scale assurance, and Stage 2 policy remain unpromoted
