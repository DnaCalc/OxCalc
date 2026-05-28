# CHARTER.md — OxCalc Charter

## 1. Mission
OxCalc defines, implements, and proves the multi-node core engine model for DNA Calc.

It owns coordinator behavior, scheduling policy, invalidation policy, and epoch-safe publication semantics while preserving profile-defined semantic equivalence across runtime strategies.

## 2. Precedence
When guidance conflicts, precedence is:
1. `../Foundation/CHARTER.md`
2. `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`
3. `../Foundation/OPERATIONS.md`
4. this `CHARTER.md`
5. this repo `OPERATIONS.md`

## 3. Scope
In scope:
1. Core graph/overlay state model and coordinator transitions.
2. Commit publication fences and deterministic rejection handling.
3. Stage 1 baseline sequential coordinator and staged concurrency promotion criteria.
4. Visibility-priority policy (`VisibleFirst`) with semantic-equivalence and fairness constraints.
5. Tree-only to tree-grid-hybrid progression semantics and gap tracking.
6. Snapshot-layer workspace representation: structure, node input, namespace, formula-binding, dependency-shape, publication, and runtime-overlay state boundaries.
7. Rust-first realization of OxCalc-owned executable artifacts for the core engine and `TraceCalc` tooling/runtime.

Out of scope:
1. Formula grammar and evaluator protocol ownership (OxFml).
2. Function semantic kernels (OxFunc).
3. UI rendering and file-adapter implementation concerns.

## 4. FEC/F3E Co-definition Rule
1. OxCalc co-defines coordinator-facing clauses of the shared FEC/F3E contract.
2. Canonical shared protocol files remain owned by OxFml.
3. OxCalc contributions must be sent via explicit handoff packets with replay evidence.

## 5. Clean-room Rule
Allowed sources:
1. public specifications and documentation,
2. published research,
3. reproducible black-box observations.

Disallowed sources:
1. proprietary or restricted sources,
2. reverse engineering of internals,
3. decompilation/disassembly of Excel internals.

## 6. Product Claims And Closure
OxCalc reports product behavior and formal/proof status separately.

A coordinator policy or engine feature can be claimed for a declared product scope when:
1. the supported scope is explicit,
2. implementation exists beyond scaffolding,
3. the relevant local, replay, Excel, or formal checks for that scope passed,
4. exclusions and successor lanes are named.

Formal proof, pack promotion, and cross-repo seam uptake remain important but are separate status dimensions unless the declared claim depends on them.

A shared FEC/F3E coordinator-facing change additionally requires:
1. spec text and realization notes,
2. required pack expectation updates where applicable,
3. deterministic replay or equivalent evidence for affected behavior,
4. cross-repo impact assessment and handoff when OxFml-owned seam text changes.

## 7. Implementation Direction
1. OxCalc-owned executable realization is Rust-first from this point onward.
2. The active repo implementation is the Rust workspace under `src/`.
3. Historical baseline runs and checked-in artifacts remain valid evidence, but they are not a second live implementation lane.
4. Rust realization must be treated as an ab initio implementation against OxCalc specs, replay artifacts, and executable comparison surfaces, not as a mechanical translation of older non-Rust shapes or idioms.

## 8. Workspace Snapshot-Layer Doctrine
OxCalc's target workspace model separates durable edited truth from derived and runtime state.

The durable workspace revision is the tuple of:
1. `StructureSnapshot` for topology, node identity, ordering, symbols, structural paths, table shape, and anchors,
2. `NodeInputSnapshot` for per-node calculation input such as empty, literal, formula text, and future host-owned input variants,
3. `NamespaceSnapshot` for host namespace, registry, capability, workspace availability, alias, and caller-context facts that affect binding or prepared identity.

Derived or retained layers include:
1. `FormulaBindingSnapshot` for typed OxFml parse/bind/prepared facts,
2. `DependencyShapeSnapshot` for dependency facts derived from structure, input, namespace, and formula-binding facts,
3. `PublicationSnapshot` for accepted observable values, diagnostics, and dependency-effect publication identity,
4. `RuntimeOverlaySet` for CTRO/dynamic dependencies, invalidation overlays, and other epoch-scoped runtime effects.

Discardable contextual views may provide parent navigation, paths, sparse readers, host lookups, and evaluator context, but they must be rebuildable and must not become durable truth.

Snapshot identities are version fences, not automatic global dirtiness claims. A
structural edit creates successor `StructureSnapshot` and `WorkspaceRevision`
identities, but derived artifacts, overlays, cached values, and published
values are invalidated or evicted by declared compatibility rules.
Conservative full rebuild remains legal when compatibility cannot be proven,
but the architecture must leave room for local structural identities,
dependency components, publication shards, and subtree hashes to prove that
unaffected regions can be retained safely.

OxCalc must not inspect formula syntax or function names to infer evaluator semantics. OxFml owns parse/bind/evaluator semantics and supplies typed facts; OxCalc consumes those facts for dependency, invalidation, scheduling, publication, replay, and retention decisions.
