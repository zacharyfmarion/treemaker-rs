# Oristudio CP Grid Parity

## Goal

Make the editable crease-pattern grid in the web CP pane match Oriedita's grid
model before adding more CP editing tools.

The current Stage 2 grid divides the rendered CP geometry bounds. Oriedita
instead draws a grid in object/paper coordinates using the saved grid model:
paper origin `(-200, 200)`, width `400 / gridSize`, optional non-square grid
vectors, grid angle, base state, interval offsets, and optional diagonal lines.
The web view and snap target logic should use that same basis.

## Approach

- Port the Oriedita grid basis math from `GridModel` and `Grid` into the web
  viewport helper.
- Use Oriedita paper bounds for editable CP documents with an active grid so
  default Oriedita/FOLD files align with the paper border.
- Generate regular, interval, and diagonal grid lines from the Oriedita index
  ranges instead of dividing the geometry bounding box.
- Use the same Oriedita grid basis for nearest grid snapping.
- Keep the existing generated TreeMaker CP renderer unchanged.

## Affected Areas

- `apps/web/src/lib/creasePatternViewport.ts`
- `apps/web/src/lib/creasePatternViewport.test.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Add Oriedita grid basis, bounds, line, and snap helpers.
- [x] Wire the editable CP pane to render Oriedita grid lines.
- [x] Add unit tests for default, hidden, angled, interval, diagonal, and snap behavior.
- [x] Update the roadmap to record grid parity as part of Stage 2.
- [x] Run focused web validation and restart the local web app.
