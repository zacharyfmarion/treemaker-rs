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
- Model the visible editing UX after Oriedita actions, not raw kernel mouse
  handlers. The UI can call lower-level Rust operations internally, but the
  rail, shortcuts, prompts, inspector controls, and command palette should be
  organized around user actions such as "Draw crease", "Set line type to
  mountain", "Select lasso", or "Angle restricted" variants.
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
- Keep active tool state separate from active tool options. Oriedita's line
  color/type selector, auxiliary input mode, angle system, division ratio, and
  snap/input resolver are tool options that feed actions; they should not be
  exposed as duplicate operation buttons.

### UI Shape

The Crease Pattern pane should grow into coordinated regions:

- Canvas: the landed pane viewport already owns pan, zoom, fit, zoom presets,
  1:1, keyboard zoom, and space-drag panning. The roadmap should preserve that
  baseline while adding selection, snapping, command previews, diagnostics, and
  live editable CP geometry.
- Left tool rail: the main Oriedita-style command rail should live on the left
  side of the CP pane. It should be icon-first, vertically grouped, and stable
  while the viewport pans or zooms. Overflow, command search, and inspector
  controls can supplement it, but they should not replace the primary left rail.
- Bottom-right contextual tool panel: active action settings, readouts, and
  step-specific controls should appear inside the CP pane in an OpenSCAD
  Studio-style contextual panel. The left rail chooses actions; the contextual
  panel edits the active action's options.
- Inspector/status region: selected entity properties, command prompts,
  validation messages, and operation results. The inspector can mirror selected
  geometry state, but active tool options should not depend on the inspector
  being open.

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
- Add active-tool option controls in a bottom-right contextual panel rather
  than making them depend on the inspector:
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

### Stage 7: Action-Based Editing UX Alignment

Intent:

- Correct the CP editor's mental model before adding more command families.
  Stage 6 made many kernel-backed operations callable, but the visible web
  surface should be organized like Oriedita actions and tool options rather
  than one button per Rust operation.

UX work:

- Replace the operation-centric left rail with an action-centric rail and
  palette-ready action registry:
  - line type selector: M, V, E, A,
  - draw actions,
  - construction action groups and dropdown variants,
  - selection add/remove/set actions,
  - transform actions,
  - edit/repair actions,
  - overflow/search for rare commands.
- Keep the rail two buttons wide on desktop and compact two-wide on narrow
  panes.
- Add active tool options for line type now. Keep auxiliary input mode, angle
  system, division count/ratio, parallel width, and candidate choice as
  action metadata until the Stage 7.5 contextual panel gives them visible
  controls.
- Make `Draw crease` behave like Oriedita's visible action: click-drag-release,
  using the current line type and input mode.
- Distinguish free draw, restricted draw, and auxiliary-line draw as action
  variants instead of treating color and insertion mode as hardcoded payloads.
- Keep unsupported action variants visible but disabled with explicit
  implementation status.

Technical work:

- Add an action-layer registry over the kernel operation registry. Each visible
  action should map to:
  - upstream `ActionType` where one exists,
  - upstream `MouseMode` or service/task behavior,
  - Rust operation ID or operation sequence,
  - tool option requirements,
  - validation status.
- Keep kernel operation IDs out of primary UI grouping. They remain dispatch
  targets and oracle identifiers, not the main user-facing taxonomy.
- Add CP tool preferences for current fold-line color, auxiliary line color,
  fold-line additional input mode, angle settings, and division parameters.
- Add an input resolver that can choose Oriedita-compatible snap behavior per
  action step: object coordinate, existing point, line segment, box, lasso path,
  polygon point, or construction candidate.
- Make preview and commit use the same resolved inputs so a visual snap cannot
  commit a near-miss point.
- Preserve undo/redo semantics: changing active options does not create history;
  committed actions do.

Validation:

- Unit-test the action registry so every visible command maps to upstream
  action/source data or an explicit unsupported reason.
- Unit-test input resolver modes and preview/commit input agreement.
- Browser-test draw crease drag, endpoint snapping, M/V/E/A color selection,
  restricted draw rejection, and two-column rail layout.
- Add oracle fixtures for pointer-sequence-sensitive actions where resolved
  kernel inputs are not enough to prove parity.

Done when:

- The CP editor feels action-based: users choose an action and options, then act
  on the canvas. Kernel operation names are still traceable in code and tests
  but no longer define the visible editing UX.

### Stage 7.5: Contextual Tool Options Panel

Intent:

- Replace the temporary active-tool input overlay with an OpenSCAD
  Studio-style bottom-right contextual panel before enabling more
  setting-heavy tools.
- Preserve Oriedita's `MouseHandlerSettingGroup` model so controls are shown
  because the active action needs them, not because individual components add
  one-off overlays.

Reference plan:

- Detailed design and tool-setting inventory live in
  `implementation-plans/oristudio-cp-contextual-tool-options.md`.

UX work:

- Add a collapsible `CpContextToolPanel` anchored to the bottom-right of the CP
  viewport, separate from the bottom-center pan/zoom toolbar.
- Keep M/V/E/A as persistent line type controls in the left rail, with their
  red/blue/black/cyan color identity preserved even when active.
- Show active action settings only when relevant:
  - division count,
  - exact division ratio,
  - angle system divider/custom angles,
  - replace/delete line type filters,
  - fix-inaccurate precision toggles,
  - polygon corner count,
  - Voronoi apply state,
  - measurement slot readouts,
  - custom circle color,
  - candidate choice/readout for ambiguous construction tools.
- Keep text editing as an on-canvas editor, matching Oriedita, while allowing
  the contextual panel to show selected-text metadata later if useful.
- Collapse or compact the panel on narrow panes so it does not cover the paper
  or conflict with drawing handles.

Technical work:

- Add an action setting-group registry equivalent to Oriedita's
  `MouseHandlerSettingGroup`, modeled after OpenSCAD's optional
  `contextPanel` tool registry.
- Move the current `CpActiveToolOptions` spike into registry-backed panel
  components.
- Replace simplified ratio `S:T` state with Oriedita's exact
  `A + B * sqrt(C) : D + E * sqrt(F)` model and computed readout.
- Reconcile angle-system defaults before claiming parity. Oriedita's model
  resets to divider/current 8, while the current Rust dispatch helper defaults
  to 4.
- Extend command payloads for settings that are currently hardcoded, especially
  fix-inaccurate precision options and exact ratio coefficients.
- Keep settings changes out of undo/redo history; only committed canvas actions
  create checkpoints.

Validation:

- Unit-test the setting-group registry so every Oriedita group has a web home
  or an explicit not-applicable reason.
- Unit-test defaults for division count, exact ratio, angle system, polygon
  count, line filters, and fix precision against Oriedita source defaults.
- Web-test that changing settings updates previews and final command payloads
  consistently without creating undo checkpoints.
- Visual-test panel placement, collapsed state, narrow-pane behavior, and
  non-overlap with the bottom viewport toolbar.

Done when:

- Active CP tools use one consistent bottom-right contextual settings surface,
  and every known Oriedita tool setting or input has a planned web control.

### Stage 8: Circles, Text, Generators, And Measurement

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

### Stage 9: Diagnostics, Checks, And Repairs

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

### Stage 10: Folding Estimate And Folded Figure UX

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

### Stage 11: Menus, Shortcuts, Desktop Parity, And Polish

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

### Stage 12: Visual And Oracle Validation Harness

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
- Contextual tool options: use a bottom-right CP contextual tool panel modeled
  after OpenSCAD Studio's 3D viewer. Do not put active tool settings in the
  inspector as their primary home, and do not keep adding one-off viewport
  overlays.
- Document mode: keep TreeMaker design documents and editable CP documents
  explicit. Avoid silently converting generated CPs into editable CP files.
- Left-rail density: expose all commands, but use groups, overflow, command
  search, and disabled explanations so the pane remains usable.
- Action taxonomy: visible CP editing should be action-first. Kernel operations
  remain necessary for parity and tests, but the user should not need to
  understand Oriedita mouse-handler names or Rust operation IDs to operate the
  editor.
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
- `apps/web/src/lib/oristudioCpActions.ts` or equivalent action-layer registry
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
- [x] Stage 5: Enable core selection, deletion, color, assignment, move, copy,
      lengthen, and operation-frame commands.
- [x] Stage 5: Add web command tests and visual workflows for each enabled
      core edit command.
- [x] Stage 6: Enable drawing and geometric construction tools with previews.
- [x] Stage 6: Validate candidate previews and final mutations separately.
- [x] Stage 7: Replace operation-centric CP editing with an action-based rail
      and palette-ready action model.
- [x] Stage 7: Add M/V/E/A line type state, Oriedita-style input modes, and
      per-action snap/input resolvers.
- [x] Stage 7: Convert draw crease free/restricted to Oriedita-style
      click-drag-release preview and commit semantics.
- [x] Stage 7: Validate action registry coverage, preview/commit input
      agreement, and pointer-sequence-sensitive draw behavior.
- [x] Stage 7.5 planning: Audit OpenSCAD's viewer contextual panel pattern and
      Oriedita's contextual setting groups.
- [x] Stage 7.5: Replace temporary active-tool inputs with the bottom-right
      contextual tool panel.
- [x] Stage 7.5: Add registry-backed controls for every Oriedita tool setting
      and all additional web context inputs.
- [x] Stage 7.5: Validate settings defaults, undo neutrality, preview/commit
      payload agreement, and responsive panel placement.
- [x] Stage 8 slice: Enable length and angle measurement tools as local
      contextual readouts.
- [x] Stage 8 slice: Validate measurement probes as non-mutating UI state with
      no CP undo/redo history entries.
- [ ] Stage 8: Enable circles, text, generators, and measurement tools.
- [ ] Stage 8: Validate annotation preservation and non-mutating measurement
      behavior.
- [ ] Stage 9: Enable checks, diagnostics, issue navigation, and repair
      commands.
- [ ] Stage 9: Validate diagnostic overlays and repair results against the
      Oriedita oracle.
- [ ] Stage 10: Enable folding estimate sessions, folded-figure preview, another
      solution, duplicate, two-color CP, and batch export as kernel support
      becomes ready.
- [ ] Stage 10: Validate folded-session command sequences and visual previews.
- [ ] Stage 11: Add menus, shortcuts, desktop parity, accessibility, responsive
      behavior, and performance polish.
- [ ] Stage 11: Validate shared command dispatch across left rail, keyboard,
      web menus, and Tauri menus.
- [ ] Stage 12: Add the fixture gallery or developer validation route.
- [ ] Stage 12: Add scripted visual checks for canonical CP workflows.
- [ ] Stage 12: Keep semantic oracle validation as the gate for marking any
      command complete.
