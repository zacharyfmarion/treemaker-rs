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
- v3/v4/v5 parsing for the current headless fixture surface.
- Canonical v5 writing and v4 export for the represented model parts.
- Typed tree, node, edge, path, condition, poly, vertex, crease, and facet data.
- Direct ports of condition feasibility formulas.
- Direct Rust port of the ALM nonlinear constrained optimizer.
- Direct ports of the scale, edge-strain, and strain optimizers for the
  headless all-owned-parts scenarios used by the upstream fixtures.
- Direct cleanup-state parity for checked-in fixtures after parse and after
  optimizer-triggered cleanup, including border/polygon/pinned/conditioned
  flags and stale crease-pattern invalidation.
- Direct ports of top-level tree polygon construction and the geometry-only
  inset/subpoly portion of `tmPoly::BuildPolyContents()`, verified against the
  C++ oracle for representative upstream fixtures.
- Direct ports of vertex, crease, facet, corridor-edge, facet-order,
  facet-color, and fold-direction construction for the checked-in oracle
  surface.
- `CPStatus` diagnostic bad-part lists exposed through core, CLI, and wasm.
- CLI and wasm bindings for parsing, checking, optimizing, building crease
  patterns, reporting diagnostics, and saving.

Engine confidence:

- The direct TreeMaker 5.0.1 ALM engine port is confirmed for the current
  supported surface. The app roadmap can treat the engine as a working
  foundation for product work rather than an open confidence risk.
- Ubuntu oracle CI is present. macOS oracle CI remains optional if the extra
  platform-cost tradeoff becomes worth it for tracking ALM floating-point
  drift.
- Keep CFSQP/RFSQP out of scope unless redistributable source and compatible
  license terms are provided. Those backends are not required for TreeMaker
  5.0.1 ALM parity.

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

Status: complete for the checked-in fixture surface. v3 reading is implemented
from the TreeMaker 5.0.1 stream readers, including legacy node/path condition
translation. CP-bearing v5 payloads are represented as typed `Poly`, `Vertex`,
`Crease`, and `Facet` data and round-trip through canonical v5 writing.
CP-bearing v4 payloads are consumed and then discarded to match
`tmTree::GetSelf`, which calls `KillPolysAndCreasePattern()` after reading v4.

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

Status: complete for the checked-in fixture/oracle surface. The Rust cleanup
path now follows the C++ `tmTree::CleanupAfterEdit()` ordering through
condition invalidation, owned path length/feasibility refresh, conditioned
flags, convex-hull border classification, pinned-state calculation, polygon
network pruning, stale poly removal, orphan vertex/crease removal, root
renumbering, cleanup-data clearing, and polygon-filled gating. The depth,
facet-order, color, and fold-direction branches remain tied to the polygon and
crease-pattern construction phases because the Rust port still does not build
those structures.

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

Status: complete for the checked-in oracle surface. Rust now ports
`tmTree::BuildTreePolys()`, `tmPolyOwner::BuildPolysFromPaths()`, polygon
content calculation, cross-path calculation, and the geometry/inset/subpoly
portion of `tmPoly::BuildPolyContents()` through spoke/ridge path creation.
The crease-building code that follows in C++ `BuildPolyContents()` remains in
Phase 5.

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

Status: complete for the checked-in oracle surface. Rust now ports the crease
pattern generation tail of `tmPoly::BuildPolyContents()` and
`tmTree::CleanupAfterEdit()`, including path/node vertices, axial/gusset/ridge/
hinge/pseudohinge creases, facet rings, vertex depth, bend classification,
local-root networks, facet-order graph splicing, facet order values, facet
colors, and crease fold directions. The C++ oracle emits detailed
vertex/crease/facet JSON for this phase, and the Rust oracle test compares
owners, locations, depths, crease/facet references, corridor edges, ordering
links, order, color, fold, and `CPStatus`.

Goal: port the full crease-pattern construction pipeline.

Work items:

- Port vertex and crease generation.
- Port facet generation and facet ownership.
- Port facet order and local-root connectivity checks.
- Port fold direction assignment.
- Implement full `CPStatus` status reporting.
- Expose bad part lists in a future diagnostic API if the CLI/wasm surface
  needs them.

Done when:

- Rust `build_polys_and_crease_pattern()` matches C++ counts and representative
  geometry for vertices, creases, facets, facet order, and fold direction.
- `Tree::cp_status()` matches the C++ oracle for every fixture.

## Phase 6: Public Surface Hardening

Status: complete for the current public surface. The CLI has command-level
coverage for help/version/datadir, inspect, check, optimize, build-cp,
export-v4, and run-fixtures. The wasm crate has `wasm-pack test --node`
coverage for load, summary, optimize, build CP, save, free, and structured
error envelopes. The README documents the ALM-only optimizer baseline and keeps
proprietary CFSQP/RFSQP backends out of scope without redistributable source.

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

## Phase 7: CPStatus Diagnostics

Status: complete for the C++ `GetCPStatus()` diagnostic surface that is
represented by the Rust model. `Tree::cp_status_report()` now returns the
status plus offending edge, poly, vertex, crease, and facet IDs. The CLI exposes
this through `check --details`, and wasm exposes `cp_status_report(handle)` with
the same structured JSON shape. Oracle tests compare diagnostic counts against
the C++ model summaries for the checked-in fixtures.

Goal: expose the bad-part lists behind TreeMaker's CP status checks without
changing the lightweight `Tree::cp_status()` API.

Work items:

- Port the bad-edge, bad-poly, bad-vertex, bad-crease, and bad-facet collection
  logic from `tmTree::GetCPStatus()`.
- Preserve C++ behavior where `POLYS_NOT_VALID` returns only a status and no
  bad part lists.
- Add CLI and wasm accessors for diagnostic consumers.
- Compare diagnostic counts with the C++ oracle on the checked-in fixture set.

Done when:

- `Tree::cp_status_report()` reports the same status and bad-part counts as the
  C++ oracle for every checked-in fixture summary.
- CLI and wasm tests cover the diagnostic report surface.

## Phase 8: Corpus, CI, And Stress Confidence

Status: complete for the checked-in and locally-gated confidence harnesses.
Large/user corpora remain external by design, Ubuntu CI now builds and runs the
C++ ALM oracle, generated families cover broader topology/condition cases, and
bounded parser/model stress tests exercise valid random trees plus malformed
fixture mutations.

Goal: harden confidence in the completed TreeMaker 5.0.1 ALM port without
checking private or large user files into git.

Work items:

- Add Ubuntu GitHub Actions CI for native Rust checks and the C++ ALM oracle.
- Refactor oracle-test comparison helpers so generated and corpus tests reuse
  the same detailed Rust/C++ assertions as the official fixture tests.
- Add deterministic generated tree-family oracle tests for deeper branching,
  many terminals, symmetry, pinned/conditioned parts, border/corner constraints,
  angle constraints, and intentionally diagnostic CP states.
- Add a corpus CLI and gated integration test that recursively scans external
  `.tmd`, `.tmd4`, and `.tmd5` directories, de-duplicates by SHA-256, parses
  with Rust, round-trips through canonical v5, and optionally compares C++
  oracle summaries.
- Add bounded fuzz/stress tests for valid-by-construction trees and malformed
  fixture mutations.
- Document corpus handling, CI commands, added dependencies, and licensing
  implications for any new crates.

Done when:

- Ubuntu CI runs `cargo fmt --check`, clippy, workspace tests, C++ oracle build,
  and oracle parity tests.
- Generated family tests compare detailed polygon, vertex, crease, facet,
  order, color, fold, and `CPStatus` data against the C++ oracle.
- The corpus command and `TREEMAKER_CORPUS_DIR`-gated test can validate a local
  private corpus without committing corpus files.
- Stress tests are deterministic and CI-bounded.
- The full local validation set in the README passes.
