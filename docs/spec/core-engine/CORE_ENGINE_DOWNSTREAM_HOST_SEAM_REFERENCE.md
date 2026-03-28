# CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md

## 1. Purpose and Status
This document defines the narrow OxCalc documentation slice that downstream hosts should use as seam-reference material.

Status:
1. active canonical companion,
2. downstream-host facing in emphasis,
3. intended for hosts such as `DNA OneCalc` that drive `OxFml` but do not depend on the OxCalc runtime,
4. interpretation-only and not a production host API freeze.

## 2. Why This Exists
`OxCalc` is not a required runtime dependency for a first downstream single-node host such as `DNA OneCalc`.

But the host-facing seam used to drive `OxFml` overlaps materially with the seam already consumed and documented between `OxCalc` and `OxFml`.

This note exists so downstream hosts can answer three questions without reading planning or historical material as if it were canonical:
1. which OxCalc docs are authoritative local seam-reference material,
2. which adjacent docs are supporting or temporary companions only,
3. which local docs are historical, mirrored, or negotiation-state material and therefore must not be treated as the shared seam source of truth.

## 3. Interpretation Rule
The seam is shared, but authority is split.

Interpret the OxCalc-local seam-reference slice under these rules:
1. `OxFml` remains authoritative for evaluator-side semantics, host-policy semantics, and the canonical shared seam meaning,
2. `OxCalc` is authoritative only for its local coordinator-facing consumption, publication, and host-packet interpretation requirements,
3. downstream hosts may use OxCalc docs to understand consumed host-packet shape and coordinator-facing non-assumptions,
4. downstream hosts must not treat OxCalc docs as permission to invent a private evaluator contract when OxFml has not frozen the shared meaning.

## 4. Authoritative OxCalc Seam-Reference Slice
The authoritative local seam-reference set for downstream hosts is:
1. `README.md`
2. `CHARTER.md`
3. `OPERATIONS.md`
4. `CURRENT_BLOCKERS.md`
5. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
6. `docs/spec/README.md`
7. `docs/spec/core-engine/CORE_ENGINE_ARCHITECTURE.md`
8. `docs/spec/core-engine/CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
9. `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
10. this document

These files are the local authority set because they define:
1. repo scope and ownership,
2. handoff and completion discipline,
3. current blocker and active-work status,
4. canonical spec filtering,
5. the coordinator-facing seam requirements that OxCalc expects to consume.

## 5. Supporting Companions For Downstream Hosts
The following docs are valid downstream reference material, but they are supporting companions rather than the core OxCalc authority set:
1. `docs/spec/core-engine/CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md`
2. `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`
3. `docs/spec/core-engine/CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md`

Use them this way:
1. `CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` is the first implementation-backed deterministic upstream-host packet companion and is useful when a downstream host needs to understand the currently exercised packet shape that drives real `OxFml` paths,
2. `CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md` is the right local reference for how OxCalc intends to consume OxFml-backed bind and evaluation products in the first TreeCalc-ready engine,
3. `CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` is only the temporary tracker for narrower open topics and non-assumptions that remain under note exchange.

## 6. Non-Authority And Historical Material
The following local material must not be treated as the current OxCalc seam-reference source of truth:
1. `docs/spec/fec-f3e/*` because it is an OxFml-owned mirror set,
2. `docs/spec/core-engine/FOUNDATION_ARCHITECTURE_SNAPSHOT.md` and `docs/spec/core-engine/FOUNDATION_OPERATIONS_SNAPSHOT.md` because they are local Foundation snapshots,
3. `docs/upstream/*` because those are note-exchange and observation docs rather than canonical seam text,
4. `docs/handoffs/*` because handoffs and receipts are state records, not stable seam-reference docs,
5. `docs/spec/core-engine/archive/*` because archive content is historical by design.

## 7. Downstream Host Packet Rule
For downstream hosts, the current OxCalc host-packet reference rule is:
1. use `CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` only as the first deterministic packet reference,
2. treat that packet as implementation-backed reference material rather than as a frozen production API,
3. keep any downstream interpretation aligned with `CORE_ENGINE_OXFML_SEAM.md`,
4. escalate shared seam changes through `OxFml` canonical docs and then update OxCalc local seam-reference docs as needed to avoid drift.

This matters especially for:
1. caller-anchor or relative-reference carriage,
2. execution-restriction transport,
3. publication and topology consequence breadth,
4. broader registered-external semantics.

## 8. Evidence-Backed Current Floor
The first deterministic host-packet floor described here is backed by current local code and fixtures:
1. `src/oxcalc-core/src/upstream_host.rs`
2. `src/oxcalc-core/src/treecalc.rs`
3. `src/oxcalc-core/tests/upstream_host_scaffolding.rs`
4. `docs/test-fixtures/core-engine/upstream-host/README.md`
5. `docs/test-fixtures/core-engine/upstream-host/MANIFEST.json`

Those artifacts prove the local floor is more than planning text, but they do not change the ownership split in Section `3`.
