//! Oriedita-compatible crease-pattern editing kernel for Ori Studio.
//!
//! This crate is intentionally conservative while the port is in progress.
//! Every known non-UI Oriedita operation is represented in the registry, but
//! unsupported operations fail with a typed error instead of fabricating nearby
//! behavior.

pub mod canonical;
pub mod geometry;
pub mod io;
pub mod model;
pub mod operations;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use canonical::CanonicalCreasePattern;
pub use model::CreasePatternModel;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreasePatternCommand {
    /// Oriedita operation represented by this command.
    pub operation: OperationId,
}

impl CreasePatternCommand {
    /// Create a command for an Oriedita operation.
    pub const fn new(operation: OperationId) -> Self {
        Self { operation }
    }
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
        "operations::construction::draw_free",
        Kernel,
        7,
        Unsupported
    ),
    descriptor!(
        MoveCreasePattern,
        "MouseHandlerMoveCreasePattern",
        "operations::transform::move_document",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        LineSegmentDelete,
        "MouseHandlerLineSegmentDelete",
        "operations::arrangement::delete_line",
        Kernel,
        5,
        Unsupported
    ),
    descriptor!(
        ChangeCreaseType,
        "MouseHandlerChangeCreaseType",
        "operations::color::change_type",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        LengthenCrease,
        "MouseHandlerLengthenCrease",
        "operations::transform::lengthen_crease",
        Kernel,
        6,
        Porting
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
        "operations::construction::draw_restricted",
        Kernel,
        7,
        Unsupported
    ),
    descriptor!(
        DrawCreaseSymmetric,
        "MouseHandlerDrawCreaseSymmetric",
        "operations::construction::draw_symmetric",
        Kernel,
        7,
        Unsupported
    ),
    descriptor!(
        DrawCreaseAngleRestricted,
        "MouseHandlerDrawCreaseAngleRestricted",
        "operations::construction::angle_restricted",
        Kernel,
        7,
        Unsupported
    ),
    descriptor!(
        DrawPoint,
        "MouseHandlerDrawPoint",
        "operations::point::draw_point",
        Kernel,
        7,
        Unsupported
    ),
    descriptor!(
        DeletePoint,
        "MouseHandlerDeletePoint",
        "operations::point::delete_point",
        Kernel,
        5,
        Unsupported
    ),
    descriptor!(
        AngleSystem,
        "MouseHandlerAngleSystem",
        "operations::construction::angle_system",
        Kernel,
        7,
        Unsupported
    ),
    descriptor!(
        DrawCreaseAngleRestricted3,
        "MouseHandlerDrawCreaseAngleRestricted3_2",
        "operations::construction::angle_restricted_3",
        Kernel,
        7,
        Unsupported
    ),
    descriptor!(
        CreaseSelect,
        "MouseHandlerCreaseSelect",
        "operations::selection::select_line",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        CreaseUnselect,
        "MouseHandlerCreaseUnselect",
        "operations::selection::unselect_line",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        CreaseMove,
        "MouseHandlerCreaseMove",
        "operations::transform::move_selection",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        CreaseCopy,
        "MouseHandlerCreaseCopy",
        "operations::transform::copy_selection",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        CreaseMakeMountain,
        "MouseHandlerCreaseMakeMountain",
        "operations::color::make_mountain",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        CreaseMakeValley,
        "MouseHandlerCreaseMakeValley",
        "operations::color::make_valley",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        CreaseMakeEdge,
        "MouseHandlerCreaseMakeEdge",
        "operations::color::make_edge",
        Kernel,
        6,
        Unsupported
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
        "operations::point::divide_segment",
        Kernel,
        7,
        Unsupported
    ),
    descriptor!(
        LineSegmentRatioSet,
        "MouseHandlerLineSegmentRatioSet",
        "operations::point::ratio_divide_segment",
        Kernel,
        7,
        Unsupported
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
        "operations::color::advance_type",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        CreaseMove4p,
        "MouseHandlerCreaseMove4p",
        "operations::transform::move_4p",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        CreaseCopy4p,
        "MouseHandlerCreaseCopy4p",
        "operations::transform::copy_4p",
        Kernel,
        6,
        Unsupported
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
        "operations::color::make_mountain_valley",
        Kernel,
        6,
        Unsupported
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
        "operations::color::alternate_mountain_valley",
        Kernel,
        6,
        Unsupported
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
        "operations::construction::make_vertex_flat_foldable",
        Kernel,
        7,
        Unsupported
    ),
    descriptor!(
        FoldableLineInput,
        "MouseHandlerFoldableLineInput",
        "operations::construction::foldable_line_input",
        Kernel,
        7,
        Unsupported
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
        Unsupported
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
        Porting
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
        "operations::construction::continuous_symmetric",
        Kernel,
        7,
        Unsupported
    ),
    descriptor!(
        DisplayLengthBetweenPoints1,
        "MouseHandlerDisplayLengthBetweenPoints",
        "operations::measure::length_between_points",
        KernelPreview,
        7,
        Unsupported
    ),
    descriptor!(
        DisplayLengthBetweenPoints2,
        "MouseHandlerDisplayLengthBetweenPoints",
        "operations::measure::length_between_points",
        KernelPreview,
        7,
        Unsupported
    ),
    descriptor!(
        DisplayAngleBetweenThreePoints1,
        "MouseHandlerDisplayAngleBetweenThreePoints",
        "operations::measure::angle_between_points",
        KernelPreview,
        7,
        Unsupported
    ),
    descriptor!(
        DisplayAngleBetweenThreePoints2,
        "MouseHandlerDisplayAngleBetweenThreePoints",
        "operations::measure::angle_between_points",
        KernelPreview,
        7,
        Unsupported
    ),
    descriptor!(
        DisplayAngleBetweenThreePoints3,
        "MouseHandlerDisplayAngleBetweenThreePoints",
        "operations::measure::angle_between_points",
        KernelPreview,
        7,
        Unsupported
    ),
    descriptor!(
        CreaseToggleMv,
        "MouseHandlerCreaseToggleMV",
        "operations::color::toggle_mountain_valley",
        Kernel,
        6,
        Unsupported
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
        Unsupported
    ),
    descriptor!(
        OperationFrameCreate,
        "MouseHandlerOperationFrameCreate",
        "operations::transform::operation_frame",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        VoronoiCreate,
        "MouseHandlerVoronoiCreate",
        "operations::generators::voronoi",
        Kernel,
        8,
        Unsupported
    ),
    descriptor!(
        FlatFoldableCheck,
        "MouseHandlerFlatFoldableCheck",
        "checks::flat_foldable",
        Kernel,
        9,
        Unsupported
    ),
    descriptor!(
        CreaseDeleteOverlapping,
        "MouseHandlerCreaseDeleteOverlapping",
        "operations::arrangement::delete_overlapping",
        Kernel,
        5,
        Unsupported
    ),
    descriptor!(
        CreaseDeleteIntersecting,
        "MouseHandlerCreaseDeleteIntersecting",
        "operations::arrangement::delete_intersecting",
        Kernel,
        5,
        Unsupported
    ),
    descriptor!(
        SelectPolygon,
        "MouseHandlerSelectPolygon",
        "operations::selection::select_polygon",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        UnselectPolygon,
        "MouseHandlerUnselectPolygon",
        "operations::selection::unselect_polygon",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        SelectLineIntersecting,
        "MouseHandlerSelectLineIntersecting",
        "operations::selection::select_intersecting_line",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        UnselectLineIntersecting,
        "MouseHandlerUnselectLineIntersecting",
        "operations::selection::unselect_intersecting_line",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        LengthenCreaseSameColor,
        "MouseHandlerLengthenCreaseSameColor",
        "operations::transform::lengthen_crease",
        Kernel,
        6,
        Porting
    ),
    descriptor!(
        FoldableLineDraw,
        "MouseHandlerFoldableLineDraw",
        "operations::construction::foldable_line_draw",
        Kernel,
        7,
        Unsupported
    ),
    descriptor!(
        ReplaceLineTypeSelect,
        "MouseHandlerReplaceTypeSelect",
        "operations::color::replace_selected_type",
        Kernel,
        6,
        Unsupported
    ),
    descriptor!(
        DeleteLineTypeSelect,
        "MouseHandlerDeleteTypeSelect",
        "operations::color::delete_selected_type",
        Kernel,
        6,
        Unsupported
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
        "model::text",
        Kernel,
        3,
        Unsupported
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
        "operations::construction::axiom5",
        Kernel,
        7,
        Unsupported
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
        "checks::fix_inaccurate",
        Kernel,
        9,
        Unsupported
    ),
    descriptor!(ImportCp, "CpImporter", "io::cp::import", Io, 4, Unsupported),
    descriptor!(ExportCp, "CpExporter", "io::cp::export", Io, 4, Unsupported),
    descriptor!(
        ImportFold,
        "FoldImporter",
        "io::fold::import",
        Io,
        4,
        Unsupported
    ),
    descriptor!(
        ExportFold,
        "FoldExporter",
        "io::fold::export",
        Io,
        4,
        Unsupported
    ),
    descriptor!(
        ImportOri,
        "OriImporter",
        "io::ori::import",
        Io,
        4,
        Unsupported
    ),
    descriptor!(
        ExportOri,
        "OriExporter",
        "io::ori::export",
        Io,
        4,
        Unsupported
    ),
    descriptor!(
        ImportOrh,
        "OrhImporter",
        "io::orh::import",
        Io,
        4,
        Unsupported
    ),
    descriptor!(
        ExportOrh,
        "OrhExporter",
        "io::orh::export",
        Io,
        4,
        Unsupported
    ),
    descriptor!(
        ImportObj,
        "ObjImporter",
        "io::obj::import",
        Io,
        4,
        Unsupported
    ),
    descriptor!(
        ExportDxf,
        "DxfExporter",
        "io::dxf::export",
        Io,
        4,
        Unsupported
    ),
    descriptor!(
        SaveConvert,
        "SaveConverter",
        "io::save::convert",
        Io,
        4,
        Unsupported
    ),
    descriptor!(
        SaveVersionDetect,
        "FileVersionTester",
        "io::save::version",
        Io,
        4,
        Unsupported
    ),
    descriptor!(
        CheckCamv,
        "CheckCAMVTask",
        "checks::camv",
        Kernel,
        9,
        Unsupported
    ),
    descriptor!(
        FoldingEstimate,
        "FoldingEstimateTask",
        "folding::estimate",
        Kernel,
        10,
        Unsupported
    ),
    descriptor!(
        FoldingEstimateSpecific,
        "FoldingEstimateSpecificTask",
        "folding::estimate_specific",
        Kernel,
        10,
        Unsupported
    ),
    descriptor!(
        FoldingEstimateSave100,
        "FoldingEstimateSave100Task",
        "folding::estimate_batch",
        Kernel,
        10,
        Unsupported
    ),
    descriptor!(
        TwoColoredCp,
        "TwoColoredTask",
        "folding::two_colored",
        Kernel,
        10,
        Unsupported
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
        "folding::commands::fold_another",
        KernelIntent,
        10,
        Unsupported
    ),
    descriptor!(
        DuplicateFoldedModel,
        "FoldingServiceImpl.duplicate",
        "folding::commands::duplicate",
        KernelIntent,
        10,
        Unsupported
    ),
    descriptor!(Check1, "Check1", "checks::check1", Kernel, 9, Unsupported),
    descriptor!(Check2, "Check2", "checks::check2", Kernel, 9, Unsupported),
    descriptor!(Check3, "Check3", "checks::check3", Kernel, 9, Unsupported),
    descriptor!(Check4, "Check4", "checks::check4", Kernel, 9, Unsupported),
    descriptor!(Fix1, "Fix1", "checks::fix1", Kernel, 9, Unsupported),
    descriptor!(Fix2, "Fix2", "checks::fix2", Kernel, 9, Unsupported),
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
///
/// Stage 1 deliberately refuses all registered operations. Later stages replace
/// individual registry entries and dispatch branches only after unit and oracle
/// coverage exists.
pub fn execute_command(
    _document: &mut CreasePatternDocument,
    command: CreasePatternCommand,
) -> Result<CommandResult> {
    match operation_status(command.operation) {
        OperationStatus::Unsupported | OperationStatus::OutOfScopeUi => {
            Err(CommandError::UnsupportedOperation {
                operation: command.operation,
            })
        }
        OperationStatus::Porting
        | OperationStatus::UnitTested
        | OperationStatus::OracleTested
        | OperationStatus::DocumentedDifference => Err(CommandError::NotImplemented {
            operation: command.operation,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            Some("operations::construction::draw_free")
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

        let error = execute_command(
            &mut document,
            CreasePatternCommand::new(OperationId::DrawCreaseFree),
        )
        .expect_err("stage one operations should be unsupported");

        assert_eq!(
            error,
            CommandError::UnsupportedOperation {
                operation: OperationId::DrawCreaseFree,
            }
        );
        assert_eq!(document, original);
    }
}
