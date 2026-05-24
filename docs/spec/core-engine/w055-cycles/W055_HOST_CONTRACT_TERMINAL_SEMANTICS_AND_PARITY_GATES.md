# W055 Host Contract Terminal Semantics And Parity Gates

Status: `active_w055_contract_target`

Owner beads: `calc-9ouy.2`, `calc-9ouy.8`, `calc-9ouy.9`, `calc-9ouy.10`

Parent workset:
`docs/worksets/W055_CIRCULAR_REFERENCES_AND_ITERATIVE_CALCULATION_EXCEL_MATCH_CLOSURE.md`

## 1. Purpose

This packet fixes the W055 contract target for circular references and
iterative calculation.

It answers the DnaTreeCalc handover at production-contract level and gives W055
implementation beads concrete fields and terminal-state rules.

## 2. Contract Decision

The production cycle configuration belongs on the per-recalc request.

The exact host-facing field is:

`OxCalcTreeContext recalculation configuration.cycle_config`

It must not be encoded in `compatibility_basis`.

Placement rules:

1. `OxCalcTreeContext recalculation configuration.cycle_config` carries selected cycle semantics for
   the current run.
2. `OxCalcTreeHostCapabilitySnapshot.capability_profile_id` remains host
   capability identity; it does not select cycle behavior.
3. `OxCalcTreeRuntimePolicy` may control trace verbosity or scheduling policy,
   but it does not select cycle profile or iteration bounds.
4. `compatibility_basis` remains correlation/provenance text. It may mention a
   fixture or observation basis, but it is not the semantic cycle config path.

## 3. Request Shape

W055 extends `OxCalcTreeContext recalculation configuration` with:

```rust
pub cycle_config: Option<OxCalcTreeCycleConfig>
```

The structured config is:

```rust
pub struct OxCalcTreeCycleConfig {
    pub cycle_profile_id: String,
    pub maximum_iterations: Option<u32>,
    pub maximum_change: Option<OxCalcTreeMaximumChange>,
}

pub struct OxCalcTreeMaximumChange {
    pub decimal: String,
}
```

Implementation may choose narrower Rust newtype names, but the host-visible
fields and meanings must stay stable:

1. `cycle_profile_id`
2. `maximum_iterations`
3. `maximum_change`

Admitted profile ids:

1. `cycle.non_iterative_stage1`
2. `cycle.excel_match_iterative`
3. `cycle.iterative_deterministic_v0`

Default rules:

1. absent `cycle_config` means `cycle.non_iterative_stage1`;
2. absent `maximum_iterations` on an iterative profile means `100`;
3. absent `maximum_change` on an iterative profile means `0.001`;
4. host-supplied `maximum_iterations` must be positive;
5. host-supplied `maximum_change` must be finite and non-negative after decimal
   parsing.

The W048 stop metric remains the maximum absolute visible numeric delta across
cycle members unless W055 evidence falsifies it.

Invalid profile ids or invalid bounds are request-validation failures. They must
not be silently lowered to `cycle.non_iterative_stage1`.

## 4. Result Shape

W055 extends `OxCalcTreeCalculationOutcome` with:

```rust
pub cycle_diagnostics: Vec<OxCalcTreeCycleDiagnostic>
```

Each diagnostic record must expose:

1. `cycle_region_id`,
2. `cycle_profile_id`,
3. `region_source`,
4. `members`,
5. `report_node_id`,
6. `member_order`,
7. `terminal_state`,
8. `publication_decision`,
9. `reject_kind`,
10. `iteration_trace`.

`report_node_id` is the host-facing equivalent of Excel
`Worksheet.CircularReference`. It is optional because iteration-enabled cases
may have no report cell and some profiles may report region membership rather
than a single root.

`iteration_trace` must expose:

1. submitted max iterations,
2. submitted max change,
3. stop metric,
4. iteration count,
5. initial vector,
6. terminal vector,
7. terminal state.

Vectors should use host-visible node ids and visible values so DnaTreeCalc can
project them without reading internal engine structures.

String diagnostics may remain as a compatibility aid, but they are not the
production cycle diagnostic surface.

## 5. Terminal States

The typed terminal-state vocabulary is:

1. `converged`
2. `max_iteration`
3. `oscillation`
4. `divergent`
5. `blocked_non_iterative`
6. `rejected_nonnumeric_or_error`

The first four states are the iterative terminal classifications requested by
DnaTreeCalc. The last two states make the no-iteration and unsupported-value
paths explicit for hosts.

## 6. Profile Publish Rules

`cycle.non_iterative_stage1`:

1. any detected cycle yields `blocked_non_iterative`;
2. the run rejects with `SyntheticCycleReject`;
3. no new cycle values are published;
4. `report_node_id` and cycle-region membership must be emitted when available.

`cycle.excel_match_iterative`:

1. `converged` publishes the whole cycle region atomically;
2. `max_iteration` publishes terminal values for W055-included surfaces when
   the accepted Excel observation set shows finite publishable terminal values;
3. `oscillation` publishes terminal values for W055-included surfaces when the
   accepted Excel observation set shows finite publishable terminal values;
4. `divergent` must be classified separately from ordinary max-iteration; it
   can enter a product claim only after observation defines whether the
   terminal values publish or the scenario is blocked/excluded;
5. nonnumeric/error terminal behavior must be observed, blocked, or explicitly
   excluded before it enters an Excel-match product claim.

`cycle.iterative_deterministic_v0`:

1. `converged` publishes the whole cycle region atomically;
2. `max_iteration`, `oscillation`, `divergent`, and
   `rejected_nonnumeric_or_error` reject with no publication;
3. member order and initial vector are canonical profile data rather than
   Excel observation data.

## 7. Parity Labels

W055 must report parity by declared scope.

Allowed labels:

1. `Excel-faithful (covered surfaces)` for the current W048/W055 single-host
   covered surfaces.
2. `Tranche A Excel parity` only after the general engine replaces fixture-keyed
   behavior for the declared Tranche A family and conformance passes.
3. `Hard-family parity lane` for dynamic arrays, data tables, external workbook
   links, and thread variants when each separate lane closes.

Forbidden label:

1. broad `Excel parity` for all circular references until cross-version,
   thread, dynamic-array, data-table, and external-link dimensions are either
   implemented, blocked with accepted limitation, or explicitly excluded from
   the named product scope.

## 8. DnaTreeCalc Acceptance Gate

W055 satisfies the DnaTreeCalc handover only when:

1. DnaTreeCalc can submit `cycle_profile_id`, `maximum_iterations`, and
   `maximum_change` through `OxCalcTreeContext recalculation configuration.cycle_config`;
2. DnaTreeCalc can read typed cycle diagnostics from
   `OxCalcTreeCalculationOutcome.cycle_diagnostics`;
3. a non-iterative circular-reference case exposes a `Worksheet.CircularReference`
   equivalent and a no-publication reject;
4. an iterative covered-surface case exposes terminal classification,
   iteration-trace summary, and published values when the profile publishes;
5. no DnaTreeCalc active corpus case relies on `compatibility_basis` to select
   cycle semantics.

Until that gate passes, DnaTreeCalc may keep a narrow acyclic or transitional
non-iterative smoke path, but iterative cycle corpus cases remain pending.

## 9. Bead Consequence

The W055 bead split is:

1. `calc-9ouy.8` owns this contract decision and downstream answer;
2. `calc-9ouy.9` owns the Rust facade implementation;
3. `calc-9ouy.2` owns the general engine semantics that consume this contract;
4. `calc-9ouy.10` owns DnaTreeCalc bridge or handoff acceptance evidence.
