# Simulator Lighting And Shadows

## Goal

Make the Simulator pane's folded 3D shape easier to read by adding default-on directional lighting, subtle projected shadows, and a compact header toggle for the effect.

## Approach

- Extend simulator view settings with a lighting toggle that defaults on.
- Add a header icon control alongside existing simulator view controls.
- Shade paper-face colors in the canvas rasterizer using a fixed upper-left/front light vector.
- Draw a soft projected model shadow before rasterized paper faces when lighting is enabled.
- Update focused Simulator panel tests for the new header control and rendering hooks.

## Affected Areas

- `apps/web/src/components/panels/SimulatorPanel.tsx`
- `apps/web/src/components/panels/SimulatorPanel.test.tsx`
- `implementation-plans/simulator-lighting-shadows.md`

## Checklist

- [x] Add implementation plan.
- [x] Add lighting view setting and header toggle.
- [x] Shade simulator paper faces and draw subtle shadows.
- [x] Update focused frontend tests.
- [x] Run focused web validation.
- [x] Prepare draft PR handoff.
