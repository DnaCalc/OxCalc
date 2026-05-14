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

## 6. Scope Boundary
This document does not claim:
1. rich-value kernel support,
2. sparse range reader production,
3. `ArgPreparationProfile::RichArgAccepted` activation,
4. trace/replay columns beyond the typed vocabulary artifact.

Those are owned by later Lane G beads and successor worksets.
