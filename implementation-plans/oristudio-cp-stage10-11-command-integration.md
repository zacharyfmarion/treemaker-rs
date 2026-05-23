# Oristudio CP Stage 10-11 Command Integration

## Goal

Finish the UI-roadmap work through Stage 11 without replacing the existing
Flat-Folder-derived folded-base and simulator preview path.

## Approach

- Rewrite Stage 10 roadmap language so Oriedita folding work means
  folded-figure session parity, not a duplicate folded-base renderer.
- Keep the existing Folded Base and Simulator panes as the default folded
  preview workflow.
- Add shared CP command IDs for the Oriedita operations that make sense from
  menus and shortcuts: diagnostics, selected-line repair, selected-line delete,
  and folded-base preview.
- Route web menu, Tauri menu, and keyboard shortcuts through the same command
  dispatcher used elsewhere in the app.
- Add inspector actions for editable CP selections and active diagnostics.

## Affected Areas

- `implementation-plans/oristudio-cp-ui-roadmap.md`
- `apps/web/src/commands/menuActions.ts`
- `apps/web/src/menus/menuDefinition.ts`
- `apps/tauri/src-tauri/src/menu.rs`
- `apps/web/src/lib/workspaceCapabilities.ts`
- `apps/web/src/lib/appKeyboard.ts`
- `apps/web/src/components/panels/InspectorPanel.tsx`
- Related unit tests.

## Checklist

- [x] Update Stage 10/11 roadmap language.
- [x] Add shared CP menu command IDs and capabilities.
- [x] Add web and native menu entries for CP diagnostics, repair, and folded
      preview.
- [x] Add keyboard routing for file/build/optimization and CP-specific delete
      and selection clearing.
- [x] Add CP inspector actions for selected editable CP entities and active
      diagnostics.
- [x] Run non-browser validation and commit.
