*Posted by Codex agent on behalf of @govert*

# HANDOFF-CALC-002: Recalc Session And Plan Template Support For W050

## Purpose
This handoff packet requests canonical OxFml-side support for the W050
session-shaped prepared-callable and plan-template model.

The goal is not to move formula-language ownership into OxCalc. The goal is
to let OxCalc consume OxFml-owned parse, bind, semantic-plan, invocation,
result, trace, and replay truth through the public consumer runtime facade
without adding private adapters around OxFml internals.

## Source Scope
- Source workset: `W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK`
- Source bead: `calc-cwpl.H1`
- Driving local evidence:
  - `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` sections 22.17, 22.18, 22.20, and 22.21
  - `docs/worksets/W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK.md`
  - `docs/upstream/NOTES_FOR_OXFML.md` sections 68-80
  - `docs/test-runs/core-engine/treecalc-local/w050-b8-treecalc-session-corpus-001`
  - `docs/test-runs/core-engine/w050-f3-derivation-trace-invoke-outcome-001`
- OxFml references reviewed read-only:
  - `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
  - `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
  - `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_IMPLEMENTATION_PROGRAM_V1.md`
  - `../OxFml/crates/oxfml_core/src/consumer/runtime/mod.rs`
  - `../OxFml/crates/oxfml_core/src/seam/mod.rs`

## Current Compatibility Position
OxCalc now exercises the current public OxFml V1 consumer runtime facade:

1. `OxfmlRecalcSessionDriver::ensure_prepared` maps to
   `RuntimeSessionFacade::open_managed_session`.
2. TreeCalc full-result invocation maps to `RuntimeSessionFacade::execute`
   because the current coordinator/evidence path needs the full
   `RuntimeFormulaResult` surface.
3. Managed commit is exercised through
   `RuntimeSessionFacade::execute_and_commit_managed` and
   `RuntimeSessionFacade::commit_managed`, but current
   `RuntimeManagedCommitResult` does not carry all full-result surfaces
   consumed by TreeCalc evidence and coordinator checks.
4. TreeCalc production invocation no longer uses the old
   `MinimalUpstreamHostPacket` production path.
5. Current V1 compatibility still requires OxCalc to derive prepared-callable
   identity, plan-template identity, hole bindings, formal-reference handles,
   and some replay/correlation diagnostics locally from public OxFml outputs.
6. Current V1 `PreparedCalls` trace mode gives OxCalc ordered prepared-call
   entries and returned values for derivation trace evidence, but not deeper
   parent/child invocation nesting.

This packet asks OxFml to decide which of those compatibility derivations
should become canonical public runtime-facade fields.

## Requested Canonical OxFml Clauses

### 1. Prepared Callable Surface
OxFml should expose, through the public consumer runtime facade, a canonical
prepared-callable result or snapshot.

Requested fields or equivalent canonical accessors:
1. `prepared_callable_key`
2. `formula_stable_id`
3. source formula text version and source token identity
4. `library_context_snapshot_ref`
5. structure-context identity
6. caller/locus context that affects binding or relative reference meaning
7. `PlanTemplate` identity or handle
8. `HoleBindings` identity or handle
9. canonical formal-reference set
10. bind diagnostics and syntax diagnostics

Requested clause direction:
1. preparing a formula through the runtime facade should expose the same
   canonical callable identity whether the consumer uses one-shot execution
   or a managed session,
2. the prepared callable should be immutable for a given source/version,
   binding world, library-context snapshot, caller/locus context, and
   structure-context identity,
3. consumers should not need to inspect OxFml internals to identify or
   reuse the prepared callable.

### 2. PlanTemplate And HoleBindings Surface
OxFml should expose canonical plan-template identity and hole-binding
identity for prepared formulas.

Requested fields or equivalent canonical accessors:
1. `shape_key`
2. `dispatch_skeleton_key`
3. `plan_template_key`
4. ordered template holes with stable `hole_id`
5. canonical hole kind
6. `hole_binding_fingerprint`
7. per-hole binding payload category
8. template reuse/cache trace identity

Requested clause direction:
1. `shape_key` should abstract literal values and concrete reference targets
   while preserving operator/function nesting, arity, lazy-control posture,
   and lambda-binding structure,
2. `dispatch_skeleton_key` should extend `shape_key` with bind-time function
   dispatch and library-context identity,
3. `plan_template_key` should extend dispatch identity with semantic-plan
   structure, coercion/helper shape, capability requirements, and
   argument-preparation profile requirements,
4. `PreparedCallable` should be representable as `PlanTemplate` plus
   `HoleBindings`, or through an equivalent OxFml-owned artifact shape,
5. current OxCalc compatibility fingerprints should remain temporary and
   should be retired when canonical OxFml fields are available.

### 3. Formal Reference And Input Transport
OxFml should expose a canonical formal-reference/input transport for cell
formula invocation.

Requested fields or equivalent canonical accessors:
1. stable `reference_handle`
2. normalized reference text or canonical reference descriptor
3. reference kind, including direct, relative/caller-sensitive, unresolved,
   host-sensitive, dynamic-potential, capability-sensitive, and
   shape/topology-sensitive families where applicable
4. caller-anchor/address-mode dependence where semantic meaning depends on it
5. reference-to-hole or reference-to-input binding identity
6. per-invocation value/reference/callable input binding records

Requested clause direction:
1. OxCalc should bind invocation inputs by canonical formal reference or
   hole identity, not by synthetic A1 cell values and defined-name stand-ins,
2. OxCalc remains responsible for workbook structure, graph targets,
   invalidation, scheduling, and publication,
3. OxFml remains responsible for formula-language reference identity,
   binding, formal parameter shape, and callable invocation semantics.

### 4. Full Managed Result Parity
OxFml should either extend managed execution/commit results or provide a
stable linked result record so the managed-session path can replace
TreeCalc's current full-result invocation path.

Requested result families:
1. `RuntimeFormulaResult` or equivalent full result surface
2. candidate result identity and payload
3. commit decision and commit bundle identity
4. reject record identity and structured reject detail
5. returned-value surface kind and payload summary/category
6. execution-outcome surface
7. comparison/publication surfaces where admitted
8. trace events and replay capture/projection handles
9. artifact reuse report or equivalent cache/reuse diagnostics

Requested clause direction:
1. managed session lifecycle should not hide any coordinator-relevant result
   family that one-shot runtime execution exposes,
2. if commit remains separate from execution, the commit result should carry
   or link to the exact candidate/execution result it committed or rejected,
3. rejected managed work must remain no-publish and replay-visible.

### 5. Trace Replay And Correlation Columns
OxFml should preserve stable structured correlation and replay columns for
the facts OxCalc currently records in `session_path_evidence.json`.

Requested fields or equivalent canonical accessors:
1. `candidate_result_id`
2. `commit_attempt_id`
3. `reject_record_id` or stable reject trace identity
4. `trace_correlation_id`
5. returned-value surface classification
6. replay-facing diagnostic categories
7. `shape_key`, `dispatch_skeleton_key`, `plan_template_key`
8. template hole identity and binding identity
9. capability/hole taxonomy columns where admitted by the receiving OxFml
   and OxFunc plans
10. parent/child prepared-call invocation structure, or explicit confirmation
    that ordered prepared-call records are the public trace granularity
11. kernel-returned value per prepared call

Requested clause direction:
1. replay/correlation facts should be structured fields, not diagnostic
   strings that OxCalc must parse,
2. OxCalc's B8 `session_path_evidence.json` should remain an OxCalc local
   evidence schema, not become the shared OxFml schema,
3. OxFml should define the canonical shared schema or result projection that
   supersedes the current compatibility evidence packet.
4. derivation-trace consumers should not need to reconstruct hidden formula
   semantics to infer invocation hierarchy beyond the structure OxFml chooses
   to expose.

### 6. Bind-Visible Metadata Invalidation
OxFml and OxFunc should expose the metadata needed for OxCalc to invalidate
prepared callables when bind-visible function behavior changes.

Requested fields or equivalent canonical accessors:
1. bind-visible `ArgPreparationProfile` metadata version
2. affected callable/function identity set when narrower invalidation is
   possible
3. canonical profile name and stable serialization
4. relationship to `StructureContextVersion` or equivalent structure/bind
   context identity

Requested clause direction:
1. if any existing OxFunc function changes its `ArgPreparationProfile`, the
   change is bind-visible to OxCalc,
2. OxCalc may conservatively rebind all formulas when only a global metadata
   version is available,
3. OxCalc may target narrower rebinds only when OxFml/OxFunc provide
   canonical affected-callable metadata.

### 7. Folding And Template-Reuse Trace
If OxFml performs compile-time folding or other identity-affecting plan
normalization, that fact should enter canonical plan-template identity and
trace output.

Requested fields or equivalent canonical accessors:
1. folded-plan identity or stable folding trace field
2. reason/classification for identity-affecting folding
3. template reuse/cache counters or trace events
4. collision/compatibility diagnostics for cache keys

Requested clause direction:
1. OxCalc should not infer compile-time folding from source text,
2. formulas that compile to the same canonical folded plan should share the
   OxFml-owned `plan_template_key` when OxFml admits that equivalence,
3. formulas that do not share canonical folded-plan identity should remain
   distinct in OxCalc evidence and caches.

## Proposed Normative Direction
These are not canonical OxFml text yet; they are candidate target statements
for synthesis on the OxFml side.

1. The public consumer runtime facade is the ordinary surface for OxCalc
   formula execution, repeated invocation, and managed-session lifecycle.
2. A prepared formula has a canonical prepared-callable identity owned by
   OxFml.
3. A prepared callable is decomposable into template identity plus binding
   identity, or an OxFml-owned equivalent that preserves the same semantics.
4. OxCalc receives formal reference/input bindings as canonical fields, not
   as synthetic workbook cells or private OxFml internals.
5. Managed-session execution and one-shot execution preserve equivalent
   coordinator-relevant result truth.
6. Candidate, commit, reject, trace, replay, returned-value surface, and
   template/hole identity facts are structured result fields.
7. Bind-visible function metadata changes are invalidation inputs for
   prepared callables.

## Migration And Fallback Impact

### If Accepted
1. OxCalc can retire synthetic A1 compatibility inputs after the public
   reference/input transport lands.
2. OxCalc can demote current V1 `PreparedCallable`, `PlanTemplate`, and
   `HoleBindings` fingerprints to migration checks and eventually remove
   them when canonical OxFml fields cover the same evidence.
3. TreeCalc can move from current full-result session invocation to a
   managed-session path once managed results carry or link to full result
   truth.
4. Replay artifacts can reference OxFml-owned structured correlation fields
   instead of diagnostic-derived strings.
5. Later W050 lanes can treat plan-template, hole-binding, metadata
   invalidation, and replay columns as canonical imported contracts.

### If Deferred
1. OxCalc can continue W050 local execution on the current public V1
   compatibility path.
2. OxCalc will keep current compatibility fingerprints and synthetic A1
   reference/input transport explicitly marked as temporary.
3. No private OxFml adapter should be introduced in OxCalc to bridge the
   missing fields.
4. W050 aggregate closure remains blocked on cross-repo acknowledgment or
   an explicit replacement plan for these canonical fields.

## Evidence And References
Current OxCalc evidence:
1. B1 first-call protocol: `c530858`
2. B2 Calculation Repository: `3b4adb7`
3. B3 public runtime session driver: `9e1a307`
4. B4 six-phase wave lifecycle: `ef2b52f`
5. B5 source-reference handles: `6ae3046`
6. B6 opaque result family coverage: `0c8fbb5`
7. B7 session-driven TreeCalc production path: `f86a034`
8. B8 session corpus evidence packet: `d82834c`
9. B9 V1 compatibility ledger: `8ea8bfd`
10. Lane B status/epic evidence: `e4ec0a9`, `175ab8a`

Validation commands already exercised for the supporting evidence include:
1. `cargo test -p oxcalc-core`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test -p oxcalc-core session_driver_ -- --nocapture`
4. `cargo test -p oxcalc-core treecalc_runner_emits_local_run_artifacts -- --nocapture`
5. `cargo run -p oxcalc-tracecalc-cli -- treecalc w050-b8-treecalc-session-corpus-001`
6. `scripts/check-worksets.ps1`
7. `br dep cycles`

The B8 checked-in evidence root is:

`docs/test-runs/core-engine/treecalc-local/w050-b8-treecalc-session-corpus-001`

## Open Questions For OxFml
1. Should OxFml expose `PreparedCallable`, `PlanTemplate`, and
   `HoleBindings` by those names, or should it provide equivalent canonical
   runtime-facade records with OxFml-preferred names?
2. Should managed commit carry a full result payload directly, or should it
   carry stable links to the exact execution result and candidate result?
3. Which formal-reference families should be mandatory in the first public
   transport, and which should remain optional/admitted-later?
4. Which trace/replay columns should be in the runtime result versus a replay
   projection service?
5. Should compile-time folding be represented as a folded semantic-plan
   identity, a trace field, or both?
6. What metadata versioning should OxFml/OxFunc expose for
   `ArgPreparationProfile` changes?

## Requested Next Step
Please review this packet against the current OxFml consumer-runtime and
semantic-plan docs and determine:

1. which clauses should be promoted directly into OxFml canonical text,
2. which field names or artifact names should be adapted,
3. which pieces require OxFunc cooperation,
4. which pieces remain deferred pending more OxCalc evidence.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFml-side acknowledgment and canonical integration
  - OxFml response on preferred field/artifact names
  - OxFunc cooperation for `ArgPreparationProfile` and any capability or
    folding fields that live in function metadata
  - OxCalc migration after receiving-side acknowledgment
