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
6. Rust-first realization of OxCalc-owned executable artifacts for the core engine and `TraceCalc` tooling/runtime.

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

## 6. Definition of Done (Lane)
A coordinator policy/spec change is done only when:
1. spec text and realization notes are updated,
2. required pack expectations are updated,
3. deterministic replay evidence exists,
4. FEC/F3E cross-repo impact is recorded.

## 7. Implementation Direction
1. OxCalc-owned executable realization is Rust-first from this point onward.
2. The active repo implementation is the Rust workspace under `src/`.
3. Historical baseline runs and checked-in artifacts remain valid evidence, but they are not a second live implementation lane.
4. Rust realization must be treated as an ab initio implementation against OxCalc specs, replay artifacts, and executable comparison surfaces, not as a mechanical translation of older non-Rust shapes or idioms.
