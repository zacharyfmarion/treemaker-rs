# Settings Theme Support

## Goal

Add a Cascade-inspired settings modal to the shared TreeMaker app and support
the same preset themes that Cascade exposes.

## Approach

- Import Cascade's preset theme definitions into the web app.
- Add a TreeMaker theme store that persists the selected theme and applies
  Cascade theme tokens to TreeMaker CSS variables.
- Build a compact settings modal with an Appearance tab for theme selection and
  a Workspace tab for layout reset.
- Route settings opening through the toolbar, shared menu dispatcher, web menu,
  keyboard shortcut, and Tauri native menu.
- Add focused frontend tests for theme persistence, preset coverage, modal
  rendering, and menu dispatch.

## Affected Areas

- `apps/web/src/themes`
- `apps/web/src/store`
- `apps/web/src/components`
- `apps/web/src/commands`
- `apps/web/src/menus`
- `apps/web/src/styles`
- `apps/tauri/src-tauri/src/menu.rs`

## Checklist

- [x] Add implementation plan and branch.
- [x] Add Cascade preset theme system.
- [x] Build settings modal and command wiring.
- [x] Style modal and theme mappings.
- [x] Add focused tests.
- [x] Run targeted validation and prepare PR handoff.
