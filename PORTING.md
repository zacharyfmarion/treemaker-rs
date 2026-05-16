# Porting Discipline

The canonical source of truth is the vendored TreeMaker 5.0.1 C++ source in
`third_party/treemaker-5.0.1`.

This port should not substitute simpler algorithms for TreeMaker behavior. When
an algorithm has not yet been ported directly from the C++ implementation, the
Rust API must return `TreeError::UnsupportedOperation` instead of fabricating a
nearby result.

Current exact/anchored surface:

- v4 fixture parsing follows the `tmTree_IO.cpp`, `tmNode.cpp`, `tmEdge.cpp`,
  and `tmPath.cpp` stream order for model-tester fixtures.
- v5 writing follows `tmTree::Putv5Self` for the model parts currently
  represented in Rust.
- v4 export follows `tmTree::Putv4Self` for the GUI-free node/edge/path surface.
- Condition stream data is typed and feasibility formulas are direct ports of
  the corresponding `tmCondition*::CalcFeasibility()` methods and
  `tmConstraintFns`.
- `tmNLCO_alm` is ported as `treemaker_core::nlco::NlcoAlm`, including the ALM
  constants, BFGS inner loop, line search, bound handling, and documented C++
  quirks needed by optimizer parity.
- `tmScaleOptimizer`, `tmEdgeOptimizer`, and `tmStrainOptimizer` are ported for
  the headless all-owned-parts usage exercised by `tmModelTester`.

Still to port directly:

- v3 reading.
- `CleanupAfterEdit()` beyond length/feasibility recalculation.
- `BuildTreePolys()`, `BuildPolysAndCreasePattern()`, facet order, and fold
  direction assignment.
- CP-bearing v4/v5 stream payloads; they currently error rather than preserving
  untyped crease-pattern structures.
