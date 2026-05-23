# Oriedita CP Stage 9 Issue Navigation

## Goal

Make structured Oriedita diagnostic results navigable between the Diagnostics
panel and the editable crease pattern pane.

## Approach

- Store the currently focused diagnostic ID in the shared workspace state.
- Auto-focus the first issue after a non-mutating check command and clear the
  focus after mutating edits, document loads, undo, redo, or clear operations.
- Render active diagnostic markers distinctly in the CP canvas.
- Let users click a Diagnostics panel issue or a canvas diagnostic marker to
  focus the same issue and activate the CP pane.
- Center the CP viewport on the focused diagnostic geometry when the CP pane is
  mounted and has measurable viewport bounds.

## Affected Areas

- `apps/web/src/store/workspaceStore/types.ts`
- `apps/web/src/store/workspaceStore/slices/creasePatternSlice.ts`
- `apps/web/src/store/workspaceStore/slices/projectSlice.ts`
- `apps/web/src/store/workspaceStore/slices/historySlice.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/DiagnosticsPanel.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `apps/web/src/components/panels/DiagnosticsPanel.test.tsx`
- `apps/web/src/store/workspaceStore/store.test.ts`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Add shared active diagnostic state and reset rules.
- [x] Select the first diagnostic after check commands and clear focus after
      document mutations/restores.
- [x] Add clickable issue rows in the Diagnostics panel.
- [x] Add clickable active diagnostic markers in the CP canvas.
- [x] Center the CP viewport on focused diagnostics.
- [x] Add focused store and React tests.
- [x] Run non-browser validation and commit the slice.
