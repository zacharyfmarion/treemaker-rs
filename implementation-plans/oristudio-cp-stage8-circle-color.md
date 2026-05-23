# OriStudio CP Stage 8 Circle Color

## Goal

Enable Oriedita's circle color change command for selected circles and cyan auxiliary lines using the contextual RGB controls.

## Approach

- Add selected circle IDs and custom circle color values to the shared command payload.
- Dispatch `CircleChangeColor` through the oracle-tested Rust circle color operation.
- Include selected circles in the web apply payload while preserving existing selected-line payloads for cyan auxiliary line compatibility.
- Mark the web command ready and keep it contextual: users select circles or aux lines, adjust RGB values, then apply.

## Affected Areas

- `crates/oristudio-cp/src/lib.rs`
- `apps/web/src/engine/oristudioCpTypes.ts`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `apps/web/src/lib/oristudioCpCommands.test.ts`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Add payload support for selected circle IDs and custom RGB color.
- [x] Dispatch `CircleChangeColor` in Rust.
- [x] Pass selected circles and RGB values from the contextual panel.
- [x] Add Rust and web tests for circle color mutation payloads.
- [x] Run focused validation.
- [x] Commit the completed Stage 8 circle-color slice.
