# W033 Object Vocabulary And LET/LAMBDA Boundary

Status: `calc-uri.6_entry_packet`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.6`
Created: 2026-05-04

## 1. Purpose

This packet aligns the W033 object vocabulary across OxCalc, read-only OxFml seam inputs, and opaque OxFunc-facing packet assumptions.

It also states the narrow `LET`/`LAMBDA` carrier fragment that W033 admits into scope. The fragment is included only because local binding, lambda values, callable identity, invocation shape, and dependency/runtime-effect visibility can affect OxCalc-visible behavior. It does not transfer general OxFunc semantic-kernel ownership into OxCalc.

## 2. Vocabulary Alignment

| W033 term | OxCalc meaning | OxFml / FEC / F3E meaning consumed by OxCalc | OxFunc relation | First W033 use |
|---|---|---|---|---|
| `structural_snapshot` | Immutable or versioned structural truth for nodes, formulas, names, and host-visible structure. | Input to evaluator sessions as host/facade context. | none directly | state model, refinement, replay identity |
| `runtime_view` | Derived view assembled for evaluation, recalc, overlays, and publication decisions. | May include evaluator-facing packet facts and host query responses. | none directly | state separation, replay bridge |
| `candidate_result` | Work result that may be accepted or rejected but is not public publication. | Evaluator result/candidate packet before OxCalc coordinator publication. | Function results may feed the candidate through OxFml. | candidate-not-publication invariant |
| `commit_bundle` | Coordinator-consumable accepted bundle that can become public only through OxCalc publication rules. | FEC/F3E commit facts and correlation keys. | none directly | commit atomicity and fence checks |
| `typed_reject` | No-publish outcome with stable reject class, detail, and replay correlation. | OxFml-owned reject taxonomy and FEC/F3E reject records. | Function/value errors may be payload causes, but taxonomy remains through OxFml/FEC. | reject-is-no-publish invariant |
| `publication` | OxCalc-owned public state transition that updates observable values/metadata atomically. | OxFml does not own publication. | none directly | atomic publication and conformance |
| `fence` | Compatibility guard over snapshot/epoch/capability/session facts before publication. | OxFml/FEC session and commit facts supply fence inputs. | none directly | stale-fence-no-publish model |
| `publication_epoch` | OxCalc-owned public version/epoch identifier. | Correlates to consumed evaluator/session facts but remains coordinator-owned. | none directly | pinned readers, replay identity |
| `trace_identity` | Stable event or trace key used to correlate evaluator facts, coordinator transitions, and replay artifacts. | OxFml trace/facade/replay keys are imported. | Prepared-call and callable carrier identities may appear in trace detail. | replay/witness bridge |
| `replay_identity` | Stable scenario/run/witness/reduction identity. | OxFml fixture and retained witness identities are read-only inputs. | Function/callable fixture identities are upstream inputs only. | witness bridge and pack mapping |
| `static_dependency` | Dependency known before evaluation from formula/bind facts. | OxFml normalized references and bind facts can contribute. | Reference-returning function semantics remain OxFunc-owned where applicable. | invalidation closure |
| `runtime_dependency` | Dependency discovered or narrowed during evaluation. | OxFml runtime-derived effects, trace facts, and prepared results can expose it. | Function kernels such as `INDIRECT` and callable invocation may produce upstream facts through OxFml. | dynamic dependency model |
| `runtime_effect_fact` | Evaluation-emitted fact that affects invalidation, replay, scheduling, or explanation. | OxFml delta/effect/trace facts are imported. | OxFunc may cause facts through prepared results but W033 consumes only surfaced facts. | replay bridge, under-invalidation guard |
| `overlay` | Runtime-derived state attached to an epoch/snapshot that cannot mutate structural truth. | OxFml packet facts may populate overlays. | none directly | overlay retention and safety |
| `callable_carrier` | A typed carrier fact that can appear in evaluator/prepared-result surfaces and may affect replay or runtime-effect visibility. | OxFml/OxFunc minimum shared callable carrier. | OxFunc owns callable semantic behavior beyond the carrier. | LET/LAMBDA boundary |
| `invocation_contract_ref` | Stable semantic reference to how a callable may be invoked. | OxFml consumes/records current shared interface candidate. | OxFunc owns the semantic invocation contract behind the reference. | LET/LAMBDA boundary |

## 3. LET/LAMBDA Carrier Fragment In Scope

W033 admits only the following carrier-level facts:

| Carrier fact | W033 reason | Source input | First downstream packet |
|---|---|---|---|
| `opaque_callable_identity` | Needed to correlate callable formation, invocation, replay, and possible runtime effects without importing AST or kernel semantics. | OxFml LET/LAMBDA prep; shared interface freeze candidate | replay/witness bridge; Lean/TLA abstract ADTs |
| `origin_kind` | Needed to distinguish inline lambda, helper-bound lambda, adopted defined-name callable, returned lambda, and other origin classes when replay or trace meaning depends on origin. | OxFml LET/LAMBDA prep; callable fixtures | vocabulary, replay bridge, handoff/watch |
| `capture_mode` | Needed to preserve lexical capture meaning and prevent dynamic rebinding from changing observable behavior. | OxFml LET/LAMBDA prep | Lean abstract carrier, replay bridge |
| `arity_shape` | Needed to distinguish correct invocation, under-application, optional/omitted behavior, and typed reject classes. | OxFml higher-order/callable fixtures | metamorphic/differential families; handoff/watch |
| `invocation_contract_ref` | Needed to keep typed invocation meaningful without exposing implementation-specific callbacks or raw ASTs. | shared interface freeze candidate | replay bridge; handoff/watch |
| `call_trace_identity` | Needed to correlate prepared calls, nested lambda invocations, returned callable values, and witness artifacts. | prepared-call replay fixtures | replay/witness bridge |
| `dependency_visibility` | Needed only where callable or `LET` capture changes static/runtime dependency facts visible to OxCalc. | prepared-call replay fixtures; OxFml effect/trace docs | dynamic dependency rows, conformance |
| `runtime_effect_visibility` | Needed only where callable execution changes trace, reject, or runtime-effect facts consumed by OxCalc. | OxFml delta/effect/trace docs; fixtures | replay bridge, pack mapping |

These richer details may remain provenance or replay sidecar facts when they are not mandatory shared carrier fields:

1. parameter names,
2. capture names,
3. body kind or body detail,
4. exact helper-local AST shape,
5. display/publication policy for callable values.

They may not disappear when W033 needs them for replay, explanation, dependency visibility, or mismatch triage.

## 4. Carrier Assumptions

W033 assumes the following until a later OxFml handoff/watch row changes the surface:

1. `LET` binding is lexical and sequential, not dynamically rebound after lambda creation.
2. Lambda parameter shadowing excludes shadowed helper names from capture reporting.
3. Exact free-helper capture is preferable where knowable; "all visible helper names" is too weak for replay or dependency claims.
4. Callable values are semantic values even if worksheet publication policy for top-level callable results is narrower.
5. Typed invocation over an opaque callable identity is acceptable only when origin kind, capture mode, arity shape, and invocation-contract meaning remain recoverable.
6. Parameter names, capture names, and body detail can remain provenance/replay detail for the current W033 pass when the minimum carrier remains smaller.
7. Ordinary function kernels, value coercion, and catalog truth stay OxFunc-owned and opaque to W033.
8. W033 consumes callable facts only through OxFml/FEC/F3E/prepared-result surfaces; it does not call or define OxFunc kernels.

## 5. Handoff And Watch Pressure

This packet does not file an OxFml handoff. It records the watch rows that `calc-uri.15` must evaluate.

| Watch ID | Pressure | Current classification | Possible later action |
|---|---|---|---|
| `W033-WATCH-LL-001` | Minimum callable carrier may need stable fields for origin kind, capture mode, arity shape, and invocation-contract meaning if OxCalc replay/conformance needs them. | `handoff_watch` | Handoff if upstream canonical text leaves these unrecoverable. |
| `W033-WATCH-LL-002` | Parameter/capture/body detail may stay provenance-only, but W033 replay/witness artifacts may need stable access to that provenance. | `handoff_watch` | Handoff or local deferred rationale depending on replay bridge findings. |
| `W033-WATCH-LL-003` | Callable publication policy can affect candidate vs commit consequences or typed no-publish outcomes. | `handoff_watch` | Handoff if W033 conformance sees coordinator-visible publication/reject ambiguity. |
| `W033-WATCH-LL-004` | Callable/provider/gating interaction can affect typed reject families or retry-relevant outcomes. | `deferred_open_lane` | Defer unless concrete W033 evidence makes it coordinator-visible. |
| `W033-WATCH-LL-005` | LET/LAMBDA dependency visibility can affect dynamic invalidation when captures or invocation cross name/context boundaries. | `handoff_watch` | Handoff if OxFml surfaced facts are insufficient for over-invalidation without under-invalidation. |

## 6. Non-Claims

W033 does not claim:

1. final shared callable transport is frozen by OxCalc,
2. OxCalc owns callable semantic value behavior,
3. OxCalc owns function catalog or coercion semantics,
4. W033 covers broad non-`LET`/`LAMBDA` higher-order language expansion,
5. top-level callable worksheet publication policy is an OxCalc-owned semantic rule,
6. dynamic provider/runtime states are settled for callable lanes.

## 7. Status

- execution_state: `object_vocabulary_and_let_lambda_boundary_packet_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - handoff/watch rows still need `calc-uri.15` packetization
  - Lean, TLA+, replay, conformance, and pack packets have not yet consumed these carrier facts
  - no OxFml handoff is filed by this packet
  - no general OxFunc semantic-kernel claim is added by this packet
