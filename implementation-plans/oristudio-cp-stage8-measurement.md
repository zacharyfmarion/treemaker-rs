# OriStudio CP Stage 8 Measurement UI

## Goal

Enable Oriedita-style CP measurement tools in the web crease-pattern editor without mutating the document or adding undo/redo history entries.

## Approach

- Treat length and angle measurements as viewport-local tool results.
- Keep the measurement action active after a completed measurement so the contextual panel remains visible and the user can repeat the probe.
- Compute length and oriented angle with the same point ordering as the `oristudio-cp` Rust operations.
- Render measurement slots in the bottom-right contextual tool panel.
- Keep kernel command dispatch out of measurement pointer completion until the Rust command bridge grows a dedicated read-only result channel.

## Affected Areas

- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/styles/theme.css`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `apps/web/src/lib/oristudioCpCommands.test.ts`
- `apps/web/src/lib/oristudioCpToolState.test.ts`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Mark measurement commands ready in the CP action registry.
- [x] Add local measurement slot state and Oriedita-compatible length/angle computation.
- [x] Render measurement readouts in the contextual tool panel.
- [x] Prove measurements do not execute mutating commands or add undo history.
- [x] Run focused web tests and type validation.
- [x] Commit the completed Stage 8 measurement slice.
