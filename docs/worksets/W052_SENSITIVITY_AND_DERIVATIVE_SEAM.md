# W052 Sensitivity And Derivative Seam

Status: `pre_planning`

Parent predecessor: `W050` (Lane G capability vocabulary admission)

Parent epic: TBD (allocated when W052 is activated)

## 1. Purpose

W052 layers the `Differentiable(parameter_set)` capability onto numeric rich values, enabling sensitivity / derivative queries over the call-site graph. With this capability in place, Goal Seek, Solver, and what-if analysis become capability queries against a graph of differentiable rich values rather than bolt-on iteration loops — and they compose with replay and the single-publisher coordinator instead of tearing into the recalc engine through opaque side paths.

W052 implements:

1. The `Differentiable(parameter_set)` capability — `partial(parameter) -> RichValue` — as an extension of the rich-value capability vocabulary W050 admits.
2. A per-kernel derivative-metadata profile in OxFunc: `Analytical(kernel)` (closed-form derivative), `Finite(epsilon)` (finite-difference fallback), `Discontinuous` (no meaningful derivative).
3. The forward sensitivity walk: given a perturbation at one input, propagate partial derivatives through the call-site graph along the kernels that expose a derivative profile.

W052 is in a deliberate `pre_planning` state. Scope, beads, exit gates, and evidence policy are decided after W050 lands the capability vocabulary. This document is pre-planning background only; do not infer a bead path or commit to artefacts from it.

## 2. Pre-Planning Background

### 2.1 The W050 commitment that makes W052 additive

W050 admits `RichValueHole(required_capability_set)` from day one with `Differentiable` named as a reserved extension capability. Because capability-vocabulary admission is additive — a new capability is a new vocabulary entry, not a change to existing hole-type identity — W052 introduces `Differentiable` without retrofitting any W050-era artefact. This is the explicit reason W050 admitted the capability-set hole shape before any kernel consumed it.

### 2.2 Goal Seek / Solver / what-if as capability queries

The design intent is that Goal Seek becomes "iterate by setting input X such that a sensitivity-targeted output matches a goal", Solver becomes a multi-variate sensitivity walk, and what-if becomes a sensitivity query over a pinned snapshot. All three compose with replay because the sensitivity walk is a deterministic function of the call-site graph and the recorded hole bindings; all three respect the single-publisher coordinator because they query rather than mutate.

### 2.3 Ownership split

W052 is primarily OxFunc-owned: per-kernel derivative metadata is OxFunc's responsibility, kernel by kernel. OxFml threads the `Differentiable` capability through semantic plan. OxCalc administers the sensitivity-query surface and the forward walk over its call-site graph. A cross-repo handoff packet to OxFunc is expected.

### 2.4 The decision W050 took on W052's behalf

The single architecturally significant W052-related decision was taken inside W050: whether OxFunc kernels carry a derivative-metadata column at all. W050 §10.12 and §11.2 record that the capability vocabulary is reserved-and-extensible. W052 does not reopen that — it populates the column.

## 3. Relationship To W050 And W049

W052 depends on W050 Lane G (the capability vocabulary). It is sequenced after W049 in the §5.1 go-forward order so that the sensitivity lane is built on a formalised core rather than alongside an unformalised one — but W052 has no hard dependency on W049; the sequencing is a quality preference, not a blocker.

## 4. Open Scoping Questions

Deferred until W050 lands and W052 is planned in detail:

- Which kernels get `Analytical` derivatives first, and which start as `Finite(epsilon)`?
- How is `Discontinuous` surfaced to a sensitivity query — as an error, a zero, or a typed discontinuity marker?
- Does W052 ship Goal Seek / Solver / what-if applications, or only the `Differentiable` capability and the forward walk, leaving the applications to a successor?
- How does the sensitivity walk interact with cycles (SCCs) and dynamic dependencies?

## 5. Status Surface

- execution_state: `pre_planning`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- prerequisites: W050 Lane G (capability vocabulary) landed
- bead_path: not yet specified — W052 epic id and bead structure allocated when W052 is activated
- exit_gate: not yet specified
- evidence_policy: not yet specified
- upstream_dependencies: `OxFunc` (primary owner of per-kernel derivative metadata), `OxFml` (threads `Differentiable` capability through semantic plan)
