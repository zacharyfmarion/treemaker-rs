# Fix Delete Node ID Contiguity

## Goal

Fix the Design pane delete-node workflow so removing a selected node preserves
TreeMaker's contiguous 1-based design IDs and does not surface an engine error.

## Approach

- Reproduce the failing path from the shared web delete-selection action into
  the engine edit API.
- Update the engine node-delete remapping so surviving nodes, edges, and
  conditions keep canonical IDs after topology changes.
- Add focused tests for deleting a non-tail node and for the web store applying
  the resulting canonical snapshot.
- Run the smallest validation set covering engine and web behavior.

## Affected Areas

- `crates/treemaker-core/src/lib.rs`
- `apps/web/src/store/workspaceStore/store.test.ts`
- `implementation-plans/fix-delete-node-id-contiguity.md`

## Checklist

- [x] Add implementation plan and branch.
- [x] Root-cause the delete-node invariant failure.
- [x] Fix engine node-delete ID remapping.
- [x] Add or update focused tests.
- [x] Run targeted validation.
- [x] Prepare draft PR handoff.
