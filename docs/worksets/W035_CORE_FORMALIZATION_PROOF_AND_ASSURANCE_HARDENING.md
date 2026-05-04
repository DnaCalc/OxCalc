# W035: Core Formalization Proof And Assurance Hardening

## Purpose

W035 continues the formalization path after W034.

W034 widened the evidence floor: TraceCalc oracle coverage, TreeCalc/CoreEngine conformance, Lean proof-family slices, TLA interleaving models, pack/capability gate binding, and continuous-scale semantic binding. W035 turns the remaining W034 residuals into the next execution tranche for stronger proof, broader oracle coverage, implementation hardening, and continuous assurance.

W035 is not a promotion workset by default. It may produce a promotion candidate only if its evidence satisfies the relevant gate. Otherwise it must record exact blockers and carry them forward.

## Position And Dependencies

- depends_on: `W034`
- parent epic: `calc-tkq`
- predecessor epic: `calc-e77`
- upstream dependencies: `OxFml`
- canonical predecessor packet: `docs/spec/core-engine/w034-formalization/W034_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md`

## Scope

### In Scope

1. Convert W034 residuals into W035 proof obligations, implementation work, handoff/watch rows, or explicit deferrals.
2. Expand TraceCalc oracle coverage into generated matrices for stale fences, dependency updates, overlay retention, and `LET`/`LAMBDA` callable surfaces.
3. Harden implementation conformance by addressing or reclassifying W034 declared gaps.
4. Deepen Lean proof artifacts by distinguishing discharged local theorems, explicit axioms, and imported seam assumptions.
5. Deepen TLA+ exploration beyond routine smoke checks and state scheduler/Stage 2 equivalence preconditions.
6. Define a continuous assurance and cross-engine differential gate that is stronger than single-run scale evidence.
7. Reassess pack capability and Stage 2 readiness with machine-readable no-promotion or promotion decisions.
8. Preserve the OxFml W073 typed conditional-formatting direction as a watch/input-contract guardrail until a concrete OxCalc request path exercises it.

### Out Of Scope

1. General OxFunc semantic kernels.
2. Direct edits to OxFml from this repo.
3. Stage 2 policy promotion without the required proof/model/replay/conformance gate.
4. Pack-grade replay or continuous-scale promotion based on bounded smoke checks, single-run timing, or proxy conformance signals.
5. UI, host, or file-adapter work unless directly required by a W035 proof/conformance artifact.

## Gate Model

### Entry Gate

1. `calc-e77` W034 parent epic has closed.
2. W034 closure audit packet exists.
3. W034 successor beads exist in `.beads/`.

### Exit Gate

1. Every W035 residual is mapped to evidence, implementation work, handoff/watch rows, or explicit deferral.
2. TraceCalc oracle expansion emits deterministic artifacts.
3. Implementation conformance hardening either reduces W034 declared gaps or records authoritative deferrals/new beads.
4. Lean and TLA work distinguishes bounded evidence from full verification.
5. Continuous assurance criteria are stronger than single-run scale evidence.
6. Pack/Stage 2 decisions state exact evidence and no-promotion blockers or promotion rationale.
7. Closure audit includes OPERATIONS Section 7 checklist, Section 9 self-audit, semantic-equivalence statement, and three-axis report.

## Bead Rollout

Parent:

1. `calc-tkq` - W035 core formalization proof and assurance hardening.

Child path:

1. `calc-tkq.1` - W035 residual proof obligation and spec evolution ledger.
2. `calc-tkq.2` - W035 TraceCalc oracle matrix expansion.
3. `calc-tkq.3` - W035 implementation conformance hardening.
4. `calc-tkq.4` - W035 Lean assumption discharge and seam proof map.
5. `calc-tkq.5` - W035 TLA non-routine exploration and scheduler equivalence preconditions.
6. `calc-tkq.6` - W035 continuous assurance and cross-engine differential gate.
7. `calc-tkq.7` - W035 pack capability and Stage 2 promotion readiness reassessment.
8. `calc-tkq.8` - W035 closure audit and next-tranche packetization.

The first W035 path is sequential to keep the successor tranche auditable. Later W035 work may split only after the residual ledger identifies disjoint evidence scopes.

## Initial Guardrails

1. TraceCalc remains the correctness oracle only for covered reference behavior.
2. TreeCalc/CoreEngine comparison rows must not count declared gaps as matches.
3. Lean/TLA artifacts must state which obligations are proved locally, assumed, bounded by model size, or external to OxCalc.
4. Continuous assurance requires recurring evidence and cross-engine differential criteria; single-run scale timing is not correctness proof.
5. OxFml-owned semantic or FEC/F3E changes require handoff packets rather than direct sibling-repo edits.

## Current Status

- execution_state: `planned_successor`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - all W035 child beads remain open
  - full Lean/TLA verification remains open
  - full TraceCalc oracle coverage remains open
  - optimized/core-engine verification remains open beyond W034 conformance
  - pack-grade replay, continuous scale assurance, and Stage 2 policy remain unpromoted
