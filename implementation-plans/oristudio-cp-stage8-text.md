# Oriedita CP Stage 8 Text Annotations

## Goal

Enable text annotation creation, selection, editing, movement, deletion, and
serialization in the crease pattern pane while preserving Oriedita's text data
model and undo behavior.

## Approach

- Add a typed `Text` command payload surface for create, move, set-content, and
  selected-delete operations.
- Reuse the existing ported Oriedita text mouse-handler helpers for create and
  drag movement where applicable.
- Use the web contextual tool panel as the text editor surface, matching the
  app's current action-based tool direction instead of adding a floating Swing-
  style text area.
- Keep right-click/box-delete parity explicitly tracked for a later text
  refinement slice if it does not fit this commit.

## Affected Areas

- `crates/oristudio-cp/src/lib.rs`
- `apps/web/src/engine/oristudioCpTypes.ts`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/lib/oristudioCpToolSettings.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Add Rust `Text` command dispatch and tests.
- [x] Add web payload types, command registry, and text contextual setting.
- [x] Add canvas create/select/drag behavior for the text tool.
- [x] Add contextual content apply/delete behavior for selected text.
- [x] Add focused unit tests for text payloads and undo-aware command execution.
- [x] Regenerate wasm bindings and run non-browser validation.
- [x] Commit the completed text slice.
