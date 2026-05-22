# OriStudio CP Drawing Tool Parity

## Goal

Bring the crease-pattern drawing tools back into parity with Oriedita's actual
interaction model before continuing to later UI stages.

The Stage 6 kernel work made the Rust operations callable, but the web UI
currently exposes too many kernel operation names directly. Oriedita separates
visible actions, active line color/type, input mode, snapping, and handler
state. The web app should preserve that mental model while keeping Ori Studio's
pane-based design.

## Source Findings

- `MouseHandlerDrawCreaseFree` is a click-drag-release tool, not a two-click
  point-sequence tool. It updates `anchorPoint` on mouse move, updates
  `releasePoint` and `dragSegment` while dragging, and commits on release.
- Free draw snaps both endpoints through `d.getClosestPoint(p)` only when the
  point is within `d.getSelectionDistance()`.
- `MouseHandlerDrawCreaseRestricted` is stricter than free draw: the start must
  resolve to an existing closest point, and release only commits if the end is
  also within selection distance.
- Oriedita's M/V/E/A controls are not separate draw tools. They set
  `CanvasModel.lineColor` to red, blue, black, or cyan. Drawing tools read that
  current color when previewing and committing.
- Auxiliary line drawing is not "Draw crease with cyan" alone. Oriedita uses
  `FoldLineAdditionalInputMode`: regular fold-line drawing uses
  `POLY_LINE_0`, auxiliary drawing uses `AUX_LINE_1`, and some delete/right-click
  flows temporarily use `BOTH_4`.
- The visible Drawing tab is organized around actions and dropdowns:
  free draw, restricted draw, angle-restricted variants, lengthen, flat-fold
  vertex, perpendicular/parallel dropdown, symmetry tools, line division,
  generators, selection dropdowns, transforms, vertex/edit/type actions.
- Oriedita's selection distance is camera-aware. The worker starts from the
  configured mouse radius and divides by camera zoom when zoomed in. The web UI
  currently uses a fixed fraction of model span, which is close enough for some
  selection but not exact for drawing input.

## Current Mismatches

- The web `Draw crease` tool is point-sequence/click-click, while Oriedita free
  draw is click-drag-release.
- The live guide is partly driven by asynchronous preview state. For simple
  free draw this can trail the pointer; Oriedita's preview is synchronous local
  state.
- The web snap target mixes vertices, points, lines, and grid through one
  nearest-target pass. Oriedita draw endpoints use closest fold-line endpoints,
  circle centers, and then grid candidates through `getClosestPoint`; line
  interiors are not endpoint snap targets for free/restricted draw.
- Preview and commit can resolve points through different state paths. That is
  how a user can see a near-vertex guide but commit a slightly different point.
- The left rail groups are operation-centric. They should become action-centric,
  with active line type and active command as separate state.
- The rail is currently one button wide. The next UI change should make it two
  buttons wide as requested, while preserving compact pane ergonomics.

## Approach

### Stage 1: Stabilize The Existing Draw Crease UX

- Change the left tool rail to two button columns.
- Make the simple draw-crease preview synchronous from pointer state.
- Use one resolved point path for status, preview, and commit.
- Prefer Oriedita endpoint/center/grid snapping for draw endpoints.
- Keep line-interior snapping available for tools that actually pick a line.
- Add regression tests for endpoint snapping, preview/commit agreement, and
  two-column rail rendering.

### Stage 2: Port Oriedita Input Semantics

- Add a web-side `CpInputResolver` that mirrors Oriedita's `getClosestPoint`
  behavior:
  - closest fold-line endpoint,
  - closest circle center,
  - visible grid point when grid is enabled,
  - selection-distance gating.
- Make selection distance zoom-aware using the current transform scale and the
  same mouse-radius concept Oriedita uses.
- Split snap intents by step type: object point, existing point, line segment,
  box, polygon/lasso path, candidate choice.
- Add unit tests for snap ordering and thresholds.

### Stage 3: Add Active Line Type State

- Add CP tool preferences for:
  - current fold-line color: mountain/red, valley/blue, edge/black,
    auxiliary/cyan,
  - auxiliary live-line color where needed,
  - fold-line additional input mode.
- Add an M/V/E/A selector at the top of the CP rail or inspector tool options.
- Route draw/color operations through this state instead of hardcoded red
  defaults.
- Preserve the distinction between drawing cyan fold-line/auxiliary semantics
  and Oriedita auxiliary-line insertion.

### Stage 4: Rebuild The Visible Tool Model Around Actions

- Introduce an action-layer registry parallel to the kernel operation registry.
- Map Oriedita `ActionType` entries to visible Ori Studio commands, including
  dropdown variants.
- Keep kernel operation IDs in payload dispatch only; do not expose every kernel
  operation as a first-class rail button.
- Reorganize the left rail into Oriedita-like sections:
  - line type,
  - draw/construction,
  - selection/transform,
  - edit/repair,
  - overflow for rare commands.
- Keep unsupported actions visible but disabled with explicit status.

### Stage 5: Match Draw/Construction Interaction Modes

- Convert free/restricted draw to click-drag-release.
- Keep multi-step construction tools as multi-step where Oriedita does.
- Add candidate selection semantics for angle, axiom, foldable-line, and
  symmetry tools instead of using nearest candidate implicitly when the UI needs
  an explicit user choice.
- Add per-tool option controls for division count, ratio, angle divider, custom
  angles, and parallel width.

### Stage 6: Oracle And Visual Validation

- Extend the Oriedita oracle with handler-level draw input fixtures where raw
  pointer sequence matters, not only resolved kernel inputs.
- Compare final canonical CP outputs for free/restricted draw, color selection,
  aux-line insertion, and snap-threshold edge cases.
- Add browser smoke tests that exercise actual pointer drag, endpoint snap,
  color selector, and visible rail layout.
- Capture before/after visual screenshots for draw preview and committed
  vertex alignment.

## Affected Areas

- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/CpToolRail.tsx`
- `apps/web/src/lib/creasePatternViewport.ts`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/lib/oristudioCpToolState.ts`
- `apps/web/src/store/workspaceStore/*`
- `apps/web/src/styles/theme.css`
- `crates/oristudio-cp/src/lib.rs`
- `crates/oristudio-cp-wasm/src/lib.rs`
- `tools/oriedita-oracle`
- `third_party/oriedita`

## Checklist

- [x] Stage 1: Make the rail two buttons wide.
- [ ] Stage 1: Remove async lag from simple draw-crease preview.
- [ ] Stage 1: Make draw preview and commit use the same snapped endpoint.
- [ ] Stage 1: Add endpoint-snap and rail-layout tests.
- [ ] Stage 2: Add Oriedita-style input resolver and zoom-aware selection distance.
- [ ] Stage 2: Split snap intents by command step type.
- [ ] Stage 3: Add active M/V/E/A line type state.
- [ ] Stage 3: Route draw commands through current line type and input mode.
- [ ] Stage 4: Add visible action registry over kernel operation registry.
- [ ] Stage 4: Rebuild the left rail around Oriedita action groupings.
- [ ] Stage 5: Convert free/restricted drawing to click-drag-release.
- [ ] Stage 5: Add command-specific candidate choice and option controls.
- [ ] Stage 6: Add handler-level Oriedita oracle fixtures for pointer sequences.
- [ ] Stage 6: Add browser smoke and visual validation for drawing parity.
