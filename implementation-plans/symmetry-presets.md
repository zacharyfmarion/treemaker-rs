# Symmetry Controls And Design Grid

## Goal

Make the Conditions panel symmetry controls easier to use by exposing common symmetry modes while keeping raw angle and origin controls available for advanced edits. Remove the unused design-view grid overlay so the drawing surface is quieter.

## Approach

- Add a Radix-based select primitive that matches the existing control styling.
- Replace the always-visible symmetry checkbox and angle/X/Y controls with a mode row.
- Use `None`, `Book`, `Diagonal`, and `Custom` dropdown options so line visibility and manual edits are explicit.
- Map Book and Diagonal preset choices onto the existing `hasSymmetry`, `symAngle`, and `symLoc` fields.
- Add a compact flip button that cycles the orientation within the selected preset type.
- Put raw angle and X/Y inputs behind an "Advanced symmetry options" disclosure and mark the dropdown as Custom after manual edits.
- Remove the design paper grid layer, menu item, and SVG overlay.

## Affected Areas

- `apps/web/src/components/ui`
- `apps/web/src/components/panels/ConditionsPanel.tsx`
- `apps/web/src/components/panels/DesignPanel.tsx`
- `apps/web/src/lib`
- `apps/web/src/styles/theme.css`
- `apps/web/package.json`

## Checklist

- [x] Add Radix Select dependency and shared select component.
- [x] Add symmetry preset dropdown, flip button, and advanced disclosure.
- [x] Add None and Custom modes for symmetry visibility and manual edits.
- [x] Remove the design grid overlay and layer toggle.
- [x] Style the new controls for the inspector panel.
- [x] Run focused web validation and check the local UI.
