# W033 Replay Witness Bridge

Status: `calc-uri.13_evidence_packet`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.13`
Created: 2026-05-04

## 1. Purpose

This packet maps current OxCalc TraceCalc/TreeCalc evidence and read-only OxFml replay fixtures into the first W033 witness bridge.

The bridge is a correlation surface. It does not make OxCalc authoritative for OxFml facts. It identifies where OxFml candidate/commit/reject/fence facts align with OxCalc coordinator outcomes and where later handoff/watch or replay widening is still required.

## 2. OxCalc Evidence Runs

| Run | Artifact root | Result |
|---|---|---|
| TraceCalc oracle/conformance run `w033-tracecalc-oracle-self-check-001` | `docs/test-runs/core-engine/tracecalc-reference-machine/w033-tracecalc-oracle-self-check-001/` | 12 scenarios passed; `engine_diff` has 0 mismatches; replay bundle validation is `bundle_valid`. |
| TreeCalc local run `w033-treecalc-witness-bridge-001` | `docs/test-runs/core-engine/treecalc-local/w033-treecalc-witness-bridge-001/` | 17 cases; `published: 9`, `rejected: 7`, `verified_clean: 1`; expectation mismatches 0; conformance pass count 17. |

Commands run:

```powershell
cargo run -p oxcalc-tracecalc-cli -- treecalc w033-treecalc-witness-bridge-001
.\scripts\compare-treecalc-local-run.ps1 -CandidateRunId w033-treecalc-witness-bridge-001 -BaselineRunId post-w031-treecalc-residual-baseline -RepoRoot C:\Work\DnaCalc\OxCalc
```

Observed TreeCalc parity outcome:

`TreeCalc local run parity check passed for 'w033-treecalc-witness-bridge-001' against baseline 'post-w031-treecalc-residual-baseline'.`

## 3. OxFml Read-Only Witness Inputs

| OxFml input | Current state | W033 use |
|---|---|---|
| `../OxFml/crates/oxfml_core/tests/fixtures/fec_commit_replay_cases.json` | contains accepted FEC commit and fence-mismatch reject cases | Source for candidate/commit/reject/fence bridge rows. |
| `../OxFml/crates/oxfml_core/tests/fixtures/witness_distillation/fec_accept_publication_lifecycle.json` | `wit.retained_local`; not GC eligible | Accepted-publication upstream witness input. |
| `../OxFml/crates/oxfml_core/tests/fixtures/witness_distillation/fec_reject_formula_token_lifecycle.json` | `wit.retained_local`; not GC eligible | Fence/token reject upstream witness input. |
| `../OxFml/crates/oxfml_core/tests/fixtures/witness_distillation/retained_witness_set_index.json` | `retained_local_floor`; `fec_commit: 2`; promotion blockers remain | Witness inventory and capability residual input. |
| `../OxFml/crates/oxfml_core/tests/fixtures/prepared_call_replay_cases.json` | contains LET/LAMBDA prepared-call/callable carrier cases | LET/LAMBDA bridge watch input. |
| `../OxFml/crates/oxfml_core/tests/fixtures/higher_order_callable_cases.json` | contains higher-order callable cases | LET/LAMBDA and callable invocation watch input. |

## 4. Minimal Witness Bridge

| Bridge ID | Upstream OxFml fact | OxFml lifecycle | OxCalc TraceCalc evidence | OxCalc TreeCalc evidence | W033 relation | Current status |
|---|---|---|---|---|---|---|
| `W033-WIT-001` | FEC accepted commit `fec_001_accept` with expected accepted decision and published payload | `fec_accept_publication_lifecycle.json`: `wit.retained_local` | `tc_accept_publish_001--witness-seed`: `wit.generated_local`; `tc_accept_publish_001` passed | `tc_local_publish_001` published; conformance state `matches_expected` | Accepted candidate/commit can become OxCalc publication only through coordinator publication. | bridged first slice |
| `W033-WIT-002` | FEC formula-token mismatch reject `fec_002_formula_token_reject` | `fec_reject_formula_token_lifecycle.json`: `wit.retained_local` | `tc_artifact_token_reject_001--witness-seed`: `wit.generated_local`; no publication after artifact-token reject | no direct formula-token case; local reject families provide no-publish support | Stale token/fence mismatch maps to no-publish typed reject. | bridged first slice with TreeCalc gap |
| `W033-WIT-003` | FEC capability-view mismatch reject `fec_003_capability_view_reject` | retained witness index shows FEC commit family retained-local floor | `tc_reject_no_publish_001--witness-seed`: `wit.generated_local`; capability mismatch no-publish | `tc_local_capability_sensitive_reject_001` and post-edit capability overlay cases rejected; conformance state `matches_expected` | Capability/fence mismatch maps to typed no-publish and runtime-effect overlay evidence. | bridged first slice |
| `W033-WIT-004` | OxFml reject/fence facts are imported, not reinterpreted | retained-local floor; promotion blockers remain | `tc_publication_fence_reject_001--witness-seed`; no publication after fence reject | local runtime rejects preserve published state for rejected cases | OxCalc consumes reject/fence meaning and coordinator owns publication/no-publication. | bridged first slice |
| `W033-WIT-005` | Runtime-derived/dynamic dependency facts | OxFml prepared-call and effect fixtures are read-only inputs | `tc_dynamic_dep_switch_001--witness-seed`; `tc_fallback_reentry_001--witness-seed` | `tc_local_dynamic_reject_001`, dependency graphs, invalidation closure artifacts | Dynamic/runtime facts must feed invalidation conservatively. | bridged first slice with upstream fixture gap |
| `W033-WIT-006` | LET/LAMBDA callable carrier facts | prepared-call and higher-order callable fixtures exist; W033 does not own OxFunc kernels | no W033 TraceCalc LET/LAMBDA scenario yet | no TreeCalc LET/LAMBDA runtime fixture yet | Carrier fact visibility is required before OxCalc can claim dependency/replay coverage. | watch/deferred |
| `W033-WIT-007` | OxFml retained witness index promotion blockers | retained-local only; pack-grade blockers listed | TraceCalc witnesses are `wit.generated_local`, `pack_eligible: false` | TreeCalc local run has local conformance, not pack-grade witness retention | Current witness bridge supports first-slice evidence only; no pack-grade promotion. | deferred capability |

## 5. Lifecycle Reading

| Lifecycle state | Source | W033 reading |
|---|---|---|
| `wit.retained_local` | OxFml retained witnesses | Useful upstream evidence input; not OxCalc-owned promotion. |
| `retained_local_floor` | OxFml retained witness index | Broader than seed fixtures but below pack-grade promotion. |
| `wit.generated_local` | W033 TraceCalc run witnesses | Local generated witnesses; not pack-eligible by default. |
| `matches_expected` | TreeCalc local conformance state | Local deterministic conformance to fixture expectations. |
| `bundle_valid` | TraceCalc replay bundle validation | Local replay projection is structurally valid for the run. |

## 6. Bridge Gaps

1. No OxFml fixture is directly replayed inside OxCalc by this packet.
2. LET/LAMBDA carrier facts are only watch/deferred until OxCalc has TraceCalc or TreeCalc carrier scenarios.
3. TreeCalc has no direct formula-token FEC reject fixture; current link is by no-publish/fence behavior class.
4. TraceCalc witnesses are generated-local and not pack-eligible.
5. OxFml retained-local witnesses remain below pack-grade promotion per OxFml retained witness index.

## 7. Downstream Obligations

1. `calc-uri.14` must use this bridge to cap replay and pack claims.
2. `calc-uri.15` must packetize LET/LAMBDA and FEC/F3E watch rows where upstream ambiguity remains.
3. `calc-uri.16` must decide whether direct OxFml fixture replay inside OxCalc becomes a successor bead.

## 8. Status

- execution_state: `replay_witness_bridge_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - no direct OxFml fixture replay is executed inside OxCalc by this packet
  - LET/LAMBDA carrier witnesses remain watch/deferred
  - pack/capability binding consumes this bridge and caps replay/pack claims
  - handoff/watch packet consumes this bridge and keeps upstream-sensitive gaps in watch/deferred lanes
