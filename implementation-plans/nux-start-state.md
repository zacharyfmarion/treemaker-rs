# NUX Start State
## Goal
Add a first-run start state before the main Ori Studio workspace that lets users create an editable crease pattern, open a supported file, or start a tree-based design.
## Approach
Introduce a focused start-screen component that uses existing theme tokens and shared workspace actions. Add a blank editable CP creation action to the shared workspace store so the CP option lands in the same pane-based editor as imported CP files. Keep file opening on the existing platform file-service path so browser and desktop behavior stay aligned.
## Affected Areas
- `apps/web/src/App.tsx` and app styling for the pre-workspace state.
- `apps/web/src/components` for the NUX start screen.
- `apps/web/src/store/workspaceStore` for creating a blank editable CP document.
- Frontend tests for store behavior and start-screen actions.
## Checklist
- [x] Inspect existing app shell, store, and CP/file workflows.
- [x] Add blank editable CP workspace action.
- [x] Add the NUX start screen and responsive styling.
- [x] Add or update focused tests.
- [x] Run targeted web validation.
- [x] Open a draft PR and start a local web server.
