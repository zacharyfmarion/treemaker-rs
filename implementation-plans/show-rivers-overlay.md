# Show Rivers Overlay

## Goal

Restore the reference TreeMaker design-view path overlay so optimized designs
show the active, border, polygon, infeasible, and conditioned paths that users
expect as river/path guidance alongside leaf circles.

## Approach

- Confirm what TreeMaker 5.0.1 actually renders for rivers and paths in the
  vendored C++ GUI and help docs.
- Keep the Rust engine unchanged because snapshots already expose the required
  path flags.
- Update the web snapshot mapper and view types so the Design pane keeps
  reference-visible paths instead of only leaf paths.
- Style non-leaf/internal paths distinctly while preserving existing active,
  feasible, infeasible, conditioned, and selected path behavior.
- Add focused web tests for the mapping behavior.

## Affected Areas

- `apps/web/src/engine/types.ts`
- `apps/web/src/engine/snapshotMapper.ts`
- `apps/web/src/lib/sampleProject.ts`
- `apps/web/src/components/panels/DesignPanel.tsx`
- `apps/web/src/components/panels/InspectorPanel.tsx`
- `apps/web/src/styles/theme.css`
- `apps/web/src/engine/snapshotMapper.test.ts`

## Checklist

- [x] Confirm upstream TreeMaker path/river behavior.
- [x] Add reference-visible path mapping and metadata.
- [x] Update Design pane rendering and inspector details.
- [x] Add focused web tests.
- [x] Run targeted web validation.
- [x] Prepare branch handoff and draft PR.
