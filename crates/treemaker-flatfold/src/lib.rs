//! Flat-Folder-derived flat-foldability and layer-order solver.
//!
//! This crate is intentionally independent from TreeMaker's tree engine. It
//! exposes a stage-oriented API so the Rust port can be validated against the
//! original Flat-Folder implementation without substituting approximate
//! algorithms while the port is in progress.

mod avl;
mod constraints;
mod conversion;
mod math;

use conversion::{build_overlap_graph, normalize_document, project_normalized};
use serde::{Deserialize, Serialize};
use treemaker_fold::FoldDocument;

pub type Result<T> = std::result::Result<T, FlatFoldError>;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum FlatFoldError {
    #[error("invalid FOLD input: {0}")]
    InvalidInput(String),
    #[error("flat-folder precision failure: {0}")]
    PrecisionFailure(String),
    #[error("assignment conflict: {0}")]
    AssignmentConflict(String),
    #[error("unsatisfied component: {0}")]
    UnsatisfiedComponent(String),
    #[error("flat-folder stage is not yet implemented: {0}")]
    Unimplemented(&'static str),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperSide {
    #[default]
    Front,
    Back,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SolutionLimit {
    All,
    Count(usize),
}

impl Default for SolutionLimit {
    fn default() -> Self {
        Self::Count(10)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum EpsilonPolicy {
    #[default]
    FlatFolderDefault,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NormalizeOptions {
    pub side: PaperSide,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalyzeOptions {
    pub normalize: NormalizeOptions,
    pub epsilon_policy: EpsilonPolicy,
    pub include_overlap_graph: bool,
}

impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            normalize: NormalizeOptions::default(),
            epsilon_policy: EpsilonPolicy::default(),
            include_overlap_graph: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SolveOptions {
    pub analyze: AnalyzeOptions,
    pub starting_face: Option<usize>,
    pub solution_limit: SolutionLimit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedFold {
    pub document: FoldDocument,
    pub vertex_vertices: Vec<Vec<usize>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    pub normalized: NormalizedFold,
    pub folded_vertices: Vec<[f64; 2]>,
    pub faces_flip: Vec<bool>,
    pub overlap: Option<OverlapGraph>,
    pub diagnostics: Vec<FlatFoldDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverlapGraph {
    pub points: Vec<[f64; 2]>,
    pub segments_points: Vec<[usize; 2]>,
    pub segments_edges: Vec<Vec<usize>>,
    pub segments_cells: Vec<Vec<usize>>,
    pub cells_segments: Vec<Vec<usize>>,
    pub cells_points: Vec<Vec<usize>>,
    pub cells_faces: Vec<Vec<usize>>,
    pub faces_cells: Vec<Vec<usize>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolveResult {
    pub analysis: Analysis,
    pub component_sizes: Vec<usize>,
    pub solution_counts: Vec<usize>,
    pub face_orders: Vec<[usize; 3]>,
    pub diagnostics: Vec<FlatFoldDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlatFoldDiagnostic {
    pub code: String,
    pub message: String,
}

pub fn normalize_fold(
    document: &FoldDocument,
    options: NormalizeOptions,
) -> Result<NormalizedFold> {
    normalize_document(document, options)
}

pub fn analyze_flat_fold(document: &FoldDocument, options: AnalyzeOptions) -> Result<Analysis> {
    let normalized = normalize_fold(document, options.normalize)?;
    let (folded_vertices, faces_flip) = project_normalized(&normalized)?;
    let overlap = if options.include_overlap_graph {
        Some(build_overlap_graph(&normalized, &folded_vertices)?)
    } else {
        None
    };
    Ok(Analysis {
        normalized,
        folded_vertices,
        faces_flip,
        overlap,
        diagnostics: Vec::new(),
    })
}

pub fn solve_flat_fold(_document: &FoldDocument, _options: SolveOptions) -> Result<SolveResult> {
    Err(FlatFoldError::Unimplemented("solve_flat_fold"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use treemaker_fold::Assignment;

    fn square_doc() -> FoldDocument {
        FoldDocument::new(
            vec![
                vec![0.0, 0.0],
                vec![1.0, 0.0],
                vec![1.0, 1.0],
                vec![0.0, 1.0],
            ],
            vec![[0, 1], [1, 2], [2, 3], [3, 0]],
        )
    }

    #[test]
    fn unported_analysis_stages_return_explicit_unimplemented_errors() {
        let doc = square_doc();
        assert_eq!(
            solve_flat_fold(&doc, SolveOptions::default()),
            Err(FlatFoldError::Unimplemented("solve_flat_fold"))
        );
    }

    #[test]
    fn normalize_defaults_missing_assignments_to_unassigned() {
        let doc = FoldDocument::new(
            vec![
                vec![0.0, 0.0],
                vec![1.0, 0.0],
                vec![1.0, 1.0],
                vec![0.0, 1.0],
            ],
            vec![[0, 1], [1, 2], [2, 3], [3, 0]],
        );

        let normalized = normalize_fold(&doc, NormalizeOptions::default()).unwrap();

        assert_eq!(normalized.document.vertices_coords.len(), 4);
        assert_eq!(normalized.document.edges_vertices.len(), 4);
        assert_eq!(normalized.document.faces_vertices.len(), 1);
        assert_eq!(
            normalized.document.edges_assignment,
            vec![Assignment::Boundary; 4]
        );
    }

    #[test]
    fn normalize_flips_faceless_front_side_mountain_valley_assignments() {
        let mut doc = FoldDocument::new(
            vec![
                vec![0.0, 0.0],
                vec![1.0, 0.0],
                vec![1.0, 1.0],
                vec![0.0, 1.0],
                vec![0.0, 0.5],
                vec![1.0, 0.5],
            ],
            vec![[0, 1], [1, 2], [2, 3], [3, 0], [4, 5]],
        );
        doc.edges_assignment = vec![
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Mountain,
        ];

        let normalized = normalize_fold(&doc, NormalizeOptions::default()).unwrap();

        assert_eq!(normalized.document.faces_vertices.len(), 2);
        assert_eq!(
            normalized
                .document
                .edges_assignment
                .iter()
                .filter(|assignment| **assignment == Assignment::Valley)
                .count(),
            1
        );
        assert!(
            !normalized
                .document
                .edges_assignment
                .contains(&Assignment::Mountain)
        );
    }

    #[test]
    fn normalize_splits_crossing_and_vertex_on_segment_lines() {
        let mut doc = FoldDocument::new(
            vec![
                vec![0.0, 0.0],
                vec![1.0, 0.0],
                vec![1.0, 1.0],
                vec![0.0, 1.0],
                vec![0.0, 0.5],
                vec![1.0, 0.5],
                vec![0.5, 0.0],
                vec![0.5, 1.0],
            ],
            vec![[0, 1], [1, 2], [2, 3], [3, 0], [4, 5], [6, 7]],
        );
        doc.edges_assignment = vec![
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Mountain,
            Assignment::Valley,
        ];

        let normalized = normalize_fold(&doc, NormalizeOptions::default()).unwrap();

        assert_eq!(normalized.document.vertices_coords.len(), 9);
        assert_eq!(normalized.document.edges_vertices.len(), 12);
        assert_eq!(normalized.document.faces_vertices.len(), 4);
        assert_eq!(
            normalized
                .document
                .edges_assignment
                .iter()
                .filter(|assignment| **assignment == Assignment::Boundary)
                .count(),
            8
        );
        assert_eq!(
            normalized
                .document
                .edges_assignment
                .iter()
                .filter(|assignment| **assignment == Assignment::Mountain)
                .count(),
            2
        );
        assert_eq!(
            normalized
                .document
                .edges_assignment
                .iter()
                .filter(|assignment| **assignment == Assignment::Valley)
                .count(),
            2
        );
    }

    #[test]
    fn analyze_projects_flat_fold_when_overlap_graph_is_not_requested() {
        let mut doc = FoldDocument::new(
            vec![
                vec![0.0, 0.0],
                vec![1.0, 0.0],
                vec![1.0, 1.0],
                vec![0.0, 1.0],
            ],
            vec![[0, 1], [1, 2], [2, 3], [3, 0], [0, 2]],
        );
        doc.edges_assignment = vec![
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Mountain,
        ];
        doc.faces_vertices = vec![vec![0, 1, 2], vec![0, 2, 3]];

        let analysis = analyze_flat_fold(
            &doc,
            AnalyzeOptions {
                include_overlap_graph: false,
                ..AnalyzeOptions::default()
            },
        )
        .unwrap();

        assert_eq!(analysis.folded_vertices.len(), 4);
        assert_eq!(analysis.faces_flip, vec![false, true]);
        assert!(analysis.overlap.is_none());
    }

    #[test]
    fn analyze_builds_overlap_graph_by_default() {
        let mut doc = FoldDocument::new(
            vec![
                vec![0.0, 0.0],
                vec![1.0, 0.0],
                vec![1.0, 1.0],
                vec![0.0, 1.0],
            ],
            vec![[0, 1], [1, 2], [2, 3], [3, 0], [0, 2]],
        );
        doc.edges_assignment = vec![
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Mountain,
        ];
        doc.faces_vertices = vec![vec![0, 1, 2], vec![0, 2, 3]];

        let analysis = analyze_flat_fold(&doc, AnalyzeOptions::default()).unwrap();
        let overlap = analysis.overlap.unwrap();

        assert!(!overlap.points.is_empty());
        assert!(!overlap.segments_points.is_empty());
        assert_eq!(
            overlap.faces_cells.len(),
            analysis.normalized.document.faces_vertices.len()
        );
    }

    #[test]
    fn constraint_counts_match_flat_folder_kabuto_fixture() {
        let document: FoldDocument = serde_json::from_str(include_str!(
            "../../../tests/fixtures/flat-folder/kabuto.fold"
        ))
        .unwrap();
        let analysis = analyze_flat_fold(&document, AnalyzeOptions::default()).unwrap();
        let constraints = crate::constraints::build_constraint_state(&analysis).unwrap();

        assert_eq!(constraints.variables.len(), 117);
        assert_eq!(constraints.constraint_counts.taco_taco, 21);
        assert_eq!(constraints.constraint_counts.taco_tortilla, 88);
        assert_eq!(constraints.constraint_counts.tortilla_tortilla, 0);
        assert_eq!(constraints.transitivity_counts.all / 3, 420);
        assert_eq!(constraints.transitivity_counts.reduced / 3, 284);
        assert_eq!(constraints.groups.len(), 3);
        assert_eq!(
            constraints.groups.iter().map(Vec::len).collect::<Vec<_>>(),
            vec![81, 18, 18]
        );
    }
}
