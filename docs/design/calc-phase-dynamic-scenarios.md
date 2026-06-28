# OxCalc Calculation-Phase Dynamic Behaviour - Design-Review Corpus

Provenance: synthesized from a six-domain calc-phase cataloguing pass (address-dynamic & volatile refs; dynamic arrays & spill; rich objects / linked data; structured tables; names / cross-workspace / external; dependency-order rework) with Excel-behaviour research. This is design-review input for revisiting the CTRO + dynamic-array-spill specification and then examining the existing calc infrastructure against it. No code.

Companion: `ctro-implementation-spec.md` (the CTRO-1/2/3 rebind.rs implementation spec; CTRO-1/2 are committed and green, CTRO-3 paused pending this review).

## 1. Scope frame

### 1.1 What we want

- A calc-phase model that is Excel-FAITHFUL for the three dynamic behaviours: (1) calc-time reference resolution (INDIRECT/OFFSET/dynamic-name/spill-anchor/structured-table/FIELDVALUE targets resolved during calc, not statically), (2) calc-time invalidation (exactly the right cells dirty when a dynamic resolution, spill extent, refresh, or availability changes), and (3) calc-time dependency-order rework (the calc chain / dependency graph mutating mid-calculation).
- A VOLATILE MODEL: a recalc-always root set (NOW/TODAY/RAND/RANDBETWEEN/RANDARRAY/OFFSET/INDIRECT/WEBSERVICE + arg-conditional CELL/INFO) seeded dirty unconditionally each recalc before closure, propagating in topological order, with per-name volatility classification (OFFSET volatile, INDEX not).
- A SMART/INCREMENTAL recalc driver that consumes a dirty closure (edit a literal -> only its dependent cone recalcs), plus a calc-chain that topologically orders the dirty set and handles forward references via reorder or on-demand precedent evaluation.
- A wired CTRO seam: dynamic-request, spill-anchor, structured-table, and workspace-availability claims actually produced and resolved during a real recalc, driving the value engines (not just classified in isolation), with the resolved target carried in identity and keyed by a stable (owner, arg-AST, mode) request key.
- Faithful error semantics across the full matrix (#REF!/#NAME?/#VALUE!/#SPILL!/#FIELD!/#CALC!/#BUSY!/#CONNECT!/#BLOCKED!), including the deleted-NAME->#NAME? vs deleted-target->#REF! split.
- Spill invalidation that seeds extent-delta cells and closes over both spill-fact and plain-range dependents; complete spill blocking (occupancy, merged, beyond-edge, table-body, mutual-collision) with a fixpoint that forces volatile-size #SPILL!.
- Cycle detection + an iterative-calculation engine (MaxIterations/MaxChange), distinct from the spill-repair fixpoint.
- External/non-terminal state handling (refresh, RTD push, WEBSERVICE, availability flips, #BUSY!) as dirtying inputs with re-enqueue across recalcs.
- Scope/namespace/host-version keying beyond cell addresses (worksheet-vs-workbook name scope, 3D sheet-axis spans, workspace availability_version, function-registry snapshot, format-sensitivity).
- A rich-object calc surface (dotted access, FIELDVALUE, #FIELD!, basicValue coercion, rich-aware fingerprinting honoring ExcludeFromCalcComparison, entity spill, refresh reclassification).

### 1.2 What we have

- A complete GridInvalidationRef semantic dependency graph (invalidation.rs) with GridDependency variants Cell/Range/Name/Table/SpillFact/SpillBlocker/AxisVisibility/AxisValue/DynamicRequest; dirty_closure_for_* fns (dynamic_request, spill_fact, spill_epoch_changes, spill_blocker, name, table); close_over_dependents BFS over scalar+compressed-range+axis-value dependents; apply_axis_edit (address-transform only); rename/delete/resize lifecycle ops.
- A CTRO rebind model (rebind.rs, CTRO-1/2 committed): classify_identity_transition (Activated/Reclassified/Released/Preserving), cause_to_reason_table, resolve_cell_dynamic_request_claim, resolve_spill_anchor_claim (anchor-union seeding, documented sharp edge), resolve_structured_table_claim (verbatim classifier round-trip). DynamicRebindErrorEffect enum INCLUDING NameOrValue (exists but unemitted). ResolvedReferenceIdentity carries before/after resolved targets.
- Value engines GridOptimizedSheet/GridCalcRefSheet with mark-all-dirty recalc in fixed BTreeMap address order, a bounded spill-repair fixpoint (converges on spill_facts-map equality, sets spill_repair_converged), warm-no-op fast path, and a spill-fingerprint/epoch ledger (spill_ledger.rs: extent/value/blocked epochs, GridSpillEpochChangeKind).
- OFFSET reference resolution with full in-bounds/resize guards mapping off-grid -> #REF! (reference_engine.rs); spill-anchor (#) resolution against committed spill_extents; implicit-intersection (@) operator (OxFunc op_implicit_intersection.rs).
- A 25-30-variant structured-table lifecycle classifier (structured_table.rs: classify_treecalc_dynamic_table_rebind, dynamic_table_target_fact_kinds, scenario changed-kind/reason/prepared-input sets, validation diagnostics, admission gating including DynamicArraySpillPolicy, per-row calc-column evaluation).
- Cross-workspace availability versioning (formula.rs: TreeCalcCrossWorkspaceAvailabilityStatus/Packet/Registry, WorkspaceUnavailable) and workspace_reverse_edges keyed by target handle + availability_version (dependency.rs).
- RTD/external host modelling (upstream_host.rs: MinimalRtdMode Disabled/CapabilityDenied/NoValueYet/ConnectionFailed/ProviderError/Value; PacketRtdProvider/HostInfoProvider) and InvalidationReasonKind::ExternallyInvalidated (mapped to Needed).
- Reason/kind vocabulary (dependency.rs): InvalidationReasonKind (StructuralRebindRequired/StructuralRecalcOnly/UpstreamPublication/ExternallyInvalidated/DynamicDependency{Activated,Released,Reclassified}/DependencyAdded/Removed), DependencyDescriptorKind (DynamicPotential/ShapeTopology/HostSensitive/CapabilitySensitive/Unresolved), NodeCalcState::CycleBlocked + cycle_members.
- OxFunc value substrate for rich objects (CalcValue core+Option<rich>; RichValue::Object/Callable/Presentation/ErrorMetadata; RichObjectValue.kvps/value_type/fallback; key_flags incl. ExcludeFromCalcComparison; one built producer, IMAGE/_webimage) and the full worksheet-error vocabulary (Field/Busy/GettingData/Spill/Calc/Blocked/Connect/Name/Ref/Value).
- OxFml binding NameKind taxonomy (ReferenceLike/ValueLike/MixedOrDeferred/HelperLocal) and NameRef carrying sheet_id + caller_context_dependent; a fixture for INDIRECT iterative-self cycle (tc_w048).

### 1.3 Gaps

- THE WIRING GAP: GridInvalidationRef + the CTRO rebind model are unwired to the value engines; closures and resolvers exist but nothing calls them during a real recalc. Claims are hand-built in tests; no orchestrator produces the (often paired table+spill) claims from a real edit.
- NO VOLATILITY MODEL: value_cache only excludes volatiles from caching; there is no volatile-root-set seeding pass, so OFFSET-to-changed-target (OFF-02), NOW/RAND chains, and volatile dynamic names go stale under any non-full recalc, and warm-no-op is unsound for volatiles.
- NO SMART/INCREMENTAL RECALC: only mark-all-dirty + visible-rect entry points; dirty_closure is computed but never consumed. No calc-chain forward-reference re-queue or topological ordering; a non-spill formula chain out of address order may read a stale precedent in one pass (unverified, highest-value to check).
- ERROR-EFFECT BUG: the cell/spill resolver emits Ref on every Release; deleted/unknown NAME must surface #NAME? via the existing-but-unemitted NameOrValue. Strict-excel INDIRECT returns a typed exclusion, so its #REF!/retarget behaviour is unimplemented.
- ANCHOR-ONLY SPILL SEEDING: grown/shrunk extent cells and plain-range overlap dependents are missed; the epoch ledger ignores anchor identity (pure-anchor-move under-dirty); non-converging volatile-size spills are not forced to #SPILL!; the blocked-extent ledger recording is asymmetric.
- INCOMPLETE SPILL BLOCKING: no structured-table-body blocker (either direction); the table-grow->spill-anchor BlockedChanged coordination is split across subsystems with no driver.
- NO CYCLE DETECTION / ITERATIVE-CALC ENGINE in the value engines (CycleBlocked exists only on the unwired tree graph); no MaxIterations/MaxChange.
- NO SHEET-AXIS edit (3D-span membership unmodelled); apply_axis_edit handles only Row/Column and never re-resolves dynamic targets after an edit.
- SCOPE-BLIND name keys (excel_grid_defined_name_key is (name,bounds); worksheet-vs-workbook shadowing unrepresentable in the grid layer despite OxFml NameRef carrying sheet_id); no redefine_defined_name reseed; no name-as-DynamicRequest for OFFSET/INDEX dynamic names.
- EXTERNAL invalidation not wired: ExternallyInvalidated has no producer; availability_version bumps don't derive an InvalidationSeed set; NoValueYet maps to #N/A not #BUSY!; no host-environment/format-only invalidation seam; no WEBSERVICE/Power-Query modelling.
- RICH-OBJECT calc surface essentially unbuilt: no dotted-access/FIELDVALUE evaluator, no #FIELD! producer, no rich-aware fingerprint honoring ExcludeFromCalcComparison, refresh is a diagnostic not a dirtying input, no schema-flip reclassification, no referencedValues/nested-entity validation, no errorSubType; rich capability scaffold is a reserved seam with no live kernels.
- CHOOSE/INDEX/IF static over-approximation unverified: no evidence the binder lowers CHOOSE branches / INDEX array args to static Range deps or enumerates both IF arms; under-dirty risk if it prunes to the live branch.

## 2. Cross-cutting themes

- VOLATILE ROOT SET (the single highest-priority missing primitive): OFFSET, INDIRECT, NOW, TODAY, RAND, RANDBETWEEN, RANDARRAY, WEBSERVICE, and arg-conditional CELL/INFO must be unconditionally seeded dirty at the START of every recalc, before closure, then propagate via close_over_dependents in topo order. OxCalc only EXCLUDES volatiles from value caching (VolatileFunction); exclusion-from-cache != force-recalc. Warm-no-op is actively UNSOUND for volatile cells (its token captures only authored state). Spans IOV-OFF-02/VOL-01/04/05, NCW-NAME-01/RTD-01/PQ-01, DAS-SPILL-VOLATILE, DOR-08/14.

- HIDDEN PRECEDENTS / IDENTITY-vs-ADDRESS: INDIRECT/OFFSET/dynamic-name targets and FIELDVALUE field selectors are constructed at calc time and are INVISIBLE to the static dependency graph. Correctness rests entirely on volatility forcing re-evaluation plus fresh resolution. Name/sheet/table lifecycle invalidation (delete/rename) is blind to text-built references. The resolved target must live in ResolvedReferenceIdentity, never as a graph edge. Spans IOV-IND-01/03/04, STBL-RENAME(INDIRECT arm), NCW-NAME-05.

- STABLE REQUEST-KEY, MOVING IDENTITY (the Reclassified sharp edge): a dynamic Reclassified can fire with a byte-identical request_key because the change is recorded in the resolved IDENTITY, not the key. The key must be (owner, argument-AST/source-handle, a1-mode), NEVER content-derived from resolved text; the key holds the UNION of old+new dependents so its closure stays correct across retarget. Downstream invalidation must key on RESOLVED identity, not selector key. Spans IOV-IND-01/VOL-05, STBL-DYNTARGET, DOR-16, NCW-NAME-01/XWB-ALIAS.

- ANCHOR-KEYED SPILL CLOSURE (the seed-by-anchor sharp edge): dirty_closure_for_spill_fact seeds strictly by the anchor cell(s), so growing/shrinking extent cells never enter the closure, and plain RANGE overlap dependents (A1:A10 vs A1#) are missed entirely. REQUIRED FIX (key recommendation): on a spill extent change, seed the CHANGED CELLS (extent delta) as cell-dirty seeds, then close over BOTH spill-fact and compressed-range dependents. The epoch ledger also ignores anchor identity (under-dirty on pure anchor move). Spans DAS-SPILL-GROW/SHRINK/ANCHORMOVE, DOR-RANGE-OVERLAP, RICH-SPILL-FIELD.

- AXIS-EDIT-DURING-REBIND: apply_axis_edit only rewrites addresses (pure arithmetic) and NEVER re-resolves a dynamic target — only from-scratch re-resolution detects divergence. Real references shift on insert; INDIRECT text does NOT. Within one recalc the structural address transform must PRECEDE dynamic re-resolution. Generalizes beyond Row/Column to the SHEET axis (3D spans) and table move/region addresses. Spans IOV-OFF-06/IND-04, DAS-SPILL-AXIS, STBL-TABLEMOVE/AUTOEXPAND, NCW-3D-01, DOR-11.

- ERROR-CODE FAITHFULNESS MATRIX: CHOOSE bad index/2D-@/TEXT(entity)/entity-arithmetic = #VALUE!; OFFSET/INDIRECT off-target-or-unresolvable, deleted column/table, deleted CELL/RANGE, collapsed external link, spill-anchor-deleted = #REF!; deleted/unknown defined NAME = #NAME?; spill blocked/beyond-edge/into-table = #SPILL!; volatile-size non-converging spill = #SPILL!; missing field / non-data-type / no-service-data = #FIELD!; in-flight = #BUSY!/GETTING_DATA; feed failed = #CONNECT!; blocked/disabled = #BLOCKED!; nested-unsupported-array = #CALC!. CRITICAL BUG: the cell/spill resolver emits Ref on every Release, but DynamicRebindErrorEffect::NameOrValue exists and must route deleted-NAME Release to #NAME?.

- NON-TERMINAL / EXTERNALLY-DRIVEN STATES: refresh, RTD push, WEBSERVICE fetch, #BUSY!/GETTING_DATA, #CONNECT!, #BLOCKED!, workspace availability flips are dirtying inputs with NO formula edit and NO structural change. Pending/external-error cells must stay dirty and re-enqueue across recalcs; warm-no-op must not finalize them; a later success must re-dirty to clear. InvalidationReasonKind::ExternallyInvalidated exists but no producer emits it from a host packet, and no glue converts an availability_version bump into an InvalidationSeed set. Spans RICH-REFRESH/BUSY, NCW-XWB-AVAIL/RTD-01/RTD-STATE/PQ-01, STBL-XWS.

- GRAPH MUTATING MID-CALCULATION: spill grow/shrink/block changes which cells EXIST; INDIRECT/OFFSET retarget rebuilds edges; chained dynamic refs propagate transitively; IF/CHOOSE branch flips (Excel keeps BOTH arms as edges — union, never prune); cycles appear only post-resolution; VBA/iterative re-entrancy reshapes tables. Excel handles all of this by calc-chain reorder + additional passes + a bounded fixpoint, never by retroactively reordering an in-flight pass. OxCalc has a single fixed-address-order pass plus a spill-repair fixpoint and no forward-reference re-queue, no cycle detection, no iterative engine.

- SCOPE / NAMESPACE / HOST-VERSION KEYING beyond cell addresses: name resolution depends on (scope-sheet, name) and the caller's sheet (worksheet-scope shadows workbook-scope); 3D-span membership depends on tab order; external refs depend on workspace availability_version; table-function admission depends on a function-registry snapshot; CELL/INFO depend on the host environment and display format. These are non-spatial invalidation axes. OxCalc's grid (name,bounds) key is scope-blind though OxFml NameRef carries sheet_id; HostSensitive/CapabilitySensitive descriptors exist but no host-environment invalidation seam reaches the value engines.

- THE WIRING GAP (everything is contingent on this): the GridInvalidationRef semantic graph and the CTRO rebind model (closures, classifiers, resolvers) are FULLY BUILT but UNWIRED to the value engines GridOptimizedSheet/GridCalcRefSheet, which use warm-no-op + spill-fingerprint invalidation + a brute-force spill-repair fixpoint and rediscover dynamic behaviour each recalc. There is no smart/incremental recalc consuming a dirty closure, no volatile-set seeding pass, no claim-producing orchestrator that emits the (often paired: table+spill) claims, and no post-axis-edit re-resolution call. The closure machinery exists; the consumer does not.

## 3. Recommended review agenda

1. Establish the recalc-driver contract first (everything else is contingent): decide how/whether the GridInvalidationRef semantic graph + CTRO rebind model wire into GridOptimizedSheet/GridCalcRefSheet. Define the smart/incremental recalc entry point that consumes a dirty closure (DOR-SMART-RECALC), since per-scenario invalidation precision is moot until a closure consumer exists.
2. VERIFY the single-pass evaluation model (UNC-SINGLE-PASS-RECURSE, highest-value open question): does oxfunc eval recurse into precedent formulas on-demand or read the partial valuation? Decide topological visit order vs Excel's forward-reference calc-chain reorder for non-spill formula chains (DOR-FORWARD-REF). A correctness bug here invalidates many downstream conclusions.
3. Specify the VOLATILE MODEL: the canonical volatile list (UNC-VOLATILE-LIST), a volatile-root-set seeding pass run before closure each recalc, topo-ordered propagation (DOR-MIX-03), per-name volatility classification (OFFSET volatile vs INDEX not, NCW-NAME-01/02), arg-conditional CELL/INFO, and the warm-no-op unsoundness fix.
4. Pin the DYNAMIC REQUEST-KEY invariant: key on (owner, argument-AST/source-handle, a1-mode), never resolved text; confirm the closure holds the old+new union across Reclassified; confirm downstream keys on RESOLVED identity not selector key (IOV-IND-01/05/VOL-05, STBL-DYNTARGET, DOR-16). Resolve whether INDIRECT/dynamic-name retargets ever materialize DynamicRequest edges from real formulas (today none do).
5. Fix the ERROR-EFFECT ROUTING: split DynamicRebindErrorEffect::Released by family/cause so deleted/unknown NAME -> NameOrValue/#NAME? and deleted CELL/RANGE/table/external -> Ref/#REF! (NCW-NAME-05, IOV-IND-02/03, DOR-10). Reconcile the seam closure with the optimized engine's textual formula-rewrite path so both agree. Walk the full error-code matrix for faithfulness.
6. Resolve the SPILL invalidation seam: replace anchor-only seeding with extent-delta cell-dirty seeding plus closure over BOTH spill-fact and compressed-range dependents (DOR-RANGE-OVERLAP); decide the shrink/vacated-cell and pure-anchor-move under-dirty edges (DAS-SPILL-SHRINK/ANCHORMOVE); standardise the blocked-extent ledger asymmetry; force #SPILL! on non-converging volatile-size spills (DAS-SPILL-NOCONVERGE).
7. Specify SPILL BLOCKING completeness: add the structured-table-body unconditional blocker (UNC-SPILL-TABLE-BLOCKER, both directions: formula-into-table DAS-SPILL-TABLE and table-grows-over-spill STBL-OVERSPILL), confirm merged/feature-region blockers, define the table-grow -> spill-anchor BlockedChanged coordination seam (currently split across two subsystems with no driver).
8. Specify NAMESPACE/SCOPE/HOST-VERSION keying: scope-aware name keys (UNC-NAME-SCOPE-KEY, surface OxFml NameRef.sheet_id into the grid key), name redefinition reseeding (NCW-NAME-04), 3D-span membership on a sheet-axis edit (NCW-3D-01, UNC-3D-ENDPOINT), workspace availability_version -> InvalidationSeed glue (NCW-XWB-AVAIL), and the function-registry/host-sensitive non-spatial axes (STBL-REGISTRY, DOR-MT-ORDER).
9. Specify EXTERNAL/NON-TERMINAL states: emit ExternallyInvalidated from RTD/host packets, model pending-external re-enqueue (NoValueYet/#BUSY!), revisit NoValueYet->NA vs Busy/GettingData (UNC-RICH-BUSY-NA), cached-vs-#REF! for closed workbooks (NCW-XWB-CACHE), and Power-Query refresh as external-seed + structural-resize (NCW-PQ-01).
10. Specify the RICH-OBJECT calc surface (mostly unbuilt): dotted-access + FIELDVALUE evaluators over RichObjectValue.kvps, #FIELD! producer, rich-aware value fingerprint honoring ExcludeFromCalcComparison key_flags (RICH-CARD), refresh-as-dirtying-input + schema-flip reclassification (RICH-REFRESH/FIELD-MISS), entity-spill via array properties (RICH-SPILL-FIELD), basicValue coercion (RICH-COERCE), and root-only referencedValues validation (RICH-NESTED).
11. Specify the AXIS-EDIT-DURING-REBIND ordering rule (structural address transform precedes dynamic re-resolution within one recalc) and the post-axis-edit re-resolution call that apply_axis_edit currently omits (IOV-OFF-06, DAS-SPILL-AXIS, STBL-TABLEMOVE, DOR-11); decide interior-insert-into-spill semantics (UNC-SPILL-INTERIOR-INSERT).
12. Specify CYCLES and ITERATIVE CALC (entirely unbuilt in the value engines): wire NodeCalcState::CycleBlocked/cycle_members from the dependency graph, add a MaxIterations/MaxChange iterative engine distinct from the spill-repair fixpoint, and define calc-time cycle discovery from dynamic refs / self-spill (DOR-CYCLE, DOR-MIX-02).
13. Confirm STATIC OVER-APPROXIMATION lowering: CHOOSE branches and INDEX array args -> static full-range Range deps (not DynamicRequests) (IOV-CHO-01/IDX-01/02, UNC-INDEX-CHOOSE-OVERAPPROX); confirm the binder enumerates BOTH IF/CHOOSE arms (UNC-IF-BRANCH-UNION, DOR-EDGE-MUTATE); confirm INDEX returns a ReferenceLike usable as a range endpoint.
14. Define the CLAIM-PRODUCING ORCHESTRATOR contract: a single edit/recalc event often must emit MULTIPLE coupled claims (table + spill never folded; auto-expand = table-key closure + address closure; PQ refresh = external + structural). Claims are hand-built in tests today; specify who produces them in a real recalc so paired fans are never lost (DOR-21, STBL-WHOLECOL/AUTOEXPAND/OVERSPILL).
15. Cross-cut: settle remaining version-sensitivity and oracle-needed opens (@ on 2D refs UNC-AT-2D, INDIRECT-@ version UNC-INDIRECT-AT-VERSION, CSE vs dynamic UNC-CSE-HANDLING, per-element spill errors UNC-RICH-SPILL-PERELEMENT, closed-workbook structured-ref value UNC-XWB-CLOSED-STRUCTURED) by fixing one target Excel version per behaviour and marking conformance tests config-dependent where Excel itself is.

## 4. Excel-behaviour uncertainty register

### 4.1 Resolved

- **UNC-VOLATILE-LIST** (medium): What is Excel's full canonical volatile-function list and its arg/version-sensitivity (CELL/INFO only for some args; are RANDARRAY/SEQUENCE volatile)?
  - Resolution: Always-volatile: NOW, TODAY, RAND, RANDBETWEEN, RANDARRAY, OFFSET, INDIRECT, and (treated as volatile externally) WEBSERVICE. Conditionally volatile: CELL and INFO only for environment-sensitive args (filename, format, recalc, etc.) — treat per-arg, not whole-function. SEQUENCE is NOT volatile. Confirm modern dynamic-array volatiles against the target Excel version.
- **UNC-INDIRECT-ERROR** (high): Does any unresolvable INDIRECT ref_text yield #REF! (vs #NAME?)?
  - Resolution: Yes. Malformed text, deleted sheet, OR non-existent/deleted defined name inside INDIRECT all collapse to #REF! — DISTINCT from a grammar-time name token in a static formula (#NAME?). #NAME? from INDIRECT only if the function name itself is misspelled. OxCalc's strict-excel INDIRECT currently returns a typed_exclusion, so this #REF! path is unimplemented for the strict profile.
- **UNC-NAME-RELEASE-EFFECT** (high): What error does a Released (deleted/unavailable) defined-NAME dynamic rebind surface? The cell resolver emits DynamicRebindErrorEffect::Ref (#REF!) on Release.
  - Resolution: A deleted/unknown defined NAME must surface #NAME?, not #REF!. DynamicRebindErrorEffect::NameOrValue already exists (rebind.rs:97) but is never emitted. Route name-family Release through NameOrValue/#NAME?; route deleted CELL/RANGE target or collapsed external link through Ref/#REF!. The seam closure and the optimized engine's textual formula rewrite must agree. Confirmed via code: only Ref is emitted today.
- **UNC-NAME-SCOPE-KEY** (high): Does Excel name resolution depend on the caller's sheet (worksheet-scope shadows workbook-scope)?
  - Resolution: Yes. A worksheet-scope name shadows a workbook-scope name of the same spelling FOR FORMULAS ON THAT SHEET; the same unqualified name on a sheet without a shadow resolves the workbook name; an unqualified sheet-scope name is invisible cross-sheet (#NAME? without the Sheet! qualifier). Name keys must encode (scope-sheet, name). OxCalc's excel_grid_defined_name_key(name,bounds) is scope-blind; OxFml NameRef already carries sheet_id.
- **UNC-INDEX-CHOOSE-OVERAPPROX** (medium): Does Excel track the WHOLE INDEX array argument and ALL CHOOSE branches as static precedents even when only one element/branch is selected (does a change in an unselected element dirty the cell)?
  - Resolution: Yes — the static parse links the entire array / all branches as precedents (over-approximation), which is how non-volatile INDEX/CHOOSE stay correct without volatility. OxCalc should record full-range/all-branch static Range deps, NOT DynamicRequests. Web search returned no direct unselected-element confirmation, hence medium.
- **UNC-SPILL-TABLE-BLOCKER** (high): Does OxCalc's blockage probe catch spilling into an Excel Table body?
  - Resolution: No — the probes enumerate merged_regions and feature_rendered_regions but a structured-table body is not an unconditional blocker. Add an explicit rule: a spill extent overlapping any table body (outside the anchor) is unconditionally #SPILL!; wire table extent/resize/move lifecycle to dirty overlapping anchors. Excel authoritative: spilled arrays aren't supported in tables.
- **UNC-SPILL-VOLATILE-SIZE** (high): What does Excel do when a spill's size never stabilizes across passes, and does OxCalc force #SPILL!?
  - Resolution: Excel resolves the anchor to #SPILL! with the 'volatile size' cause. OxCalc sets spill_repair_converged=false at the formula_cells pass bound but does NOT force #SPILL! on non-converged anchors — a confirmed gap. The fixpoint bound is a termination guard, not a circular detector nor a volatile-size error producer.
- **UNC-RICH-BASICVALUE** (medium): What is the exact basicValue for a Stocks/Geography entity used in arithmetic?
  - Resolution: Treat entity basicValue as #VALUE! (basicType error) per the data-types spec, unless a specific data type declares a scalar fallback; IMAGE/_webimage is the exception (fallback is the published string). This is why FIELDVALUE is required to extract numbers and TEXT(entity) gives #VALUE!.
- **UNC-RICH-BUSY-NA** (medium): In-flight linked-data resolution: #BUSY!/GETTING_DATA vs #N/A? OxCalc maps RtdProviderResult::NoValueYet to #N/A.
  - Resolution: Excel historically used GETTING_DATA and now commonly surfaces #BUSY! for linked-data resolution in flight; #N/A is a distinct 'no data' result, not 'still fetching'. The OxFunc value model has both Busy and GettingData codes; the NoValueYet->NA mapping is a simplification to revisit (NoValueYet -> Busy/GettingData for the linked-data path). #BUSY! is non-terminal and must not be cached as final.
- **UNC-RICH-GRANULARITY** (medium): Does Excel track field-granular dependencies (B2 depends on A2.Price only) or cell-granular (all of A2)?
  - Resolution: Cell-granular: any change to a precedent cell dirties dependents, then result-equality (warm-no-op) stops needless downstream propagation. Field-granular tracking is an optimization, not required behaviour. Faithful baseline: dirty all dotted/FIELDVALUE readers on any entity change in the source cell.
- **UNC-RICH-REFRESH-SCOPE** (high): Does refreshing one data-type cell recompute ALL cells of that data type, or only ones whose data changed?
  - Resolution: Refresh updates the selected cell plus all cells with the SAME data type (the invalidation trigger is data-type-scoped), then per-cell result-equality determines downstream propagation. Faithful baseline: a refresh dirties every cell of that data type and their dependents; warm-no-op prunes unchanged results.
- **UNC-XWB-STALE-CACHE** (high): For a closed-workbook external ref with a cached value but a changed on-disk source, is staleness surfaced?
  - Resolution: Served silently from cache; no staleness signal until the source is opened (Available transition) or links are manually updated (Data > Edit Links). Model as a successful resolution against the cached value with no dirtying while Unavailable.
- **UNC-AUTOEXPAND-DIRTY** (high): On table auto-expand, is dependent dirtying driven by the table-name key or by the cell addresses newly entering the extent (or both)?
  - Resolution: Both: structured-reference dependents are dirtied via the table key (dirty_closure_for_table), while any A1-style formula pointing at the newly-absorbed cell is dirtied via the cell/range edge. OxCalc must run the table-key closure AND an address-level closure for the grown region; these are distinct edges in GridInvalidationRef.
- **UNC-PRESERVE-INPUTS** (medium): For ReferencePreserving (SaveReopen identical) the classifier clears changed_dependency_kinds, invalidation_reasons AND prepared_identity_inputs, whereas TypedExclusion clears only dependency_fact_kinds. Is clearing prepared_identity_inputs Excel-faithful or an OxCalc modelling choice?
  - Resolution: An OxCalc modelling choice (no dirtying => no identity inputs need recomputing), consistent with Excel preserving values on an identity-stable reopen. The asymmetry between the two clear-paths is a design-review checkpoint (a known sharp edge), not a bug.

### 4.2 Open (need decisions / oracle checks)

- **UNC-IF-BRANCH-UNION** (medium): Does Excel's dependency tree include BOTH arms of IF/CHOOSE (union), and does OxCalc's reference binder enumerate both arms as static edges?
  - Proposed: Excel keeps BOTH arms as precedents (it does not prune the untaken branch), so a later edit to an untaken-branch precedent still dirties the cell. OPEN for OxCalc: inspect bind_grid_formula_for_transform to confirm conditional sub-expressions contribute references unconditionally; if only the live branch is enumerated, set_cell_dependencies receives an under-complete edge set and under-dirties.
- **UNC-AT-2D** (medium): Does a genuine 2D reference under @ (e.g. =@A1:B2) always yield #VALUE!, or does modern Excel sometimes spill/intersect?
  - Proposed: Keep #VALUE! as the faithful baseline for a 2D area with no single caller-row/col intersection (matches the OxFunc op_implicit_intersection baseline). Where the caller row AND col both fall inside the 2D range, Excel selects the single intersecting cell — confirm whether OxFunc should support that case rather than erroring.
- **UNC-INDIRECT-AT-VERSION** (low): How does @ / implicit intersection interact with a dynamic ref resolving to a multi-cell rect (=@INDIRECT("A1:A10") vs bare)?
  - Proposed: Version-dependent: dynamic-array Excel spills the bare form and @ applies implicit intersection (single owner-row cell or #VALUE!); legacy/CSE Excel implicitly intersects the bare form. The strict-excel profile should fix one version's semantics and document it. Not directly web-verified for INDIRECT specifically.
- **UNC-SPILL-INTERIOR-INSERT** (low): When a row is inserted INSIDE a live spill extent (between anchor and last ghost), does Excel grow the spill by a blank row, push the array down, or break it?
  - Proposed: Proposed: treat an insert at/above the anchor as pure anchor relocation (address rewrite); treat an interior insert as forcing a from-scratch re-resolution of the anchor (AxisEditDuringRebind), since the array contents do not contain the inserted blank — Excel re-lays the unchanged result at the shifted anchor. Not confirmed by Microsoft docs; verify against live Excel.
- **UNC-BLOCKED-EXTENT-LEDGER** (medium): What extent is recorded in the spill ledger for a blocked anchor, and what does A2# of a blocked anchor return?
  - Proposed: OxCalc records the FULL would-be extent for an occupancy block but a 1x1 anchor rect for an out-of-bounds/overflow block (publish paths differ) — a confirmed asymmetry to standardise or document. For consumers, A2# of a blocked anchor in Excel resolves to the anchor cell only (holding #SPILL!), so SUM(A2#) propagates #SPILL!/#REF! — needs a live-Excel oracle check for the exact downstream code.
- **UNC-SPILL-ANCHORMOVE-UNDERDIRTY** (medium): Is a spill whose anchor moves but whose extent/value/blocked are all identical under-dirtied (epoch change-kind ignores anchor identity)?
  - Proposed: Confirmed via code: spill_epoch_change_kind keys on extent/blocked/value only, so a pure anchor move with identical extent/value/blocked yields no change-kind -> ReferencePreserving -> empty closure. Decide whether Excel can even produce this case (an anchor move usually changes value/extent); if reachable, add anchor identity to the change-kind or seed both anchors unconditionally.
- **UNC-CSE-HANDLING** (medium): How does OxCalc handle explicitly-CSE-entered (legacy {=...}) array formulas vs dynamic arrays?
  - Proposed: CSE has a fixed pre-allocated extent: recomputes in place, never auto-resizes, never #SPILL!, truncates over-large and #N/A-pads under-large. Model CSE as N committed formula cells sharing one expression (fixed Range dependents), distinct from the dynamic anchor+ghost+SpillFact+epoch model. Confirm OxCalc's CSE handling for round-trip fidelity. Version-sensitive.
- **UNC-RICH-SPILL-PERELEMENT** (low): Do #FIELD!/per-element errors short-circuit a spilling FIELDVALUE, or appear per element?
  - Proposed: Per-element: extraction failures appear as per-element #FIELD! within the spill (the array still spills). A whole-array structural failure (nested unsupported array) yields #CALC!; a blocked target yields a single #SPILL! at the anchor. Needs empirical confirmation against live Excel.
- **UNC-3D-ENDPOINT** (medium): Deleting an ENDPOINT sheet of a 3D span: collapse to the surviving endpoint, or #REF!?
  - Proposed: Model interior-sheet deletion as DependencyRemoved (span shrinks, no error); endpoint deletion as: span collapses to surviving sheets if any interior/other endpoint remains, full collapse (both endpoints gone, or the only referenced sheet deleted) -> #REF!. A single-sheet ref to a deleted sheet is unambiguously #REF!. Version-specific oracle check recommended.
- **UNC-XWB-OPEN-GATE** (medium): Is opening a source workbook an automatic recalc-on-availability, or gated by the Update Links prompt / automatic-vs-manual setting?
  - Proposed: Treat availability-open as a recalc trigger for external-link consumers, but gate the actual value refresh on the workbook's update-links setting (automatic by default, can be manual/prompt-suppressed). Model the gate as part of the availability_version bump being applied or deferred.
- **UNC-WEBSERVICE-GRANULARITY** (medium): Does WEBSERVICE recalc on EVERY recalc (fully volatile) or only on explicit recalc/open?
  - Proposed: Model WEBSERVICE as fully volatile (recalc-always set) like OFFSET/INDIRECT, re-fetching on each recalc — the conservative Excel-faithful default matching its volatile classification. Exact triggering granularity is fuzzy in sources.
- **UNC-SINGLE-PASS-RECURSE** (medium): In a single mark-all-dirty pass with no spill refs, does oxfunc eval recurse into precedent FORMULAS on-demand, or read the partial shared valuation (risking a stale precedent for out-of-address-order formula chains)?
  - Proposed: OPEN and HIGHEST-VALUE to verify: trace evaluate_formula_with_oxfml's reference enumeration against the GridOptimizedValuation reader. If a reference to a not-yet-visited formula cell returns a partial/blank value, OxCalc needs either a topological visit order or Excel's forward-reference re-queue for correctness on non-spill chains. calc_ref_sheet.rs visits authored cells in BTreeMap order exactly once; only spill-repair re-runs.
- **UNC-MIDCALC-RERESOLVE** (medium): Does Excel ever re-resolve a structured reference DURING a single recalc pass (true mid-calc graph mutation), or only across passes?
  - Proposed: Within a normal pass the calc chain is fixed; structural mutation comes from VBA/automation re-entrancy or iterative calc, handled by re-marking dirty and running ADDITIONAL passes, not by reordering an in-flight pass. Model mid-calc reshape as a re-pass/re-seed, never as retroactive reordering. OxCalc has no calc-loop wiring for this.
- **UNC-XWB-CLOSED-STRUCTURED** (low): Cross-workspace structured-reference value while the source workbook is CLOSED: cached last-saved values, #REF!, or update-links prompt?
  - Proposed: Treat WorkspaceClose as 'cached values retained, edges Released' for the model, but flag the surfaced VALUE as configuration-dependent (calculate-on-open, update-links prompt, trust-center) in any conformance test; do not assert a single value.

## 5. Scenario corpus (85 scenarios)

### address-dynamic-volatile (20)

#### IOV-IND-01 - INDIRECT(text) retargets when its text precedent changes  _(reference-resolution|dependency-order-rework)_
- **Setup:** A1="B"&C1 -> "B5"; D1=INDIRECT(A1). C1=5 so D1 points at B5(=100); B9=999.
- **Event:** C1 5->9; A1 recomputes to "B9"; D1's INDIRECT re-resolves at calc time.
- **Excel:** D1=999. INDIRECT re-runs every recalc (volatile) and resolves "B9" to B9. No static D1->B5/B9 edge exists; only D1->A1 (text source). Correctness comes from volatility forcing recompute then fresh resolution.
- **Consequence:** CTRO CellDynamicRequest IdentityReclassified: before target=B5, after target=B9, request_key STABLE (keyed on request/owner not resolved target). dirty_closure_for_dynamic_request fans from the request key's pre-seeded dependents. Sharp edge: Reclassified can fire with a byte-identical request_key because the moved target lives only in ResolvedReferenceIdentity (rebind.rs:164). Same mechanism as STBL-22, DOR-16, NAME-03.

#### IOV-IND-02 - INDIRECT of unresolvable text -> #REF! (not #NAME?)  _(reference-resolution|invalidation)_
- **Setup:** A1="not_a_ref"; B1=INDIRECT(A1). Also covers deleted sheet text and non-existent defined name inside INDIRECT.
- **Event:** Recalc evaluates INDIRECT("not_a_ref"); text matches no A1/R1C1 syntax and no defined name.
- **Excel:** B1=#REF! (NOT #NAME?). Malformed text, deleted sheet, OR non-existent defined name all collapse to #REF! from INDIRECT specifically. #NAME? from INDIRECT only if the function name itself is misspelled.
- **Consequence:** CTRO IdentityReleased -> Released/Ref, reasons {DynamicDependencyReleased}, kind {DynamicPotential}. Critical contrast with grammar-time name token (#NAME?). GAP: strict-excel INDIRECT returns typed_exclusion strict_excel_profile_not_supported (formula.rs:2357) so this #REF! path is not implemented for the strict profile.

#### IOV-IND-03 - INDIRECT to a defined name later deleted; name-lifecycle invalidation misses it  _(reference-resolution|invalidation)_
- **Setup:** Name MyRng->Sheet1!$A$1:$A$3. B1=SUM(INDIRECT("MyRng")).
- **Event:** User deletes name MyRng; recalc runs.
- **Excel:** B1=#REF! (INDIRECT cannot resolve the missing name; not #NAME? because failure is INDIRECT text resolution, not a grammar name token). Contrast: static =SUM(MyRng) becomes =SUM(#REF!) at commit time.
- **Consequence:** No static Name edge exists for the INDIRECT (it is hidden inside text), so delete_defined_name's name closure (dirty_closure_for_name) does NOT catch B1. B1 is only recomputed because INDIRECT is volatile. Core 'hidden precedent' gap: name lifecycle invalidation is blind to INDIRECT-by-text dependents.

#### IOV-IND-04 - INDIRECT cross-sheet text where sheet is renamed/deleted  _(reference-resolution|invalidation)_
- **Setup:** C1=INDIRECT("Data!A1"); sheet Data exists, Data!A1=42.
- **Event:** Sheet Data renamed to Archive (or deleted); recalc.
- **Excel:** C1=#REF!. INDIRECT does NOT auto-rewrite sheet text on rename (unlike a normal reference, which Excel rewrites). The deliberate 'rename-proof anchor' idiom and a rename/delete footgun. Same as STBL-09 (INDIRECT of structured-ref text does not track rename).
- **Consequence:** CTRO IdentityReleased. Rename does not transform the text, so apply_axis_edit/namespace rename cannot keep C1 correct; only from-scratch re-resolution (driven by volatility) detects divergence. Mirrors the AxisEditDuringRebind sharp edge (DOR-11): an axis/rename edit rewrites addresses but never re-resolves a dynamic text target.

#### IOV-IND-05 - INDIRECT(...,FALSE) R1C1-mode resolution is owner-position-sensitive  _(reference-resolution)_
- **Setup:** A1="R[1]C[1]"; B5=INDIRECT(A1,FALSE).
- **Event:** Recalc resolves text in R1C1 mode relative to B5, targeting C6.
- **Excel:** B5 = value of C6 (relative R1C1 against owning cell). With a1=TRUE, "R[1]C[1]" is not valid A1 text -> #REF!. The a1 flag changes the parse grammar of the same text, so the resolved target is parse-mode-dependent.
- **Consequence:** request_key must encode (text, a1-mode, owner-cell) so two cells with identical text but different owners/modes do not collide in dynamic_dependents_by_request. Copying a relative-R1C1 INDIRECT changes the target without changing the text. OxCalc GridDependency::DynamicRequest(String) keying must include a1 flag + owner anchor.

#### IOV-OFF-01 - OFFSET drifts off-grid / zero-or-negative height-width -> #REF!  _(reference-resolution|invalidation)_
- **Setup:** B1=OFFSET($A$1,C1,0) (drift) or OFFSET($A$1,0,0,C1,1) (resize). C1 driven to -1, past max bound, 0, or -2.
- **Event:** Recalc re-resolves OFFSET to an off-grid rect or with height/width<1.
- **Excel:** B1=#REF! when top/left<1, height/width<1, or bottom/right exceeds the sheet bound.
- **Consequence:** CTRO IdentityReleased -> Released/Ref; drift in/out of bounds detected ONLY by re-running OFFSET (volatile). Confirmed present: offset_reference (reference_engine.rs:783-832) enforces new_top>=1,new_left>=1,new_height>=1,new_width>=1,bottom<=max_rows,right<=max_cols and maps off-grid -> ProviderFailure -> #REF!.

#### IOV-OFF-02 - OFFSET drifts onto a new in-bounds target whose value later changes (stale without volatile re-seed)  _(invalidation)_
- **Setup:** B1=OFFSET($A$1,C1,0). C1=2 -> A3=10. A3 later set 10->77 with C1 unchanged.
- **Event:** A3's value changes; nothing else edited.
- **Excel:** B1=77, because OFFSET is volatile and B1 recalcs on ANY workbook change and re-reads A3. No static A3->B1 edge needed.
- **Consequence:** Volatility-propagation case: a value change at the RESOLVED target (A3) is on no recorded precedent edge of B1; a non-volatile graph leaves B1 stale. GAP: optimized warm-no-op/spill-fingerprint invalidation has no 'recalc-always volatile cells' notion; a volatile cell-list must be seeded dirty every pass independent of the graph. Same root gap as IOV-VOL-01, NCW-VOL-01, DOR-14.

#### IOV-OFF-03 - OFFSET with dynamic height/width is an array source whose spill extent changes  _(reference-resolution|invalidation|dependency-order-rework)_
- **Setup:** B1=OFFSET($A$1,0,0,D1,1) as a dynamic array. D1=3 spills B1:B3.
- **Event:** D1 3->5; OFFSET yields a 5-tall array; spill extent grows B1:B5 (or #SPILL! if B4/B5 occupied).
- **Excel:** Spill grows to B1:B5; dependents of the spill anchor B1# see ExtentChanged. Bridges this domain to the SpillAnchorRef family.
- **Consequence:** Two coupled claims: (1) CellDynamicRequest for OFFSET retarget; (2) SpillAnchorRef on B1 with SpillEpochChanged(ExtentChanged). Sharp edge: spill closures seed by ANCHOR, so a shrink dirties NO vacated-extent cells (B4,B5) unless they declared a spill-fact dep. offset_reference returns a single rect; the spill ledger/epoch wiring is in the value engine and is NOT driven by OFFSET re-resolution today. Couples with DAS-SPILL-GROW, DOR-08.

#### IOV-OFF-05 - OFFSET on a multi-area source -> Unsupported(Transform) must map to #REF!  _(reference-resolution)_
- **Setup:** Name Multi -> (A1:A2,C1:C2). B1=OFFSET(Multi,1,0).
- **Event:** Recalc tries to offset a multi-area reference.
- **Excel:** Excel: OFFSET requires a single contiguous reference; multi-area -> #REF!/#VALUE! by context.
- **Consequence:** OxCalc returns ReferenceSystemError::Unsupported(Transform) when rects.len()!=1 (reference_engine.rs). Design-review: verify the OFFSET function wrapper maps Unsupported(Transform) -> #REF! rather than leaking an internal Unsupported error. No dynamic retarget recorded for an unsupported source.

#### IOV-OFF-06 - OFFSET anchor moved by an axis edit during the same recalc (rewrite-then-re-resolve order)  _(dependency-order-rework|invalidation)_
- **Setup:** B1=OFFSET($A$1,2,0). A row inserted above row 1 so $A$1 tracks to $A$2.
- **Event:** Row insert at top, then recalc. The anchor address is rewritten by the structural edit; the offset re-applied to the new anchor.
- **Excel:** Excel rewrites literal $A$1 -> $A$2 (structural fix-up), THEN OFFSET re-applies offset 2 from A2. Address-rewrite (static) composed with calc-time re-resolution.
- **Consequence:** AxisEditDuringRebind -> reasons {StructuralRebindRequired}, kind {DynamicPotential}. apply_axis_edit (invalidation.rs:564) rewrites addresses but does NOT re-run OFFSET; ORDER constraint: structural address transform must precede dynamic re-resolution within one recalc, else OFFSET applies from the stale anchor. Note the contrast with IOV-IND-04: a real reference anchor IS rewritten; INDIRECT text is NOT.

#### IOV-VOL-01 - Volatile root set: NOW/TODAY/RAND/RANDBETWEEN/INFO/CELL re-seed dirty every recalc  _(invalidation|dependency-order-rework)_
- **Setup:** A1=NOW(); B1=A1+1; C1=B1&" hrs". Also RAND/RANDBETWEEN (effectful+volatile), CELL/INFO (env-sensitive arg forms).
- **Event:** Any recalc anywhere (unrelated edit, or F9).
- **Excel:** All volatile cells AND their downstream cone recalc EVERY recalc, regardless of precedent change. RAND yields a new value each pass; CELL("format",D1) is sensitive to D1's display FORMAT (not a value precedent).
- **Consequence:** Requires a dedicated volatile-root set unconditionally added to the dirty seed each recalc BEFORE closure; the existing close_over_dependents then carries the cone. GAP (highest priority): no volatility model. value_cache.rs only EXCLUDES volatile/effectful paths from CACHING (VolatileFunction); exclusion-from-cache != force-recalc. Format-only edits as an invalidation source are unmodelled. Unifies with IOV-OFF-02, NCW-VOL-01, DAS-SPILL-VOLATILE, DOR-14.

#### IOV-VOL-04 - Volatility transitively forces a long non-volatile dependent chain (topo-ordered)  _(dependency-order-rework)_
- **Setup:** A1=TODAY(); A2=A1+1; ...; A100=A99+1 (plain arithmetic).
- **Event:** Date rolls / F9 / any recalc.
- **Excel:** All A1..A100 recalc, in calc-chain order, every recalc — the entire downstream cone of one volatile root.
- **Consequence:** Volatile root A1 seeds dirty; close_over_dependents (invalidation.rs:1402) carries the chain. The EXISTING dirty-closure machinery is sufficient for PROPAGATION; the only missing piece is the per-cycle volatile SEED. Cost: a single volatile root can dirty an arbitrarily large cone each recalc; the calc chain must still topologically order them (see DOR-MIX-03).

#### IOV-VOL-05 - Volatile output feeds a dynamic resolver (INDIRECT of NOW-derived text)  _(reference-resolution|invalidation)_
- **Setup:** A1="B"&(HOUR(NOW())+1); C1=INDIRECT(A1).
- **Event:** Hour rolls; NOW re-evaluates, A1 text changes, INDIRECT re-resolves (e.g. B13->B14).
- **Excel:** C1 retargets as the hour changes. Two volatility sources compound: NOW (root volatile) drives A1; INDIRECT (volatile) re-resolves.
- **Consequence:** Volatile seeding (IOV-VOL-01) AND CellDynamicRequest reclassification (IOV-IND-01) interact. OPEN: is the request_key keyed by LITERAL text (changing) or by owner+arg-AST identity (stable)? rebind.rs assumes a STABLE request_key across reclassification; a content-derived key would orphan old buckets and break the closure invariant. Proposed: key on (owner, argument-AST/source-handle, a1-mode), never the resolved text.

#### IOV-CHO-01 - CHOOSE reference form selects a different area; all branches are static precedents (non-volatile)  _(reference-resolution|dependency-order-rework)_
- **Setup:** B1=SUM(CHOOSE(C1,A1:A3,A4:A6,A7:A9)). C1=1 -> SUM(A1:A3).
- **Event:** C1 1->3; CHOOSE returns A7:A9.
- **Excel:** B1=SUM(A7:A9). CHOOSE is NOT volatile; all candidate areas are STATIC precedents recorded at parse time, so Excel recomputes when C1 changes AND when any candidate area changes. Over-approximation by tracking ALL branches.
- **Consequence:** Static Range edges B1->A1:A9 (all branches) exist; no DynamicRequest. A change in A7:A9 dirties B1 via the normal Range closure even when C1 selects branch 1. Dependency-order rework WITHOUT graph mutation: selected branch changes, precedent SET is constant. GAP: confirm OxCalc lowers CHOOSE branches to static Range deps. Same strategy as IOV-IDX-01, contrast with DOR-09 IF-branch union.

#### IOV-CHO-02 - CHOOSE index out of range -> #VALUE!  _(reference-resolution)_
- **Setup:** B1=CHOOSE(C1,A1:A3,A4:A6). C1=5 (2 branches).
- **Event:** Recalc with index 5 > branch count.
- **Excel:** B1=#VALUE! (index <1 or > value count). NOT #REF!.
- **Consequence:** Value-level error, no identity release; static precedent set unchanged. Error-code matrix for this domain: CHOOSE bad index=#VALUE!; OFFSET/INDIRECT off-target or unresolvable=#REF!; misspelled function name=#NAME?; spill into occupied=#SPILL!.

#### IOV-IDX-01 - INDEX reference form returns a moving cell; whole array stays a precedent (non-volatile)  _(reference-resolution|dependency-order-rework)_
- **Setup:** B1=INDEX(A1:A10,C1). C1=3 -> A3.
- **Event:** C1 3->7 (now A7); separately A3 changes while C1=7.
- **Excel:** On C1 3->7, B1=value of A7. On A3 change while C1=7, B1 STILL recalculates because Excel records the WHOLE array A1:A10 as B1's precedent. INDEX is non-volatile but tracks the entire array argument.
- **Consequence:** Static Range edge B1->A1:A10 (full array); no DynamicRequest. The selected element moving (A3->A7) is internal and not a graph change. Over-approximation keeps Excel correct without volatility. GAP: confirm OxCalc lowers INDEX array arg to a static full-array Range dep.

#### IOV-IDX-02 - INDEX reference form as a range endpoint: A1:INDEX(A:A,C1) builds a dynamic range non-volatilely  _(reference-resolution|dependency-order-rework)_
- **Setup:** B1=SUM(A1:INDEX(A:A,C1)). C1=5 -> SUM(A1:A5).
- **Event:** C1 5->8; endpoint extends to A8.
- **Excel:** B1=SUM(A1:A8). INDEX returns a REFERENCE used as a range terminus; the resolved range is a calc-time fact. Non-volatile; the canonical INDIRECT replacement idiom. Same shape as NCW-NAME-02 ($A$2:INDEX($A:$A,COUNTA)).
- **Consequence:** Constructed-range extent is dynamic but Excel tracks the conservative super-range (the column) as precedent. Calc-time reference CONSTRUCTION (not just selection). OxCalc options: (a) over-approximate to A:A as a Range dep (Excel-faithful, avoids hidden-precedent staleness), or (b) model a DynamicRequest for the endpoint. Confirm the reference engine can return a ReferenceLike from INDEX and union it into a range endpoint.

#### DOR-MIX-02 - Dynamic reference resolves to the owning cell (calc-time-discovered circular edge)  _(reference-resolution|invalidation)_
- **Setup:** A1=INDIRECT("A1") or A1=OFFSET(A1,0,0). Also covers A1=SUM(A1#) / A1=A1#+1 self-spill.
- **Event:** Recalc; the resolved target equals the owning cell.
- **Excel:** Circular reference. Iterative calc OFF -> 0 / circular warning; ON -> iterates per settings. Excel detects the cycle at CALC time because the edge only exists post-resolution (invisible to the static graph).
- **Consequence:** A CALC-TIME-discovered edge, not a static one — dependency-order rework that introduces a cycle into the calc chain mid-calculation. Fixture exists: tc_w048_excel_ctro_indirect_iterative_self_001. Self-spill (A1#) must be caught by the spill-repair fixpoint's non-convergence guard rather than looped forever; confirm circular-policy vs fixpoint-bound interaction (may surface #REF! rather than a circular warning). Ties to DOR-12/13 iterative calc.

#### DOR-MIX-03 - Volatile recalc ordering: a non-volatile cell reads a volatile cell that precedes it  _(dependency-order-rework)_
- **Setup:** A1=B1+1 (non-volatile); B1=OFFSET(...) (volatile).
- **Event:** Recalc seeds B1 (volatile) dirty; A1 depends on B1.
- **Excel:** Excel must compute B1 before A1 every recalc; out-of-order processing would read a stale B1.
- **Consequence:** Volatile seeding interacts with calc-chain topology: seeding B1 dirty is necessary but not sufficient — A1 must also be dirtied (via closure) AND ordered after B1. close_over_dependents produces a SET, not an order; the engine's calc-chain layer must topologically order it. Volatile handling is 'add root + re-run closure + preserve topo order', not just 'add to dirty set'.

#### IOV-MIX-01 - Implicit intersection / @ on a dynamic reference result  _(reference-resolution)_
- **Setup:** C1=INDIRECT("A1:A10") (bare, would spill 10) vs =@INDIRECT("A1:A10").
- **Event:** Recalc evaluates the multi-cell result in a scalar context.
- **Excel:** Dynamic-array Excel: bare INDIRECT("A1:A10") spills 10 cells; =@INDIRECT applies implicit intersection to one cell on the formula's row (or #VALUE!). Legacy/CSE Excel: bare form implicitly intersects. @ changes the calc-time SHAPE of the dynamic result. Version-dependent.
- **Consequence:** Implicit intersection picks ONE cell from the resolved rect at calc time based on owner row/col — owner-position-sensitive resolution layered on the dynamic retarget. Interacts with SpillAnchorRef when the un-intersected form spills. Same @ machinery as DAS-SPILL-AT, RICH-13, STBL-13. Specify per target Excel version; flag version-sensitive.

### dependency-order-rework (5)

#### DOR-FORWARD-REF - Forward reference / out-of-order precedent in a single mark-all-dirty pass (calc-chain reorder gap)  _(dependency-order-rework)_
- **Setup:** A1=INDIRECT("Z1"), Z1=10 (precedent Z1 visited AFTER A1 in address order). Also chained dynamic refs A1="D1",B1=INDIRECT(A1),C1=B1+1.
- **Event:** Single mark-all-dirty pass visits A1 before its precedent; B1 re-resolves and C1 must follow.
- **Excel:** Excel's calc chain, on encountering a dirty/uncalculated precedent mid-evaluation, STOPS, moves the precedent's formula immediately before the current one, and SAVES the reordered chain (amortized). Final A1 reads the current Z1; B1=20,C1=21 in one self-consistent pass.
- **Consequence:** GAP: OxCalc has NO forward-reference re-queue or calc-chain reorder — it visits cells strictly in BTreeMap address order ONCE; only the spill-repair fixpoint re-runs (and only when spill refs are present). A formula-to-formula forward dependency in a single non-spill pass can read a STALE precedent value. The seam's dirty_closure_for_dynamic_request + close_over_dependents would carry C1 transitively (test cell_activated_dirties_request_closure_with_chained_dependent) but the seam is unwired. HIGHEST-VALUE VERIFICATION: does oxfunc eval recurse into precedent FORMULAS on-demand, or read the partial valuation? (UNC-SINGLE-PASS-RECURSE). Covers DOR-06, DOR-07.

#### DOR-EDGE-MUTATE - Edges added/removed mid-life: IF-branch flip (both arms are precedents) and name-delete edge drop  _(dependency-order-rework|invalidation)_
- **Setup:** C1=IF(A1>0,B1,5); A1=-1 then edited to 1 (live branch flips to read B1). Separately MyRange=A1:A3 deleted while C1=SUM(MyRange).
- **Event:** Recalc after the branch flip; recalc after the name delete.
- **Excel:** IF: C1=B1; Excel keeps BOTH potential precedents as edges (it does NOT prune the untaken branch), so a later B1 edit already dirties C1. Name delete: C1=#NAME? and the name edge is dropped.
- **Consequence:** Excel's static dependency tree is the UNION of all branches (over-approximation); a from-scratch resolver that prunes to the live branch would MISS a later untaken-branch-precedent edit (under-dirty). set_cell_dependencies records whatever set the binder supplies — if the binder enumerates only the live branch, the seam under-dirties. OPEN UNC-IF-BRANCH-UNION: does OxCalc's reference binder enumerate BOTH IF/CHOOSE arms as static edges? delete_defined_name dirties the name-key closure BEFORE purge (correct). Contrast with IOV-CHO-01/IOV-IDX-01 (CHOOSE/INDEX over-approximate by design). Covers DOR-09, DOR-10.

#### DOR-CYCLE - Circular reference: iterative calc OFF (warning/0) vs ON (bounded iteration)  _(dependency-order-rework)_
- **Setup:** A1=B1+1,B1=A1+1 (iterative OFF). Or A1=B1*0.1,B1=C1-A1 with iterative ON (max 100 iter, max change 0.001).
- **Event:** Recalc encounters the cycle.
- **Excel:** OFF: circular-reference warning; cycle cells resolve to 0; status bar flags it. ON: Excel iterates up to MaxIterations or until the largest single-cell change < MaxChange, seeding each iteration from the previous iteration's result.
- **Consequence:** GAP: OxCalc value engines have NO cycle detection and NO iterative-calc engine (no MaxIterations/MaxChange/epsilon convergence). recalc visits cells once in address order; a direct cycle would read stale/partial values producing arbitrary non-Excel numbers with no warning. The unwired dependency.rs graph HAS NodeCalcState::CycleBlocked + cycle_members but is not connected to the value engines. The ONLY fixpoint is spill-repair, which converges on spill_facts-map EQUALITY (exact, not epsilon) bounded by formula_cells — structurally different from iterative calc. Ties to DOR-MIX-02 (dynamic-ref self-cycle), DAS-SPILL-NOCONVERGE. Covers DOR-12, DOR-13.

#### DOR-SMART-RECALC - Smart/incremental recalc: edit a literal recalcs only its dependent cone (the marquee gap)  _(recalc-scheduling|invalidation)_
- **Setup:** Large sheet; A1 literal edited; only D1=A1*2 and E1=D1 depend on it.
- **Event:** Edit A1; recalc.
- **Excel:** Excel marks only A1's transitive dependents dirty and recalcs D1,E1 (nothing else). Smart recalc.
- **Consequence:** GAP (marquee 'what we want vs what we have'): the value engines have ONLY recalculate_MARK_ALL_DIRTY entry points (plus recalculate_visible_rect, a viewport scope not a dependency scope). There is NO incremental recalc consuming a dirty closure. GridInvalidationRef::dirty_closure computes EXACTLY this forward closure but is UNWIRED — the closure machinery exists, the consumer (a dirty-driven recalc) does not. Every other scenario's invalidation precision is moot until a smart-recalc driver consumes the closures. Covers DOR-23.

#### DOR-MT-ORDER - Multithreaded calc ordering; INDIRECT forces a single-threaded serialization barrier  _(recalc-scheduling|dependency-order-rework)_
- **Setup:** Two independent formula columns (no cross-edges) plus one column containing INDIRECT.
- **Event:** Recalc with multithreaded calculation enabled.
- **Excel:** Excel partitions the calc chain into independently-calculable groups across threads; the INDIRECT column is forced single-threaded (INDIRECT defeats MT calc) and cannot join a parallel group.
- **Consequence:** Dynamic/host-sensitive edges force a serialization barrier; the schedulable partition shrinks. OxCalc exposes partition_witness_report computing a max_parallelism_bound from non-overlapping storage regions — a STORAGE partition, not a dependency-respecting calc partition, and it does NOT account for INDIRECT/dynamic serialization. DependencyDescriptorKind::HostSensitive/DynamicPotential are the right vocabulary but no scheduler consumes them. GAP: no dependency-aware multithreaded calc ordering. Same HostSensitive/DynamicPotential descriptors as STBL-REGISTRY, IOV-VOL-01 (CELL/INFO). Covers DOR-15.

### dynamic-arrays-spill (19)

#### DAS-SPILL-GROW - Spill extent grows (SEQUENCE/FILTER/UNIQUE n increases); new ghost cells appear  _(invalidation|dependency-order-rework)_
- **Setup:** A1=SEQUENCE(B1), B1=3 -> A1:A3. C1=SUM(A1#). Also FILTER/UNIQUE result-size growth from upstream data.
- **Event:** B1 3->5 (or predicate matches 6 vs 4 rows); the array result grows.
- **Excel:** A1 spills A1:A5; new ghost cells A4:A5 become read-only spilled cells owned by A1. A1# resolves to the live extent so C1=SUM(A1#) recomputes over the larger range. No #SPILL! if the new cells are empty; #SPILL! if occupied (collapses A1# to a 1x1 blocked anchor).
- **Consequence:** Anchor spill epoch ExtentChanged (or ExtentAndValueChanged). SHARP EDGE: dirty_closure_for_spill_fact seeds by ANCHOR not extent, so the freshly-grown ghost cells A4:A5 are NOT in the closure. New ghost cells are produced by the value-engine spill-repair pass (push_dense_value_payload over the new extent), not by the invalidation closure. The new larger extent must be probed for blockage BEFORE commit. Canonical for IOV-OFF-03, DOR-01, RICH-09, STBL-03, DAS-SPILL-CASCADE.

#### DAS-SPILL-SHRINK - Spill extent shrinks; vacated ghost cells revert to blank and stale dependents must recalc  _(invalidation|dependency-order-rework)_
- **Setup:** A1=SEQUENCE(B1), B1=5 -> A1:A5. D6=A4 (plain cell ref into a current ghost cell).
- **Event:** B1 5->2; result shrinks to 2 rows; A3:A5 vacate.
- **Excel:** A1 spills A1:A2; A3:A5 revert to empty; D6=A4 recomputes to 0/blank.
- **Consequence:** Spill epoch ExtentChanged. SHARP EDGE confirmed (rebind.rs:311-336): closure seeds by anchor only, so a SHRINK dirties NO extent cells — D6 (scalar dependent of the ex-ghost A4) is reached only via dependents_by_cell[A4], NOT via the spill closure. The value engine clears vacated cells (clear_formula_output_for_anchor) so a mark-all-dirty pass gives D6 a fresh blank, but a TARGETED/incremental recalc driven by the seam would MISS D6. Seam vs value-engine divergence on vacated-cell dependents. Same as DOR-02.

#### DAS-SPILL-BLOCK - #SPILL! when the spill range is blocked by an occupied cell, merged region, or grid bound  _(reference-resolution|invalidation)_
- **Setup:** A1=SEQUENCE(3) wants A1:A3 but A2 holds literal 99; OR A2:B2 merged; OR anchor near sheet edge spilling past max_rows.
- **Event:** A1 recalculates and the result cannot lay down.
- **Excel:** A1=#SPILL! (blocked / beyond-edge). No ghost cells written; the obstruction keeps its value. A1# resolves to a blocked 1x1 anchor-only extent.
- **Consequence:** publish_formula_value detects the block via optimized_spill_extent_is_blocked / reference_spill_extent_is_blocked (merged_regions are unconditional blockers, confirmed) and writes GridSpillFact{blocked:true} + a #SPILL! CalcValue. LEDGER ASYMMETRY: occupancy-block records the FULL would-be extent; out-of-bounds-block records a 1x1 anchor_cell_rect (spill_extent_for_array returns None) — relevant for fingerprint/epoch comparisons. Covers SPILL-04, SPILL-14, SPILL-18, DOR-03.

#### DAS-SPILL-UNBLOCK - Unblocking a #SPILL! by clearing the obstruction triggers re-spill  _(invalidation|dependency-order-rework)_
- **Setup:** A1=#SPILL! (blocked by literal in A2). C1=A1#.
- **Event:** User deletes the literal in A2; recalc.
- **Excel:** A1 re-spills A1:A3; A1# resolves to A1:A3; C1 recomputes over the live extent.
- **Consequence:** BlockedChanged epoch (true->false) -> cause_to_reason_table maps to {StructuralRebindRequired, DynamicDependencyReclassified}. The obstructing cell A2 is a SpillBlocker dependency: clearing it must dirty the blocked anchor A1 via spill_blocker_dependents_by_cell (dirty_closure_for_spill_blocker). The blocker->anchor edge is the only thing that makes a blocked anchor recompute when an unrelated cell is cleared. NOTE: the optimized engine rediscovers unblocking via the spill-repair fixpoint, not via the unwired GridInvalidationRef.

#### DAS-SPILL-TABLE - Spilling into an Excel Table body is unconditionally #SPILL!  _(reference-resolution|invalidation)_
- **Setup:** Table1 occupies A1:C10; E1 inside/overlapping the table body has =SEQUENCE(3) attempting to spill into table rows.
- **Event:** Recalc evaluates a spilling formula whose extent overlaps a table region.
- **Excel:** #SPILL! — dynamic arrays cannot spill inside an Excel Table; a table column expects one (implicit-intersection) result per row. Authoritative: 'Spilled array formulas aren't supported in Excel tables.' A hard structural rule, not a per-cell occupancy check. Same invariant as STBL-13.
- **Consequence:** GAP (high confidence): blockage probes enumerate merged_regions and feature_rendered_regions but do NOT enumerate a structured-table body as an unconditional blocker. Needs a table-overlap blocker rule. Once modelled, a table resize/move (StructuredTableRebind) newly overlapping the extent must flip the anchor's BlockedChanged epoch (table-extent edge -> blocked epoch). Couples with STBL-12 (table grows OVER an external spill anchor).

#### DAS-SPILL-COLLIDE - Two spilling arrays collide as upstream data resizes (calc-order/anchor-position dependent)  _(invalidation|dependency-order-rework)_
- **Setup:** A1=SEQUENCE(B1), A5=SEQUENCE(3) spilling A5:A7. B1=2 -> A1:A2 (no collision).
- **Event:** B1 2->6; A1 wants A1:A6 overlapping A5's anchor/spill.
- **Excel:** Anchor-position dependent: A1's extent reaching A5 collides with the already-anchored A5, so A1=#SPILL!; A5 keeps spilling A5:A7. The encroaching array loses; the pre-existing anchored array is preserved.
- **Consequence:** Blockage is evaluated against the CURRENT valuation's spill_facts; the spill-repair fixpoint (repair_optimized_spills_with_oxfml, up to formula_cells passes) resolves mutual/competing spills — this IS calc-time dependency-order rework. blocked_formula_spill_extent_contains_anchor guards against an anchor sitting inside another formula's blocked extent. Iteration order + fixpoint determine winner/loser. Covers SPILL-07.

#### DAS-SPILL-HASH - Spill-range reference A2# tracks the live extent (grow/shrink invisibly to authored text)  _(reference-resolution)_
- **Setup:** A2=UNIQUE(C2:C100) spills A2:A8. F1=SUM(A2#).
- **Event:** C-data changes so UNIQUE yields 12 distinct values; A2 spills A2:A13.
- **Excel:** F1=SUM(A2#) automatically sums A2:A13 (the live extent) with no edit to F1. The # operator resolves at calc time against the current spill extent, not a frozen range.
- **Consequence:** A2# is a SpillAnchorRef. spill_rect() strips '#', resolves the anchor address, then looks up spill_extents (the committed ledger) — resolution is to the LIVE committed extent. The resolved identity is the GridSpillEpochSnapshot; an extent change makes before!=after -> Reclassified, dirtying F1 via dirty_closure_for_spill_fact(A2). F1's authored text is unchanged; only the resolved extent moves.

#### DAS-SPILL-DELANCHOR - Spill anchor deleted; A2# references go #REF!  _(reference-resolution|invalidation|dependency-order-rework)_
- **Setup:** A2=SEQUENCE(5) spilling A2:A6. F1=SUM(A2#).
- **Event:** User deletes/clears the formula in anchor A2.
- **Excel:** The entire spill A2:A6 disappears (ghosts cleared); F1=SUM(A2#) becomes #REF! (the spill range no longer exists). This is #REF!, not #SPILL!.
- **Consequence:** Spill epoch Removed -> resolve_spill_anchor_claim status Released, error_effect Ref. dirty_closure_for_spill_fact seeds from before-anchor A2 (after is None). spill_rect returns UnresolvedReference -> #REF!. The anchor removal drops the GridSpillFact (anchors_removed). Covers SPILL-09.

#### DAS-SPILL-AXIS - Spill anchor moved by a row/column insert (axis edit) during a rebind  _(reference-resolution|dependency-order-rework)_
- **Setup:** A2=SEQUENCE(3) at A2:A4. F1=SUM(A2#). User inserts a new row above row 2.
- **Event:** Row insert shifts the anchor A2->A3 (extent A3:A5); recalc.
- **Excel:** Excel re-targets A2# in F1 to the moved anchor; F1 keeps summing the live extent at its new location.
- **Consequence:** SHARP EDGE (AxisEditDuringRebind): apply_axis_edit only REWRITES addresses (transform_dependency_for_axis_edit rebuilds the GridInvalidationRef) — it does NOT re-resolve the dynamic target; only from-scratch re-resolution detects divergence. The spill ledger is re-keyed (overlays.spill_facts redistributed; spill_value_fingerprints transformed; refresh_spill_epoch_ledger). OPEN: an insert INSIDE a live extent — grow by a blank row, push down, or break? See uncertainty UNC-SPILL-INTERIOR-INSERT. Same cross-cutting cause as IOV-OFF-06, STBL-21, DOR-11, NCW-3D-02.

#### DAS-SPILL-AT - Implicit intersection @ collapses a would-be array to a single caller-aligned value  _(reference-resolution)_
- **Setup:** B1=@A1:A10 or =@SEQUENCE(10); caller at row 1.
- **Event:** Recalc evaluates the @-wrapped operand.
- **Excel:** @ implicit intersection: single-column picks the caller's row; single-row picks the caller's column; 1x1 passthrough; array operand takes top-left; genuine 2D reference -> #VALUE! (current OxFunc baseline). Creates NO spill.
- **Consequence:** @ is a scalarizing operator (OP_IMPLICIT_INTERSECTION) — the dependency is to the SELECTED single cell, a normal scalar edge, not a range/spill dep. If the caller moves (axis edit) the intersection cell changes -> re-resolution needed. A SpillAnchor operand under @ resolves the live extent then takes top-left. OPEN: does modern Excel ever spill/intersect a 2D ref differently vs #VALUE!? See UNC-AT-2D. Covers SPILL-11, RICH-13.

#### DAS-CSE - Legacy CSE array formula vs dynamic array (fixed extent, no spill)  _(reference-resolution|invalidation)_
- **Setup:** A1:A3 = {=B1:B3*2} CSE (3 committed cells) vs C1==B1:B3*2 dynamic (spills C1:C3).
- **Event:** B1:B3 values change; recalc.
- **Excel:** CSE recomputes IN PLACE over its fixed pre-allocated extent, never auto-resizes, never raises #SPILL! (too-large truncated, too-small #N/A-padded). CSE cells are all committed formula cells, not ghost cells. Modern Excel auto-converts a new single-cell array entry to dynamic.
- **Consequence:** CSE has a STATIC extent: no spill epoch, no anchor/ghost distinction — behaves like N committed formula cells sharing one expression (fixed Range dependents). Dynamic arrays have a live extent + ghost cells + SpillFact + epoch ledger. Two different dependency models; matters for round-trip fidelity. GAP: confirm OxCalc handling of explicitly-CSE-entered formulas. Version-sensitive.

#### DAS-SPILL-GHOST - Ghost cells are read-only; forcing a value into one breaks the spill  _(invalidation|dependency-order-rework)_
- **Setup:** A1=SEQUENCE(3) spilling A1:A3; A2,A3 are ghost cells.
- **Event:** User forces a literal into ghost cell A2 (e.g. paste).
- **Excel:** The spill becomes blocked: A1=#SPILL! and A3 reverts to empty. Ghost cells cannot independently hold formulas/values while the spill is live.
- **Consequence:** Writing into a ghost cell flips the anchor's blocked epoch (BlockedChanged). The authored A2 becomes a SpillBlocker for A1; spill_blocker_dependents_by_cell[A2] must include A1 so the edit dirties the anchor. The value engine re-probes on next recalc, publishes #SPILL!, and clears the previously-published ghost A3. Covers SPILL-13.

#### DAS-SPILL-VALUEONLY - Spill value-only change (same extent) -> UpstreamPublication, no re-topologize  _(invalidation)_
- **Setup:** A1=SORT(C1:C5) spilling A1:A5 (or =SEQUENCE(3)*B1). G1=INDEX(A1#,3).
- **Event:** A value in C1:C5 changes (SORT reorders) or B1 1->2; result is still 5 rows / 3 rows.
- **Excel:** Same extent; cell contents change; G1 recomputes against the new contents. No topology change.
- **Consequence:** Spill epoch ValueChanged ONLY -> cause_to_reason_table {UpstreamPublication}, kind {DynamicPotential} — NO ShapeTopology/StructuralRecalcOnly (reserved for extent changes). value_fingerprint differs so the ledger bumps value_epoch. Cheapest non-trivial transition: re-publish same-shape payload, dirty consumers, no extent rework. Covers SPILL-15, DOR-18.

#### DAS-SPILL-EXTENTVALUE - ExtentAndValueChanged in one recalc (shape and values move together)  _(invalidation|dependency-order-rework)_
- **Setup:** A1=SEQUENCE(B1)*C1 with B1 (height) AND C1 (scale) both edited in one batch.
- **Event:** Single recalc: extent grows AND every value changes.
- **Excel:** New larger extent published with new values; both shape- and value-dependents re-fire.
- **Consequence:** ExtentAndValueChanged folds into the SAME reason set as ExtentChanged (StructuralRecalcOnly + DynamicDependencyReclassified; kinds DynamicPotential+ShapeTopology) — the value change is subsumed by the structural change, NOT given a separate UpstreamPublication. structural_change is None (the anchor did not move, so no AnchorAdded/Removed). Verify spill_epoch_change_kind precedence yields ExtentAndValueChanged (not ExtentChanged-wins-drops-value) when both differ. Covers DOR-19.

#### DAS-SPILL-CASCADE - Cascading spill: A2# feeds another spilling formula (chained extents resize together)  _(dependency-order-rework)_
- **Setup:** A2=SEQUENCE(B1) spilling; D2=A2#*2 (itself spilling D2:D{n}).
- **Event:** B1 changes, resizing A2; D2 must re-spill to the matching new size.
- **Excel:** D2's extent follows A2's live extent; an A2 resize cascades to a D2 resize, which may collide downstream and raise #SPILL!.
- **Consequence:** Multi-level dependency-order rework: D2 cannot be sized until A2's new extent is committed. The spill-repair FIXPOINT (bounded by formula_cells passes) re-converges chained spills — each pass re-reads valuation.spill_facts so A2's new extent is visible to D2's blockage probe next pass. The fixpoint, not a topological pre-sort, handles the mid-calc dependency mutation. Covers SPILL-19, DOR-21 (table-grows-into-spill cascade).

#### DAS-SPILL-NOCONVERGE - Volatile-size spill never stabilizes -> Excel #SPILL! (volatile size); OxCalc gap  _(dependency-order-rework|invalidation)_
- **Setup:** A1 spills a dynamic array whose SIZE depends on a cell its own spill overwrites/feeds (oscillating size).
- **Event:** Recalc: spill-repair iterates; extent changes every pass and does not converge.
- **Excel:** Excel resolves the anchor to #SPILL! with the 'volatile size' cause when the size keeps changing and does not stabilize across additional passes.
- **Consequence:** OxCalc bounds the repair loop at formula_cells passes and sets spill_repair_converged=false if it never stabilizes (calc_ref_sheet.rs:733-751) but does NOT then force #SPILL! (volatile size) on the non-converged anchors — it stops with whatever the last pass produced. GAP vs Excel's explicit volatile-size #SPILL!. Distinct from the iterative-calc cap (DOR-13): the fixpoint bound is a termination guard, not an error producer. Covers DOR-04.

#### DAS-SPILL-VOLATILE - RANDARRAY/volatile spiller re-fires every recalc (value-epoch treadmill)  _(invalidation)_
- **Setup:** A1=RANDARRAY(3) spilling A1:A3. G1=SUM(A1#).
- **Event:** Any full recalc (F9).
- **Excel:** A1 produces fresh random values every recalc (same 3-row extent unless its size arg changes); G1 recomputes each time.
- **Consequence:** Volatility unconditionally re-evaluates the anchor each recalc; the spill epoch sees ValueChanged every recalc (value_fingerprint differs) -> UpstreamPublication dirties A1# consumers each pass. Extent stable so no ShapeTopology. A VALUE-epoch-only treadmill: cheap shape-wise, but the ledger never reaches epochs_preserved for a volatile anchor. WARM-NO-OP UNSOUNDNESS: the fast path captures only authored state and would WRONGLY return the cached valuation for a volatile formula whose precedents are byte-identical. Couples with IOV-VOL-01, DOR-08, DOR-14. Covers SPILL-17.

#### DAS-SPILL-ANCHORMOVE - Distinct before/after anchors (spill anchor relocates) union both closures  _(dependency-order-rework|reference-resolution)_
- **Setup:** A spill whose anchor moves A1->B1 across recalc, with separate dependents on each anchor.
- **Event:** Recalc: before-anchor A1 and after-anchor B1 differ; extent also changes.
- **Excel:** Dependents of the OLD anchor (now stale) AND the NEW anchor must both recalc.
- **Consequence:** resolve_spill_anchor_claim builds {before.anchor, after.anchor} and unions their closures (test spill_distinct_before_after_anchors_union_both_closures). UNDER-DIRTY EDGE: spill_epoch_change_kind keys on extent/value/blocked and IGNORES anchor identity, so a pure anchor move with identical extent/value/blocked yields no change-kind -> ReferencePreserving -> empty closure. Decide whether Excel can produce this case; if reachable, add anchor identity to the change-kind or seed both anchors unconditionally. Covers DOR-20.

#### DOR-RANGE-OVERLAP - Plain range A1:A10 overlapping a growing spill must be dirtied cell-by-cell (not via spill-fact edge)  _(invalidation|dependency-order-rework)_
- **Setup:** A1 spills A1:A3 -> A1:A5. F1=SUM(A1:A10) uses a PLAIN range overlapping the spill, NOT A1#.
- **Event:** Spill grows A1:A3 -> A1:A5; A4,A5 change blank->value within F1's range.
- **Excel:** F1 recalcs because A4,A5 changed within its range. Excel dirties range-overlap dependents on any cell whose value changed.
- **Consequence:** This is WHY anchor-only spill closures (DAS-SPILL-GROW/SHRINK) are insufficient: F1 has a compressed_range edge (A1:A10), NOT a spill-fact edge. The seam WOULD catch F1 IF the newly-spilled cells A4/A5 were fed as Range/Cell dirty seeds into close_over_dependents (which checks compressed_range_dependents_containing). KEY DESIGN RECOMMENDATION: on a spill extent change, seed the CHANGED CELLS (extent delta) as cell-dirty seeds, THEN close over BOTH spill-fact and compressed-range dependents. Today nothing does this seeding. Covers DOR-24.

### names-cross-workspace (13)

#### NCW-NAME-01 - OFFSET-based dynamic named range resolves (and is volatile) at calc time  _(reference-resolution)_
- **Setup:** Workbook name Data=OFFSET($A$1,0,0,COUNTA($A:$A),1). B1=SUM(Data). Column A has 5 contiguous values.
- **Event:** F9 or any edit (OFFSET is volatile, Data re-resolves every recalc).
- **Excel:** Data re-resolves to A1:A5; SUM=5-cell sum. The name's referred extent is a calc-time output of OFFSET, never a static rect bound at parse time. B1 recalcs even when no precedent changed.
- **Consequence:** CellDynamicRequest: B1 carries a DynamicRequest(request_key=name-handle), NOT a Range edge. A COUNTA-driven extent change is IdentityReclassified; the closure seeds by request key (parallel to spill-anchor seeding). GAP: OxCalc models name extent STATICALLY in GridNameDependency.extent; a dynamic OFFSET-name needs the DynamicRequest edge plus per-name volatility classification. Nothing re-runs OFFSET to detect divergence (only from-scratch re-resolution does). Same family as IOV-IND-01, IOV-OFF-03. Covers NAME-01.

#### NCW-NAME-02 - INDEX-based dynamic name is non-volatile (per-name volatility classification)  _(reference-resolution|invalidation)_
- **Setup:** Name Data=$A$2:INDEX($A:$A,COUNTA($A:$A)). B1=SUM(Data).
- **Event:** Edit an unrelated cell (no precedent of Data or COUNTA changed).
- **Excel:** B1 does NOT recalc — INDEX returning a reference is non-volatile, unlike OFFSET. B1 recalcs only when the A-column cells feeding COUNTA/INDEX actually change.
- **Consequence:** The dynamic name re-resolves its extent when its OWN precedents (the A column) change, but must NOT be in the volatile/recalc-always set. Distinguishing OFFSET (volatile) from INDEX (non-volatile) dynamic names is a per-name volatility classification driving whether the DynamicRequest is re-evaluated unconditionally. GAP: OxCalc has no volatility flag on name resolution. Same idiom as IOV-IDX-02. Covers NAME-02.

#### NCW-NAME-04 - Name redefinition (Refers To changed) re-seeds all dependents  _(invalidation|dependency-order-rework)_
- **Setup:** Name Rate -> $B$1; many cells =x*Rate. User edits the name so Rate -> $B$2.
- **Event:** Committing the redefinition (itself a recalc trigger).
- **Excel:** All cells using Rate recalc against $B$2. Adding/editing/deleting a defined name is itself a recalc trigger.
- **Consequence:** A Name-edge retarget. GAP: no redefine_defined_name(name,new_extent) on GridInvalidationRef — only rename/delete (resize_table exists for tables but no name analogue). The closure must be dirty_closure_for_name(name) (old dependents), the Name dep's extent rewritten to the new rect, and downstream scalar/range reverse-edges rebuilt because the underlying cells changed. Covers NAME-04.

#### NCW-NAME-05 - Name deletion -> #NAME? (NOT #REF!); error-effect routing  _(invalidation|reference-resolution)_
- **Setup:** Name Rate exists; A1=Rate*2, A2=A1+1. Also C1=SUM(MyRange) where MyRange is a name. Delete the name.
- **Event:** Delete the defined name; recalc.
- **Excel:** A1=#NAME? (Excel does NOT rewrite the formula text); #NAME? propagates (A2=#NAME?). This is #NAME? (unknown/deleted name), DISTINCT from grid range-delete -> #REF!. Per MEMORY: tree refs to deleted NAMES -> #NAME? + COMMIT; grid range delete -> #REF!.
- **Consequence:** delete_defined_name computes dirty_closure_for_name_keys([name_key]) first, then purges name edges (correct ordering). BUG/GAP: resolve_cell_dynamic_request_claim maps Released -> error_effect Ref (#REF!), wrong for a deleted NAME. DynamicRebindErrorEffect::NameOrValue EXISTS (rebind.rs:97) but is NOT emitted by the cell/spill resolvers. RESOLUTION: split Release error effect by family/cause — deleted/unknown NAME -> NameOrValue/#NAME?; deleted CELL/RANGE or collapsed external link -> Ref/#REF!. The optimized engine's textual formula rewrite (transform_sparse_point_formulas_for_defined_name_delete) and the seam closure must AGREE. Covers NAME-05, DOR-10, and resolves UNC-NAME-RELEASE-EFFECT.

#### NCW-NAME-SCOPE - Worksheet-scope name shadows workbook-scope of same spelling; adding a shadow flips resolution  _(reference-resolution|dependency-order-rework)_
- **Setup:** Workbook Tax->Sheet1!$Z$1(0.1); Sheet2-scope Tax->Sheet2!$Z$1(0.2). =Price*Tax on Sheet2. Also Sheet2-scope Region referenced cross-sheet as Sheet2!Region. Trigger: add a Sheet2-scope Tax where none existed.
- **Event:** Recalc of the Sheet2 formula; or committing a newly-shadowing sheet-scope name.
- **Excel:** On Sheet2 the worksheet-scope Tax (0.2) wins — narrower scope shadows workbook scope FOR FORMULAS ON THAT SHEET. The identical formula on Sheet1 resolves the workbook Tax (0.1). An unqualified sheet-scope name is invisible from another sheet (#NAME? without the Sheet! qualifier). Creating a sheet-scope shadow re-binds all that sheet's consumers to the new name and recalcs them.
- **Consequence:** The name_key MUST encode scope (sheet vs workbook) AND the resolving sheet; the same spelling on two sheets must NOT share a dependent set. GAP: excel_grid_defined_name_key(name,bounds) is scope-BLIND. Adding a shadow is a calc-time rebind of resolution identity (IdentityReclassified) PLUS a graph rework moving reverse edges workbook-key -> sheet-key. OxFml NameRef already carries sheet_id + caller_context_dependent — the scope-aware keying exists at the bind layer but is collapsed by OxCalc's (name,bounds) grid key. Covers NAME-06, NAME-07, NAME-08.

#### NCW-NAME-KIND - Name resolves to a value (constant) vs a reference: edge or no edge  _(reference-resolution)_
- **Setup:** Name Pi==3.14159 (value-like, no cell); Name Col==Sheet1!$A:$A (reference-like). A1=Pi*2, B1=COUNTA(Col).
- **Event:** Recalc.
- **Excel:** Pi yields the constant (no cell to depend on); Col yields a reference COUNTA enumerates. A value-like name has no precedent edge; a reference-like name does.
- **Consequence:** NameKind (ValueLike vs ReferenceLike) decides whether a dependent gets a precedent edge at all. A value-like name only re-evaluates when the NAME DEFINITION changes (a namespace edit), never via a cell precedent. Misclassifying ValueLike as ReferenceLike creates phantom edges; the reverse misses invalidation. OxFml already distinguishes NameKind::ValueLike/ReferenceLike/MixedOrDeferred/HelperLocal; the calc-graph consequence (edge or no edge) is the design point. Covers NAME-09, XWB-05 (foreign-namespace composition).

#### NCW-3D-01 - 3D reference span: per-sheet edges; sheet insert/delete reworks membership at calc time  _(reference-resolution|dependency-order-rework|invalidation)_
- **Setup:** =SUM(Jan:Mar!B2), sheets ordered Jan,Feb,Mar. Triggers: edit Feb!B2; drag a new sheet FebB between Feb and Mar; delete an endpoint Mar.
- **Event:** Edit a member cell; or a sheet-axis structural edit (insert/move/delete within or at the endpoints).
- **Excel:** SUM operates over the contiguous span by TAB ORDER between the endpoints. Editing any member's B2 recalcs. Inserting/moving a sheet between endpoints ABSORBS it into the span (SUM picks up FebB!B2). Deleting an interior sheet shrinks the span; deleting BOTH endpoints (or the only referenced sheet) -> #REF!.
- **Consequence:** The 3D span is a SET of per-sheet Cell edges; span membership is a STRUCTURAL input. Sheet insert between endpoints = DependencyAdded (new Cell edge); interior delete = DependencyRemoved; full collapse = Released/#REF!. GAP: apply_axis_edit handles Row/Column axes only — there is NO sheet-axis edit that reworks 3D-span membership. The span->member expansion (which sheets fall between Jan and Mar) is unmodelled. OPEN UNC-3D-ENDPOINT (endpoint-collapse rule). Covers 3D-01, 3D-02, 3D-03.

#### NCW-XWB-CACHE - Cross-workbook reference: closed source served from cache; missing+uncached -> #REF!  _(reference-resolution|invalidation)_
- **Setup:** A1='C:\[Sales.xlsx]Q1'!$B$2 with Sales.xlsx closed; the link table caches 42. Contrast A1='C:\[Missing.xlsx]S'!$B$2 with no source and no cache.
- **Event:** Recalc while the source is closed / missing.
- **Excel:** Cached: A1=42 (Excel serves the hidden link-table value silently; no error merely because the source is closed). Missing AND uncached: #REF!. The difference is whether the link table holds a value.
- **Consequence:** The external value is a host-supplied input keyed by an availability_version. A recalc with the source closed reads the cache and is NOT dirtied by the source's unknown live changes (WorkspaceQualifiedTarget, Unavailable-but-cached: resolution succeeds, no rebind). Missing+uncached -> WorkspaceUnavailable/WorkspaceProviderMissing -> Released-equivalent #REF!. OxCalc TreeCalcCrossWorkspaceAvailabilityStatus has Available/Unavailable/Degraded but no explicit 'Unavailable-served-from-cache' state. resolve_treecalc_workspace_host_path_base returns WorkspaceUnavailable for missing. Covers XWB-01, XWB-04. Resolved by UNC-XWB-STALE-CACHE.

#### NCW-XWB-AVAIL - Source workbook open/close flips availability_version (recalc-on-availability)  _(invalidation|dependency-order-rework)_
- **Setup:** A1='C:\[Sales.xlsx]Q1'!$B$2 cached 42; real on-disk value now 99. Open then close Sales.xlsx. Also XWB defined-name =Budget.xlsx!AnnualTotal.
- **Event:** Open Sales.xlsx (Unavailable/Cached -> Available), then close (Available -> Unavailable/Cached).
- **Excel:** Open: Excel refreshes the link, A1=99 (gated by the update-links prompt / automatic-update setting). Close: A1 keeps 99 (now cached); closing does NOT, by itself, dirty dependents (the last live value becomes the cache). An external defined name resolves in the TARGET workbook's namespace.
- **Consequence:** availability_version flips; the WorkspaceQualified reverse-edge fires: all cells with a workspace_target on that handle are dirtied -> from-scratch re-resolution of every external-link consumer. structural_change=WorkspaceAvailabilityChanged. OxCalc already has workspace_reverse_edges keyed by target handle + availability_version. GAP: nothing converts an availability bump into an InvalidationSeed set (no glue from the availability packet to derive_invalidation_closure). XWB defined-name composes name-namespace + availability — neither composes today. Parallels STBL-XWS. OPEN UNC-XWB-OPEN-GATE. Covers XWB-02, XWB-03, XWB-05.

#### NCW-XWB-ALIAS - Workspace alias re-points to a different workbook mid-session  _(dependency-order-rework)_
- **Setup:** An alias selector maps to workspace handle wsA; cross-workspace refs resolve through the alias.
- **Event:** WorkspaceAliasMutation: the alias re-pointed to wsB.
- **Excel:** References through the alias now resolve into wsB's namespace; targets in wsA-not-wsB break (#REF!); targets in both rebind to wsB's identity.
- **Consequence:** Alias mutation is a host-namespace version bump: every consumer of the alias re-resolves (IdentityReclassified or Released) — dependency-order rework because resolved target node IDs change wholesale. OxCalc has WorkspaceAliasMutation in the lifecycle set and add_alias on the registry, but the registry is immutable per resolution; no mid-session re-key of existing edges. Same trigger that produces STBL-DYNTARGET(b) byte-identical-key Reclassified. Covers XWB-06.

#### NCW-RTD-01 - RTD push / WEBSERVICE volatile external fetch seeds dependents externally  _(invalidation)_
- **Setup:** A1=RTD("prog",,"TICKER"); B1=A1*Shares. Also A1=WEBSERVICE(url); B1=FILTERXML(A1,xpath).
- **Event:** RTD server pushes a new topic value (async, no edit); OR F9 (WEBSERVICE volatile re-fetch).
- **Excel:** RTD: A1 updates to the pushed value and B1 recalcs — a topic update is an externally-driven recalc seed independent of any cell edit. WEBSERVICE: re-fetches on every recalc (volatile), returns payload or #VALUE!/#N/A on failure; B1 re-parses.
- **Consequence:** ExternallyInvalidated seed on A1 -> UpstreamPublication to B1 (the scalar reverse-edge). OxCalc has InvalidationReasonKind::ExternallyInvalidated mapped to NodeCalcState::Needed — exactly the RTD-push seed — but NO producer emits it from an RTD/host packet. WEBSERVICE is structurally an external volatile leaf (recalc-always set, like OFFSET/INDIRECT) that re-fetches on the recalc clock rather than async-pushes; non-deterministic from the graph's view. Couples with IOV-VOL-01, RICH-REFRESH. Covers RTD-01, RTD-04. OPEN UNC-WEBSERVICE-GRANULARITY.

#### NCW-RTD-STATE - RTD topic lifecycle: NoValueYet (#N/A Getting Data) then value; connection lost / provider error  _(reference-resolution|invalidation)_
- **Setup:** A1=RTD(...) connected but topic not delivered; later delivered; later server disconnects / ProviderError / CapabilityDenied.
- **Event:** First recalc after entry; an async push; a disconnect.
- **Excel:** NoValueYet -> #N/A (Getting Data) until the first push, then recalcs to the value. ConnectionFailed -> error/last-value-then-error per the COM RTD contract; ProviderError -> the host-provided code; CapabilityDenied (RTD disabled) -> typically #N/A or blocked.
- **Consequence:** NoValueYet is a transient state that must transition to a value on a LATER async seed without a user edit — the dependency must remain 'pending external' so the eventual push re-seeds it (the calc chain must re-enqueue across recalcs; warm-no-op must not finalize it). The value-to-error transition is an ExternallyInvalidated seed. Error codes are host-provided, not derivable from the formula. OxCalc has MinimalRtdMode covering all states but only as single-formula recalc inputs, not graph seeds. Mirrors RICH-BUSY for linked data. Covers RTD-02, RTD-03.

#### NCW-PQ-01 - Power Query / external query refresh as an external seed plus a structural resize  _(invalidation|dependency-order-rework)_
- **Setup:** A table loaded by Power Query; formulas reference the loaded range/table columns.
- **Event:** Query refresh (manual/scheduled) replaces the loaded data, possibly resizing the output range/table.
- **Excel:** On refresh the loaded range/table is rewritten; dependents recalc. A resize gives structured references and spill consumers a new extent (rows added/removed).
- **Consequence:** Refresh is an external seed PLUS a potential structural (extent/topology) change — a StructuredTableRebind with TableExtentChanged when the table resizes, layered on ExternallyInvalidated for the value change. Dependents of added rows enter the closure via row-membership; removed rows drop edges. Maps to TreeCalcTableUpdateScenarioKind RowInsert/RowDelete/TableResize combined with external invalidation. The structured-table classifier covers the resize half; the external-trigger half is the new seam. Combines STBL-AUTOEXPAND (resize) with NCW-RTD-01 (external seed). Covers PQ-01.

### rich-objects (9)

#### RICH-FIELD-DOT - Dotted field access on an entity cell (=A2.Price)  _(reference-resolution)_
- **Setup:** A2 holds a Stocks linked-entity (EntityCellValue with property Price). B2=A2.Price.
- **Event:** B2 evaluated; the .Price selector resolves at calc time against the rich object in A2.
- **Excel:** B2 returns the scalar Price (a number, possibly with a number-format). The dot selector is a case-insensitive display-name property lookup; spaces require bracket form (A2.[52 Week High]). Resolves the entity's properties map; not a function call.
- **Consequence:** B2 depends on A2 (cell) AND on the field identity 'Price' inside A2's entity. Field-level dependency is finer than cell, but Excel invalidates at CELL granularity (any entity change dirties all dotted readers). GAP: no dotted-access evaluator in OxCalc; no FieldAccess dependency variant. OxFunc models the entity as RichObjectValue.kvps keyed by String; selection is a kvp lookup by key.

#### RICH-FIELD-MISS - Dotted/FIELDVALUE access to a nonexistent field or a non-data-type cell -> #FIELD!  _(reference-resolution)_
- **Setup:** A2 holds a Stocks entity (or a plain number 42). B2=A2.Nonexistent or =FIELDVALUE(A2,"Nonexistent"); or B2=A2.Price where A2=42.
- **Event:** Field selector resolves against the entity's property map (key absent) or against a cell with no rich object.
- **Excel:** #FIELD!. Three documented causes: (a) field does not exist for the data type, (b) the referenced value is not a data type, (c) the online service has no data for that value+field. (a)/(b) deterministic; (c) external/refresh-dependent. Distinct from #VALUE!.
- **Consequence:** #FIELD! still depends on A2 (cell) and the field-name argument. If A2 later becomes a data type (Activated, CTRO) or a refresh adds the field, B2 must re-resolve from #FIELD! to a value — a calc-time identity transition keyed on A2's value-tag (scalar vs rich), not just its content. WorksheetErrorCode::Field exists in OxFunc; no producer emits it. Distinguishing (a) vs (c) needs entity schema (RichObjectType.required_keys) vs live kvps. Covers RICH-02, RICH-15, RICH-08(reverse).

#### RICH-FIELD-DYNAMIC - Field name supplied by a cell reference (FIELDVALUE(A2,B1)) is a cell-driven dynamic request  _(reference-resolution|dependency-order-rework)_
- **Setup:** A2 is a Geography entity; B1 holds text 'Population'. C2=FIELDVALUE(A2,B1).
- **Event:** FIELDVALUE reads B1 at calc time to pick WHICH property of A2 to extract.
- **Excel:** C2 returns A2's Population. B1 edited to 'Area' -> C2=A2.Area; B1 a non-field string -> #FIELD!. The dynamic, cell-driven field selection that dot syntax cannot do.
- **Consequence:** C2 has TWO dependencies jointly determining the resolved target: the entity in A2 and the selector text in B1. A change to B1 is a calc-time reference-resolution change (the request key 'A2.field=<B1 value>' changes) with no structural edit — a CellDynamicRequest analog. Maps onto GridDependency::DynamicRequest(String) but that graph is unwired and no request-key derivation exists for FIELDVALUE. Covers RICH-03.

#### RICH-COERCE - Implicit rich-to-scalar coercion uses basicValue fallback  _(reference-resolution)_
- **Setup:** A2 holds an IMAGE rich value (fallback=published text) or a Stocks/Geography entity (basicValue typically #VALUE!). B2=A2&"" or =A2+0.
- **Event:** An operator/function expecting a scalar receives a rich CalcValue and coerces to its core/basicValue at calc time.
- **Excel:** Excel substitutes the entity's basicValue. IMAGE fallback is the published string; Stocks/Geography basicValue is #VALUE! (so =A2+0 on Geography gives #VALUE!; TEXT(A2) on a linked entity gives #VALUE!; workaround FIELDVALUE(A2,"Name")).
- **Consequence:** Coercion reads only the core/fallback, so a refresh changing entity PROPERTIES but not the fallback should not change a coercing dependent's value — yet at cell granularity Excel still dirties it (warm-no-op opportunity). The rich-value analog of the spill-fingerprint: a value-level dependency observing only part of the rich object. OxFunc CalcValue keeps core + Option<rich>; coercion paths drop rich and read core. Covers RICH-04, RICH-16.

#### RICH-SPILL-FIELD - FIELDVALUE over a range of data-type cells, or an array-valued field, spills  _(invalidation|dependency-order-rework)_
- **Setup:** A2:A11 are ten Stocks entities; C2=FIELDVALUE(A2:A11,"Price"). OR A2 entity whose History property is an ArrayCellValue; B2=A2.History.
- **Event:** FIELDVALUE broadcasts the field extraction (10x1 spill) or extracts an array property that spills from B2.
- **Excel:** A 10x1 spill of Prices anchored at C2 (#SPILL! if target occupied). Per-element missing-field -> per-element #FIELD! within the spill, not whole-array failure. An array property spills as a dynamic array; blocked -> #SPILL!; nested/unsupported array -> #CALC!.
- **Consequence:** A single cell A2 owns a rich-value dependency AND, transitively through B2, a spill anchor whose extent is determined by entity content. A refresh that resizes History changes B2's extent -> ExtentChanged epoch (subject to the seed-by-anchor sharp edge, DAS-SPILL-GROW). The value_fingerprint must capture the EXTRACTED field, not the whole entity, to be precise. Field extraction converts a cell dependency into a spill-extent dependency mid-calc. Covers RICH-05, RICH-09.

#### RICH-REFRESH - Linked-data REFRESH mutates entity fields externally (no formula change)  _(invalidation)_
- **Setup:** A2 is a Stocks entity; B2=A2.Price. User triggers Data Type > Refresh (or workbook recalc with a stale cache).
- **Event:** External provider re-resolves the entity; A2's entity object is replaced with new property values at calc time, outside formula evaluation.
- **Excel:** B2 recomputes to the new Price. Refresh updates the selected cell PLUS all cells of the SAME data type. A refresh mutates stored cell values externally (no formula change) and must dirty all dotted/FIELDVALUE readers and downstream. The rich-value analog of an RTD tick.
- **Consequence:** A calc-time invalidation with NO structural edit and NO formula change — the source cell's stored value mutates externally; an external event the calc chain must accept as a dirtying input. OxCalc models the state machine via MinimalRtdMode + RtdProvider (upstream_host.rs) but RTD is a returned-value-surface DIAGNOSTIC, not wired into GridInvalidationRef; there is no RichRefresh/LinkedEntity dependency variant and no entity-schema-vs-live-kvps reclassification on refresh. Couples with NCW-RTD-01, RICH-FIELD-MISS (schema flip). Covers RICH-06, RICH-08.

#### RICH-BUSY - Refresh in-flight / connection failure: #BUSY!, #CONNECT!, #BLOCKED! propagation (non-terminal)  _(reference-resolution|invalidation)_
- **Setup:** A2 a stock whose provider has not returned (NoValueYet), or feed fails (ConnectionFailed), or query blocked (CapabilityDenied). B2=A2.Price, C2=B2*100.
- **Event:** Calc runs while A2 resolution is pending/failed.
- **Excel:** NoValueYet -> #BUSY!/GETTING_DATA (non-terminal, 'getting data from the web'); ConnectionFailed -> #CONNECT!; CapabilityDenied (empty stock arg, external data disabled) -> #BLOCKED!. Dependents propagate. #BUSY! must NOT be cached as the final answer; a later successful refresh must re-dirty and clear the error.
- **Consequence:** A pending/external-errored cell is perpetually dirty until the provider resolves — the calc chain must re-enqueue across recalcs rather than treat the first pass as authoritative. The engine must distinguish 'errored due to external state' (re-attemptable) from 'errored due to formula' (stable until edit). Warm-no-op must treat Busy/GettingData/Connect/Blocked as non-terminal. OxCalc maps NoValueYet -> NA, not Busy — a faithfulness gap (UNC-BUSY-NA). Covers RICH-07, RICH-14, overlaps NCW-RTD-02/03.

#### RICH-CARD - Card vs cell display and display-only fields must not dirty dependents  _(invalidation)_
- **Setup:** A2 is an entity; user opens the card vs leaves the cell collapsed. Or A2=IMAGE(...) whose _DisplayString is flagged ExcludeFromCalcComparison and a refresh changes only that string.
- **Event:** A recalc occurs / a refresh changes only a presentation field.
- **Excel:** Card vs cell is DISPLAY only; it never alters the calc value nor triggers recalc. A change only to a display string flagged ExcludeFromCalcComparison should NOT dirty dependents that consumed the identity; only calc-significant fields (e.g. WebImageIdentifier) count as value changes.
- **Consequence:** Opening a card must NOT mark dependents dirty: presentation state is outside the dependency graph. The rich-object value fingerprint must respect per-field calc-comparison flags (include identity keys, exclude _DisplayString) — a whole-object fingerprint over-invalidates; a fallback-only fingerprint under-invalidates. OxFunc has RichObjectType.key_flags with ExcludeFromCalcComparison; the spill_ledger value_fingerprint closure is rich-UNAWARE and would need to honor these flags. Covers RICH-11, RICH-12.

#### RICH-NESTED - Nested entity carrying its own referencedValues -> #VALUE! (root-only validity rule)  _(reference-resolution)_
- **Setup:** A custom function/add-in produces an entity where a NESTED entity defines its own referencedValues array.
- **Event:** Excel validates the cell-value tree when the value is produced.
- **Excel:** #VALUE! (GeneralException in the JS API). Only the ROOT entity may define referencedValues; nested entities reference the root's by ReferenceCellValue index.
- **Consequence:** A structural-validity rule on the rich value: an ill-formed entity collapses to #VALUE! at calc time; dependents see #VALUE!, not a rich object. OxFunc RichObjectData::Object(Box<RichObjectValue>) allows nesting but has no referencedValues concept and no root-only validation — a faithfulness gap if custom-function-produced entities are supported. Covers RICH-10.

### structured-tables (19)

#### STBL-THISROW - [@Col]/[#This Row] resolves current row via caller context; outside a data row -> #VALUE!  _(reference-resolution)_
- **Setup:** In Table1 column Total: =[@Qty]*[@Price]. Also a cell far outside the table body holding =Table1[@Price].
- **Event:** Recalc of the body cell (resolves caller row) and of the out-of-table cell (no caller row).
- **Excel:** Body cell: intersect the named column with the formula's physical row -> single current-row product; no spill. Out-of-table cell: #VALUE! (the implicit intersection has no current row to use).
- **Consequence:** Dependency on StructuredTableCallerContext + StructuredTableDataRegion for that row only. dynamic_table_target_fact_kinds(CurrentRow)={TableIdentity,RowOrder,DataRegion,CallerRowContext}. Missing caller -> MissingCallerContext/MissingCallerTableRegion diagnostic -> TypedExclusion (#VALUE!; oxfunc_opaque_reference_admitted=false). Distinct from STBL-WHOLECOL: @ never spills. Covers STBL-01, STBL-02.

#### STBL-WHOLECOL - Table[Col] whole-column reference outside the table spills; row append grows it  _(reference-resolution|invalidation)_
- **Setup:** Outside Table1: G1=Table1[Amount] (no @). Returns the whole column as an array and spills from G1.
- **Event:** A row appended to Table1 (edit-time auto-expand), then recalc.
- **Excel:** The G1 spill grows by one cell to match the new data extent; structured references auto-adjust without editing the formula.
- **Consequence:** Two-stage: (1) the row insert changes StructuredTableRowMembership+RowOrder+DataRegion -> dirty_closure_for_table(table_name) seeds G1; (2) G1's OWN spill ledger then records a SpillAnchorRef epoch (Added/ExtentChanged) on G1's anchor. rebind.rs: 'A table that grows into a spill is modelled as a separate SpillAnchorRef claim, never folded into this consequence.' The structured-table feeder dirties the consumer; the consumer's spill ledger records the extent epoch. Couples with DOR-21. OPEN UNC-WHOLECOL-LEGACY. Covers STBL-03.

#### STBL-AUTOEXPAND - Auto-expand / calculated-column fill is an EDIT-time reshape, not calc-time resolution  _(dependency-order-rework|invalidation)_
- **Setup:** Table1=A1:C5; user types in D2 (one col right) or A6 (one row below). Or a calc-column =[@Amount]*0.1 in every row; user edits one cell or inserts a row.
- **Event:** Typing/edit commits: Excel auto-expands the table extent and auto-fills the calc-column BEFORE the ensuing recalc.
- **Excel:** Table grows to include the adjacent cell; a new column gets a default name; a new row auto-receives the calc-column formula; existing Table[Col] refs see the larger extent. Auto-expand does NOT fire if the option is off, undone, or the adjacent cell is a spill range. An overriding constant breaks calc-column uniformity (exception) without erroring.
- **Consequence:** The reshape mutates the table extent in the graph (resize_table / GridTableDependency extent update) BEFORE the calc chain runs, then dirties dependents — dependency-order rework driven by an EDIT, not a mid-calc resolution. SHARP EDGE: resize_table rebuilds the whole GridInvalidationRef from transformed semantic deps (replace_semantic_dependencies); cell-level addresses of in-table formulas must ALSO be shifted (apply_axis_edit) — two distinct graph operations that must be ordered consistently. OPEN UNC-AUTOEXPAND-DIRTY (table-key vs address closure). BodyFormulaEdit fans out across every row of the column. Covers STBL-04, STBL-05.

#### STBL-COLDEL - Referenced column deleted -> #REF! at calc time  _(reference-resolution|invalidation)_
- **Setup:** H1=SUM(Table1[Commission]); another uses Table1[[#Headers],[Commission]].
- **Event:** Commission column deleted; recalc.
- **Excel:** Both -> #REF! (Excel rewrites the selector to Table1[#REF]). Header-targeting refs also break.
- **Consequence:** ColumnDelete -> changed kinds {ColumnIdentity,HeaderText,HeaderRegion,DataRegion}; validate_treecalc_table_reference_after_update emits MissingColumn for a selected_column_id absent from the surviving set -> StructuredTableLoweringBlocker::MissingSelectedColumn -> dependent recomputes to #REF! (error_effect Ref). Covers STBL-06.

#### STBL-TBLDEL - Whole table deleted -> #REF!; closure must seed OLD table key before purge  _(invalidation|dependency-order-rework)_
- **Setup:** H1=COUNTA(Table1[#Data]); other cells reference Table1.
- **Event:** Table1 deleted; recalc.
- **Excel:** All references to Table1 -> #REF!; the table identity is gone.
- **Consequence:** Critical ordering: delete_table dirties dirty_closure_for_table_keys([table_key]) FIRST, then transform drops the deps, else referrers never recompute to #REF!. The feeder mirrors this: status Released, after_name=None, closure seeded from before_name only. Per MEMORY: structured-table delete is the #REF! family (vs name delete -> #NAME?). Covers STBL-07, DOR-22.

#### STBL-RENAME - Table/column rename re-keys live references; values preserved (text INDIRECT does NOT track)  _(reference-resolution|invalidation)_
- **Setup:** Sales[Amount] refs; table Sales renamed Revenue. Or Table1[Qty] refs; column Qty renamed Quantity. Contrast: =SUM(INDIRECT("Sales[Amount]")).
- **Event:** Rename commits; recalc.
- **Excel:** Excel rewrites live structured refs (Sales[...]->Revenue[...], [Qty]->[Quantity]) automatically; values PRESERVED, no #REF!. INDIRECT-built text refs do NOT track the rename -> #REF! (the rename binding cannot reach a string literal).
- **Consequence:** TableRename: changed {Identity,EnclosingTable}, reasons {ContextChanged,StructuralRebindRequired}; rename_table re-keys table_dependents_by_key old->new and dirties union(old,new). In the dynamic feeder before_name!=after_name -> DynamicDependencyReclassified + structural_change=TableKeyChanged. ColumnRename also maps to TableKeyChanged (values unchanged, key re-bound). The INDIRECT-text case is a CellDynamicRequest (UnsupportedRuntimeStructuredReferenceParsing -> TypedExclusion + #REF!), same family as IOV-IND-04. Covers STBL-08, STBL-09, STBL-19.

#### STBL-ROWREORDER - Row reorder changes RowOrder not membership; order-sensitive formulas recompute  _(invalidation)_
- **Setup:** H1=INDEX(Table1[Amount],2) (positional); H2=SUM(Table1[Amount]) (order-insensitive).
- **Event:** Sort/reorder rows; recalc.
- **Excel:** INDEX(...,2) returns the value now in the 2nd data row (changed); SUM unchanged in value. Both recompute (data-region dependents).
- **Consequence:** RowReorder: changed {RowOrder,CallerContext,Identity}, reasons {RowOrderChanged,CallerContextChanged,ContextChanged}. RowMembership is NOT in the changed set — distinguishing reorder from insert/delete. Both H1,H2 dirtied (conservative); only H1 changes value. The membership-vs-order distinction. Covers STBL-10.

#### STBL-SECTION - Section selectors ([#Totals], [#Headers], [#All]) depend on section presence/region growth  _(reference-resolution|invalidation)_
- **Setup:** =Table1[[#Totals],[Amount]]; =ROWS(Table1[#All]); =SUM(Table1) (defaults to [#Data]).
- **Event:** Total Row toggled OFF (then a data row inserted); recalc.
- **Excel:** [#Totals] -> #REF! when the totals row is off (restored when toggled back). [#All] spans headers+data+totals; ROWS grows by 1 for totals and again for the inserted data row. SUM(Table1) (==[#Data]) ignores totals and grows only with data rows. [#Headers] breaks if the header row is removed (rare).
- **Consequence:** validate_treecalc_table_reference_after_update emits TotalsRowAbsent/HeaderRowAbsent -> blocker -> #REF!. dynamic_table_target_fact_kinds(Section)={TableIdentity,HeaderRegion,DataRegion,TotalsRegion}. TotalsRowToggle dirties via TotalsRegion; RowInsert via DataRegion+RowMembership. [#Data]-default and [#All] dependents have DIFFERENT changed-kind sensitivity to the same toggle. Covers STBL-11, STBL-20.

#### STBL-OVERSPILL - Table grows over a grid cell holding a spill anchor -> #SPILL! (blocked epoch flip)  _(invalidation|dependency-order-rework)_
- **Setup:** G1 (outside Table1)=SEQUENCE(5) spilling G1:G5. Table1 below is resized / a row inserted so the body overlaps G3.
- **Event:** Table resize/auto-expand pushes the body into the spill footprint; recalc.
- **Excel:** G1=#SPILL! — the spill output is blocked by table cells; the spill collapses to the anchor showing #SPILL!.
- **Consequence:** Two coupled effects: (a) the table extent change dirties table dependents; (b) G1's SpillAnchorRef epoch flips BlockedChanged(false->true). These are SEPARATE facts (GridSpillDependency(anchor=G1) blocked-epoch vs the table's DataRegion) with NO single seam coordinating them — must be coordinated by the (unbuilt) calc driver. Sharp edge: blocked-vs-value epoch divergence — extent did not change, only blocked flipped. The inverse of DAS-SPILL-TABLE (formula spilling INTO a table). Covers STBL-12.

#### STBL-OMITTED - Bare [Col] (omitted table name) rebinds to the enclosing table by physical location  _(reference-resolution)_
- **Setup:** Inside Table1, a body formula uses bare [Amount]; the OmittedTableName selector resolves to the enclosing table.
- **Event:** The cell is cut/pasted into a different table, or the enclosing table is restructured; recalc.
- **Excel:** Bare [Col] always means 'the table I am physically inside'. Moved into Table2 -> Table2[Col]; moved outside any table -> #NAME?/invalid. Enclosing-table-relative, resolved by physical location at calc time.
- **Consequence:** OmittedTableName -> dependency on StructuredTableEnclosingTable; changed-kind StructuredTableEnclosingTable. Resolver uses TreeCalcTableCatalogResolutionLayer::OmittedCallerTable + blocker OmittedTableEnclosingMismatch/MissingEnclosingTableContext when the enclosing context is gone. Covers STBL-14.

#### STBL-CROSSTABLE - Cross-table reference recomputes on the OTHER table's change (multiple table keys per cell)  _(invalidation)_
- **Setup:** Table1 column = SUMIF(Table2[Region],[@Region],Table2[Sales]).
- **Event:** Table2 gains a row (auto-expand); recalc.
- **Excel:** Every Table1 row's SUMIF recomputes because Table2's data region grew; results reflect the new Table2 row.
- **Consequence:** Table1 column cells carry table deps keyed on BOTH Table1 (enclosing, for [@Region]) and Table2 (cross-table data). A Table2 RowInsert dirties only via the Table2 key: dirty_closure_for_table('Table2') -> all Table1 rows. The closure must UNION both keys' dependents; table_dependents_by_key is per-key. Covers STBL-15.

#### STBL-XWS - Cross-workspace table: close -> UnavailableTarget; reopen -> rebind; alias remap -> reclassify  _(reference-resolution|invalidation)_
- **Setup:** Formula references [Book2]Sales[Amount] (CrossWorkspaceTable). Also a WorkspaceAliasMutation re-pointing an alias.
- **Event:** Book2 closed (then reopened); or alias re-pointed mid-session; recalc.
- **Excel:** Closed: last cached values served; a recalc needing live data may show stale or #REF! if the link is broken. Reopen: live binding restored, values refresh. Alias remap: refs resolve into the new namespace; targets in old-not-new break (#REF!).
- **Consequence:** CrossWorkspaceTable adds WorkspaceAvailability fact + HostSensitive changed-kind + StructuralRebindRequired + HostNamespaceVersion prepared-input. WorkspaceClose -> UnavailableTarget (error_effect Ref); WorkspaceOpen -> DependencyAdded + re-resolution; structural_change=WorkspaceAvailabilityChanged. Availability is a host-version-keyed dependency, NOT a cell dependency. Same seam as NCW-XWB-01..06. AxisEditDuringRebind: an availability flip DURING a rebind is the cross-cutting case. Covers STBL-16, DOR-25.

#### STBL-SAVEREOPEN - Save-reopen with byte-identical identity is ReferencePreserving (clears reasons+kinds+inputs)  _(invalidation)_
- **Setup:** A workbook with structured-table formulas is saved and reopened with no structural change.
- **Event:** SaveReopen lifecycle, recalc.
- **Excel:** Values preserved; no recalc-forced change to structured references (a full open-recalc may run but values are stable).
- **Consequence:** SaveReopen + before==after -> classify clears changed_dependency_kinds, invalidation_reasons AND prepared_identity_inputs (ReferencePreserving). SHARP EDGE: ReferencePreserving clears ALL THREE sets, while TypedExclusion clears ONLY dependency_fact_kinds and still flags #REF!. The two 'clear' paths are ASYMMETRIC and must not be conflated. OPEN UNC-PRESERVE-INPUTS (is clearing prepared_identity_inputs Excel-faithful or a modelling choice). Covers STBL-17, DOR-17.

#### STBL-DYNTARGET - Dynamic selector resolving to a non-table target, or whose resolved table moves under a stable key  _(reference-resolution|invalidation)_
- **Setup:** (a) A dynamic/INDIRECT selector that looks like a table selector resolves to a name/range that is not a table. (b) A selector whose text is byte-identical but whose RESOLVED table identity changes (tables swap names, alias remap).
- **Event:** Recalc re-resolves the selector.
- **Excel:** (a) Not an admissible table binding; Excel treats it as a name/range (or #NAME?/#REF! by shape). (b) The reference now points at a different physical table; values reflect the new target though the selector text never changed.
- **Consequence:** (a) cause=DynamicTargetNotTable -> Unresolved changed-kind + DependencyReclassified + TypedExclusion (no table ReferenceLike); dependency_fact_kinds cleared; oxfunc_opaque_reference_admitted=false. (b) classify inserts DynamicDependencyReclassified BECAUSE before_resolved_table_identity != after_resolved_table_identity, even though the selector-keyed dynamic_rebind_identity is STABLE — the SHARP EDGE: a Reclassified fires with a byte-identical request_key. Downstream invalidation must key on RESOLVED identity, not selector key. Same edge as IOV-IND-01, DOR-16. Covers STBL-18, STBL-22.

#### STBL-REGISTRY - Function-registry snapshot mutation reclassifies table-function admission (non-spatial axis)  _(invalidation)_
- **Setup:** A table formula uses a function whose admission depends on the registry snapshot (custom/LAMBDA). The OxFunc registry snapshot changes.
- **Event:** FunctionRegistrySnapshotMutation, recalc.
- **Excel:** Analogue: a function's availability/definition changing forces dependents to re-evaluate (or #NAME? if removed). Capability-sensitive.
- **Consequence:** FunctionRegistrySnapshotMutation: changed {CapabilitySensitive}, reason {DependencyReclassified}, prepared-input RegistrySnapshotIdentity. The dependency is keyed on the registry snapshot identity, NOT a cell — a non-spatial invalidation axis. StructuredTableDependencyFactKind::FunctionRegistrySnapshot -> CapabilitySensitive. Same descriptor family as CELL/INFO HostSensitive (IOV-VOL-01) and INDIRECT serialization (DOR-15). Covers STBL-23.

#### STBL-COLINSERT - Column insert/reorder shifts positional-column keys but not named-column refs  _(invalidation|dependency-order-rework)_
- **Setup:** H1=INDEX(Table1[#Data],2,3) (positional 3rd column); H2=Table1[Amount] (named). A new column inserted before Amount.
- **Event:** ColumnInsert, recalc.
- **Excel:** H2 (named) still points at Amount regardless of position — value unchanged. H1 (positional INDEX col 3) now hits a different physical column — value changes.
- **Consequence:** ColumnInsert: changed {ColumnIdentity,HeaderText,HeaderRegion,DataRegion,TotalsRegion,Identity}. Both dirtied; only positional changes value. The column-identity key remap (StructuredTableColumnIdentity) is the named-ref mechanism; positional access additionally rides the apply_axis_edit address-transform path on the table's internal column addresses. Covers STBL-24.

#### STBL-TABLEMOVE - Table move (relocation, same identity) shifts region addresses but preserves the name binding  _(dependency-order-rework)_
- **Setup:** External =SUM(Table1[Amount]); inside-table =[@Amount]. Table1 cut and moved to a new location.
- **Event:** Table move commits; recalc.
- **Excel:** Structured references work unchanged (name binding is location-independent); A1-style refs to the table's old cells shift. Values preserved.
- **Consequence:** TableMove: changed {Identity,HeaderRegion,DataRegion,TotalsRegion,EnclosingTable}, reasons {ContextChanged,StructuralRebindRequired}; structural_change=TableExtentChanged. Region REFS (addresses) change but the table KEY does not — apply_axis_edit-style address transform updates region anchors while table_dependents_by_key is untouched. Sharp edge: an axis/move edit rewrites addresses and never re-resolves the dynamic target; region refs vs table key diverge here. Covers STBL-21.

#### STBL-BODYEDIT - Body cell value edit dirties only the data region (cheapest table invalidation)  _(invalidation)_
- **Setup:** Table1 column Amount; H1=SUM(Table1[Amount]). User edits one Amount cell's value.
- **Event:** BodyCellEdit, recalc.
- **Excel:** H1 recomputes the new sum; table identity/shape/columns unchanged.
- **Consequence:** BodyCellEdit: changed {DataRegion} only, reason {RegionChanged} only, prepared_identity_inputs EMPTY. The minimal-invalidation contract: no identity re-key, no structural rebind; dirties the data-region dependents of that one cell/column. The benign baseline against which all other table scenarios add structural/identity work. Covers STBL-26.

#### STBL-MIDCALC - Mid-calc table shape change via VBA/iterative recalc (dependency graph mutates during calc)  _(dependency-order-rework|invalidation)_
- **Setup:** A formula's output feeds a process (macro/automation/iterative model) that changes a table's row count while other formulas reference that table within the same recalc pass.
- **Event:** During a single recalc, the table extent changes after some dependents were scheduled.
- **Excel:** Excel's calc chain is fixed for a pass; genuine mid-pass structural mutation comes from VBA re-entrancy (Worksheet_Change) or iterative calc. Excel re-marks dependents dirty and runs ADDITIONAL passes; it does NOT re-resolve a structured ref mid-cell or retroactively reorder an in-flight pass.
- **Consequence:** The true 'dependency graph mutating mid-calculation' case: a TableResize discovered after the chain was built must enqueue a re-pass (VolatileReevaluation cause), re-seeding dirty_closure_for_table. The rework cannot retroactively reorder already-evaluated cells; it must schedule another pass. GAP: GridInvalidationRef is not driven by the value engines, so multi-pass re-dirty from a mid-calc table reshape is unmodelled. OPEN UNC-MIDCALC-RERESOLVE. Covers STBL-25.

