//! Oriedita-compatible crease-pattern editing kernel for Ori Studio.
//!
//! This crate is intentionally conservative while the port is in progress.
//! Every known non-UI Oriedita operation is represented in the registry, but
//! unsupported operations fail with a typed error instead of fabricating nearby
//! behavior.

pub mod canonical;
pub mod checks;
mod fold_graph;
pub mod folding;
pub mod geometry;
pub mod io;
pub mod model;
pub mod operations;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use canonical::CanonicalCreasePattern;
use geometry::{Epsilon, LineColor, LineSegment, Point, Polygon, determine_line_segment_distance};
pub use model::CreasePatternModel;

const DEFAULT_SELECTION_DISTANCE: f64 = 1.0;
const ORIEDITA_PAPER_SIZE: f64 = 400.0;
const DEFAULT_ANGLE_SYSTEM_DIVIDER: i32 = 4;
const DEFAULT_ANGLE_SYSTEM_ANGLES: [f64; 6] = [40.0, 60.0, 80.0, 30.0, 50.0, 100.0];
const DEFAULT_LINE_DIVISION_COUNT: usize = 2;
const DEFAULT_LINE_RATIO: f64 = 1.0;

/// Crate-local result type.
pub type Result<T> = std::result::Result<T, CommandError>;

/// Editable crease-pattern document state.
///
/// Stage 1 only defines the carrier type needed by the command contract.
/// Geometry, lines, circles, text, and Oriedita metadata are added by later
/// stages under the same type.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreasePatternDocument {
    /// Optional user-visible document title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Editable Oriedita-compatible crease-pattern model state.
    #[serde(default)]
    pub crease_pattern: CreasePatternModel,
    /// Transient Oriedita operation-frame state used by frame selection tools.
    #[serde(default)]
    pub operation_frame: operations::transform::OperationFrame,
    /// Namespaced metadata preserved before full model support lands.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

impl CreasePatternDocument {
    /// Return a canonical semantic view suitable for parity comparisons.
    pub fn canonical(&self, tolerance: f64) -> CanonicalCreasePattern {
        CanonicalCreasePattern::from_document(self, tolerance)
    }
}

/// A command request against a crease-pattern document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreasePatternCommand {
    /// Oriedita operation represented by this command.
    pub operation: OperationId,
    /// Resolved model-space inputs for the operation.
    #[serde(default)]
    pub payload: CreasePatternCommandPayload,
}

impl CreasePatternCommand {
    /// Create a command for an Oriedita operation.
    pub fn new(operation: OperationId) -> Self {
        Self {
            operation,
            payload: CreasePatternCommandPayload::default(),
        }
    }

    /// Attach resolved model-space inputs.
    pub fn with_payload(mut self, payload: CreasePatternCommandPayload) -> Self {
        self.payload = payload;
        self
    }
}

/// Resolved command inputs supplied by the UI after hit testing and selection.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreasePatternCommandPayload {
    /// One-based Oriedita line IDs resolved by the UI.
    #[serde(default)]
    pub line_ids: Vec<usize>,
    /// Resolved model-space points, in the same order as the active tool steps.
    #[serde(default)]
    pub points: Vec<geometry::Point>,
    /// Optional active Oriedita line color for commands that use the current color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_color: Option<geometry::LineColor>,
    /// Optional model-space hit tolerance for point/line tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_distance: Option<f64>,
    /// Optional active grid width for grid-spaced construction tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_width: Option<f64>,
    /// Optional active angle-system divider. Oriedita's default divider is 4.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angle_system_divider: Option<i32>,
    /// Optional custom angle-system values used when the divider is zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angles: Option<[f64; 6]>,
    /// Optional zero-based construction candidate selected by the UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_index: Option<usize>,
    /// Optional division count for line division tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub division_count: Option<usize>,
    /// Optional first ratio value for ratio division tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ratio_s: Option<f64>,
    /// Optional second ratio value for ratio division tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ratio_t: Option<f64>,
    /// Optional model-space width for parallel-width construction tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    /// Optional source custom line type for replace-type commands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_from_line_type: Option<model::CustomLineType>,
    /// Optional destination custom line type for replace-type commands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_to_line_type: Option<model::CustomLineType>,
    /// Optional custom line type for delete-type commands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_line_type: Option<model::CustomLineType>,
}

/// Result returned by a successfully executed command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResult {
    /// Oriedita operation that was executed.
    pub operation: OperationId,
    /// Implementation status after execution.
    pub status: OperationStatus,
    /// Human-readable diagnostics emitted by the command.
    pub diagnostics: Vec<String>,
}

/// Transient candidate geometry for active construction tools.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommandPreview {
    /// Candidate, guide, or would-be committed line segments.
    pub segments: Vec<geometry::LineSegment>,
    /// Candidate commit points, such as angle-restricted convergence points.
    pub points: Vec<geometry::Point>,
    /// Human-readable diagnostics emitted by the preview query.
    pub diagnostics: Vec<String>,
}

/// Error returned by command dispatch.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    /// The operation is known but has not been ported yet.
    #[error("Oriedita operation {operation:?} is not supported yet")]
    UnsupportedOperation {
        /// Unsupported operation.
        operation: OperationId,
    },
    /// The operation is actively tracked but has no executable implementation.
    #[error("Oriedita operation {operation:?} is not implemented yet")]
    NotImplemented {
        /// Not-yet-implemented operation.
        operation: OperationId,
    },
    /// The operation received invalid input.
    #[error("invalid input for Oriedita operation {operation:?}: {message}")]
    InvalidInput {
        /// Operation receiving invalid input.
        operation: OperationId,
        /// Explanation suitable for logs or user-facing diagnostics.
        message: String,
    },
}

/// High-level implementation state for a source-mapped Oriedita operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationStatus {
    /// The operation is known but not available.
    Unsupported,
    /// Implementation work has started but parity is incomplete.
    Porting,
    /// Rust unit coverage exists, but oracle coverage is incomplete.
    UnitTested,
    /// Rust behavior matches the pinned Oriedita oracle for committed fixtures.
    OracleTested,
    /// Behavior intentionally differs and is documented.
    DocumentedDifference,
    /// Swing/UI-only behavior that does not belong in this crate.
    OutOfScopeUi,
}

/// Source-map classification for an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationCategory {
    /// Non-UI kernel behavior.
    Kernel,
    /// File import/export behavior.
    Io,
    /// Handler/service source used to define command intent.
    KernelIntent,
    /// Preview-producing behavior represented as model-space candidates.
    KernelPreview,
    /// UI preview behavior that is not a kernel mutation.
    UiPreviewOnly,
    /// Swing/UI-only behavior outside this crate.
    OutOfScopeUi,
}

/// Identifier for every source-mapped Oriedita non-UI operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum OperationId {
    DrawCreaseFree,
    MoveCreasePattern,
    LineSegmentDelete,
    ChangeCreaseType,
    LengthenCrease,
    SquareBisector,
    Inward,
    PerpendicularDraw,
    SymmetricDraw,
    DrawCreaseRestricted,
    DrawCreaseSymmetric,
    DrawCreaseAngleRestricted,
    DrawPoint,
    DeletePoint,
    AngleSystem,
    DrawCreaseAngleRestricted3,
    CreaseSelect,
    CreaseUnselect,
    CreaseMove,
    CreaseCopy,
    CreaseMakeMountain,
    CreaseMakeValley,
    CreaseMakeEdge,
    BackgroundChangePosition,
    LineSegmentDivision,
    LineSegmentRatioSet,
    PolygonSetNoCorners,
    CreaseAdvanceType,
    CreaseMove4p,
    CreaseCopy4p,
    FishBoneDraw,
    CreaseMakeMv,
    DoubleSymmetricDraw,
    CreasesAlternateMv,
    DrawCreaseAngleRestricted5,
    VertexMakeAngularlyFlatFoldable,
    FoldableLineInput,
    ParallelDraw,
    VertexDeleteOnCrease,
    CircleDraw,
    CircleDrawThreePoint,
    CircleDrawSeparate,
    CircleDrawTangentLine,
    CircleDrawInverted,
    CircleDrawFree,
    CircleDrawConcentric,
    CircleDrawConcentricSelect,
    CircleDrawConcentricTwoCircleSelect,
    ParallelDrawWidth,
    ContinuousSymmetricDraw,
    DisplayLengthBetweenPoints1,
    DisplayLengthBetweenPoints2,
    DisplayAngleBetweenThreePoints1,
    DisplayAngleBetweenThreePoints2,
    DisplayAngleBetweenThreePoints3,
    CreaseToggleMv,
    CircleChangeColor,
    CreaseMakeAux,
    OperationFrameCreate,
    VoronoiCreate,
    FlatFoldableCheck,
    CreaseDeleteOverlapping,
    CreaseDeleteIntersecting,
    SelectPolygon,
    UnselectPolygon,
    SelectLineIntersecting,
    UnselectLineIntersecting,
    LengthenCreaseSameColor,
    FoldableLineDraw,
    ReplaceLineTypeSelect,
    DeleteLineTypeSelect,
    SelectLasso,
    UnselectLasso,
    Text,
    DrawBlintz,
    DrawFishBase,
    DrawDoveBase,
    DrawBirdBase,
    DrawFrogBase,
    ModifyCalculatedShape,
    MoveCalculatedShape,
    ChangeStandardFace,
    AddFoldingConstraint,
    Axiom5,
    Axiom7,
    FixInaccurate,
    ImportCp,
    ExportCp,
    ImportFold,
    ExportFold,
    ImportOri,
    ExportOri,
    ImportOrh,
    ExportOrh,
    ImportObj,
    ExportDxf,
    SaveConvert,
    SaveVersionDetect,
    CheckCamv,
    FoldingEstimate,
    FoldingEstimateSpecific,
    FoldingEstimateSave100,
    TwoColoredCp,
    Fold,
    FoldAnother,
    DuplicateFoldedModel,
    Check1,
    Check2,
    Check3,
    Check4,
    Fix1,
    Fix2,
    OrganizeCircles,
}

/// Source-map descriptor for an Oriedita operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationDescriptor {
    /// Stable operation identifier.
    pub id: OperationId,
    /// Pinned Oriedita source element.
    pub upstream: &'static str,
    /// Planned Rust module/function home.
    pub target: &'static str,
    /// Source-map category.
    pub category: OperationCategory,
    /// Planned port stage from `implementation-plans/oriedita-port.md`.
    pub stage: u8,
    /// Current implementation status.
    pub status: OperationStatus,
}

macro_rules! descriptor {
    ($id:ident, $upstream:literal, $target:literal, $category:ident, $stage:literal, $status:ident) => {
        OperationDescriptor {
            id: OperationId::$id,
            upstream: $upstream,
            target: $target,
            category: OperationCategory::$category,
            stage: $stage,
            status: OperationStatus::$status,
        }
    };
}

const OPERATION_DESCRIPTORS: &[OperationDescriptor] = &[
    descriptor!(
        DrawCreaseFree,
        "MouseHandlerDrawCreaseFree",
        "operations::construction::draw_crease_segment",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        MoveCreasePattern,
        "MouseHandlerMoveCreasePattern",
        "runtime camera pan, no persisted CP mutation",
        UiPreviewOnly,
        0,
        OutOfScopeUi
    ),
    descriptor!(
        LineSegmentDelete,
        "MouseHandlerLineSegmentDelete",
        "operations::arrangement::delete_line_segments_for_indices",
        Kernel,
        5,
        OracleTested
    ),
    descriptor!(
        ChangeCreaseType,
        "MouseHandlerChangeCreaseType",
        "operations::color::change_crease_type",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        LengthenCrease,
        "MouseHandlerLengthenCrease",
        "operations::transform::lengthen_crease",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        SquareBisector,
        "MouseHandlerSquareBisector",
        "operations::construction::square_bisector_*",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        Inward,
        "MouseHandlerInward",
        "operations::construction::inward",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        PerpendicularDraw,
        "MouseHandlerPerpendicularDraw",
        "operations::construction::perpendicular_projection",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        SymmetricDraw,
        "MouseHandlerSymmetricDraw",
        "operations::construction::symmetric_draw",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        DrawCreaseRestricted,
        "MouseHandlerDrawCreaseRestricted",
        "operations::construction::draw_crease_segment",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        DrawCreaseSymmetric,
        "MouseHandlerDrawCreaseSymmetric",
        "operations::construction::mirror_selected_lines",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        DrawCreaseAngleRestricted,
        "MouseHandlerDrawCreaseAngleRestricted",
        "operations::construction::angle_restricted_converging_candidates",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        DrawPoint,
        "MouseHandlerDrawPoint",
        "operations::point::draw_point_on_segment",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        DeletePoint,
        "MouseHandlerDeletePoint",
        "operations::point::delete_point",
        Kernel,
        5,
        OracleTested
    ),
    descriptor!(
        AngleSystem,
        "MouseHandlerAngleSystem",
        "operations::construction::angle_system_candidates",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        DrawCreaseAngleRestricted3,
        "MouseHandlerDrawCreaseAngleRestricted3_2",
        "operations::construction::draw_crease_angle_restricted_3_candidates",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        CreaseSelect,
        "MouseHandlerCreaseSelect",
        "operations::selection::select_indices/select_box",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        CreaseUnselect,
        "MouseHandlerCreaseUnselect",
        "operations::selection::unselect_indices/unselect_box",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        CreaseMove,
        "MouseHandlerCreaseMove",
        "operations::transform::move_selected_lines",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        CreaseCopy,
        "MouseHandlerCreaseCopy",
        "operations::transform::copy_selected_lines",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        CreaseMakeMountain,
        "MouseHandlerCreaseMakeMountain",
        "operations::color::make_mountain",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        CreaseMakeValley,
        "MouseHandlerCreaseMakeValley",
        "operations::color::make_valley",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        CreaseMakeEdge,
        "MouseHandlerCreaseMakeEdge",
        "operations::color::make_edge",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        BackgroundChangePosition,
        "MouseHandlerBackgroundChangePosition",
        "none",
        OutOfScopeUi,
        0,
        OutOfScopeUi
    ),
    descriptor!(
        LineSegmentDivision,
        "MouseHandlerLineSegmentDivision",
        "operations::point::divide_segment_by_count",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        LineSegmentRatioSet,
        "MouseHandlerLineSegmentRatioSet",
        "operations::point::divide_segment_by_ratio",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        PolygonSetNoCorners,
        "MouseHandlerPolygonSetNoCorners",
        "operations::generators::regular_polygon",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        CreaseAdvanceType,
        "MouseHandlerCreaseAdvanceType",
        "operations::color::advance_line_type",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        CreaseMove4p,
        "MouseHandlerCreaseMove4p",
        "operations::transform::move_selected_lines_by_points",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        CreaseCopy4p,
        "MouseHandlerCreaseCopy4p",
        "operations::transform::copy_selected_lines_by_points",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        FishBoneDraw,
        "MouseHandlerFishBoneDraw",
        "operations::construction::fishbone_draw",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        CreaseMakeMv,
        "MouseHandlerCreaseMakeMV",
        "operations::color::alternate_mountain_valley_along",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        DoubleSymmetricDraw,
        "MouseHandlerDoubleSymmetricDraw",
        "operations::construction::double_symmetric_draw",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        CreasesAlternateMv,
        "MouseHandlerCreasesAlternateMV",
        "operations::color::alternate_mountain_valley_crossing",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        DrawCreaseAngleRestricted5,
        "MouseHandlerDrawCreaseAngleRestricted5",
        "operations::construction::draw_crease_angle_restricted_5",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        VertexMakeAngularlyFlatFoldable,
        "MouseHandlerVertexMakeAngularlyFlatFoldable",
        "operations::construction::make_vertex_flat_foldable_candidates",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        FoldableLineInput,
        "MouseHandlerFoldableLineInput",
        "operations::construction::foldable_line_input_candidates",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        ParallelDraw,
        "MouseHandlerParallelDraw",
        "operations::construction::parallel_draw",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        VertexDeleteOnCrease,
        "MouseHandlerVertexDeleteOnCrease",
        "operations::point::delete_vertex_on_crease",
        Kernel,
        5,
        OracleTested
    ),
    descriptor!(
        CircleDraw,
        "MouseHandlerCircleDraw",
        "operations::circle::draw",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        CircleDrawThreePoint,
        "MouseHandlerCircleDrawThreePoint",
        "operations::circle::through_three_points",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        CircleDrawSeparate,
        "MouseHandlerCircleDrawSeparate",
        "operations::circle::separate",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        CircleDrawTangentLine,
        "MouseHandlerCircleDrawTangentLine",
        "operations::circle::tangent_line",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        CircleDrawInverted,
        "MouseHandlerCircleDrawInverted",
        "operations::circle::inverted",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        CircleDrawFree,
        "MouseHandlerCircleDrawFree",
        "operations::circle::free",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        CircleDrawConcentric,
        "MouseHandlerCircleDrawConcentric",
        "operations::circle::concentric",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        CircleDrawConcentricSelect,
        "MouseHandlerCircleDrawConcentricSelect",
        "operations::circle::concentric_select",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        CircleDrawConcentricTwoCircleSelect,
        "MouseHandlerCircleDrawConcentricTwoCircleSelect",
        "operations::circle::concentric_two_circle_select",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        ParallelDrawWidth,
        "MouseHandlerParallelDrawWidth",
        "operations::construction::parallel_width_indicators",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        ContinuousSymmetricDraw,
        "MouseHandlerContinuousSymmetricDraw",
        "operations::construction::continuous_symmetric_draw",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        DisplayLengthBetweenPoints1,
        "MouseHandlerDisplayLengthBetweenPoints",
        "operations::measure::length_between_points",
        KernelPreview,
        7,
        OracleTested
    ),
    descriptor!(
        DisplayLengthBetweenPoints2,
        "MouseHandlerDisplayLengthBetweenPoints",
        "operations::measure::length_between_points",
        KernelPreview,
        7,
        OracleTested
    ),
    descriptor!(
        DisplayAngleBetweenThreePoints1,
        "MouseHandlerDisplayAngleBetweenThreePoints",
        "operations::measure::angle_between_three_points",
        KernelPreview,
        7,
        OracleTested
    ),
    descriptor!(
        DisplayAngleBetweenThreePoints2,
        "MouseHandlerDisplayAngleBetweenThreePoints",
        "operations::measure::angle_between_three_points",
        KernelPreview,
        7,
        OracleTested
    ),
    descriptor!(
        DisplayAngleBetweenThreePoints3,
        "MouseHandlerDisplayAngleBetweenThreePoints",
        "operations::measure::angle_between_three_points",
        KernelPreview,
        7,
        OracleTested
    ),
    descriptor!(
        CreaseToggleMv,
        "MouseHandlerCreaseToggleMV",
        "operations::color::toggle_mountain_valley",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        CircleChangeColor,
        "MouseHandlerCircleChangeColor",
        "operations::circle::change_color",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        CreaseMakeAux,
        "MouseHandlerCreaseMakeAux",
        "operations::color::make_aux",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        OperationFrameCreate,
        "MouseHandlerOperationFrameCreate",
        "operations::transform::operation_frame_press/drag/release",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        VoronoiCreate,
        "MouseHandlerVoronoiCreate",
        "operations::generators::voronoi_press/apply",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        FlatFoldableCheck,
        "MouseHandlerFlatFoldableCheck",
        "checks::flat_foldable_boundary_check",
        Kernel,
        9,
        OracleTested
    ),
    descriptor!(
        CreaseDeleteOverlapping,
        "MouseHandlerCreaseDeleteOverlapping",
        "operations::arrangement::delete_overlapping",
        Kernel,
        5,
        OracleTested
    ),
    descriptor!(
        CreaseDeleteIntersecting,
        "MouseHandlerCreaseDeleteIntersecting",
        "operations::arrangement::delete_intersecting",
        Kernel,
        5,
        OracleTested
    ),
    descriptor!(
        SelectPolygon,
        "MouseHandlerSelectPolygon",
        "operations::selection::select_polygon",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        UnselectPolygon,
        "MouseHandlerUnselectPolygon",
        "operations::selection::unselect_polygon",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        SelectLineIntersecting,
        "MouseHandlerSelectLineIntersecting",
        "operations::selection::select_intersecting_line",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        UnselectLineIntersecting,
        "MouseHandlerUnselectLineIntersecting",
        "operations::selection::unselect_intersecting_line",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        LengthenCreaseSameColor,
        "MouseHandlerLengthenCreaseSameColor",
        "operations::transform::lengthen_crease",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        FoldableLineDraw,
        "MouseHandlerFoldableLineDraw",
        "operations::construction::foldable_line_draw_operation_mode",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        ReplaceLineTypeSelect,
        "MouseHandlerReplaceTypeSelect",
        "operations::color::replace_line_type_for_indices",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        DeleteLineTypeSelect,
        "MouseHandlerDeleteTypeSelect",
        "operations::color::delete_line_type_for_indices",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        SelectLasso,
        "MouseHandlerSelectLasso",
        "operations::selection::select_lasso",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        UnselectLasso,
        "MouseHandlerUnselectLasso",
        "operations::selection::unselect_lasso",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        Text,
        "MouseHandlerText",
        "operations::text",
        Kernel,
        6,
        OracleTested
    ),
    descriptor!(
        DrawBlintz,
        "MouseHandlerDrawBlintz",
        "operations::generators::default_molecule",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        DrawFishBase,
        "MouseHandlerDrawFishBase",
        "operations::generators::default_molecule",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        DrawDoveBase,
        "MouseHandlerDrawDoveBase",
        "operations::generators::default_molecule",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        DrawBirdBase,
        "MouseHandlerDrawBirdBase",
        "operations::generators::default_molecule",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        DrawFrogBase,
        "MouseHandlerDrawFrogBase",
        "operations::generators::default_molecule",
        Kernel,
        8,
        OracleTested
    ),
    descriptor!(
        ModifyCalculatedShape,
        "MouseHandlerModifyCalculatedShape",
        "folding::modify_calculated_shape",
        Kernel,
        10,
        Unsupported
    ),
    descriptor!(
        MoveCalculatedShape,
        "MouseHandlerMoveCalculatedShape",
        "folding::move_calculated_shape",
        Kernel,
        10,
        Unsupported
    ),
    descriptor!(
        ChangeStandardFace,
        "MouseHandlerChangeStandardFace",
        "folding::change_standard_face",
        Kernel,
        10,
        Unsupported
    ),
    descriptor!(
        AddFoldingConstraint,
        "MouseHandlerAddFoldingConstraints",
        "folding::constraints",
        Kernel,
        10,
        Unsupported
    ),
    descriptor!(
        Axiom5,
        "MouseHandlerAxiom5",
        "operations::construction::axiom5_indicators",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        Axiom7,
        "MouseHandlerAxiom7",
        "operations::construction::axiom7_*",
        Kernel,
        7,
        OracleTested
    ),
    descriptor!(
        FixInaccurate,
        "MouseHandlerCreaseFixInaccurate",
        "checks::fix_inaccurate_for_indices",
        Kernel,
        9,
        OracleTested
    ),
    descriptor!(ImportCp, "CpImporter", "io::cp::import", Io, 4, UnitTested),
    descriptor!(ExportCp, "CpExporter", "io::cp::export", Io, 4, UnitTested),
    descriptor!(
        ImportFold,
        "FoldImporter",
        "io::fold::import",
        Io,
        4,
        UnitTested
    ),
    descriptor!(
        ExportFold,
        "FoldExporter",
        "io::fold::export",
        Io,
        4,
        UnitTested
    ),
    descriptor!(
        ImportOri,
        "OriImporter",
        "io::ori::import",
        Io,
        4,
        UnitTested
    ),
    descriptor!(
        ExportOri,
        "OriExporter",
        "io::ori::export",
        Io,
        4,
        UnitTested
    ),
    descriptor!(
        ImportOrh,
        "OrhImporter",
        "io::orh::import",
        Io,
        4,
        OracleTested
    ),
    descriptor!(
        ExportOrh,
        "OrhExporter",
        "io::orh::export",
        Io,
        4,
        OracleTested
    ),
    descriptor!(
        ImportObj,
        "ObjImporter",
        "io::obj::import",
        Io,
        4,
        OracleTested
    ),
    descriptor!(
        ExportDxf,
        "DxfExporter",
        "io::dxf::export",
        Io,
        4,
        OracleTested
    ),
    descriptor!(
        SaveConvert,
        "SaveConverter",
        "io::save::convert",
        Io,
        4,
        UnitTested
    ),
    descriptor!(
        SaveVersionDetect,
        "FileVersionTester",
        "io::save::version",
        Io,
        4,
        UnitTested
    ),
    descriptor!(
        CheckCamv,
        "CheckCAMVTask",
        "checks::check_camv_task",
        Kernel,
        9,
        OracleTested
    ),
    descriptor!(
        FoldingEstimate,
        "FoldingEstimateTask",
        "folding::FoldingEstimateSession",
        Kernel,
        10,
        Porting
    ),
    descriptor!(
        FoldingEstimateSpecific,
        "FoldingEstimateSpecificTask",
        "folding::folding_estimate_to_case",
        Kernel,
        10,
        Porting
    ),
    descriptor!(
        FoldingEstimateSave100,
        "FoldingEstimateSave100Task",
        "folding::folding_estimate_save_batch",
        Kernel,
        10,
        Porting
    ),
    descriptor!(
        TwoColoredCp,
        "TwoColoredTask",
        "folding::two_colored_folding_estimate_from_segments",
        Kernel,
        10,
        Porting
    ),
    descriptor!(
        Fold,
        "FoldingServiceImpl.fold",
        "folding::commands::fold",
        KernelIntent,
        10,
        Unsupported
    ),
    descriptor!(
        FoldAnother,
        "FoldingServiceImpl.foldAnother",
        "folding::fold_another",
        KernelIntent,
        10,
        Porting
    ),
    descriptor!(
        DuplicateFoldedModel,
        "FoldingServiceImpl.duplicate",
        "folding::duplicate_estimation_order_for_display",
        KernelIntent,
        10,
        Porting
    ),
    descriptor!(Check1, "Check1", "checks::check1", Kernel, 9, OracleTested),
    descriptor!(Check2, "Check2", "checks::check2", Kernel, 9, OracleTested),
    descriptor!(Check3, "Check3", "checks::check3", Kernel, 9, OracleTested),
    descriptor!(Check4, "Check4", "checks::check4", Kernel, 9, OracleTested),
    descriptor!(
        Fix1,
        "Fix1",
        "operations::arrangement::fix1",
        Kernel,
        9,
        OracleTested
    ),
    descriptor!(
        Fix2,
        "Fix2",
        "operations::arrangement::fix2",
        Kernel,
        9,
        OracleTested
    ),
    descriptor!(
        OrganizeCircles,
        "OrganizeCircles",
        "operations::circle::organize",
        Kernel,
        8,
        OracleTested
    ),
];

/// Return all source-mapped Oriedita operation descriptors.
pub fn operation_descriptors() -> &'static [OperationDescriptor] {
    OPERATION_DESCRIPTORS
}

/// Return the descriptor for one operation.
pub fn operation_descriptor(operation: OperationId) -> Option<&'static OperationDescriptor> {
    operation_descriptors()
        .iter()
        .find(|descriptor| descriptor.id == operation)
}

/// Return the current implementation status for one operation.
pub fn operation_status(operation: OperationId) -> OperationStatus {
    operation_descriptor(operation)
        .map(|descriptor| descriptor.status)
        .unwrap_or(OperationStatus::Unsupported)
}

/// Dispatch a command against a crease-pattern document.
pub fn execute_command(
    document: &mut CreasePatternDocument,
    command: CreasePatternCommand,
) -> Result<CommandResult> {
    let status = operation_status(command.operation);
    match status {
        OperationStatus::Unsupported | OperationStatus::OutOfScopeUi => {
            return Err(CommandError::UnsupportedOperation {
                operation: command.operation,
            });
        }
        OperationStatus::Porting
        | OperationStatus::UnitTested
        | OperationStatus::OracleTested
        | OperationStatus::DocumentedDifference => {}
    }

    let changed = match command.operation {
        OperationId::DrawCreaseFree | OperationId::DrawCreaseRestricted => {
            let points = required_points(&command, 2)?;
            usize::from(operations::construction::draw_crease_segment(
                &mut document.crease_pattern,
                &LineSegment::with_color(points[0], points[1], active_line_color(&command)),
                operations::construction::DrawCreaseTarget::FoldLine,
            ))
        }
        OperationId::LineSegmentDelete => {
            let line_indices = required_line_indices(&command)?;
            operations::arrangement::delete_line_segments_for_indices(
                &mut document.crease_pattern,
                &line_indices,
            )
        }
        OperationId::ChangeCreaseType => {
            let line_indices = required_line_indices(&command)?;
            line_indices
                .iter()
                .filter(|index| {
                    operations::color::change_crease_type(&mut document.crease_pattern, **index)
                })
                .count()
        }
        OperationId::DeletePoint => {
            let points = required_points(&command, 1)?;
            let before = document.crease_pattern.line_segments.len();
            operations::arrangement::del_v_at_point(
                &mut document.crease_pattern,
                points[0],
                selection_distance(&command),
                Epsilon::UNKNOWN_1EN6,
            );
            before.abs_diff(document.crease_pattern.line_segments.len())
        }
        OperationId::DrawPoint => {
            let points = required_points(&command, 1)?;
            let (index, _) = nearest_line_segment(
                &document.crease_pattern,
                points[0],
                selection_distance(&command),
            )?;
            usize::from(operations::point::draw_point_on_segment(
                &mut document.crease_pattern,
                index,
                points[0],
                selection_distance(&command),
            ))
        }
        OperationId::CreaseSelect => {
            if command.payload.line_ids.is_empty() {
                let polygon = required_selection_polygon(&command)?;
                operations::selection::select_box(&mut document.crease_pattern, &polygon)
            } else {
                let line_indices = required_line_indices(&command)?;
                operations::selection::select_indices(&mut document.crease_pattern, &line_indices)
            }
        }
        OperationId::CreaseUnselect => {
            if command.payload.line_ids.is_empty() {
                let polygon = required_selection_polygon(&command)?;
                operations::selection::unselect_box(&mut document.crease_pattern, &polygon)
            } else {
                let line_indices = required_line_indices(&command)?;
                operations::selection::unselect_indices(&mut document.crease_pattern, &line_indices)
            }
        }
        OperationId::CreaseMakeMountain => {
            let line_indices = required_line_indices(&command)?;
            operations::color::make_mountain(&mut document.crease_pattern, &line_indices)
        }
        OperationId::CreaseMakeValley => {
            let line_indices = required_line_indices(&command)?;
            operations::color::make_valley(&mut document.crease_pattern, &line_indices)
        }
        OperationId::CreaseMakeEdge => {
            let line_indices = required_line_indices(&command)?;
            operations::color::make_edge(&mut document.crease_pattern, &line_indices)
        }
        OperationId::CreaseMakeAux => {
            let line_indices = required_line_indices(&command)?;
            operations::color::make_aux(&mut document.crease_pattern, &line_indices)
        }
        OperationId::CreaseToggleMv => {
            let line_indices = required_line_indices(&command)?;
            operations::color::toggle_mountain_valley(&mut document.crease_pattern, &line_indices)
        }
        OperationId::CreaseAdvanceType => {
            let line_indices = required_line_indices(&command)?;
            line_indices
                .iter()
                .filter(|index| {
                    operations::color::advance_line_type(&mut document.crease_pattern, **index)
                })
                .count()
        }
        OperationId::CreaseMove => {
            let line_indices = required_line_indices(&command)?;
            let points = required_points(&command, 2)?;
            set_selected_line_flags(&mut document.crease_pattern, &line_indices);
            operations::transform::move_selected_lines(
                &mut document.crease_pattern,
                points[0].delta(points[1]),
            )
        }
        OperationId::CreaseCopy => {
            let line_indices = required_line_indices(&command)?;
            let points = required_points(&command, 2)?;
            set_selected_line_flags(&mut document.crease_pattern, &line_indices);
            operations::transform::copy_selected_lines(
                &mut document.crease_pattern,
                points[0].delta(points[1]),
            )
        }
        OperationId::CreaseMove4p => {
            let line_indices = required_line_indices(&command)?;
            let points = required_points(&command, 4)?;
            set_selected_line_flags(&mut document.crease_pattern, &line_indices);
            operations::transform::move_selected_lines_by_points(
                &mut document.crease_pattern,
                points[0],
                points[1],
                points[2],
                points[3],
            )
        }
        OperationId::CreaseCopy4p => {
            let line_indices = required_line_indices(&command)?;
            let points = required_points(&command, 4)?;
            set_selected_line_flags(&mut document.crease_pattern, &line_indices);
            operations::transform::copy_selected_lines_by_points(
                &mut document.crease_pattern,
                points[0],
                points[1],
                points[2],
                points[3],
            )
        }
        OperationId::LineSegmentDivision => {
            let segment = required_or_nearest_line_segment(document, &command)?;
            operations::point::divide_segment_by_count(
                &mut document.crease_pattern,
                &segment,
                division_count(&command),
            )
        }
        OperationId::LineSegmentRatioSet => {
            let segment = required_or_nearest_line_segment(document, &command)?;
            operations::point::divide_segment_by_ratio(
                &mut document.crease_pattern,
                &segment,
                ratio_s(&command),
                ratio_t(&command),
            )
        }
        OperationId::SquareBisector => {
            if command.payload.line_ids.len() >= 3 {
                let line_indices = required_line_indices(&command)?;
                let first =
                    line_segment_for_operation(document, command.operation, line_indices[0])?;
                let second =
                    line_segment_for_operation(document, command.operation, line_indices[1])?;
                let destination =
                    line_segment_for_operation(document, command.operation, line_indices[2])?;
                usize::from(
                    operations::construction::square_bisector_from_lines_to_destination(
                        &mut document.crease_pattern,
                        &first,
                        &second,
                        &destination,
                        active_line_color(&command),
                    ),
                )
            } else {
                let points = required_points(&command, 4)?;
                let (_, destination) = nearest_line_segment(
                    &document.crease_pattern,
                    points[3],
                    selection_distance(&command),
                )?;
                usize::from(
                    operations::construction::square_bisector_from_points_to_destination(
                        &mut document.crease_pattern,
                        points[0],
                        points[1],
                        points[2],
                        &destination,
                        active_line_color(&command),
                    ),
                )
            }
        }
        OperationId::Inward => {
            let points = required_points(&command, 3)?;
            operations::construction::inward(
                &mut document.crease_pattern,
                points[0],
                points[1],
                points[2],
                active_line_color(&command),
            )
        }
        OperationId::PerpendicularDraw => {
            let points = required_points_at_least(&command, 2)?;
            let (_, base) = nearest_line_segment(
                &document.crease_pattern,
                points[1],
                selection_distance(&command),
            )?;
            if points.len() >= 3 {
                let (_, destination) = nearest_line_segment(
                    &document.crease_pattern,
                    points[2],
                    selection_distance(&command),
                )?;
                let indicator = operations::construction::perpendicular_indicator(
                    &document.crease_pattern,
                    points[0],
                    &base,
                )
                .unwrap_or_else(|| LineSegment::new(points[0], points[1]));
                usize::from(operations::construction::perpendicular_draw_to_destination(
                    &mut document.crease_pattern,
                    points[0],
                    &indicator,
                    &destination,
                    active_line_color(&command),
                ))
            } else if let Some(indicator) = operations::construction::perpendicular_indicator(
                &document.crease_pattern,
                points[0],
                &base,
            ) {
                usize::from(operations::construction::commit_perpendicular_indicator(
                    &mut document.crease_pattern,
                    &indicator,
                    active_line_color(&command),
                ))
            } else {
                usize::from(operations::construction::perpendicular_projection(
                    &mut document.crease_pattern,
                    points[0],
                    &base,
                    active_line_color(&command),
                ))
            }
        }
        OperationId::SymmetricDraw => {
            let points = required_points(&command, 2)?;
            let (_, source) = nearest_line_segment(
                &document.crease_pattern,
                points[0],
                selection_distance(&command),
            )?;
            let (_, mirror) = nearest_line_segment(
                &document.crease_pattern,
                points[1],
                selection_distance(&command),
            )?;
            usize::from(operations::construction::symmetric_draw(
                &mut document.crease_pattern,
                &source,
                &mirror,
                active_line_color(&command),
            ))
        }
        OperationId::DrawCreaseSymmetric => {
            let line_indices = required_line_indices(&command)?;
            let points = required_points(&command, 2)?;
            set_selected_line_flags(&mut document.crease_pattern, &line_indices);
            operations::construction::mirror_selected_lines(
                &mut document.crease_pattern,
                &LineSegment::new(points[0], points[1]),
            )
        }
        OperationId::DrawCreaseAngleRestricted => {
            let points = required_points(&command, 3)?;
            let segment = LineSegment::new(points[0], points[1]);
            let candidates = operations::construction::angle_restricted_converging_candidates(
                &segment,
                angle_system_divider(&command),
                angle_system_angles(&command),
            );
            let converge_point =
                nearest_candidate_point(&command, points[2], &candidates.intersections)?;
            operations::construction::draw_crease_angle_restricted_converging(
                &mut document.crease_pattern,
                &segment,
                converge_point,
                active_line_color(&command),
            )
        }
        OperationId::AngleSystem => {
            let points = required_points(&command, 3)?;
            let candidates = operations::construction::angle_system_candidates(
                points[0],
                points[1],
                angle_system_divider(&command),
                angle_system_angles(&command),
            );
            let selected = nearest_candidate_segment(&command, points[2], &candidates)?;
            let (_, destination) = nearest_line_segment(
                &document.crease_pattern,
                points[2],
                selection_distance(&command),
            )?;
            usize::from(operations::construction::angle_system_draw_to_destination(
                &mut document.crease_pattern,
                points[1],
                &selected,
                &destination,
                active_line_color(&command),
            ))
        }
        OperationId::DrawCreaseAngleRestricted3 => {
            let points = required_points(&command, 3)?;
            let candidates = operations::construction::draw_crease_angle_restricted_3_candidates(
                points[0],
                points[1],
                angle_system_divider(&command),
                angle_system_angles(&command),
            );
            let selected = nearest_candidate_segment(&command, points[2], &candidates)?;
            usize::from(
                operations::construction::draw_crease_angle_restricted_3_to_point(
                    &mut document.crease_pattern,
                    points[2],
                    points[1],
                    &selected,
                    selection_distance(&command),
                    active_line_color(&command),
                ),
            )
        }
        OperationId::FishBoneDraw => {
            let points = required_points(&command, 2)?;
            let grid_width = grid_width(&command, &document.crease_pattern);
            operations::construction::fishbone_draw(
                &mut document.crease_pattern,
                &LineSegment::new(points[0], points[1]),
                grid_width,
                active_line_color(&command),
                selection_distance(&command),
            )
        }
        OperationId::DoubleSymmetricDraw => {
            let points = required_points(&command, 2)?;
            operations::construction::double_symmetric_draw(
                &mut document.crease_pattern,
                &LineSegment::new(points[0], points[1]),
            )
        }
        OperationId::DrawCreaseAngleRestricted5 => {
            let points = required_points(&command, 2)?;
            usize::from(operations::construction::draw_crease_angle_restricted_5(
                &mut document.crease_pattern,
                points[0],
                points[1],
                angle_system_divider(&command),
                angle_system_angles(&command),
                selection_distance(&command),
                active_line_color(&command),
            ))
        }
        OperationId::VertexMakeAngularlyFlatFoldable => {
            let points = required_points(&command, 2)?;
            let candidates = operations::construction::make_vertex_flat_foldable_candidates(
                &document.crease_pattern,
                points[0],
                grid_width(&command, &document.crease_pattern),
                active_line_color(&command),
            );
            let selected = nearest_candidate_segment(&command, points[1], &candidates.candidates)?;
            let (_, destination) = nearest_line_segment(
                &document.crease_pattern,
                points[1],
                selection_distance(&command),
            )?;
            usize::from(
                operations::construction::make_vertex_flat_foldable_to_destination(
                    &mut document.crease_pattern,
                    points[0],
                    &selected,
                    &destination,
                    candidates.commit_color,
                ),
            )
        }
        OperationId::FoldableLineInput => {
            let points = required_points(&command, 2)?;
            let input = LineSegment::new(points[0], points[1]);
            usize::from(operations::construction::foldable_line_input_direct(
                &mut document.crease_pattern,
                &input,
                active_line_color(&command),
            ))
        }
        OperationId::ParallelDraw => {
            let points = required_points(&command, 3)?;
            let (_, parallel_segment) = nearest_line_segment(
                &document.crease_pattern,
                points[1],
                selection_distance(&command),
            )?;
            let (_, destination_segment) = nearest_line_segment(
                &document.crease_pattern,
                points[2],
                selection_distance(&command),
            )?;
            usize::from(operations::construction::parallel_draw(
                &mut document.crease_pattern,
                points[0],
                &parallel_segment,
                &destination_segment,
                active_line_color(&command),
            ))
        }
        OperationId::ParallelDrawWidth => {
            let points = required_points(&command, 2)?;
            let selected_segment = required_or_nearest_line_segment(document, &command)?;
            let width = command
                .payload
                .width
                .filter(|width| width.is_finite() && *width > 0.0)
                .unwrap_or_else(|| determine_line_segment_distance(points[1], &selected_segment));
            let indicators =
                operations::construction::parallel_width_indicators(&selected_segment, width);
            let selected = nearest_candidate_segment(&command, points[1], &indicators)?;
            usize::from(operations::construction::commit_parallel_width_indicator(
                &mut document.crease_pattern,
                &selected,
                active_line_color(&command),
            ))
        }
        OperationId::ContinuousSymmetricDraw => {
            let points = required_points(&command, 2)?;
            operations::construction::continuous_symmetric_draw(
                &mut document.crease_pattern,
                points[0],
                points[1],
                active_line_color(&command),
            )
        }
        OperationId::FoldableLineDraw => {
            let points = required_points(&command, 2)?;
            let mode = operations::construction::foldable_line_draw_operation_mode(
                &document.crease_pattern,
                points[0],
                selection_distance(&command),
            );
            if mode == operations::construction::FoldableLineDrawOperationMode::DrawCreaseFree
                || operations::construction::foldable_line_draw_switches_to_free(
                    points[1],
                    points[0],
                    selection_distance(&command),
                )
            {
                usize::from(operations::construction::draw_crease_segment(
                    &mut document.crease_pattern,
                    &LineSegment::with_color(points[0], points[1], active_line_color(&command)),
                    operations::construction::DrawCreaseTarget::FoldLine,
                ))
            } else {
                let candidates = operations::construction::make_vertex_flat_foldable_candidates(
                    &document.crease_pattern,
                    points[0],
                    grid_width(&command, &document.crease_pattern),
                    active_line_color(&command),
                );
                let selected =
                    nearest_candidate_segment(&command, points[1], &candidates.candidates)?;
                let (_, destination) = nearest_line_segment(
                    &document.crease_pattern,
                    points[1],
                    selection_distance(&command),
                )?;
                usize::from(
                    operations::construction::make_vertex_flat_foldable_to_destination(
                        &mut document.crease_pattern,
                        points[0],
                        &selected,
                        &destination,
                        candidates.commit_color,
                    ),
                )
            }
        }
        OperationId::Axiom5 => {
            let points = required_points_at_least(&command, 3)?;
            let (_, target_segment) = nearest_line_segment(
                &document.crease_pattern,
                points[1],
                selection_distance(&command),
            )?;
            let indicators = operations::construction::axiom5_indicators(
                &document.crease_pattern,
                points[0],
                &target_segment,
                points[2],
            )
            .ok_or_else(|| CommandError::InvalidInput {
                operation: command.operation,
                message: "resolved Axiom 5 inputs do not produce a fold candidate".to_string(),
            })?;
            if points.len() >= 4 {
                let (_, destination) = nearest_line_segment(
                    &document.crease_pattern,
                    points[3],
                    selection_distance(&command),
                )?;
                usize::from(operations::construction::axiom5_draw_to_destination(
                    &mut document.crease_pattern,
                    points[2],
                    &indicators[0],
                    &indicators[1],
                    &destination,
                    points[3],
                    active_line_color(&command),
                ))
            } else {
                let selected = nearest_candidate_segment(&command, points[2], &indicators)?;
                usize::from(operations::construction::commit_axiom5_indicator(
                    &mut document.crease_pattern,
                    &selected,
                    active_line_color(&command),
                ))
            }
        }
        OperationId::Axiom7 => {
            let points = required_points_at_least(&command, 3)?;
            let (_, target_segment) = nearest_line_segment(
                &document.crease_pattern,
                points[1],
                selection_distance(&command),
            )?;
            let (_, perpendicular_segment) = nearest_line_segment(
                &document.crease_pattern,
                points[2],
                selection_distance(&command),
            )?;
            let indicator = operations::construction::axiom7_indicator(
                &document.crease_pattern,
                points[0],
                &target_segment,
                &perpendicular_segment,
            )
            .ok_or_else(|| CommandError::InvalidInput {
                operation: command.operation,
                message: "resolved Axiom 7 inputs do not produce a fold candidate".to_string(),
            })?;
            if points.len() >= 4 {
                let (_, destination) = nearest_line_segment(
                    &document.crease_pattern,
                    points[3],
                    selection_distance(&command),
                )?;
                usize::from(operations::construction::axiom7_draw_to_destination(
                    &mut document.crease_pattern,
                    &indicator,
                    &destination,
                    active_line_color(&command),
                ))
            } else {
                usize::from(operations::construction::commit_axiom7_indicator(
                    &mut document.crease_pattern,
                    &indicator,
                    active_line_color(&command),
                ))
            }
        }
        OperationId::CreaseMakeMv => {
            let points = required_points(&command, 2)?;
            let guide = LineSegment::with_color(points[0], points[1], active_line_color(&command));
            operations::color::alternate_mountain_valley_along(
                &mut document.crease_pattern,
                &guide,
                active_line_color(&command),
            )
        }
        OperationId::CreasesAlternateMv => {
            let points = required_points(&command, 2)?;
            let guide = LineSegment::with_color(points[0], points[1], active_line_color(&command));
            operations::color::alternate_mountain_valley_crossing(
                &mut document.crease_pattern,
                &guide,
                active_line_color(&command),
            )
        }
        OperationId::VertexDeleteOnCrease => {
            let points = required_points(&command, 1)?;
            let before = document.crease_pattern.line_segments.len();
            operations::arrangement::del_v_at_point_color_change(
                &mut document.crease_pattern,
                points[0],
                selection_distance(&command),
                Epsilon::UNKNOWN_1EN6,
            );
            before.abs_diff(document.crease_pattern.line_segments.len())
        }
        OperationId::OperationFrameCreate => {
            let points = required_points_at_least(&command, 2)?;
            let mut state = operations::transform::operation_frame_press(
                &document.crease_pattern,
                &mut document.operation_frame,
                points[0],
                selection_distance(&command),
            );
            for point in points
                .iter()
                .copied()
                .skip(1)
                .take(points.len().saturating_sub(2))
            {
                operations::transform::operation_frame_drag(
                    &document.crease_pattern,
                    &mut document.operation_frame,
                    &mut state,
                    point,
                    selection_distance(&command),
                );
            }
            operations::transform::operation_frame_release(
                &document.crease_pattern,
                &mut document.operation_frame,
                &state,
                points[points.len() - 1],
                selection_distance(&command),
            );
            usize::from(document.operation_frame.active)
        }
        OperationId::CreaseDeleteOverlapping => {
            let points = required_points(&command, 2)?;
            delete_lines_along(document, &points, false)
        }
        OperationId::CreaseDeleteIntersecting => {
            let points = required_points(&command, 2)?;
            delete_lines_along(document, &points, true)
        }
        OperationId::SelectLineIntersecting => {
            let points = required_points(&command, 2)?;
            let selection = geometry::LineSegment::new(points[0], points[1]);
            operations::selection::select_intersecting_line(
                &mut document.crease_pattern,
                &selection,
            )
        }
        OperationId::SelectPolygon => {
            let polygon = required_selection_polygon(&command)?;
            operations::selection::select_polygon(&mut document.crease_pattern, &polygon)
        }
        OperationId::UnselectPolygon => {
            let polygon = required_selection_polygon(&command)?;
            operations::selection::unselect_polygon(&mut document.crease_pattern, &polygon)
        }
        OperationId::UnselectLineIntersecting => {
            let points = required_points(&command, 2)?;
            let selection = geometry::LineSegment::new(points[0], points[1]);
            operations::selection::unselect_intersecting_line(
                &mut document.crease_pattern,
                &selection,
            )
        }
        OperationId::FixInaccurate => {
            let line_indices = required_line_indices(&command)?;
            checks::fix_inaccurate_for_indices(
                &mut document.crease_pattern,
                &line_indices,
                checks::FixInaccurateOptions::default(),
            )
            .num_fixed_lines
        }
        OperationId::LengthenCrease => {
            let points = required_points(&command, 3)?;
            operations::transform::lengthen_crease(
                &mut document.crease_pattern,
                LineSegment::with_color(points[0], points[1], LineColor::Magenta5),
                points[2],
                selection_distance(&command),
                operations::transform::LengthenColorMode::Current(active_line_color(&command)),
            )
        }
        OperationId::LengthenCreaseSameColor => {
            let points = required_points(&command, 3)?;
            operations::transform::lengthen_crease(
                &mut document.crease_pattern,
                LineSegment::with_color(points[0], points[1], LineColor::Magenta5),
                points[2],
                selection_distance(&command),
                operations::transform::LengthenColorMode::SameAsOriginal,
            )
        }
        OperationId::ReplaceLineTypeSelect => {
            let line_indices = required_line_indices(&command)?;
            operations::color::replace_line_type_for_indices(
                &mut document.crease_pattern,
                &line_indices,
                command
                    .payload
                    .custom_from_line_type
                    .unwrap_or(model::CustomLineType::Any),
                command
                    .payload
                    .custom_to_line_type
                    .unwrap_or(model::CustomLineType::Edge),
            )
        }
        OperationId::DeleteLineTypeSelect => {
            let line_indices = required_line_indices(&command)?;
            operations::color::delete_line_type_for_indices(
                &mut document.crease_pattern,
                &line_indices,
                command
                    .payload
                    .custom_line_type
                    .unwrap_or(model::CustomLineType::Any),
            )
        }
        OperationId::SelectLasso => {
            let polygon = required_selection_polygon(&command)?;
            operations::selection::select_lasso(&mut document.crease_pattern, &polygon)
        }
        OperationId::UnselectLasso => {
            let polygon = required_selection_polygon(&command)?;
            operations::selection::unselect_lasso(&mut document.crease_pattern, &polygon)
        }
        _ => {
            return Err(CommandError::NotImplemented {
                operation: command.operation,
            });
        }
    };

    Ok(CommandResult {
        operation: command.operation,
        status,
        diagnostics: vec![format!("Changed {changed} line(s)")],
    })
}

/// Query transient candidate geometry for an active construction command.
pub fn preview_command(
    document: &CreasePatternDocument,
    command: CreasePatternCommand,
) -> Result<CommandPreview> {
    let status = operation_status(command.operation);
    match status {
        OperationStatus::Unsupported | OperationStatus::OutOfScopeUi => {
            return Err(CommandError::UnsupportedOperation {
                operation: command.operation,
            });
        }
        OperationStatus::Porting
        | OperationStatus::UnitTested
        | OperationStatus::OracleTested
        | OperationStatus::DocumentedDifference => {}
    }

    let mut preview = CommandPreview::default();
    let points = &command.payload.points;

    match command.operation {
        OperationId::DrawCreaseFree
        | OperationId::DrawCreaseRestricted
        | OperationId::DrawCreaseSymmetric
        | OperationId::DoubleSymmetricDraw
        | OperationId::ContinuousSymmetricDraw
        | OperationId::FishBoneDraw
        | OperationId::FoldableLineInput
        | OperationId::FoldableLineDraw
            if points.len() >= 2 =>
        {
            preview.segments.push(LineSegment::with_color(
                points[0],
                points[1],
                active_line_color(&command),
            ));
        }
        OperationId::Inward if points.len() >= 3 => {
            let center = geometry::center(points[0], points[1], points[2]);
            preview.segments.extend(
                points.iter().take(3).map(|point| {
                    LineSegment::with_color(*point, center, active_line_color(&command))
                }),
            );
        }
        OperationId::PerpendicularDraw if points.len() >= 2 => {
            let (_, base) = nearest_line_segment(
                &document.crease_pattern,
                points[1],
                selection_distance(&command),
            )?;
            if let Some(indicator) = operations::construction::perpendicular_indicator(
                &document.crease_pattern,
                points[0],
                &base,
            ) {
                preview.segments.push(indicator);
            } else {
                preview.segments.push(LineSegment::with_color(
                    points[0],
                    geometry::find_projection(
                        geometry::StraightLine::from_segment(&base),
                        points[0],
                    ),
                    active_line_color(&command),
                ));
            }
        }
        OperationId::DrawCreaseAngleRestricted if points.len() >= 2 => {
            let candidates = operations::construction::angle_restricted_converging_candidates(
                &LineSegment::new(points[0], points[1]),
                angle_system_divider(&command),
                angle_system_angles(&command),
            );
            preview.segments = candidates.indicators;
            preview.points = candidates.intersections;
        }
        OperationId::AngleSystem if points.len() >= 2 => {
            preview.segments = operations::construction::angle_system_candidates(
                points[0],
                points[1],
                angle_system_divider(&command),
                angle_system_angles(&command),
            );
        }
        OperationId::DrawCreaseAngleRestricted3 if points.len() >= 2 => {
            preview.segments = operations::construction::draw_crease_angle_restricted_3_candidates(
                points[0],
                points[1],
                angle_system_divider(&command),
                angle_system_angles(&command),
            );
        }
        OperationId::DrawCreaseAngleRestricted5 if points.len() >= 2 => {
            let snapped = operations::construction::snap_to_close_point_in_active_angle_system(
                &document.crease_pattern,
                points[0],
                points[1],
                angle_system_divider(&command),
                angle_system_angles(&command),
                selection_distance(&command),
            );
            preview.segments.push(LineSegment::with_color(
                points[0],
                snapped,
                active_line_color(&command),
            ));
        }
        OperationId::VertexMakeAngularlyFlatFoldable if !points.is_empty() => {
            let candidates = operations::construction::make_vertex_flat_foldable_candidates(
                &document.crease_pattern,
                points[0],
                grid_width(&command, &document.crease_pattern),
                active_line_color(&command),
            );
            preview.segments = candidates.candidates;
        }
        OperationId::ParallelDraw if points.len() >= 2 => {
            let (_, parallel_segment) = nearest_line_segment(
                &document.crease_pattern,
                points[1],
                selection_distance(&command),
            )?;
            preview.segments.push(LineSegment::with_color(
                points[0],
                Point::new(
                    points[0].x + parallel_segment.determine_bx() - parallel_segment.determine_ax(),
                    points[0].y + parallel_segment.determine_by() - parallel_segment.determine_ay(),
                ),
                active_line_color(&command),
            ));
        }
        OperationId::ParallelDrawWidth if points.len() >= 2 => {
            let selected_segment = required_or_nearest_line_segment(document, &command)?;
            let width = command
                .payload
                .width
                .filter(|width| width.is_finite() && *width > 0.0)
                .unwrap_or_else(|| determine_line_segment_distance(points[1], &selected_segment));
            preview.segments =
                operations::construction::parallel_width_indicators(&selected_segment, width)
                    .into_iter()
                    .collect();
        }
        OperationId::Axiom5 if points.len() >= 3 => {
            let (_, target_segment) = nearest_line_segment(
                &document.crease_pattern,
                points[1],
                selection_distance(&command),
            )?;
            if let Some(indicators) = operations::construction::axiom5_indicators(
                &document.crease_pattern,
                points[0],
                &target_segment,
                points[2],
            ) {
                preview.segments = indicators.into_iter().collect();
            }
        }
        OperationId::Axiom7 if points.len() >= 3 => {
            let (_, target_segment) = nearest_line_segment(
                &document.crease_pattern,
                points[1],
                selection_distance(&command),
            )?;
            let (_, perpendicular_segment) = nearest_line_segment(
                &document.crease_pattern,
                points[2],
                selection_distance(&command),
            )?;
            if let Some(indicator) = operations::construction::axiom7_indicator(
                &document.crease_pattern,
                points[0],
                &target_segment,
                &perpendicular_segment,
            ) {
                preview.segments.push(indicator);
            }
        }
        OperationId::SquareBisector if points.len() >= 3 => {
            let center = geometry::center(points[0], points[1], points[2]);
            preview.segments.push(LineSegment::with_color(
                points[1],
                center,
                active_line_color(&command),
            ));
        }
        OperationId::SymmetricDraw if points.len() >= 2 => {
            let (_, source) = nearest_line_segment(
                &document.crease_pattern,
                points[0],
                selection_distance(&command),
            )?;
            let (_, mirror) = nearest_line_segment(
                &document.crease_pattern,
                points[1],
                selection_distance(&command),
            )?;
            let mut clone = document.clone();
            if operations::construction::symmetric_draw(
                &mut clone.crease_pattern,
                &source,
                &mirror,
                active_line_color(&command),
            ) {
                preview.segments = clone
                    .crease_pattern
                    .line_segments
                    .into_iter()
                    .skip(document.crease_pattern.line_segments.len())
                    .collect();
            }
        }
        _ => {
            if points.len() >= 2 {
                preview.segments.push(LineSegment::with_color(
                    points[points.len() - 2],
                    points[points.len() - 1],
                    active_line_color(&command),
                ));
            }
        }
    }

    Ok(preview)
}

fn required_line_indices(command: &CreasePatternCommand) -> Result<Vec<usize>> {
    if command.payload.line_ids.is_empty() {
        return Err(CommandError::InvalidInput {
            operation: command.operation,
            message: "select at least one line".to_string(),
        });
    }

    command
        .payload
        .line_ids
        .iter()
        .map(|line_id| {
            line_id
                .checked_sub(1)
                .ok_or_else(|| CommandError::InvalidInput {
                    operation: command.operation,
                    message: "line IDs are one-based".to_string(),
                })
        })
        .collect()
}

fn required_points(command: &CreasePatternCommand, count: usize) -> Result<Vec<geometry::Point>> {
    if command.payload.points.len() != count {
        return Err(CommandError::InvalidInput {
            operation: command.operation,
            message: format!("expected {count} resolved point(s)"),
        });
    }
    Ok(command.payload.points.clone())
}

fn required_points_at_least(
    command: &CreasePatternCommand,
    count: usize,
) -> Result<Vec<geometry::Point>> {
    if command.payload.points.len() < count {
        return Err(CommandError::InvalidInput {
            operation: command.operation,
            message: format!("expected at least {count} resolved point(s)"),
        });
    }
    Ok(command.payload.points.clone())
}

fn required_selection_polygon(command: &CreasePatternCommand) -> Result<Polygon> {
    let points = required_points_at_least(command, 2)?;
    if points.len() == 2 {
        return Ok(rectangle_polygon(points[0], points[1]));
    }
    Ok(Polygon::new(points))
}

fn rectangle_polygon(a: geometry::Point, b: geometry::Point) -> Polygon {
    let min_x = a.x.min(b.x);
    let max_x = a.x.max(b.x);
    let min_y = a.y.min(b.y);
    let max_y = a.y.max(b.y);
    Polygon::new(vec![
        geometry::Point::new(min_x, min_y),
        geometry::Point::new(max_x, min_y),
        geometry::Point::new(max_x, max_y),
        geometry::Point::new(min_x, max_y),
    ])
}

fn active_line_color(command: &CreasePatternCommand) -> LineColor {
    command.payload.line_color.unwrap_or(LineColor::Red1)
}

fn angle_system_divider(command: &CreasePatternCommand) -> i32 {
    command
        .payload
        .angle_system_divider
        .filter(|divider| *divider >= 0)
        .unwrap_or(DEFAULT_ANGLE_SYSTEM_DIVIDER)
}

fn angle_system_angles(command: &CreasePatternCommand) -> [f64; 6] {
    command
        .payload
        .angles
        .unwrap_or(DEFAULT_ANGLE_SYSTEM_ANGLES)
}

fn selection_distance(command: &CreasePatternCommand) -> f64 {
    command
        .payload
        .selection_distance
        .filter(|distance| distance.is_finite() && *distance > 0.0)
        .unwrap_or(DEFAULT_SELECTION_DISTANCE)
}

fn grid_width(command: &CreasePatternCommand, model: &CreasePatternModel) -> f64 {
    command
        .payload
        .grid_width
        .filter(|width| width.is_finite() && *width > 0.0)
        .unwrap_or_else(|| {
            let grid_size = f64::from(model.grid.grid_size.max(1));
            ORIEDITA_PAPER_SIZE / grid_size
        })
}

fn division_count(command: &CreasePatternCommand) -> usize {
    command
        .payload
        .division_count
        .filter(|count| *count > 0)
        .unwrap_or(DEFAULT_LINE_DIVISION_COUNT)
}

fn ratio_s(command: &CreasePatternCommand) -> f64 {
    command
        .payload
        .ratio_s
        .filter(|ratio| ratio.is_finite() && *ratio >= 0.0)
        .unwrap_or(DEFAULT_LINE_RATIO)
}

fn ratio_t(command: &CreasePatternCommand) -> f64 {
    command
        .payload
        .ratio_t
        .filter(|ratio| ratio.is_finite() && *ratio >= 0.0)
        .unwrap_or(DEFAULT_LINE_RATIO)
}

fn set_selected_line_flags(model: &mut CreasePatternModel, line_indices: &[usize]) {
    operations::selection::unselect_all(model);
    operations::selection::select_indices(model, line_indices);
}

fn nearest_line_segment(
    model: &CreasePatternModel,
    point: Point,
    max_distance: f64,
) -> Result<(usize, LineSegment)> {
    let mut best: Option<(usize, LineSegment, f64)> = None;
    for (index, segment) in model.line_segments.iter().enumerate() {
        let distance = determine_line_segment_distance(point, segment);
        if best
            .as_ref()
            .is_none_or(|(_, _, best_distance)| distance < *best_distance)
        {
            best = Some((index, segment.clone(), distance));
        }
    }

    let Some((index, segment, distance)) = best else {
        return Err(CommandError::InvalidInput {
            operation: OperationId::DrawPoint,
            message: "document has no line segment candidates".to_string(),
        });
    };

    if distance > max_distance {
        return Err(CommandError::InvalidInput {
            operation: OperationId::DrawPoint,
            message: format!(
                "nearest line is outside selection distance ({distance:.6} > {max_distance:.6})"
            ),
        });
    }

    Ok((index, segment))
}

fn required_or_nearest_line_segment(
    document: &CreasePatternDocument,
    command: &CreasePatternCommand,
) -> Result<LineSegment> {
    if let Some(line_id) = command.payload.line_ids.first() {
        let index = line_id
            .checked_sub(1)
            .ok_or_else(|| CommandError::InvalidInput {
                operation: command.operation,
                message: "line IDs are one-based".to_string(),
            })?;
        return line_segment_for_operation(document, command.operation, index);
    }

    let points = required_points_at_least(command, 1)?;
    nearest_line_segment(
        &document.crease_pattern,
        points[0],
        selection_distance(command),
    )
    .map(|(_, segment)| segment)
    .map_err(|_| CommandError::InvalidInput {
        operation: command.operation,
        message: "pick or select a line segment".to_string(),
    })
}

fn line_segment_for_operation(
    document: &CreasePatternDocument,
    operation: OperationId,
    index: usize,
) -> Result<LineSegment> {
    document
        .crease_pattern
        .line_segments
        .get(index)
        .cloned()
        .ok_or_else(|| CommandError::InvalidInput {
            operation,
            message: format!("line index {} is out of bounds", index + 1),
        })
}

fn nearest_candidate_segment(
    command: &CreasePatternCommand,
    point: Point,
    candidates: &[LineSegment],
) -> Result<LineSegment> {
    if candidates.is_empty() {
        return Err(CommandError::InvalidInput {
            operation: command.operation,
            message: "no construction candidates are available".to_string(),
        });
    }

    if let Some(index) = command.payload.candidate_index {
        return candidates
            .get(index)
            .cloned()
            .ok_or_else(|| CommandError::InvalidInput {
                operation: command.operation,
                message: format!("candidate index {index} is out of bounds"),
            });
    }

    candidates
        .iter()
        .min_by(|left, right| {
            determine_line_segment_distance(point, left)
                .partial_cmp(&determine_line_segment_distance(point, right))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .ok_or_else(|| CommandError::InvalidInput {
            operation: command.operation,
            message: "no construction candidates are available".to_string(),
        })
}

fn nearest_candidate_point(
    command: &CreasePatternCommand,
    point: Point,
    candidates: &[Point],
) -> Result<Point> {
    if candidates.is_empty() {
        return Err(CommandError::InvalidInput {
            operation: command.operation,
            message: "no construction candidate points are available".to_string(),
        });
    }

    if let Some(index) = command.payload.candidate_index {
        return candidates
            .get(index)
            .copied()
            .ok_or_else(|| CommandError::InvalidInput {
                operation: command.operation,
                message: format!("candidate index {index} is out of bounds"),
            });
    }

    candidates
        .iter()
        .copied()
        .min_by(|left, right| {
            left.distance(point)
                .partial_cmp(&right.distance(point))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .ok_or_else(|| CommandError::InvalidInput {
            operation: command.operation,
            message: "no construction candidate points are available".to_string(),
        })
}

fn delete_lines_along(
    document: &mut CreasePatternDocument,
    points: &[geometry::Point],
    include_intersections: bool,
) -> usize {
    let before = document.crease_pattern.line_segments.len();
    let selection = geometry::LineSegment::new(points[0], points[1]);
    let deleted = if include_intersections {
        operations::arrangement::delete_intersecting_or_overlapping_lines_along(
            &mut document.crease_pattern,
            &selection,
        )
    } else {
        operations::arrangement::delete_overlapping_lines_along(
            &mut document.crease_pattern,
            &selection,
        )
    };

    if deleted {
        before.saturating_sub(document.crease_pattern.line_segments.len())
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{LineColor, Point};
    use std::collections::HashSet;

    #[test]
    fn registry_has_no_duplicate_operation_ids() {
        let mut ids = HashSet::new();

        for descriptor in operation_descriptors() {
            assert!(
                ids.insert(descriptor.id),
                "duplicate operation descriptor for {:?}",
                descriptor.id
            );
        }
    }

    #[test]
    fn registry_includes_representative_source_mapped_operations() {
        assert_eq!(
            operation_descriptor(OperationId::DrawCreaseFree).map(|descriptor| descriptor.target),
            Some("operations::construction::draw_crease_segment")
        );
        assert_eq!(
            operation_descriptor(OperationId::ImportFold).map(|descriptor| descriptor.category),
            Some(OperationCategory::Io)
        );
        assert_eq!(
            operation_descriptor(OperationId::Check4).map(|descriptor| descriptor.stage),
            Some(9)
        );
        assert_eq!(
            operation_status(OperationId::BackgroundChangePosition),
            OperationStatus::OutOfScopeUi
        );
    }

    #[test]
    fn registry_uses_dispatchable_status_values() {
        for descriptor in operation_descriptors() {
            assert!(
                matches!(
                    descriptor.status,
                    OperationStatus::Unsupported
                        | OperationStatus::Porting
                        | OperationStatus::UnitTested
                        | OperationStatus::OracleTested
                        | OperationStatus::DocumentedDifference
                        | OperationStatus::OutOfScopeUi
                ),
                "{:?} uses a status marker that command dispatch does not handle",
                descriptor.id
            );
        }
    }

    #[test]
    fn unsupported_dispatch_returns_typed_error_without_mutating_document() {
        let mut document = CreasePatternDocument {
            title: Some("fixture".to_string()),
            metadata: BTreeMap::new(),
            ..CreasePatternDocument::default()
        };
        let original = document.clone();

        let error = execute_command(&mut document, CreasePatternCommand::new(OperationId::Fold))
            .expect_err("stage one operations should be unsupported");

        assert_eq!(
            error,
            CommandError::UnsupportedOperation {
                operation: OperationId::Fold,
            }
        );
        assert_eq!(document, original);
    }

    #[test]
    fn command_dispatch_applies_oracle_tested_line_color_mutations() {
        let mut document = CreasePatternDocument::default();
        document.crease_pattern.add_line(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            LineColor::Blue2,
        );
        document.crease_pattern.add_line(
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            LineColor::Red1,
        );

        let result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::CreaseMakeMountain).with_payload(
                CreasePatternCommandPayload {
                    line_ids: vec![1, 2],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("selected line color command should execute");

        assert_eq!(result.operation, OperationId::CreaseMakeMountain);
        assert_eq!(result.status, OperationStatus::OracleTested);
        assert_eq!(result.diagnostics, vec!["Changed 1 line(s)"]);
        assert_eq!(
            document.crease_pattern.line_segments[0].color,
            LineColor::Red1
        );
        assert_eq!(
            document.crease_pattern.line_segments[1].color,
            LineColor::Red1
        );
    }

    #[test]
    fn command_dispatch_deletes_resolved_line_targets() {
        let mut document = CreasePatternDocument::default();
        for x in [0.0, 1.0, 2.0] {
            document.crease_pattern.add_line(
                Point::new(x, 0.0),
                Point::new(x, 1.0),
                LineColor::Black0,
            );
        }

        let result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::LineSegmentDelete).with_payload(
                CreasePatternCommandPayload {
                    line_ids: vec![1, 3],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("delete command should execute");

        assert_eq!(result.diagnostics, vec!["Changed 2 line(s)"]);
        assert_eq!(document.crease_pattern.line_segments.len(), 1);
        assert_eq!(document.crease_pattern.line_segments[0].a.x, 1.0);
    }

    #[test]
    fn command_dispatch_moves_resolved_selected_lines() {
        let mut document = CreasePatternDocument::default();
        document.crease_pattern.add_line(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            LineColor::Red1,
        );
        document.crease_pattern.add_line(
            Point::new(0.0, 2.0),
            Point::new(1.0, 2.0),
            LineColor::Blue2,
        );

        let result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::CreaseMove).with_payload(
                CreasePatternCommandPayload {
                    line_ids: vec![1],
                    points: vec![Point::new(0.0, 0.0), Point::new(2.0, 3.0)],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("move command should execute");

        assert_eq!(result.status, OperationStatus::OracleTested);
        assert_eq!(result.diagnostics, vec!["Changed 1 line(s)"]);
        assert_eq!(document.crease_pattern.line_segments.len(), 2);
        assert_eq!(
            document.crease_pattern.line_segments[0].a,
            Point::new(0.0, 2.0)
        );
        assert_eq!(
            document.crease_pattern.line_segments[1].a,
            Point::new(2.0, 3.0)
        );
        assert_eq!(
            document.crease_pattern.line_segments[1].b,
            Point::new(3.0, 3.0)
        );
    }

    #[test]
    fn command_dispatch_copies_resolved_selected_lines() {
        let mut document = CreasePatternDocument::default();
        document.crease_pattern.add_line(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            LineColor::Red1,
        );

        let result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::CreaseCopy).with_payload(
                CreasePatternCommandPayload {
                    line_ids: vec![1],
                    points: vec![Point::new(0.0, 0.0), Point::new(0.0, 2.0)],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("copy command should execute");

        assert_eq!(result.diagnostics, vec!["Changed 1 line(s)"]);
        assert_eq!(document.crease_pattern.line_segments.len(), 2);
        assert_eq!(
            document.crease_pattern.line_segments[0].a,
            Point::new(0.0, 0.0)
        );
        assert_eq!(
            document.crease_pattern.line_segments[1].a,
            Point::new(0.0, 2.0)
        );
        assert_eq!(
            document.crease_pattern.line_segments[1].b,
            Point::new(1.0, 2.0)
        );
    }

    #[test]
    fn command_dispatch_copies_resolved_selected_lines_by_four_points() {
        let mut document = CreasePatternDocument::default();
        document.crease_pattern.add_line(
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            LineColor::Red1,
        );

        let result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::CreaseCopy4p).with_payload(
                CreasePatternCommandPayload {
                    line_ids: vec![1],
                    points: vec![
                        Point::new(0.0, 0.0),
                        Point::new(1.0, 0.0),
                        Point::new(0.0, 0.0),
                        Point::new(0.0, 2.0),
                    ],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("four-point copy command should execute");

        assert_eq!(result.status, OperationStatus::OracleTested);
        assert_eq!(document.crease_pattern.line_segments.len(), 2);
        assert_close(document.crease_pattern.line_segments[1].a.x, 0.0);
        assert_close(document.crease_pattern.line_segments[1].a.y, 2.0);
        assert_close(document.crease_pattern.line_segments[1].b.x, -2.0);
        assert_close(document.crease_pattern.line_segments[1].b.y, 2.0);
    }

    #[test]
    fn command_dispatch_deletes_lines_overlapping_resolved_drag_segment() {
        let mut document = delete_along_fixture();

        let result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::CreaseDeleteOverlapping).with_payload(
                CreasePatternCommandPayload {
                    points: vec![Point::new(2.0, 0.0), Point::new(8.0, 0.0)],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("overlapping-line delete command should execute");

        assert_eq!(result.status, OperationStatus::OracleTested);
        assert_eq!(result.diagnostics, vec!["Changed 1 line(s)"]);
        assert_eq!(document.crease_pattern.line_segments.len(), 2);
        assert_eq!(
            document.crease_pattern.line_segments[0].a,
            Point::new(5.0, -5.0)
        );
        assert_eq!(
            document.crease_pattern.line_segments[1].a,
            Point::new(0.0, 1.0)
        );
    }

    #[test]
    fn command_dispatch_deletes_lines_intersecting_resolved_drag_segment() {
        let mut document = delete_along_fixture();

        let result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::CreaseDeleteIntersecting).with_payload(
                CreasePatternCommandPayload {
                    points: vec![Point::new(2.0, 0.0), Point::new(8.0, 0.0)],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("intersecting-line delete command should execute");

        assert_eq!(result.status, OperationStatus::OracleTested);
        assert_eq!(result.diagnostics, vec!["Changed 2 line(s)"]);
        assert_eq!(document.crease_pattern.line_segments.len(), 1);
        assert_eq!(
            document.crease_pattern.line_segments[0].a,
            Point::new(0.0, 1.0)
        );
    }

    #[test]
    fn command_dispatch_selects_lines_intersecting_resolved_drag_segment() {
        let mut document = delete_along_fixture();

        let result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::SelectLineIntersecting).with_payload(
                CreasePatternCommandPayload {
                    points: vec![Point::new(2.0, 0.0), Point::new(8.0, 0.0)],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("intersecting-line select command should execute");

        assert_eq!(result.status, OperationStatus::OracleTested);
        assert_eq!(result.diagnostics, vec!["Changed 2 line(s)"]);
        assert_eq!(
            document
                .crease_pattern
                .line_segments
                .iter()
                .map(|line| line.selected)
                .collect::<Vec<_>>(),
            vec![2, 2, 0]
        );
    }

    #[test]
    fn command_dispatch_unselects_lines_intersecting_resolved_drag_segment() {
        let mut document = delete_along_fixture();
        for line in &mut document.crease_pattern.line_segments {
            *line = line.with_selected(2);
        }

        let result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::UnselectLineIntersecting).with_payload(
                CreasePatternCommandPayload {
                    points: vec![Point::new(2.0, 0.0), Point::new(8.0, 0.0)],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("intersecting-line unselect command should execute");

        assert_eq!(result.diagnostics, vec!["Changed 2 line(s)"]);
        assert_eq!(
            document
                .crease_pattern
                .line_segments
                .iter()
                .map(|line| line.selected)
                .collect::<Vec<_>>(),
            vec![0, 0, 2]
        );
    }

    #[test]
    fn command_dispatch_fixes_inaccurate_selected_lines_with_default_options() {
        let mut document = CreasePatternDocument::default();
        document.crease_pattern.add_line(
            Point::new(0.1954, 0.0),
            Point::new(10.0, 0.0),
            LineColor::Red1,
        );

        let result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::FixInaccurate).with_payload(
                CreasePatternCommandPayload {
                    line_ids: vec![1],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("fix inaccurate command should execute");

        assert_eq!(result.status, OperationStatus::OracleTested);
        assert_eq!(result.diagnostics, vec!["Changed 1 line(s)"]);
        assert_close(document.crease_pattern.line_segments[0].a.x, 0.1953125);
    }

    #[test]
    fn command_dispatch_routes_stage_five_selection_polygons() {
        let mut document = CreasePatternDocument::default();
        document.crease_pattern.add_line(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            LineColor::Red1,
        );
        document.crease_pattern.add_line(
            Point::new(10.0, 10.0),
            Point::new(11.0, 10.0),
            LineColor::Blue2,
        );

        let result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::SelectPolygon).with_payload(
                CreasePatternCommandPayload {
                    points: vec![Point::new(-1.0, -1.0), Point::new(2.0, 1.0)],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("polygon selection should execute");

        assert_eq!(result.status, OperationStatus::OracleTested);
        assert_eq!(result.diagnostics, vec!["Changed 1 line(s)"]);
        assert_eq!(
            document
                .crease_pattern
                .line_segments
                .iter()
                .map(|line| line.selected)
                .collect::<Vec<_>>(),
            vec![2, 0]
        );

        execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::UnselectLasso).with_payload(
                CreasePatternCommandPayload {
                    points: vec![
                        Point::new(-1.0, -1.0),
                        Point::new(2.0, -1.0),
                        Point::new(2.0, 1.0),
                        Point::new(-1.0, 1.0),
                    ],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("lasso unselection should execute");
        assert_eq!(document.crease_pattern.line_segments[0].selected, 0);
    }

    #[test]
    fn command_dispatch_routes_stage_five_type_and_vertex_commands() {
        let mut document = CreasePatternDocument::default();
        document.crease_pattern.add_line(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            LineColor::Red1,
        );
        document.crease_pattern.add_line(
            Point::new(1.0, 0.0),
            Point::new(2.0, 0.0),
            LineColor::Red1,
        );
        document.crease_pattern.add_line(
            Point::new(0.0, 1.0),
            Point::new(1.0, 1.0),
            LineColor::Blue2,
        );

        execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::ReplaceLineTypeSelect).with_payload(
                CreasePatternCommandPayload {
                    line_ids: vec![1, 3],
                    custom_from_line_type: Some(model::CustomLineType::Valley),
                    custom_to_line_type: Some(model::CustomLineType::Edge),
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("replace line type should execute");
        assert_eq!(
            document.crease_pattern.line_segments[2].color,
            LineColor::Black0
        );

        execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::DeletePoint).with_payload(
                CreasePatternCommandPayload {
                    points: vec![Point::new(1.0, 0.0)],
                    selection_distance: Some(1.0),
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("delete point should execute");
        assert_eq!(document.crease_pattern.line_segments.len(), 2);
        assert_eq!(
            document.crease_pattern.line_segments[1].a,
            Point::new(0.0, 0.0)
        );
        assert_eq!(
            document.crease_pattern.line_segments[1].b,
            Point::new(2.0, 0.0)
        );
    }

    #[test]
    fn command_dispatch_routes_operation_frame_create() {
        let mut document = CreasePatternDocument::default();

        let result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::OperationFrameCreate).with_payload(
                CreasePatternCommandPayload {
                    points: vec![Point::new(0.0, 0.0), Point::new(4.0, 3.0)],
                    selection_distance: Some(0.5),
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("operation frame create should execute");

        assert_eq!(result.status, OperationStatus::OracleTested);
        assert_eq!(result.diagnostics, vec!["Changed 1 line(s)"]);
        assert!(document.operation_frame.active);
        assert_eq!(document.operation_frame.p1(), Point::new(0.0, 0.0));
        assert_eq!(document.operation_frame.p3(), Point::new(4.0, 3.0));
    }

    #[test]
    fn command_dispatch_requires_resolved_line_targets() {
        let mut document = CreasePatternDocument::default();

        let error = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::CreaseMakeValley),
        )
        .expect_err("selected line commands require line IDs");

        assert_eq!(
            error,
            CommandError::InvalidInput {
                operation: OperationId::CreaseMakeValley,
                message: "select at least one line".to_string(),
            }
        );
    }

    #[test]
    fn command_dispatch_requires_resolved_points_for_transform_commands() {
        let mut document = CreasePatternDocument::default();
        document.crease_pattern.add_line(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            LineColor::Red1,
        );

        let error = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::CreaseMove).with_payload(
                CreasePatternCommandPayload {
                    line_ids: vec![1],
                    points: vec![Point::new(0.0, 0.0)],
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect_err("move commands require a source and destination point");

        assert_eq!(
            error,
            CommandError::InvalidInput {
                operation: OperationId::CreaseMove,
                message: "expected 2 resolved point(s)".to_string(),
            }
        );
    }

    #[test]
    fn command_dispatch_requires_resolved_points_for_drag_delete_commands() {
        let mut document = delete_along_fixture();

        let error = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::CreaseDeleteOverlapping),
        )
        .expect_err("drag-delete commands require a drag segment");

        assert_eq!(
            error,
            CommandError::InvalidInput {
                operation: OperationId::CreaseDeleteOverlapping,
                message: "expected 2 resolved point(s)".to_string(),
            }
        );
    }

    #[test]
    fn command_dispatch_requires_resolved_points_for_intersecting_selection_commands() {
        let mut document = delete_along_fixture();

        let error = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::SelectLineIntersecting),
        )
        .expect_err("intersecting-line selection commands require a drag segment");

        assert_eq!(
            error,
            CommandError::InvalidInput {
                operation: OperationId::SelectLineIntersecting,
                message: "expected 2 resolved point(s)".to_string(),
            }
        );
    }

    #[test]
    fn command_dispatch_routes_stage_six_draw_and_point_commands() {
        let mut document = CreasePatternDocument::default();

        let draw_result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::DrawCreaseFree).with_payload(
                CreasePatternCommandPayload {
                    points: vec![Point::new(0.0, 0.0), Point::new(2.0, 0.0)],
                    line_color: Some(LineColor::Blue2),
                    selection_distance: Some(0.5),
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("free draw should execute through the command dispatcher");

        assert_eq!(draw_result.status, OperationStatus::OracleTested);
        assert_eq!(document.crease_pattern.line_segments.len(), 1);
        assert_eq!(
            document.crease_pattern.line_segments[0].color,
            LineColor::Blue2
        );

        let point_result = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::DrawPoint).with_payload(
                CreasePatternCommandPayload {
                    points: vec![Point::new(1.0, 0.0)],
                    selection_distance: Some(0.5),
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("draw point should split the nearest target segment");

        assert_eq!(point_result.status, OperationStatus::OracleTested);
        assert_eq!(document.crease_pattern.line_segments.len(), 2);
        assert!(
            document
                .crease_pattern
                .line_segments
                .iter()
                .any(|segment| segment.a == Point::new(1.0, 0.0)
                    || segment.b == Point::new(1.0, 0.0))
        );
    }

    #[test]
    fn command_preview_returns_stage_six_candidates_without_mutating_document() {
        let mut document = CreasePatternDocument::default();
        document.crease_pattern.add_line(
            Point::new(-1.0, 1.0),
            Point::new(2.0, 1.0),
            LineColor::Black0,
        );
        let before = document.clone();

        let preview = preview_command(
            &document,
            CreasePatternCommand::new(OperationId::DrawCreaseAngleRestricted).with_payload(
                CreasePatternCommandPayload {
                    points: vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)],
                    angle_system_divider: Some(4),
                    line_color: Some(LineColor::Red1),
                    selection_distance: Some(0.5),
                    ..CreasePatternCommandPayload::default()
                },
            ),
        )
        .expect("angle-restricted construction should expose preview candidates");

        assert_eq!(document, before);
        assert!(!preview.segments.is_empty());
        assert!(!preview.points.is_empty());
        assert!(
            preview
                .segments
                .iter()
                .any(|segment| segment.color == LineColor::Orange4)
        );
    }

    fn delete_along_fixture() -> CreasePatternDocument {
        let mut document = CreasePatternDocument::default();
        document.crease_pattern.add_line(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            LineColor::Red1,
        );
        document.crease_pattern.add_line(
            Point::new(5.0, -5.0),
            Point::new(5.0, 5.0),
            LineColor::Blue2,
        );
        document.crease_pattern.add_line(
            Point::new(0.0, 1.0),
            Point::new(10.0, 1.0),
            LineColor::Cyan3,
        );
        document
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {actual} to be within tolerance of {expected}"
        );
    }
}
