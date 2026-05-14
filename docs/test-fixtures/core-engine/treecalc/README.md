# TreeCalc Fixture Policy

TreeCalc fixtures are evidence for OxCalc coordinator behavior. They are
not a place for OxCalc to own spreadsheet formula semantics.

Allowed fixture shapes:
1. No-formula structural records: a node may be constant or structural
   only and omit a formula binding.
2. Opaque OxFml source: use `RawOxfml` with `source_text` plus explicit
   `reference_carriers` for dependency or evaluator-fact projection.
3. Legacy structured quarantine: `Literal`, `Reference`, `Binary`, and
   `FunctionCall` variants remain only as migration scaffolding for checked
   in fixtures, unit tests, and scale/demo generators.

New or materially changed formula fixtures should prefer `RawOxfml`.
Structured variants may be touched only to preserve or quarantine existing
coverage, and must not be introduced as a production-facing formula model.

Manifest policy tags:
1. `fixture-policy:opaque-oxfml-source` marks representative fixtures that
   delegate formula syntax to OxFml source text and carry only explicit
   dependency/evaluator facts.
2. `fixture-policy:legacy-structured-quarantine` marks representative
   fixtures that still exercise the quarantined structured migration path.

The Rust policy classifier is `FixtureFormulaAst::policy_class()`.
`treecalc_fixture_policy_tags_match_representative_cases` guards the
representative tags against the fixture expression shape, while
`checked_in_treecalc_fixtures_execute_against_local_runtime` continues to
prove the active corpus executes through the local runtime.
