# W034 Lean Proof-Family Deepening

Status: `calc-e77.4_lean_proof_family`
Workset: `W034`
Parent epic: `calc-e77`
Bead: `calc-e77.4`
Created: 2026-05-05

## 1. Purpose

This packet records the W034 Lean proof-family deepening slice.

The target is to split the post-W033 Lean surface into adjacent checked proof modules for:

1. publication fences and typed no-publish rejection,
2. static, runtime, and dynamic-shape dependency closure,
3. protected overlay retention and release safety,
4. the narrow `LET`/`LAMBDA` callable-carrier fragment,
5. replay-equivalent observable histories,
6. refinement and comparison classification obligations.

This slice does not claim full Lean verification of the core engine, full OxFml formalization, full OxFunc semantic modeling, Stage 2 contention promotion, pack-grade replay, or fully independent evaluator diversity.

## 2. OxFml Formatting Intake

The current OxFml formatting update was reviewed before this packet was finalized.

Current intake:

1. `format_delta` and `display_delta` remain distinct canonical seam categories.
2. Broader display-facing closure remains deferred unless concrete publication or replay evidence exposes a mismatch.
3. OxFml W073 now treats `VerificationConditionalFormattingRule.typed_rule` as the typed-only input contract for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage` option families.
4. The W073 bounded `thresholds` string convention is intentionally ignored for those aggregate and visualization option families.
5. This Lean bead does not construct OxFml conditional-formatting request payloads and does not require an OxCalc code patch.

Decision for this bead: `no_local_formatting_patch_required`.

If later W034 or TreeCalc work constructs those W073 payload families, OxCalc must emit typed metadata and must not rely on bounded threshold strings for those families.

## 3. Lean Artifact Set

New checked artifacts:

| Artifact | Proof surface |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W034PublicationFences.lean` | snapshot, compatibility, and capability-view fence compatibility; reject-is-no-publish; atomic publication envelope |
| `formal/lean/OxCalc/CoreEngine/W034DependenciesOverlays.lean` | dependency closure contains static, runtime, and dynamic-shape update dependencies; no-under-invalidation shape; protected overlay retention and safety |
| `formal/lean/OxCalc/CoreEngine/W034LetLambdaReplay.lean` | full callable surface implies value and callable identity; value-only higher-order row is not full callable conformance; independent writes commute before a check observation |
| `formal/lean/OxCalc/CoreEngine/W034RefinementObligations.lean` | comparison-state classification; declared local gaps are not conformance matches; W034 independent conformance has no unexpected mismatches but does not promote full independent evaluator diversity |

Existing Lean artifacts were rechecked as the base family:

1. `formal/lean/OxCalc/CoreEngine/Stage1State.lean`
2. `formal/lean/OxCalc/CoreEngine/W033FirstSlice.lean`
3. `formal/lean/OxCalc/CoreEngine/W033PostSlice.lean`

## 4. Proof-Obligation Map

| Obligation | Lean evidence | Replay/conformance link | Carry after this bead |
|---|---|---|---|
| `W034-OBL-001` stale-fence and reject depth | `W034PublicationFences.lean` proves mismatched snapshot, compatibility, and capability-view fences cannot satisfy `FenceCompatible`; reject leaves publication and commit history unchanged | `w034-tracecalc-oracle-deepening-001` stale/capability reject scenarios | broader stale-token matrix remains open for later evidence |
| `W034-OBL-002` dynamic dependency negative cases | `W034DependenciesOverlays.lean` includes dynamic-shape update dependencies in closure and states no-under-invalidation shape | TraceCalc dynamic dependency negative scenario and W034 comparison declared-gap rows | runtime dependency update breadth remains a later TLA/pack concern |
| `W034-OBL-003` overlay retention and eviction pressure | `W034DependenciesOverlays.lean` proves protected overlays are retained by protected-only retention and remain safe when the input set is safe | TraceCalc overlay retain/release scenario | interleaving pressure remains `calc-e77.5` |
| `W034-OBL-004` `LET`/`LAMBDA` carrier breadth | `W034LetLambdaReplay.lean` separates value-surface refinement from full callable metadata refinement and proves the W034 value-only TreeCalc row is not a full callable-surface proof | W034 TraceCalc higher-order scenario and W034 TreeCalc value counterpart | broad callable publication policy and general OxFunc semantics remain out of scope |
| `W034-OBL-005` optimized/core-engine conformance | `W034RefinementObligations.lean` encodes match/gap/missing/mismatch classifications and W034 observed summary facts | W034 independent conformance run: 15 rows, 0 missing artifacts, 0 unexpected mismatches, 6 declared local gaps | full independent evaluator diversity remains unpromoted |
| `W034-OBL-007` Lean proof-family depth | all four W034 Lean files check independently alongside the W033 base files | linked to W034 TraceCalc and conformance packets above | broader module hierarchy, imported OxFml formal links, and deeper theorem coverage remain open |
| `W034-OBL-013` formatting/display seam | carried as an explicit non-modeled watch input in this packet | no current W034 formatting payload artifacts | handoff only if later concrete evidence exposes a mismatch |
| `W034-OBL-014` W073 typed conditional-formatting payload | carried as typed-only input-contract guardrail | no current OxCalc request construction path | if exercised later, use `typed_rule` metadata and do not rely on bounded strings |

## 5. Check Commands

Lean checks:

```powershell
lean formal\lean\OxCalc\CoreEngine\Stage1State.lean
lean formal\lean\OxCalc\CoreEngine\W033FirstSlice.lean
lean formal\lean\OxCalc\CoreEngine\W033PostSlice.lean
lean formal\lean\OxCalc\CoreEngine\W034PublicationFences.lean
lean formal\lean\OxCalc\CoreEngine\W034DependenciesOverlays.lean
lean formal\lean\OxCalc\CoreEngine\W034LetLambdaReplay.lean
lean formal\lean\OxCalc\CoreEngine\W034RefinementObligations.lean
```

Result: all checked successfully with exit code `0`.

Repository validation for this bead:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed with CRLF normalization warnings only |

## 6. Semantic-Equivalence Statement

This bead adds checked Lean artifacts and documentation only. It does not change coordinator scheduling, invalidation strategy, publication semantics, reject policy, TraceCalc transition behavior, TreeCalc execution behavior, OxFml fixture content, pack decisions, or formatting/display seam meaning.

Observable runtime behavior is invariant under this bead. The Lean files model and prove bounded invariants over abstract state shapes; they do not introduce a runtime producer or change any execution path.

## 7. Limits

The checked Lean artifacts are theorem-bearing model slices, not a full proof of the production implementation.

Known limits after this bead:

1. no imported Lean linkage to OxFml formal artifacts yet,
2. no full proof of all coordinator transitions,
3. no Stage 2 contention proof or promotion,
4. no pack-grade replay promotion,
5. no proof that declared local gaps are semantically acceptable beyond their explicit classification,
6. no conditional-formatting request construction or display-facing proof lane.

## 8. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for in-scope items? | yes; this packet records artifacts, proof-obligation mapping, OxFml formatting intake, limits, and status |
| 2 | Pack expectations updated for affected packs? | yes; no pack promotion is made, and pack/capability remains mapped to `calc-e77.6` |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; this Lean slice links to the W034 TraceCalc oracle run and W034 independent conformance run rather than emitting new runtime behavior |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 6 records that no runtime policy or strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 formatting is a watch input only for this bead, and no concrete OxFml mismatch appeared |
| 6 | All required tests pass? | yes; see Section 5 |
| 7 | No known semantic gaps remain in declared target? | yes for this Lean proof-family target; broader Lean/TLA/pack/conformance lanes remain mapped to later W034 beads |
| 8 | Completion language audit passed? | yes; this packet does not claim full formalization, full Lean verification, Stage 2 promotion, pack-grade replay, or full formatting/display closure |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth did not change |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; W034 current-state text now records this Lean proof-family slice |
| 11 | execution-state blocker surface updated? | yes; `calc-e77.4` is represented in `.beads/` |

## 9. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-e77.4` asks for checked Lean artifacts and an updated proof-obligation map for fences, dependency closure, overlay safety, LET/LAMBDA carrier visibility, replay history equivalence, and refinement obligations |
| Gate criteria re-read | pass; checked Lean artifacts and this proof-obligation map exist |
| Silent scope reduction check | pass; broader Lean verification, imported OxFml formal linkage, TLA contention modeling, pack-grade replay, and Stage 2 promotion are explicitly carried rather than silently claimed |
| "Looks done but is not" pattern check | pass; these are bounded checked proof slices, not proof of the full production implementation |
| Result | pass for the `calc-e77.4` declared Lean proof-family target |

## 10. Three-Axis Report

- execution_state: `calc-e77.4_lean_proof_family_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-e77.5` TLA model-family and contention precondition slice
  - `calc-e77.6` pack capability and continuous scale gate binding
  - `calc-e77.7` W034 closure audit and successor packetization
  - broader Lean proof depth and imported OxFml formal linkage beyond this checked slice
