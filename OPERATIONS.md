# OPERATIONS.md — OxCalc Operations

## 1. Purpose
Define day-to-day execution rules for core multi-node recalc and coordinator policy.

## 2. Operating Principles
1. Semantic stability is invariant under strategy changes.
2. Coordinator is single publisher at baseline.
3. Overlay lifecycle must be deterministic and epoch-safe.
4. Visibility-priority optimization must preserve stabilized semantic equivalence.

## 3. Staged Realization
1. Stage 1:
   - sequential deterministic topo/SCC baseline,
   - atomic commit bundle handling,
   - conservative fallback allowed.
2. Stage 2:
   - partitioned parallel evaluators,
   - deterministic contention replay required,
   - snapshot/token fence hardening required.
3. Stage 3:
   - advanced policy lanes (dynamic-topo/SAC-like),
   - bounded experimental-lane policy,
   - parity evidence required before promotion.

## 4. Required Packs (baseline)
1. `PACK.visibility.policy_equivalence`
2. `PACK.visibility.starvation_bound`
3. `PACK.dag.dynamic_dependency_bind_semantics`
4. `PACK.dag.cycle_iterative_semantics`
5. `PACK.dag.external_stream_ordering`
6. `PACK.overlay.fallback_economics`
7. `PACK.concurrent.epochs` (including reject-fence and overlay-GC safety lanes)

## 5. Cross-Repo Handoff Rule
When OxCalc needs FEC/F3E protocol updates:
1. issue a handoff packet to OxFml,
2. include exact clause changes,
3. include evidence/replay artifacts,
4. include migration and fallback impact.

## 6. Promotion Gate
No scheduler/invalidation policy promotion without:
1. deterministic replay for affected classes,
2. updated pack expectations and matrix links,
3. explicit semantic-equivalence statement.
