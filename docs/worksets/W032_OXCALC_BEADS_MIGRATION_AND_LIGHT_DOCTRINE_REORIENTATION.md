# W032: OxCalc Beads Migration And Light Doctrine Reorientation

## Purpose
Migrate OxCalc from the older workset-plus-feature-register-plus-blocker execution model to a lighter `workset -> epic -> bead` model.
This packet owns the doctrine rewrite, `WORKSET_REGISTER` introduction, `.beads/` bootstrap, validator addition, and the reorientation of active repo-level docs away from ad hoc execution-state ownership.

## Position and Dependencies
- **Depends on**: W026
- **Blocks**: none
- **Cross-repo**: none

## Scope
### In scope
1. create `docs/WORKSET_REGISTER.md` as the ordered workset authority
2. create `docs/BEADS.md` as the local bead-method authority
3. bootstrap `.beads/` as the live execution-state surface
4. add a minimal `scripts/check-worksets.ps1` validator
5. reorient `README.md`, `OPERATIONS.md`, `AGENTS.md`, `docs/worksets/README.md`, and `docs/IN_PROGRESS_FEATURE_WORKLIST.md` to the lighter doctrine split
6. record the active TreeCalc packet line in the register, including reached-gate `W025` and `W026` plus live execution targets `W027` through `W031`
7. retire `CURRENT_BLOCKERS.md` from active ordinary blocker ownership without claiming a broad archive-wave reduction beyond this packet

### Out of scope
1. broad historical workset removal from the active tree
2. full archive-wave execution beyond lightweight doctrine reorientation
3. any new TreeCalc engine implementation widening beyond doctrine and planning-surface changes
4. any new upstream seam negotiation

## Deliverables
1. `docs/WORKSET_REGISTER.md`
2. `docs/BEADS.md`
3. bootstrapped `.beads/`
4. `scripts/check-worksets.ps1`
5. updated active doctrine/docs pointing to the new execution model
6. a compact history pointer for retired execution-surface ownership changes

## Gate Model
### Entry gate
- W026 has reached its declared gate for the first TreeCalc consumed-seam packet
- the active TreeCalc line is stable enough to register explicitly without reopening W026 scope

### Exit gate
- `docs/WORKSET_REGISTER.md` exists and is the ordered workset authority
- `docs/BEADS.md` exists and defines the local bead method
- `.beads/` exists and is the declared live execution-state surface
- ordinary blocker truth no longer belongs to `CURRENT_BLOCKERS.md`
- active repo-level docs no longer treat `docs/worksets/README.md` or `docs/IN_PROGRESS_FEATURE_WORKLIST.md` as live execution trackers
- the active TreeCalc packet line is represented in the register
- the validator passes

## Execution Packet Additions
### Environment Preconditions
- PowerShell available on PATH
- repo-local docs and scripts are writable
- `br` / `bv` may be unavailable during bootstrap; bootstrap files may therefore be initialized directly

### Evidence Layout
- canonical artifact root:
  - `docs/WORKSET_REGISTER.md`
  - `docs/BEADS.md`
  - `.beads/`
  - `scripts/check-worksets.ps1`
  - this packet
- checked-in or ephemeral:
  - checked-in
- baseline run naming:
  - none

### Replay-Corpus Readiness
- required replay classes with scenario ids:
  - none
- reserve or later replay classes:
  - all runtime replay classes remain owned by their existing and successor engine worksets

### Coupled Widening Rule
- engine surfaces widened in this packet:
  - none
- oracle/conformance surfaces widened in the same slice:
  - none
- widened comparison artifact:
  - none

## Migration Result
Current OxCalc doctrine result is:
1. ordered workset truth now lives in `docs/WORKSET_REGISTER.md`
2. the local bead method now lives in `docs/BEADS.md`
3. `.beads/` now owns live execution-state truth
4. `CURRENT_BLOCKERS.md` is no longer an ordinary blocker ledger
5. `docs/worksets/README.md` and `docs/IN_PROGRESS_FEATURE_WORKLIST.md` are now lighter reference surfaces rather than status boards

## Pre-Closure Verification Checklist
1. Spec text and realization notes updated for all in-scope items: yes
2. Workset-register expectations updated for affected packets: yes
3. At least one deterministic replay artifact exists per in-scope behavior: yes, not applicable for this doctrine/bootstrap packet
4. Semantic-equivalence statement provided for policy or strategy changes: yes
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: yes
6. All required tests pass: yes
7. No known semantic gaps remain in declared scope: yes
8. Completion language audit passed: yes
9. `WORKSET_REGISTER.md` updated when ordered workset truth changed: yes
10. `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed: yes
11. execution-state blocker surface updated (`.beads/` for ordinary blockers; prose blocker surface only for exceptional narrative blockers): yes

## Completion Claim Self-Audit
1. Scope re-read: pass
2. Gate criteria re-read: pass
3. Silent scope reduction check: pass
4. "Looks done but is not" pattern check: pass
5. Self-audit inclusion in report: pass

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: none
- claim_confidence: high
- reviewed_inbound_observations: none
