# CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md

## 1. Purpose and Status
This document defines the narrow OxCalc documentation slice that downstream hosts should use as seam-reference material.

Status:
1. active canonical companion,
2. downstream-host facing in emphasis,
3. intended for hosts such as `DNA OneCalc` that drive `OxFml` but do not depend on the OxCalc runtime,
4. interpretation-only and not a production host API freeze,
5. distinct from the OxCalc tree-runtime consumer contract now defined in `CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` for TreeCalc-style hosts.

## 2. Why This Exists
`OxCalc` is not a required runtime dependency for a first downstream single-node host such as `DNA OneCalc`.

But the host-facing seam used to drive `OxFml` overlaps materially with the seam already consumed and documented between `OxCalc` and `OxFml`.

This note exists so downstream hosts can answer three questions without reading planning or historical material as if it were canonical:
1. which OxCalc docs are authoritative local seam-reference material,
2. which adjacent docs are supporting or temporary companions only,
3. which local docs are historical, mirrored, or negotiation-state material and therefore must not be treated as the shared seam source of truth.

Actual OxCalc runtime consumers should not use this document as their primary contract.
They should use `CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`.

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
4. `docs/WORKSET_REGISTER.md`
5. `docs/BEADS.md`
6. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
7. `docs/spec/README.md`
8. `docs/spec/core-engine/CORE_ENGINE_ARCHITECTURE.md`
9. `docs/spec/core-engine/CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
10. `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
11. this document

Classification: **canonical-local-reference**.

These files are the local authority set because they define:
1. repo scope and ownership,
2. handoff and completion discipline,
3. current workset order and execution-method ownership,
4. canonical spec filtering,
5. the coordinator-facing seam requirements that OxCalc expects to consume.

### 4.1 Document Classification Summary
The following table classifies every OxCalc document that a downstream host is likely to encounter.
See `docs/spec/README.md` for the classification vocabulary.

| Document | Class | Downstream host use |
|---|---|---|
| `README.md` | canonical-local-reference | Repo scope, dependency constitution, entry point. |
| `CHARTER.md` | canonical-local-reference | Scope, ownership split, co-definition rule. |
| `OPERATIONS.md` | canonical-local-reference | Handoff and completion discipline. |
| `docs/WORKSET_REGISTER.md` | canonical-local-reference | Ordered workset truth and rollout shape. |
| `docs/BEADS.md` | canonical-local-reference | Local bead-method and execution-state ownership. |
| `docs/IN_PROGRESS_FEATURE_WORKLIST.md` | canonical-local-reference | High-level seam, TreeCalc, and replay feature map. |
| `docs/spec/README.md` | canonical-local-reference | Spec filter and reading order. |
| `docs/spec/core-engine/CORE_ENGINE_ARCHITECTURE.md` | canonical-local-reference | Core-engine architecture and evaluator boundary. |
| `docs/spec/core-engine/CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md` | canonical-local-reference | Candidate-versus-publication and coordinator rules. |
| `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` | canonical-local-reference | Actual OxCalc tree-runtime consumer contract for TreeCalc-style hosts. |
| `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` | canonical-local-reference | Canonical OxCalc-local seam companion. |
| this document | canonical-local-reference | Downstream host authority filter. |
| `docs/spec/core-engine/CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` | supporting-companion | First deterministic upstream-host packet; reference material only. |
| `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md` | supporting-companion | TreeCalc-facing consumer model and pipeline plan. |
| `docs/spec/core-engine/CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` | temporary-planning | Narrower open topics and non-assumptions; not seam authority. |
| `docs/spec/fec-f3e/*` | historical/non-authority | OxFml-owned mirror set; not OxCalc authority. |
| `docs/upstream/*` | historical/non-authority | Note-exchange and observation docs; not canonical seam text. |
| `docs/handoffs/*` | historical/non-authority | State records; not stable seam-reference docs. |
| `CURRENT_BLOCKERS.md` | historical/non-authority | Retired active blocker surface; kept only as a historical pointer. |
| `docs/spec/core-engine/archive/*` | historical/non-authority | Historical by design. |
| `docs/spec/core-engine/FOUNDATION_*_SNAPSHOT.md` | historical/non-authority | Local Foundation snapshots; not OxCalc-owned. |

## 5. Supporting Companions For Downstream Hosts
The following docs are valid downstream reference material, but they are supporting companions or temporary planning material rather than the core OxCalc authority set.
Actual runtime consumers should instead read `CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` before these companions.

### 5.1 Supporting-Companion Documents
Classification: **supporting-companion**.
1. `docs/spec/core-engine/CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md`
2. `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`

Use them this way:
1. `CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` is the first implementation-backed deterministic upstream-host packet companion and is useful when a downstream host needs to understand the currently exercised packet shape that drives real `OxFml` paths. It is reference material for understanding the live seam, not a host API to adopt verbatim (see Section 7),
2. `CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md` is the right local reference for how OxCalc intends to consume OxFml-backed bind and evaluation products in the first TreeCalc-ready engine.

### 5.2 Temporary-Planning Documents
Classification: **temporary-planning**.
1. `docs/spec/core-engine/CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md`

Use it this way:
1. `CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` is only the temporary tracker for narrower open topics and non-assumptions that remain under note exchange. It is not seam authority. It will be superseded or retired when the topics it tracks are consumed into executed seam intake work.

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

### 7.1 Interpretation Model For The First Deterministic Upstream Host Packet
The upstream host packet defined in `CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` exists because OxCalc needed a reusable, deterministic way to drive real OxFml paths in automated scaffolding.

Downstream hosts such as `DNA OneCalc` should interpret it as follows:

1. **Shared OxFml seam requirements** — fields that any host driving OxFml must carry in some form because OxFml requires them as input:
   - formula text and formula artifact identity,
   - caller anchor and structure-context identity where reference meaning depends on them,
   - binding-world facts such as cell fixtures, defined-name bindings, and table context,
   - typed host-query mode facts,
   - runtime library-context snapshot.

2. **OxCalc coordinator-specific fields** — fields that the packet carries because OxCalc needs them for coordinator, replay, and publication purposes. Downstream hosts that do not run the OxCalc coordinator may narrow, rename, or omit these if their own host profile does not need them:
   - `fixture_input_id` and `formula_slot_id` as local scaffolding identifiers,
   - `structure_context_version` as coordinator-owned version tracking,
   - replay-projection metadata.

3. **Non-adoption rule** — a downstream host must not copy this packet verbatim as its own host API. The packet is wider than what a single-node host needs, carries OxCalc-specific naming, and does not represent a shared seam freeze. The correct approach is:
   - understand the seam inputs OxFml requires from any host (per OxFml canonical docs),
   - understand which of those inputs the OxCalc packet already carries and how,
   - narrow and productize the host's own packet shape against its own host-profile ladder,
   - preserve enough seam-facing honesty that OxFml-owned semantics are not reinterpreted locally.

Current public-entry note:
1. the current OxCalc-backed deterministic packet is now exercised against OxFml's landed `consumer::runtime` and `consumer::replay` public modules,
2. downstream hosts should treat that as the live ordinary integration floor rather than expecting direct `oxfml_core::host` access to remain public.

### 7.2 Still Provisional Or Narrower-Than-Final
The following aspects of the host packet are still provisional or narrower than the eventual seam:
1. caller-anchor and address-mode carriage for relative-reference families beyond the first exercised TreeCalc subset,
2. execution-restriction transport shape beyond the current semantic minimum,
3. publication and topology consequence breadth beyond the currently exercised local floor,
4. broader registered-external execution semantics beyond the current `registered_external_present` boolean.

Downstream hosts should not build against these provisional areas as if they were frozen.
They should track the corresponding residual lanes in `CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` and in `CORE_ENGINE_OXFML_SEAM.md` Sections 21–22.

## 8. Evidence-Backed Current Floor
The first deterministic host-packet floor described here is backed by current local code and fixtures:
1. `src/oxcalc-core/src/upstream_host.rs`
2. `src/oxcalc-core/src/treecalc.rs`
3. `src/oxcalc-core/tests/upstream_host_scaffolding.rs`
4. `docs/test-fixtures/core-engine/upstream-host/README.md`
5. `docs/test-fixtures/core-engine/upstream-host/MANIFEST.json`

Those artifacts prove the local floor is more than planning text, but they do not change the ownership split in Section `3`.
They also show that the current ordinary OxCalc-facing host packet is now routed through the OxFml V1 consumer facade, not through private host substrate modules.

## 9. Cross-Reference Map
This section lists the key cross-links among the OxCalc seam-reference docs for downstream hosts.

| From | To | Reason |
|---|---|---|
| `docs/spec/README.md` | this document | Entry point directs downstream hosts here. |
| `README.md` | this document | Startup docs list includes this document. |
| this document | `CORE_ENGINE_OXFML_SEAM.md` | Canonical local seam companion (Section 4, item 9). |
| this document | `CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` | Actual runtime-consumer contract for TreeCalc-style hosts. |
| this document | `CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` | First deterministic host packet (Section 5.1, item 1; Section 7). |
| this document | `CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md` | TreeCalc consumer model (Section 5.1, item 2). |
| this document | `CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` | Open topics and non-assumptions (Section 5.2). |
| `CORE_ENGINE_OXFML_SEAM.md` | this document | Section 1 directs downstream hosts here first. |
| `CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` | this document | Section 1 and Section 6 reference this document for authority filtering. |
| `CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md` | this document | Section 1 directs downstream hosts here first. |
| `CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` | this document | Section 1 directs downstream hosts here first. |
| `docs/IN_PROGRESS_FEATURE_WORKLIST.md` IP-13 | this document | Downstream integration seam-reference rule. |
