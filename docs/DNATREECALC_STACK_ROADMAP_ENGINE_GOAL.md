# DNA TreeCalc Stack Roadmap Engine Goal

Status: `active_orientation`

This note connects the DNA TreeCalc stack-requirements roadmap to OxCalc's current engine worksets.
It is not a second execution board. `docs/WORKSET_REGISTER.md` owns OxCalc workset order, `.beads/`
owns live execution state, and `../DnaTreeCalc/docs/ux/stack-requirements/ROADMAP.md` owns the
consumer-side roadmap.

## Goal Statement

Advance the DNA TreeCalc stack roadmap by making each skin-facing capability read or command the
engine truth at its owning layer.

For OxCalc, that means turning the serious roadmap asks into engine capabilities in dependency order:
first expose typed state the engine already computes, then strengthen transaction and publication
semantics, then build the retained revision and candidate-overlay substrates that make undo, history,
and what-if views truthful. DnaTreeCalc skins should become richer because they can read typed
published facts or send closed typed intents; they must not become richer by parsing formulas,
recomputing values, fabricating dependency facts, synthesizing transaction ids, or replaying inverse
edits outside the engine.

Each iteration should start with the earliest unmet roadmap requirement, verify the readiness claim
against live OxCalc/OxFml/DnaTreeCalc code, classify it as `expose`, `extend`, or `new substrate`,
and then do the smallest useful ownership-correct tranche. If OxCalc owns the truth, implement or
specify it here; if OxFml owns formula text, parse, bind, rewrite, or render behavior, file or consume
the OxFml handoff; if the host owns projection and dispatch, leave OxCalc with a clear contract the
host can consume without semantic reconstruction.

The running success test is simple: a future FLOW or ATLAS skin can show more structure, explain
more calculation behavior, or issue a stronger authoring command while OxCalc remains the owner of
multi-node scheduling, dependency facts, invalidation, publication, transaction identity, overlays,
epochs, and revisions.

## Iteration Loop

Use this loop for each stack-roadmap tranche that touches OxCalc:

1. Name the exact DnaTreeCalc roadmap wave and requirement.
2. Verify the current readiness tag against code, not just the requirement document.
3. Map the requirement to the owning layer: OxFml, OxCalc, DnaTreeCalc host, or skin.
4. If OxCalc owns it, map it to the current OxCalc workset or create the next bead under the correct
   workset.
5. Implement or spike the smallest tranche that gives a downstream host a typed fact, typed edit, or
   typed blocker.
6. Update the OxCalc contract/spec surface consumed by hosts.
7. Require downstream evidence when the claim is host-visible: a DnaTreeCalc programmable Skin IR
   test, direct `OxCalcTree` consumer test, retained replay artifact, or handoff acknowledgement.
8. Report product scope, evidence, exclusions, and formal status separately.

## Roadmap-To-OxCalc Map

| DnaTreeCalc roadmap area | OxCalc responsibility | Current OxCalc home | Status |
|---|---|---|---|
| W0/W1 typed published facts | Node identity, dependency kinds, invalidation reasons, run/calc state, value epochs, traces, runtime effects, and reference-resolution facts | W050/W051/W056/W057 plus public `OxCalcTree` host contract | Mostly exposed for current Skin IR use; keep widening only where current code evidence exists. |
| W2 structural authoring | Atomic edit transactions, legality/impact planning, invalidation preview, and real transaction ids | W050 session model, W057 snapshot model, transaction-scope slices | Current DnaTreeCalc structural surface has real transaction ids; broader scoped verbs continue under W3 command expansion. |
| W3 formula/reference/content authoring | Rebind after typed edits, collection membership mutation substrate, duplicate/subtree dependency preservation, and transaction-backed publication | W056 for reference/table facts, W059 for authored input authority, W060 for runtime reference representation | Mixed: read facts exist for several selectors; set-membership write and formula rebind remain engine/API work, not host workarounds. |
| W4a revision graph | Retained parent-linked revision DAG and cursor; undo/redo by revision navigation, never inverse replay | `OxCalcTreeContext` retained revision graph plus in-memory `navigate_workspace_revision`; bounded oldest-first retention policy on `OxCalcTreeContextOptions` | Scoped engine substrate implemented: in-memory parent-linked revisions, transaction predecessor/successor ids, retained-state navigation, transaction invalidation summaries, and bounded oldest-first eviction are implemented. Persistence policy is explicit: workspace snapshots persist the active revision/layers, not the navigable history DAG. |
| W4b candidate overlays | N addressable, layerable, non-publishing candidate contexts with publish/discard semantics | `CORE_ENGINE_CANDIDATE_OVERLAY_HANDLE_SPIKE.md`, CTRO/overlay lineage from W047/W050, first slice `calc-etez`, continuation bead `calc-4ipg` | First substrate, commit, DnaTreeCalc projection, parented layering, conservative same-node conflict policy, candidate-private revision-history, candidate basis-retention pin, candidate structural projection/read, structural candidate rename/move/delete intent, constant/empty candidate add-node, formula-literal candidate add-node dry-bind, candidate-only run projection, inherited table-column formula candidate add-node, candidate private-edit invalidation-summary, first speculation-budget/GC, and host-retention pin slices implemented: OxCalc can open an opaque candidate handle on a retained revision, apply a private node edit, evaluate private candidate values, discard without publishing, commit when the live basis revision still matches, open a child over a parent candidate's private state, refresh parented candidates from parent-private edits made after the child was opened, reject stale rebase with a typed `CandidateRebaseConflict` report when live and candidate private edits overlap on the same stable node, allow non-overlapping stale candidates to rebase, expose candidate-private revision graph entries with real transaction ids, keep host-visible candidate basis revisions retained under bounded revision retention while a candidate is open, expose candidate-private node views after private structural edits without rewriting the live workspace view, dry-bind prospective candidate-added formula nodes against candidate-private structure, attach planning-derived transaction summaries to supported candidate apply-only private edits, compute typed candidate pressure, reap unprotected candidates to a requested budget, and protect explicitly host-pinned candidates from budget reaping. DnaTreeCalc projects candidate values, parent handles, private revision history, candidate-private node structure, candidate calculation run facts for candidate-only nodes, retention pin counts, reason-specific pressure counts, and closed rename/move/delete/add-by-NodeKey/reap/pin/unpin candidate intents separately from published node state; it also reads inherited table-column formula text from candidate-private table metadata and dry-binds it before candidate add. Richer structural merge algebra, candidate add-node template initial content, and scenario/what-if UX remain open. |
| W5 delta/platform support | Versioned projection facts and engine-side invalidated-node basis the host can turn into deltas | W054 retention/eviction and public host contract surfaces | Full snapshots exist downstream; delta-only/resync discipline remains future contract work. |
| W6 templates/tables/import/export/frontier | Table lifecycle facts, structural table operations, external references, replay-visible import/export equivalence, sensitivity/goal-seek substrates | W056 table/reference lowering, W052 sensitivity, W054 retention, W060 reference system | Table/reference slices are active; templates and scenario/frontier features wait on prior substrates. |

## Immediate Engine Cursor

The consumer roadmap cursor is now W4b after scoped W4a closure. OxCalc should
not drift into small UI-improvement mode. The next OxCalc-relevant stack moves
are:

1. Keep W3 handoffs explicit where they remain prerequisites or parallel engine
   truth, but do not let them block the W4b candidate substrate:
   - `set-membership-write`: add a transaction-backed reference-collection membership/order edit
     substrate, or keep it explicitly blocked. First substrate slice is present: OxCalc validates
     owner/source-reference handles through current dependency descriptors and returns typed
     unknown/non-editable collection errors for derived collections. Positive authored
     membership/order storage remains open.
   - formula/subtree rebind after paste, fill, duplicate, or reference insertion: consume OxFml-owned
     rewrite APIs and perform OxCalc-owned rebind/invalidation/publication.
   - authored input and literal value authority: continue W059 so OxCalc stops owning local
     authored-text interpretation.
2. Continue W4b candidate-overlay work under bead `calc-4ipg` after the first
   non-publishing handle slice:
   - richer structural merge algebra and add-node parent/order conflict classification,
   - candidate add-node template initial content once template substrate exists,
   - scenario/what-if UX on the proven candidate substrate.
4. Keep W054 bounded-memory work aligned with downstream needs:
   - deterministic retention classes,
   - pin rules for host-visible snapshots/revisions/candidates,
   - replay-visible eviction and conservative fallback counters.

## Per-Tranche Checklist

- [ ] `Roadmap item`: exact DnaTreeCalc wave and requirement named.
- [ ] `Owning truth`: OxCalc owns the engine fact/mutation being changed.
- [ ] `Readiness`: live code confirms `expose`, `extend`, or `new substrate`.
- [ ] `Workset/bead`: mapped to the current OxCalc workset and bead state.
- [ ] `Contract`: `OxCalcTree` host contract or core-engine spec updated when host-visible.
- [ ] `Evidence`: OxCalc tests, replay/model evidence, and downstream Skin IR or host exercise where
      product-visible.
- [ ] `No fabrication`: host/skin does not parse formulas, compute values, invent dependency facts,
      fake transaction ids, or implement undo by inverse replay.
- [ ] `Status`: product scope, evidence, still-open gaps, and formal status reported separately.
