# CP Tool Selection Polish

## Goal

Make crease-pattern editing feel stable and repeatable by defaulting to box
selection, deleting selected points from the keyboard, preserving viewport
position after edits, keeping tools active after successful actions, and
tightening the CP fit view.

## Approach

- Keep the existing Oriedita-style action registry and make repeatable tool
  behavior the default after successful commands.
- Route Delete through selected CP vertices and points as well as selected
  lines, using the existing `DeletePoint` operation for resolved selected
  coordinates.
- Only auto-fit editable CP documents when the document changes, not after
  every geometry mutation.
- Fit the editable CP paper rectangle itself with a small buffer instead of the
  oversized editable canvas region.
- Add focused web tests for keyboard deletion, repeatable tools, fit scale, and
  viewport preservation.

## Affected Areas

- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `apps/web/src/commands/menuActions.ts`
- `apps/web/src/commands/menuActions.test.ts`
- `apps/web/src/lib/creasePatternViewport.ts`
- `apps/web/src/lib/workspaceCapabilities.ts`
- `apps/web/src/store/workspaceStore/capabilities.ts`
- `apps/web/src/store/workspaceStore/useWorkspaceCapabilities.ts`

## Checklist

- [x] Add implementation plan.
- [x] Preserve active CP tools after successful actions.
- [x] Support Delete for selected CP points/vertices.
- [x] Prevent editable CP edits from auto-fitting the viewport.
- [x] Tighten editable CP fit-to-view.
- [x] Add/update focused frontend tests.
- [x] Run focused web validation.
- [x] Prepare draft PR handoff.
