# Ori Studio CP Stage 5 Completion

## Goal

Complete Stage 5 of the Oriedita CP UI roadmap by routing the already ported
core edit operations through the web crease-pattern pane, making those edits
participate in undo/redo, and preserving the Oriedita-backed Rust crate as the
source of behavior.

## Approach

- Extend the `oristudio-cp` command payload and dispatcher for the remaining
  Stage 5 non-UI operations instead of duplicating algorithms in TypeScript.
- Keep the web pane responsible for viewport concerns only: resolved model
  points, selected line IDs, drag paths, active line color defaults, and
  selection-distance conversion.
- Store editable CP history as full crate document snapshots plus web
  selection state, then restore through the wasm `load_document` path for
  undo/redo.
- Mark Stage 5 commands ready only when their mutation path is backed by
  existing Rust unit/oracle coverage or new focused dispatch tests.
- Leave Stage 6 drawing/construction previews and Stage 7/8/9 specialty tools
  explicitly out of this stage.

## Affected Areas

- `crates/oristudio-cp`: command payload, Stage 5 command dispatch, operation
  frame serialization, focused command-dispatch tests.
- `crates/oristudio-cp-wasm`: editable document load/snapshot compatibility.
- `apps/web/src/store/workspaceStore`: editable CP history, undo/redo,
  capability gating, command selection sync.
- `apps/web/src/components/panels`: Stage 5 tool activation, point-sequence
  tools, freehand drag-path tools, previews, operation-frame rendering.
- `apps/web/src/lib/oristudioCpCommands.ts`: command readiness and input-mode
  metadata.

## Checklist

- [x] Add remaining Stage 5 command dispatch routes to `oristudio-cp`.
- [x] Preserve Oriedita command defaults for active line color and custom line
      type filters.
- [x] Serialize operation-frame state through Rust and wasm snapshots.
- [x] Wire editable CP undo/redo through snapshot restore, including selection
      state.
- [x] Enable Stage 5 tool-rail commands in the CP pane.
- [x] Add point-sequence and freehand drag-path UI command flows.
- [x] Render command previews and persisted operation-frame geometry.
- [x] Add focused Rust and web tests for the new command and history paths.
- [x] Validate with focused Rust, web test, typecheck, and lint commands.
