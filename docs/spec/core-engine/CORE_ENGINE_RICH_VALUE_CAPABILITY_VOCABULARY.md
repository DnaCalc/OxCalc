# Core Engine Rich-Value Capability Vocabulary

## 1. Purpose and Status
This document defines the OxCalc-local typed capability vocabulary admitted by
W050 Lane G for future `RichValueHole(required_capability_set)` identity.

Status:
1. active W050 companion,
2. identity and replay vocabulary only,
3. no rich-value-producing kernel activation in this lane.

## 2. Initial Capability Selectors
The initial selector vocabulary is:

| selector | typed parameters | meaning |
|---|---|---|
| `Indexable` | `rank`, `index_type`, `element_value_class` | supports deterministic indexed access to elements |
| `Enumerable` | `element_value_class`, `order_guarantee` | supports deterministic iteration over elements |
| `Shaped` | `extent_class` | exposes extent/cardinality shape facts |
| `Materialisable` | `target_class` | can produce an ordinary value or array payload when a kernel requires material bytes |

The first parameter value families are intentionally small:

| parameter | initial values |
|---|---|
| `rank` | `AnyRank`, exact positive ranks such as `2` |
| `index_type` | `GridCoordinate`, `Ordinal` |
| `element_value_class` | `AnyValue` |
| `order_guarantee` | `DeterministicStable`, `RowMajorStable` |
| `extent_class` | `AnyExtent`, `RectangularGrid` |
| `target_class` | `EvalValueOrArray` |

## 3. Replay Identity Rule
A required capability set is part of replay/template identity.

Stable identity rules:
1. each capability emits a typed stable key containing the selector and all
   typed parameter values,
2. a required set sorts capability keys by byte-stable key order,
3. duplicate capability keys collapse,
4. the sorted required-set key is the replay identity member.

Producer admission is a superset check: a producer may provide more
capabilities than required, but the `RichValueHole` identity remains the
required set, not the producer's concrete rich-value class and not the
producer's full capability set.

### 3.1 RichValueHole Template-Key Rule
`RichValueHole(required_capability_set)` is a first-class hole-taxonomy member.
The required capability set is included in the hole stable key, and the hole
stable key is included in `plan_template_key` material alongside the other
template-hole facts.

Two otherwise identical templates with different required capability sets must
therefore have different plan-template key material. This is an identity rule
only; it does not imply that any current production path emits rich holes.

### 3.2 Trace/Replay Columns
Trace and replay surfaces reserve the nested
`rich_value_capability_columns` field for prepared formula identities,
derivation trace records, derivation template selections, and derivation
template holes.

The column schema is:
1. `required_capability_set_keys` — sorted, deduplicated capability-set keys
   required by rich-value holes in the template surface,
2. `producer_capability_set_keys` — reserved for published rich-value producer
   capability sets once rich producers exist,
3. `exercised_capability_keys` — reserved for replay-visible capability
   operations once rich kernels exercise them.

Current V1 production paths emit no `RichValueHole` and no rich producer
capability sets. Their runtime columns are therefore empty/reserved, and
ordinary serialized trace artifacts omit the nested field when all three
columns are empty.

### 3.3 `RichArgAccepted` Reservation
The kernel-side argument-preparation profile reserved for this vocabulary is:

`ArgPreparationProfile::RichArgAccepted(capability_set)`

This profile is sibling-owned by OxFunc and threaded by OxFml when it exists.
OxCalc does not define the enum variant locally and does not activate any rich
kernel in W050.

Reservation rules:
1. `capability_set` uses the same typed stable-key vocabulary as
   `RichValueHole(required_capability_set)`,
2. a producer satisfies the profile only by publishing a capability-set
   stable-key superset of the required set,
3. switching any existing OxFunc function to this profile is bind-visible and
   requires `ArgPreparationProfile` metadata versioning,
4. W050 records the additive identity shape only; first rich-kernel activation
   belongs to successor work.

## 4. Additive Extension Rule
New capability selectors or new parameter values are additive only through new
stable keys. Existing traces recorded with an older required-set key remain
byte-stable and do not silently alias traces recorded with new capabilities.

Changing the meaning of an existing selector key or parameter value is not an
additive extension. It requires a new selector/parameter key or a replay
migration proof.

## 5. Evidence
The checked OxCalc-local vocabulary artifact is:

`docs/test-runs/core-engine/w050-g1-rich-capability-vocabulary-001/run_artifact.json`

Validation command:

```powershell
cargo test -p oxcalc-core rich_value_capability -- --nocapture
```

The test compares the checked artifact against the Rust vocabulary surface in
`src/oxcalc-core/src/rich_value_capability.rs`.

The checked `RichValueHole(required_capability_set)` identity artifact is:

`docs/test-runs/core-engine/w050-g2-rich-value-hole-capability-requirements-001/run_artifact.json`

Validation command:

```powershell
cargo test -p oxcalc-core rich_value_hole -- --nocapture
```

The test proves that changing the required capability set changes
plan-template key material while current V1 production paths still emit no
rich-value holes.

The checked trace/replay column artifact is:

`docs/test-runs/core-engine/w050-g3-capability-trace-replay-columns-001/run_artifact.json`

Validation command:

```powershell
cargo test -p oxcalc-core capability_set_trace_replay -- --nocapture
```

The test proves that current V1 runtime trace columns are empty/reserved and
that a reserved rich requirement projects the sorted required capability-set
key into the same column schema.

The checked `RichArgAccepted` reservation artifact is:

`docs/test-runs/core-engine/w050-g4-richargaccepted-reservation-001/reservation_artifact.json`

Validation command:

```powershell
cargo test -p oxcalc-core rich_argaccepted_reservation -- --nocapture
```

The test binds the reserved profile name to the OxCalc capability vocabulary,
records the read-only sibling source observation, and proves the checked note
does not claim OxFunc/OxFml code movement or rich-kernel activation.

## 6. Scope Boundary
This document does not claim:
1. rich-value kernel support,
2. sparse range reader production,
3. `ArgPreparationProfile::RichArgAccepted` activation,
4. producer capability-set emission by current V1 kernels.

Those are owned by later Lane G beads and successor worksets.
