# Sequence Planner Speed

## Goal

Improve folding-sequence planning latency without weakening correctness: accepted
states must still come from validated flat-fold target solves, and unsupported
or unfinished behavior must remain explicit instead of being approximated.

## Approach

Benchmark current planner behavior on representative local dataset FOLD files,
including converted CP corpus examples. Then remove duplicate work in two
quality-preserving places: reuse target-state resolutions inside the Rust
planner search, prune candidate states whose crease-assignment keys are already
queued or explored, and expose a combined WASM API so the web app does not solve
the same target once for analysis and once for planning.

## Affected Areas

- `crates/treemaker-sequence` planning search, stats, tests, and benchmark
  example.
- `crates/treemaker-wasm` sequence bindings.
- `apps/web/src/workers` sequence worker API.
- `apps/web/src/store/workspaceStore` sequence planning flow and tests.

## Checklist

- [x] Add a repeatable planner benchmark command.
- [x] Record baseline timings on local dataset examples.
- [x] Add planner target-state caching and duplicate candidate pruning.
- [x] Add a combined target-plus-plan WASM API and route web planning through it.
- [x] Add focused Rust and web unit tests.
- [x] Record after timings and report speedup.
- [x] Run formatting, tests, lint/typecheck/build coverage for changed areas.
