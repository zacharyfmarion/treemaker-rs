# OriStudio CP Stage 7 Action UX

## Goal

Complete the Stage 7 UI shift from operation-centric crease-pattern editing to
Oriedita-style visible actions and tool options.

The visible CP toolbar should expose actions such as line type, draw crease,
restricted draw, selection, construction, and repair. The Rust operation IDs
remain the dispatch and oracle-validation layer, but they should no longer be
the primary mental model for users.

## Approach

- Add a web-side action registry over the existing CP command registry.
- Add M/V/E/A line type state as a tool option, not as duplicate draw tools.
- Route draw/construction payloads through the current line type instead of a
  hardcoded red default.
- Convert free and restricted draw crease to Oriedita-style click-drag-release.
- Use an Oriedita-style draw endpoint resolver that snaps to line endpoints,
  point/circle centers, and visible grid points, without snapping draw endpoints
  to line interiors.
- Keep simple draw preview synchronous from pointer state so the visible guide
  and committed endpoints use the same resolved points.
- Keep unsupported action variants visible but explicitly disabled.

## Affected Areas

- `apps/web/src/lib/oristudioCpActions.ts`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/lib/oristudioCpToolState.ts`
- `apps/web/src/lib/creasePatternViewport.ts`
- `apps/web/src/components/panels/CpToolRail.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/styles/theme.css`
- `implementation-plans/oristudio-cp-ui-roadmap.md`
- `implementation-plans/oristudio-cp-drawing-tool-parity.md`

## Checklist

- [x] Add action registry and action coverage tests.
- [x] Rebuild the left rail around visible actions and line type options.
- [x] Add active M/V/E/A line type state and route payload defaults through it.
- [x] Add Oriedita-style draw endpoint snapping.
- [x] Convert free/restricted draw crease to click-drag-release.
- [x] Add/update unit tests for action state, snap parity, and pointer behavior.
- [x] Run web validation.
- [x] Commit Stage 7.
