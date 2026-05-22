# Oriedita CP Stage 8 Circle Modes

## Goal

Expose the remaining Oriedita circle editing modes in the crease pattern pane
without weakening the already-oracle-tested Rust geometry behavior.

## Approach

- Keep the kernel API operation-based and typed: circle IDs are one-based UI
  selections, line IDs are one-based selected segments, points are resolved
  model-space inputs, and candidate indices are zero-based Oriedita indicators.
- Use selection plus the bottom-right contextual panel for circle modes whose
  Oriedita Swing handlers depend on selected circles or indicator choice.
- Preserve point-driven behavior where Oriedita accepts it: tangent-line can
  use one selected circle plus a clicked point, while two-circle tangent uses
  selected circles and contextual apply.
- Add previews for selected-circle candidates so users can see tangent,
  inversion, and concentric indicator geometry before applying.

## Affected Areas

- `crates/oristudio-cp/src/lib.rs`
- `apps/web/src/engine/oristudioCpTypes.ts`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/lib/oristudioCpToolSettings.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Add Rust command dispatch and preview support for tangent, inverted, and
      concentric selected-circle modes.
- [x] Mark remaining circle commands ready in the web registry with contextual
      setting groups and candidate controls.
- [x] Wire web selection/click payloads for selected-circle apply flows and
      one-circle-plus-point tangent creation.
- [x] Add focused Rust, wasm, and web unit tests for the new payload paths.
- [x] Regenerate wasm bindings and run non-browser validation.
- [x] Commit the completed circle-mode slice.
