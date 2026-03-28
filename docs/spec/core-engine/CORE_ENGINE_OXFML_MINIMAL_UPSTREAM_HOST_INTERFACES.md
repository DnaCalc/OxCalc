# CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md

## 1. Purpose and Status
This document defines the first OxCalc-owned minimal upstream host interface package used to drive OxFml in deterministic automated scaffolding.

Status:
1. active implementation companion,
2. scoped to the first deterministic host-stand-in packet for automated tests,
3. not a freeze of the production OxCalc coordinator API,
4. intentionally narrower than the full W026 seam intake.

For downstream hosts such as `DNA OneCalc`, this document is seam-reference material only.
It describes the first implementation-backed packet that can drive real `OxFml` paths, but it does not make OxCalc a runtime dependency and it does not freeze a final host API.

## 2. Why This Exists
OxCalc now has a first seam-backed TreeCalc lane, but OxFml consumption should not depend on ad hoc host construction embedded inside one runtime path.

For automated scaffolding we need:
1. a reusable packet carrying host/coordinator-owned truths,
2. a deterministic adapter that can drive real OxFml parse, bind, semantic-plan, evaluation, candidate, and commit behavior,
3. a minimal field set that is honest about current ownership without over-freezing later coordinator APIs.

## 3. Ownership Rule
This minimal package preserves the current shared ownership split.

### 3.1 OxCalc-owned
OxCalc owns:
1. the stand-in packet as a test-host and coordinator-input carrier,
2. caller anchor and structure-context identity as host-owned truths,
3. cell fixtures, defined-name bindings, and table context supplied into OxFml,
4. the decision to widen this packet later or wrap it in a larger production coordinator transport.

### 3.2 OxFml-owned
OxFml remains authoritative for:
1. formula grammar and bind meaning,
2. evaluator semantics,
3. candidate, commit, reject, trace, and typed host-provider outcome meaning,
4. typed query bundle and library-context interpretation once the inputs are supplied.

## 4. Minimal Packet Shape
The first minimal upstream host interface package is:

### 4.1 Formula slot facts
1. `fixture_input_id`
2. optional `formula_slot_id`
3. `formula_stable_id`
4. `formula_text`
5. `formula_text_version`
6. `formula_channel_kind`
7. `caller_anchor`
8. optional `active_selection_anchor`
9. `structure_context_version`

### 4.2 Binding-world facts
1. `cell_fixture`
2. `defined_name_bindings`
3. optional `table_catalog`
4. optional `enclosing_table_ref`
5. optional `caller_table_region`

### 4.3 Typed query facts
1. `host_info_mode`
2. `rtd_mode`
3. `locale_context_kind`
4. optional `now_serial`
5. optional `random_value`
6. `registered_external_present`

### 4.4 Runtime catalog facts
1. optional `library_context_snapshot`

### 4.5 First replay capture projection
1. deterministic `FirstHostReplayCapturePacket` emission from the same minimal packet
2. carried `library_context_snapshot_ref` projection derived from the runtime catalog snapshot

## 5. Current Realized Minimal Behavior
The current realized package is intentionally narrow.

It supports:
1. direct OxFml recalculation through `SingleFormulaHost`,
2. deterministic bind-context projection for test scaffolding,
3. defined-name value and reference bindings,
4. cell fixtures,
5. first table-context carriage through `table_catalog`, `enclosing_table_ref`, and `caller_table_region`,
6. multiple bounded evaluator-facing structured-reference families on top of that same table-context packet, currently:
   - current-row structured reference
   - explicit-column aggregate
   - headers-section return
   - data-qualified multi-column aggregate
7. typed host-info stand-ins, including unsupported-query, provider-failure, directory-value, and mixed directory-value-plus-filename-provider-failure outcomes,
8. typed RTD stand-ins,
9. locale-context selection,
10. in-memory library-context snapshot carriage,
11. first replay-capture packet emission from the same deterministic host packet.

It does not yet widen to:
1. production coordinator API freeze,
2. full caller-anchor and address-mode closure,
3. final execution-restriction transport closure,
4. broader registered-external execution semantics,
5. broader TreeCalc replay and retained-witness lanes.

## 6. Automated Scaffolding Role
This packet is the first honest OxCalc-side answer to the OxFml stand-in packet lane:
1. tests can now build a deterministic packet,
2. the packet can drive real OxFml execution,
3. the same packet family can be reused by the TreeCalc seam-backed lane,
4. future fixture hosts can widen the packet without rewriting the ownership split.

For authority and interpretation:
1. `CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md` defines where this document sits in the OxCalc seam-reference set,
2. `CORE_ENGINE_OXFML_SEAM.md` remains the canonical OxCalc-local seam companion,
3. `CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` carries the narrower residual topics that are not yet closed here.

## 7. Current Code Surface
The current implementation lives in:
1. `src/oxcalc-core/src/upstream_host.rs`

The current live consumer is:
1. `src/oxcalc-core/src/treecalc.rs`

The current deterministic compare discipline is exercised through:
1. `scripts/compare-treecalc-local-run.ps1`

The current crate-level scaffolding tests live in:
1. `src/oxcalc-core/tests/upstream_host_scaffolding.rs`

The current checked-in fixture corpus lives in:
1. `docs/test-fixtures/core-engine/upstream-host`

## 8. Non-Assumptions
This document does not claim:
1. that W026 is fully discharged,
2. that the broader host/runtime seam is frozen,
3. that `registered_external_present` means broader external-provider execution scope is active,
4. that the packet already captures every later TreeCalc or product-host truth.

## 9. Immediate Use
Immediate intended use is:
1. automated OxFml-facing scaffolding in OxCalc,
2. deterministic stand-in host tests,
3. shared reuse by the current TreeCalc direct-host slice.

## 10. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - broader W026 bind/reference intake remains open beyond this minimal packet
  - caller-anchor/address-mode breadth, execution-restriction transport breadth, and broader publication/topology breadth remain narrower seam lanes
  - first table-context carriage and four bounded evaluator-facing structured-reference families are fixture-covered in the first corpus, but richer structured-reference evaluator families are not yet fixture-covered
  - this packet is ready for deterministic automated scaffolding, first capture-packet testing, and first data-driven fixture use, but not a production coordinator API freeze
