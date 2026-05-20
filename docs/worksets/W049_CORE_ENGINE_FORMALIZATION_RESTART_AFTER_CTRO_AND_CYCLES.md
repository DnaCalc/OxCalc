# W049 Core Engine Formalization Restart After CTRO And Cycles

Status: `queued_successor`

Parent predecessors:
- `W048` circular dependency calculation processing
- `W050` formula-authority rework
- `W051` sparse range readers
- `W054` bounded-memory and pinned-epoch GC

Parent epic: allocate when W049 starts.

## 1. Purpose

W049 restarts formalization against the current Rust engine instead of
continuing the old broad formalization chain.

The workset exists to produce a small set of useful formal and checker
artifacts that constrain real engine behavior. It should not produce decorative
Lean/TLA/checker files that pass without exercising implementation facts.

W049 is sequenced after W050, W051, and W054 so the formal target is the
settled Stage-1 engine:

1. session-shaped OxFml runtime seam,
2. prepared formula package and plan-template identity,
3. formal input/reference transport,
4. sparse-reader behavior where it is part of ordinary worksheet calculation,
5. deterministic retention and pinned-epoch memory rules,
6. W048 cycle behavior as predecessor evidence.

## 2. Current Product Status

W049 does not implement worksheet behavior for users. It is an assurance
workset.

Current status:

1. The W046 formalization chain is retired to provenance.
2. W048 provides the current cycle evidence floor, scoped to one observed Excel
   host/version.
3. W050 provides the current formula-authority seam.
4. W049 has not started execution and has no product or proof claim yet.

## 3. What W049 Must Fix

W049 inherits the W046 fresh-eyes lessons. The important rules are:

1. No record-projection proofs. A theorem must constrain behavior, not only
   project a field out of a record.
2. No tiny smoke models presented as useful model checking.
3. No checker fallback to empty inputs. Missing evidence must fail loudly.
4. No evidence roots without a consumer. Every cited test run must be bound to
   a bead, checker, proof, or closure decision.
5. No loose terminology. Define candidate, accepted candidate, commit,
   publication, reject, refinement, binding, bridge, and kernel match before
   using them as proof terms.
6. Keep titles honest. Say "checked at this bounded scope" when that is all
   the evidence supports.

## 4. Inherited W046 Obligations

W046 routed four successor obligations under the wrong successor label. W049
owns their disposition. When W049 starts, each item must be taken on, deferred,
or dropped with a recorded reason.

| Obligation | Required disposition |
| --- | --- |
| Rust Tarjan and topological queue proof | Prove or check the real Rust SCC/order behavior, or state the exact blocker. |
| Proof-carrying trace sidecar enrichment | Exercise real paths, especially rebind-required-and-rejected paths. |
| Dynamic dependency positive publication refinement | Define a real observation/refinement relation over CTRO publication behavior. |
| Pack and operated-service readiness gate | State evidence consequence only; do not promote readiness without direct evidence. |

## 5. Initial Lanes

The first W049 rollout should create these beads:

1. terminology and authority glossary,
2. inherited W046 obligation disposition,
3. evidence-root audit and cleanup,
4. selected proof/checker target list,
5. implementation-bound checker or proof artifacts,
6. cycle and CTRO formalization intake from W048/W047/W050,
7. closure review and successor routing.

## 6. Closure Gate

W049 can close only when:

1. every selected formal/checker target has direct implementation or replay
   evidence,
2. every checker fails on missing or malformed required input,
3. every proof/model title matches what was actually proved or checked,
4. inherited W046 obligations are closed, deferred, or dropped with reasons,
5. unbound evidence roots are bound or removed from the active path,
6. product status, evidence, still-open work, and formal status are reported
   separately.

## 7. Status

Product status: no user-facing feature claim. W049 is queued assurance work.

Evidence: predecessor review findings from W046, W048 cycle evidence, and W050
formula-authority closure.

Still open: all W049 lanes. No W049 epic exists yet.

Formal status: no new W049 proof/model claim yet.
