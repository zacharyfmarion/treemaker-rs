# Symmetry Authoring Mode

## Goal

Make symmetry a canvas authoring workflow: users can draw centerline structure
once, mirror side branches from the Design pane, and bulk-apply leaf symmetry
conditions without manually pairing every terminal node.

## Approach

- Add paper-space symmetry geometry helpers and pair-detection logic.
- Add workspace actions for mirrored branch creation, paired node dragging,
  symmetry preview, and applying pair/on-axis leaf conditions.
- Add Mirror and Pair Leaves controls to the Design canvas toolbar.
- Render axis snap feedback, mirrored ghost branches, and a compact preview
  overlay for pair detection.
- Keep engine edits and persistent conditions on the existing frontend/WASM
  contract.

## Affected Areas

- `apps/web/src/lib`
- `apps/web/src/store/workspaceStore`
- `apps/web/src/components/panels/DesignPanel.tsx`
- `apps/web/src/styles/theme.css`

## Checklist

- [x] Add symmetry helper functions and tests.
- [x] Add workspace actions for mirrored authoring and pair application.
- [x] Add Design pane toolbar controls, previews, and drag behavior.
- [x] Add focused web tests for helpers, store behavior, and UI surface.
- [x] Run web validation.
