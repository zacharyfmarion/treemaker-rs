# Oriedita Port Roadmap

## Goal

Port the non-UI crease-pattern editing functionality from Oriedita into Rust
without reducing the behavior surface. The first milestone is a tested Rust
kernel and oracle workflow that can support the same operations Oriedita
supports, even though wiring every individual button into the TreeMaker UI is
out of scope for this plan.

The port must be direct and parity-oriented:

- Oriedita behavior is the canonical reference for this work.
- Unported operations must be represented explicitly and return a typed
  `UnsupportedOperation` or `NotImplemented` result.
- Do not replace Oriedita behavior with simpler approximations.
- Every ported operation needs focused Rust unit tests and oracle validation
  against Oriedita.
- Staging is allowed; scope reduction is not.

## Source Baseline

Use a pinned Oriedita source snapshot as the oracle baseline. The research
snapshot inspected for this plan was:

- Repository: `https://github.com/oriedita/oriedita`
- Commit: `9d39135ae232cc03be4ffaf74baa7ae2df970507`
- License: MIT

The important upstream modules are:

- `origami`: non-UI origami and folding logic.
- `oriedita-data`: save models plus `.cp`, `.fold`, `.ori`, `.orh`, `.obj`, and
  `.dxf` import/export support.
- `oriedita-common`: shared service interfaces and application models.
- `oriedita`: service glue, command tasks, and mouse-handler implementations.
- `oriedita-ui`: Swing UI. This plan uses it only to understand operation
  intent; direct UI porting is out of scope.

The upstream code areas that must be source-mapped before implementation are:

- `origami/crease_pattern/*`
- `origami/crease_pattern/element/*`
- `origami/crease_pattern/util/*`
- `origami/crease_pattern/worker/*`
- `origami/data/*`
- `origami/folding/*`
- `oriedita-data/src/main/java/oriedita/editor/export/*`
- `oriedita-data/src/main/java/oriedita/editor/save/*`
- `oriedita/src/main/java/oriedita/editor/handler/*`
- `oriedita/src/main/java/oriedita/editor/task/*`
- `oriedita/src/main/java/oriedita/editor/service/impl/FoldingServiceImpl.java`

## Scope

In scope for this plan:

- A Rust crate for editable crease-pattern data and operations.
- Oriedita-compatible geometry primitives, tolerances, and classification
  behavior.
- Oriedita-compatible line colors and fold assignments.
- Circle, text, grid, camera-independent metadata, and Oriedita FOLD extension
  preservation.
- Import/export behavior for Oriedita-supported crease-pattern file formats
  where the behavior is non-UI.
- Command-level APIs for every Oriedita crease-pattern editing operation.
- Unit tests for primitives, algorithms, operations, and serializers.
- Oracle validation against a headless Oriedita harness.
- Explicit placeholders for unported operations.

Out of scope for this initial plan:

- Designing or wiring every Oriedita button into the React UI.
- Recreating Oriedita's Swing layout, panels, or icon taxonomy.
- Pixel-perfect drawing of Oriedita UI previews.
- Shipping a public Rust crate before parity coverage is credible.

UI integration can start only after the kernel exposes stable command APIs and
the oracle harness can prove behavior for the commands being surfaced.

## Proposed Architecture

Add a new crate:

- `crates/oristudio-cp`

Use existing crates instead of duplicating them:

- `treemaker-fold` remains the FOLD document and serialization layer.
- `treemaker-flatfold` remains the flat-foldability and layer-order solver.
- `treemaker-core` continues to own TreeMaker tree generation.
- `treemaker-wasm` exposes the CP kernel to the web app after kernel parity is
  established.

Suggested crate modules:

- `geometry`: Oriedita-compatible point, line, segment, circle, polygon,
  rectangle, orientation, distance, projection, intersection, angle, rotation,
  symmetry, and epsilon behavior.
- `model`: mutable crease-pattern document, stable IDs, line colors,
  assignment mapping, selection flags, circles, text, grid metadata, and
  Oriedita-specific metadata.
- `fold_graph`: conversion between editable CP state and `FoldDocument`,
  topology inference, faces, edges, vertices, and canonical graph comparison.
- `operations`: command-level editing operations matching Oriedita mouse modes
  and menu actions.
- `checks`: flat-foldability checks, Maekawa/Kawasaki/Fushimi-style diagnostics,
  little-big-little checks, and repair/fix operations.
- `folding`: Oriedita folding-estimation parity surface where it is not already
  covered by `treemaker-flatfold`.
- `io`: `.cp`, `.fold`, `.ori`, `.orh`, `.obj`, `.dxf`, and Oriedita extension
  import/export behavior.
- `oracle`: test-only canonicalization helpers and fixtures shared with
  `crates/oracle-tests`.

Public API shape:

- `CreasePatternDocument`
- `CreasePatternCommand`
- `CommandResult`
- `CommandError::UnsupportedOperation`
- `CommandError::InvalidInput`
- `CommandError::OracleMismatch` for test harness reporting only
- `OperationStatus::{Ported, Unsupported}`

The command API should be UI-agnostic. It should accept model-space points,
selected IDs, command parameters, and document state. It should not know about
React, Swing, pointer events, toolbars, or keyboard shortcuts.

## Oracle Strategy

Use Oriedita as a semantic oracle, not as a raw file byte oracle.

Create a headless Java oracle under one of:

- `third_party/oriedita` for a pinned upstream source snapshot.
- `tools/oriedita-oracle` for a small Java CLI that links to the pinned source.
- `crates/oracle-tests` for Rust tests that call the oracle binary.

The oracle CLI should accept:

- An input `.cp`, `.fold`, `.ori`, or JSON fixture.
- A JSON command sequence.
- Oracle options: epsilon policy, starting face, include faces, include circles,
  include text, include checks, include folded output.
- A requested output kind: canonical document, operation diagnostics,
  foldability report, folding snapshot, or exported file.

The Rust side should apply the same command sequence and compare canonical
outputs.

Do not compare raw `.fold` text as the main assertion. Canonicalize instead:

- Normalize coordinate precision by the operation-specific epsilon.
- Deduplicate and sort vertices by coordinates.
- Sort undirected edges by endpoint coordinates plus assignment/color.
- Compare faces modulo cyclic rotation and reversed winding when appropriate.
- Compare line colors and FOLD assignments separately.
- Compare circles by center, radius, color, and custom color.
- Compare text by position and content.
- Compare Oriedita namespaced metadata as structured values.
- Compare diagnostics by code, location, and tolerance-normalized payload.
- For folding layer orders, compare invariants and valid partial orders when
  multiple exact layer orders are acceptable.

Oracle requirements for each operation:

- Minimal fixture.
- Edge-case fixture near tolerance boundaries.
- Fixture with existing intersections.
- Fixture with circles/text where the operation should preserve them.
- Fixture with auxiliary lines where Oriedita behavior differs from fold lines.
- Round-trip fixture where applicable.

Oracle tests should be gated so normal Rust unit tests do not require Java:

- Unit tests: always run with `cargo test -p oristudio-cp`.
- Oracle tests: run when `ORIEDITA_ORACLE=tools/oriedita-oracle/build/...` is
  set.
- Corpus tests: run only against an external corpus path and never commit
  private user files.

## Test Strategy

Every operation should have three layers of coverage:

- Primitive unit tests: exact geometry and epsilon behavior.
- Operation unit tests: Rust-only fixtures for command semantics.
- Oracle tests: Rust result versus Oriedita result after canonicalization.

Test data layout:

- `crates/oristudio-cp/tests/fixtures`
- `crates/oracle-tests/tests/oriedita_oracle.rs`
- `tools/oriedita-oracle/fixtures`
- `tests/fixtures/oriedita` for shared open fixtures if useful
- external corpus path for non-committed real-world patterns

Required test categories:

- Import/export round trips.
- Topology inference and face reconstruction.
- Line addition and automatic splitting.
- Intersection and overlap classification.
- Selection and deletion behavior.
- Move, copy, rotate, scale, mirror, and operation-frame transforms.
- Line color and assignment conversion.
- Circle construction and circle-line/circle-circle intersections.
- Text preservation.
- Foldability checks and diagnostics.
- Folding estimation stages and snapshots.
- Error behavior and `UnsupportedOperation` stability.

## Non-UI Functional Inventory

This inventory is intentionally broad. Items may be implemented in stages, but
they should not disappear from the port.

### Core Geometry

Upstream references:

- `OritaCalc`
- `Point`
- `LineSegment`
- `Line`
- `StraightLine`
- `Circle`
- `Polygon`
- `Rectangle`
- `Epsilon`

Port all behavior for:

- Point equality with Oriedita tolerances.
- Distances, projections, nearest points, and nearest segment distances.
- Segment and infinite-line intersection classifications.
- Sweet versus strict intersection rules.
- Parallel, overlapping, crossing, T-junction, endpoint, and degenerate cases.
- Angles in Oriedita's degree ranges.
- Segment length changes.
- Segment rotation and point rotation.
- Parallel offsets.
- Symmetry/reflection across a line.
- Perpendiculars and bisectors.
- Internal division ratios.
- Triangle/circumcenter helpers.
- Circle-circle intersections.
- Circle-line tangencies and non-intersection helper lines.
- Polygon containment and path interactions used by box/lasso selection.

### Editable Crease Pattern Model

Upstream references:

- `FoldLineSet`
- `LineSegmentSet`
- `PointSet`
- `PointLineMap`
- `LineColor`
- `CustomLineTypes`
- `LineSegmentSave`

Port all behavior for:

- Fold line, auxiliary line, and auxiliary live line storage.
- Oriedita line colors: black, red, blue, cyan, orange, magenta, green, yellow,
  purple, and other/custom.
- Assignment conversion between Oriedita colors and FOLD assignments.
- Selection flags and selection modes.
- Stable save/load of selected, unselected, auxiliary, and custom-color lines.
- Circle storage and custom circle colors.
- Text storage and preservation.
- Grid size/style metadata where it affects file behavior.
- Bounding boxes and document extents.
- Connected-component selection.
- Point maps and vertex-neighborhood queries.
- Quad-tree or equivalent spatial acceleration with parity-safe results.

### Line Arrangement Operations

Upstream references:

- `FoldLineSet.divideLineSegmentWithNewLines`
- `FoldLineSet.divideIntersectionsFast`
- `LineSegmentSetWorker`
- `IntersectDivide`
- `OverlappingLineRemoval`
- `BranchTrim`
- `Fix1`
- `Fix2`

Port all behavior for:

- Adding segments.
- Splitting new lines against existing lines.
- Splitting old lines against new lines.
- Removing exact and tolerance-overlapping lines.
- Deleting intersecting lines.
- Deleting overlapping lines.
- Vertex deletion and vertex merge behavior.
- Branch trimming.
- Repairing inaccurate vertices/segments.
- Preserving colors and active states through splits.
- Handling auxiliary-line replacement rules.
- Maintaining deterministic output ordering where Oriedita does.

### Selection Operations

Upstream references:

- `BaseMouseHandlerLineSelect`
- `MouseHandlerCreaseSelect`
- `MouseHandlerCreaseUnselect`
- `MouseHandlerSelectPolygon`
- `MouseHandlerUnselectPolygon`
- `MouseHandlerSelectLineIntersecting`
- `MouseHandlerUnselectLineIntersecting`
- `MouseHandlerSelectLasso`
- `MouseHandlerUnselectLasso`
- `OperationFrame`

Port all command behavior for:

- Select all.
- Unselect all.
- Box select and box unselect.
- Polygon select and polygon unselect.
- Lasso select and lasso unselect.
- Select by intersecting line.
- Unselect by intersecting line.
- Select connected from a point.
- Select by color/type.
- Delete selected.
- Export/copy selected subset semantics.

### Line Color and Assignment Commands

Upstream references:

- `MouseHandlerChangeCreaseType`
- `MouseHandlerCreaseAdvanceType`
- `MouseHandlerCreaseMakeMountain`
- `MouseHandlerCreaseMakeValley`
- `MouseHandlerCreaseMakeEdge`
- `MouseHandlerCreaseMakeAux`
- `MouseHandlerCreaseMakeMV`
- `MouseHandlerCreaseToggleMV`
- `MouseHandlerCreasesAlternateMV`
- `MouseHandlerReplaceTypeSelect`
- `MouseHandlerDeleteTypeSelect`
- `All_s_step_to_orisenAction`

Port all behavior for:

- Change clicked line type.
- Cycle/advance crease type.
- Set mountain.
- Set valley.
- Set edge/boundary.
- Set auxiliary.
- Set mountain/valley from context.
- Toggle mountain/valley.
- Alternate mountain/valley on selected or discovered sequences.
- Replace selected line type.
- Delete selected line type.
- Convert selected or all auxiliary lines to crease lines where Oriedita allows.
- Preserve custom colors and Oriedita-specific edge colors in FOLD extras.

### Drawing and Construction Commands

Upstream references:

- `MouseHandlerDrawCreaseFree`
- `MouseHandlerDrawCreaseRestricted`
- `MouseHandlerDrawCreaseSymmetric`
- `MouseHandlerDrawCreaseAngleRestricted`
- `MouseHandlerDrawCreaseAngleRestricted3_2`
- `MouseHandlerDrawCreaseAngleRestricted5`
- `MouseHandlerSquareBisector`
- `MouseHandlerInward`
- `MouseHandlerPerpendicularDraw`
- `MouseHandlerSymmetricDraw`
- `MouseHandlerDoubleSymmetricDraw`
- `MouseHandlerContinuousSymmetricDraw`
- `MouseHandlerParallelDraw`
- `MouseHandlerParallelDrawWidth`
- `MouseHandlerFishBoneDraw`
- `MouseHandlerFoldableLineInput`
- `MouseHandlerFoldableLineDraw`
- `MouseHandlerVertexMakeAngularlyFlatFoldable`
- `MouseHandlerAxiom5`
- `MouseHandlerAxiom7`

Port all command behavior for:

- Free line drawing.
- Restricted line drawing.
- Symmetric line drawing.
- Angle-restricted line drawing.
- 3-angle and 5-angle restricted variants.
- `DRAW_CREASE_ANGLE_RESTRICTED_13` and
  `DRAW_CREASE_ANGLE_RESTRICTED_3_18` are ported and oracle-tested for divider
  angle systems, including deterministic fan candidates, selected fan-line
  commits, convergence intersections, and worker-style fold-line insertion.
  The Rust converging custom-angle branch stays non-panicking for six angles;
  Oriedita's source indexes that branch as if the angle array were 1-based.
- `ANGLE_SYSTEM_16` is ported and oracle-tested for resolved two-point
  candidate generation and destination commit. It preserves divider/custom
  angle preview candidates, alternating indicator colors, and worker-style
  insertion after a chosen candidate/destination are resolved.
- Square bisector construction.
- Inward construction.
- Perpendicular construction.
- Single, double, and continuous symmetry construction.
- Parallel construction.
- Parallel-by-width construction.
- Fishbone construction.
- `FOLDABLE_LINE_INPUT_39` is ported and oracle-tested for generated
  foldability candidates, fallback/manual input segments, direct endpoint
  commit, and destination-line commit after UI resolution.
- `FOLDABLE_LINE_DRAW_71` is ported and oracle-tested for its non-UI routing
  behavior between free crease drawing and vertex flat-foldable construction,
  including the drag switch back to free drawing.
- `VERTEX_MAKE_ANGULARLY_FLAT_FOLDABLE_38` is ported and oracle-tested for the
  resolved invalid-vertex kernel: odd-degree candidate generation, single-line
  color preservation, and destination commit.
- `AXIOM_5` is ported and oracle-tested for indicator generation, direct
  indicator commit, and destination-intersection commit after the target point,
  target line, and pivot point are resolved.
- Axiom 7 construction.
- Preview-independent computed candidates for every construction command.

The Rust command API should expose deterministic candidate lists when Oriedita
offers multiple valid construction lines. The UI can later decide how to present
those candidates.

### Point and Vertex Commands

Upstream references:

- `MouseHandlerDrawPoint`
- `MouseHandlerDeletePoint`
- `MouseHandlerVertexDeleteOnCrease`
- `MouseHandlerLineSegmentDivision`
- `MouseHandlerLineSegmentRatioSet`

Port all behavior for:

- Draw point.
- Delete point.
- Delete vertex on crease.
- Divide segment by count.
- Divide segment by ratio.
- Set and apply internal division ratio.
- Maintain Oriedita's endpoint selection and tolerance behavior.

### Transform Commands

Upstream references:

- `MouseHandlerMoveCreasePattern`
- `MouseHandlerCreaseMove`
- `MouseHandlerCreaseCopy`
- `MouseHandlerCreaseMove4p`
- `MouseHandlerCreaseCopy4p`
- `MouseHandlerLengthenCrease`
- `MouseHandlerLengthenCreaseSameColor`
- `OperationFrame`

Port all behavior for:

- Move entire crease pattern.
- Move selected lines.
- Copy selected lines.
- Four-point move.
- Four-point copy.
- Lengthen clicked crease.
- Lengthen same-color connected creases.
- Operation-frame transforms that are independent of UI drawing.
- Preserve split/merge behavior after transformed lines are inserted.

### Circle Commands

Upstream references:

- `MouseHandlerCircleDraw`
- `MouseHandlerCircleDrawThreePoint`
- `MouseHandlerCircleDrawSeparate`
- `MouseHandlerCircleDrawTangentLine`
- `MouseHandlerCircleDrawInverted`
- `MouseHandlerCircleDrawFree`
- `MouseHandlerCircleDrawConcentric`
- `MouseHandlerCircleDrawConcentricSelect`
- `MouseHandlerCircleDrawConcentricTwoCircleSelect`
- `MouseHandlerCircleChangeColor`
- `OrganizeCircles`

Port all behavior for:

- Circle by center/radius.
- Circle through three points.
- Separate circle construction.
- Tangent-line construction.
- Inverted-circle construction.
- Free circle drawing.
- Concentric circle drawing.
- Concentric select and two-circle select modes.
- Circle color changes and custom colors.
- Circle intersection application to lines.
- Circle-circle and line-circle helper generation.
- Circle organization and cleanup behavior.

### Polygon, Base, and Pattern Generators

Upstream references:

- `MouseHandlerPolygonSetNoCorners`
- `MouseHandlerDrawPattern`
- `MouseHandlerDrawBlintz`
- `MouseHandlerDrawFishBase`
- `MouseHandlerDrawDoveBase`
- `MouseHandlerDrawBirdBase`
- `MouseHandlerDrawFrogBase`
- `MouseHandlerVoronoiCreate`

Port all behavior for:

- Regular polygon creation.
- Blintz base generation.
- Fish base generation.
- Dove base generation.
- Bird base generation.
- Frog base generation.
- Voronoi creation.

### Diagnostics and Checks

Upstream references:

- `Check1`
- `Check2`
- `Check3`
- `Check4`
- `FlatFoldabilityViolation`
- `LittleBigLittleViolation`
- `MouseHandlerFlatFoldableCheck`
- `CheckCAMVTask`

Port all behavior for:

- Check 1 diagnostics.
- Check 2 diagnostics.
- Check 3 diagnostics.
- Check 4 diagnostics.
- Combined angle and mountain/valley checks.
- Maekawa-style incorrect-type diagnostics.
- Kawasaki/Fushimi-style angle diagnostics.
- Little-big-little diagnostics.
- Diagnostic colors and affected-line/vertex references.
- Check result preservation in the document model.

### Folding Estimation and Folded-Model Data

Upstream references:

- `FoldedFigure`
- `FoldedFigure_Worker`
- `FoldedFigure_Configurator`
- `WireFrame_Worker`
- `HierarchyList`
- `SubFace`
- `Face`
- `CustomConstraint`
- `AdditionalEstimationAlgorithm`
- `SubFacePriority`
- `ItalianoAlgorithm`
- `SwappingAlgorithm`
- `FoldingEstimateTask`
- `FoldingEstimateSpecificTask`
- `FoldingEstimateSave100Task`
- `TwoColoredTask`
- `FoldingServiceImpl`

Port or account for all behavior for:

- Recognize crease pattern.
- Draw wire graph.
- Draw X-ray graph.
- Build development/folded paper state.
- Starting face behavior.
- Face and subface generation.
- Hierarchy list construction.
- Constraint generation and propagation.
- Additional estimation.
- Possible-overlap search.
- Find another overlap.
- Two-color crease-pattern generation.
- Folded-figure duplication semantics.
- Custom folding constraints.
- Modifying calculated shape.
- Moving calculated shape.
- Changing standard face.

Where `treemaker-flatfold` already provides equivalent or stronger behavior,
document the mapping and still validate against Oriedita fixtures. If behavior
differs, the Rust API must expose the Oriedita-compatible path separately or
return `UnsupportedOperation` until parity is implemented.

### Import and Export

Upstream references:

- `CpImporter`
- `CpExporter`
- `FoldImporter`
- `FoldExporter`
- `OriImporter`
- `OriExporter`
- `OrhImporter`
- `OrhExporter`
- `ObjImporter`
- `DxfExporter`
- `OrieditaFoldFile`
- `Save`, `SaveV1_0`, `SaveV1_1`, `SaveConverter`, `TextSave`

Port all non-UI behavior for:

- `.cp` import.
- `.cp` export.
- `.fold` import.
- `.fold` export.
- Oriedita FOLD extensions for circles, texts, custom colors, grid style, and
  grid size.
- `.ori` import.
- `.ori` export.
- Legacy `.ori` conversion.
- `.orh` import.
- `.orh` export.
- `.obj` import.
- `.dxf` export.
- Save conversion and version detection.
- File-format warnings as structured diagnostics rather than UI dialogs.

SVG, PNG, and JPG export are not part of the kernel's first milestone unless
they are needed for oracle fixtures. They can be planned as a later rendering
surface because the current app already owns SVG/PNG CP export.

## Staged Plan

### Stage 0: Source Map and Parity Matrix

Deliverables:

- Add this roadmap.
- Add `implementation-plans/oriedita-source-map.md`.
- Record every relevant upstream class, mouse mode, service command, task,
  importer, exporter, and diagnostic.
- Classify each item as one of:
  - `kernel`
  - `io`
  - `oracle-only`
  - `ui-preview-only`
  - `out-of-scope-ui`
- Assign a Rust target module for every `kernel` and `io` item.
- Add an implementation status table with `Unsupported` as the default.

Validation:

- `git diff --check`
- Manual source-map review against Oriedita commit
  `9d39135ae232cc03be4ffaf74baa7ae2df970507`

### Stage 1: Crate and Error Contract

Deliverables:

- Add `crates/oristudio-cp`.
- Define public data types and feature-status registry.
- Define `UnsupportedOperation` and `NotImplemented` errors.
- Add crate-level docs describing Oriedita parity discipline.
- Add no-op command registry entries for the full source-mapped operation list.
- Wire workspace dependencies and baseline tests.

Validation:

- `cargo fmt --check`
- `cargo test -p oristudio-cp`
- `cargo clippy -p oristudio-cp --all-targets -- -D warnings`

### Stage 2: Oriedita Geometry Primitives

Deliverables:

- Port points, lines, segments, straight lines, circles, polygons, rectangles,
  and epsilon constants.
- Port OritaCalc geometry functions with unit tests.
- Add strict and sweet intersection variants.
- Add test fixtures for all Oriedita intersection enum outcomes.

Oracle:

- Java oracle command for primitive geometry cases.
- Rust comparison tests for distances, intersections, angles, projections, and
  circle helpers.

Validation:

- `cargo test -p oristudio-cp geometry`
- `ORIEDITA_GEOMETRY_ORACLE=... cargo test -p oristudio-cp --test oriedita_geometry_oracle`

Status:

- Complete for primitive carriers and pure geometry helpers in
  `crates/oristudio-cp/src/geometry`.
- The Stage 2 oracle currently validates line-segment intersection
  classifications against pinned Oriedita source. Additional oracle commands
  for distances, angles, projections, polygons, and circles remain required
  before those helpers can be promoted from unit-tested to oracle-tested.
- OritaCalc helpers that require `FoldLineSet` mutation or Java path objects
  remain tracked under later model, arrangement, and selection stages rather
  than being exposed as supported Rust operations.

### Stage 3: Editable CP Model and Canonical Comparison

Deliverables:

- Port `FoldLineSet` and related data model semantics.
- Add Oriedita line colors and FOLD assignment mapping.
- Add circles, text, custom colors, selection flags, active states, and grid
  metadata.
- Add canonical graph serializer used by oracle tests.
- Add deterministic ID strategy for UI-facing use without changing semantic
  comparisons.

Oracle:

- Import simple Oriedita save data.
- Canonicalize and compare document state after load/save.

Validation:

- `cargo test -p oristudio-cp model`
- `ORIEDITA_MODEL_ORACLE=... cargo test -p oristudio-cp --test oriedita_model_oracle`

Status:

- Complete for the editable save/model carriers: fold lines, auxiliary lines,
  circles, points, text annotations, custom colors, selected/active line
  fields, grid metadata, Oriedita custom line type selectors, FOLD assignment
  mapping, deterministic element IDs, and canonical comparison.
- The model oracle currently validates `CustomLineTypes` against pinned
  Oriedita source. Full save-file and `FoldLineSet` mutation oracle coverage
  starts in Stage 4 and Stage 5 because it depends on import/export and line
  arrangement behavior.
- Spatial acceleration, `PointLineMap`, face/topology generation, and
  split/merge mutations remain explicitly tracked in later stages.

### Stage 4: Import and Export Parity

Deliverables:

- Port `.cp` import/export.
- Port `.fold` import/export with Oriedita extensions.
- Port `.ori` import/export and legacy conversion.
- Port `.orh` import/export.
- Port `.obj` import and `.dxf` export.
- Emit structured warnings for lossy formats.

Oracle:

- Round-trip Oriedita upstream test resources.
- Compare canonical graph state, circles, text, grid metadata, and custom
  colors.
- Compare exported `.cp` and `.fold` semantically.

Validation:

- `cargo test -p oristudio-cp io`
- `ORIEDITA_ORACLE=... cargo test -p oracle-tests --test oriedita_oracle io`

Status:

- In progress. CP import/export, FOLD JSON import/export for Oriedita extension
  arrays, `.ori` JSON save import/export for `v1` and `v1.1`, `.orh`
  import/export, OBJ import, and DXF export have Rust unit coverage.
- The Oriedita Java oracle now validates `.orh` import/export, `.obj` import,
  and `.dxf` export against pinned upstream source. `.dxf` export uses
  Java-style floating-point text formatting for exact exporter parity.
- FOLD import now applies Oriedita's line-only coordinate-normalization
  transform; circles and text intentionally remain in their source FOLD
  coordinates because Oriedita copies them outside the moved `FoldLineSet`.
  FOLD export now reconstructs Oriedita-style point/edge/face topology before
  writing, including the `Face` sentinel quirk that suppresses faces when the
  Euler check fails. The topology path is oracle-tested against Oriedita's
  `WireFrame_Worker`/`PointSet` classes; full raw `.fold` file comparison still
  needs a fold-library-aware oracle.
- `.ori` unknown/newer-version behavior is explicit: strict import rejects it,
  while a permissive entry point mirrors Oriedita's "open anyway" path. Legacy
  `.ori` conversion beyond the shared `v1`/`v1.1` payload and full
  save-version detection remain unsupported until their source-compatible
  parsers/exporters land.
- `.orh` preserves Oriedita's legacy importer quirks: auxiliary lines exported
  by Oriedita are not imported by Oriedita, and import preallocates an extra
  default line/circle slot. The `<Kousi>` grid section is parsed by Oriedita
  into a local copy after default grid state has already been copied into the
  save, so imported saves keep the default grid. These are covered as
  intentional parity behavior.

### Stage 5: Arrangement, Split, Merge, and Cleanup

Deliverables:

- Port line insertion.
- Port intersection splitting.
- Port overlap removal.
- Port point/vertex deletion.
- Port branch trimming.
- Port inaccurate-line fixes.
- Port connected component and point-line map behavior.
- Add spatial acceleration only after simple parity tests pass.

Oracle:

- Command-sequence fixtures for insert, split, overlap, delete, trim, and fix.
- Tolerance boundary fixtures.

Validation:

- `cargo test -p oristudio-cp arrangement`
- `ORIEDITA_ORACLE=... cargo test -p oracle-tests --test oriedita_oracle arrangement`

Status:

- In progress. `IntersectDivide` has a direct Rust implementation for pairwise
  crossing, T-shape, contained-overlap, and partial-overlap mutation semantics,
  plus a simple dynamic scan for full arrangement splitting. The Oriedita Java
  oracle now validates representative pairwise splits and a full crossing
  arrangement fixture.
- `FoldLineSet.divideLineSegmentWithNewLines` and `divideIntersectionsFast`
  have direct Rust implementations for insertion-time splitting, including the
  Oriedita cyan auxiliary-line branch behavior and exact-duplicate deletion
  staging. The Java oracle validates representative direct pair cases and
  full new-line insertion fixtures against real `FoldLineSet`.
- `OverlappingLineRemoval` has a direct Rust implementation with unit coverage
  for Oriedita's "keep the earlier duplicate" behavior and the optional
  precision radius.
- `FoldLineSet.deleteInsideLine` is ported for overlap-only deletion and
  overlap-or-X-intersection deletion, matching the tools behind
  `CREASE_DELETE_OVERLAPPING_64` and `CREASE_DELETE_INTERSECTING_65`. Both
  modes are covered by the real `FoldLineSet` Java oracle.
- `BranchTrim.apply`, `deleteLineSegment_vertex`, and the same-color
  `del_V(Point, ...)` vertex merge path are ported with oracle coverage,
  including Oriedita's branch-trim loop restart quirk and `del_V`'s always-false
  return value.
- The color-changing `del_V_cc(Point, ...)`, direct `del_V(LineSegment,
  LineSegment)`, `del_V_all`, and `del_V_all_cc` variants are ported and
  oracle-tested. The point-based color-changing path preserves Oriedita's
  immutable-line stale-color quirk, while the direct pair/all variants use
  Oriedita's explicit color-combination matrix.
- `Fix1.apply` and `Fix2.apply` are ported and oracle-tested for duplicate
  repair, inaccurate-overlap selection marking, and near-T split insertion
  order.
- `FIX_INACCURATE_107` is ported and oracle-tested as
  `checks::fix_inaccurate_for_indices`. The port includes Oriedita's automatic
  BP grid search, bundled 22.5-degree binary correction table, square/default
  xform behavior, selected-line replacement, and post-insertion line division.
  UI bulletin-board messages, selection gesture wiring, and async `check4`
  refresh are left for UI integration.
- The current arrangement worker intentionally uses simple scans instead of
  Oriedita's quadtree acceleration. That keeps mutation parity visible first;
  spatial acceleration remains deferred until broader split/merge behavior is
  oracle-backed.

### Stage 6: Selection, Color, and Transform Commands

Deliverables:

- Port selection commands.
- Port line type/color commands.
- Port delete/replace type commands.
- Port move/copy/4-point transform commands.
- Port lengthen commands.
- Port operation-frame semantics that affect model output.

Oracle:

- Command-sequence fixtures for all selection modes and all color/type changes.
- Transform fixtures with existing intersections and auxiliary lines.

Validation:

- `cargo test -p oristudio-cp commands_selection`
- `cargo test -p oristudio-cp commands_transform`
- `ORIEDITA_ORACLE=... cargo test -p oracle-tests --test oriedita_oracle commands`

Status:

- In progress. `operations::color` now ports the shared
  `FoldLineSet.setColor(Collection, LineColor)` behavior, including the
  auxiliary-live-line replacement path that deletes, re-adds, and insertion-splits
  converted cyan lines. The Oriedita oracle validates that path and
  `LineColor.changeMV`-style mountain/valley toggling.
- `operations::selection` now ports the FoldLineSet-level selection primitives:
  select/unselect all, index-based line select/unselect, box selection using
  `lineSegmentsInside`, polygon select/unselect using `select_Takakukei`, line
  intersection/overlap selection using `select_lX`, lasso selection/unselection
  with `INTERSECT_CONTAIN`, and `selectProbablyConnected`. These paths have
  Rust unit coverage and Oriedita oracle coverage. The tests preserve
  Oriedita's distinction between box selection, which selects any line touching
  a box boundary or interior, polygon selection, which ignores
  outside-to-outside lines that merely pass through the polygon, and lasso
  selection, which uses Java `Line2D`-style boundary intersection plus strict
  endpoint containment.
- Selected-line make mountain/valley/edge/aux, `CREASE_ADVANCE_TYPE_30`, the
  overlapping-line alternation used by `CREASE_MAKE_MV_34`, and the
  crossing-line alternation used by `CREASES_ALTERNATE_MV_36` now have Rust
  unit coverage and Oriedita oracle coverage. The make-fold-color commands
  include the handler-level `fix2` pass, `make_aux` uses Oriedita's delete/add
  replacement path rather than generic `setColor`, and advance-type preserves
  the delete-then-append ordering side effect.
- `REPLACE_LINE_TYPE_SELECT_72`, `DELETE_LINE_TYPE_SELECT_73`, and
  `FoldLineSet.delSelectedLineSegmentFast` now have Rust helpers and Oriedita
  oracle coverage. This slice also fixed `FoldLineSet.setColor(Collection,
  LineColor)` duplicate handling so a selected line value changes every equal
  duplicate, matching Oriedita's `HashSet` membership pass.
- `operations::transform` now ports the FoldLineSet mutations behind
  `CREASE_MOVE_21`, `CREASE_COPY_22`, `CREASE_MOVE_4P_31`, and
  `CREASE_COPY_4P_32`: selected-line extraction, delete-or-copy behavior,
  translation or four-point scale/rotate/translate, insertion splitting, and
  final unselect-all. The oracle validates translation, selected move/copy, and
  four-point move/copy against real `FoldLineSet`. Oriedita's
  `MOVE_CREASE_PATTERN_2` handler is camera panning rather than a persisted CP
  mutation, so it remains a later UI/runtime mapping rather than a kernel edit.
- `OPERATION_FRAME_CREATE_61` is ported and oracle-tested as a transient
  model-space frame state machine with identity-camera semantics. It preserves
  create, corner drag, side drag, box drag, endpoint/circle-center snapping,
  tiny-frame deactivation, and reset behavior; actual screen-camera projection
  remains a UI integration concern.
- `TEXT` is ported and oracle-tested for the non-UI annotation semantics:
  create-or-select press, selected text dragging, point deletion, box deletion,
  selected-text state updates, and Oriedita's default headless text bounds. Text
  editing widget focus, cursor changes, and rendered font metrics remain UI
  integration concerns.
- The core `OritaCalc.extendToIntersectionPoint_2` helper used by lengthen and
  several construction tools is ported and oracle-tested for crossing-line and
  collinear-endpoint fixtures. The higher-level `LENGTHEN_CREASE_5` and
  `LENGTHEN_CREASE_SAME_COLOR_70` model-space mutation is now ported through
  candidate-line discovery, target-line mode selection, current-color versus
  same-color handling, and worker-style insertion splitting. These rows remain
  `Porting` rather than fully oracle-tested because Oriedita's final
  `applyLineSegmentCircleIntersection` side effect still needs circle fixtures
  and a Rust equivalent.
- `LINE_SEGMENT_DIVISION_27` and `LINE_SEGMENT_RATIO_SET_28` are ported as
  `operations::point` commands once the handler has resolved its drag segment
  and numeric parameters. The port preserves Oriedita's generated subsegment
  formulas, ratio-command endpoint reversal, and worker-style insertion
  splitting against existing lines. The committed oracle fixtures are line-only;
  the `CreasePattern_Worker.addLineSegment` circle-intersection side effect
  remains tracked with the Stage 8 circle tool work.
- `CHANGE_CREASE_TYPE_4` is ported and oracle-tested for resolved line targets.
  The line-segment portions of `LINE_SEGMENT_DELETE_3` are ported for
  single-line delete-with-vertex-cleanup and box-resolved line deletion, but the
  handler remains `Porting` until circle deletion and separate aux-line storage
  deletion modes are covered.
- The display measurement tools are ported as pure `operations::measure`
  helpers. Both length slots share `length_between_points`; all three angle
  slots share `angle_between_three_points`, preserving Oriedita's
  `OritaCalc.angle(center, first, center, third)` orientation and 0-360 degree
  range.
- `DRAW_POINT_14` is ported as `draw_point_on_segment`: after the UI has
  resolved/snapped the target point and target segment, the kernel projects the
  target onto the segment and calls Oriedita's `applyLineSegmentDivide` behavior
  only for strict interior projections within the selection distance. This tool
  does not add an Oriedita save-model point.
- The shared line insertion portion of `DRAW_CREASE_FREE_1` and
  `DRAW_CREASE_RESTRICTED_11` is ported as `draw_crease_segment`, including
  fold-line worker insertion/splitting and separate aux-line insertion. These
  rows remain `Porting` until the surrounding snap/endpoint-state semantics and
  the worker's circle-intersection side effects are covered.
- `DRAW_CREASE_SYMMETRIC_12` is ported as `mirror_selected_lines` once the
  mirror axis is resolved. It snapshots selected lines, mirrors them with
  `OritaCalc.findLineSymmetryLineSegment`, splits inserted copies, and clears
  selection afterward like the handler.
- Full handler parity remains open for nearest-click line selection, Java2D
  path/lasso selection, operation-frame behavior, lengthen commands, and
  construction-handler `fix2` chaining that is not part of the selected color
  commands above.

### Stage 7: Construction Tools

Deliverables:

- Port free, restricted, symmetric, angle-restricted, perpendicular, parallel,
  bisector, inward, fishbone, foldable-line, angular-flat-foldable, axiom 5,
  and axiom 7 command behavior.
- Expose candidate previews as pure model results.
- Keep unsupported candidate-selection variants explicit until oracle-tested.

Oracle:

- One fixture per construction mode.
- Multi-candidate fixtures for axiom and angle tools.
- Degenerate and tolerance-boundary fixtures.

Validation:

- `cargo test -p oristudio-cp construction`
- `ORIEDITA_ORACLE=... cargo test -p oracle-tests --test oriedita_oracle construction`

Status:

- `PARALLEL_DRAW_40` and `PARALLEL_DRAW_WIDTH_51` are ported and
  oracle-tested for resolved model-space selections. The port preserves
  Oriedita's `s_step_additional_intersection` behavior, fixed-distance
  `moveParallel` indicators, selected indicator recoloring, and worker-style
  insertion splitting.
- `PERPENDICULAR_DRAW_9` is ported and oracle-tested for the immediate
  projection branch and the full-extend indicator branch after resolved
  model-space inputs. Destination selection uses the same
  `s_step_additional_intersection` helper as the parallel tool.
- `SYMMETRIC_DRAW_10` is ported and oracle-tested for resolved two-line
  construction inputs. Three-point UI selection maps to the same kernel path
  after Oriedita turns the three points into two construction segments.
- `DOUBLE_SYMMETRIC_DRAW_35` is ported and oracle-tested for the resolved drag
  axis kernel. The implementation preserves Oriedita's snapshot iteration,
  valid endpoint/T-intersection filter, far-endpoint reflection, extension to
  the next hit, and worker-style insertion splitting.
- `CONTINUOUS_SYMMETRIC_DRAW_52` is ported and oracle-tested for the resolved
  two-point kernel. The implementation preserves Oriedita's recursive
  lengthen-to-first-hit behavior, black-edge and start-loop stopping, T-vertex
  branching through surrounding fold lines, reflection across each hit line,
  and alternating mountain/valley insertion.
- `INWARD_8` is ported and oracle-tested for the resolved three-point kernel,
  preserving Oriedita's incenter calculation and per-ray insertion behavior.
- `SQUARE_BISECTOR_7` is ported and oracle-tested across all resolved kernels:
  the three-point destination branch, the non-parallel two-line destination
  branch, the parallel-line purple indicator branch, direct indicator commit,
  and the parallel-line two-destination commit.
- `FISH_BONE_DRAW_33` is ported and oracle-tested for resolved drag segments,
  including grid-station stepping, parallel-excluded proximity skips, forward
  hit detection, `extendToIntersectionPoint_2` rib placement, red/blue
  alternation, and Oriedita's `del_V` call when both ribs are added.
- `AXIOM_7` is ported and oracle-tested for resolved inputs, including purple
  indicator construction, direct indicator commit, and destination clipping via
  the same extended-segment helper used by the parallel/perpendicular tools.
- `DRAW_CREASE_ANGLE_RESTRICTED_5_37` is ported and oracle-tested for resolved
  model-space inputs, including active angle-system snapping, nearby line
  intersection correction, nearby endpoint/circle-center correction, and
  worker-style insertion. UI grid candidate snapping remains an input resolver
  concern for the later UI integration layer.

### Stage 8: Circle, Polygon, Base, and Generator Tools

Deliverables:

- Port all circle tools. Basic restricted/free/three-point circle creation,
  separate circles, concentric variants, inversion, and circle organization are
  now ported and oracle-tested for resolved model-space inputs. Custom circle
  and cyan auxiliary-line color changes are also oracle-tested for resolved
  targets. Tangent-line candidate generation is in progress with point-circle
  and common two-circle candidate fixtures covered. Regular polygon generation
  and the bundled Blintz, fish, dove, bird, and frog base generators are
  oracle-tested for resolved anchor points.
- Port regular polygon creation.
- Port Blintz, fish, dove, bird, and frog base generators.
- Port Voronoi creation. The stateful click/add/remove seed workflow,
  Oriedita's fast `voronoi_02` clipping routine, apply-to-lines behavior, and
  seed-circle creation are now ported and oracle-tested for resolved
  model-space points.
- Preserve generated-line colors and split behavior.

Oracle:

- Fixture per circle mode.
- Fixture per base generator; the five default molecule FOLD resources are
  vendored in `crates/oristudio-cp/resources/default-molecules/` and compared
  against an Oriedita oracle command that imports, transforms, filters,
  recolors, and inserts the same pattern lines.
- Voronoi fixtures with stable canonical comparison.

Validation:

- `cargo test -p oristudio-cp generated_geometry`
- `ORIEDITA_ORACLE=... cargo test -p oracle-tests --test oriedita_oracle generated_geometry`

### Stage 9: Diagnostics and Repair

Deliverables:

- Port the interactive `FLAT_FOLDABLE_CHECK_63` boundary-line probe.
- Port Check1 through Check4.
- Port combined angle/MV task behavior.
- Port flat-foldability violation structures.
- Port little-big-little diagnostics.
- Port repair commands tied to diagnostic state.
- Add structured diagnostic IDs suitable for future UI display.

Oracle:

- Compare diagnostic locations, types, and affected elements.
- Include fixtures from Oriedita upstream tests and new edge cases.

Validation:

- `cargo test -p oristudio-cp checks`
- `ORIEDITA_ORACLE=... cargo test -p oracle-tests --test oriedita_oracle checks`

Status:

- `FLAT_FOLDABLE_CHECK_63` is ported and oracle-tested as
  `checks::flat_foldable_boundary_check` for resolved boundary segments. The
  port preserves Oriedita's strict X-intersection requirement, folding-line-only
  crossing count, boundary-order sorting, repeated reflection composition, and
  cyan/magenta/yellow line-step coloring.
- `Check1` and `Check2` are ported and oracle-tested as `checks::check1` and
  `checks::check2`. These return Oriedita's diagnostic segment lists for
  non-auxiliary overlap/containment pairs and sweet-tolerance T-intersection
  pairs, preserving duplicate reporting and pair order.
- `Check3` is ported and oracle-tested as `checks::check3`. It returns
  Oriedita's legacy zero-length vertex marker list for boundary-count,
  Maekawa, interior extended-Fushimi, and side extended-Fushimi failures,
  preserving repeated endpoint passes and duplicate markers.
- `Check4` is ported and oracle-tested as `checks::check4` plus structured
  `FlatFoldabilityViolation` and `LittleBigLittleSegment` payloads. The port
  builds an Oriedita-style point-to-line map, evaluates number-of-folds,
  Maekawa, angle, and little-big-little rules, and preserves LBL line ordering
  and flags. The Rust API returns deterministic point-map order; the oracle
  command validates against Oriedita's static per-point checker rather than the
  UI executor queue order.
- `CheckCAMVTask` is ported and oracle-tested as `checks::check_camv_task`.
  The non-UI API recomputes `Check4` violations and returns the dirty flag that
  the Java task sets on `CanvasModel`; asynchronous executor scheduling remains
  UI/runtime integration.
- The selected-line precision repair path behind `FIX_INACCURATE_107` is
  ported and oracle-tested, including BP and 22.5-degree modes. Stage 9 repair
  work now covers `Fix1`, `Fix2`, and the precision-data command.

### Stage 10: Folding Estimation Parity Surface

Deliverables:

- Source-map Oriedita folding stages to existing `treemaker-flatfold` behavior.
- Port missing Oriedita-specific folding stages where required.
- Add Oriedita-compatible folded snapshot structures.
- Add custom constraint support.
- Add two-color crease-pattern generation.
- Add another-overlap and duplicate-folded-model behavior.
- Status: the reusable `fold_graph` topology layer now backs both FOLD export
  and folding preparation. The first Oriedita `WireFrame_Worker` pass is ported
  as `folding::estimate_wireframe*`, including starting-face resolution,
  adjacent face-position propagation, and folded wireframe vertex coordinates,
  with direct oracle validation through `wireframe-folding-summary`.
- Status: `LineSegmentSetWorker.split_arrangement_for_SubFace_generation()` is
  ported as `folding::prepare_subface_segments`, covering the point removal,
  endpoint-tolerant duplicate removal, intersection division, and second cleanup
  pass that Oriedita runs before generating subdivided folded faces.
- Status: `FoldedFigure_Configurator.configureSubFaces()` is ported as
  `folding::configure_subfaces_from_segments`, covering subface-to-face
  membership, maximum face overlap count, and Oriedita's reduced subface set
  pruning. The first `HierarchyList_configure` step is ported as
  `folding::initial_hierarchy_from_segments`, covering mountain/valley-derived
  face-above-face relations and Oriedita's same-parity adjacent-face error.
  The 3-face and 4-face equivalence-condition discovery passes are ported as
  `folding::equivalence_condition_candidates_from_segments`; this covers
  condition discovery and pair normalization. A fixed-point port of
  `AdditionalEstimationAlgorithm` now derives the same final hierarchy relation
  table for covered fixtures as `folding::additional_estimation_from_segments`.
  The optimized Italiano/reactive data structures, remove-mode condition
  pruning as an observable API, custom constraints, overlap permutation search,
  full `FoldingEstimateTask`, full `TwoColoredTask`, and folded-model mutation
  commands remain intentionally unsupported until their stages are ported.
- Status: Oriedita's `PairGuide` and `ChainPermutationGenerator` are ported as
  `folding::ChainPermutationGenerator`, including persistent guides, temporary
  guides, top/bottom constraints, lock-chain initialization, reset, and
  next-permutation behavior. Oracle commands `chain-permutation-summary` and
  `chain-permutation-temp-summary` compare the Rust sequence against Oriedita
  before the overlap search is wired on top.
- Status: Oriedita's `SubFace.setGuideMap()` hierarchy-to-permutation-guide
  setup is ported as `folding::SubFacePermutationSearch`, including local face
  mapping, transitive reduction of hierarchy guides, retained local equivalence
  conditions, and initialized subface ordering sequences. The oracle command
  `subface-guide-permutation-summary` compares those sequences with Oriedita.
- Status: Oriedita's per-subface `possible_overlapping_search()` consistency
  loop is ported for hierarchy contradictions plus 3-face and 4-face
  penetration-condition checks, including temporary guide insertion and
  permutation advancement. The `CombinationGenerator` high-permutation
  accelerator remains an explicit typed unsupported path until its own stage is
  ported. The oracle command `subface-overlap-search-summary` compares the
  resulting status, permutation count, and top-to-bottom face order.
- Status: Oriedita's `SubFacePriority` pass is ported as
  `folding::prioritize_subfaces`, including empty-pair observer counts, priority
  selection by new information count, face-count tie-breaking, and valid
  subface count calculation. The oracle command `subface-priority-summary`
  compares ordered subface indices and valid counts.
- Status: Oriedita's worker-level overlap search is ported as
  `folding::possible_overlap_search_for_subfaces` and
  `folding::possible_overlap_search_for_subfaces_with_swap`, composing the
  priority pass, valid subface guide-map setup, per-subface consistency search,
  permutation carry/rollback, hierarchy stacking insertion, final AEA feedback
  into permutation search, the false-result last-hierarchy snapshot, realtime
  AEA checkpoints, subface swapping, swap counters, and temp-guide clearing
  during swap-over. Final recovery when AEA identifies an unsearched reduced
  subface is ported too: the missing subface is promoted into the valid set,
  given a guide map from the saved hierarchy, and the search continues. The
  Rust port uses the existing fixed-point/one-pass AEA implementation rather
  than Oriedita's optimized reactive Italiano data structures, so that
  difference is performance-oriented rather than a scoped behavior reduction.
  Oracle commands `worker-overlap-search-summary`,
  `worker-overlap-search-swap-summary`, and
  `worker-overlap-ordered-summary` compare the resulting status, hierarchy,
  swap-order state, and final-recovery behavior.
- Status: The same worker searches are exposed from folded line segments as
  `folding::overlap_search_from_segments` and
  `folding::overlap_search_from_segments_with_swap`, running the folded
  wireframe, subface arrangement, subface membership, hierarchy setup,
  equivalence discovery, priority, and worker search pipeline. Oracle commands
  `worker-overlap-from-segments-summary` and
  `worker-overlap-from-segments-swap-summary` compare this against actual
  Oriedita `FoldedFigure_Configurator.HierarchyList_configure` setups followed
  by `FoldedFigure_Worker.possible_overlapping_search(false|true)`.
- Status: The two-color folding preparation branch through
  `FoldedFigure.folding_estimated_02col()` and stage 03 is ported as
  `folding::two_colored_subface_segments_from_segments`. This keeps the
  development-view coordinates, uses face-position topology rather than folded
  point coordinates, and then runs the same subface-arrangement cleanup as
  Oriedita. The oracle command `two-colored-subface-arrangement` compares the
  resulting subdivided line set. Full `TwoColoredTask` command semantics,
  selected-range preconditions, and folded-figure state updates remain tracked
  until the command/service layer is ported.
- Status: The first-solution folded-figure estimate state machine is ported as
  `folding::folding_estimate_from_segments`, covering Oriedita estimation
  order normalization, stage/display transitions, text-result updates, and the
  first order-5 overlap solution from a fresh folded figure. The oracle command
  `folding-estimate-summary` compares the task-level step, display style,
  discovered-case count, text result, and worker hierarchy snapshot. Stateful
  `ORDER_6` "find another" enumeration and `FoldingEstimateSave100Task` batch
  export remain separate command-layer stages.
- Status: Oriedita's `SubFaceSwappingAlgorithm` support logic is ported as
  `folding::SubFaceSwapper`, including visited-subface tracking, dead-end
  recording, repeated-prefix history detection, swap-counter-driven reverse
  swaps, and `shouldEstimate` gating. The oracle command
  `subface-swapper-summary` compares the isolated ordering state machine with
  Oriedita; the worker-level search now also exercises it through the
  `possible_overlapping_search(true)` oracle path.

Oracle:

- Use Oriedita's folding test resources.
- Compare folded graph snapshots where deterministic.
- Compare validity, diagnostic, and partial-order invariants where exact layer
  order can vary.

Validation:

- `cargo test -p oristudio-cp --test folding`
- `ORIEDITA_GEOMETRY_ORACLE=... cargo test -p oristudio-cp --test oriedita_folding_oracle`
- `ORIEDITA_GEOMETRY_ORACLE=... cargo test -p oristudio-cp`
- `cargo test -p treemaker-flatfold`
- `ORIEDITA_ORACLE=... cargo test -p oracle-tests --test oriedita_oracle folding`

### Stage 11: WASM Boundary Without UI Button Integration

Deliverables:

- Expose stable command dispatch through `treemaker-wasm`.
- Expose import/export and diagnostics APIs.
- Expose operation status so the app can show unsupported operations later.
- Add WASM tests that call commands directly.

Out of scope:

- Adding every button to the React UI.
- Redesigning the CP tool palette.

Validation:

- `wasm-pack build crates/treemaker-wasm --target bundler`
- `wasm-pack test --node crates/treemaker-wasm`
- Targeted web typecheck only if TypeScript bindings change.

### Stage 12: External Corpus and Release Gate

Deliverables:

- Add external corpus harness for `.cp`, `.fold`, `.ori`, and `.orh`.
- Add summary reports for unsupported operations, oracle mismatches, and
  tolerated floating-point differences.
- Document known parity caveats.
- Make every unsupported item visible in generated status output.

Validation:

- Full Rust workspace tests.
- Full oracle tests with pinned Oriedita.
- External corpus run from a private path.
- Web/WASM checks if bindings are affected.

## Checklist

- [x] Stage 0: Source map and parity matrix.
- [x] Stage 1: Crate and error contract.
- [x] Stage 2: Oriedita geometry primitives.
- [x] Stage 3: Editable CP model and canonical comparison.
- [ ] Stage 4: Import and export parity.
- [ ] Stage 5: Arrangement, split, merge, and cleanup.
- [ ] Stage 6: Selection, color, and transform commands.
- [ ] Stage 7: Construction tools.
- [ ] Stage 8: Circle, polygon, base, and generator tools.
- [ ] Stage 9: Diagnostics and repair.
- [ ] Stage 10: Folding estimation parity surface.
- [ ] Stage 11: WASM boundary without UI button integration.
- [ ] Stage 12: External corpus and release gate.

## Acceptance Criteria

This plan is complete when:

- The source map accounts for every non-UI Oriedita crease-pattern operation.
- Every unported operation has an explicit unsupported status and typed error.
- Every ported operation has Rust unit tests.
- Every ported operation has Oriedita oracle validation or a documented reason
  why oracle comparison is not meaningful.
- `.cp`, `.fold`, `.ori`, and `.orh` behavior is semantically compatible with
  Oriedita for committed fixtures.
- The external corpus harness can report compatibility without committing
  private files.
- The WASM API can drive the full non-UI command surface, even if the React UI
  has not exposed each command yet.

## Open Questions

- Whether the pinned Oriedita source should be vendored under `third_party/` or
  fetched by an oracle setup script.
- Whether the `oristudio-cp` crate should publish independently later or remain
  a private workspace crate until the Oriedita parity surface is credible.
- Whether Oriedita folding estimation should be a compatibility layer over
  `treemaker-flatfold` or a separate direct port where behavior differs.
- Whether SVG/JPG/PNG rendering export belongs in the CP kernel or remains in
  app-specific rendering code.
- Which Oriedita UI preview behaviors need non-UI candidate APIs before actual
  button integration begins.
