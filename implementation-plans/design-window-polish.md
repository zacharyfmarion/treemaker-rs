# Design Window Polish

## Goal

Make the Design pane feel like a polished editor viewport by adding pan, zoom,
fit controls, practical layer toggles, and rendering bounds that keep large leaf
circles visible outside the paper.

## Approach

- Add the Cascade-style `react-zoom-pan-pinch` viewport dependency to the web app.
- Extract design viewport geometry helpers for dynamic SVG world bounds and
  pointer-to-paper conversion.
- Refactor `DesignPanel` so the SVG design world sits inside a transformable
  viewport with zoom toolbar controls, keyboard shortcuts, and UI-only layer
  visibility state.
- Keep document editing behavior unchanged: create/drag coordinates remain
  clamped to paper and layer/view state must not affect saved project data.

## Affected Areas

- `apps/web/package.json` and `package-lock.json`
- `apps/web/src/components/panels/DesignPanel.tsx`
- `apps/web/src/styles/theme.css`
- Web unit tests for design viewport geometry and layer-state behavior

## Checklist

- [x] Add implementation plan and viewport dependency.
- [x] Add tested dynamic world-bounds and pointer conversion helpers.
- [x] Refactor Design pane to use pan/zoom/fit and layer controls.
- [x] Validate with web lint, typecheck, and tests.
- [x] Prepare branch handoff and draft PR.
