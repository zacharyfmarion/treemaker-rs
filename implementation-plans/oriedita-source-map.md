# Oriedita Source Map and Parity Matrix

## Goal

This document is Stage 0 for the Oriedita port. It records the upstream source
surface that must be accounted for before implementation starts, maps each area
to a planned Rust home in `oristudio-cp`, and sets the initial parity status for
every item to `Unsupported`.

Nothing in this map means the feature is implemented. The point is the
opposite: every behavior is visible, named, and waiting for a unit-test and
oracle-tested port.

## Baseline

- Upstream repository: `https://github.com/oriedita/oriedita`
- Upstream commit inspected: `9d39135ae232cc03be4ffaf74baa7ae2df970507`
- Upstream license: MIT
- Planned Rust crate: `crates/oristudio-cp`
- Planned Rust package name: `oristudio-cp`

## Status Values

- `Unsupported`: command or behavior is known but not ported.
- `Porting`: implementation work has started but oracle parity is incomplete.
- `Unit-tested`: Rust unit coverage exists but Oriedita oracle coverage is not
  complete.
- `Oracle-tested`: Rust behavior matches the pinned Oriedita oracle for
  committed fixtures.
- `Documented-difference`: behavior intentionally differs and is documented.
- `Out-of-scope-ui`: Swing/UI-only behavior that does not belong in the initial
  non-UI kernel.

Default status for every item in this document is `Unsupported` unless a later
stage explicitly changes it.

## Rust Target Modules

| Target | Responsibility |
| --- | --- |
| `geometry` | Oriedita-compatible primitive geometry and epsilon behavior. |
| `model` | Editable crease-pattern document, lines, colors, selection, circles, text, and metadata. |
| `fold_graph` | Conversion between editable CP state and FOLD graph topology. |
| `io` | `.cp`, `.fold`, `.ori`, `.orh`, `.obj`, and `.dxf` import/export. |
| `operations::arrangement` | Add/delete/split/merge/cleanup line arrangement behavior. |
| `operations::selection` | Select/unselect, lasso, polygon, connected, type, and intersecting-line selection. |
| `operations::color` | Crease type, line color, assignment, replacement, and deletion commands. |
| `operations::construction` | Draw, bisect, inward, perpendicular, symmetric, parallel, foldable-line, and axiom commands. |
| `operations::point` | Point creation, deletion, vertex deletion, and line division commands. |
| `operations::transform` | Move, copy, lengthen, four-point transforms, and operation-frame semantics. |
| `operations::circle` | Circle creation, tangency, intersection, concentric, inversion, and color commands. |
| `operations::generators` | Regular polygon, base pattern, and Voronoi generators. |
| `operations::measure` | Non-mutating distance and angle measurement command results. |
| `checks` | Check1-Check4, CAMV, flat-foldability, and repair diagnostics. |
| `folding` | Oriedita-compatible folding-estimation surface and folded-model data. |
| `oracle` | Test-only canonicalization and oracle result comparison helpers. |

## Upstream Package Map

| Upstream path | Role | Rust target | Classification | Stage | Status |
| --- | --- | --- | --- | --- | --- |
| `origami/Epsilon.java` | Global tolerance constants. | `geometry::epsilon` | kernel | 2 | Unit-tested |
| `origami/crease_pattern/OritaCalc.java` | Core static geometry helpers. | `geometry::orita_calc` | kernel | 2 | Porting |
| `origami/crease_pattern/element/Point.java` | 2D point primitive. | `geometry::point` | kernel | 2 | Unit-tested |
| `origami/crease_pattern/element/Line.java` | PointSet line primitive. | `geometry::line` | kernel | 2 | Unit-tested |
| `origami/crease_pattern/element/StraightLine.java` | Infinite line primitive. | `geometry::straight_line` | kernel | 2 | Unit-tested |
| `origami/crease_pattern/element/LineSegment.java` | Segment primitive, colors, active state, intersection enum. | `geometry::segment`, `model::line` | kernel | 2-3 | Oracle-tested |
| `origami/crease_pattern/element/LineColor.java` | Oriedita line colors and fold-color meanings. | `model::line_color` | kernel | 3 | Unit-tested |
| `origami/crease_pattern/element/Circle.java` | Circle primitive and color metadata. | `geometry::circle`, `model::circle` | kernel | 2-3 | Unit-tested |
| `origami/crease_pattern/element/Polygon.java` | Polygon containment and selection geometry. | `geometry::polygon` | kernel | 2 | Unit-tested |
| `origami/crease_pattern/element/Rectangle.java` | Rectangular selection and bounds. | `geometry::rectangle` | kernel | 2 | Unit-tested |
| `origami/crease_pattern/CustomLineTypes.java` | Custom line metadata. | `model::line_color` | kernel | 3 | Unsupported |
| `origami/crease_pattern/FoldLineSet.java` | Main editable line/circle set. | `model`, `operations::*` | kernel | 3-9 | Unsupported |
| `origami/crease_pattern/LineSegmentSet.java` | Line arrangement set used for folding and export. | `fold_graph`, `operations::arrangement` | kernel | 3-5 | Unsupported |
| `origami/crease_pattern/PointSet.java` | Vertex/edge/face topology for folding/export. | `fold_graph` | kernel | 3-4 | Unsupported |
| `origami/crease_pattern/PointLineMap.java` | Point-to-line neighborhood lookup. | `model::topology` | kernel | 3 | Unsupported |
| `origami/crease_pattern/FlatFoldabilityViolation.java` | Diagnostic payload. | `checks::diagnostic` | kernel | 9 | Unsupported |
| `origami/crease_pattern/LittleBigLittleViolation.java` | Diagnostic payload. | `checks::diagnostic` | kernel | 9 | Unsupported |
| `origami/crease_pattern/LassoInteractionMode.java` | Lasso selection mode. | `operations::selection` | kernel | 6 | Unsupported |
| `origami/crease_pattern/FoldingException.java` | Folding error surface. | `folding::error` | kernel | 10 | Unsupported |
| `origami/crease_pattern/util/CreasePattern_Worker_Toolbox.java` | Shared CP worker helpers. | `operations::*` | kernel | 5-9 | Unsupported |
| `origami/crease_pattern/worker/WireFrame_Worker.java` | Face/topology/folding preparation. | `fold_graph`, `folding` | kernel | 4, 10 | Unsupported |
| `origami/crease_pattern/worker/LineSegmentSetWorker.java` | Arrangement cleanup for folded subfaces. | `folding`, `operations::arrangement` | kernel | 5, 10 | Unsupported |
| `origami/crease_pattern/worker/FoldedFigure_Worker.java` | Folded-model hierarchy and overlap solving. | `folding` | kernel | 10 | Unsupported |
| `origami/crease_pattern/worker/FoldedFigure_Configurator.java` | Subface and hierarchy setup. | `folding` | kernel | 10 | Unsupported |
| `origami/crease_pattern/worker/SelectMode.java` | Select/unselect mode enum. | `operations::selection` | kernel | 6 | Unsupported |
| `origami/crease_pattern/worker/foldlineset/BranchTrim.java` | Branch trimming cleanup. | `operations::arrangement` | kernel | 5 | Unsupported |
| `origami/crease_pattern/worker/foldlineset/Check1.java` | Diagnostic check. | `checks` | kernel | 9 | Unsupported |
| `origami/crease_pattern/worker/foldlineset/Check2.java` | Diagnostic check. | `checks` | kernel | 9 | Unsupported |
| `origami/crease_pattern/worker/foldlineset/Check3.java` | Diagnostic check. | `checks` | kernel | 9 | Unsupported |
| `origami/crease_pattern/worker/foldlineset/Check4.java` | Diagnostic check. | `checks` | kernel | 9 | Unsupported |
| `origami/crease_pattern/worker/foldlineset/Fix1.java` | Repair operation. | `checks`, `operations::arrangement` | kernel | 9 | Unsupported |
| `origami/crease_pattern/worker/foldlineset/Fix2.java` | Repair operation. | `checks`, `operations::arrangement` | kernel | 9 | Unsupported |
| `origami/crease_pattern/worker/foldlineset/OrganizeCircles.java` | Circle cleanup/organization. | `operations::circle` | kernel | 8 | Unsupported |
| `origami/crease_pattern/worker/linesegmentset/GetBoundingBox.java` | Bounds helper. | `model::bounds` | kernel | 3 | Unsupported |
| `origami/crease_pattern/worker/linesegmentset/IntersectDivide.java` | Segment arrangement split helper. | `operations::arrangement` | kernel | 5 | Unsupported |
| `origami/crease_pattern/worker/linesegmentset/OverlappingLineRemoval.java` | Overlap cleanup helper. | `operations::arrangement` | kernel | 5 | Unsupported |
| `origami/data/quadTree/*` | Spatial acceleration and collectors. | `model::spatial` | kernel | 3, 5 | Unsupported |
| `origami/data/listMatrix/*` | Matrix-like adjacency storage. | `folding`, `fold_graph` | kernel | 10 | Unsupported |
| `origami/data/symmetricMatrix/*` | Symmetric relation storage. | `folding` | kernel | 10 | Unsupported |
| `origami/data/tree/*` | AVL/BST utilities. | `folding` | kernel | 10 | Unsupported |
| `origami/data/save/LineSegmentSave.java` | Save DTO for lines/circles. | `io::save` | io | 4 | Unsupported |
| `origami/data/save/PointSave.java` | Save DTO for points. | `io::save` | io | 4 | Unsupported |
| `origami/folding/FoldedFigure.java` | Folding-stage coordinator. | `folding` | kernel | 10 | Unsupported |
| `origami/folding/HierarchyList.java` | Face-order relation table. | `folding::hierarchy` | kernel | 10 | Unsupported |
| `origami/folding/element/Face.java` | Folded face data. | `folding::face` | kernel | 10 | Unsupported |
| `origami/folding/element/SubFace.java` | Subface data and overlap relations. | `folding::subface` | kernel | 10 | Unsupported |
| `origami/folding/constraint/CustomConstraint.java` | User folding constraints. | `folding::constraints` | kernel | 10 | Unsupported |
| `origami/folding/algorithm/*` | Inference, priority, Italiano, swapping algorithms. | `folding::solver` | kernel | 10 | Unsupported |
| `origami/folding/permutation/*` | Permutation and constraint combinatorics. | `folding::solver` | kernel | 10 | Unsupported |
| `origami/folding/util/*` | Folding utility data structures. | `folding::util` | kernel | 10 | Unsupported |
| `oriedita-data/export/*` | Import/export implementations. | `io` | io | 4 | Unsupported |
| `oriedita-data/save/*` | Oriedita save models and version conversion. | `io::save` | io | 4 | Unsupported |
| `oriedita/src/main/java/oriedita/editor/task/*` | Non-UI task semantics. | `checks`, `folding` | kernel | 9-10 | Unsupported |
| `oriedita/src/main/java/oriedita/editor/service/impl/FoldingServiceImpl.java` | Folding command routing. | `folding::commands` | kernel | 10 | Unsupported |
| `oriedita/src/main/java/oriedita/editor/handler/*` | Mouse/tool command intent. | `operations::*`, `folding` | kernel-intent | 5-10 | Unsupported |

## Mouse Mode Command Matrix

| Mouse mode | Upstream handler | Rust target | Classification | Stage | Status |
| --- | --- | --- | --- | --- | --- |
| `DRAW_CREASE_FREE_1` | `MouseHandlerDrawCreaseFree` | `operations::construction::draw_free` | kernel | 7 | Unsupported |
| `MOVE_CREASE_PATTERN_2` | `MouseHandlerMoveCreasePattern` | `operations::transform::move_document` | kernel | 6 | Unsupported |
| `LINE_SEGMENT_DELETE_3` | `MouseHandlerLineSegmentDelete` | `operations::arrangement::delete_line` | kernel | 5 | Unsupported |
| `CHANGE_CREASE_TYPE_4` | `MouseHandlerChangeCreaseType` | `operations::color::change_type` | kernel | 6 | Unsupported |
| `LENGTHEN_CREASE_5` | `MouseHandlerLengthenCrease` | `operations::transform::lengthen` | kernel | 6 | Unsupported |
| `SQUARE_BISECTOR_7` | `MouseHandlerSquareBisector` | `operations::construction::square_bisector` | kernel | 7 | Unsupported |
| `INWARD_8` | `MouseHandlerInward` | `operations::construction::inward` | kernel | 7 | Unsupported |
| `PERPENDICULAR_DRAW_9` | `MouseHandlerPerpendicularDraw` | `operations::construction::perpendicular` | kernel | 7 | Unsupported |
| `SYMMETRIC_DRAW_10` | `MouseHandlerSymmetricDraw` | `operations::construction::symmetric` | kernel | 7 | Unsupported |
| `DRAW_CREASE_RESTRICTED_11` | `MouseHandlerDrawCreaseRestricted` | `operations::construction::draw_restricted` | kernel | 7 | Unsupported |
| `DRAW_CREASE_SYMMETRIC_12` | `MouseHandlerDrawCreaseSymmetric` | `operations::construction::draw_symmetric` | kernel | 7 | Unsupported |
| `DRAW_CREASE_ANGLE_RESTRICTED_13` | `MouseHandlerDrawCreaseAngleRestricted` | `operations::construction::angle_restricted` | kernel | 7 | Unsupported |
| `DRAW_POINT_14` | `MouseHandlerDrawPoint` | `operations::point::draw_point` | kernel | 7 | Unsupported |
| `DELETE_POINT_15` | `MouseHandlerDeletePoint` | `operations::point::delete_point` | kernel | 5 | Unsupported |
| `ANGLE_SYSTEM_16` | `MouseHandlerAngleSystem` | `operations::construction::angle_system` | kernel | 7 | Unsupported |
| `DRAW_CREASE_ANGLE_RESTRICTED_3_18` | `MouseHandlerDrawCreaseAngleRestricted3_2` | `operations::construction::angle_restricted_3` | kernel | 7 | Unsupported |
| `CREASE_SELECT_19` | `MouseHandlerCreaseSelect` | `operations::selection::select_line` | kernel | 6 | Unsupported |
| `CREASE_UNSELECT_20` | `MouseHandlerCreaseUnselect` | `operations::selection::unselect_line` | kernel | 6 | Unsupported |
| `CREASE_MOVE_21` | `MouseHandlerCreaseMove` | `operations::transform::move_selection` | kernel | 6 | Unsupported |
| `CREASE_COPY_22` | `MouseHandlerCreaseCopy` | `operations::transform::copy_selection` | kernel | 6 | Unsupported |
| `CREASE_MAKE_MOUNTAIN_23` | `MouseHandlerCreaseMakeMountain` | `operations::color::make_mountain` | kernel | 6 | Unsupported |
| `CREASE_MAKE_VALLEY_24` | `MouseHandlerCreaseMakeValley` | `operations::color::make_valley` | kernel | 6 | Unsupported |
| `CREASE_MAKE_EDGE_25` | `MouseHandlerCreaseMakeEdge` | `operations::color::make_edge` | kernel | 6 | Unsupported |
| `BACKGROUND_CHANGE_POSITION_26` | `MouseHandlerBackgroundChangePosition` | none | ui-preview-only | later UI | Out-of-scope-ui |
| `LINE_SEGMENT_DIVISION_27` | `MouseHandlerLineSegmentDivision` | `operations::point::divide_segment` | kernel | 7 | Unsupported |
| `LINE_SEGMENT_RATIO_SET_28` | `MouseHandlerLineSegmentRatioSet` | `operations::point::ratio_divide_segment` | kernel | 7 | Unsupported |
| `POLYGON_SET_NO_CORNERS_29` | `MouseHandlerPolygonSetNoCorners` | `operations::generators::regular_polygon` | kernel | 8 | Unsupported |
| `CREASE_ADVANCE_TYPE_30` | `MouseHandlerCreaseAdvanceType` | `operations::color::advance_type` | kernel | 6 | Unsupported |
| `CREASE_MOVE_4P_31` | `MouseHandlerCreaseMove4p` | `operations::transform::move_4p` | kernel | 6 | Unsupported |
| `CREASE_COPY_4P_32` | `MouseHandlerCreaseCopy4p` | `operations::transform::copy_4p` | kernel | 6 | Unsupported |
| `FISH_BONE_DRAW_33` | `MouseHandlerFishBoneDraw` | `operations::construction::fishbone` | kernel | 7 | Unsupported |
| `CREASE_MAKE_MV_34` | `MouseHandlerCreaseMakeMV` | `operations::color::make_mountain_valley` | kernel | 6 | Unsupported |
| `DOUBLE_SYMMETRIC_DRAW_35` | `MouseHandlerDoubleSymmetricDraw` | `operations::construction::double_symmetric` | kernel | 7 | Unsupported |
| `CREASES_ALTERNATE_MV_36` | `MouseHandlerCreasesAlternateMV` | `operations::color::alternate_mountain_valley` | kernel | 6 | Unsupported |
| `DRAW_CREASE_ANGLE_RESTRICTED_5_37` | `MouseHandlerDrawCreaseAngleRestricted5` | `operations::construction::angle_restricted_5` | kernel | 7 | Unsupported |
| `VERTEX_MAKE_ANGULARLY_FLAT_FOLDABLE_38` | `MouseHandlerVertexMakeAngularlyFlatFoldable` | `operations::construction::make_vertex_flat_foldable` | kernel | 7, 9 | Unsupported |
| `FOLDABLE_LINE_INPUT_39` | `MouseHandlerFoldableLineInput` | `operations::construction::foldable_line_input` | kernel | 7 | Unsupported |
| `PARALLEL_DRAW_40` | `MouseHandlerParallelDraw` | `operations::construction::parallel` | kernel | 7 | Unsupported |
| `VERTEX_DELETE_ON_CREASE_41` | `MouseHandlerVertexDeleteOnCrease` | `operations::point::delete_vertex_on_crease` | kernel | 5 | Unsupported |
| `CIRCLE_DRAW_42` | `MouseHandlerCircleDraw` | `operations::circle::draw` | kernel | 8 | Unsupported |
| `CIRCLE_DRAW_THREE_POINT_43` | `MouseHandlerCircleDrawThreePoint` | `operations::circle::through_three_points` | kernel | 8 | Unsupported |
| `CIRCLE_DRAW_SEPARATE_44` | `MouseHandlerCircleDrawSeparate` | `operations::circle::separate` | kernel | 8 | Unsupported |
| `CIRCLE_DRAW_TANGENT_LINE_45` | `MouseHandlerCircleDrawTangentLine` | `operations::circle::tangent_line` | kernel | 8 | Unsupported |
| `CIRCLE_DRAW_INVERTED_46` | `MouseHandlerCircleDrawInverted` | `operations::circle::inverted` | kernel | 8 | Unsupported |
| `CIRCLE_DRAW_FREE_47` | `MouseHandlerCircleDrawFree` | `operations::circle::free` | kernel | 8 | Unsupported |
| `CIRCLE_DRAW_CONCENTRIC_48` | `MouseHandlerCircleDrawConcentric` | `operations::circle::concentric` | kernel | 8 | Unsupported |
| `CIRCLE_DRAW_CONCENTRIC_SELECT_49` | `MouseHandlerCircleDrawConcentricSelect` | `operations::circle::concentric_select` | kernel | 8 | Unsupported |
| `CIRCLE_DRAW_CONCENTRIC_TWO_CIRCLE_SELECT_50` | `MouseHandlerCircleDrawConcentricTwoCircleSelect` | `operations::circle::concentric_two_circle_select` | kernel | 8 | Unsupported |
| `PARALLEL_DRAW_WIDTH_51` | `MouseHandlerParallelDrawWidth` | `operations::construction::parallel_width` | kernel | 7 | Unsupported |
| `CONTINUOUS_SYMMETRIC_DRAW_52` | `MouseHandlerContinuousSymmetricDraw` | `operations::construction::continuous_symmetric` | kernel | 7 | Unsupported |
| `DISPLAY_LENGTH_BETWEEN_POINTS_1_53` | `MouseHandlerDisplayLengthBetweenPoints` variant | `operations::measure::length_between_points` | kernel | 7 | Unsupported |
| `DISPLAY_LENGTH_BETWEEN_POINTS_2_54` | `MouseHandlerDisplayLengthBetweenPoints` variant | `operations::measure::length_between_points` | kernel | 7 | Unsupported |
| `DISPLAY_ANGLE_BETWEEN_THREE_POINTS_1_55` | `MouseHandlerDisplayAngleBetweenThreePoints` variant | `operations::measure::angle_between_points` | kernel | 7 | Unsupported |
| `DISPLAY_ANGLE_BETWEEN_THREE_POINTS_2_56` | `MouseHandlerDisplayAngleBetweenThreePoints` variant | `operations::measure::angle_between_points` | kernel | 7 | Unsupported |
| `DISPLAY_ANGLE_BETWEEN_THREE_POINTS_3_57` | `MouseHandlerDisplayAngleBetweenThreePoints` variant | `operations::measure::angle_between_points` | kernel | 7 | Unsupported |
| `CREASE_TOGGLE_MV_58` | `MouseHandlerCreaseToggleMV` | `operations::color::toggle_mountain_valley` | kernel | 6 | Unsupported |
| `CIRCLE_CHANGE_COLOR_59` | `MouseHandlerCircleChangeColor` | `operations::circle::change_color` | kernel | 8 | Unsupported |
| `CREASE_MAKE_AUX_60` | `MouseHandlerCreaseMakeAux` | `operations::color::make_aux` | kernel | 6 | Unsupported |
| `OPERATION_FRAME_CREATE_61` | `MouseHandlerOperationFrameCreate` | `operations::transform::operation_frame` | kernel | 6 | Unsupported |
| `VORONOI_CREATE_62` | `MouseHandlerVoronoiCreate` | `operations::generators::voronoi` | kernel | 8 | Unsupported |
| `FLAT_FOLDABLE_CHECK_63` | `MouseHandlerFlatFoldableCheck` | `checks::flat_foldable` | kernel | 9 | Unsupported |
| `CREASE_DELETE_OVERLAPPING_64` | `MouseHandlerCreaseDeleteOverlapping` | `operations::arrangement::delete_overlapping` | kernel | 5 | Unsupported |
| `CREASE_DELETE_INTERSECTING_65` | `MouseHandlerCreaseDeleteIntersecting` | `operations::arrangement::delete_intersecting` | kernel | 5 | Unsupported |
| `SELECT_POLYGON_66` | `MouseHandlerSelectPolygon` | `operations::selection::select_polygon` | kernel | 6 | Unsupported |
| `UNSELECT_POLYGON_67` | `MouseHandlerUnselectPolygon` | `operations::selection::unselect_polygon` | kernel | 6 | Unsupported |
| `SELECT_LINE_INTERSECTING_68` | `MouseHandlerSelectLineIntersecting` | `operations::selection::select_intersecting_line` | kernel | 6 | Unsupported |
| `UNSELECT_LINE_INTERSECTING_69` | `MouseHandlerUnselectLineIntersecting` | `operations::selection::unselect_intersecting_line` | kernel | 6 | Unsupported |
| `LENGTHEN_CREASE_SAME_COLOR_70` | `MouseHandlerLengthenCreaseSameColor` | `operations::transform::lengthen_same_color` | kernel | 6 | Unsupported |
| `FOLDABLE_LINE_DRAW_71` | `MouseHandlerFoldableLineDraw` | `operations::construction::foldable_line_draw` | kernel | 7 | Unsupported |
| `REPLACE_LINE_TYPE_SELECT_72` | `MouseHandlerReplaceTypeSelect` | `operations::color::replace_selected_type` | kernel | 6 | Unsupported |
| `DELETE_LINE_TYPE_SELECT_73` | `MouseHandlerDeleteTypeSelect` | `operations::color::delete_selected_type` | kernel | 6 | Unsupported |
| `SELECT_LASSO_74` | `MouseHandlerSelectLasso` | `operations::selection::select_lasso` | kernel | 6 | Unsupported |
| `UNSELECT_LASSO_75` | `MouseHandlerUnselectLasso` | `operations::selection::unselect_lasso` | kernel | 6 | Unsupported |
| `TEXT` | `MouseHandlerText` | `model::text`, `operations::text` | kernel | 3, 4 | Unsupported |
| `DRAW_BLINTZ` | `MouseHandlerDrawBlintz` | `operations::generators::blintz` | kernel | 8 | Unsupported |
| `DRAW_FISH_BASE` | `MouseHandlerDrawFishBase` | `operations::generators::fish_base` | kernel | 8 | Unsupported |
| `DRAW_DOVE_BASE` | `MouseHandlerDrawDoveBase` | `operations::generators::dove_base` | kernel | 8 | Unsupported |
| `DRAW_BIRD_BASE` | `MouseHandlerDrawBirdBase` | `operations::generators::bird_base` | kernel | 8 | Unsupported |
| `DRAW_FROG_BASE` | `MouseHandlerDrawFrogBase` | `operations::generators::frog_base` | kernel | 8 | Unsupported |
| `MODIFY_CALCULATED_SHAPE_101` | `MouseHandlerModifyCalculatedShape` | `folding::modify_calculated_shape` | kernel | 10 | Unsupported |
| `MOVE_CALCULATED_SHAPE_102` | `MouseHandlerMoveCalculatedShape` | `folding::move_calculated_shape` | kernel | 10 | Unsupported |
| `CHANGE_STANDARD_FACE_103` | `MouseHandlerChangeStandardFace` | `folding::change_standard_face` | kernel | 10 | Unsupported |
| `ADD_FOLDING_CONSTRAINT` | `MouseHandlerAddFoldingConstraints` | `folding::constraints` | kernel | 10 | Unsupported |
| `AXIOM_5` | `MouseHandlerAxiom5` | `operations::construction::axiom5` | kernel | 7 | Unsupported |
| `AXIOM_7` | `MouseHandlerAxiom7` | `operations::construction::axiom7` | kernel | 7 | Unsupported |
| `FIX_INACCURATE_107` | `MouseHandlerCreaseFixInaccurate` | `checks::fix_inaccurate`, `operations::arrangement` | kernel | 5, 9 | Unsupported |

## Step Handler Infrastructure

| Upstream source | Role | Rust target | Classification | Stage | Status |
| --- | --- | --- | --- | --- | --- |
| `StepMouseHandler.java` | Multi-step command lifecycle. | `operations::command_state` | kernel | 1 | Unsupported |
| `StepGraph.java` | Step transition wiring. | `operations::command_state` | kernel | 1 | Unsupported |
| `StepFactory.java` | Reusable point/box/line-selection steps. | `operations::command_state` | kernel | 1, 6-8 | Unsupported |
| `ObjCoordStepNode.java` | Model-space point input step. | `operations::command_state` | kernel | 1 | Unsupported |
| `SelectPointStepNode.java` | Point selection step. | `operations::command_state` | kernel | 1, 6-8 | Unsupported |
| `BoxSelectStepNode.java` | Box selection step. | `operations::selection` | kernel | 6 | Unsupported |
| `BoxSelectLinesStepNode.java` | Box line selection step. | `operations::selection` | kernel | 6 | Unsupported |
| `IPreviewStepNode.java` | Preview marker. | command result candidates | kernel-preview | 7-8 | Unsupported |
| `ICameraStepNode.java` | Camera/model coordinate marker. | none | out-of-scope-ui | later UI | Out-of-scope-ui |

The Rust kernel should not copy Swing event lifecycles, but it must preserve
the command-state semantics that affect generated candidates and final model
mutation.

## Import and Export Matrix

| Upstream source | Behavior | Rust target | Stage | Status |
| --- | --- | --- | --- | --- |
| `CpImporter.java` | `.cp` import. | `io::cp::import` | 4 | Unsupported |
| `CpExporter.java` | `.cp` export and lossy-format warning. | `io::cp::export` | 4 | Unsupported |
| `FoldImporter.java` | `.fold` import and coordinate normalization. | `io::fold::import` | 4 | Unsupported |
| `FoldExporter.java` | `.fold` export, face reconstruction, Oriedita extras. | `io::fold::export` | 4 | Unsupported |
| `OriImporter.java` | `.ori` import. | `io::ori::import` | 4 | Unsupported |
| `OriExporter.java` | `.ori` export. | `io::ori::export` | 4 | Unsupported |
| `OrhImporter.java` | `.orh` import. | `io::orh::import` | 4 | Unsupported |
| `OrhExporter.java` | `.orh` export. | `io::orh::export` | 4 | Unsupported |
| `ObjImporter.java` | `.obj` import. | `io::obj::import` | 4 | Unsupported |
| `DxfExporter.java` | `.dxf` export. | `io::dxf::export` | 4 | Unsupported |
| `OrieditaFoldFile.java` | FOLD extension fields. | `io::fold::oriedita_extensions` | 4 | Unsupported |
| `Save.java` | Main save model. | `io::save` | 4 | Unsupported |
| `BaseSave.java` | Shared save payload. | `io::save` | 4 | Unsupported |
| `SaveV1_0.java` | Legacy save payload. | `io::save::legacy` | 4 | Unsupported |
| `SaveV1_1.java` | Legacy save payload. | `io::save::legacy` | 4 | Unsupported |
| `SaveConverter.java` | Save-version conversion. | `io::save::convert` | 4 | Unsupported |
| `SaveProvider.java` | Save instance factory. | `io::save` | 4 | Unsupported |
| `FileVersionTester.java` | Save-version detection. | `io::save::version` | 4 | Unsupported |
| `TextSave.java` | Text persistence. | `model::text`, `io::save` | 3-4 | Unsupported |

## Task and Service Matrix

| Upstream source | Behavior | Rust target | Stage | Status |
| --- | --- | --- | --- | --- |
| `CheckCAMVTask.java` | Combined angle and MV diagnostics. | `checks::camv` | 9 | Unsupported |
| `FoldingEstimateTask.java` | Folding estimate execution. | `folding::estimate` | 10 | Unsupported |
| `FoldingEstimateSpecificTask.java` | Refold with selected starting face/state. | `folding::estimate_specific` | 10 | Unsupported |
| `FoldingEstimateSave100Task.java` | Batch/export folding estimates. | `folding::estimate_batch` | 10 | Unsupported |
| `TwoColoredTask.java` | Two-colored CP generation. | `folding::two_colored` | 10 | Unsupported |
| `FoldingServiceImpl.fold` | Determine fold scope and start folding. | `folding::commands::fold` | 10 | Unsupported |
| `FoldingServiceImpl.folding_estimated` | Reuse existing fold input for selected figure. | `folding::commands::estimate` | 10 | Unsupported |
| `FoldingServiceImpl.createTwoColoredCp` | Generate two-colored CP from selected lines. | `folding::commands::two_colored` | 10 | Unsupported |
| `FoldingServiceImpl.foldAnother` | Request another overlap solution. | `folding::commands::fold_another` | 10 | Unsupported |
| `FoldingServiceImpl.duplicate` | Duplicate folded model and replay estimate order. | `folding::commands::duplicate` | 10 | Unsupported |

## Diagnostic Matrix

| Upstream source | Behavior | Rust target | Stage | Status |
| --- | --- | --- | --- | --- |
| `Check1.java` | Check1 fold-line diagnostic. | `checks::check1` | 9 | Unsupported |
| `Check2.java` | Check2 fold-line diagnostic. | `checks::check2` | 9 | Unsupported |
| `Check3.java` | Check3 angle/Fushimi diagnostic. | `checks::check3` | 9 | Unsupported |
| `Check4.java` | Check4 CAMV/little-big-little diagnostic. | `checks::check4` | 9 | Unsupported |
| `FlatFoldabilityViolation.java` | Flat-foldability violation payload. | `checks::diagnostic` | 9 | Unsupported |
| `LittleBigLittleViolation.java` | Little-big-little violation payload. | `checks::diagnostic` | 9 | Unsupported |
| `MouseHandlerFlatFoldableCheck.java` | User-triggered flat-foldable check. | `checks::flat_foldable` | 9 | Unsupported |
| `MouseHandlerCreaseFixInaccurate.java` | Inaccurate line repair command. | `checks::fix_inaccurate` | 9 | Unsupported |
| `Fix1.java` | Repair helper. | `checks::fix1` | 9 | Unsupported |
| `Fix2.java` | Repair helper. | `checks::fix2` | 9 | Unsupported |

## Oracle Fixture Families

The oracle harness should grow in the same order as the stages:

| Fixture family | Purpose | First stage |
| --- | --- | --- |
| `geometry-primitives` | Distance, angle, projection, intersection, circle helpers. | 2 |
| `model-roundtrip` | Fold lines, aux lines, selection, circles, text, colors. | 3 |
| `io-roundtrip` | `.cp`, `.fold`, `.ori`, `.orh`, `.obj`, `.dxf`. | 4 |
| `arrangement` | Add, split, overlap, delete, branch trim, fix inaccurate. | 5 |
| `selection` | Box, polygon, lasso, connected, line-intersection selection. | 6 |
| `color-transform` | Type changes, MV toggles, move/copy/lengthen transforms. | 6 |
| `construction` | Draw/restricted/symmetric/parallel/axiom/foldable commands. | 7 |
| `circle-generator` | Circle modes, polygon/base generators, Voronoi. | 8 |
| `checks` | Check1-Check4, CAMV, flat-foldability, repair diagnostics. | 9 |
| `folding` | Folding stages, subfaces, hierarchy, constraints, overlap search. | 10 |

## Stage 0 Completion Notes

This map intentionally over-includes non-UI behavior. Later stages may split a
row into several smaller operations, but they should not delete rows simply
because a behavior is difficult to port.

Before Stage 1 begins:

- Keep `Unsupported` as the default command result for every listed operation.
- Add any missed Oriedita class or command to this file before implementing it.
- If an upstream behavior turns out to be Swing-only, mark it
  `Out-of-scope-ui` with a reason rather than silently dropping it.
- If `treemaker-flatfold` can satisfy an Oriedita folding behavior, keep the row
  and change the future implementation note to a compatibility mapping.
