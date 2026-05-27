# Design Symmetry Toolbar UX

## Goal

Align Design pane symmetry controls with the compact crease-pattern symmetry
toolbar UX, and remove symmetry setup from the Conditions panel so that panel
stays focused on constraints.

## Approach

- Replace the Design toolbar's wide `Mirror Nodes` button with a compact
  symmetry menu chip in the bottom viewport toolbar.
- Move tree symmetry enable, preset, flip, axis visibility, mirror-authoring,
  and numeric axis controls into that menu.
- Keep the existing layer menu as a general visibility menu, with the new
  symmetry menu also controlling the symmetry-axis layer.
- Remove the old symmetry setup section from Conditions while keeping condition
  creation actions such as `Node on symmetry` and `Pair nodes`.
- Update focused Design panel tests for the new toolbar entry point.

## Affected Areas

- `apps/web/src/components/panels/DesignPanel.tsx`
- `apps/web/src/components/panels/ConditionsPanel.tsx`
- `apps/web/src/components/panels/DesignPanel.test.tsx`
- `apps/web/src/styles/theme.css`

## Checklist

- [x] Move design symmetry setup into a compact toolbar popover.
- [x] Remove symmetry setup controls from Conditions.
- [x] Update tests for the new Design toolbar UX.
- [x] Run focused web validation and a browser smoke check.
