# CP Symmetry Command Redo

## Goal

Make crease-pattern symmetry behavior consistent across canvas tools, menu
actions, contextual actions, and keyboard delete while keeping mirrored
selection as an explicit opt-in preference that defaults off.

## Approach

- Add a CP symmetry `mirrorSelection` setting and expose it in the symmetry menu.
- Centralize command payload validation and symmetry batching in the CP
  symmetry helper used by the workspace store.
- Treat selection tools separately from selection-scoped edit commands so
  symmetric edits can affect mirrored geometry without visibly selecting it.
- Harden command payloads before wasm execution so malformed payloads fail in
  the frontend with a friendly error.
- Cover the command gateway, menu routing, UI setting, and native project
  migration with focused tests.

## Affected Areas

- `apps/web/src/lib/oristudioCpSymmetry.ts`
- `apps/web/src/store/workspaceStore`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/lib/nativeProjectFile.ts`
- Focused web tests for CP symmetry, menu actions, store behavior, panel UI, and
  native project migration.

## Checklist

- [x] Add CP symmetry state and UI for opt-in mirrored selection.
- [x] Centralize CP command preparation and payload validation.
- [x] Preserve symmetric edit behavior for menu/key/context/tool commands.
- [x] Add regression tests for selected-line edit commands and selection tools.
- [x] Add persistence/migration and panel coverage for `mirrorSelection`.
- [x] Run targeted Vitest files, then web lint, typecheck, and tests.
