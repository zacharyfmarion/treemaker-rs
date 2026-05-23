# Oriedita CP Stage 9 Flat-Foldable Boundary Check

## Goal

Enable Oriedita's flat-foldable boundary check from the CP tool rail using the
existing oracle-tested Rust check implementation.

## Approach

- Treat FlatFoldableCheck as a non-mutating diagnostic command.
- Use the existing drag-path input mode as the first UI workflow: users draw a
  boundary loop and release near the start point to close it.
- Convert the resolved model-space path into yellow Oriedita boundary segments,
  then call `checks::flat_foldable_boundary_check`.
- Return one structured diagnostic entry containing the checked boundary
  segments with Oriedita's result color: cyan for flat-foldable, magenta for not
  flat-foldable, and yellow for invalid boundary intersections or incomplete
  input.
- Keep this as the initial boundary workflow. A closer multi-click/multi-drag
  reproduction of `MouseHandlerFlatFoldableCheck` can still be layered on later
  if manual testing shows the drag-path version is not close enough.

## Affected Areas

- `crates/oristudio-cp/src/lib.rs`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/DiagnosticsPanel.tsx`
- `apps/web/src/styles/theme.css`
- `apps/web/src/lib/oristudioCpCommands.test.ts`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `apps/web/src/components/panels/DiagnosticsPanel.test.tsx`
- `apps/web/src/store/workspaceStore/slices/projectSlice.ts`
- `apps/web/src/store/workspaceStore/store.test.ts`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Add Rust command dispatch for FlatFoldableCheck.
- [x] Keep FlatFoldableCheck out of undo history and dirty tracking.
- [x] Mark FlatFoldableCheck ready with drag-path UX metadata.
- [x] Render boundary-check diagnostic colors in the CP overlay and diagnostics
      list.
- [x] Add focused Rust and web tests.
- [x] Run non-browser validation and commit the slice.
