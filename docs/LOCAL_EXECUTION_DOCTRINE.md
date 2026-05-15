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

W050 application:
1. the formula-authority rework did not stop at replacing code paths; it also
   refreshed checked derivation trace artifacts, TreeCalc runner outputs,
   OxFml runtime package/formal-input evidence, and W050 seam/status docs in
   the same pass,
2. successor formalization and sparse/rich work are routed to W049/W051/W054
   rather than being treated as W050 closure gaps,
3. the lesson generalizes from "engine and oracle" to "engine, consumed seam,
   and replay evidence" when the changed behavior crosses repo boundaries.

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

### Lesson 11: Replay Projection Must Stay Additive
W018 only stayed coherent because replay-appliance bundle roots, validator artifacts, and explain artifacts were emitted as additive sidecars over the canonical OxCalc-native artifact surface.

Reason:
1. replay consumers need normalized artifacts,
2. OxCalc still needs its native artifacts to remain the semantic authority,
3. replacing native artifacts would have muddied parity checks and semantic ownership.

Rule:
1. replay-facing projections must be additive to canonical OxCalc-native artifacts,
2. a replay projection wave may widen emitted evidence, but it must not replace the native artifact root as the semantic authority.

### Lesson 12: Projection, Validator, and Explain Should Ship Together
W018 moved cleanly once bundle emission, validation, and explain were treated as one artifact family rather than three disconnected later steps.

Reason:
1. a projection without validation is weak evidence,
2. validation without explain makes capability promotion stall,
3. splitting them too far apart creates avoidable rework in manifests and baselines.

Rule:
1. replay-facing execution packets should declare emitted projection, validator output, explain output, and capability consequence together,
2. a capability-promotion wave is not gate-ready if one of those surfaces is deferred without being called out explicitly.

### Lesson 13: Capability Claims Follow Checked-In Evidence, Not Code Shape
The W018 capability refresh only became trustworthy once the checked-in baselines, validator artifacts, explain artifacts, canonical manifest, and run-local snapshots all agreed.

Reason:
1. code can get ahead of evidence,
2. run-local snapshots can drift from the canonical manifest,
3. capability promotion becomes sloppy if the checked-in evidence root is not authoritative.

Rule:
1. capability claims move only when checked-in replay-facing baselines and emitted validator or explain artifacts exist,
2. run-local capability snapshots must be regenerated when the canonical manifest changes,
3. code-level readiness without checked-in evidence is not a capability promotion act.

### Lesson 14: Capability-Ladder Waves Need Their Successor Packet Before Closure
W018 closed cleanly only once W019 existed as the explicit follow-on for `cap.C4.distill_valid` and `cap.C5.pack_valid`.

Reason:
1. otherwise future capability work gets smeared back into the closing packet,
2. workset closure and feature continuation become blurry again,
3. later widening risks silently reopening a packet that already reached its gate.

Rule:
1. any capability-promotion wave should author its successor packet before closure if later capability levels are already known,
2. broader feature continuation must point to that successor rather than implicitly reopening the closing workset.

## 4. Immediate Promotion Targets
These lessons should now influence:
1. `OPERATIONS.md`
2. `docs/worksets/README.md`
3. later execution packets after `W013`
4. replay and harness planning packets where corpus coverage and emitted-artifact policy matter

## 5. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes:
  - this local doctrine now reflects W013, W014, and the W050 formula-authority/replay-evidence lesson, but later replay-appliance and retained-witness waves may add further lessons
  - later execution waves may add or refine local lessons
