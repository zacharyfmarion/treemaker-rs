# Oristudio CP UI Integration Roadmap

## Goal

Integrate the full Oriedita-style crease-pattern editing surface into the web
app's Crease Pattern pane without reducing the behavior surface.

This roadmap is for UI integration. The non-UI Rust port remains governed by
`implementation-plans/oriedita-port.md` and
`implementation-plans/oriedita-source-map.md`. The web UI should expose only
kernel-backed behavior, oracle-backed behavior, or explicit `Not implemented`
states. It should not approximate Oriedita operations with simpler local logic.

The first complete product target is:

- A user can open or generate a crease pattern.
- The Crease Pattern pane becomes a real editing workspace, not just a viewer.
- Every Oriedita CP edit, construction, check, file, and folding-estimate
  command has a visible home in the UI.
- Commands that are not ported yet are discoverable but disabled or return a
  clear `Not implemented` result.
- Each enabled command is covered by unit tests, oracle validation, and at
  least one visual workflow check.
- TreeMaker-generated crease patterns remain inspectable even before a user
  chooses to convert them into editable CP documents.

## Approach

### Product Principles

- Keep the Crease Pattern pane as a dense design-tool surface: compact toolbars,
  dockable panels, icon buttons with tooltips, mode groups, and a quiet
  inspector. Do not recreate Oriedita's Swing layout directly.
- Preserve Oriedita behavior, not Oriedita chrome. Button grouping, menus, and
  shortcuts can be modernized as long as command semantics stay identical.
- Treat the Oriedita source map as the completeness checklist. A UI command is
  not complete until it is mapped to an upstream behavior, a Rust command, a
  web command ID, and validation coverage.
- Use explicit capability states: `Ready`, `Porting`, `Not implemented`,
  `Unavailable for current selection`, and `Unavailable for current document`.
- Keep generated TreeMaker CPs and editable Oristudio CP documents distinct.
  Generated CPs start as read-only artifacts; editing them requires a deliberate
  conversion into an editable CP document.
- Make command feedback spatial. New, changed, selected, invalid, and diagnostic
  geometry should be visible on the drawing surface, not only in a toast.
- Keep one user command equal to one undo checkpoint. Pointer previews,
  candidate overlays, hover highlights, and measurement probes should not create
  history entries.
- Prefer a shared web command dispatcher from the start so browser controls,
  keyboard shortcuts, and later Tauri native menus call the same actions.

### UI Shape

The Crease Pattern pane should grow into three coordinated regions:

- Canvas: the landed pane viewport already owns pan, zoom, fit, zoom presets,
  1:1, keyboard zoom, and space-drag panning. The roadmap should preserve that
  baseline while adding selection, snapping, command previews, diagnostics, and
  live editable CP geometry.
- Left tool rail: the main Oriedita-style command rail should live on the left
  side of the CP pane. It should be icon-first, vertically grouped, and stable
  while the viewport pans or zooms. Overflow, command search, and inspector
  controls can supplement it, but they should not replace the primary left rail.
- Inspector/status region: selected entity properties, active tool options,
  command prompts, validation messages, and operation results.

The first viewport should still be the pattern itself. Controls should support
serious work without covering the paper. The app should avoid oversized cards,
marketing copy, or explanatory panels in the editing surface.

### Stage 0: UI Inventory And Command Taxonomy

Intent:

- Turn the Oriedita source map into a UI command matrix before wiring buttons.
- Decide where every upstream mouse mode, menu action, task, import/export
  action, and folding service command lives in the web app.

UX work:

- Define command groups:
  - Select and edit.
  - Draw folds and points.
  - Construct by geometry.
  - Transform and operation frame.
  - Color and assignment.
  - Circles, text, and annotations.
  - Generators and base molecules.
  - Measure.
  - Check and fix.
  - Fold estimate and folded figure.
  - File import/export.
- Define the left-rail placement, grouping, icon, tooltip, keyboard shortcut,
  and command-palette label for every command, including disabled commands.
- Decide which operations appear in the always-visible left rail versus
  overflow menus or command search. Oriedita has too many modes to expose as a
  single ungrouped wall of buttons without making the pane slower to use.

Technical work:

- Add a web command registry for CP commands with upstream source-map IDs,
  required capabilities, required selection, and current implementation status.
- Add capability queries from `oristudio-cp` or the WASM bridge so the UI does
  not hard-code which kernel commands are ready.
- Add a disabled-state renderer that can explain exactly why a command cannot
  run.

Validation:

- Unit-test the command registry so every source-map command has one UI entry
  or an explicit `Out-of-scope-ui` reason.
- Snapshot-test disabled labels and status copy.

Done when:

- Reviewing the command registry is enough to see which Oriedita features are
  surfaced, hidden, blocked, or intentionally UI-only.

### Stage 1: Oristudio CP Runtime Bridge

Intent:

- Give the web app a typed, worker-safe bridge to the `oristudio-cp` kernel.

UX work:

- Show document-level capability state in the CP pane:
  - Generated TreeMaker CP: read-only artifact.
  - Imported line-only CP: editable lines, limited simulation until topology is
    derived.
  - Imported/saved Oristudio CP: fully editable document metadata.
  - Unsupported file or operation: clear message with preserved source data
    when possible.

Technical work:

- Add a WASM bridge for `oristudio-cp`, either as a new
  `crates/oristudio-cp-wasm` crate or as a clearly separated module in the
  existing WASM package.
- Run CP operations in a worker, not on the main thread.
- Add TypeScript models for editable CP documents, line IDs, point IDs, circle
  IDs, text IDs, selected IDs, command results, diagnostics, folded-figure
  state, and unsupported-operation errors.
- Store CP documents separately from TreeMaker `project.creases` artifacts.
- Add conversion from generated TreeMaker CP artifacts into an editable CP
  document with an explicit user action.
- Add command-result highlighting: created IDs, changed IDs, deleted IDs,
  warnings, diagnostics, and next-step prompts.

Validation:

- Unit-test TypeScript mapping of WASM command results and errors.
- Add worker tests for load, serialize, mutate, undo checkpoint, and error
  propagation.
- Verify a read-only TreeMaker CP still renders exactly as it does today.

Done when:

- The web app can load an editable CP document in memory, call a no-op or
  simple kernel command through the worker, and update state without blocking
  the canvas.

### Stage 2: Viewport, Selection, Snapping, And History Foundation

Intent:

- Extend the zoomable CP viewport that landed on `main` into an editing-grade
  viewport. The current baseline already includes `react-zoom-pan-pinch`, fit,
  zoom in/out, zoom presets, 1:1, keyboard shortcuts, auto-fit on loaded CPs,
  and space-drag panning.

UX work:

- Preserve the landed pan/zoom/fit behavior and add zoom-to-selection, reset
  view, grid visibility, editable paper bounds, and optional rulers.
- Support selection for lines, vertices, facets/faces, circles, text, and
  folded-figure elements where applicable.
- Support click, shift/meta multi-select, box select, lasso-ready gesture
  capture, background clear, and Escape clear/cancel.
- Show hover affordances without changing selection.
- Add snap controls for grid, vertices, lines, intersections, angles, and
  active construction candidates.
- Add a status strip for cursor coordinates, active snap target, selected count,
  active tool prompt, and current document capability.

Technical work:

- Build on the existing `CreasePatternPanel`, `ViewportToolbar`, and
  `react-zoom-pan-pinch` wiring instead of replacing it.
- Introduce editable CP viewport state independent from the TreeMaker design
  viewport.
- Add hit testing and spatial indexing for large patterns.
- Decide rendering layers:
  - SVG is acceptable for small-to-medium editable documents and semantic hit
    targets.
  - Canvas or hybrid SVG/canvas should be introduced when performance testing
    shows large Oriedita patterns stutter.
- Add CP-specific undo/redo snapshots or command deltas. Do not reuse TreeMaker
  text snapshots if editable CP documents need a richer serialized form.

Validation:

- Keep the existing CP viewport tests for zoom controls, fit, keyboard
  shortcuts, and space-pan behavior green.
- Unit-test selection reducers and history granularity.
- Add Playwright or browser-plugin visual checks for zoom, pan, fit, selection,
  hover, disabled commands, and grid toggles.
- Add large-pattern performance fixtures before claiming the renderer scales.

Done when:

- A user can comfortably inspect and select any editable CP entity before
  command-specific tools exist.

### Stage 3: Tool Shell And Progressive Disclosure

Intent:

- Make the CP pane feel like an editor with a stable command surface, even
  while many commands are still disabled.

UX work:

- Add the primary tool shell as a left-side vertical rail inside the CP pane,
  following Oriedita's spatial convention while using Ori Studio's visual
  language.
- Add compact tool groups with icons and tooltips. Use text labels only where
  the command would otherwise be ambiguous.
- Keep viewport zoom controls separate from the main command rail. The current
  bottom viewport toolbar can remain focused on navigation unless later testing
  shows it conflicts with editing handles.
- Add a command search/palette entrypoint for rare Oriedita commands.
- Add active-tool option controls in the inspector/status area:
  - line color,
  - fold assignment,
  - angle system,
  - division count or ratio,
  - snap toggles,
  - selection mode,
  - circle mode,
  - preview/commit behavior.
- Add tool prompts for multi-step commands, for example "Pick source line",
  "Pick destination point", "Drag operation frame", and "Choose candidate".
- Add a consistent `Not implemented` presentation for visible but unavailable
  commands.

Technical work:

- Add a CP tool state machine that can represent Oriedita-style multi-step
  handlers without copying Swing event lifecycles into React.
- Map pointer input to model-space points, candidate queries, preview results,
  and final command commits.
- Add cancellation semantics: Escape cancels the current tool step first, then
  clears selection if no tool step is active.

Validation:

- Unit-test tool-state transitions for cancel, commit, reset, mode switch, and
  command error.
- Visual-test that active tools show prompts and previews without shifting
  layout.

Done when:

- Every Oriedita command has a visible UI home and a coherent state before
  feature-by-feature command enablement begins.

### Stage 4: CP File Workflows

Intent:

- Make editable CP documents durable before deep editing tools become primary.

UX work:

- Add Open/Import support for `.cp`, `.fold`, `.ori`, `.orh`, and supported
  Oriedita-related formats as kernel support becomes available.
- Add Save/Save As for the canonical editable CP document format.
- Add export for `.cp`, `.fold`, `.svg`, `.png`, and later `.dxf`/`.obj` where
  supported by the kernel.
- Surface lossy export warnings before writing.
- Preserve unsupported metadata and explain what is editable versus preserved.
- Keep browser and desktop behavior aligned through the existing file-service
  abstraction.

Technical work:

- Route CP file actions through shared command IDs, not component-local
  callbacks.
- Add CP document dirty state, title/path state, recent files, and unsaved
  navigation warnings.
- Preserve imported TreeMaker projects and imported CP documents as separate
  document modes.

Validation:

- Unit-test import/export state transitions and dirty-state behavior.
- Add oracle-backed round-trip tests for each enabled format.
- Add visual checks for open, save-as, lossy warning, and unsupported metadata
  messages.

Done when:

- A CP document can be opened, edited by at least one kernel command, saved,
  reopened, and visually confirmed in the pane.

### Stage 5: Core Editing Commands

Intent:

- Enable the commands that make an imported or generated CP practically
  editable.

UX work:

- Enable select/unselect by click, box, polygon, lasso, and intersecting line.
- Enable delete line, delete point, delete vertex on crease, delete overlapping,
  delete intersecting, and fix inaccurate.
- Enable line color and assignment changes: mountain, valley, edge, aux, toggle
  MV, advance type, alternate MV, replace type, delete type, and custom line
  type behavior.
- Enable move, copy, lengthen, four-point move/copy, and operation-frame
  transform.
- Show before/after highlights so bulk operations are understandable.

Technical work:

- Wire command IDs to `oristudio-cp` operations with selection and parameter
  payloads.
- Make kernel command errors visible but non-destructive.
- Keep selection stable across mutations where Oriedita does, and clear/remap
  selection where entity IDs no longer exist.

Validation:

- Each enabled command needs:
  - Rust unit tests in `oristudio-cp`.
  - Oracle validation against pinned Oriedita.
  - Web command unit tests.
  - At least one visual workflow check when the command has pointer UX.

Done when:

- A user can clean up and recolor a real CP without dropping into another app.

### Stage 6: Drawing And Construction Tools

Intent:

- Bring over the high-value Oriedita construction experience.

UX work:

- Enable draw crease free/restricted, draw point, divide by count, divide by
  ratio, perpendicular, parallel, parallel width, square bisector, inward,
  symmetric draw, mirror selected lines, double symmetric draw, continuous
  symmetric draw, fishbone, angle system, angle restricted variants, axiom 5,
  axiom 7, foldable line input/draw, and angular flat-foldable vertex tools.
- Show construction candidates as transient overlays with clear commit targets.
- Let users change tool parameters without leaving the active tool when that
  mirrors Oriedita behavior.

Technical work:

- Add candidate-query commands to the WASM bridge for multi-step previews.
- Add overlay layers for candidate lines, rejected candidates, snap points,
  operation handles, and final generated geometry.
- Persist only final committed geometry.

Validation:

- Oracle-test candidate generation separately from final mutation when Oriedita
  exposes distinct behavior.
- Visual-test multi-step construction prompts and preview overlays.

Done when:

- The CP pane can perform Oriedita's main crease construction workflows with
  the same generated geometry.

### Stage 7: Circles, Text, Generators, And Measurement

Intent:

- Complete the non-folding editing and annotation surface.

UX work:

- Enable circle creation modes: free, through three points, separate, tangent
  line, inverted, concentric, concentric select, and concentric two-circle
  select.
- Enable circle color changes and circle organization.
- Enable text creation/editing/preservation.
- Enable regular polygon, Voronoi, and default base molecule generators.
- Enable distance and angle measurement probes with non-mutating results.
- Keep annotations visually secondary to fold geometry unless selected.

Technical work:

- Add circle and text layers to rendering, hit testing, selection, inspector,
  serialization, and export.
- Add measurement result types that can be displayed and copied without
  mutating document history.

Validation:

- Oracle-test circle geometry, text persistence, generators, and measurement
  results.
- Visual-test annotation selection, editing, and export preview.

Done when:

- Circle-heavy and annotation-bearing Oriedita documents can be edited without
  data loss.

### Stage 8: Diagnostics, Checks, And Repairs

Intent:

- Make the CP window trustworthy for validating work, not just drawing it.

UX work:

- Enable Check1, Check2, Check3, Check4, CAMV, flat-foldable boundary checks,
  little-big-little diagnostics, Fix1, Fix2, and fix inaccurate.
- Add a diagnostics layer with clickable issue markers.
- Add a diagnostics panel list that filters, focuses, and explains issues.
- Keep repair commands explicit. Do not auto-fix without a user command.

Technical work:

- Add diagnostic result models with codes, locations, involved entity IDs,
  severity, and repair availability.
- Connect diagnostics panel selection to canvas focus/highlight.
- Persist or invalidate diagnostics according to document mutations.

Validation:

- Oracle-test diagnostic codes and normalized locations.
- Unit-test diagnostics panel filtering and focus behavior.
- Visual-test issue overlays on representative invalid patterns.

Done when:

- Users can locate and repair the same classes of CP problems Oriedita reports.

### Stage 9: Folding Estimate And Folded Figure UX

Intent:

- Surface the Oriedita folding-estimate workflow once the non-UI folding
  session APIs are ready.

UX work:

- Add folding commands:
  - Fold estimate order 1 through 5.
  - Fold estimate order 6 where supported.
  - Fold to case.
  - Another solution.
  - Duplicate folded figure.
  - Two-color CP.
  - Save 100 simulations/export batch.
  - Change standard face, move calculated shape, modify calculated shape, and
    folding constraints when the kernel exposes them.
- Add folded-figure session state with visible solution number, face count,
  subface count, warning status, and whether another solution is available.
- Reuse or extend the existing Folded Base pane for folded-figure previews
  rather than hiding results inside the CP command rail.
- Make `Another solution` disabled only when the kernel says no next solution
  exists, not merely because the UI has lost session state.
- Treat Save 100 carefully:
  - browser mode may need a ZIP or explicit multi-download flow,
  - desktop mode can ask for a directory,
  - filenames should match oracle-tested batch naming.
- Two-color CP must respect Oriedita's selection/list assumptions. Until the
  required selection workflow exists, the button should be visible but explain
  the missing prerequisite.

Technical work:

- Add folded-session state to the workspace store separate from editable CP
  document state.
- Add command APIs for starting, continuing, duplicating, and exporting folded
  sessions.
- Add rendering for folded faces, face order overlays, selected face, and
  calculated-shape handles.
- Add cancellation and progress reporting for long folding estimates.

Validation:

- Oracle-test folded-session command sequences and batch filenames.
- Add visual tests for first solution, next solution, duplicated solution, and
  folded-preview rendering.
- Add large-pattern timeout/cancellation tests.

Done when:

- A user can run and inspect Oriedita-equivalent folding estimates from the CP
  window without losing the editable CP document state.

### Stage 10: Menus, Shortcuts, Desktop Parity, And Polish

Intent:

- Make the CP editor feel integrated into the whole app instead of bolted onto
  one pane.

UX work:

- Add CP commands to app menus, contextual menus, keyboard shortcuts, and
  command search.
- Add context-sensitive inspector actions for selected lines, vertices, circles,
  text, diagnostics, and folded faces.
- Add accessibility labels, focus rings, keyboard navigation, and tooltip
  coverage.
- Ensure compact responsive behavior for narrow panes.
- Add performance affordances for large documents: layer toggles, simplified
  rendering while panning, and command progress.

Technical work:

- Route Tauri native menu IDs through the same CP command registry.
- Persist CP viewport, visible layers, and tool preferences where appropriate.
- Add telemetry-free local performance counters for development diagnostics if
  needed.

Validation:

- Unit-test menu and shortcut dispatch.
- Visual-test responsive pane widths and dark/light themes.
- Run web lint, typecheck, unit tests, and production build before shipping a
  polished stage.

Done when:

- CP editing works through the left rail, menu, keyboard, and desktop shell
  paths with one shared command implementation.

### Stage 11: Visual And Oracle Validation Harness

Intent:

- Prevent the UI from quietly diverging from Oriedita as the command surface
  grows.

UX work:

- Add a fixture gallery or developer-only validation route for canonical CP
  documents, construction examples, invalid patterns, and folded solutions.
- Show command result summaries and oracle comparison status in development
  builds.

Technical work:

- Add scripted web workflows that load fixtures, run command sequences, and
  capture screenshots.
- Compare kernel outputs to Oriedita oracle canonical data.
- Compare web-rendered screenshots to approved baselines where the visual
  output is stable enough to make pixel checks useful.
- Add canvas/SVG sanity checks: nonblank render, expected bounds, selected
  entity visible, diagnostics visible, and no text/control overlap.
- Keep real user corpus files outside the repository. Use an external corpus
  path for broad compatibility checks.

Validation:

- Rust oracle tests remain the semantic source of truth.
- Web visual tests verify that the same command results are visible and usable.
- Manual QA checklists exist only for interactions that automated tools cannot
  inspect reliably.

Done when:

- Every enabled CP UI stage has semantic oracle coverage and visual evidence
  that users can see and control the result.

### Decision Points

- WASM packaging: prefer a separate `oristudio-cp-wasm` package if the bridge
  grows independently from TreeMaker engine APIs; prefer a module inside the
  existing bridge only if that keeps build and worker plumbing simpler without
  mixing responsibilities.
- Rendering backend: begin with the current SVG mental model, but measure large
  documents early. Move to hybrid canvas/SVG if hit testing or repaint costs
  make SVG unsuitable.
- Navigation controls: keep the landed bottom viewport toolbar as the navigation
  surface. The left rail is for CP editing commands, not zoom presets.
- Document mode: keep TreeMaker design documents and editable CP documents
  explicit. Avoid silently converting generated CPs into editable CP files.
- Left-rail density: expose all commands, but use groups, overflow, command
  search, and disabled explanations so the pane remains usable.
- Folding UI ownership: CP pane owns fold-estimate commands; Folded Base pane
  should own folded preview inspection when a folded result exists.

## Affected Areas

- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/ViewportToolbar.tsx`
- `apps/web/src/components/panels/FoldedBasePanel.tsx`
- `apps/web/src/components/panels/InspectorPanel.tsx`
- `apps/web/src/components/panels/DiagnosticsPanel.tsx`
- `apps/web/src/components/ui/*`
- `apps/web/src/store/workspaceStore/*`
- `apps/web/src/store/layoutStore.ts`
- `apps/web/src/commands/menuActions.ts`
- `apps/web/src/lib/creasePatternImport.ts`
- `apps/web/src/lib/creaseExport.ts`
- `apps/web/src/lib/selection.ts`
- `apps/web/src/lib/geometry.ts`
- `apps/web/src/lib/designViewport.ts`
- `apps/web/src/engine/*`
- `apps/web/src/workers/*`
- `apps/web/src/generated/*`
- `apps/web/src/styles/theme.css`
- `apps/tauri/src-tauri/*` for later native menu and file-dialog parity
- `crates/oristudio-cp`
- optional `crates/oristudio-cp-wasm`
- `crates/oracle-tests`
- `tools/oriedita-oracle`
- `tests/fixtures/oriedita` or equivalent shared fixture location

## Checklist

- [x] Stage 0: Build the CP UI command matrix from the Oriedita source map.
- [x] Stage 0: Assign every upstream command a left-rail/menu/palette home or an
      explicit UI-only/out-of-scope reason.
- [x] Stage 0: Add capability and disabled-state vocabulary for CP commands.
- [x] Stage 1: Add the `oristudio-cp` web runtime bridge.
- [x] Stage 1: Add editable CP document state separate from generated
      TreeMaker CP artifacts.
- [x] Stage 1: Add command-result and unsupported-operation mapping.
- [x] Stage 2 baseline: Rebase onto the landed CP pane pan/zoom/fit viewport
      work from `main`.
- [x] Stage 2: Extend the landed viewport with edit-grade CP viewport state.
- [x] Stage 2: Add CP selection, hit testing, snapping, history, and status UI.
- [x] Stage 2: Align editable CP grid rendering and grid snapping with
      Oriedita's paper-coordinate grid model.
- [x] Stage 2: Validate viewport behavior with unit and visual tests.
- [x] Stage 3: Add the left-side tool rail, command groups, active-tool
      prompts, and Not-implemented presentation.
- [x] Stage 3: Add CP tool-state transitions for multi-step commands.
- [x] Stage 4: Add CP open/import/save/export workflows and dirty-state
      handling.
- [x] Stage 4: Validate file workflows with oracle-backed round trips.
- [x] Stage 5 slice: Enable selected-line delete, mountain/valley/edge/aux,
      and toggle-M/V commands through oracle-tested kernel dispatch.
- [x] Stage 5 slice: Enable selected-line two-point and four-point move/copy
      commands through oracle-tested transform dispatch.
- [x] Stage 5 slice: Enable two-point overlapping/intersecting crease deletion
      commands through oracle-tested drag-line dispatch.
- [x] Stage 5 slice: Enable two-point intersecting-line select/unselect commands
      with kernel-selected flag sync back into the web selection model.
- [x] Stage 5 slice: Enable selected-line fix-inaccurate repair with
      oracle-tested default Oriedita fix options.
- [ ] Stage 5: Enable core selection, deletion, color, assignment, move, copy,
      lengthen, and operation-frame commands.
- [ ] Stage 5: Add web command tests and visual workflows for each enabled
      core edit command.
- [ ] Stage 6: Enable drawing and geometric construction tools with previews.
- [ ] Stage 6: Validate candidate previews and final mutations separately.
- [ ] Stage 7: Enable circles, text, generators, and measurement tools.
- [ ] Stage 7: Validate annotation preservation and non-mutating measurement
      behavior.
- [ ] Stage 8: Enable checks, diagnostics, issue navigation, and repair
      commands.
- [ ] Stage 8: Validate diagnostic overlays and repair results against the
      Oriedita oracle.
- [ ] Stage 9: Enable folding estimate sessions, folded-figure preview, another
      solution, duplicate, two-color CP, and batch export as kernel support
      becomes ready.
- [ ] Stage 9: Validate folded-session command sequences and visual previews.
- [ ] Stage 10: Add menus, shortcuts, desktop parity, accessibility, responsive
      behavior, and performance polish.
- [ ] Stage 10: Validate shared command dispatch across left rail, keyboard,
      web menus, and Tauri menus.
- [ ] Stage 11: Add the fixture gallery or developer validation route.
- [ ] Stage 11: Add scripted visual checks for canonical CP workflows.
- [ ] Stage 11: Keep semantic oracle validation as the gate for marking any
      command complete.
