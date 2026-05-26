# Ori Studio Native File Format

## Goal

Add a native `.osf` JSON project format that preserves Ori Studio workspace
state across tree and editable crease-pattern workflows, while keeping
TreeMaker, CP, FOLD, SVG, and PNG as explicit import/export formats.

## Approach

- Add a typed project-file serializer, parser, and v1 migration harness in the
  shared web app.
- Make Save and Save As write `.osf` for both tree and editable CP documents.
- Keep `.tmd*`, `.cp`, and `.fold` openable through existing loaders, but route
  future persistence through native project files.
- Restore native tree documents through the TreeMaker engine and native CP
  documents through the Oriedita CP runtime, then rebuild derived artifacts.
- Update recents/autosave and file UI copy so native save semantics are clear.

## Affected Areas

- `apps/web/src/lib`
- `apps/web/src/store/workspaceStore`
- `apps/web/src/components/panels/FilesPanel.tsx`
- `apps/web/src/lib/workspaceCapabilities.ts`
- `apps/web/src/platform/fileService.ts`
- `apps/tauri/src-tauri`

## Checklist

- [x] Add implementation plan.
- [x] Add native project-file types, serializer, parser, and tests.
- [x] Wire open/save/autosave/recents to `.osf`.
- [x] Update UI copy and file-service extension handling.
- [x] Register `.osf` with the macOS bundle and route Finder-opened files through native open.
- [x] Assign the `.osf` document type an icon based on the macOS app icon.
- [x] Run focused validation.
