//! Research primitives for deriving folding sequences from FOLD crease patterns.
//!
//! This crate intentionally builds on `treemaker-flatfold` instead of
//! reimplementing flat-folding. The current public surface is Phase 1 of the
//! roadmap: resolve a deterministic target folded state and expose enough
//! diagnostics for later planning stages.

use serde::{Deserialize, Serialize};
use treemaker_flatfold::{
    ConstraintSummary, FlatFoldError, OverlapGraph, SolveOptions, solve_flat_fold,
};
use treemaker_fold::{FoldDocument, build_faces_edges, validate_basic};

pub use treemaker_flatfold::SolutionLimit;

pub type Result<T> = std::result::Result<T, SequenceError>;

const ID_MAP_EPSILON: f64 = 1.0e-9;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SequenceError {
    #[error("invalid folding-sequence input: {0}")]
    InvalidInput(String),
    #[error("invalid sequence state: {diagnostics_len} diagnostic(s)")]
    InvalidState { diagnostics_len: usize },
    #[error("target layer order is ambiguous: {states} possible state(s)")]
    AmbiguousLayerOrder { states: String },
    #[error("folding-sequence component is not yet implemented: {0}")]
    NotImplemented(&'static str),
    #[error(transparent)]
    FlatFold(#[from] FlatFoldError),
}

impl SequenceError {
    pub fn code(&self) -> &'static str {
        match self {
            SequenceError::InvalidInput(_) => "invalid_input",
            SequenceError::InvalidState { .. } => "invalid_state",
            SequenceError::AmbiguousLayerOrder { .. } => "ambiguous_layer_order",
            SequenceError::NotImplemented(_) => "not_implemented",
            SequenceError::FlatFold(error) => flatfold_error_code(error),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TargetStateOptions {
    pub solution_limit: SolutionLimit,
    pub starting_face: Option<usize>,
    pub require_unique_layer_order: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetState {
    pub normalized: FoldDocument,
    pub folded_vertices: Vec<[f64; 2]>,
    pub faces_flip: Vec<bool>,
    pub overlap: OverlapGraph,
    pub face_orders: Vec<[i64; 3]>,
    pub selected_solution_index: usize,
    pub states: String,
    pub component_sizes: Vec<usize>,
    pub solution_counts: Vec<usize>,
    pub constraints: ConstraintSummary,
    pub id_map: TargetIdMap,
    pub diagnostics: Vec<SequenceDiagnostic>,
}

impl TargetState {
    pub fn from_fold(document: &FoldDocument, options: TargetStateOptions) -> Result<Self> {
        reject_zero_solution_limit(&options.solution_limit)?;
        let solved = solve_flat_fold(
            document,
            SolveOptions {
                starting_face: options.starting_face,
                solution_limit: options.solution_limit.clone(),
                ..SolveOptions::default()
            },
        )?;
        let overlap = solved.analysis.overlap.clone().ok_or_else(|| {
            SequenceError::InvalidInput(
                "flat-fold target analysis did not include an overlap graph".to_string(),
            )
        })?;
        let ambiguous = solution_counts_are_ambiguous(&solved.solution_counts);
        if options.require_unique_layer_order && ambiguous {
            return Err(SequenceError::AmbiguousLayerOrder {
                states: solved.states,
            });
        }
        let normalized = solved.analysis.normalized.document;
        let id_map = TargetIdMap::from_documents(document, &normalized);
        let mut diagnostics = Vec::new();
        diagnostics.extend(
            solved
                .analysis
                .diagnostics
                .into_iter()
                .map(|diagnostic| SequenceDiagnostic::info(diagnostic.code, diagnostic.message)),
        );
        diagnostics.extend(
            solved
                .diagnostics
                .into_iter()
                .map(|diagnostic| SequenceDiagnostic::info(diagnostic.code, diagnostic.message)),
        );
        if ambiguous {
            diagnostics.push(SequenceDiagnostic::warning(
                "ambiguous_layer_order",
                format!(
                    "flat-fold solver found {} possible layer-order state(s); target state uses deterministic first-solution face orders",
                    solved.states
                ),
            ));
        }
        if solution_limit_may_have_truncated(&options.solution_limit, &solved.solution_counts) {
            diagnostics.push(SequenceDiagnostic::warning(
                "solution_limit_reached",
                "solution limit was reached for at least one layer-order component; ambiguity may be undercounted",
            ));
        }
        Ok(Self {
            normalized,
            folded_vertices: solved.analysis.folded_vertices,
            faces_flip: solved.analysis.faces_flip,
            overlap,
            face_orders: solved.face_orders,
            selected_solution_index: 0,
            states: solved.states,
            component_sizes: solved.component_sizes,
            solution_counts: solved.solution_counts,
            constraints: solved.constraints,
            id_map,
            diagnostics,
        })
    }

    pub fn has_layer_order_ambiguity(&self) -> bool {
        solution_counts_are_ambiguous(&self.solution_counts)
    }

    pub fn normalized_frame(&self) -> TargetFrame {
        TargetFrame {
            document: self.normalized.clone(),
            face_orders: self.face_orders.clone(),
        }
    }

    pub fn folded_frame(&self) -> TargetFrame {
        let mut document = self.normalized.clone();
        document.vertices_coords = self
            .folded_vertices
            .iter()
            .map(|[x, y]| vec![*x, *y])
            .collect();
        if !document
            .frame_classes
            .iter()
            .any(|class| class == "foldedForm")
        {
            document.frame_classes.push("foldedForm".to_string());
        }
        TargetFrame {
            document,
            face_orders: self.face_orders.clone(),
        }
    }
}

pub fn resolve_target_state(
    document: &FoldDocument,
    options: TargetStateOptions,
) -> Result<TargetState> {
    TargetState::from_fold(document, options)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetFrame {
    pub document: FoldDocument,
    pub face_orders: Vec<[i64; 3]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetIdMap {
    pub normalized_vertices_to_input_vertices: Vec<Vec<usize>>,
    pub normalized_edges_to_input_edges: Vec<Vec<usize>>,
    pub normalized_faces_to_input_faces: Vec<Vec<usize>>,
}

impl TargetIdMap {
    pub fn from_documents(input: &FoldDocument, normalized: &FoldDocument) -> Self {
        let normalized_vertices_to_input_vertices = map_vertices(input, normalized);
        let normalized_edges_to_input_edges = map_edges(input, normalized);
        let normalized_faces_to_input_faces =
            map_faces(input, normalized, &normalized_vertices_to_input_vertices);
        Self {
            normalized_vertices_to_input_vertices,
            normalized_edges_to_input_edges,
            normalized_faces_to_input_faces,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
}

impl SequenceDiagnostic {
    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Info,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code: code.into(),
            message: message.into(),
        }
    }
}

pub fn plan_folding_sequence(_target: &TargetState) -> Result<()> {
    Err(SequenceError::NotImplemented("folding sequence planner"))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SequenceState {
    pub id: String,
    pub document: FoldDocument,
    pub active_creases: Vec<usize>,
    pub face_orders: Vec<[i64; 3]>,
    pub folded_vertices: Vec<[f64; 2]>,
    pub unresolved_regions: Vec<UnresolvedRegion>,
    pub provenance: StateProvenance,
    pub layer_order_policy: LayerOrderPolicy,
    pub diagnostics: Vec<SequenceDiagnostic>,
}

impl SequenceState {
    pub fn from_target(id: impl Into<String>, target: &TargetState) -> Self {
        let active_creases = target
            .normalized
            .edges_assignment
            .iter()
            .enumerate()
            .filter_map(|(edge, assignment)| assignment.is_driven_crease().then_some(edge))
            .collect();
        Self {
            id: id.into(),
            document: target.normalized.clone(),
            active_creases,
            face_orders: target.face_orders.clone(),
            folded_vertices: target.folded_vertices.clone(),
            unresolved_regions: Vec::new(),
            provenance: StateProvenance::target_state(target.selected_solution_index),
            layer_order_policy: LayerOrderPolicy::Preserved,
            diagnostics: target.diagnostics.clone(),
        }
    }

    pub fn to_frame(&self) -> TargetFrame {
        TargetFrame {
            document: self.document.clone(),
            face_orders: self.face_orders.clone(),
        }
    }

    pub fn validate(&self) -> Result<ValidationReport> {
        validate_sequence_state(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateProvenance {
    pub source: StateSource,
    pub selected_solution_index: Option<usize>,
    pub predecessor: Option<String>,
    pub step_id: Option<String>,
}

impl StateProvenance {
    pub fn target_state(selected_solution_index: usize) -> Self {
        Self {
            source: StateSource::TargetState,
            selected_solution_index: Some(selected_solution_index),
            predecessor: None,
            step_id: None,
        }
    }

    pub fn rewrite(predecessor: impl Into<String>, step_id: impl Into<String>) -> Self {
        Self {
            source: StateSource::Rewrite,
            selected_solution_index: None,
            predecessor: Some(predecessor.into()),
            step_id: Some(step_id.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateSource {
    Input,
    TargetState,
    Rewrite,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerOrderPolicy {
    Preserved,
    RelaxedWithDiagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedRegion {
    pub id: String,
    pub creases: Vec<usize>,
    pub faces: Vec<usize>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InstructionStep {
    PrecreaseRegion(StepDetails),
    SimpleFold(StepDetails),
    ReverseFold(StepDetails),
    SquashFold(StepDetails),
    RabbitEar(StepDetails),
    MoleculeCollapse(StepDetails),
    SimultaneousCollapse(StepDetails),
    ManualChoice(ManualChoiceStep),
    UnsupportedRegion(UnsupportedRegionStep),
}

impl InstructionStep {
    pub fn id(&self) -> &str {
        match self {
            InstructionStep::PrecreaseRegion(details)
            | InstructionStep::SimpleFold(details)
            | InstructionStep::ReverseFold(details)
            | InstructionStep::SquashFold(details)
            | InstructionStep::RabbitEar(details)
            | InstructionStep::MoleculeCollapse(details)
            | InstructionStep::SimultaneousCollapse(details) => &details.id,
            InstructionStep::ManualChoice(step) => &step.id,
            InstructionStep::UnsupportedRegion(step) => &step.id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepDetails {
    pub id: String,
    pub label: String,
    pub affected_creases: Vec<usize>,
    pub affected_faces: Vec<usize>,
    pub before_state: String,
    pub after_state: String,
    pub metadata: MoveMetadata,
    pub diagnostics: Vec<SequenceDiagnostic>,
}

impl StepDetails {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        before_state: impl Into<String>,
        after_state: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            affected_creases: Vec::new(),
            affected_faces: Vec::new(),
            before_state: before_state.into(),
            after_state: after_state.into(),
            metadata: MoveMetadata::default(),
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MoveMetadata {
    pub difficulty: MoveDifficulty,
    pub layer_mode: LayerMode,
    pub confidence: f64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveDifficulty {
    #[default]
    Unknown,
    Simple,
    Intermediate,
    Complex,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerMode {
    #[default]
    Unknown,
    SingleLayer,
    MultiLayer,
    Simultaneous,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManualChoiceStep {
    pub id: String,
    pub label: String,
    pub before_state: String,
    pub choices: Vec<ManualChoice>,
    pub diagnostics: Vec<SequenceDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManualChoice {
    pub id: String,
    pub label: String,
    pub affected_creases: Vec<usize>,
    pub affected_faces: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnsupportedRegionStep {
    pub id: String,
    pub label: String,
    pub before_state: String,
    pub region: UnresolvedRegion,
    pub diagnostics: Vec<SequenceDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub state_id: String,
    pub diagnostics: Vec<SequenceDiagnostic>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

pub fn inspect_sequence_state(state: &SequenceState) -> ValidationReport {
    let mut diagnostics = Vec::new();

    if let Err(error) = validate_basic(&state.document) {
        diagnostics.push(SequenceDiagnostic::error(
            "invalid_fold_topology",
            error.to_string(),
        ));
    }
    if !state.document.faces_vertices.is_empty()
        && let Err(error) = build_faces_edges(&state.document)
    {
        diagnostics.push(SequenceDiagnostic::error(
            "invalid_face_cycles",
            error.to_string(),
        ));
    }
    let edge_count = state.document.edges_vertices.len();
    for crease in &state.active_creases {
        if *crease >= edge_count {
            diagnostics.push(SequenceDiagnostic::error(
                "active_crease_out_of_bounds",
                format!("active crease {crease} is outside edge range 0..{edge_count}"),
            ));
            continue;
        }
        let assignment = state.document.assignment_for_edge(*crease);
        if !assignment.is_driven_crease() {
            diagnostics.push(SequenceDiagnostic::error(
                "active_crease_not_driven",
                format!(
                    "active crease {crease} has assignment {}; expected M, V, or F",
                    assignment.as_str()
                ),
            ));
        }
    }
    validate_folded_vertices(state, &mut diagnostics);
    validate_face_orders(state, &mut diagnostics);
    validate_unresolved_regions(state, &mut diagnostics);

    if state.layer_order_policy == LayerOrderPolicy::RelaxedWithDiagnostic
        && !state
            .diagnostics
            .iter()
            .chain(diagnostics.iter())
            .any(|diagnostic| diagnostic.code == "layer_order_relaxed")
    {
        diagnostics.push(SequenceDiagnostic::error(
            "missing_layer_order_relaxed_diagnostic",
            "layer_order_policy is relaxed, but no layer_order_relaxed diagnostic explains why",
        ));
    }

    ValidationReport {
        state_id: state.id.clone(),
        diagnostics,
    }
}

pub fn validate_sequence_state(state: &SequenceState) -> Result<ValidationReport> {
    let report = inspect_sequence_state(state);
    if !report.is_valid() {
        return Err(SequenceError::InvalidState {
            diagnostics_len: report.diagnostics.len(),
        });
    }
    Ok(report)
}

fn reject_zero_solution_limit(solution_limit: &SolutionLimit) -> Result<()> {
    if matches!(solution_limit, SolutionLimit::Count(0)) {
        return Err(SequenceError::InvalidInput(
            "solution_limit must be positive".to_string(),
        ));
    }
    Ok(())
}

fn validate_folded_vertices(state: &SequenceState, diagnostics: &mut Vec<SequenceDiagnostic>) {
    let vertex_count = state.document.vertices_coords.len();
    if state.folded_vertices.len() != vertex_count {
        diagnostics.push(SequenceDiagnostic::error(
            "folded_vertices_length",
            format!(
                "folded_vertices length {} does not match document vertex count {vertex_count}",
                state.folded_vertices.len()
            ),
        ));
        return;
    }
    for (index, [x, y]) in state.folded_vertices.iter().enumerate() {
        if !x.is_finite() || !y.is_finite() {
            diagnostics.push(SequenceDiagnostic::error(
                "non_finite_folded_vertex",
                format!("folded vertex {index} contains a non-finite coordinate"),
            ));
        }
    }
}

fn validate_face_orders(state: &SequenceState, diagnostics: &mut Vec<SequenceDiagnostic>) {
    let face_count = state.document.faces_vertices.len();
    for (index, [above, below, orientation]) in state.face_orders.iter().enumerate() {
        let Ok(above) = usize::try_from(*above) else {
            diagnostics.push(SequenceDiagnostic::error(
                "face_order_out_of_bounds",
                format!("face order {index} has negative above face {above}"),
            ));
            continue;
        };
        let Ok(below) = usize::try_from(*below) else {
            diagnostics.push(SequenceDiagnostic::error(
                "face_order_out_of_bounds",
                format!("face order {index} has negative below face {below}"),
            ));
            continue;
        };
        if above >= face_count || below >= face_count {
            diagnostics.push(SequenceDiagnostic::error(
                "face_order_out_of_bounds",
                format!(
                    "face order {index} references faces {above} and {below}; valid range is 0..{face_count}"
                ),
            ));
        }
        if !matches!(*orientation, -1 | 1) {
            diagnostics.push(SequenceDiagnostic::error(
                "face_order_orientation",
                format!("face order {index} orientation must be -1 or 1, got {orientation}"),
            ));
        }
    }
}

fn validate_unresolved_regions(state: &SequenceState, diagnostics: &mut Vec<SequenceDiagnostic>) {
    let edge_count = state.document.edges_vertices.len();
    let face_count = state.document.faces_vertices.len();
    for region in &state.unresolved_regions {
        for crease in &region.creases {
            if *crease >= edge_count {
                diagnostics.push(SequenceDiagnostic::error(
                    "unresolved_crease_out_of_bounds",
                    format!(
                        "unresolved region {} references crease {crease}; valid range is 0..{edge_count}",
                        region.id
                    ),
                ));
            }
        }
        for face in &region.faces {
            if *face >= face_count {
                diagnostics.push(SequenceDiagnostic::error(
                    "unresolved_face_out_of_bounds",
                    format!(
                        "unresolved region {} references face {face}; valid range is 0..{face_count}",
                        region.id
                    ),
                ));
            }
        }
        if region.reason.trim().is_empty() {
            diagnostics.push(SequenceDiagnostic::error(
                "unresolved_region_missing_reason",
                format!("unresolved region {} must include a reason", region.id),
            ));
        }
    }
}

fn solution_counts_are_ambiguous(solution_counts: &[usize]) -> bool {
    solution_counts.iter().any(|count| *count > 1)
}

fn solution_limit_may_have_truncated(limit: &SolutionLimit, solution_counts: &[usize]) -> bool {
    match limit {
        SolutionLimit::All => false,
        SolutionLimit::Count(limit) => solution_counts.iter().any(|count| count == limit),
    }
}

fn flatfold_error_code(error: &FlatFoldError) -> &'static str {
    match error {
        FlatFoldError::InvalidInput(_) => "invalid_input",
        FlatFoldError::PrecisionFailure(_) => "precision_failure",
        FlatFoldError::AssignmentConflict(_) => "assignment_conflict",
        FlatFoldError::UnsatisfiedComponent(_) => "unsatisfied_component",
        FlatFoldError::Unimplemented(_) => "not_implemented",
    }
}

fn map_vertices(input: &FoldDocument, normalized: &FoldDocument) -> Vec<Vec<usize>> {
    normalized
        .vertices_coords
        .iter()
        .map(|normalized_vertex| {
            input
                .vertices_coords
                .iter()
                .enumerate()
                .filter_map(|(input_index, input_vertex)| {
                    points_close(normalized_vertex, input_vertex).then_some(input_index)
                })
                .collect()
        })
        .collect()
}

fn map_edges(input: &FoldDocument, normalized: &FoldDocument) -> Vec<Vec<usize>> {
    normalized
        .edges_vertices
        .iter()
        .map(|[normalized_a, normalized_b]| {
            let Some(a) = point_at(normalized, *normalized_a) else {
                return Vec::new();
            };
            let Some(b) = point_at(normalized, *normalized_b) else {
                return Vec::new();
            };
            input
                .edges_vertices
                .iter()
                .enumerate()
                .filter_map(|(input_edge_index, [input_a, input_b])| {
                    let source_a = point_at(input, *input_a)?;
                    let source_b = point_at(input, *input_b)?;
                    normalized_edge_is_on_input_edge(a, b, source_a, source_b)
                        .then_some(input_edge_index)
                })
                .collect()
        })
        .collect()
}

fn map_faces(
    input: &FoldDocument,
    normalized: &FoldDocument,
    vertex_map: &[Vec<usize>],
) -> Vec<Vec<usize>> {
    normalized
        .faces_vertices
        .iter()
        .map(|normalized_face| {
            input
                .faces_vertices
                .iter()
                .enumerate()
                .filter_map(|(input_face_index, input_face)| {
                    face_vertices_map_to_input_face(normalized_face, input_face, vertex_map)
                        .then_some(input_face_index)
                })
                .collect()
        })
        .collect()
}

fn face_vertices_map_to_input_face(
    normalized_face: &[usize],
    input_face: &[usize],
    vertex_map: &[Vec<usize>],
) -> bool {
    if normalized_face.len() != input_face.len() {
        return false;
    }
    normalized_face.iter().all(|vertex| {
        vertex_map.get(*vertex).is_some_and(|source_vertices| {
            source_vertices
                .iter()
                .any(|source| input_face.contains(source))
        })
    }) && input_face.iter().all(|source| {
        normalized_face.iter().any(|vertex| {
            vertex_map
                .get(*vertex)
                .is_some_and(|source_vertices| source_vertices.contains(source))
        })
    })
}

fn normalized_edge_is_on_input_edge(
    normalized_a: [f64; 2],
    normalized_b: [f64; 2],
    input_a: [f64; 2],
    input_b: [f64; 2],
) -> bool {
    point_on_segment(normalized_a, input_a, input_b)
        && point_on_segment(normalized_b, input_a, input_b)
}

fn point_at(document: &FoldDocument, vertex: usize) -> Option<[f64; 2]> {
    let coords = document.vertices_coords.get(vertex)?;
    Some([*coords.first()?, *coords.get(1)?])
}

fn points_close(a: &[f64], b: &[f64]) -> bool {
    let (Some(ax), Some(ay), Some(bx), Some(by)) = (a.first(), a.get(1), b.first(), b.get(1))
    else {
        return false;
    };
    (*ax - *bx).abs() <= ID_MAP_EPSILON && (*ay - *by).abs() <= ID_MAP_EPSILON
}

fn point_on_segment(point: [f64; 2], a: [f64; 2], b: [f64; 2]) -> bool {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [point[0] - a[0], point[1] - a[1]];
    let length_sq = ab[0] * ab[0] + ab[1] * ab[1];
    if length_sq <= ID_MAP_EPSILON {
        return false;
    }
    let cross = (ab[0] * ap[1] - ab[1] * ap[0]).abs();
    let scale = length_sq.sqrt().max(1.0);
    if cross > ID_MAP_EPSILON * scale {
        return false;
    }
    let dot = ap[0] * ab[0] + ap[1] * ab[1];
    dot >= -ID_MAP_EPSILON && dot <= length_sq + ID_MAP_EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;
    use treemaker_fold::Assignment;

    fn two_face_valley() -> FoldDocument {
        let mut document = FoldDocument::new(
            vec![
                vec![0.0, 0.0],
                vec![1.0, 0.0],
                vec![1.0, 1.0],
                vec![0.0, 1.0],
            ],
            vec![[0, 1], [1, 2], [2, 3], [3, 0], [0, 2]],
        );
        document.edges_assignment = vec![
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Valley,
        ];
        document.faces_vertices = vec![vec![0, 1, 2], vec![0, 2, 3]];
        document
    }

    #[test]
    fn target_state_wraps_flatfold_result() {
        let target = resolve_target_state(&two_face_valley(), TargetStateOptions::default())
            .expect("target state");

        assert_eq!(
            target.folded_vertices.len(),
            target.normalized.vertices_coords.len()
        );
        assert_eq!(
            target.faces_flip.len(),
            target.normalized.faces_vertices.len()
        );
        assert_eq!(target.selected_solution_index, 0);
        assert!(!target.overlap.cells_faces.is_empty());
        assert_eq!(
            target.id_map.normalized_vertices_to_input_vertices.len(),
            target.normalized.vertices_coords.len()
        );
    }

    #[test]
    fn folded_frame_uses_folded_coordinates_without_losing_face_orders() {
        let target = resolve_target_state(&two_face_valley(), TargetStateOptions::default())
            .expect("target state");
        let folded = target.folded_frame();

        assert_eq!(
            folded.document.vertices_coords.len(),
            target.folded_vertices.len()
        );
        assert_eq!(folded.face_orders, target.face_orders);
        assert!(
            folded
                .document
                .frame_classes
                .iter()
                .any(|class| class == "foldedForm")
        );
    }

    #[test]
    fn unfinished_planner_is_explicitly_not_implemented() {
        let target = resolve_target_state(&two_face_valley(), TargetStateOptions::default())
            .expect("target state");
        let error = plan_folding_sequence(&target).expect_err("planner is not implemented");

        assert_eq!(error.code(), "not_implemented");
    }

    #[test]
    fn zero_solution_limit_is_rejected_before_flatfolding() {
        let error = resolve_target_state(
            &two_face_valley(),
            TargetStateOptions {
                solution_limit: SolutionLimit::Count(0),
                ..TargetStateOptions::default()
            },
        )
        .expect_err("zero limit should be invalid");

        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn sequence_state_from_target_validates() {
        let target = resolve_target_state(&two_face_valley(), TargetStateOptions::default())
            .expect("target state");
        let state = SequenceState::from_target("target", &target);
        let report = state.validate().expect("state validates");

        assert_eq!(report.state_id, "target");
        assert!(report.diagnostics.is_empty());
        assert_eq!(state.to_frame().face_orders, target.face_orders);
    }

    #[test]
    fn sequence_state_validator_reports_bad_active_crease() {
        let target = resolve_target_state(&two_face_valley(), TargetStateOptions::default())
            .expect("target state");
        let mut state = SequenceState::from_target("bad-active", &target);
        state.active_creases.push(0);
        state
            .active_creases
            .push(state.document.edges_vertices.len());
        let report = inspect_sequence_state(&state);

        assert!(!report.is_valid());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "active_crease_not_driven")
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "active_crease_out_of_bounds")
        );
        assert_eq!(
            state.validate().expect_err("state should fail").code(),
            "invalid_state"
        );
    }

    #[test]
    fn sequence_state_validator_reports_layer_order_errors() {
        let target = resolve_target_state(&two_face_valley(), TargetStateOptions::default())
            .expect("target state");
        let mut state = SequenceState::from_target("bad-order", &target);
        state.face_orders = vec![[0, 99, 0]];
        state.layer_order_policy = LayerOrderPolicy::RelaxedWithDiagnostic;
        let report = inspect_sequence_state(&state);

        assert!(!report.is_valid());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "face_order_out_of_bounds")
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "face_order_orientation")
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "missing_layer_order_relaxed_diagnostic")
        );
    }

    #[test]
    fn instruction_steps_serialize_with_stable_kind_names() {
        let mut details = StepDetails::new("step-1", "Fold along the diagonal", "s0", "s1");
        details.affected_creases = vec![4];
        details.metadata.difficulty = MoveDifficulty::Simple;
        details.metadata.layer_mode = LayerMode::SingleLayer;
        details.metadata.confidence = 1.0;
        let step = InstructionStep::SimpleFold(details);
        let value = serde_json::to_value(&step).expect("serialize step");

        assert_eq!(value["kind"], "simple_fold");
        assert_eq!(step.id(), "step-1");
    }
}
