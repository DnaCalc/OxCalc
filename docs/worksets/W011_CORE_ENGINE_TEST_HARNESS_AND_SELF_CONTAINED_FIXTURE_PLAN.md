# W011: Core Engine Test Harness and Self-Contained Fixture Plan

## Purpose
Define the self-contained OxCalc test harness shape needed to test coordinator, publication, invalidation, overlay, replay, and scaling behavior without depending on full Excel formula language or full OxFml function semantics.

## Position and Dependencies
- **Depends on**: W003, W004, W005, W009, W010
- **Blocks**: none
- **Cross-repo**: must remain compatible with accepted OxFml seam direction, but should minimize dependence on full OxFml implementation breadth

## Scope
### In scope
1. Minimal OxFml-facing test-double interface surface required by OxCalc.
2. Self-contained fixture lifecycle for structural setup, candidate-result injection, publication, replay capture, and teardown.
3. Scriptable test-host environment for deterministic scenario execution.
4. Alternate non-Excel calculation space for scalable and traceable engine-only testing.

### Out of scope
1. Full production host design.
2. Full Excel formula-language semantics.
3. Full function library semantics.
4. Immediate implementation of the full harness.

## Deliverables
1. Test-double interface packet for the minimum evaluator-facing surface OxCalc needs.
2. Fixture lifecycle packet for setup, step execution, capture, assert, and teardown.
3. Scriptable host packet for deterministic scenario authoring.
4. Alternate calculation-space packet for correctness and scaling tests independent of Excel semantics.

## Gate Model
### Entry gate
- W003 and W004 define the Stage 1 coordinator and recalc packet boundaries.
- W005 accepted seam direction is available.
- W009 and W010 define replay classes, pack hooks, and measurement families the harness must support.

### Exit gate
- The self-contained harness boundary is explicit enough to author the first unit-test fixture and scriptable host.
- The minimum OxFml test-double interface is explicit enough to avoid coupling core-engine tests to the full formula language.
- The alternate calculation space is explicit enough to support correctness and scale experiments with traceable semantics.

## Minimal OxFml Test-Double Interface Surface
The self-contained harness should define the smallest evaluator-facing surface OxCalc needs for Stage 1 tests.

### Required evaluator-side capabilities
1. open a candidate work session against a structural snapshot or compatibility basis
2. produce an evaluator-side `AcceptedCandidateResult` payload with stable identity
3. produce typed reject outcomes with machine-readable reject detail
4. surface runtime-derived effect families that OxCalc actually coordinates on in Stage 1
5. preserve deterministic replay identifiers for candidate-result and reject events

### Explicit non-goals for the test double
1. full formula parsing
2. full binding semantics
3. full function evaluation
4. Excel-compatibility behavior outside what the harness is intentionally simulating

## Fixture Lifecycle
The fixture lifecycle should be explicit and replay-friendly.

### F1. Structural setup
1. construct a structural snapshot from scripted node declarations
2. attach immutable artifact handles or synthetic evaluator payload references

### F2. Runtime initialization
1. initialize runtime view state and coordinator baseline state
2. optionally seed overlay or pinning conditions for targeted scenarios

### F3. Scripted step execution
1. apply structural or upstream changes
2. admit candidate work
3. inject accepted candidate results or typed rejects from the test double
4. execute publication or reject consequences deterministically

### F4. Capture
1. capture trace events
2. capture replay bundle fragments
3. capture counter snapshots relevant to the scenario
4. capture stable published view and pinned-reader observations

### F5. Assertions
1. assert structural snapshot invariants
2. assert publication or reject behavior
3. assert pinned-view behavior where relevant
4. assert counter and replay expectations where relevant

### F6. Teardown or reset
1. release readers
2. release retained runtime state
3. reset deterministic host state for the next scenario

## Scriptable Host Environment
The self-contained harness should include a small scriptable host rather than requiring direct unit tests to encode every transition imperatively.

### Host responsibilities
1. scenario declaration
2. deterministic step sequencing
3. controlled injection of evaluator outcomes
4. trace and counter capture
5. simple expectation assertions

### Script surface expectations
The first scriptable host should support operations such as:
1. declare node
2. connect dependency
3. mark stale or inject upstream change
4. pin reader
5. admit work
6. emit accepted candidate result
7. emit typed reject
8. publish
9. unpin reader
10. inspect trace, counters, or published view

## Alternate Calculation Space
The harness should define a small alternate calculation space whose semantics are intentionally not Excel.
Its purpose is to test OxCalc's engine behavior, not Excel-language fidelity.

### Design goals
1. deterministic
2. easy to trace by hand
3. scalable to larger synthetic graphs
4. capable of expressing dynamic-dependency and rejection cases
5. decoupled from Excel formula grammar and function library breadth

### Suggested node families
1. constant node
2. pure dependency aggregate node
3. conditional dependency node
4. dynamic-dependency node
5. capability-gated node
6. synthetic cycle-region node
7. synthetic delayed or staged node where replay timing matters

### Why this alternate space is needed
1. it isolates core-engine correctness from formula-language correctness
2. it makes scale and stress scenarios easier to generate
3. it gives a traceable substrate for replay and measurement work
4. it avoids blocking core-engine testing on full OxFml completeness

## Harness-to-Workset Binding
This harness planning packet should feed:
1. W003 for coordinator fixture expectations
2. W004 for invalidation, overlay, and fallback scenarios
3. W009 for replay corpus authoring
4. W010 for counter and experiment capture

## Initial Scenario Families The Harness Must Support
1. accept-and-publish success path
2. reject-is-no-publish path
3. candidate-result versus publication separation
4. pinned-reader stability under later work
5. overlay retention and release behavior
6. dynamic-dependency fallback behavior
7. synthetic scale graph runs for work-volume and retention counters

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes |
| 6 | All required tests pass? | no |
| 7 | No known semantic gaps remain in declared scope? | no |
| 8 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 9 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 10 | CURRENT_BLOCKERS.md updated (new/resolved)? | no |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - no harness or fixture artifact has been authored yet
  - exact minimal test-double payload shape still needs alignment with W003 and W004 packets
  - the alternate calculation-space semantics are named, but not yet formalized into scenario schemas
  - no scriptable host schema exists yet
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
