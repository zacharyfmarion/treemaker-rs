# Oriedita CP Menu Action Split

## Goal

Separate crease-pattern editing controls into clear UX buckets:

- Persistent canvas tools stay in the left rail.
- Live options, starting with M/V/E/A line type, move to the bottom viewport toolbar.
- One-shot or selection-scoped actions move to the Crease Pattern menu and are enabled only when they can run.

This keeps Oriedita parity visible without making every command feel like a pointer mode.

## Approach

Treat a CP command as a left-rail tool only when the user must keep that mode active and interact with the canvas. Selection-scoped commands such as making selected lines mountain folds, advancing crease type, deleting selected lines, diagnostics, and repairs belong in the Crease Pattern menu.

Parameterized actions still need a home for settings. Menu items such as Replace Selected Line Type, Delete Selected Line Type, Fix Inaccurate Creases, and Change Circle Color should open the existing contextual settings panel from the menu, then apply from that panel. They should not remain rail buttons just because they have inputs.

The bottom toolbar becomes the place for live viewport and edit options. The line type selector should sit there with stable M/V/E/A color identity, next to grid and snapping controls.

## Affected Areas

- CP action and command registry placement.
- Crease Pattern menu definitions for web and Tauri.
- Workspace capability gating for selected-line and selected-circle actions.
- Menu action dispatch, including menu-launched contextual CP actions.
- Crease pattern panel toolbar rendering and menu-action request handling.
- Focused unit tests for registry placement, menu dispatch, capabilities, and panel rendering.

## Checklist

- [x] Move live line type options from the left rail to the bottom toolbar.
- [x] Mark selection-scoped and one-shot CP actions as menu placement, not rail placement.
- [x] Add Crease Pattern menu entries for selected-line actions, diagnostics, repair, and annotation cleanup/color actions.
- [x] Gate menu items by editable CP state and relevant selection counts.
- [x] Route parameterized menu actions into the existing contextual settings panel.
- [x] Update web/Tauri menu definitions and focused tests.
- [x] Run targeted validation without browser smoke tests.
