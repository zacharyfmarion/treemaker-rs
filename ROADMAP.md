# TreeMaker Rust Port Roadmap

This repository is a direct Rust/WASM port of the model side of TreeMaker
5.0.1. The vendored source in `third_party/treemaker-5.0.1` is the behavioral
baseline. Later TreeMaker branches may be used only to identify documented bug
fixes; they are not allowed to redefine expected behavior.

The porting rule is simple: when C++ behavior has not been ported and verified,
the Rust API must say so explicitly. It should not approximate missing geometry,
optimizer, or serialization behavior.

## Current Baseline

Implemented:

- Rust workspace with `treemaker-core`, `treemaker-cli`, `treemaker-wasm`, and
  `oracle-tests`.
- Vendored TreeMaker 5.0.1 GPL source and model-tester fixtures.
- v4/v5 parsing for the current model-only fixture surface.
- Canonical v5 writing and v4 export for the represented model parts.
- Typed tree, node, edge, path, and condition data.
- Direct ports of condition feasibility formulas.
- Direct Rust port of the ALM nonlinear constrained optimizer.
- Direct ports of the scale, edge-strain, and strain optimizers for the
  headless all-owned-parts scenarios used by the upstream fixtures.
- CLI and wasm bindings for parsing, checking, optimizing, and saving.

Explicitly incomplete:

- v3 reading.
- CP-bearing v4/v5 stream payloads.
- Full `tmTree::CleanupAfterEdit()` lifecycle.
- Tree polygon, subpoly, crease, vertex, facet, facet order, and fold direction
  construction.
- Full `CPStatus` parity beyond the currently represented model surface.

## Phase 1: Oracle Foundation

Status: complete for the checked-in model-tester fixtures. The oracle harness
builds and the gated Rust parity test runs. Two scale-optimizer records
(`tmModelTester_2.tmd5` and `tmModelTester_3.tmd5`) are treated as visible
local-compiler ALM drift on Apple Clang 17 rather than Rust parity gates; the
shipped `tmModelTester.out.txt` remains the historical golden for those values.

Goal: make the canonical C++ 5.0.1 model executable from this repository without
wxWidgets, GUI code, CFSQP, RFSQP, or WNLIB, and use it as the fixture oracle for
the Rust port.

Work items:

- Add a model-only C++ oracle harness under `tools/oracle`.
- Compile only the vendored model sources needed for the default ALM backend.
- Emit stable JSON lines for fixture summaries and optimizer results.
- Run the oracle against `tests/fixtures/tmModelTester_1.tmd5` through
  `tmModelTester_5.tmd5`.
- Add a gated Rust integration test so CI can compare against a locally built
  oracle without making every contributor compile C++ by default.

Done when:

- `tools/oracle/build_oracle.sh` builds a native `treemaker-oracle` binary.
- `treemaker-oracle run-fixtures --fixture-dir tests/fixtures` prints stable
  parse and optimizer records for every checked-in fixture.
- The Rust test suite still passes without requiring the C++ oracle.

## Phase 2: Stream I/O Completeness

Goal: make the Rust parser/writer cover the same file format surface as
TreeMaker 5.0.1 for headless model data.

Work items:

- Port v3 reading directly from the C++ stream readers.
- Represent CP-bearing v4/v5 payloads with typed Rust data instead of rejecting
  them.
- Preserve 1-based external indices and TreeMaker stream ordering exactly.
- Expand golden tests to include v3, v4, v5, and CP-bearing round trips.

Done when:

- Every upstream `.tmd`, `.tmd4`, and `.tmd5` fixture that the C++ model can
  read is readable by Rust.
- Rust canonical summaries match C++ oracle summaries after round trips.

## Phase 3: Cleanup Lifecycle

Goal: port `tmTree::CleanupAfterEdit()` and all invalidation/rebuild state that
drives model feasibility and crease-pattern readiness.

Work items:

- Port all cleanup-stage flag updates in C++ order.
- Port tree path rebuilding, path length recalculation, active/border/polygon
  path classification, node/edge/path condition marking, and pinned state.
- Add oracle comparisons for feasibility, active paths, pinned edges/nodes, and
  condition state after edits and optimizations.

Done when:

- Rust cleanup state matches the C++ oracle for all checked-in fixtures after
  parsing and after each optimizer.

## Phase 4: Polygon And Subpoly Construction

Goal: port TreeMaker polygon generation directly from the C++ geometry code.

Work items:

- Port `BuildTreePolys()`.
- Port convex hull and border path logic.
- Port subpoly creation, inset node/path logic, and polygon validity checks.
- Add geometry tolerance tests against oracle counts and representative
  coordinates.

Done when:

- Rust `build_tree_polys()` succeeds on the same fixtures as C++ and reports
  matching polygon/subpoly structure within numeric tolerances.

## Phase 5: Crease Pattern Generation

Goal: port the full crease-pattern construction pipeline.

Work items:

- Port vertex and crease generation.
- Port facet generation and facet ownership.
- Port facet order and local-root connectivity checks.
- Port fold direction assignment.
- Implement full `CPStatus` reporting with bad part lists.

Done when:

- Rust `build_polys_and_crease_pattern()` matches C++ counts and representative
  geometry for vertices, creases, facets, facet order, and fold direction.
- `Tree::cp_status()` matches the C++ oracle for every fixture.

## Phase 6: Public Surface Hardening

Goal: make the port reliable as a native CLI library and wasm package.

Work items:

- Expand CLI snapshots for inspect, check, optimize, build-cp, export-v4, and
  run-fixtures.
- Expand wasm tests under Node with the same error codes/messages as native.
- Keep wasm-compatible code free of filesystem, threading, locale, and platform
  dependencies inside `treemaker-core`.
- Document unsupported proprietary optimizer backends as permanently out of
  scope unless redistributable source and license terms are provided.

Done when:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `wasm-pack build crates/treemaker-wasm --target bundler`
- Oracle parity tests pass for the checked-in fixtures when the C++ oracle is
  enabled.
