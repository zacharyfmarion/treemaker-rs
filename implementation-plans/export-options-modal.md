# Export Options Modal
## Goal
Add SVG and PNG export options so users can choose the crease-pattern view mode, whether unassigned creases are included, and whether background facet colors are shown, with a live preview before saving.
## Approach
Route SVG and PNG exports through one shared options path in the workspace store, extend crease-pattern serialization/rendering with explicit export options, and add a focused modal hosted by the existing command dialog infrastructure.
## Affected Areas
- `apps/web/src/lib/creaseExport.ts`
- `apps/web/src/store/commandDialogStore.ts`
- `apps/web/src/components/CommandDialogModal.tsx`
- `apps/web/src/store/workspaceStore`
- `apps/web/src/styles/theme.css`
- Web unit tests for export serialization, store export behavior, and the modal
## Checklist
- [x] Add export option types and serializer/render support.
- [x] Add an export options modal with live preview.
- [x] Route SVG and PNG exports through the modal from shared command entry points.
- [x] Add a background color toggle that exports a white paper background when disabled.
- [x] Update focused frontend tests.
- [x] Run web validation.
