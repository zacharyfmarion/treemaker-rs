# CP Infinite Canvas

## Goal

Make imported/editable crease-pattern documents feel like an infinite canvas instead of a tight paper square with only a small border around it.

## Approach

- Keep generated TreeMaker crease-pattern rendering on the existing compact paper viewport.
- Add a larger editable CP canvas rectangle centered on the existing paper coordinate system so imported CPs start with visible workspace around the square and can be panned into empty space.
- Use fixed Oriedita paper bounds for editable/imported CP rendering so geometry outside `-200..200` draws outside the paper instead of resizing or squashing the square.
- Route editable CP pointer conversion, grid rendering, and diagnostic focusing through the active editable canvas rectangle while preserving fixed model-to-paper coordinates.
- Render the visible editor grid in full-canvas mode and cap dense grid line generation so grid rendering stays bounded.
- Treat the editable CP paper square as a transparent coordinate guide so blank or open CP documents have a uniform canvas background.
- Start newly created CP documents with explicit square border creases and a more readable default grid interval.
- Cover the viewport helper behavior and panel rendering with focused web tests.

## Affected Areas

- `apps/web/src/lib/creasePatternViewport.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/styles/theme.css`
- `apps/web/src/lib/oristudioCpStarterDocument.ts`
- `apps/web/src/store/workspaceStore/oristudioCpRuntime.ts`
- `apps/web/src/lib/creasePatternViewport.test.ts`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `apps/web/src/lib/oristudioCpStarterDocument.test.ts`
- `apps/web/src/store/workspaceStore/store.test.ts`

## Checklist

- [x] Add editable CP canvas constants and grid helper support.
- [x] Render imported/editable CPs on the expanded canvas.
- [x] Make blank editable CP canvases visually uniform.
- [x] Add starter square border and readable default grid interval for newly created CP documents.
- [x] Update pointer and diagnostic mapping for the expanded canvas.
- [x] Add or update tests.
- [x] Run web validation.
- [x] Open a draft PR and start a local test server.
