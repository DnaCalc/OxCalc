# W058 Retained-Replay Outstanding Registry

Status: `active_registry`
Date: 2026-05-30

Parent epic: `calc-gogj`

## 1. Purpose
This workset is the central registry of **outstanding retained-replay (OxReplay) obligations** for non-table TreeCalc reference families, and the home for reconciling the `ProductGreen` evidence bar.

It exists because a W056 evidence pass (A1/A2 = `calc-4vs8.71`/`.72`) found that the retained-replay half of "product-green" is genuinely absent for non-table reference families, while the direct-context corpora are green.

## 2. Decision (2026-05-30)
The DNA TreeCalc **UX prototype proceeds on direct-context-green evidence**. True retained-replay (`ProductGreen`) is **deferred** and tracked here rather than faked. `A1`/`A2` are deferred and blocked on `calc-4vs8.74`.

This is an accept-and-defer decision, not a closure: the families resolve and evaluate correctly today; only the retained-replay conformance artifact is missing.

## 3. The cross-repo deliverable (what "unblocked" means)
Retained replay for a non-table family requires, end-to-end:
1. **DnaTreeCalc** emits normalized-replay artifacts for the family (producer side; direct-context corpora already exist).
2. **OxReplay** retains and `validate`/`diff`/`replay`s those artifacts as **opaque JSON**, without TreeCalc-private parsing (today OxReplay retained validation covers only the table lane — `TreeCalcTableReplayEvidenceLane`, `oxreplay-qb9` / `host_rollout_w056_table_third_pass_001`).
3. **OxCalc** flips the family `evidence_status` `DirectContextSliceGreen → ProductGreen` and clears `blocks_w056_non_table_closure` in `src/oxcalc-core/src/formula.rs`.

## 4. ProductGreen bar reconciliation (`calc-gogj.1`)
The `children` (`@CHILDREN`/`.*`) family is already marked `ProductGreen` in `formula.rs` **without** a retained artifact ("no retained non-table replay requirement yet"). The bar is therefore applied inconsistently. Reconcile by either (a) redefining `ProductGreen` so it does not imply retained replay for navigation families, or (b) downgrading `children` to `DirectContextSliceGreen` until a retained artifact exists. Pick one and apply it uniformly.

## 5. Registry — outstanding retained-replay items

### 5.1 Prototype navigation families (deferred; blocked by `calc-4vs8.74`)
Each is direct-context green; `replay_status` in `formula.rs` reads "retained non-table replay missing".

| Family | Examples | Direct-context | Retained replay | Producer (DnaTreeCalc) |
| --- | --- | --- | --- | --- |
| walk-up / dotted descent | `Margin`, `Q1.Margin` | green | **missing** | `dtc-z0i.4` |
| meta accessors | `@PARENT/@NAME/@INDEX/@FORMULA` | green | **missing** | (meta corpus) |
| siblings | `@PREV/@NEXT` | green | **missing** | `dtc-z0i.15/.18` |
| children | `@CHILDREN`, `.*` | green (`ProductGreen`*) | **missing*** | `dtc-z0i` |
| ordered selectors | `@PRECEDING/@FOLLOWING/@ANCESTORS` (incl. tailed) | green | **missing** | `dtc-z0i.3/.9` |
| recursive descent | `**` | green | **missing** | `dtc-z0i.3/.9` |
| ancestor/root anchors | `^`, `[]` | green | **missing** | `dtc-z0i.4` |
| bare host names | defined-name lane | green | **missing** | `dtc-z0i.4` |

\* children is the inconsistency reconciled by `calc-gogj.1`.

### 5.2 Lower-priority families (outstanding, not in the prototype set)
Cross-workspace references, dynamic `INDIRECT`, raw reference-literal arrays, structural-edit rebind, and diagnostics also lack retained non-table replay. They are out of the prototype scope (see W056 deprioritization) and registered here for completeness.

## 6. Relationship to other worksets
- **W056** (`calc-4vs8`) owns the reference resolution + the prototype evidence beads `A1`/`A2`; those are deferred here.
- **`calc-4vs8.33`** remains the broader non-table evidence umbrella; its full closure is gated on the same retained-replay infrastructure.
- This workset does not own reference resolution or producer corpora — only the retained-replay obligation tracking and the bar reconciliation.
