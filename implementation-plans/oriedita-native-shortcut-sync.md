# Oriedita Native Shortcut Sync

## Goal

Sync Tauri native menu accelerators with Ori Studio's resolved shortcut
bindings after the browser-first shortcut registry and rebinding UI have landed.

## Approach

- Keep command ownership in the shared web shortcut registry.
- Add a thin Tauri bridge that can update native menu accelerators for menu
  command ids.
- On app startup and whenever shortcut overrides change, send resolved menu
  shortcuts from the web app to the Tauri shell.
- Do not expose CP tool or viewport shortcuts as native menu accelerators unless
  those actions are also represented by native menu items.
- Treat the web shortcut runtime as the owner for CP tools, line-type actions,
  and viewport actions in both browser and desktop. Native Tauri accelerators
  should only handle `target: "menu"` shortcut definitions, then emit the same
  stable `menu-action` ids already used by the shared frontend command layer.
- Account for actions with multiple resolved chords, such as Delete and
  Backspace. The bridge should either set the platform-preferred native
  accelerator and leave aliases to the web runtime, or support multiple native
  menu items only if Tauri exposes a safe first-class path for that.
- Preserve static default accelerators as a fallback when the bridge is
  unavailable.

## Affected Areas

- `apps/web/src/keyboard/`
- `apps/web/src/menus/tauriMenuListener.ts`
- `apps/tauri/src-tauri/src/menu.rs`
- `apps/tauri/src-tauri/src/lib.rs`

## Checklist

- [ ] Add a web-to-Tauri command for resolved menu accelerator updates.
- [ ] Convert resolved `KeyChord` values to Tauri accelerator strings.
- [ ] Update native menu items when overrides change.
- [ ] Keep browser runtime behavior unchanged.
- [ ] Add desktop validation with `npm run check:desktop`.
