# CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md - Theory Exposition and Complementary Paths

## 1. Purpose and Scope
This document is the theory-focused companion to `CORE_ENGINE_FORMAL_MODEL.md`.

It does two things:
1. confirms coverage of the main deep-research and review corpus used for the current core-engine direction,
2. explains the underlying theory stack and the major complementary paths that are intentionally not baseline right now.

Normative source-of-truth remains:
- `CHARTER.md`
- `ARCHITECTURE_AND_REQUIREMENTS.md`
- `CORE_ENGINE_FORMAL_MODEL.md`
- `OPERATIONS.md`

This document is explanatory and synthesis-supporting, not a replacement for normative contracts.

## 2. Coverage Confirmation (Research -> Model)
The current core-engine model is comprehensive with bounded, explicit deferrals. Coverage status:

| Source family | Core contribution | Current status |
|---|---|---|
| DAG theory catalog + family map + transfer + synthesis (`R-DAG-02/03/04/05`) | Layered baseline: deterministic topo/SCC, fixed-point framing, invalidation-state discipline, staged advanced lanes | Integrated |
| External reconciliation + obligations + empirical packs (`R-DAG-09/10/11`) | Proof/pack candidate set, staged Now/Next/Later adoption discipline | Integrated with partial deferral by design |
| FEC/F3E current spec + observations + conformance matrix (`FEC-SPEC/OBS/MATRIX`) | Transactional evaluator seam, typed deltas, typed rejects, snapshot/capability fences, spill events | Integrated, with concurrency-hardening open gates |
| Deep design/review outputs (`DR-CORE-DESIGN`, `DR-CHATGPT-PRO`, `DR-CLAUDE-OPUS`, `DR-DUAL-*`) | Option-B staged architecture pressure-tested against alternatives and failure modes | Integrated (adapted where sources were over-normative) |
| Synthesis promotion runs (`SYN-20260308-CORE`, `SYN-20260309-IMPROVEMENT`, `SYN-20260309-LAYOUT`) | Promotion decisions, accepted/adapted/deferred split, source-of-truth updates | Integrated |
| Formatting/visibility model (`FMT-VIS-MODEL`) | Formula-semantic formatting boundary and visibility policy constraints | Integrated (conditional-format observability still provisional) |

Comprehensive does not mean "everything promoted." It means all high-value inputs are either:
1. integrated into baseline contracts, or
2. retained as explicit deferred/provisional items with rationale and gates.

## 3. Underlying Theory Stack

### 3.1 Semantic Layer: Fixed Points and Stabilization
The semantic target is a stabilized result per epoch, with cycle behavior controlled by profile policy. This comes from order-theoretic fixed-point framing and SCC-local cycle handling rather than from any single scheduler implementation (`R-DAG-02`, `R-DAG-05`, `R-DAG-10`).

Core transfer:
- semantics are defined independently from execution strategy,
- cycle modes (`PriorValueFallback`, `CycleError`, `Iterative`) are profile-scoped semantic policies,
- bounded iterative behavior is deterministic and traceable even when full fixed-point guarantees are not assumed.

### 3.2 Structural Graph Layer: Topo/SCC as Baseline Skeleton
Static DAG + SCC decomposition is the minimum robust substrate for deterministic spreadsheet recalc (`R-DAG-02`, `R-DAG-03`, `R-DAG-05`).

Core transfer:
- acyclic regions: deterministic topological evaluation,
- cyclic regions: SCC-isolated policy evaluation,
- structural edits: deterministic graph refresh and invalidation propagation.

### 3.3 Incremental Layer: Invalidation as First-Class State
The research converges on explicit invalidation state (`clean/stale/necessary/recomputed`) as the practical heart of correctness and performance (`R-DAG-02`, `R-DAG-05`, `R-DAG-07`, `R-DAG-08`).

Core transfer:
- dirty-closure remains baseline,
- invalidation state must be explicit and auditable,
- algorithm changes are allowed only if stabilized semantics and replay properties are preserved.

### 3.4 Dynamic-Dependency Layer: Structural Graph + Calc-Time Overlay
Dynamic references (INDIRECT/OFFSET-class behavior) require run-time observed dependencies to exist as an overlay over structural dependencies, not as silent mutation of structural truth (`R-DAG-03`, `R-DAG-05`, `R-DAG-10`, `DR-CORE-DESIGN`, `DR-DUAL-CLAUDE`).

Core transfer:
- effective dependency relation is structural deps union runtime-observed deps,
- overlay identity is fenced by snapshot/token/bind/profile context,
- overlay lifecycle/eviction must be epoch-safe and deterministic.

### 3.5 Spill and Virtual Region Theory
Dynamic-array spill semantics require explicit region events and targeted invalidation behavior. This is captured in FEC/F3E current typed spill events and in core overlay semantics (`FEC-SPEC`, `FEC-MATRIX`, `DR-CORE-DESIGN`).

Core transfer:
- spill state is a first-class derived artifact,
- spill takeover/clearance/blocked events are semantic scheduler inputs,
- spill-derived invalidation remains overlay-driven and replayable.

### 3.6 Transactional Seam Theory: FEC/F3E as Evaluator Boundary
The evaluator seam is modeled as a transactional lane (`prepare -> open_session/capability_view -> execute -> commit`) with strict fence checks and typed outcomes (`FEC-SPEC`, `FEC-OBS`, `FEC-MATRIX`).

Core transfer:
- accepted commit publishes one atomic derived bundle,
- rejected commit is a no-publish outcome with typed reject detail,
- seam emits evidence and deltas; global recalc policy remains engine/coordinator-owned.

### 3.7 Concurrency and MVCC Theory: Single-Publisher Discipline
The current proven-safe direction is staged concurrency with one publication authority and explicit snapshot/token/capability fences (`AAR-CORE`, `CORE-FORMAL`, `FEC-OBS`, `DR-CHATGPT-PRO`, `DR-DUAL-GPT54`).

Core transfer:
- Stage 1: sequential coordinator baseline,
- Stage 2: partitioned parallel evaluators behind same commit/publish authority,
- Stage 3: advanced concurrency lanes only after deterministic parity evidence.

### 3.8 Formatting, Display, and the Evaluation Boundary
Formatting is not purely display-only when formula semantics depend on format behavior. The current model keeps this through evaluator seams with profile gating (`FMT-VIS-MODEL`, `AAR-CORE`, `CORE-FORMAL`).

Core transfer:
- `TEXT(value, format_text)` is explicit format-string conversion by default,
- formatting-sensitive formula behavior crosses FEC/F3E and invalidation overlays when enabled,
- conditional-format observability remains explicitly provisional pending empirical closure.

### 3.9 Visibility as Scheduling Metadata, Not Semantic Truth
Visibility can affect priority but cannot alter stabilized outputs (`CORE-FORMAL`, `AAR-CORE`, `DR-CORE-DESIGN`, `DR-DUAL-CLAUDE`).

Core transfer:
- visible-first policy is optional and profile/policy-scoped,
- fairness/starvation bounds are required when enabled,
- equivalence proof/pack obligations are required before broad promotion.

### 3.10 External Streams and Time Models
External updates are baseline explicit operations with ordered envelopes. Differential/timely-style semantics are reserved for stream-heavy lanes (`R-DAG-02`, `R-DAG-03`, `R-DAG-04`, `R-DAG-05`, `R-DAG-09`).

Core transfer:
- stream behavior is versioned by profile (`ExternalInvalidationV0`, `TopicEnvelopeV1`, `RtdLifecycleV2`),
- deterministic ordering/dedupe is baseline,
- advanced timestamp/delta models are staged, not universal baseline.

## 4. Complementary Paths Not Taken as Baseline (Yet)

### 4.1 Dynamic Topological Maintenance as Default
Not baseline now:
- more complex invariants and fallback complexity than current proof/evidence maturity supports.

Retained value:
- strong latency gains for edit-heavy workloads.

Promotion trigger:
- `PACK.dag.dynamic_topo_vs_rebuild` parity + economics closure (`R-DAG-11`, `SYN-20260309-LAYOUT`).

### 4.2 Full Self-Adjusting Computation as Primary Runtime
Not baseline now:
- trace machinery and lifecycle complexity are high relative to current Stage-1/Stage-2 needs.

Retained value:
- best fit for high-dynamic-dependency hotspots.

Promotion trigger:
- scoped lane parity via `PACK.dag.dynamic_dependency_bind_semantics` and deterministic replay closure (`R-DAG-10`, `R-DAG-11`).

### 4.3 Whole-Language Incremental Lambda-Calculus/Derivative Engine
Not baseline now:
- broad transform burden and function-surface complexity are high.

Retained value:
- stronger incrementalization correctness pathway for selected function families.

Promotion trigger:
- bounded function-class pilots with measurable win and proof tractability (`R-DAG-02`, `R-DAG-04`).

### 4.4 Differential/Timely Model as General Recalc Backbone
Not baseline now:
- conceptual/runtime overhead not justified for ordinary workbook recalc.

Retained value:
- excellent fit for high-rate external stream profiles.

Promotion trigger:
- stream-heavy profile evidence where baseline envelope model is insufficient (`R-DAG-05`, `R-DAG-09`).

### 4.5 Semiring Provenance as Default Explainability Substrate
Not baseline now:
- integration and data-volume complexity are high for initial product stages.

Retained value:
- principled explainability and provenance composition.

Promotion trigger:
- explainability requirements that cannot be met with current trace metadata (`R-DAG-02`, `R-DAG-04`).

### 4.6 Full Lock-Free Speculative Coordinator from Phase 1
Not baseline now:
- current seam evidence is still single-thread oriented; contention replay hardening is an explicit gap.

Retained value:
- higher throughput and lower commit contention in later phases.

Promotion trigger:
- deterministic contention-replay closure and coordinator hardening evidence (`FEC-OBS`, `FEC-MATRIX`, `DR-CORE-DESIGN`).

## 5. What This Means for Design Work Going Forward
1. The baseline is a layered, deterministic, replay-first core with explicit runtime overlays and transactional evaluator seam.
2. Advanced lanes remain available and intentionally pre-framed, but they are gated by proof/pack economics.
3. Theory-to-pack translation is mandatory for promotion; theory statements without executable obligations remain non-baseline.

## 6. Source References

### 6.1 Core Governance and Model
- `AAR-CORE`: `ARCHITECTURE_AND_REQUIREMENTS.md`
- `CORE-FORMAL`: `CORE_ENGINE_FORMAL_MODEL.md`
- `OPS-CORE`: `OPERATIONS.md`

### 6.2 DAG Deep-Research Lane
- `R-DAG-02`: `research/runs/20260305-201430-dag-computation-theory-deep-research-pass-01/outputs/02_theory_and_math_catalog.md`
- `R-DAG-03`: `research/runs/20260305-201430-dag-computation-theory-deep-research-pass-01/outputs/03_algorithm_family_map.md`
- `R-DAG-04`: `research/runs/20260305-201430-dag-computation-theory-deep-research-pass-01/outputs/04_dnacalc_transfer_matrix.md`
- `R-DAG-05`: `research/runs/20260305-201430-dag-computation-theory-deep-research-pass-01/outputs/05_deep_research_synthesis.md`
- `R-DAG-09`: `research/runs/20260305-201430-dag-computation-theory-deep-research-pass-01/outputs/09_external_report_reconciliation.md`
- `R-DAG-10`: `research/runs/20260305-201430-dag-computation-theory-deep-research-pass-01/outputs/10_conformance_and_proof_obligations.md`
- `R-DAG-11`: `research/runs/20260305-201430-dag-computation-theory-deep-research-pass-01/outputs/11_empirical_pack_definitions.md`

### 6.3 FEC/F3E Current Best Spec Set (current)
- `FEC-SPEC`: `reference/conformance/excel-worksheet-engine/model/fec-f3e/FEC_F3E_REDESIGN_SPEC.md`
- `FEC-SYNTH`: `reference/conformance/excel-worksheet-engine/model/fec-f3e/FEC_F3E_REDESIGN_SYNTHESIS.md`
- `FEC-OBS`: `reference/conformance/excel-worksheet-engine/model/fec-f3e/FEC_F3E_REDESIGN_OBSERVATIONS.md`
- `FEC-MATRIX`: `reference/conformance/excel-worksheet-engine/model/fec-f3e/FEC_F3E_PROTOCOL_CONFORMANCE_MATRIX.csv`

### 6.4 Deep Design and Review Outputs
- `DR-CORE-DESIGN`: `prompts/runs/20260308-171858-core-engine-fec-f3e-deep-research-pack-01/responses/deep_research_core_engine_fec_f3e_design.md`
- `DR-CHATGPT-PRO`: `prompts/runs/20260308-171858-core-engine-fec-f3e-deep-research-pack-01/responses/chatgpt_pro_response.md`
- `DR-CLAUDE-OPUS`: `prompts/runs/20260308-171858-core-engine-fec-f3e-deep-research-pack-01/responses/claude_opus_response.md`
- `DR-DUAL-GPT54`: `prompts/runs/20260308-182605-core-engine-fec-f3e-dual-model-review-pass-01/responses/gpt54/03_review2_final.md`
- `DR-DUAL-CLAUDE`: `prompts/runs/20260308-184205-core-engine-fec-f3e-dual-model-review-pass-02/responses/claude/03_review2_final.md`

### 6.5 Formatting and Visibility Model
- `FMT-VIS-MODEL`: `reference/conformance/excel-worksheet-engine/model/EXCEL_FORMATTING_HIERARCHY_AND_VISIBILITY_MODEL.md`

### 6.6 Synthesis Promotion Runs
- `SYN-20260308-CORE`: `synthesis/runs/20260308-213253-core-engine-fec-f3e-synthesis-pass-02/outputs/synthesis_report.md`
- `SYN-20260309-IMPROVEMENT`: `synthesis/runs/20260309-004109-improvement-notes-synthesis-pass-01/outputs/synthesis_report.md`
- `SYN-20260309-LAYOUT`: `synthesis/runs/20260309-072109-core-engine-program-layout-synthesis-pass-01/outputs/synthesis_report.md`
