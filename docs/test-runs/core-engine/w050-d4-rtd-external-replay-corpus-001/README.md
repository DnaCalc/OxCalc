# W050 D4 RTD / External Replay Corpus

Run id: `w050-d4-rtd-external-replay-corpus-001`

Purpose: pin the first deterministic OxCalc-local replay corpus for external
topic updates across all three `StreamSemanticsVersion` settings:
`ExternalInvalidationV0`, `TopicEnvelopeV1`, and `RtdLifecycleV2`.

Fixture:
- `fixture.json` records the topic subscription surface and topic update
  sequence.

Run artifact:
- `run_artifact.json` records the normalized publication result for each
  stream selector and the validation commands.

Validation commands:
- `cargo test -p oxcalc-core rtd_external_replay_corpus -- --nocapture`
- `cargo test -p oxcalc-core stream_semantics -- --nocapture`
- `cargo test -p oxcalc-core`

Result: all three selector profiles publish the same normalized values for the
recorded topic sequences. External update dispatch remains invalidation-only;
publication still occurs through `TreeCalcCoordinator`.
