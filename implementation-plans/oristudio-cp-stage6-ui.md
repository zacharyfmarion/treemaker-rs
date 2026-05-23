# OriStudio CP Stage 6 UI

## Goal

Enable the Oriedita drawing and geometric construction workflows in the crease
pattern pane without hiding unimplemented behavior behind approximate tools.

## Approach

- Wire oracle-tested construction kernels through `oristudio-cp` command
  dispatch.
- Add a typed preview query that returns transient candidate segments and
  candidate points separately from committed geometry.
- Extend the WASM and web worker bridge so the React canvas can ask the kernel
  for construction previews while the tool is active.
- Keep tools that require parameters or candidate choices explicit in the
  command payload instead of baking in silent UI-only assumptions.
- Keep the UI implementation stage-scoped: individual toolbar/inspector
  parameter controls can expand after this stage, but Stage 6 commands must
  have honest tool prompts and visible preview overlays.

## Affected Areas

- `crates/oristudio-cp`
- `crates/oristudio-cp-wasm`
- `apps/web/src/engine`
- `apps/web/src/workers`
- `apps/web/src/store/workspaceStore`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`

## Checklist

- [x] Add construction payload parameters and preview result types.
- [x] Dispatch Stage 6 final mutations through oracle-tested Rust kernels.
- [x] Expose preview queries through WASM, worker runtime, and Zustand actions.
- [x] Enable Stage 6 command registry entries with Oriedita-style prompts.
- [x] Render live construction and candidate overlays in the CP canvas.
- [x] Add Rust unit tests for dispatch and preview query behavior.
- [x] Add web unit tests for registry readiness, payloads, and overlay behavior.
- [x] Run targeted Rust and web validation.
- [x] Update the UI roadmap Stage 6 checklist.
- [x] Commit the completed stage.
