# CAMV Angle Tolerance Setting

## Goal

Let users tune the angular tolerance used by the editable crease-pattern CAMV
flat-foldability check while preserving Oriedita-compatible defaults.

## Approach

- Keep the current Oriedita epsilon as the default CAMV angle tolerance.
- Add an optional CAMV angle-tolerance command payload field in the
  `oristudio-cp` kernel so web callers can opt into a looser check.
- Persist a Diagnostics settings value in the shared web settings store and
  expose it in the settings modal.
- Pass the setting into explicit Check4/CAMV commands and into always-on CAMV
  refreshes after load, mutation, undo, and redo.
- Add focused kernel and web tests for the tolerance behavior and settings
  wiring.

## Affected Areas

- `crates/oristudio-cp/src/checks.rs`
- `crates/oristudio-cp/src/lib.rs`
- `apps/web/src/engine/oristudioCpTypes.ts`
- `apps/web/src/store/settingsStore.ts`
- `apps/web/src/store/workspaceStore/slices/projectSlice.ts`
- `apps/web/src/store/workspaceStore/slices/historySlice.ts`
- `apps/web/src/components/SettingsModal.tsx`
- `apps/web/src/styles/theme.css`
- Focused Rust and web tests

## Checklist

- [x] Inspect existing CAMV diagnostics and settings patterns.
- [x] Add kernel command support for an optional CAMV angle tolerance.
- [x] Add persisted settings UI and web command payload wiring.
- [x] Add focused regression tests.
- [x] Regenerate generated wasm artifacts as needed.
- [x] Run local validation and prepare PR handoff.
