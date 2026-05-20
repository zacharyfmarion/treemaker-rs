# Crease Pattern View Controls

## Goal

Give the Crease Pattern pane the same pan, zoom, fit, actual-size, and keyboard
view controls that make the Design pane comfortable to inspect.

## Approach

- Extract the shared viewport toolbar controls from the Design pane into a
  reusable panel component.
- Wrap the Crease Pattern SVG in the same `react-zoom-pan-pinch` viewport
  behavior used by the Design pane.
- Keep crease-pattern color mode and selection behavior unchanged.
- Add focused web component tests for CP viewport controls.

## Affected Areas

- `apps/web/src/components/panels/DesignPanel.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/ViewportToolbar.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `apps/web/src/styles/theme.css`

## Checklist

- [x] Add implementation plan.
- [x] Extract reusable viewport toolbar controls.
- [x] Port pan, zoom, fit, and keyboard controls to the CP pane.
- [x] Update styles and component tests.
- [x] Run focused web validation.
- [x] Prepare draft PR handoff.
