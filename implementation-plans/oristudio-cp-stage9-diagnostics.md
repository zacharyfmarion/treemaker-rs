# Oriedita CP Stage 9 Diagnostics

## Goal

Bring the Oriedita check/diagnostic commands into the crease pattern UI with
structured, renderable results instead of plain status strings.

## Approach

- Start with non-mutating checks: Check1, Check2, Check3, Check4, and CAMV
  should return oracle-tested diagnostic geometry and should not create undo
  history entries.
- Add mutating repair commands Fix1 and Fix2 through the same command dispatch
  path as other CP edits.
- Store the latest structured diagnostic entries on the editable CP command
  result, render their markers in the CP pane, and summarize them in the
  Diagnostics panel.
- Keep the flat-foldable boundary check as a follow-up slice because its
  Oriedita handler is a multi-point transient boundary workflow, not a simple
  selected-document check.

## Affected Areas

- `crates/oristudio-cp/src/lib.rs`
- `apps/web/src/engine/oristudioCpTypes.ts`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/store/workspaceStore/slices/projectSlice.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/DiagnosticsPanel.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `apps/web/src/components/panels/DiagnosticsPanel.test.tsx`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Add structured diagnostic result entries for Check1, Check2, Check3,
      Check4, and CAMV.
- [x] Add command dispatch for Fix1 and Fix2 repairs.
- [x] Prevent non-mutating diagnostic checks from creating CP undo history or
      dirtying the document.
- [x] Render latest diagnostic markers in the CP canvas and summarize them in
      the Diagnostics panel.
- [x] Mark Check1-4, CAMV, Fix1, and Fix2 ready in the web command registry.
- [x] Add focused Rust, wasm, and web tests for diagnostic command results,
      history neutrality, overlays, and repair dispatch.
- [x] Regenerate wasm bindings and run non-browser validation.
- [x] Prepare the completed diagnostics slice for commit.
