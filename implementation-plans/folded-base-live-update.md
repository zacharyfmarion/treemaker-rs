# Folded Base Live Update

## Goal

Keep the Folded Base pane current while editing an imported crease pattern, surface folding solve errors in the pane, and simplify the default folded-base render so it looks like folded paper instead of a diagnostic overlay.

## Approach

- Regenerate folded artifacts from the editable CP document's current FOLD export after mutating CP commands.
- Let the Folded Base pane request artifacts automatically when opened with stale or missing folded-base data.
- Remove the manual refresh affordance and status text from the Folded Base toolbar.
- Default the SVG render to visible folded paper only, with compact toolbar icon toggles for wireframe and translucent layer inspection.
- Update focused frontend tests around CP artifact refresh and Folded Base rendering controls.

## Affected Areas

- `apps/web/src/store/workspaceStore/slices/creasePatternSlice.ts`
- `apps/web/src/store/workspaceStore/slices/projectSlice.ts`
- `apps/web/src/components/panels/FoldedBasePanel.tsx`
- `apps/web/src/components/panels/FoldedBasePanel.test.tsx`
- `apps/web/src/store/workspaceStore/store.test.ts`
- `apps/web/src/styles/theme.css`

## Checklist

- [x] Add implementation plan.
- [x] Auto-refresh folded artifacts after editable CP mutations.
- [x] Simplify Folded Base rendering and view controls.
- [x] Update focused tests.
- [x] Run focused web validation.
- [x] Prepare draft PR handoff.
