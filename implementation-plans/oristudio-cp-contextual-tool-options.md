# OriStudio CP Contextual Tool Options

## Goal

Replace the temporary crease-pattern active-tool input overlay with a
bottom-right contextual tool panel modeled after OpenSCAD Studio's 3D viewer,
while preserving Oriedita's setting-group semantics.

This file now tracks both the planning audit and the Stage 7.5 implementation
work. It identifies every Oriedita tool setting or input that needs a visible
home before more CP tools are enabled in the web app.

## Approach

### Reference Pattern

OpenSCAD Studio uses a clean split between tool selection and tool-specific
controls:

- `apps/ui/src/components/three-viewer/ViewerToolPalette.tsx` renders a compact
  left-side tool palette.
- `apps/ui/src/components/three-viewer/viewerToolRegistry.ts` lets each active
  tool optionally register a `contextPanel`.
- `apps/ui/src/components/three-viewer/panels/ToolPanel.tsx` renders the active
  context panel as a collapsible, bottom-right overlay inside the viewer pane.
- `MeasurePanel`, `SectionPlanePanel`, and `BBoxPanel` keep tool-specific
  help, readouts, toggles, segmented controls, sliders, and reset actions out
  of the main tool palette.

The CP editor should copy that architecture, not the exact 3D viewer content.
The left rail chooses actions. A bottom-right `CpContextToolPanel` exposes only
the controls, readouts, and hints needed by the currently active CP action.

### Oriedita Source Model

Oriedita already has the right abstraction:

- `MouseHandlerSettingGroup` enumerates contextual groups: `LINE_COLOR`,
  `ANGLE_SYSTEM`, `ERASER_COLOR`, `SWITCH_COLOR`, `POLYGON_POINT_COUNT`,
  `LINE_DIVISION_COUNT`, `LINE_DIVISION_RATIO`, `APPLY_LINES`,
  `LINE_SELECT_HELP_TEXT`, and `FIX_PRECISION`.
- Mouse handlers return setting groups through `getSettings()`.
- `ToolSettingsPanel` hides all tool UIs, then shows only the groups returned
  by the active mouse handler.

The web app should introduce an equivalent action-level registry. The registry
belongs above Rust operation IDs because the visible UI is now action-based.

### Proposed Web Shape

- Add `CpContextToolPanel` inside the CP viewport, anchored bottom-right.
- Keep the current bottom-center viewport toolbar for pan/zoom/fit controls.
- Keep the left CP rail for actions and persistent M/V/E/A line type selection.
- Render no context panel when the active action has no settings, outputs, or
  step-specific help beyond the status strip.
- Make the panel collapsible and max-height constrained, matching OpenSCAD's
  `ToolPanel` behavior.
- Stop pointer propagation from panel controls so edits do not begin on the
  canvas while a user is changing a setting.
- On narrow panes, collapse by default or expose a compact settings button so
  the panel does not cover the paper.

### State Rules

- Tool settings are preferences, not document geometry.
- Changing a setting must not create an undo checkpoint.
- The preview and commit path must read the same settings snapshot.
- Pointer steps, candidate previews, and command payload assembly must share
  one resolver so visible previews cannot commit different values.
- Persist only preferences Oriedita treats as app/session settings. Do not
  write UI-only panel expansion state into CP files.

### Current Prototype To Replace

`CreasePatternPanel.tsx` currently has a small `CpActiveToolOptions` overlay for
division count and ratio. It is useful as a spike, but it should not become the
final UX:

- It is positioned top-right instead of bottom-right.
- It is component-local instead of registry-backed.
- The ratio UI is simplified to `S:T`; Oriedita uses
  `A + B * sqrt(C) : D + E * sqrt(F)`. The implemented panel now presents a
  friendlier left/right ratio editor and presets first, with the exact
  coefficient editor tucked under an `Exact form` disclosure.
- The default ratio should come from Oriedita's model (`1 : sqrt(2)`), not the
  temporary web default of `1:1`.

## Tool Settings Inventory

This inventory separates Oriedita setting groups from additional action state
the web UI must surface for full UX parity.

### Oriedita Setting Groups

| Setting group | Oriedita source | Tools/actions that use it | Required web controls | Parity notes |
| --- | --- | --- | --- | --- |
| `LINE_COLOR` | `LineColorPicker`, `CanvasModel.lineColor` | Persistent M/V/E/A line type used by draw and many construction operations | Keep M/V/E/A in the left rail; optionally mirror active type as a compact readout in the context panel | No current handler returns this group, but line color is a persistent option in Oriedita. It should not be duplicated as four draw tools. |
| `ANGLE_SYSTEM` | `AngleSystemUi`, `AngleSystemModel` | `AngleSystem`, `DrawCreaseAngleRestricted`, `DrawCreaseAngleRestricted3`, `DrawCreaseAngleRestricted5`, line-select base handlers | Divider stepper, divider preset label, custom A/B/C angle fields, custom-mode apply button, current mode highlight | Oriedita reset sets divider/current to 8, B divider to 12, and custom angles to 40/60/80 and 30/50/100. The current Rust dispatch default is 4; reconcile before marking parity. |
| `ERASER_COLOR` | `EraserColorUi`, `CanvasModel.delLineType` | `DeleteLineTypeSelect` | Custom line type select: Any, Edge, M/V, Mountain, Valley, Aux | Current web payload hardcodes `Any`; the panel must own this setting. |
| `SWITCH_COLOR` | `SetLineColorUi`, `CanvasModel.customFromLineType/customToLineType` | `ReplaceLineTypeSelect` | Source line type select, destination line type select, swap button | Destination list is Edge/Mountain/Valley/Aux. Swap is disabled when source is Any or M/V. |
| `POLYGON_POINT_COUNT` | `PolygonUi`, `ApplicationModel.numPolygonCorners` | `PolygonSetNoCorners` | Integer stepper/input for corner count | Oriedita defaults to 5 and clamps to 3..100. |
| `LINE_DIVISION_COUNT` | `LineDivisionUi`, `ApplicationModel.foldLineDividingNumber` | `LineSegmentDivision` | Integer stepper/input for division count | Oriedita defaults to 2 and clamps to at least 1. Confirm whether `1` should be allowed as a no-op before keeping the current web minimum of 2. |
| `LINE_DIVISION_RATIO` | `LineDivisonRatioUi`, `InternalDivisionRatioModel` | `LineSegmentRatioSet` | Left/right ratio fields, presets, computed `S:T` readout, and an exact `A + B * sqrt(C)` over `D + E * sqrt(F)` disclosure | Default is `A=1, B=0, C=0, D=0, E=1, F=2`, which computes `1 : sqrt(2)`. Negative radicands clamp to 0; negative computed halves clear the corresponding multiplier. |
| `APPLY_LINES` | `ApplyLinesUi` | `VoronoiCreate` | Apply button, seed count/readout, clear/reset affordance if the web state needs one | Oriedita's panel only exposes Apply; seed add/remove happens through canvas clicks. The web panel can show seed count without changing behavior. |
| `LINE_SELECT_HELP_TEXT` | `LineSelectHelpTextUI`, `BaseMouseHandlerLineSelect` | Intersecting line select/unselect/delete handlers that inherit `BaseMouseHandlerLineSelect` | Help text/readout, not a mutable setting | Oriedita says "Hold CTRL to use snapping." The web equivalent should match actual modifier behavior after snap parity is finalized. |
| `FIX_PRECISION` | `FixPrecisionUi`, `FixPrecisionModel` | `FixInaccurate` | 22.5 deg toggle, box-pleating toggle, precision slider, numeric precision field | Oriedita defaults to precision `0.05`, `useBP=true`, `use22_5=true`. Current web dispatch hardcodes defaults and the payload has no fields for these options yet. |

### Additional Web Context State

| Context state | Tools/actions | Required web controls | Parity notes |
| --- | --- | --- | --- |
| Fold-line additional input mode | Draw fold line vs auxiliary line variants | Action variant or compact toggle, not just cyan color | Oriedita distinguishes line color from `FoldLineAdditionalInputMode`; cyan alone is not full auxiliary-line semantics. |
| Construction candidate choice | Angle-system variants, axiom tools, parallel width, and other multi-candidate tools | Candidate list/readout or next/previous controls, plus canvas candidate picking | The Rust kernel can choose nearest candidate when `candidate_index` is absent, but Oriedita-style UX needs explicit visual candidate selection when candidates are ambiguous. |
| Parallel-width live width | `ParallelDrawWidth` | Live width readout; optional numeric override only if oracle review confirms it does not change Oriedita semantics | Oriedita derives width from a drag segment, then asks the user to choose one of two indicator lines. |
| Measurement slots | Length 1/2 and Angle 1/2/3 measurement tools | Read-only values for L1, L2, A1, A2, A3; active slot indicator | Oriedita stores these in `MeasuresModel`, and numeric text fields can reference `L1`, `L2`, `A1`, `A2`, and `A3`. |
| Text editing | `Text` | On-canvas text editor for selected text; context panel may show selected text metadata but should not replace direct editing | Oriedita positions a text area over the selected text and records on focus loss. Empty text is removed. |
| Custom circle color | `CircleChangeColor` | Color swatch/picker and selected circle/aux-line target readout | Oriedita uses `ApplicationModel.circleCustomizedColor`, default RGB roughly 100/200/200. |
| Grid and snap parameters | Grid-snapped drawing, grid-width-dependent construction, point selection | Grid visibility/snap toggles can stay with viewport/grid controls; active tool panel should show only tool-relevant snap state | `grid_width` should be derived from document grid metadata unless a specific Oriedita setting says otherwise. |
| Hit radius/selection distance | All point, line, and candidate pickers | Development/advanced setting or preferences, not a normal active-tool knob | Oriedita's selection distance is camera-aware. The web resolver should keep this as shared input infrastructure. |

### Input-Bearing Commands Outside The Panel

Some CP commands require user input but should not be owned by the bottom-right
tool panel:

- File import/export, save conversion, DXF/OBJ/FOLD/ORI/ORH options: use the
  existing file-service and dialog/menu flow from Stage 4.
- Folding estimate variants, "fold to case", "save 100", two-color CP, and
  duplicate/another-solution flows: belong to the folded-session UI from Stage
  10, with the Folded Base pane owning folded-result inspection.
- Background image positioning: remains UI-only/out-of-scope for the CP kernel
  unless background image support is deliberately added later.
- Global preferences such as grid colors, app colors, line widths, animation,
  and mouse radius: belong in preferences or viewport controls, though their
  resolved values may feed the active input resolver.

## Affected Areas

- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/CpToolRail.tsx`
- `apps/web/src/lib/oristudioCpActions.ts`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/lib/oristudioCpToolSettings.ts`
- `apps/web/src/lib/oristudioCpToolState.ts`
- `apps/web/src/engine/oristudioCpTypes.ts`
- `apps/web/src/store/workspaceStore/*`
- `apps/web/src/styles/theme.css`
- `crates/oristudio-cp/src/lib.rs`
- `crates/oristudio-cp-wasm/src/lib.rs`
- `third_party/oriedita/oriedita-ui/src/main/java/oriedita/editor/swing/toolsetting/*`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Audit OpenSCAD Studio's 3D viewer contextual panel pattern.
- [x] Audit Oriedita `MouseHandlerSettingGroup` and toolsetting UIs.
- [x] Inventory active CP tools that require settings, readouts, or explicit
      contextual controls.
- [x] Add a CP action setting-group registry modeled after Oriedita and
      OpenSCAD's `contextPanel` registration.
- [x] Replace `CpActiveToolOptions` with bottom-right `CpContextToolPanel`.
- [x] Move division count into the registry-backed panel.
- [x] Replace simplified ratio `S:T` inputs with the exact Oriedita
      `A + B * sqrt(C) : D + E * sqrt(F)` editor.
- [x] Add settings state for angle system, delete/replace line filters,
      polygon count, fix precision, measurements, custom circle color, and
      Voronoi apply state.
- [x] Extend command payloads for missing settings such as fix precision. Exact
      division-ratio coefficients stay in web tool state and dispatch their
      computed ratio values to the kernel, matching the current Rust operation
      contract.
- [x] Add tests proving contextual settings are visible before one-shot
      commands execute and that commit payloads use the edited settings.
- [x] Add undo-history coverage showing local settings changes do not create
      tree or CP history checkpoints.
- [x] Add browser visual checks for panel placement, collapse behavior, narrow
      pane behavior, and no overlap with the bottom viewport toolbar.
