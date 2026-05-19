# Fold Artifact Topology Debug

## Goal

Explain why the supplied model cannot provide simulator-ready fold artifacts and make the UI-facing artifact path report that cause without blocking folded-base data.

## Approach

Reproduce the failing FOLD artifact generation, compare Rust output with the vendored TreeMaker 5.0.1 oracle, inspect the generated crease/facet topology around the reported edge and bad facets, and separate folded-base artifacts from simulator-only artifacts.

## Root Cause

The model rebuilds to `facets_not_valid` in both Rust and the TreeMaker 5.0.1 C++ oracle. TreeMaker identifies odd-degree interior vertices 3 and 20 plus facets 13, 18, 21, and 23 as invalid. Those facets are sliver facets with no axial or gusset bottom crease, so TreeMaker stops before facet ordering, facet coloring, and crease assignment. Because fold assignment never runs, all exported creases still have flat fold values. Edge 60 was only the first malformed flat helper edge that the strict simulator-prep path rejected.

## Affected Areas

- `crates/treemaker-fold`
- `crates/treemaker-core`
- `apps/web`

## Checklist

- [x] Reproduce the failure using the supplied `.tmd5` file.
- [x] Identify why reported edge 60 lacks two adjacent triangular faces.
- [x] Compare against the TreeMaker 5.0.1 C++ oracle.
- [x] Implement a focused artifact/error-surface fix.
- [x] Add or update regression coverage.
- [x] Run targeted validation.
