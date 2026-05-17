# TreeMaker Web App Roadmap

This file tracks the GUI/web-app work for `treemaker-rs`. The Rust engine
roadmap remains in `ROADMAP.md`; this roadmap is for product, UI, browser,
WASM, and eventual Tauri desktop work.

## Current Decision

Build the first web app in this repository.

Why:

- The GUI needs new `treemaker-core` and `treemaker-wasm` editing/snapshot
  APIs, so keeping app and engine together avoids version skew.
- The repository is already the canonical home of the Rust/WASM port, which
  makes development and parity testing easier.
- Discoverability can be handled through README links, hosted demos, and later
  package/release artifacts without splitting the source too early.

Revisit a split only after the UI can release independently from engine
changes.

## Product Direction

The first usable workflow is:

1. Draw or import a tree.
2. Edit node and edge properties.
3. Optimize the tree, starting with scale optimization.
4. Generate the crease pattern.
5. Inspect, save, and export the result.

The old wxWidgets TreeMaker UI is reference material for behavior, not a UX
template. The web app should feel like a modern pane-based design tool using
the visual language and layout patterns from Cascade and OpenSCAD Studio:
Dockview panes, compact toolbars, quiet inspector panels, icon buttons,
segmented controls, Radix-style primitives, Zustand stores, and CSS token
themes.

## Phase Status

### Phase 1: Roadmap And Web Shell

Status: complete for the initial browser shell.

Goal: create the project structure for a browser-first GUI that can build in
CI and leaves room for Tauri.

Work items:

- Add this roadmap and link it from the repository README.
- Add a Vite/React/TypeScript app under `apps/web`.
- Add root package scripts for web development and validation.
- Add Dockview layout shell with panes for Design, Inspector, Crease Pattern,
  Diagnostics, and Files/Examples.
- Add shared UI primitives and theme tokens adapted from Cascade/OpenSCAD
  Studio.
- Keep the first shell functional without requiring engine editing APIs yet.

Done when:

- `npm run build:web` succeeds.
- `npm run test:web` succeeds once tests exist.
- Existing Rust checks still pass.

### Phase 2: Editable Engine Contract

Status: complete for the initial core and wasm editing contract.

Goal: expose a safe, UI-friendly editing API instead of making the browser
construct TreeMaker internals by hand.

Work items:

- Add design-level public types for nodes, edges, conditions, and snapshots.
- Add `Tree::new_design`, `Tree::from_design`, `Tree::to_design`,
  `Tree::snapshot`, and edit operations for add/move/delete node, add/delete
  edge, update edge properties, update paper settings, and update labels.
- Rebuild all tree paths and derived state after topology edits.
- Invalidate stale polygons, vertices, creases, and facets after user edits.
- Expose the same contract through `treemaker-wasm`.

Done when:

- Rust unit tests cover creating, editing, saving, optimizing, and building a
  small tree.
- WASM tests cover create/edit/snapshot/optimize/build/save/free.

### Phase 3: Tree Drawing MVP

Status: not started.

Goal: make the central Design pane a real editor for drawing trees.

Work items:

- Render paper, grid, tree edges, nodes, labels, leaf circles, paths, and
  selection in SVG.
- Support select, drag, add node, add edge, delete, pan, zoom, and fit view.
- Add Inspector controls for tree, node, and edge selections.
- Add dirty-state tracking and local project state.

Done when:

- A user can create a three-leaf tree from a blank document, set edge lengths,
  and save a `.tmd5`.

### Phase 4: Optimization Workflow

Status: started; the browser app now runs the engine in a Web Worker and can
optimize the starter tree from the toolbar.

Goal: run optimization from the browser without freezing the UI.

Work items:

- Run `treemaker-wasm` inside a Web Worker via Comlink.
- Add Optimize Scale as the primary action.
- Surface busy state, errors, optimization reports, feasibility, and active
  path overlays.
- Add Edge and Strain optimization after Scale is reliable in the UI.

Done when:

- A drawn tree can be optimized and the visual node positions update from the
  engine snapshot.

### Phase 5: Crease Pattern Generation

Status: started; the browser app can request crease-pattern generation from
the worker after optimization and map the resulting snapshot into the CP pane.

Goal: render generated crease patterns clearly and exportably.

Work items:

- Add Build Crease Pattern command.
- Render creases, vertices, facets, fold assignment, and paper border.
- Add MVF and AGRH color modes plus overlay toggles.
- Explain partial/failure states from `cp_status_report`.

Done when:

- A simple optimized tree can generate a visible crease pattern in the Crease
  Pattern pane.

### Phase 6: Files, Examples, And Persistence

Status: not started.

Goal: make the app usable across sessions.

Work items:

- Add browser open/import, save/download, autosave, and unsaved-change guards.
- Add examples based on existing generated tree families.
- Preserve imported crease-pattern payloads until the first edit.
- Add SVG and PNG export for crease renderings.

Done when:

- A browser-only user can load, edit, optimize, build, save, reload, and export.

### Phase 7: Conditions And Advanced Controls

Status: not started.

Goal: expose TreeMaker constraints without recreating the old menu-heavy UX.

Work items:

- Add symmetry settings and overlays.
- Add condition creation/editing for fixed nodes, edge/corner constraints,
  paired/symmetric nodes, active paths, angle constraints, fixed edge lengths,
  and same-strain edges.
- Add condition diagnostics and selection affordances.

Done when:

- The common symmetry and constraint workflows from TreeMaker tutorials are
  expressible in the web UI.

### Phase 8: Tauri Desktop App

Status: not started.

Goal: wrap the same app in a desktop shell without forking the UI.

Work items:

- Add Tauri v2 app structure.
- Add native file dialogs, menus, recent files, close guards, and window title
  integration.
- Keep browser mode and desktop mode behind a small platform bridge.

Done when:

- Desktop builds can open/save local `.tmd5` files and run the same core UI
  workflow as the web app.

### Phase 9: Release And Maintenance

Status: not started.

Goal: make the GUI easy to validate, discover, and release.

Work items:

- Add CI for Rust, WASM, web typecheck/build, unit tests, and Playwright smoke
  tests.
- Update README and docs to present Rust API, CLI, WASM, web app, and desktop
  app surfaces clearly.
- Keep this roadmap updated as each phase lands.

Done when:

- The hosted/browser app and desktop app are documented release surfaces.

## Testing Strategy

- Rust unit tests for design editing, path rebuilding, invalidation, import,
  export, optimization, and crease-pattern generation.
- WASM tests for the JS boundary and structured error envelopes.
- Web unit tests for stores, coordinate transforms, selection, render mapping,
  and inspector edits.
- Playwright smoke tests for draw -> optimize -> build CP -> save/reload.
- Visual checks for nonblank Design and Crease Pattern panes at desktop and
  mobile sizes.
