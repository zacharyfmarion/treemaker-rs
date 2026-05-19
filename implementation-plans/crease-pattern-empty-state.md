# Crease Pattern Empty State

## Goal

Make the crease-pattern workflow clearer by showing an empty state in the CP
pane and preventing Build CP until a design has been successfully optimized.

## Approach

- Add a shared frontend workflow availability helper for Optimize Scale and
  Build CP.
- Use the helper in the toolbar, CP pane, folded-base pane, simulator pane, and
  store action guard.
- Update the CP pane to show a centered empty state with the next workflow
  action: Optimize Scale before optimization, then Build CP once optimized.
- Keep this as shared web UI behavior with no Rust, WASM, or Tauri-native menu
  changes.

## Affected Areas

- `apps/web/src/App.tsx`
- `apps/web/src/components/panels`
- `apps/web/src/store/workspaceStore/slices/creasePatternSlice.ts`
- `apps/web/src/lib`
- `apps/web/src/styles`

## Checklist

- [x] Add implementation plan and branch.
- [x] Add tested workflow availability helper.
- [x] Guard Build CP in the shared workspace action.
- [x] Add CP pane empty state and update related workflow buttons.
- [x] Validate with web lint, typecheck, and tests.
