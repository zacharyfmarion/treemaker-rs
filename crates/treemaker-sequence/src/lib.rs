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
use treemaker_fold::FoldDocument;

pub use treemaker_flatfold::SolutionLimit;

pub type Result<T> = std::result::Result<T, SequenceError>;

const ID_MAP_EPSILON: f64 = 1.0e-9;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SequenceError {
    #[error("invalid folding-sequence input: {0}")]
    InvalidInput(String),
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

fn reject_zero_solution_limit(solution_limit: &SolutionLimit) -> Result<()> {
    if matches!(solution_limit, SolutionLimit::Count(0)) {
        return Err(SequenceError::InvalidInput(
            "solution_limit must be positive".to_string(),
        ));
    }
    Ok(())
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
}
