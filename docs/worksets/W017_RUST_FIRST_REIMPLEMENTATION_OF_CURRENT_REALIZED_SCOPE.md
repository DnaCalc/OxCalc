# W017: Execution Sequence C - Rust-First Reimplementation of Current Realized Scope

## Purpose
Operationalize the implementation-direction change after `W015`.

This packet exists to:
1. move OxCalc implementation execution onto an idiomatic Rust footing,
2. preserve the current exercised .NET realization as semantic evidence, executable comparison surface, and parity reference,
3. ensure the Rust lane is driven by OxCalc specs, replay artifacts, oracle behavior, and baseline runs rather than by mechanical cross-language translation,
4. define the first parity-oriented Rust execution wave before deeper replay-appliance or retained-witness rollout continues.

## Position and Dependencies
- **Depends on**: W014, W015
- **Blocks**: W016
- **Cross-repo**: no new seam authority by default; any shared seam pressure discovered during Rust realization still routes through W005

## Scope
### In scope
1. Define the Rust-first implementation direction for OxCalc executable work.
2. Execute the first planning and sequencing packet for ab initio Rust realization of the currently exercised structural, coordinator, recalc, `TraceCalc`, and tool-host scope.
3. Declare the comparison role of the current .NET implementation and checked-in baseline runs.
4. Lock the Rust quality floor, crate-boundary direction, and validation requirements for the reimplementation wave.
5. Define parity, conformance, and evidence gates for Rust replacing the current .NET implementation as the active realization lane for the declared scope.

### Out of scope
1. Stage 2 concurrency realization.
2. New semantic widening beyond the already exercised W014/W015 scope.
3. Witness-distillation execution and retained-failure rollout.
4. Generic cross-language transliteration of current .NET code structure.
5. Immediate retirement of the .NET implementation before Rust parity evidence exists.

## Deliverables
1. A clear Rust-first realization doctrine and execution boundary for OxCalc.
2. An execution-sequenced reimplementation plan covering:
   - structural snapshot kernel,
   - coordinator/publication baseline,
   - recalc and overlay baseline,
   - `TraceCalc` scenario loading, runner, engine machine, reference machine, and tool host.
3. A validation plan that uses current replay artifacts, checked-in baseline runs, and engine-versus-oracle comparisons as the Rust parity floor.
4. A crate and evidence plan that makes the Rust implementation idiomatic rather than .NET-shaped.
5. A concrete execution sequence with per-phase entry and exit gates for the Rust realization wave.

## Gate Model
### Entry gate
- W014 has established the widened Stage 1 baseline and active normative run `w014-stage1-widening-baseline`.
- W015 has established replay-facing coherence, normalized event-family mapping, and adapter doctrine.
- The current .NET implementation remains available as an exercised comparison surface.

### Exit gate
- The Rust-first implementation direction is explicit in OxCalc-local doctrine and planning.
- The Rust reimplementation is defined as an ab initio realization against spec and evidence, not as a mechanical translation lane.
- The .NET implementation’s new role as executable comparison/evidence reference is explicit.
- Validation gates for Rust parity against the current exercised scope are explicit enough to drive the first Rust execution packet.
- W016 is explicitly sequenced after the Rust reimplementation direction change rather than being treated as the immediate next implementation lane.

## Sequence Preconditions
Execution Sequence C assumes the following preconditions already hold:
1. `W014` has reached its final gate and `w014-stage1-widening-baseline` remains the active normative baseline,
2. `W015` has established local replay-coherence rules, replay-facing scenario metadata, and normalized event-family doctrine,
3. the current .NET implementation remains executable and can still serve as a parity reference during the Rust transition,
4. the current `TraceCalc` corpus through `R8` remains the minimum replay and parity surface for the first Rust wave,
5. the Rust-first doctrine in `OPERATIONS.md` has been adopted before any new Rust code lands.

## Execution Packet Additions

### Environment Preconditions
- required tools:
  - `cargo`
  - `rustc`
  - `powershell`
- optional tools:
  - `lean`
  - `tlc`
  - `dotnet`
- fallback rules:
  - `dotnet` remains strongly preferred during the Rust parity phase because the current exercised implementation is the executable comparison surface
  - if `dotnet` becomes unavailable later, Rust parity claims must rely on previously checked-in baseline artifacts plus reference-machine comparison evidence
  - if `lean` or `tlc` are unavailable, W017 may proceed on implementation-direction and parity-planning scope, but no fresh formal-tool evidence may be implied

### Evidence Layout
- canonical artifact root:
  - carried baseline artifact roots remain under `docs/test-runs/core-engine/tracecalc-reference-machine/`
- checked-in or ephemeral:
  - existing checked-in W013/W014 baselines remain active references during Rust rollout
  - Rust parity runs should use explicit new run ids rather than silently rewriting the carried .NET-era baselines
- baseline run naming:
  - carried active baseline entering W017: `w014-stage1-widening-baseline`
  - first Rust parity baseline should be promoted under a distinct Rust-specific run id

### Replay-Corpus Readiness
- required replay classes with scenario ids:
  - `R1` -> `tc_accept_publish_001`
  - `R2` -> `tc_reject_no_publish_001`
  - `R3` -> `tc_multinode_dag_publish_001`, `tc_publication_fence_reject_001`
  - `R4` -> `tc_pinned_view_stability_001`
  - `R5` -> `tc_overlay_retention_001`
  - `R6` -> `tc_artifact_token_reject_001`, `tc_publication_fence_reject_001`
  - `R7` -> `tc_verify_clean_no_publish_001`
  - `R8` -> `tc_fallback_reentry_001`
- reserve or later replay classes:
  - reduced-witness, retained-failure, and replay-appliance bundle realization remain later lanes

### Coupled Widening Rule
- engine surfaces widened in this packet:
  - none in semantic scope
  - implementation-language realization changes from .NET-first local code to Rust-first local code
- oracle/conformance surfaces widened in the same slice:
  - Rust parity comparison against the carried oracle and baseline run surfaces
  - no semantic widening is allowed without a successor workset
- widened comparison artifact:
  - the first Rust parity run against `w014-stage1-widening-baseline`

## Rust Reimplementation Doctrine
1. The Rust implementation is an ab initio realization against OxCalc specs, replay artifacts, baseline runs, and conformance behavior.
2. The current .NET code is a useful executable reference, but it is not the architecture template.
3. Rust design should prefer:
   - explicit data types and enums over class hierarchies,
   - borrowing and ownership-driven interfaces over shared mutable service graphs,
   - explicit module boundaries and crate surfaces over wide mutable object collaboration,
   - deterministic serialization and artifact emission as first-class design constraints.
4. The Rust lane must start with `#![forbid(unsafe_code)]` and warning-clean validation.
5. Any future exception to `unsafe` requires a separate explicit doctrine decision and may not be smuggled into this workset.

## Critical-Path Doctrine
The critical path for this Rust-first wave is:
1. establish crate and module boundaries that reflect OxCalc semantics rather than the current .NET object layout,
2. define parity validation against the carried baseline and reference-machine surfaces,
3. realize the structural and coordinator kernel in Rust,
4. realize the widened recalc and overlay baseline in Rust,
5. realize the `TraceCalc` runner/tool lane in Rust,
6. emit one explicit Rust parity baseline run before any deeper replay or retained-witness wave proceeds.

W005 remains a carried seam condition across the sequence.
W006 also remains active because the Rust realization must continue to bind back into the formal and replay-assurance surfaces rather than drifting from them.

## Execution Sequence C

### Sequence 1. Rust Workspace and Module Boundary Definition
Primary work areas:
- Rust crate layout
- carried interface and artifact contracts

Entry gate:
- the current W014/W015 baseline surfaces are stable enough to serve as parity authority
- the repo-level Rust-first doctrine is explicit

Execution objective:
- define the initial Rust workspace and module boundaries for:
  1. structural snapshot kernel,
  2. coordinator/publication layer,
  3. recalc/overlay layer,
  4. `TraceCalc` contracts and runner support,
  5. Rust tool host entrypoint.

Exit gate:
- Rust crate boundaries are explicit enough to start code without reopening language-direction questions,
- the plan states what existing .NET components remain as executable reference only,
- no crate boundary depends on copying current .NET service or object-graph structure mechanically.

### Sequence 2. Parity and Validation Contract Definition
Primary work areas:
- parity commands
- artifact comparison rules
- carried baseline policy

Entry gate:
- Sequence 1 exit gate has passed.

Execution objective:
- lock the exact parity-validation contract for the first Rust execution wave.

Exit gate:
- the minimum Rust parity surface is explicit:
  1. carried replay classes,
  2. carried scenario ids,
  3. carried view, reject, trace, counter, and mismatch surfaces,
  4. carried baseline run ids,
- validation commands are explicit for Rust formatting, lint, tests, and parity comparison,
- baseline regeneration versus transient comparison policy is explicit.

### Sequence 3. Structural and Coordinator Rust Reimplementation
Primary work areas:
- structural snapshot kernel
- publication and candidate/reject baseline

Entry gate:
- Sequences 1 and 2 have passed.

Execution objective:
- realize the current structural and coordinator baseline in idiomatic Rust without widening semantics.

Exit gate:
- Rust covers the current structural snapshot, pinning, candidate-result, reject, and publication baseline for the declared W014/W015 surface,
- the Rust design is demonstrably ownership- and enum-driven rather than .NET-shaped,
- parity checks against the carried baseline are defined for this slice.

### Sequence 4. Recalc and Overlay Rust Reimplementation
Primary work areas:
- widened Stage 1 recalc
- planner-driven DAG handling
- first SCC-oriented handling
- overlay retention and fallback baseline

Entry gate:
- Sequence 3 exit gate has passed.

Execution objective:
- realize the widened W014 recalc and overlay surface in Rust.

Exit gate:
- Rust covers the multi-node DAG and first SCC-oriented Stage 1 surface already exercised by W014,
- the Rust slice preserves the observable semantics of the carried replay corpus,
- no semantic widening is introduced without a successor workset.

### Sequence 5. TraceCalc and Tool Host Rust Reimplementation
Primary work areas:
- scenario loading
- runner emission
- engine/reference-machine comparison integration
- Rust host entrypoint

Entry gate:
- Sequences 3 and 4 have passed enough to provide stable engine and oracle hooks.

Execution objective:
- realize the `TraceCalc` runner/tool lane in Rust while preserving OxCalc-owned artifact meaning.

Exit gate:
- Rust can consume the carried `TraceCalc` corpus,
- Rust emits artifact shapes that are still comparable against the current OxCalc contracts,
- the tool-host lane no longer depends on the .NET runner for the declared Rust-covered scope.

### Sequence 6. Rust Parity Baseline and Closure Evidence
Primary work areas:
- Rust baseline run
- Rust-versus-carried-baseline comparison
- semantic-equivalence statement

Entry gate:
- Sequences 1 through 5 have passed.

Execution objective:
- produce the first Rust-specific parity baseline and use it as the closure anchor for this packet.

Exit gate:
- at least one explicit Rust baseline run exists under the canonical artifact root,
- the Rust run is compared against `w014-stage1-widening-baseline` and the carried oracle/conformance surfaces,
- any semantic-equivalence claim is stated explicitly,
- any remaining reliance on .NET is narrow and explicitly recorded.

## Parallel Side-Lane Rules
Execution Sequence C remains critical-path driven, but these side-lane rules apply:
1. crate-boundary notes and parity-contract notes should stabilize before broad Rust coding begins,
2. carried baseline and replay corpus policies should be tightened before any Rust-specific emitted baseline is proposed,
3. formal and replay bindings may be updated alongside the Rust implementation, but they must not be used to hide parity gaps,
4. if Rust realization exposes narrower seam pressure, that pressure must route through W005 rather than being normalized locally.

## Carried Conditions Outside The 1-6 Sequence
The following remain active across the sequence but are not numbered sub-phases:
1. W001 remains the canonical-spec and repo-integration maintenance lane.
2. W005 remains the seam-alignment and follow-on-handoff lane.
3. W006 remains the assurance-binding lane connecting Rust realization back to Lean, TLA+, replay, and pack surfaces.
4. W016 remains planned but not active until the Rust-first direction has been discharged through this execution packet.

## Final Gate For Execution Sequence C
Execution Sequence C reaches its final gate only when all of the following hold:
1. Sequences 1 through 6 have each met their declared exit gate.
2. The Rust-first implementation direction is no longer only doctrinal; it is backed by a concrete Rust execution path covering the declared current realized scope.
3. The current .NET implementation’s role has been narrowed to explicit parity/evidence reference for any still-unported surfaces.
4. A Rust baseline run exists and is comparable against the carried W014 baseline and current oracle surfaces.
5. No silent semantic drift has been introduced during the language transition.
6. W016 remains correctly sequenced as a later lane rather than being used to hide unresolved Rust parity work.

This final gate is not the end of OxCalc implementation work in full.
It is the end of the implementation-direction change wave that makes Rust the active realization path for the current declared scope.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | no |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes |
| 6 | All required tests pass? | no |
| 7 | No known semantic gaps remain in declared scope? | no |
| 8 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 9 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 10 | CURRENT_BLOCKERS.md updated (new/resolved)? | yes |

## Status
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - no Rust crates or parity artifacts exist yet
  - crate-boundary and validation-plan decisions are declared, not realized
  - current .NET code still carries the exercised implementation surface
- claim_confidence: draft
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
