# Oriedita CP Always-On CAMV Diagnostics

## Goal

Keep CAMV/theorem diagnostics visible by default while editing crease patterns.

## Approach

- Store the latest automatic CAMV result separately from `lastCommandResult`.
- Refresh CAMV after loading an editable CP, after mutating CP commands, and
  after CP undo/redo restores.
- Do not mark the document dirty, push undo history, or steal viewport focus
  when the automatic CAMV refresh runs.
- Render persistent CAMV markers and issue HUD even when the latest command
  result is an ordinary edit. Suppress the floating HUD for automatic CAMV OK
  results so the canvas only calls out actual issues.
- Preserve explicit non-CAMV diagnostic command results so Check1/2/3/4 can
  still be inspected separately.

## Affected Areas

- `apps/web/src/store/workspaceStore/types.ts`
- `apps/web/src/store/workspaceStore/slices/projectSlice.ts`
- `apps/web/src/store/workspaceStore/slices/historySlice.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/DiagnosticsPanel.tsx`
- `apps/web/src/store/workspaceStore/store.test.ts`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Add persistent CAMV result state.
- [x] Refresh CAMV after load, mutating commands, undo, and redo.
- [x] Render persistent CAMV overlays and issue HUD independently from edit command
      results.
- [x] Keep automatic refresh out of history, dirty, and viewport focus changes.
- [x] Add focused tests.
- [x] Run non-browser validation and commit.
