# W034 Formalization Deepening Plan

Status: `w034_entry_plan`
Workset: `W034`
Parent epic: `calc-e77`

## 1. Intent

W034 turns the W033 first-pass formalization and post-W033 successor slices into the next execution tranche for core-engine tempering.

The intent is to support future implementation improvement by tightening the relationship between:

1. OxCalc-owned specs,
2. TraceCalc oracle behavior,
3. optimized/core-engine behavior,
4. Lean proof obligations,
5. TLA+ state/interleaving models,
6. replay and witness evidence,
7. pack/capability decisions,
8. OxFml seam facts consumed by OxCalc.

W034 keeps the W033 rule that formalization is a discovery and evolution process. If evidence shows the current model is weak or wrong, the expected act is to update the relevant OxCalc-owned spec, open an implementation bead, record a deferred lane, or file an OxFml handoff where the boundary requires it.

## 2. Seed Residuals From W033

| Residual | Source | W034 owner |
|---|---|---|
| Broader Lean module-family split and proof obligations | `W033_FORMAL_MODEL_FAMILY_WIDENING.md` | `calc-e77.4` |
| Deeper TLA model families and contention preconditions | `W033_FORMAL_MODEL_FAMILY_WIDENING.md`, `W033_TLA_BRIDGE_FIRST_SLICE.md` | `calc-e77.5` |
| Stage 2 contention remains unpromoted | `W033_FORMAL_MODEL_FAMILY_WIDENING.md`, `W033_METAMORPHIC_SCALE_SEMANTIC_BINDING.md` | `calc-e77.5`, `calc-e77.6` |
| Pack-grade replay remains unpromoted | `W033_PACK_CAPABILITY_POST_W033_DECISION.md` | `calc-e77.6` |
| Continuous scale assurance remains unpromoted | `W033_METAMORPHIC_SCALE_SEMANTIC_BINDING.md` | `calc-e77.6` |
| TraceCalc oracle coverage needs broader stale-fence, dependency, overlay, and callable cases | W033 closure and successor packets | `calc-e77.2` |
| Optimized/core-engine conformance needs broader comparison evidence | `W033_INDEPENDENT_CONFORMANCE_WIDENING.md` | `calc-e77.3` |
| OxFml seam-watch rows remain active inputs | `W033_OXFML_HANDOFF_WATCH_PACKET.md` | `calc-e77.1`, later execution beads as needed |

## 3. Current OxFml Intake

The current inbound OxFml ledger is `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`.

W034 starts with these relevant readings:

1. `format_delta` and `display_delta` remain distinct seam categories.
2. Broader display-facing closure is not assumed.
3. Current formatting-sensitive evaluator facts can be semantic and replay-significant where OxFml surfaces them.
4. The W073 conditional-formatting typed payload direction changes OxFml input metadata expectations for aggregate and visualization CF families; OxCalc has no current local request-construction path for those payloads.
5. The formatting update is therefore a W034 seam-watch input unless a later W034 artifact exercises that path and finds concrete mismatch.

Current W034 intake after `calc-e77.2`:

1. OxFml now treats `VerificationConditionalFormattingRule.typed_rule` as the active input contract for the W073 aggregate and visualization families.
2. The W034 TraceCalc oracle-deepening slice does not construct those conditional-formatting packets.
3. No OxCalc code-path patch or OxFml handoff is required by this TraceCalc slice.

## 4. Bead-To-Artifact Plan

| Bead | Primary artifact outcome |
|---|---|
| `calc-e77.1` | W034 residual obligation and authority ledger under this directory: `W034_RESIDUAL_OBLIGATION_AND_AUTHORITY_LEDGER.md`. |
| `calc-e77.2` | Widened TraceCalc scenarios, deterministic run root, and oracle self-check packet. |
| `calc-e77.3` | Optimized/core-engine conformance comparison runner or widened runner output, with mismatch classification. |
| `calc-e77.4` | Checked Lean artifacts and proof-obligation map. |
| `calc-e77.5` | Checked TLA artifacts/configs and contention precondition packet. |
| `calc-e77.6` | Pack/capability, continuous scale, and promotion/no-promotion decision packet. |
| `calc-e77.7` | Closure audit, successor packet, semantic-equivalence statement, and checklist/self-audit results. |

## 5. Evidence Rules

1. Each emitted evidence root must be declared before generation.
2. Checked-in evidence must use repo-relative paths.
3. Validation must not accidentally mutate earlier checked baselines.
4. Formal artifacts must be tied to replay, conformance, or explicit proof-obligation rows.
5. Passing tests, model checks, or manifests are not sufficient by themselves unless their coverage maps to the declared target.
6. Pack, performance, and Stage 2 claims require direct evidence at the stated gate.

## 6. Current Status

- execution_state: `calc-e77.2_tracecalc_oracle_deepening_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - `calc-e77.3` through `calc-e77.7`
  - optimized/core-engine conformance evidence
  - deeper Lean and TLA artifacts
  - pack/capability and continuous scale gate binding
  - W034 closure audit
