# LOCAL_EXECUTION_DOCTRINE.md — OxCalc Local Execution Lessons

## 1. Purpose
Record OxCalc-local execution lessons that were learned from exercised work rather than inherited from sibling repos.

These lessons are operating doctrine.
They should influence future workset design, execution packet authoring, and closure claims.

## 2. Current Basis
The current local lesson set is derived from:
1. `W013_EXECUTION_SEQUENCE_A_TREECALC_STAGE1_IMPLEMENTATION.md`
2. `W014_EXECUTION_SEQUENCE_B_STAGE1_WIDENING_AND_EVIDENCE_HARDENING.md`
3. the first realized `TraceCalc` validator, runner, engine, and reference machine
4. the emitted baseline runs at:
   - `docs/test-runs/core-engine/tracecalc-reference-machine/w013-sequence-a-baseline/`
   - `docs/test-runs/core-engine/tracecalc-reference-machine/w014-stage1-widening-baseline/`

## 3. Local Lessons

### Lesson 1: Execution Packets Must Declare Their Artifact Root Up Front
If an execution wave expects emitted evidence, it must declare the canonical artifact root before implementation begins.

Reason:
1. late artifact-root decisions create avoidable churn,
2. replay and pack references become unstable,
3. closure slows down because emitted evidence has to be normalized retroactively.

Rule:
1. every execution packet that expects emitted evidence must name one canonical artifact root,
2. it must also declare whether those artifacts are checked in or ephemeral.

### Lesson 2: Tracked Artifacts Must Use Repo-Relative Paths Only
Tracked emitted artifacts must not contain machine-specific absolute paths.

Reason:
1. absolute paths break determinism,
2. they reduce portability across checkouts and CI,
3. they pollute conformance diffs with environment noise rather than semantic signal.

Rule:
1. any checked-in artifact must use repo-relative paths,
2. absolute paths are allowed only in transient local diagnostics that are not tracked.

### Lesson 3: Replay Classes Need Corpus Scenarios Before Harness Work Starts
It is too late to discover missing replay coverage after the runner and oracle already exist.

Reason:
1. implementation then drives corpus shape instead of the declared replay model,
2. closure gets blocked by avoidable scenario gaps,
3. replay claims become weaker than the planning packet suggests.

Rule:
1. if a replay class is declared as part of an execution-wave exit gate, at least one scenario must exist for it before harness implementation begins,
2. missing replay classes must be listed as explicit pre-entry gaps, not silently deferred.

### Lesson 4: Harness and Oracle Output Shape Must Be Tested, Not Just Their Logic
The first failure in W013 was not a semantic engine error.
It was an emitted-artifact shape mismatch.

Reason:
1. a runner can be semantically correct and still violate the contract surface,
2. artifact-shape regressions break replay, pack binding, and downstream tooling.

Rule:
1. harness tests must assert emitted artifact shape as well as scenario pass or fail state,
2. artifact normalization is part of the implementation contract, not cleanup.

### Lesson 5: One Stable Checked-In Baseline Run Is Worth More Than Many Ad Hoc Smoke Runs
The baseline checked-in run created a clear evidence anchor.

Reason:
1. it gave W009, W011, W012, and W013 one shared reference surface,
2. it made closure easier to audit,
3. it reduced ambiguity about which run was normative.

Rule:
1. each execution wave that produces emitted evidence should designate one stable checked-in baseline run,
2. ad hoc smoke runs are useful, but they are not closure evidence unless promoted.

### Lesson 6: Tool Availability Must Be Declared as an Execution Precondition
Lean was available.
`tlc` was not.
That difference mattered.

Reason:
1. formal lanes can appear symmetrical on paper while being asymmetrical in practice,
2. closure language becomes sloppy when missing tools are discovered late.

Rule:
1. execution packets must declare required tools, optional tools, and fallback evidence rules,
2. if a tool is unavailable, that must be stated in the evidence report with the exact fallback used.

### Lesson 7: Pack Names Alone Are Not Enough
Pack planning was present before W013, but not yet sharp enough to point directly at exercised artifacts.

Reason:
1. closure is slower when packs are named abstractly,
2. later validation has to reconstruct which scenarios and artifacts satisfy which pack surfaces.

Rule:
1. planning packets should link each named pack to replay classes, scenario ids, and emitted artifact paths as soon as those exist,
2. pack references without evidence links remain planning-only.

### Lesson 8: Engine and Oracle Widening Must Move Together
W014 only became stable once the planner-driven widening was applied to both the real engine and the `TraceCalc Reference Machine` in the same slice.

Reason:
1. semantic widening on one side alone creates noisy conformance failures,
2. late oracle alignment obscures whether the defect is semantic or mechanical,
3. widened replay evidence is much easier to trust when engine and oracle evolve together.

Rule:
1. any semantic widening lane must update engine, oracle, and conformance comparison in the same execution slice,
2. widening work is not gate-ready if one of those three surfaces is left behind.

### Lesson 9: Workset Closure and Feature-Area Continuation Must Stay Separate
W014 reinforced that a workset can reach its declared gate while the broader feature area still continues in later waves.

Reason:
1. later widening should not silently reopen already-closed worksets,
2. closure loses meaning if broader feature continuation is treated as retroactive incompleteness,
3. the feature register and workset sequence serve different purposes and should say so clearly.

Rule:
1. once a workset reaches its declared gate, later widening should land in a successor workset or explicit extension lane,
2. the feature register may remain `in-progress` while the completed workset stays closed for its declared scope,
3. docs should state that distinction explicitly rather than implying one invalidates the other.

### Lesson 10: Replay-Facing Scenarios Need Stronger Metadata Discipline
The widened W014 scenarios were authorable without strict metadata, but W015 and W016 will need stable replay and witness anchors immediately.

Reason:
1. replay-bundle projection depends on explicit replay classes and equality surfaces,
2. witness reduction depends on stable scenario, phase, event-group, reject, and view anchors,
3. leaving these as optional in practice creates cleanup work in later replay lanes.

Rule:
1. any new replay-facing scenario must carry `replay_projection`,
2. any scenario intended for retained-failure or witness reduction lanes must carry `witness_anchors`,
3. “optional” metadata in the base schema may still be required by narrower execution packets.

## 4. Immediate Promotion Targets
These lessons should now influence:
1. `OPERATIONS.md`
2. `docs/worksets/README.md`
3. later execution packets after `W013`
4. replay and harness planning packets where corpus coverage and emitted-artifact policy matter

## 5. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - this local doctrine now reflects W013 and W014 lessons, but later replay-appliance and retained-witness waves may add further lessons
  - later execution waves may add or refine local lessons
