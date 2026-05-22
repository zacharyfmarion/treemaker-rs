# Step-Scoped Sequence Simulation

## Goal

Let a user simulate one folding-sequence step at a time from the Sequence pane while keeping the existing whole-model simulator as the default. Manual collapse steps are allowed only as approximate previews and must be labeled as such.

## Approach

Add per-crease fold profiles to `@treemaker/origami-simulator`, then adapt sequence state snapshots into simulator FOLD documents plus fold ranges. Store the active simulator scope in the workspace store, expose a per-step action in the Sequence pane, and make the Simulator pane render either whole-model mode or the selected step mode with highlighted affected creases and faces.

## Affected Areas

- `packages/origami-simulator` types, dynamic solver, controller, and tests.
- `apps/web/src/lib` sequence-to-simulator adapter and tests.
- `apps/web/src/store/workspaceStore` sequence simulation focus state and invalidation.
- `apps/web/src/components/panels/SequencePanel.tsx` and `SimulatorPanel.tsx`.
- `apps/web/src/styles/theme.css` for compact step simulation controls and badges.

## Checklist

- [x] Add per-crease fold profile API and solver support.
- [x] Add simulator package tests for profile behavior and whole-mode fallback.
- [x] Add sequence simulation focus state and stale-focus invalidation.
- [x] Add sequence-to-simulator adapter with normal-step and manual-collapse behavior.
- [x] Add Sequence pane simulate-step action.
- [x] Add Simulator pane Whole/Step mode, step errors, warnings, and highlights.
- [x] Add focused web tests for adapter, store, Sequence pane, and Simulator pane.
- [x] Run focused simulator and web validations.
- [x] Commit the completed implementation.
