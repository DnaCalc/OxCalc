# W053 Staged Concurrency Stage 2

Status: `pre_planning`

Parent predecessors: `W050` (Stage 1 sequential coordinator on the new session model) and `W049` (formalised Stage-1 baseline)

Parent epic: TBD (allocated when W053 is activated)

## 1. Purpose

W053 lands Stage 2 of the Foundation staged-realization contract: partitioned parallel evaluators behind the same single-publisher coordinator authority. The target is wall-clock speedup on multi-core hardware without losing the Stage 1 single-publisher correctness invariant.

The W050 §10 design baseline is deliberately Stage-2-shaped: independent acyclic nodes carry no ordering constraint beyond the dependency graph, so a partitioned scheduler can evaluate them concurrently while the single Coordinator commit authority stays intact. W053 realises that latent shape.

W053 implements:

1. Partitioned parallel evaluators over the dependency graph, with the partition boundary respecting SCC isolation and the topological frontier.
2. Speculative evaluation as the conflict-resolution discipline: an evaluator may invoke a `PreparedCallable` with provisional reference bindings; the result is held as a speculative candidate tagged with its input-binding fingerprint; at Coordinator commit time the fingerprint is checked against actual published values, and the candidate is promoted on match or discarded and re-invoked on mismatch.
3. The semantic-equivalence demonstration: observable results under partitioned Stage 2 must be invariant against the formalised Stage-1 baseline (Foundation semantic-equivalence-under-strategy-change doctrine; `AGENTS.md` Rule 8).

W053 is in a deliberate `pre_planning` state. Scope, beads, exit gates, and evidence policy are decided after W050 lands the Stage 1 sequential coordinator and W049 produces the formalised baseline. This document is pre-planning background only; do not infer a bead path or commit to artefacts from it.

## 2. Pre-Planning Background

### 2.1 Why W049 is a hard dependency, not just a sequence preference

Foundation's staged-realization contract requires Stage 2 to demonstrate *semantic equivalence under strategy change*. W049 produces the formalised Stage-1 baseline that W053 proves equivalence *against*. Without W049 first, W053 has no rigorous reference to compare partitioned and speculative behaviour to — it could only assert equivalence, not demonstrate it.

### 2.2 The single-publisher invariant is preserved, not relaxed

Stage 2 does not introduce multiple publishers. It introduces multiple *evaluators* behind one Coordinator. Conflict resolution stays at one well-defined choke point — the commit-time fingerprint check. The Stage 1 invariants (atomic publication, reject-is-no-publish, observer-stable state) are unchanged; what changes is only how many evaluators produce candidates in flight.

### 2.3 W053 revisits the W054 retention model

W054 specifies bounded-memory and pinned-epoch GC for the Stage-1 sequential engine. Speculative evaluation introduces a new retention class — provisional speculative candidates held pending fingerprint check — that W054's sequential model does not cover. W053 revisits and extends the W054 retention model for partitioned and speculative evaluators.

### 2.4 Foundation gate dependency

W053 also depends on the Foundation Wave B FEC/F3E concurrency-hardening gates. Stage 2 contention remains unpromoted unless deterministic parity / equivalence evidence gates are satisfied.

## 3. Relationship To W050, W049, W054

- W050: provides the Stage-1 sequential coordinator, the session model, the dependency graph, and the prepared-callable cache that W053 partitions over.
- W049: provides the formalised Stage-1 baseline that W053 proves semantic equivalence against.
- W054: provides the Stage-1 bounded-memory retention model that W053 extends for speculative candidates.

## 4. Open Scoping Questions

Deferred until W050, W049, and W054 land and W053 is planned in detail:

- What is the partition strategy — static graph partition, work-stealing, or frontier-driven?
- How aggressive is speculation — speculate on any not-yet-final input, or only on inputs likely to be stable?
- What is the fingerprint granularity for the commit-time check?
- How is the semantic-equivalence demonstration structured — differential replay against the W049 baseline, a TLA model of the partitioned scheduler, or both?
- Does W053 target Stage 3 (advanced incremental lanes) at all, or strictly Stage 2?

## 5. Status Surface

- execution_state: `pre_planning`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- prerequisites: W050 (Stage 1 sequential coordinator on the new session model), W049 (formalised Stage-1 baseline), Foundation Wave B FEC/F3E concurrency-hardening gates
- bead_path: not yet specified — W053 epic id and bead structure allocated when W053 is activated
- exit_gate: not yet specified — Stage 2 contention remains unpromoted unless deterministic parity / equivalence evidence gates are satisfied
- evidence_policy: not yet specified
- upstream_dependencies: Foundation Wave B FEC/F3E concurrency-hardening gates; to be re-evaluated when the W053 plan is finalised
