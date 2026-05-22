# Folding Sequence ML Readiness

## Goal

Decide when machine learning belongs in the folding-sequence planner.

## Approach

Keep V1 symbolic. The planner must use validated geometry and graph rewrites for
target-state resolution, state validation, and accepted moves. ML is not allowed
to validate geometry, invent folds, or turn unsupported regions into plausible
instructions.

The only candidate ML role is offline ranking of already-valid symbolic
candidates. The runtime can collect stable planner traces, but product behavior
must remain deterministic and symbolic until a trace corpus justifies an
experiment.

## Affected Areas

- `crates/treemaker-sequence` trace schema and replay tests.
- Future offline scripts for ranking experiments, if enough successful traces
  exist.
- The Sequence research UI, if a future debug overlay compares symbolic score
  and experimental ranker score.

## Checklist

- [x] Add a stable trace schema for planner status, score, search stats,
      candidates, and diagnostics.
- [x] Add replay-style tests that compare trace score against current planner
      score.
- [x] Keep production planner behavior independent of ML output.
- [x] Set the initial recommendation to collect more symbolic traces before any
      ML work.
- [ ] Revisit only after at least 500 successful symbolic traces are available.
