# Shortcut Runtime Dispatch

## Goal

Replace focus-fragile panel keyboard routing with a browser-first shortcut
runtime that derives active scopes from app state, preserves existing global
commands such as undo/redo/delete, and leaves Tauri native accelerator syncing
as a menu-only follow-up.

## Approach

- Add a shared shortcut runtime that centralizes event dispatch, computes scope
  stacks from `activeEditingSurface`, and lets panels register viewport or CP
  action executors.
- Keep component-local keyboard handling only for intentionally non-rebindable
  gestures such as Escape tool cancellation and Space pan.
- Track active editing surface from capture-phase panel interactions so stopped
  child events do not leave the runtime in the wrong scope.
- Support multiple default chords per action for platform aliases such as
  Delete and Backspace.
- Keep Tauri's native side scoped to stable menu command accelerators; CP tools
  and viewport actions stay in the web runtime.

## Affected Areas

- `apps/web/src/keyboard/`
- `apps/web/src/lib/appKeyboard.ts`
- `apps/web/src/App.tsx`
- `apps/web/src/components/panels/DesignPanel.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/SettingsModal.tsx`
- `implementation-plans/oriedita-native-shortcut-sync.md`

## Checklist

- [x] Add central shortcut runtime and executor registration.
- [x] Change shortcut definitions and store helpers to support chord aliases.
- [x] Move viewport and CP action dispatch out of panel-local keydown listeners.
- [x] Install app shortcut routing on document capture so focused panels cannot
  strand platform editing chords.
- [x] Add capture-phase active-surface tracking for design and CP panels.
- [x] Update settings/menu/tooltips/tests for multiple shortcut labels.
- [x] Make undo/redo resolve to a fallback editable history stack when the
  active surface is stale.
- [x] Keep standard undo/redo default chords available when persisted overrides
  are stale or cleared.
- [x] Clarify the native Tauri accelerator sync plan.
- [x] Validate with web checks and browser smoke tests.
