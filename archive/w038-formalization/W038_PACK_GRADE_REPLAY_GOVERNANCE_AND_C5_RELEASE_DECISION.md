# W038 Pack-Grade Replay Governance And C5 Release Decision

Status: `calc-zsr.8_pack_c5_release_decision_validated`
Workset: `W038`
Parent epic: `calc-zsr`
Bead: `calc-zsr.8`

## 1. Purpose

This packet records the W038 pack-grade replay governance and C5 release decision.

The target is not to promote `cap.C5.pack_valid`, pack-grade replay, Stage 2 policy, operated services, fully independent evaluator diversity, full Lean/TLA verification, general OxFunc kernels, or release-grade verification. The target is to reassess C5 after the direct W038 evidence tranche and emit a machine-readable no-promotion decision that names the remaining blockers without converting proxy evidence into release claims.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W038_CORE_FORMALIZATION_RELEASE_GRADE_CLOSURE_HARDENING.md` | W038 gate model and `calc-zsr.8` target |
| `docs/spec/core-engine/w038-formalization/W038_RESIDUAL_RELEASE_GRADE_OBLIGATION_LEDGER_AND_OBJECTIVE_MAP.md` | W038 obligations `W038-OBL-019` and `W038-OBL-020` |
| `archive/test-runs-core-engine-w038-w045/tracecalc-authority/w038-tracecalc-authority-discharge-001/` | W038 TraceCalc authority input |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w038-optimized-core-conformance-disposition-001/` | W038 optimized/core conformance input |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w038-proof-model-assumption-discharge-001/` | W038 proof/model and totality-boundary input |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w038-stage2-partition-replay-001/` | W038 bounded Stage 2 replay and no-promotion input |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w038-operated-assurance-alert-quarantine-001/` | W038 operated-assurance and service-disposition input |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w038-diversity-seam-watch-001/` | W038 independent-diversity and OxFml seam-watch input |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | inbound OxFml watch ledger; no direct OxFml edit authority in this repo |

## 3. Artifact Surface

Run id: `w038-pack-c5-release-decision-001`

| Artifact | Result |
|---|---|
| `src/oxcalc-tracecalc/src/pack_capability.rs` | W038 pack/C5 release-decision profile in the pack-capability runner |
| `archive/test-runs-core-engine-w038-w045/pack-capability/w038-pack-c5-release-decision-001/run_summary.json` | 13 satisfied inputs, 25 blockers, 0 missing artifacts, highest honest capability `cap.C4.distill_valid` |
| `archive/test-runs-core-engine-w038-w045/pack-capability/w038-pack-c5-release-decision-001/evidence/evidence_index.json` | evidence index across retained pack inputs, W037 carried direct evidence, and W038 direct packets |
| `archive/test-runs-core-engine-w038-w045/pack-capability/w038-pack-c5-release-decision-001/decision/pack_capability_decision.json` | C5 and Stage 2 remain unpromoted |
| `archive/test-runs-core-engine-w038-w045/pack-capability/w038-pack-c5-release-decision-001/replay-appliance/bundle_manifest.json` | replay-appliance decision bundle manifest |
| `archive/test-runs-core-engine-w038-w045/pack-capability/w038-pack-c5-release-decision-001/replay-appliance/validation/bundle_validation.json` | bundle validation status `bundle_valid` |

## 4. C5 Release Decision

The W038 decision keeps `highest_honest_capability=cap.C4.distill_valid`.

The C5 gate remains blocked because W038 still lacks:

1. operated pack-grade replay governance,
2. production Stage 2 partition-analyzer soundness and pack-grade replay equivalence,
3. an operated continuous-assurance service,
4. an operated cross-engine differential service,
5. an enforcing alert/quarantine dispatcher,
6. a retained history/witness service with lifecycle guarantees,
7. full optimized/core-engine verification,
8. full Lean/TLA verification for the claimed scope,
9. callable metadata projection evidence,
10. fully independent evaluator diversity,
11. broad OxFml display/publication closure,
12. general OxFunc kernel verification in the appropriate owner boundary.

## 5. Promotion Guard

The W038 pack decision forbids these conversions:

| Evidence type | Forbidden promotion |
|---|---|
| TraceCalc authority discharge | release-grade verification by itself |
| optimized/core conformance dispositions | full optimized/core verification |
| proof/model assumption discharge | full Lean/TLA verification |
| bounded Stage 2 replay | production Stage 2 policy or pack-grade replay equivalence |
| local alert/quarantine evidence | operated dispatcher or service |
| file-backed cross-engine evidence | operated differential service |
| direct OxFml and W073 seam-watch rows | broad OxFml closure, general OxFunc kernel, pack-grade replay, or C5 |
| timing/scale evidence | semantic correctness proof |

## 6. OxFml Watch

No OxFml handoff is filed by this bead.

The current OxFml formatting update is carried through the W038 Stage 2 and diversity/seam-watch packets: W073 aggregate/visualization conditional-formatting metadata is `typed_rule`-only for the watched families, while scalar/operator/expression threshold text remains separate. The pack decision consumes that as aligned watch evidence only.

## 7. Semantic-Equivalence Statement

This bead adds a W038 pack-capability profile, emitted decision artifacts, and status/spec text.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, continuous-assurance runner semantics, pack runtime behavior, service behavior, alert dispatch behavior, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The W038 pack decision reads existing evidence and writes a no-promotion decision packet.

## 8. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc pack_capability -- --nocapture` | passed; 6 tests |
| `cargo run -p oxcalc-tracecalc-cli -- pack-capability w038-pack-c5-release-decision-001` | passed; emitted 13 satisfied inputs, 25 blockers, and highest honest capability `cap.C4.distill_valid` |
| JSON parse for `archive/test-runs-core-engine-w038-w045/pack-capability/w038-pack-c5-release-decision-001/**/*.json` | passed; 5 JSON files parsed |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo test -p oxcalc-tracecalc` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-zsr.9` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W038 README/status surfaces, generated pack artifacts, and feature map record the slice |
| 2 | Pack expectations updated for affected packs? | yes; this packet is the W038 pack/C5 release decision and keeps C5 unpromoted |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for decision emission; pack-grade replay governance remains absent and is explicitly blocked |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current OxFml watch rows are aligned and no handoff trigger exists |
| 6 | All required tests pass? | yes; see Section 8 |
| 7 | No known semantic gaps remain in declared scope? | yes for this target after artifact generation; remaining C5 blockers are explicit |
| 8 | Completion language audit passed? | yes; no C5, Stage 2, full verification, operated service, independent evaluator, broad OxFml, general OxFunc, or pack-grade replay promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W038 pack/C5 decision |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-zsr.8` closure and `calc-zsr.9` readiness |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-zsr.8` asks to reassess pack-grade replay and C5 after W038 direct evidence |
| Gate criteria re-read | pass; C5 is not promoted because direct pack-governance, service, conformance, proof/model, Stage 2, and diversity evidence still have exact blockers |
| Silent scope reduction check | pass; remaining blockers are explicit and routed |
| "Looks done but is not" pattern check | pass; bounded replay, local alert/quarantine, watch rows, direct external slices, and file-backed evidence are not represented as C5 |
| Result | pass for the `calc-zsr.8` pack/C5 release-decision target |

## 11. Three-Axis Report

- execution_state: `calc-zsr.8_pack_c5_release_decision_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-zsr.9` W038 closure audit and release-grade verification decision remains open
  - `cap.C5.pack_valid` remains unpromoted
  - pack-grade replay governance remains absent
  - production Stage 2 policy and pack-grade replay equivalence remain unpromoted
  - operated continuous-assurance service, alert dispatcher, cross-engine differential service, and retained history service remain unpromoted
  - full optimized/core-engine verification, full Lean/TLA verification, fully independent evaluator diversity, broad OxFml display/publication closure, callable metadata projection, and general OxFunc kernels remain unpromoted
