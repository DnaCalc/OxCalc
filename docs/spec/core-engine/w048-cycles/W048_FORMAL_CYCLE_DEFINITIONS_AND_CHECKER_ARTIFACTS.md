# W048 Formal Cycle Definitions And Checker Artifacts

Status: `checked_model_projection`

## 1. Scope

This packet introduces the W048-owned formal/checker surface for cycle behavior. It is intentionally small and executable: the definitions are projected over checked W048 TreeCalc artifacts rather than promoted as a full proof of all future cycle profiles.

## 2. Definitions

| Name | Definition |
| --- | --- |
| `GraphLayer` | One materialized graph basis: structural, published-effective, or candidate-effective. |
| `ForwardEdge(owner, target)` | Dependency edge from formula owner to precedent target. |
| `ReverseEdge(target, owner)` | Converse of a forward edge in the same graph layer. |
| `CycleRegion` | A non-trivial SCC or a self-loop SCC in a graph layer. |
| `CycleReject` | Candidate state where a cycle violates the active profile and no publication bundle is emitted. |
| `NoOverlayCommit` | Rejected candidate-effective overlay facts are not admitted into the published effective graph basis. |
| `ReleaseReentry` | Later graph transition where a prior cyclic basis is replaced by an acyclic basis and affected members/dependents recompute. |

## 3. Executable Obligations

The W048 checker verifies the following against `w048-treecalc-cycles-001`:

1. every forward edge has a reverse-edge converse;
2. every reverse edge has a forward-edge converse;
3. every recorded cycle region has at least one member and exists only in a layer with cycle evidence;
4. W048 structural and CTRO cycle fixtures reject with `SyntheticCycleReject`;
5. rejected W048 cycle fixtures emit no publication bundle;
6. rejected W048 cycle fixtures retain prior published values;
7. the release/re-entry fixture initially rejects and then publishes owner plus downstream dependent after the post-edit acyclic transition;
8. graph checker summary remains 33 cases / 99 layers / 12 cycle-region records / 0 errors.

## 4. Artifacts

| Artifact | Path |
| --- | --- |
| executable checker | `scripts/check-w048-formal-cycle-artifacts.ps1` |
| checker summary | `docs/test-runs/core-engine/formal/w048-cycle-artifacts-001/w048_formal_cycle_checker_summary.json` |
| TLA model sketch | `formal/tla/CoreEngineW048CycleRegions.tla` |
| TLA smoke config | `formal/tla/CoreEngineW048CycleRegions.smoke.cfg` |

## 5. Limits And Blockers

This packet is a checked model projection, not a full mechanized proof of the future iterative profile. The following remain open until successor beads or worksets add deeper formalization:

1. full proof that dynamic graph maintenance is equivalent to from-scratch SCC classification under all CTRO updates;
2. mechanized proof of `cycle.iterative_deterministic_v0` convergence/terminal determinism;
3. Excel-match iterative model, blocked on root/order/update observations;
4. proof-carrying trace integration for all cycle cases.

## 6. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - full iterative-profile proof obligations route to successor formalization work.
  - Excel-match profile remains blocked by observation gaps from `W048_ITERATIVE_PROFILE_DECISION_AND_EXCEL_DISPOSITION.md`.
  - Innovation profile formal obligations route to `calc-zci1.8` and successor worksets.
