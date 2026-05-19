//! Flat-Folder-derived flat-foldability and layer-order solver.
//!
//! This crate is intentionally independent from TreeMaker's tree engine. It
//! exposes a stage-oriented API so the Rust port can be validated against the
//! original Flat-Folder implementation without substituting approximate
//! algorithms while the port is in progress.

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperSide {
    Front,
    Back,
}

impl Default for PaperSide {
    fn default() -> Self {
        Self::Front
    }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EpsilonPolicy {
    FlatFolderDefault,
}

impl Default for EpsilonPolicy {
    fn default() -> Self {
        Self::FlatFolderDefault
    }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolveOptions {
    pub analyze: AnalyzeOptions,
    pub starting_face: Option<usize>,
    pub solution_limit: SolutionLimit,
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self {
            analyze: AnalyzeOptions::default(),
            starting_face: None,
            solution_limit: SolutionLimit::default(),
        }
    }
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
    _document: &FoldDocument,
    _options: NormalizeOptions,
) -> Result<NormalizedFold> {
    Err(FlatFoldError::Unimplemented("normalize_fold"))
}

pub fn analyze_flat_fold(_document: &FoldDocument, _options: AnalyzeOptions) -> Result<Analysis> {
    Err(FlatFoldError::Unimplemented("analyze_flat_fold"))
}

pub fn solve_flat_fold(_document: &FoldDocument, _options: SolveOptions) -> Result<SolveResult> {
    Err(FlatFoldError::Unimplemented("solve_flat_fold"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_doc() -> FoldDocument {
        FoldDocument::new(vec![vec![0.0, 0.0]], Vec::new())
    }

    #[test]
    fn unported_stages_return_explicit_unimplemented_errors() {
        let doc = empty_doc();
        assert_eq!(
            normalize_fold(&doc, NormalizeOptions::default()),
            Err(FlatFoldError::Unimplemented("normalize_fold"))
        );
        assert_eq!(
            analyze_flat_fold(&doc, AnalyzeOptions::default()),
            Err(FlatFoldError::Unimplemented("analyze_flat_fold"))
        );
        assert_eq!(
            solve_flat_fold(&doc, SolveOptions::default()),
            Err(FlatFoldError::Unimplemented("solve_flat_fold"))
        );
    }
}
