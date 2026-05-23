# Oriedita CP Stage 8 Voronoi

## Goal

Enable Oriedita's Voronoi creation workflow in the crease pattern pane without
approximating the upstream behavior.

## Approach

- Reuse the ported Rust `VoronoiState` and replay the UI's seed-press history
  through `voronoi_press` for both preview and final apply.
- Treat Voronoi as a stateful web tool: each canvas click records one seed
  press, the kernel preview displays the resulting Voronoi lines and resolved
  seed points, and the contextual panel applies or clears the pending diagram.
- Preserve Oriedita details: nearest existing CP point snapping, seed removal by
  clicking within selection distance, cyan seed circles, and active line color
  for committed Voronoi lines.

## Affected Areas

- `crates/oristudio-cp/src/lib.rs`
- `apps/web/src/engine/oristudioCpTypes.ts`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/lib/oristudioCpToolSettings.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Add Rust command dispatch and preview coverage for `VoronoiCreate`.
- [x] Enable the Voronoi tool in the web command registry and settings panel.
- [x] Add stateful canvas click handling, apply, and clear controls.
- [x] Add focused unit tests for payloads, preview, apply, and clear behavior.
- [x] Regenerate wasm bindings and run non-browser validation.
- [x] Commit the completed Voronoi slice.
