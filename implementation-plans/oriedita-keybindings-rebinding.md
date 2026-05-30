# Oriedita Keybindings And Rebinding

## Goal

Port Oriedita's keybinding defaults and rebinding workflow into Ori Studio while
keeping the shared browser and Tauri app on one extensible command architecture.

The result should let users inspect, search, clear, reset, and rebind shortcuts
for global commands, editable crease-pattern tools, viewport actions, and future
features without hard-coding shortcuts in individual components. Browser runtime
must handle reserved or unreliable browser shortcuts explicitly instead of
pretending every desktop key chord can be captured.

## Approach

### Oriedita Reference Behavior

The relevant Oriedita pieces are:

- `third_party/oriedita/oriedita/src/main/resources/hotkey.properties`
  stores default shortcuts by action key. Empty values mean no shortcut.
- `third_party/oriedita/oriedita/src/main/resources/categories.csv`
  groups actions for the preferences hotkey table.
- `third_party/oriedita/oriedita-ui/src/main/java/oriedita/editor/action/ActionType.java`
  is the canonical action-key enum used by the button service.
- `third_party/oriedita/oriedita/src/main/java/oriedita/editor/service/impl/ButtonServiceImpl.java`
  owns the action-to-keystroke map, updates Swing input maps, updates tooltips,
  tracks conflicts with a bidirectional map, and ignores shortcuts while text
  fields are focused.
- `third_party/oriedita/oriedita-ui/src/main/java/oriedita/editor/swing/dialog/SelectKeyStrokeDialog.java`
  captures a key stroke, blocks conflicting assignments, allows clearing, and
  writes user overrides.
- `third_party/oriedita/oriedita-ui/src/main/java/oriedita/editor/swing/dialog/PreferenceDialog.java`
  renders the searchable hotkey table, per-row reset buttons, assigned-only
  filtering, and reset-all behavior.
- `third_party/oriedita/oriedita-common/src/main/java/oriedita/editor/tools/ResourceUtil.java`
  resolves defaults from bundled resources and overrides from the user config
  directory.

Oriedita's model is worth copying conceptually, but not literally. Swing input
maps and Java `KeyStroke` strings should become typed browser/Tauri shortcut
data, and component-local listeners should read from one registry.

### Shortcut Registry

Add a new frontend shortcut domain, likely under `apps/web/src/keyboard/`.

Core types:

- `ShortcutActionId`: a stable string id for anything bindable.
  Menu actions can use existing `MenuActionId` values. CP tool actions can use
  existing `OristudioCpActionId` values. Viewport-only actions should get stable
  ids such as `viewport.zoomIn`, `viewport.fit`, and `viewport.actualSize`.
- `ShortcutScope`: `global`, `design`, `crease-pattern`, `viewport`, `modal`,
  and future scopes. Scope priority decides what wins when the same chord is
  valid in multiple places.
- `KeyChord`: normalized modifier and key data. Use a `primary` modifier for
  Cmd-on-mac and Ctrl-elsewhere, while still supporting explicit `ctrl`, `meta`,
  `alt`, and `shift`.
- `ShortcutDefinition`: label, category, scope, default chord, optional
  platform defaults, Oriedita `upstreamAction`, enabled status, conflict policy,
  reserved-key policy, and display metadata.
- `ShortcutBindingState`: persisted user overrides only, not a full copy of
  defaults. A missing override means "use current default"; an explicit `null`
  means "disabled".

Build the registry from existing source-of-truth metadata:

- Global commands from `MENU_ACTION_IDS` and `getMenuBarDef`.
- CP tool and line-type actions from `ORISTUDIO_CP_ACTIONS`, using their
  existing `upstreamAction` fields to map Oriedita defaults.
- Viewport commands currently handled in `DesignPanel` and
  `CreasePatternPanel`, especially zoom in/out, fit, actual size, and
  temporary pan mode.
- Non-rebindable safety commands such as modal Escape handling can remain
  component-local, but should be documented as intentionally not bindable.

### Oriedita Defaults Import

Create a small deterministic importer or checked-in generated map from:

- `hotkey.properties`
- `categories.csv`

The importer should parse Oriedita Java keystroke strings like `ctrl shift V`,
`DELETE`, and `F` into `KeyChord` values. It should also emit diagnostics for:

- Oriedita actions that have no Ori Studio equivalent yet.
- Ori Studio bindable actions with no Oriedita source action.
- Duplicate Oriedita defaults, such as legacy or overlapping actions.
- Browser-reserved defaults that need a different web effective default.

Initial mapping should include at least:

- `newAction` -> `file.new`
- `openAction` -> `file.open`
- `saveAction` -> `file.save`
- `saveAsAction` -> `file.saveAs`
- `prefAction` -> `file.settings`
- `exitAction` -> `app.quit` in desktop only
- `undoAction` / `redoAction` -> `edit.undo` / `edit.redo`
- `copyClipboardAction`, `cutClipboardAction`, `pasteClipboardAction` ->
  clipboard edit commands
- CP line colors: `colRedAction`, `colBlueAction`, `colBlackAction`,
  `colCyanAction`
- CP tools through existing `upstreamAction` metadata, for example
  `angleBisectorAction`, `perpendicularDrawAction`, `symmetricDrawAction`,
  `foldableLineDrawAction`, `fishBoneDrawAction`, and related Oriedita tool
  ids.

The registry should prefer Oriedita defaults where they are safe and scoped.
Existing Ori Studio shortcuts that conflict with Oriedita defaults should be
resolved intentionally. For example, current `L` and `V` CP tool shortcuts
conflict with Oriedita line-color defaults where `L` is black/edge and `V` is
valley.

### Browser Reserved Keys

Add a reserved-key catalog in the shortcut layer. It should classify chords by
runtime and platform:

- `hard-reserved`: the app cannot reliably intercept or should never steal it,
  such as browser location/tab/window controls and OS-level quit shortcuts.
- `soft-reserved`: the app may intercept it in some browsers, but users should
  see a warning because the browser or assistive tooling may still win.
- `text-editing`: shortcuts that should not fire while an input, textarea,
  select, or contenteditable element is focused.
- `allowed`: safe for app dispatch.

Policy:

- Browser defaults should be an effective browser-safe profile, not a blind copy
  of every desktop Oriedita chord.
- Tauri desktop can use a more native/Oriedita-like profile because native menu
  accelerators are available.
- Users can assign soft-reserved chords after a warning.
- Hard-reserved chords should be blocked in browser runtime.
- Single-letter CP tool shortcuts should only work when the CP editing surface
  is focused, never while typing into fields.
- `Space` for temporary pan should stay a viewport gesture and remain
  non-rebindable unless a later interaction design needs it.

This is especially important for Oriedita defaults like `ctrl R`, which means a
tool in Oriedita but refresh in browsers, and for current app shortcuts such as
`CmdOrCtrl+B`, which conflict with Oriedita `rabbitEarAction`.

### Dispatch Architecture

Replace `apps/web/src/lib/appKeyboard.ts` with a shortcut dispatcher that:

- Normalizes `KeyboardEvent` to `KeyChord`.
- Checks `event.defaultPrevented`, composition state, editable targets, and
  active modal state.
- Resolves the active shortcut scope stack from app state and focused editor
  surface.
- Finds the highest-priority enabled binding for that chord.
- Checks workspace capabilities before executing command-backed shortcuts.
- Calls one executor interface instead of component-specific hard-coded
  branches.

Execution targets:

- `MenuActionId` -> existing `handleMenuAction`.
- CP line-type action -> update active CP line type.
- CP command action -> activate the CP tool or execute the command depending on
  action placement and status.
- Viewport action -> active viewport controller callback.

The design and CP panels should register viewport-specific handlers with the
shortcut manager instead of adding their own `Cmd/Ctrl +/-/0/1` listeners. Escape
tool cancellation can remain panel-local until it is intentionally made
rebindable.

### Settings UI

Add a `Shortcuts` tab to `SettingsModal`.

The tab should be a compact editor-oriented table, not a marketing screen:

- Search by action label, category, shortcut, and Oriedita action id.
- Category sections based on Oriedita `categories.csv`, with Ori Studio-only
  categories for viewport and app-specific commands.
- Assigned-only toggle.
- Rows with icon, label, current shortcut, default shortcut, scope, status, and
  reset button.
- Capture dialog or inline capture state with clear, cancel, and confirm.
- Conflict message that names the existing action.
- Reserved-browser warning that distinguishes hard block from soft warning.
- Reset row, reset category, and reset all shortcuts actions.

Also update:

- Web menu shortcut labels to read from resolved bindings.
- CP rail and toolbar tooltips to include resolved shortcuts.
- Help content to reference the Shortcuts settings tab.
- Optional later polish: hold Alt/Option to show shortcut badges over visible
  tool buttons, inspired by Oriedita's Alt help overlay.

### Persistence

Use localStorage first because the shared web app already uses it for theme and
layout.

Suggested key:

- `oristudio-shortcuts-v1`

Store only user overrides:

```json
{
  "version": 1,
  "bindings": {
    "file.save": { "chord": { "primary": true, "key": "s" } },
    "cp.action.angle-bisector": null
  }
}
```

When defaults evolve, unchanged actions receive new defaults automatically while
user overrides are preserved. Unknown saved action ids should be ignored but not
crash loading.

A later enhancement can import/export JSON or import an Oriedita
`hotkey.properties` file by mapping known action ids and reporting skipped
entries.

### Tauri Menu Accelerators

Native menu accelerators in `apps/tauri/src-tauri/src/menu.rs` currently repeat
shortcut strings by hand. They should eventually be driven by the same resolved
bindings used by web menus.

Implementation options:

- Generate a shared static default shortcut manifest from TypeScript source for
  the Rust menu builder at build time.
- Or add a Tauri command that receives resolved menu accelerators from the web
  frontend after startup and calls Tauri menu item accelerator updates.

The important rule is that Tauri remains a thin shell. It can own native menu
accelerators, but the command ids and user override semantics should remain in
the shared frontend shortcut registry.

### Validation

Unit tests should cover:

- Oriedita hotkey parsing and formatting.
- Browser/Tauri display formatting, including primary modifier labels.
- Conflict detection and clear/reset behavior.
- Reserved-key classification.
- Loading, saving, versioning, and ignoring malformed localStorage data.
- Dynamic menu shortcut labels.
- Global and scoped shortcut dispatch.
- CP action shortcut mapping through `upstreamAction`.
- Settings UI capture, conflict, reserved-key, clear, and reset flows.

Run the smallest validation set for each implementation step:

- Shortcut core and UI: `npm run lint:web`, `npm run typecheck:web`,
  `npm run test:web`.
- Tauri menu accelerator changes: `npm run check:desktop`.
- Docs-only plan changes: `git diff --check`.

## Affected Areas

- `apps/web/src/keyboard/` or equivalent new shortcut core.
- `apps/web/src/store/shortcutStore.ts`.
- `apps/web/src/lib/appKeyboard.ts`.
- `apps/web/src/commands/menuActions.ts`.
- `apps/web/src/menus/menuDefinition.ts`.
- `apps/web/src/components/SettingsModal.tsx`.
- `apps/web/src/components/MenuBar.tsx`.
- `apps/web/src/components/panels/CreasePatternPanel.tsx`.
- `apps/web/src/components/panels/DesignPanel.tsx`.
- `apps/web/src/components/panels/CpToolRail.tsx`.
- `apps/web/src/lib/oristudioCpActions.ts`.
- `apps/web/src/lib/oristudioCpCommands.ts`.
- `apps/tauri/src-tauri/src/menu.rs`.
- `apps/tauri/src-tauri/src/lib.rs` if a menu accelerator update command is
  added.
- Existing web and desktop tests around keyboard, menus, settings, and CP tool
  actions.

## Checklist

- [x] Add shortcut core types, key normalization, parser, formatter, and tests.
- [x] Add Oriedita hotkey/category import data with diagnostics for unmapped,
      duplicate, and reserved defaults.
- [x] Add `shortcutStore` with persisted overrides, clear/reset operations, and
      migration-safe loading.
- [x] Define the bindable action registry for menu commands, CP tools, line
      types, and viewport commands.
- [x] Replace hard-coded global keyboard routing with scoped shortcut dispatch.
- [x] Move design and CP viewport zoom shortcuts onto the shared dispatcher.
- [x] Reconcile current Ori Studio defaults with Oriedita defaults and document
      intentional browser-safe deviations.
- [x] Update web menu labels, CP rail labels/tooltips, and help copy to read
      resolved shortcuts.
- [x] Add the Settings Shortcuts tab with search, assigned-only filter,
      capture, clear, per-row reset, reset-all, conflict handling, and
      reserved-key warnings.
- [x] Create a follow-up plan for Tauri native menu accelerator sync.
- [x] Run web validation.
- [x] Skip desktop validation because this PR does not change Tauri runtime code.
