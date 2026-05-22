# High Accuracy Step Simulation

## Goal

Add an optional high-accuracy mode for sequence step simulation that uses smaller solver increments and more settling work, improving step-local previews without changing the default whole-model simulator.

## Approach

Keep whole-model simulation behavior unchanged. Add a scale-aware solver timestep multiplier to `@treemaker/origami-simulator`, then route the web Simulator pane through explicit run presets. Step mode defaults to `Accurate`, with a compact `Fast` / `Accurate` control for quick comparison. Accurate mode uses the same fold profile but advances the physics solver with a smaller adaptive timestep, smaller percent jumps, slower playback, and longer settle loops.

## Affected Areas

- `packages/origami-simulator` solver options, dynamic solver, and tests.
- `apps/web/src/lib` simulator run preset helper and tests.
- `apps/web/src/components/panels/SimulatorPanel.tsx` timing, controls, and tests.
- `apps/web/src/styles/theme.css` compact accuracy control styling.

## Checklist

- [x] Add scale-aware simulator timestep multiplier.
- [x] Add focused simulator package coverage for the smaller timestep.
- [x] Add web run presets for whole, fast step, and accurate step modes.
- [x] Wire the Simulator pane to default step previews to Accurate mode.
- [x] Add/update focused web tests for the mode control and presets.
- [x] Run targeted validation.
- [x] Commit the completed implementation.
