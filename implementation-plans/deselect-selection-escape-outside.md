# Deselect Selection With Escape And Background Click

## Goal

Make selections created by Select All behave like a normal editor selection:
Escape clears them, and clicking outside selected parts clears them instead of
leaving everything selected.

## Approach

- Add Escape handling to the shared app keyboard layer so it dispatches the
  existing deselect behavior outside text inputs and modal handlers.
- Update Design pane background clicks to clear active multi-selection rather
  than adding a node when the click is outside selected parts.
- Update Crease Pattern pane background clicks to clear crease/facet selection.
- Cover the behavior with focused web unit tests.

## Affected Areas

- `apps/web/src/App.tsx`
- `apps/web/src/components/panels/DesignPanel.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- Web unit tests for app shortcuts and panel background selection behavior

## Checklist

- [x] Inspect existing shortcut, selection, and pointer handling.
- [x] Implement Escape and background-click deselection.
- [x] Add focused web tests for the new behavior.
- [x] Run targeted web validation.
- [x] Prepare branch handoff and draft PR.
