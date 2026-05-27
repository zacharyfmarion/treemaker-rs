# CP Copy Paste Transform

## Goal

Add app-level copy and paste for selected lines in the editable crease-pattern
editor, with `Cmd/Ctrl+C` and `Cmd/Ctrl+V` working through the existing shared
Edit command path. Selected crease-pattern contents should get a subtle
Photoshop/Affinity-style transform affordance: a dotted selection bounds
rectangle, compact contextual controls, and resize/rotate handles.

Generated TreeMaker crease patterns remain read-only artifacts. This feature
targets editable Oristudio CP documents loaded in `documentMode ===
'crease-pattern'` with an active `oristudioCpDocument`.

## UX

- `Cmd/Ctrl+C` copies the selected CP line geometry by value.
- `Cmd/Ctrl+V` inserts an offset copy near the source selection, selects the
  newly inserted lines, and records one undo checkpoint.
- Any active editable CP selection gets a subtle dotted oriented bounds
  rectangle around its selected contents. For the first implementation, the
  transformable content is selected line segments; the overlay should be
  designed so circles and text can join the same selection model later.
- A compact contextual menu appears near the selection bounds with transform
  actions:
  - flip horizontal
  - flip vertical
  - rotate left 90 degrees
  - rotate right 90 degrees
- Corner-adjacent rotate zones free-rotate the selected geometry around the
  selection center. Holding Shift snaps rotation to 22.5 degree increments.
- The transform rectangle follows the selection orientation instead of snapping
  back to viewport axes after arbitrary rotation, but a freshly selected shape
  starts with an axis-aligned rectangle until the user rotates it through the
  transform control.
- Edge and corner handles appear on the transform rectangle. Midpoint handles
  resize one axis, corner handles resize both axes, and corner-adjacent rotate
  zones show a rotate cursor.
- Dragging inside the dotted selection bounds free-moves the selected geometry.
  If CP snapping is enabled, translated endpoints can snap to the grid,
  unselected vertices, and unselected lines so the moved shape falls neatly into
  place without snapping back to its own source geometry.
- Move snapping also treats the selection-bounds corners as anchors and treats
  paper corners as point-like snap targets, so a transform-box corner can land
  exactly on a paper corner.
- During rotate or move drag, show a lightweight ghost preview and status
  readout. Commit on pointer release; Escape cancels the active transform before
  commit.
- The Crease Pattern menu gets a `Transform Selection` submenu with the same
  actions for desktop parity and discoverability.
- The interaction model is: select something, then manipulate the visible
  transform box. Pasted geometry simply becomes the active selection, so it gets
  the same transform affordance as any other selected CP contents.

## Approach

- Extend the workspace clipboard from tree-only payloads to a discriminated
  clipboard union, for example `{ kind: 'tree', ... }` and `{ kind:
  'cp-lines', lines, anchor, bounds, pasteCount }`.
- Copy CP lines by value from
  `oristudioCpDocument.document.crease_pattern.line_segments`, preserving
  endpoints, color, active state, selection state, customized color metadata,
  and source bounds. Do not store only line IDs because document edits can make
  IDs stale before paste.
- Add helpers in `apps/web/src/lib/creasePatternClipboard.ts` for:
  - extracting selected line payloads
  - computing selection bounds and center
  - applying translation, rotation, and mirror transforms
  - generating the default paste offset
- Add a selection-transform overlay helper in the CP viewport layer for:
  - deriving an SVG-space oriented dotted rectangle from selected model-space
    geometry
  - placing the compact contextual menu near the rectangle without covering the
    selected geometry
  - placing resize and rotate handles that remain usable under zoom
  - converting pointer movement around the selection center into a rotation
    angle
  - snapping the angle to 22.5 degree increments while Shift is held
  - dragging the selection bounds as a move handle, with translated endpoints
    evaluated against the existing CP snap targets
  - resizing the selected geometry from the frame corners and midpoints
- Add a kernel-backed insertion command instead of faking paste through
  `CreaseCopy`. The existing `CreaseCopy` operation transforms current document
  line IDs, but clipboard paste needs to insert copied geometry even after the
  source selection or source document has changed.
- Extend the wasm bridge and worker API with dedicated `insertLineSegments` and
  `replaceLineSegments` calls. These stay outside the Oriedita command registry
  because they operate on app-owned clipboard geometry rather than an upstream
  mouse handler operation.
- In the Rust CP kernel, insert provided line segments directly into the
  editable fold-line set and mark inserted pieces selected so the UI can sync
  the newly pasted or transformed selection from line `selected` flags.
- Add menu action IDs for CP selection transforms, but keep `edit.copy` and
  `edit.paste` as the keyboard entry points. In CP mode, those generic commands
  should branch to CP clipboard actions.
- Keep one user operation equal to one history checkpoint:
  - copy: no history
  - paste: one history entry
  - contextual flip or fixed rotation: one history entry
  - free rotate drag: no history while previewing, one history entry on pointer
    release
  - free move drag: no history while previewing, one history entry on pointer
    release
  - canceled transform previews: no history
- Preserve line colors during mirror. Mirroring geometry should not silently
  swap mountain and valley assignments.

## Affected Areas

- `apps/web/src/store/workspaceStore/types.ts`
- `apps/web/src/store/workspaceStore/slices/clipboardSlice.ts`
- `apps/web/src/store/workspaceStore/slices/projectSlice.ts`
- `apps/web/src/store/workspaceStore/capabilities.ts`
- `apps/web/src/store/workspaceStore/useWorkspaceCapabilities.ts`
- `apps/web/src/lib/workspaceCapabilities.ts`
- `apps/web/src/lib/appKeyboard.ts`
- `apps/web/src/commands/menuActions.ts`
- `apps/web/src/menus/menuDefinition.ts`
- `apps/tauri/src-tauri/src/menu.rs`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/lib/creasePatternClipboard.ts`
- `apps/web/src/engine/oristudioCpTypes.ts`
- `apps/web/src/workers/oristudioCpWorker.ts`
- `crates/oristudio-cp/src/lib.rs`
- `crates/oristudio-cp/src/operations/transform.rs`
- `crates/oristudio-cp-wasm/src/lib.rs`
- Related web, Rust, wasm, and oracle tests.

## Checklist

- [x] Define the CP line clipboard payload and transform helpers in TypeScript.
- [x] Update workspace clipboard state to support both tree and CP payloads
      without breaking existing tree copy/paste.
- [x] Enable `edit.copy` for editable CP documents with selected lines.
- [x] Enable `edit.paste` for editable CP documents when the clipboard contains
      copied CP lines.
- [x] Route `Cmd/Ctrl+C` and `Cmd/Ctrl+V` through the existing `edit.copy` and
      `edit.paste` handlers in CP mode.
- [x] Add the Rust kernel insert-lines command and wasm payload typing.
- [x] Paste copied CP lines with a default offset and select the newly inserted
      lines.
- [x] Add selected CP transform commands for rotate left, rotate right, mirror
      horizontal, mirror vertical, and arbitrary angle.
- [x] Add web and Tauri menu entries for the transform actions.
- [x] Render a dotted transform bounds rectangle for editable CP selections.
- [x] Rotate the transform rectangle with the selected geometry.
- [x] Add a compact contextual transform menu that appears whenever the CP
      selection has transformable contents.
- [x] Add a corner rotate handle with free rotation by default and Shift-held
      22.5 degree snapping.
- [x] Match the rotate affordance to the chosen visual without an external icon.
- [x] Support free-moving the selected CP line geometry by dragging inside the
      dotted transform bounds.
- [x] Make move dragging respect enabled CP snapping while excluding selected
      source lines from snap candidates.
- [x] Snap move-drag selection bounds corners to paper corners.
- [x] Add a ghost preview and angle readout while rotating, with Escape cancel
      before commit.
- [x] Add a ghost preview and move readout while translating, with Escape cancel
      before commit.
- [x] Add resize handles on the transform rectangle corners and midpoints.
- [x] Add a rotate cursor for corner-adjacent rotate zones.
- [x] Make undo/redo restore CP document state and pasted-line selection.
- [x] Add unit tests for capabilities, menu routing, clipboard payload
      construction, selection bounds, rotation snapping, and transform math.
- [x] Add Rust tests for inserting line segments and preserving color/custom
      metadata.
- [x] Add wasm tests for executing the insert-lines command through the worker
      payload shape.
- [x] Run focused validation: `npm run test:web`, `cargo test -p
      oristudio-cp`, and `wasm-pack test --node crates/oristudio-cp-wasm`.
- [x] Run follow-up validation for the transform-handle and move-drag UX:
      web lint, web typecheck, focused clipboard/store tests, and browser smoke.
- [x] Run follow-up validation for the reduced transform menu: web lint, web
      typecheck, focused menu/capability tests, desktop check, and browser
      smoke.
- [x] Run follow-up validation for oriented transform frames and resize handles:
      web lint, web typecheck, focused geometry/store/panel tests, and browser
      smoke.
- [x] Run follow-up validation for default axis-aligned selection frames and
      rotate-only frame orientation: web lint, web typecheck, focused
      geometry/panel tests, and browser smoke.
- [x] Allow modifier-click line selection through the transform move overlay
      and validate with focused panel coverage plus browser smoke.
- [x] Hide transform-box controls outside the default selection mode so
      selected-line tools can capture their own source/destination input.
- [x] Surface Oriedita's selected-crease reflection command as "Reflect
      selection over line" in the Transform tools.
- [x] Make Reflect selection over line accept either two snapped points or a
      clicked existing line for the reflection axis.
- [x] Move Lengthen by Same Color into the Draw tools and highlight the pending
      source line while lengthen tools wait for the target line.
