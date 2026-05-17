# TreeMaker Product Roadmap

This roadmap tracks the browser and eventual desktop product surface for
TreeMaker. `ROADMAP.md` tracks the Rust engine port. `WEB_ROADMAP.md` tracks
implementation phases for the web app. This file is the product backlog: what
the app should let a user do, in the order we should make it useful.

The original wxWidgets TreeMaker app is a feature reference, not a UX template.
The new app should preserve the important origami-design capabilities while
presenting them as a modern pane-based design tool.

## Product Goal

A user should be able to start from a blank tree, draw and constrain a design,
optimize it, generate a crease pattern, inspect problems, save/reopen the work,
and export a usable pattern from either the browser or the later Tauri desktop
app.

## Phase 0: Current Browser Spine

Status: in progress.

Goal: prove the end-to-end engine loop in a modern UI.

Implemented or started:

- Empty Design workspace.
- Click empty paper to add a node.
- Click empty paper with a node selected to add and attach a new node.
- Select, drag, and delete nodes and edges.
- Basic node and edge inspector editing.
- Design and Crease Pattern panes as tabs in the main work area.
- WASM engine in a Web Worker.
- Scale optimization.
- Build crease pattern and switch to the CP tab.
- Basic CP rendering with MVF/AGRH color modes.

Done when:

- A user can draw a simple three-leaf tree from a blank document, optimize it,
  build a visible crease pattern, and understand whether the action succeeded.

## Phase 1: Saveable Browser MVP

Priority: highest.

Goal: make browser work durable and shareable.

Features:

- Open/import `.tmd`, `.tmd4`, and `.tmd5`.
- Save/download canonical `.tmd5`.
- Export legacy `.tmd4`.
- Preserve imported crease-pattern payloads until the first structural edit.
- Autosave recent browser projects.
- Unsaved-change warnings.
- Example gallery using checked-in generated families.
- Project title and basic document metadata.

Done when:

- A browser-only user can load an existing TreeMaker file, edit it, optimize,
  build CP, save, reload, and get the same design back.

## Phase 2: History, Clipboard, And Selection

Priority: highest after files.

Goal: make editing safe enough for real design work.

Features:

- Undo and redo for all document edits and optimization/build commands.
- Cut, copy, paste, clear/delete.
- Multi-selection for nodes, edges, paths, creases, facets, and conditions.
- Select all, select none, select by index.
- Select path from two selected nodes.
- Select movable parts.
- Select corridor facets after CP generation.
- Better keyboard shortcuts and command routing.

Done when:

- A user can experiment without fear, select the parts they mean, and recover
  from mistakes without rebuilding the design.

## Phase 3: Core Tree Editing Tools

Priority: high.

Goal: cover the important original Edit menu workflows without copying its
menu-heavy structure.

Features:

- Make selected node the root.
- Split selected edge by ratio or absolute distance.
- Set selected edge length.
- Scale selected edge lengths.
- Renormalize to selected edge.
- Renormalize to unit scale.
- Absorb selected nodes.
- Absorb redundant nodes.
- Absorb selected edges.
- Perturb selected nodes.
- Perturb all nodes.
- Remove strain from selection.
- Remove all strain.
- Relieve selection strain.
- Relieve all strain.
- Stub tools: pick stub nodes, pick stub poly, add largest stub nodes, add
  largest stub poly.
- Triangulate tree.

Done when:

- Common model cleanup and refinement operations from the original app are
  available from contextual commands or inspector actions.

## Phase 4: Full Optimization Workflow

Priority: high.

Goal: bring the original optimization power into a non-blocking browser UI.

Features:

- Edge optimization for selected movable nodes/strainable edges.
- Strain minimization.
- Selection-based optimization.
- Progress reporting from the worker.
- Cancel optimization.
- Revert or keep partial results after cancellation or poor convergence.
- Clear feasibility states and failure reasons.
- Diagnostics for equality/variable overconstraint warnings.
- Active path and infeasible path overlays.

Done when:

- A user can choose the right optimization type, see progress, cancel safely,
  and understand why an optimization failed or produced strain.

## Phase 5: Conditions And Symmetry

Priority: high for serious TreeMaker users.

Goal: expose TreeMaker constraints as clear, inspectable design rules.

Features:

- Paper size editor.
- Symmetry line setup, edit controls, and overlay.
- Node fixed to symmetry line.
- Node fixed to paper edge.
- Node fixed to corner.
- Node fixed to position, including X-only and Y-only constraints.
- Nodes paired about symmetry line.
- Nodes collinear.
- Edge length fixed.
- Edges same strain.
- Path active.
- Path angle fixed.
- Path angle quantized.
- Remove node, edge, path, and all conditions.
- Condition list panel.
- Condition inspector for every condition type.
- Condition overlays and feasibility diagnostics.

Done when:

- Designs from TreeMaker tutorials that rely on symmetry and constraints can be
  built in the new app without opening the old app.

## Phase 6: Crease Pattern Workspace

Priority: medium-high.

Goal: make generated crease patterns inspectable, exportable, and trustworthy.

Features:

- Kill/rebuild crease pattern.
- CP status report with bad edges, polys, vertices, creases, and facets.
- Select and inspect vertices, creases, facets, polys, and paths.
- Fold assignment display controls.
- MVF and AGRH color modes with clear legend.
- Overlay toggles for vertices, indices, labels, flags, facet order, facet
  arrows, corridor edges, and facet fills.
- SVG export.
- PNG export.
- Print and print-preview equivalent for browser and desktop.

Done when:

- A user can generate a CP, inspect every relevant part, diagnose bad output,
  and export a pattern suitable for downstream use.

## Phase 7: View Settings And Navigation

Priority: medium.

Goal: replace the old View Settings palette with fast, task-oriented controls.

Features:

- Pan, zoom, and fit controls.
- Fit to screen, width, and height.
- Default, Tree, Creases, and Plan view presets.
- Toggle symmetry lines.
- Toggle leaf, branch, and sub nodes.
- Toggle node dots, circles, indices, coordinates, elevation, depth, labels,
  and flags.
- Toggle edge dots, lines, indices, lengths, strain, labels, and flags.
- Toggle leaf, branch, sub, active, border, polygon, infeasible, and conditioned
  paths.
- Toggle poly, vertex, crease, facet, and condition overlays.
- Persist per-project or per-user view settings.

Done when:

- Dense designs remain readable because users can quickly switch visual layers
  for the task at hand.

## Phase 8: Desktop App With Tauri

Priority: medium after browser MVP.

Goal: wrap the same app in a native desktop shell without forking product logic.

Features:

- Tauri v2 app shell.
- Native open/save/save-as dialogs.
- Native recent files.
- Native app menus and keyboard shortcuts.
- Window title and modified-state integration.
- Close guards for unsaved work.
- Native print path where useful.
- Preferences storage.
- Platform bridge shared by browser and desktop modes.

Done when:

- The desktop app feels like a normal local design tool while sharing the same
  editor, engine worker, and roadmap as the browser app.

## Phase 9: Help, Learning, And Release Polish

Priority: medium.

Goal: make the app discoverable for new users and reliable for release.

Features:

- Built-in tutorial flow for first tree -> optimize -> CP.
- Help/about pages with TreeMaker licensing and engine notes.
- Command search.
- Error messages that suggest a next action.
- Accessibility pass for keyboard and screen reader basics.
- Playwright smoke tests for core workflows.
- Visual regression checks for Design and CP panes.
- Hosted demo documentation.
- Release notes and screenshots.

Done when:

- A new user can learn the workflow without the old documentation, and a release
  can be validated repeatedly before shipping.

## Deferred Or Debug-Only Original Features

These existed in or around the original app but should not lead product work
unless a real workflow needs them:

- Debug-only tree state generators.
- Low-level optimizer backend toggles.
- Internal cleanup timing flags.
- Legacy wxWidgets window-management quirks.
- Exact menu hierarchy parity.

## Roadmap Maintenance

- Update phase status when a phase becomes started or complete.
- Keep original-app parity features here, implementation details in
  `WEB_ROADMAP.md`, and engine internals in `ROADMAP.md`.
- Prefer product-complete slices over menu-for-menu parity.
