# W062 R0 — stash@{0} triage record (calc-5kqg.1)

Date: 2026-07-04. Stash: `On main: workflow leftovers: C3 walk-up + BO
node-as-function + A1/A2, unreviewed (wf_03bfbbc8)` — 1,531 lines over
`consumer.rs` (+866), `treecalc.rs` (+557), `formula.rs` (+78),
`tree_reference_resolution.rs` (+76), `.beads/issues.jsonl`. Based on
`a174bdda`, which is **not an ancestor of current main** — CTRO
(`e069136e`), the OVL series, and the name-resolution-index refactor all
landed after it. `git merge-tree` confirms conflicts in all four code
files. Verdict: **DROP all code; harvest the items below.** The stash is
dropped once this record is committed.

## Group verdicts

- **BO node-as-function (~800 lines): DROPPED as superseded.** Main
  reimplemented node-as-function via `RichValue::Callable` carried on
  `working_calc_values` + `callable_binding_from_calc_value` /
  `runtime_binding_for_node` (`treecalc.rs:5361`, `:5400`), not the
  stash's wave-threaded `node_callables` map. The stash's consumer.rs
  targets (`direct_name_carriers_from_oxfml_probe`,
  `typed_exclusion_diagnostics`, `oxfml_context_bind_probe`) no longer
  exist on main; the negative test at `consumer.rs:15695` asserts node
  function calls are callable candidates now.
- **Walk-up precedence A1/A2/C3 (doc + corpus, no logic): DROPPED as
  code, HARVESTED as spec** (below). The corpus asserts on an
  `ambiguous_host_name` diagnostic string that no longer matches main —
  `Ambiguous` is now handled in `tree_reference_system.rs:302` as
  `ReferenceAtomBindResult::Rejected`. A fresh corpus is authored in R3
  against the real rejection path.
- **Bead edits: DROPPED.** `calc-uanv` already exists on main; the
  `.69`/`.73` edits are stale churn.

## Harvest 1 — tree-profile name-precedence spec (input to D2)

From the stash's `tree_reference_resolution.rs` module doc (diff lines
1085–1152), anchored to DnaTreeCalc `CORE_MODEL_SPEC §3.2/§3.4/§3.7/
§3.9/§10.3`. The D2 design adopts this as the tree-profile precedence
contract:

1. **Nearest-scope-wins**: a symbol resolves by walking up from the
   referencing node's scope; the first scope containing the name wins.
2. **Ancestor-by-own-name has no priority**: an ancestor whose own name
   matches the symbol does not outrank a name defined in a nearer scope.
3. **Within-scope collision ⇒ Ambiguous** (typed rejection, never a
   silent pick).
4. **Self-reference resolves to self**, and then participates in normal
   cycle handling.

Note: collision cases constructed via case-only sibling names
(`Dup`/`dup`) depend on today's case-sensitive structural uniqueness and
break when `calc-uanv` (case-insensitive sibling uniqueness) lands —
the R3 corpus must construct ambiguity another way.

## Harvest 2 — callable-model design inputs (input to D2/D3)

Two ideas from the dropped BO machinery that main's simpler
value-channel path does not obviously cover:

1. **First-class caller→captured invalidation edges** — editing a node
   captured by a callable must re-evaluate callers through the published
   dependency graph.
2. **Transitive-capture closure** for callable-calls-callable chains.

**Open verification action for D2/D3**: check whether main's
`callable_binding_from_calc_value` path actually invalidates callers
when a captured node is edited — the stash existed partly to guarantee
this, and main's path may not. If it does not, that is a live defect the
D3 design must close (and a candidate fail-until-fixed test now).

## Side finding — stale W056 inventory note on main (own tidy bead)

`formula.rs:2340` still categorizes `node_as_function_lambda` as
`DirectContextTypedPending` ("…typed pending/exclusion until the product
lane is implemented") while the callable lane IS implemented
(`consumer.rs:15695`). Tracked as its own bead; the stash's formula.rs
edit tried to fix exactly this, so the correction is warranted even
though the stash version is dropped.
