# W035 Residual Proof Obligation And Spec Evolution Ledger

Status: `calc-tkq.1_residual_ledger_validated`
Workset: `W035`
Parent epic: `calc-tkq`
Bead: `calc-tkq.1`

## 1. Purpose

This ledger converts W034 closure residuals into W035 proof obligations, implementation hardening targets, OxFml watch/handoff rows, declared evidence roots, and promotion limits.

W035 begins from a strong but bounded W034 floor. The W034 tranche is closed for its declared target, but the broader formalization objective remains partial. This ledger prevents that residual work from living only in prose.

## 2. Authority Inputs Reviewed

| Input | Role in W035 |
|---|---|
| `docs/spec/core-engine/w034-formalization/W034_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md` | canonical W034 closure and successor packet |
| `docs/worksets/W035_CORE_FORMALIZATION_PROOF_AND_ASSURANCE_HARDENING.md` | W035 workset scope and gate |
| `docs/spec/core-engine/w034-formalization/W034_RESIDUAL_OBLIGATION_AND_AUTHORITY_LEDGER.md` | predecessor residual map through W034 |
| `docs/spec/core-engine/w034-formalization/W034_TRACECALC_ORACLE_DEEPENING.md` | W034 oracle evidence and limits |
| `docs/spec/core-engine/w034-formalization/W034_INDEPENDENT_CONFORMANCE_WIDENING.md` | W034 conformance rows and declared gaps |
| `docs/spec/core-engine/w034-formalization/W034_LEAN_PROOF_FAMILY_DEEPENING.md` | W034 checked Lean proof slices and assumption limits |
| `docs/spec/core-engine/w034-formalization/W034_TLA_MODEL_FAMILY_AND_CONTENTION_PRECONDITIONS.md` | W034 checked TLA smoke models and contention preconditions |
| `docs/spec/core-engine/w034-formalization/W034_PACK_CAPABILITY_AND_CONTINUOUS_SCALE_GATE_BINDING.md` | W034 pack/scale gate decisions and no-promotion reasons |
| `formal/lean/OxCalc/CoreEngine/*.lean` | current Lean proof surface |
| `formal/tla/CoreEngine*.tla` and `formal/tla/*.cfg` | current TLA+ model surface |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current inbound OxFml observation ledger |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 typed conditional-formatting payload contract |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | downstream-facing W073 typed-rule uptake note |
| `../OxFml/docs/spec/` | read-only OxFml canonical spec inputs |

## 3. Evidence Roots Declared

W035 may emit artifacts under these roots:

1. `docs/spec/core-engine/w035-formalization/`
2. `docs/test-runs/core-engine/tracecalc-reference-machine/w035-*`
3. `docs/test-runs/core-engine/treecalc-local/w035-*`
4. `docs/test-runs/core-engine/independent-conformance/w035-*`
5. `docs/test-runs/core-engine/pack-capability/w035-*`
6. `docs/test-runs/core-engine/continuous-assurance/w035-*`
7. `formal/lean/OxCalc/CoreEngine/W035*.lean`
8. `formal/tla/CoreEngineW035*.tla`
9. `formal/tla/CoreEngineW035*.cfg`

Checked-in evidence must use repo-relative paths. Validation runs must not mutate prior W034 baselines unless a later bead explicitly regenerates and supersedes them.

## 4. Promotion Limits

W035 starts with these limits:

1. `cap.C5.pack_valid` is not promoted.
2. continuous scale assurance is not promoted.
3. full Lean verification is not claimed.
4. full TLA+ verification is not claimed.
5. fully independent evaluator implementation diversity is not claimed.
6. Stage 2 contention policy is not promoted.
7. W073 conditional-formatting typed metadata is watch/input-contract evidence until a W035 artifact constructs that payload family; if exercised, `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage` options.

Any later W035 promotion candidate must include machine-readable evidence, a semantic-equivalence statement, and an updated pack/capability decision.

## 5. Residual Obligation Matrix

| Obligation id | Area | W034 floor | W035 owner | Required W035 disposition |
|---|---|---|---|---|
| `W035-OBL-001` | TraceCalc stale-fence matrix | W034 has focused snapshot, compatibility, and capability-view reject scenarios | `calc-tkq.2` | generate a broader stale-fence matrix or classify remaining matrix rows |
| `W035-OBL-002` | TraceCalc dependency-update matrix | W034 has dynamic dependency negative and shape-update cases | `calc-tkq.2` | expand runtime/static/dynamic dependency update rows and no-under-invalidation assertions |
| `W035-OBL-003` | Overlay retention/eviction pressure | W034 has TraceCalc/TLA protected-overlay evidence | `calc-tkq.2`, `calc-tkq.5` | widen retention/release/eviction interleavings beyond routine smoke |
| `W035-OBL-004` | `LET`/`LAMBDA` callable surface | W034 separates value conformance from callable metadata gaps | `calc-tkq.2`, `calc-tkq.3`, `calc-tkq.4` | expand callable matrix and map OxFml/OxFunc-opaque assumptions |
| `W035-OBL-005` | implementation conformance gaps | W034 has 6 declared local gaps, 0 unexpected mismatches | `calc-tkq.3` | turn each gap into match, implementation bead, handoff/watch row, or deferral |
| `W035-OBL-006` | fully independent evaluator diversity | W034 explicitly keeps this unpromoted | `calc-tkq.3`, `calc-tkq.6`, `calc-tkq.7` | define stronger differential criteria or retain blocker |
| `W035-OBL-007` | Lean assumption discharge | W034 checks adjacent proof slices | `calc-tkq.4` | inventory assumptions and discharge local obligations where practical |
| `W035-OBL-008` | imported seam proof map | W034 carries OxFml/FEC/F3E facts as abstract inputs | `calc-tkq.4` | distinguish proved OxCalc facts, OxFml-owned facts, and OxFunc-opaque facts |
| `W035-OBL-009` | TLA non-routine exploration | W034 checks bounded smoke configs | `calc-tkq.5` | run deeper or broader configs where practical and record state-space limits |
| `W035-OBL-010` | scheduler/Stage 2 equivalence | W034 states Stage 2 preconditions missing | `calc-tkq.5`, `calc-tkq.7` | define promotion preconditions and no-promotion blockers |
| `W035-OBL-011` | continuous assurance | W034 binds seven scale runs semantically but lacks recurring evidence | `calc-tkq.6` | define scheduled/continuous assurance and cross-engine diff criteria |
| `W035-OBL-012` | pack capability | W034 pack decision has 12 blockers | `calc-tkq.7` | reassess blockers after W035 evidence and emit no-promotion or promotion decision |
| `W035-OBL-013` | OxFml W073 formatting input contract | W034 records typed-only aggregate/visualization CF metadata | all W035 beads where exercised | use `typed_rule` for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`; do not rely on W072 bounded `thresholds` strings for those families; file handoff only on concrete mismatch |
| `W035-OBL-014` | spec evolution discipline | W034 treats specs as evolvable evidence surfaces | every W035 bead | patch specs, create implementation beads, file handoffs, or record deferrals when evidence changes understanding |
| `W035-OBL-015` | evidence non-mutation | W034 checked-in baselines exist | every W035 bead | declare new run ids and avoid accidental mutation of W034 baselines |

## 6. Bead Mapping

| Bead | Primary obligations |
|---|---|
| `calc-tkq.2` | `W035-OBL-001`, `W035-OBL-002`, `W035-OBL-003`, `W035-OBL-004` |
| `calc-tkq.3` | `W035-OBL-004`, `W035-OBL-005`, `W035-OBL-006` |
| `calc-tkq.4` | `W035-OBL-004`, `W035-OBL-007`, `W035-OBL-008`, `W035-OBL-013` |
| `calc-tkq.5` | `W035-OBL-003`, `W035-OBL-009`, `W035-OBL-010` |
| `calc-tkq.6` | `W035-OBL-006`, `W035-OBL-011` |
| `calc-tkq.7` | `W035-OBL-006`, `W035-OBL-010`, `W035-OBL-012` |
| `calc-tkq.8` | `W035-OBL-014`, `W035-OBL-015`, all open-lane audit rows |

`calc-tkq.2` evidence is now recorded in `docs/spec/core-engine/w035-formalization/W035_TRACECALC_ORACLE_MATRIX_EXPANSION.md` and `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/`.

Current `calc-tkq.2` obligation disposition:

| Obligation id | `calc-tkq.2` disposition |
|---|---|
| `W035-OBL-001` | stale-fence matrix has 4 covered TraceCalc rows and 0 failed/missing rows |
| `W035-OBL-002` | dependency-update matrix has 5 covered TraceCalc rows and 0 failed/missing rows |
| `W035-OBL-003` | overlay-retention matrix has 2 covered TraceCalc rows; multi-reader release ordering is classified to `calc-tkq.5` |
| `W035-OBL-004` | callable-carrier matrix has 4 covered TraceCalc rows; full OxFunc kernel semantics are classified to `calc-tkq.4` |

## 7. OxFml Watch And Handoff Rules

Current watch rows:

1. W073 typed conditional-formatting metadata remains `typed_rule`-only for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those W073 families.
3. `thresholds` remains an OxFml input only for scalar/operator/expression rule families where threshold text is the rule input.
4. `format_delta` and `display_delta` remain distinct canonical categories.
5. Runtime facade and replay comparison-view updates remain OxFml-owned inputs unless W035 exposes a concrete mismatch.

No OxFml handoff is filed by this bead. A W035 handoff is required only if evidence shows an OxFml-owned evaluator, FEC/F3E, runtime facade, or formatting clause is insufficient for an exercised OxCalc artifact.

## 8. Semantic-Equivalence Statement

This bead adds a W035 residual ledger and updates planning/status surfaces only. It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, TraceCalc semantics, TreeCalc semantics, Lean/TLA model semantics, pack-decision logic, or OxFml evaluator behavior.

Observable runtime behavior is invariant under this bead because it introduces no runtime producer, evaluator, coordinator transition, formal theorem, TLA action, or fixture expectation change.

## 9. Verification

| Command | Result |
|---|---|
| `scripts/check-worksets.ps1` | passed; `worksets=13`, `beads total=79`, `open=8`, `in_progress=0`, `ready=1`, `blocked=6`, `closed=71` |
| `br dep cycles --json` | passed; `count=0` |
| `git diff --check` | passed; CRLF normalization warnings only |
| `rg -n "VerificationConditionalFormattingRule\|typed_rule\|conditional_formatting" src docs/spec/core-engine/w035-formalization docs/worksets/W035_CORE_FORMALIZATION_PROOF_AND_ASSURANCE_HARDENING.md docs/IN_PROGRESS_FEATURE_WORKLIST.md` | passed; matches are watch/planning rows plus the existing pack-capability watch input string, with no OxCalc request-construction path for W073 payloads |

Cargo, Lean, and TLC validation were not run for this bead because it changes planning/spec/status surfaces only and introduces no Rust, Lean, TLA+, fixture, or replay semantics.

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this ledger declares W035 obligations, evidence roots, and promotion limits |
| 2 | Pack expectations updated for affected packs? | yes; pack remains unpromoted and `calc-tkq.7` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for carried W034 inputs; this bead emits no new behavior |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 typed-only conditional-formatting input was assessed, watch rows are carried, and no concrete OxCalc handoff trigger exists |
| 6 | All required tests pass? | yes; planning/spec validation commands passed, and no runtime/formal test lane is in scope for this bead |
| 7 | No known semantic gaps remain in declared scope? | yes for this ledger target; all known residuals are mapped |
| 8 | Completion language audit passed? | yes; broader formalization objectives remain partial |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | already updated by W034 closure packet |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 carries the W035 residual ledger and W073 typed-only input-contract watch row |
| 11 | execution-state blocker surface updated? | yes; W035 beads exist in `.beads/` and `calc-tkq.1` is recorded with closure evidence |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-tkq.1` asks for W035 residual proof obligations and spec-evolution ledger |
| Gate criteria re-read | pass; W034 residuals are mapped and W035 roots/limits are declared |
| Silent scope reduction check | pass; no broader formalization objective is claimed as satisfied |
| "Looks done but is not" pattern check | pass; this is a ledger/planning bead, not implementation or proof promotion |
| Result | pass for the `calc-tkq.1` ledger target |

## 12. Three-Axis Report

- execution_state: `calc-tkq.1_residual_ledger_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-tkq.2` through `calc-tkq.8` remain open
  - full formalization, full Lean/TLA verification, full TraceCalc oracle coverage, optimized/core-engine verification, pack-grade replay, continuous scale assurance, and Stage 2 policy remain partial
