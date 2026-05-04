# W033 TLA Bridge First Slice

Status: `calc-uri.12_evidence_packet`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.12`
Created: 2026-05-04

## 1. Purpose

This packet records the first W033 TLA bridge slice.

The first slice uses the existing `CoreEngineStage1` TLA model as the runnable smoke model for coordinator, publication, reject, pinned reader, overlay, and recalc-state safety. It also declares the W033 model-family rollout needed for later FEC bridge, dynamic dependency, and LET/LAMBDA carrier interleavings.

## 2. Runnable TLA Slice

| Artifact | Role |
|---|---|
| `formal/tla/CoreEngineStage1.tla` | Existing Stage 1 model for coordinator, publication, reject, pin, overlay, recalc actions, and first safety invariants. |
| `formal/tla/CoreEngineStage1.smoke.cfg` | Bounded smoke config used for W033 first-slice validation. |
| `formal/tla/CoreEngineStage1.cfg` | Deeper exploration config declared but not used as routine W033 closure evidence. |

Smoke command:

```powershell
$env:TLA2TOOLS_JAR = (Resolve-Path ..\OxFml\formal\tools\tla2tools.jar).Path
.\scripts\run-tlc.ps1 formal\tla\CoreEngineStage1.tla formal\tla\CoreEngineStage1.smoke.cfg
```

Observed result:

| Metric | Value |
|---|---:|
| TLC version | `2.19 of 08 August 2024` |
| Distinct initial states | 1 |
| States generated | 4855 |
| Distinct states found | 908 |
| States left on queue | 0 |
| Complete state graph depth | 5 |
| Result | No error found |

## 3. Checked Smoke Invariants

| Invariant | W033 claim linkage |
|---|---|
| `TypeInvariant` | State variables stay inside declared domains. |
| `NoTornPublication` | Published view is either null or a declared publication id. |
| `RejectIsNoPublish` | Reject decisions carry null publication id. |
| `CandidateIsNotPublication` | Accepted candidates are not publication records. |
| `PinnedReaderStability` | Active pinned readers keep compatible snapshot/publication ids. |
| `OverlayEvictionSafety` | Protected overlays are not eviction-eligible. |

## 4. W033 TLA Model-Family Rollout

| Planned model family | First purpose | Status |
|---|---|---|
| `CoreEngineStage1` | Existing runnable Stage 1 coordinator/publication/reject/pin/overlay floor. | smoke checked |
| `CoreOxfmlFecBridge` | Model imported OxFml session, candidate, commit, reject, fence, trace, and runtime-effect facts entering OxCalc coordinator state. | planned |
| `CorePublicationFence` | Focus stale/incompatible fence cannot publish and accept/reject publication transitions. | planned |
| `CorePinnedReaderOverlay` | Focus pinned-reader compatibility and protected overlay retention/eviction interleavings. | planned |
| `CoreDynamicDependency` | Focus dependency update and recalc invalidation interleavings with conservative over-invalidation. | planned |
| `CoreLetLambdaCarrier` | Focus LET/LAMBDA carrier fact visibility without importing OxFunc kernels. | planned/deferred until replay bridge selects carrier cases |
| `CoreStage2Contention` | Future concurrency/contention pressure model. | deferred; Stage 2 not promoted by W033 first slice |

## 5. Declared Config Policy

1. Smoke configs must be small enough to run during routine W033 validation.
2. Non-smoke configs may be declared as deeper exploration when the state space is larger or termination is not routine.
3. Every model-check evidence claim must record the spec file, config file, TLC version, command, state count, and result.
4. Stage 2 concurrency configs remain deferred until Stage 1 bridge obligations are stable.

## 6. Current Gaps

This packet does not yet model:

1. imported OxFml/FEC/F3E candidate, commit, reject, fence, and trace ADTs,
2. runtime-derived dependency facts from OxFml,
3. LET/LAMBDA callable carrier visibility,
4. dynamic dependency interleavings beyond the existing overlay/invalidation skeleton,
5. broad concurrency/contention behavior,
6. production scheduling-policy equivalence.

## 6.1 Post-W033 Successor Slice

The successor packet `W033_FORMAL_MODEL_FAMILY_WIDENING.md` adds:

1. `formal/tla/CoreEnginePostW033.tla`,
2. `formal/tla/CoreEnginePostW033.smoke.cfg`.

The new smoke model covers imported candidate facts, static and runtime dependency closure, callable carrier visibility, protected overlay safety, compatible-fence publication, reject/no-publish decisions, candidate-not-publication, and explicit no-promotion of Stage 2 contention.

It remains a bounded successor slice. It does not replace the planned deeper model families or promote Stage 2 concurrency policy.

## 7. Status

- execution_state: `tla_bridge_first_slice_smoke_checked`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - full FEC bridge model family remains planned
  - the post-W033 successor slice covers first smoke surfaces for FEC bridge, dynamic dependency, and LET/LAMBDA carrier facts, but deeper model families remain open
  - Stage 2 contention remains deferred
  - pack/capability binding has not yet consumed this TLA evidence
