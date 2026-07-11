# O-2 design notes — one OxFml front end per evaluation

Status: pinned design notes (2026-07-11), pre-implementation. Companion to
`CALC_OPTIMIZATION_GROUNDING_AND_ROADMAP.md` §6 O-2. Execution bead: see the O-2
bead referencing this file. Written after code-verified scoping; file:line anchors
are July 2026 and will drift — symbol names are the durable pointers.

## 1. The three-pass anatomy (verified)

Per general-path formula evaluation, OxCalc + OxFml currently run the front end
(lex → parse → red → bind → semantic plan) three times, and the M-3 counters now
measure all of it (`oxfml_execute_calls`, `oxfml_structural_bind_calls` — 2× per
cell on mark-all, `oxfml_metadata_bind_calls`, `provider_builds`):

1. **Inside `RuntimeEnvironment::execute` → `execute_with_host`**
   (OxFml consumer/runtime/mod.rs:555): `compile_runtime_prepare_request` (:3627)
   runs the full front end ONLY to produce the prepared-formula identity, the
   locale capability gate, registry capability denials, and formal-reference
   facts; then `host.recalc_with_library_context_view_and_reference_bind_profile`
   (host/mod.rs:425) runs the entire front end AGAIN inside the fresh
   `SingleFormulaHost` (its `cached_artifacts` start empty on the grid path).
2. **OxCalc structural-dependency install**
   (`grid_structural_dependencies_for_formula`, runtime_trace.rs:333) — an
   independent parse+bind per cell per visit, twice per cell on mark-all
   (pre-worklist install + per-evaluation re-install).
3. **OxCalc metadata classification re-bind**
   (`bind_grid_formula_for_transform`, optimized_sheet.rs) after evaluation.

## 2. Load-bearing discoveries (do not rediscover these)

- **The two OxFml passes are NOT reuse-compatible today.**
  (a) Bind contexts differ: the env side (`bind_context_for_source`,
  consumer/runtime/mod.rs:614) includes `host_name_bind_records`,
  `function_surfaces`, `name_caller_context_dependencies`, and formal-input
  defined names; the host side (host/mod.rs:482) rebuilds a NARROWER context
  from copied host fields with a `..BindContext::default()` tail. Different
  context ⇒ different `bind_context_fingerprint` ⇒ `bind_formula_incremental`
  refuses reuse. (b) Semantic-plan catalog identities differ:
  `"oxfunc:runtime-facade-session"` (pass 1) vs `"oxfunc:host"` (pass 2),
  so plan reuse keys can never match either.
- **Correctness question to investigate (flag, not assertion):** pass 2's bound
  formula — the one actually evaluated — binds WITHOUT `function_surfaces` and
  `host_name_bind_records`, while the prepared identity is computed from pass 1's
  richer bind. For plain grid formulas the contexts coincide in effect; for
  host-name/registry-view-bearing formulas the identity and the evaluated
  artifact may derive from differently-bound trees. Tests are green, so any
  divergence is currently unobserved — but the unification (below) should be
  treated as a potential behavior alignment, not a pure refactor.
- **Formal-input bindings force pass 1 to stay pre-recalc in that lane:**
  `apply_formal_input_bindings_to_host` (consumer/runtime/mod.rs:890) validates
  binding handles against pass-1 `formal_references` AND installs defined names
  that participate in pass-2 binding. Identity-after-recalc is impossible there
  without restructuring formal inputs.
- **Key-churn blast radius:** unifying bind contexts or catalog identities
  changes `bind_hash` → `semantic_plan_key` → `RuntimePreparedFormulaIdentity`.
  Prepared identities are retention keys (`PreparedFormulaRetention`,
  per-edge value cache basis) and appear in replay artifacts. The unification
  slice must plan artifact re-baselining and retention-key migration in the
  same change, with the tree-lane replay fixtures regenerated deliberately.
- **Seeding pass-1 artifacts into the host cache without unification wins only
  parse+red** (text-equality reuse), at the cost of cloning the green tree
  (pass 1 currently lets `bind_formula` consume it). Bind — the dominant cost,
  with its `format!("{:?}")` whole-tree hash — would still re-run. Judged not
  worth landing as a standalone slice.

## 3. Sliced plan (order matters)

- **O-2.i — unify the bind context.** `SingleFormulaHost` recalc accepts the
  caller-supplied `BindContext` (the env's full one) instead of rebuilding a
  narrower one; kill the `..default()` tail. One context, one fingerprint.
  Includes the catalog-identity decision (one identity string, chosen once).
  This is the key-churn slice: re-baseline replay artifacts + retention keys
  here and nowhere else. Investigate the correctness flag above as part of it.
- **O-2.ii — host prepare/evaluate split.** Split host recalc into
  `prepare_artifacts` (front end, populates `CachedHostArtifacts`, returns
  reuse report) and `evaluate_prepared` (context build + evaluation). This is
  the seam O-1's template store will feed later.
- **O-2.iii — facade fast lane.** In `execute_with_host`, when there are no
  formal-input bindings: host.prepare first, then compute prepared identity +
  registry denials FROM the host artifacts (now context-identical by O-2.i),
  enforce the locale gate between prepare and evaluate, then evaluate_prepared.
  ONE front end per execute on the grid path. The formal-input lane keeps
  pass 1 (documented, tree-lane-only cost).
- **O-2.iv — OxCalc-side bound-facts reuse** for structural install and
  metadata classification (kill passes 2 and 3): cache bound facts per
  (template, context-epoch) in `GridDerivedState` beside the O-3 plan cache,
  instantiated per cell by relative-offset transform. Pairs naturally with O-4
  (region-granular installs); needs the OxFml transform seam, not new parsing.
  Also kill the mark-all double-install (pre-worklist + per-eval re-install)
  by making the pre-install pass produce what the per-eval install needs.

Gates for every slice: M-3 counter floors re-pinned with explanations
(expected end state on the 2-formula fixture: mark-all 2/2/0/2 after O-2.iii +
metadata fold-in, then per-template counts after O-2.iv); differential clean;
tree-lane replay artifacts regenerated only in O-2.i.
