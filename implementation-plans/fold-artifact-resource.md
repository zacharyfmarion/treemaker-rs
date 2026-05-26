# Fold Artifact Resource

## Goal

Move folded-base, simulator, and sequence artifact lifecycle into the workspace
store so document changes, undo/redo, and visible panels share one reliable
derived-resource state machine.

## Approach

- Add explicit artifact status, revision, resolved revision, and request id
  fields to the crease-pattern store slice.
- Route every tree/CP document mutation through shared artifact invalidation
  instead of manually clearing `foldArtifacts` in each caller.
- Make `ensureFoldArtifacts` and `refreshFoldArtifacts` own async loading,
  errors, partial artifact payloads, and stale-result guards.
- Keep panels focused on rendering store state; effects may request artifacts
  when visible but must not own loading/error truth.

## Affected Areas

- Workspace store types and crease-pattern/history/project/editing/clipboard
  slices.
- Folded Base, Simulator, and Sequence panels.
- Frontend store and panel tests covering artifact invalidation, refresh, and
  error display.

## Checklist

- [x] Add derived artifact resource state and helper actions.
- [x] Replace scattered artifact invalidation with shared stale-marking.
- [x] Route CP undo/redo through the same document-changed invalidation path.
- [x] Remove panel-owned artifact loading state.
- [x] Add tests for unsolvable CP errors, undo/redo recompute, and stale async responses.
- [x] Run focused web validation.
