# OriStudio CP Stage 8 Shapes And Generators

## Goal

Enable the non-stateful Stage 8 CP shape and generator commands whose Rust kernels are already oracle-tested: simple circle creation, regular polygons, and default base molecules.

## Approach

- Add command-dispatch arms for simple circle creation and base generator operations.
- Add polygon corner payload plumbing from the contextual panel to the Rust command payload.
- Mark only the point-sequence operations ready in the web command registry.
- Add circle preview geometry to the command preview contract so circle tools do not preview as ordinary crease lines.
- Leave stateful tools such as Voronoi, text editing, selected-circle concentric modes, tangent/inversion modes, and circle color editing visible but not implemented until their UI state models are designed.

## Affected Areas

- `crates/oristudio-cp/src/lib.rs`
- `crates/oristudio-cp-wasm/src/lib.rs`
- `apps/web/src/engine/oristudioCpTypes.ts`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Add Rust command dispatch for simple circle creation.
- [x] Add Rust command dispatch for regular polygon and default base molecules.
- [x] Add preview-circle support through the wasm/web preview contract.
- [x] Wire polygon corner payloads and mark ready web commands.
- [x] Add Rust and web tests for the enabled commands.
- [x] Regenerate the OriStudio CP wasm package.
- [x] Run focused validation.
- [x] Commit the completed Stage 8 shapes/generators slice.
