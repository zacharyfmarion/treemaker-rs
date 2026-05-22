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
use treemaker_fold::{
    Assignment, FoldDocument, build_edges_faces, build_faces_edges, validate_basic,
};

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

pub fn plan_folding_sequence(target: &TargetState) -> Result<SequencePlan> {
    plan_folding_sequence_with_options(target, SequencePlanOptions::default())
}

pub fn plan_folding_sequence_with_options(
    target: &TargetState,
    options: SequencePlanOptions,
) -> Result<SequencePlan> {
    let target_state = SequenceState::from_target("target", target);
    target_state.validate()?;

    let mut reverse_states = vec![target_state.clone()];
    let mut reverse_steps = Vec::new();
    let mut current = target_state;
    let mut explored_states = 1usize;
    let mut applied_steps = 0usize;
    let mut branches_pruned = 0usize;
    let mut budget_exhausted = false;
    let mut transform_diagnostics = Vec::new();

    loop {
        if current.active_creases.is_empty() {
            break;
        }
        if applied_steps >= options.max_steps {
            budget_exhausted = true;
            break;
        }
        if explored_states >= options.max_states {
            budget_exhausted = true;
            break;
        }
        let simple_folds = detect_simple_folds(&current)?;
        let pruned = simple_folds.len().saturating_sub(1);
        if let Some(simple_fold) = choose_simple_fold(simple_folds) {
            branches_pruned += pruned;
            let next_state_id = format!("state-{}", reverse_states.len());
            let before_reverse = current.id.clone();
            let next = apply_reverse_simple_fold(&current, &next_state_id, &simple_fold)?;
            next.validate()?;
            reverse_steps.push(simple_fold.to_forward_step(
                applied_steps,
                next.id.clone(),
                before_reverse,
            ));
            reverse_states.push(next.clone());
            current = next;
            explored_states += 1;
            applied_steps += 1;
            continue;
        }

        let complex_candidates = recognize_complex_moves(&current)?;
        let mut applied_complex = None;
        let mut unsupported_complex_candidates = 0usize;
        for candidate in &complex_candidates {
            let next_state_id = format!("state-{}", reverse_states.len());
            let result = apply_complex_transform(&current, &next_state_id, candidate)?;
            if result.status == ComplexTransformStatus::Applied {
                if let (Some(next), Some(step)) = (result.after_state, result.step) {
                    applied_complex = Some((next, step, result.diagnostics));
                    break;
                }
            } else {
                unsupported_complex_candidates += 1;
            }
        }
        if let Some((next, step, diagnostics)) = applied_complex {
            branches_pruned += unsupported_complex_candidates;
            transform_diagnostics.extend(diagnostics);
            next.validate()?;
            reverse_steps.push(step);
            reverse_states.push(next.clone());
            current = next;
            explored_states += 1;
            applied_steps += 1;
            continue;
        }

        break;
    }

    let complex_candidates = recognize_complex_moves(&current)?;
    let complex_transform_results = complex_candidates
        .iter()
        .map(|candidate| {
            apply_complex_transform(
                &current,
                &format!("state-{}", reverse_states.len()),
                candidate,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let unresolved_regions = unresolved_regions_for_state(&current);
    let status = if unresolved_regions.is_empty() {
        PlanStatus::Complete
    } else if complex_candidates
        .iter()
        .any(|candidate| candidate.kind == ComplexMoveKind::SimultaneousCollapse)
    {
        PlanStatus::Unsupported
    } else {
        PlanStatus::Partial
    };

    let mut diagnostics = target.diagnostics.clone();
    diagnostics.extend(transform_diagnostics);
    if !unresolved_regions.is_empty() {
        diagnostics.push(SequenceDiagnostic::warning(
            "unsupported_region",
            "planner stopped with crease groups outside the Phase 3 simple-fold rule set",
        ));
    }
    if budget_exhausted {
        diagnostics.push(SequenceDiagnostic::warning(
            "search_budget_exhausted",
            "planner returned the best partial state reached before the configured search budget",
        ));
    }
    for transform in &complex_transform_results {
        diagnostics.extend(transform.diagnostics.clone());
    }

    let mut steps = reverse_steps.into_iter().rev().collect::<Vec<_>>();
    for (index, step) in steps.iter_mut().enumerate() {
        set_step_id(step, format!("step-{}", index + 1));
    }
    if let Some(region) = unresolved_regions.first() {
        let mut step_diagnostics = Vec::new();
        if let Some(transform) = complex_transform_results.first() {
            step_diagnostics.extend(transform.diagnostics.clone());
        }
        steps.push(InstructionStep::UnsupportedRegion(UnsupportedRegionStep {
            id: format!("step-{}", steps.len() + 1),
            label: "Unsupported collapse region".to_string(),
            before_state: current.id.clone(),
            region: region.clone(),
            diagnostics: step_diagnostics,
        }));
    }

    let mut states = reverse_states;
    states.reverse();
    Ok(SequencePlan {
        status,
        steps,
        states,
        diagnostics,
        unresolved_regions,
        search: SearchStats {
            states_explored: explored_states,
            branches_pruned,
            repeated_states: 0,
            timed_out: budget_exhausted,
            budget_exhausted,
            best_unresolved_creases: current.active_creases.len(),
        },
    })
}

pub fn plan_reference_precreases(document: &FoldDocument) -> Result<ReferencePlan> {
    plan_reference_precreases_with_options(document, ReferencePlanOptions::default())
}

pub fn plan_reference_precreases_with_options(
    document: &FoldDocument,
    _options: ReferencePlanOptions,
) -> Result<ReferencePlan> {
    validate_basic(document).map_err(|error| SequenceError::InvalidInput(error.to_string()))?;
    Err(SequenceError::NotImplemented(
        "V2 ReferenceFinder-style reference/precrease planner",
    ))
}

pub fn trace_plan(plan: &SequencePlan) -> PlannerTrace {
    PlannerTrace {
        schema_version: 1,
        planner_version: env!("CARGO_PKG_VERSION").to_string(),
        status: plan.status.clone(),
        score: plan.score(),
        search: plan.search.clone(),
        candidates: plan
            .steps
            .iter()
            .enumerate()
            .map(|(index, step)| trace_candidate_for_step(index, step, plan))
            .collect(),
        diagnostics: plan
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect(),
        ml_decision: ml_readiness_decision(1, usize::from(plan.status == PlanStatus::Complete)),
    }
}

pub fn ml_readiness_decision(total_traces: usize, complete_traces: usize) -> MlReadinessDecision {
    const MINIMUM_SUCCESSFUL_TRACES: usize = 500;
    if complete_traces >= MINIMUM_SUCCESSFUL_TRACES {
        MlReadinessDecision {
            recommendation: MlRecommendation::ConsiderOfflineRanker,
            reason:
                "enough successful symbolic traces exist to justify an offline ranking experiment"
                    .to_string(),
            minimum_successful_traces: MINIMUM_SUCCESSFUL_TRACES,
        }
    } else if total_traces > 0 {
        MlReadinessDecision {
            recommendation: MlRecommendation::CollectMoreTraces,
            reason: "symbolic planner traces are useful, but the successful trace count is still too small for ML"
                .to_string(),
            minimum_successful_traces: MINIMUM_SUCCESSFUL_TRACES,
        }
    } else {
        MlReadinessDecision {
            recommendation: MlRecommendation::KeepSymbolic,
            reason: "no validated trace corpus exists yet; ML must not affect production behavior"
                .to_string(),
            minimum_successful_traces: MINIMUM_SUCCESSFUL_TRACES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Complete,
    Partial,
    Unsupported,
    InvalidInput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SequencePlan {
    pub status: PlanStatus,
    pub steps: Vec<InstructionStep>,
    pub states: Vec<SequenceState>,
    pub diagnostics: Vec<SequenceDiagnostic>,
    pub unresolved_regions: Vec<UnresolvedRegion>,
    pub search: SearchStats,
}

impl SequencePlan {
    pub fn score(&self) -> PlanScore {
        PlanScore {
            unresolved_creases: self
                .unresolved_regions
                .iter()
                .map(|region| region.creases.len())
                .sum(),
            unresolved_regions: self.unresolved_regions.len(),
            unsupported_steps: self
                .steps
                .iter()
                .filter(|step| matches!(step, InstructionStep::UnsupportedRegion(_)))
                .count(),
            total_steps: self.steps.len(),
            layer_order_ambiguity: self
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "ambiguous_layer_order")
                .count(),
            simultaneous_candidates: self
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "simultaneous_collapse_unsupported")
                .count(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchStats {
    pub states_explored: usize,
    pub branches_pruned: usize,
    pub repeated_states: usize,
    pub timed_out: bool,
    pub budget_exhausted: bool,
    pub best_unresolved_creases: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequencePlanOptions {
    pub max_steps: usize,
    pub max_states: usize,
}

impl Default for SequencePlanOptions {
    fn default() -> Self {
        Self {
            max_steps: 64,
            max_states: 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PlanScore {
    pub unresolved_creases: usize,
    pub unresolved_regions: usize,
    pub unsupported_steps: usize,
    pub total_steps: usize,
    pub layer_order_ambiguity: usize,
    pub simultaneous_candidates: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerTrace {
    pub schema_version: u32,
    pub planner_version: String,
    pub status: PlanStatus,
    pub score: PlanScore,
    pub search: SearchStats,
    pub candidates: Vec<TraceCandidate>,
    pub diagnostics: Vec<String>,
    pub ml_decision: MlReadinessDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceCandidate {
    pub step_id: String,
    pub kind: String,
    pub affected_creases: Vec<usize>,
    pub accepted: bool,
    pub unresolved_after: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MlReadinessDecision {
    pub recommendation: MlRecommendation,
    pub reason: String,
    pub minimum_successful_traces: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MlRecommendation {
    KeepSymbolic,
    CollectMoreTraces,
    ConsiderOfflineRanker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferencePlanOptions {
    pub max_axiom_depth: usize,
}

impl Default for ReferencePlanOptions {
    fn default() -> Self {
        Self { max_axiom_depth: 6 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePlanStatus {
    Complete,
    Partial,
    NotImplemented,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferencePlan {
    pub status: ReferencePlanStatus,
    pub steps: Vec<ReferenceConstructionStep>,
    pub diagnostics: Vec<SequenceDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReferenceConstructionStep {
    ReferenceFold {
        id: String,
        label: String,
        axiom: String,
        target_creases: Vec<usize>,
    },
    PrecreaseRegion {
        id: String,
        label: String,
        creases: Vec<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleFoldRule {
    pub crease: usize,
    pub faces: Vec<usize>,
    pub assignment: Assignment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexMoveKind {
    ReverseFold,
    SquashFold,
    RabbitEar,
    MoleculeCollapse,
    SimultaneousCollapse,
}

impl ComplexMoveKind {
    fn diagnostic_code(&self) -> &'static str {
        match self {
            ComplexMoveKind::ReverseFold => "reverse_fold_not_implemented",
            ComplexMoveKind::SquashFold => "squash_fold_not_implemented",
            ComplexMoveKind::RabbitEar => "rabbit_ear_not_implemented",
            ComplexMoveKind::MoleculeCollapse => "molecule_collapse_not_implemented",
            ComplexMoveKind::SimultaneousCollapse => "simultaneous_collapse_unsupported",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexMoveCandidate {
    pub kind: ComplexMoveKind,
    pub center_vertex: Option<usize>,
    pub creases: Vec<usize>,
    pub faces: Vec<usize>,
    pub metadata: MoveMetadata,
    pub diagnostics: Vec<SequenceDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexTransformStatus {
    Applied,
    Unsupported,
    InvalidCandidate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexTransformResult {
    pub status: ComplexTransformStatus,
    pub candidate: ComplexMoveCandidate,
    pub before_state: String,
    pub after_state: Option<SequenceState>,
    pub step: Option<InstructionStep>,
    pub diagnostics: Vec<SequenceDiagnostic>,
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
            .filter_map(|(edge, assignment)| {
                matches!(assignment, Assignment::Mountain | Assignment::Valley).then_some(edge)
            })
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

pub fn detect_simple_fold(state: &SequenceState) -> Result<Option<SimpleFoldRule>> {
    Ok(choose_simple_fold(detect_simple_folds(state)?))
}

pub fn detect_simple_folds(state: &SequenceState) -> Result<Vec<SimpleFoldRule>> {
    let edges_faces = if state.document.edges_faces.is_empty() {
        build_edges_faces(&state.document)
            .map_err(|error| SequenceError::InvalidInput(error.to_string()))?
    } else {
        state.document.edges_faces.clone()
    };
    let boundary_vertices = boundary_vertex_flags(&state.document);
    let mut active_creases = state.active_creases.clone();
    active_creases.sort_unstable();
    let mut out = Vec::new();
    for crease in active_creases {
        let assignment = state.document.assignment_for_edge(crease);
        if !matches!(assignment, Assignment::Mountain | Assignment::Valley) {
            continue;
        }
        let Some([a, b]) = state.document.edges_vertices.get(crease).copied() else {
            continue;
        };
        let faces = edges_faces.get(crease).cloned().unwrap_or_default();
        if faces.len() == 2
            && boundary_vertices.get(a).copied().unwrap_or(false)
            && boundary_vertices.get(b).copied().unwrap_or(false)
        {
            out.push(SimpleFoldRule {
                crease,
                faces,
                assignment,
            });
        }
    }
    out.sort_by_key(simple_fold_sort_key);
    Ok(out)
}

fn choose_simple_fold(mut simple_folds: Vec<SimpleFoldRule>) -> Option<SimpleFoldRule> {
    simple_folds.sort_by_key(simple_fold_sort_key);
    simple_folds.into_iter().next()
}

fn simple_fold_sort_key(rule: &SimpleFoldRule) -> (usize, usize, u8) {
    let assignment_rank = match rule.assignment {
        Assignment::Valley => 0,
        Assignment::Mountain => 1,
        _ => 2,
    };
    (rule.faces.len(), rule.crease, assignment_rank)
}

pub fn recognize_complex_moves(state: &SequenceState) -> Result<Vec<ComplexMoveCandidate>> {
    if state.active_creases.is_empty() {
        return Ok(Vec::new());
    }
    let edges_faces = if state.document.edges_faces.is_empty() {
        build_edges_faces(&state.document)
            .map_err(|error| SequenceError::InvalidInput(error.to_string()))?
    } else {
        state.document.edges_faces.clone()
    };
    let boundary_vertices = boundary_vertex_flags(&state.document);
    let mut active_by_vertex = vec![Vec::new(); state.document.vertices_coords.len()];
    for crease in &state.active_creases {
        let Some([a, b]) = state.document.edges_vertices.get(*crease).copied() else {
            continue;
        };
        if let Some(creases) = active_by_vertex.get_mut(a) {
            creases.push(*crease);
        }
        if let Some(creases) = active_by_vertex.get_mut(b) {
            creases.push(*crease);
        }
    }

    let mut candidates = Vec::new();
    for (vertex, creases) in active_by_vertex.into_iter().enumerate() {
        if boundary_vertices.get(vertex).copied().unwrap_or(false) || creases.len() < 3 {
            continue;
        }
        let kind = classify_complex_candidate(state, &creases);
        let mut faces = Vec::new();
        for crease in &creases {
            if let Some(edge_faces) = edges_faces.get(*crease) {
                faces.extend(edge_faces.iter().copied());
            }
        }
        faces.sort_unstable();
        faces.dedup();
        candidates.push(ComplexMoveCandidate {
            kind: kind.clone(),
            center_vertex: Some(vertex),
            creases,
            faces,
            metadata: MoveMetadata {
                difficulty: match kind {
                    ComplexMoveKind::ReverseFold => MoveDifficulty::Intermediate,
                    ComplexMoveKind::SquashFold
                    | ComplexMoveKind::RabbitEar
                    | ComplexMoveKind::MoleculeCollapse
                    | ComplexMoveKind::SimultaneousCollapse => MoveDifficulty::Complex,
                },
                layer_mode: match kind {
                    ComplexMoveKind::SimultaneousCollapse => LayerMode::Simultaneous,
                    ComplexMoveKind::ReverseFold
                    | ComplexMoveKind::SquashFold
                    | ComplexMoveKind::RabbitEar => LayerMode::MultiLayer,
                    ComplexMoveKind::MoleculeCollapse => LayerMode::Simultaneous,
                },
                confidence: 0.6,
                notes: vec![
                    "topology-only recognition; transform intentionally not implemented"
                        .to_string(),
                ],
            },
            diagnostics: vec![SequenceDiagnostic::warning(
                kind.diagnostic_code(),
                "recognized complex move pattern, but no validated rewrite exists yet",
            )],
        });
    }
    candidates.sort_by_key(|candidate| {
        (
            complex_kind_rank(&candidate.kind),
            candidate.center_vertex.unwrap_or(usize::MAX),
        )
    });
    Ok(candidates)
}

pub fn apply_complex_transform(
    state: &SequenceState,
    _next_state_id: &str,
    candidate: &ComplexMoveCandidate,
) -> Result<ComplexTransformResult> {
    let mut diagnostics = inspect_complex_transform_candidate(state, candidate);
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    {
        return Ok(ComplexTransformResult {
            status: ComplexTransformStatus::InvalidCandidate,
            candidate: candidate.clone(),
            before_state: state.id.clone(),
            after_state: None,
            step: None,
            diagnostics,
        });
    }

    if phase12_complex_transform_is_supported(state, candidate) {
        return apply_reverse_complex_group(state, _next_state_id, candidate, diagnostics);
    }

    diagnostics.push(SequenceDiagnostic::warning(
        "complex_transform_not_implemented",
        format!(
            "{:?} transform is routed through the Phase 11 harness, but no validated rewrite has been implemented yet",
            candidate.kind
        ),
    ));
    diagnostics.push(SequenceDiagnostic::warning(
        candidate.kind.diagnostic_code(),
        format!(
            "{:?} pattern recognized for creases {:?}, but the move transform is not implemented yet",
            candidate.kind, candidate.creases
        ),
    ));

    Ok(ComplexTransformResult {
        status: ComplexTransformStatus::Unsupported,
        candidate: candidate.clone(),
        before_state: state.id.clone(),
        after_state: None,
        step: None,
        diagnostics,
    })
}

fn phase12_complex_transform_is_supported(
    state: &SequenceState,
    candidate: &ComplexMoveCandidate,
) -> bool {
    matches!(
        candidate.kind,
        ComplexMoveKind::ReverseFold | ComplexMoveKind::SquashFold
    ) && active_creases_match_candidate(state, candidate)
}

fn active_creases_match_candidate(state: &SequenceState, candidate: &ComplexMoveCandidate) -> bool {
    let mut active = state.active_creases.clone();
    let mut candidate_creases = candidate.creases.clone();
    active.sort_unstable();
    candidate_creases.sort_unstable();
    active == candidate_creases
}

fn apply_reverse_complex_group(
    state: &SequenceState,
    next_state_id: &str,
    candidate: &ComplexMoveCandidate,
    mut diagnostics: Vec<SequenceDiagnostic>,
) -> Result<ComplexTransformResult> {
    let mut document = state.document.clone();
    for crease in &candidate.creases {
        if let Some(assignment) = document.edges_assignment.get_mut(*crease) {
            *assignment = Assignment::Flat;
        }
        if let Some(angle) = document.edges_fold_angle.get_mut(*crease) {
            *angle = Some(0.0);
        }
    }

    let target = match resolve_target_state(&document, TargetStateOptions::default()) {
        Ok(target) => target,
        Err(error) => {
            diagnostics.push(SequenceDiagnostic::warning(
                "complex_transform_target_solve_failed",
                format!(
                    "{:?} candidate could not be accepted because the reverse state failed target resolution: {error}",
                    candidate.kind
                ),
            ));
            return Ok(ComplexTransformResult {
                status: ComplexTransformStatus::Unsupported,
                candidate: candidate.clone(),
                before_state: state.id.clone(),
                after_state: None,
                step: None,
                diagnostics,
            });
        }
    };

    let mut next = SequenceState::from_target(next_state_id.to_string(), &target);
    next.provenance = StateProvenance::rewrite(
        &state.id,
        format!(
            "reverse-complex-{}",
            complex_kind_label(&candidate.kind).replace(' ', "-")
        ),
    );
    next.active_creases
        .retain(|crease| !candidate.creases.contains(crease));
    next.validate()?;

    diagnostics.push(SequenceDiagnostic::info(
        "complex_transform_applied",
        format!(
            "{:?} transform accepted as an isolated local complex collapse over creases {:?}",
            candidate.kind, candidate.creases
        ),
    ));
    let step = complex_candidate_to_forward_step(candidate, next.id.clone(), state.id.clone());

    Ok(ComplexTransformResult {
        status: ComplexTransformStatus::Applied,
        candidate: candidate.clone(),
        before_state: state.id.clone(),
        after_state: Some(next),
        step: Some(step),
        diagnostics,
    })
}

pub fn inspect_complex_transform_candidate(
    state: &SequenceState,
    candidate: &ComplexMoveCandidate,
) -> Vec<SequenceDiagnostic> {
    let mut diagnostics = Vec::new();
    let edge_count = state.document.edges_vertices.len();
    let face_count = state.document.faces_vertices.len();
    let vertex_count = state.document.vertices_coords.len();

    if candidate.creases.is_empty() {
        diagnostics.push(SequenceDiagnostic::error(
            "complex_candidate_empty",
            "complex transform candidate has no creases",
        ));
    }
    if has_duplicates(&candidate.creases) {
        diagnostics.push(SequenceDiagnostic::error(
            "complex_candidate_duplicate_crease",
            "complex transform candidate contains duplicate creases",
        ));
    }
    if has_duplicates(&candidate.faces) {
        diagnostics.push(SequenceDiagnostic::error(
            "complex_candidate_duplicate_face",
            "complex transform candidate contains duplicate faces",
        ));
    }

    for crease in &candidate.creases {
        if *crease >= edge_count {
            diagnostics.push(SequenceDiagnostic::error(
                "complex_candidate_crease_out_of_bounds",
                format!("complex candidate crease {crease} is outside edge range 0..{edge_count}"),
            ));
            continue;
        }
        if !state.active_creases.contains(crease) {
            diagnostics.push(SequenceDiagnostic::error(
                "complex_candidate_crease_not_active",
                format!(
                    "complex candidate crease {crease} is not active in state {}",
                    state.id
                ),
            ));
        }
        let assignment = state.document.assignment_for_edge(*crease);
        if !matches!(assignment, Assignment::Mountain | Assignment::Valley) {
            diagnostics.push(SequenceDiagnostic::error(
                "complex_candidate_crease_not_mv",
                format!(
                    "complex candidate crease {crease} has assignment {}; expected M or V",
                    assignment.as_str()
                ),
            ));
        }
    }

    for face in &candidate.faces {
        if *face >= face_count {
            diagnostics.push(SequenceDiagnostic::error(
                "complex_candidate_face_out_of_bounds",
                format!("complex candidate face {face} is outside face range 0..{face_count}"),
            ));
        }
    }

    if let Some(vertex) = candidate.center_vertex {
        if vertex >= vertex_count {
            diagnostics.push(SequenceDiagnostic::error(
                "complex_candidate_center_out_of_bounds",
                format!("complex candidate center vertex {vertex} is outside vertex range 0..{vertex_count}"),
            ));
        } else if boundary_vertex_flags(&state.document)
            .get(vertex)
            .copied()
            .unwrap_or(false)
        {
            diagnostics.push(SequenceDiagnostic::error(
                "complex_candidate_center_on_boundary",
                format!("complex candidate center vertex {vertex} lies on the paper boundary"),
            ));
        }
    }

    diagnostics
}

fn classify_complex_candidate(state: &SequenceState, creases: &[usize]) -> ComplexMoveKind {
    match creases.len() {
        3 => ComplexMoveKind::ReverseFold,
        4 => ComplexMoveKind::RabbitEar,
        5..=6 => ComplexMoveKind::MoleculeCollapse,
        _ => {
            let valley_count = creases
                .iter()
                .filter(|crease| state.document.assignment_for_edge(**crease) == Assignment::Valley)
                .count();
            let mountain_count = creases.len().saturating_sub(valley_count);
            if valley_count > mountain_count {
                ComplexMoveKind::SimultaneousCollapse
            } else {
                ComplexMoveKind::SquashFold
            }
        }
    }
}

fn complex_kind_rank(kind: &ComplexMoveKind) -> usize {
    match kind {
        ComplexMoveKind::ReverseFold => 0,
        ComplexMoveKind::RabbitEar => 1,
        ComplexMoveKind::SquashFold => 2,
        ComplexMoveKind::MoleculeCollapse => 3,
        ComplexMoveKind::SimultaneousCollapse => 4,
    }
}

fn has_duplicates(values: &[usize]) -> bool {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted.windows(2).any(|pair| pair[0] == pair[1])
}

fn apply_reverse_simple_fold(
    state: &SequenceState,
    next_state_id: &str,
    rule: &SimpleFoldRule,
) -> Result<SequenceState> {
    let mut document = state.document.clone();
    if let Some(assignment) = document.edges_assignment.get_mut(rule.crease) {
        *assignment = Assignment::Flat;
    }
    if let Some(angle) = document.edges_fold_angle.get_mut(rule.crease) {
        *angle = Some(0.0);
    }
    let target = resolve_target_state(&document, TargetStateOptions::default())?;
    let mut next = SequenceState::from_target(next_state_id.to_string(), &target);
    next.provenance =
        StateProvenance::rewrite(&state.id, format!("reverse-simple-{}", rule.crease));
    next.active_creases.retain(|crease| *crease != rule.crease);
    Ok(next)
}

fn unresolved_regions_for_state(state: &SequenceState) -> Vec<UnresolvedRegion> {
    if state.active_creases.is_empty() {
        return Vec::new();
    }
    let mut faces = Vec::new();
    if let Ok(edges_faces) = if state.document.edges_faces.is_empty() {
        build_edges_faces(&state.document)
    } else {
        Ok(state.document.edges_faces.clone())
    } {
        for crease in &state.active_creases {
            if let Some(edge_faces) = edges_faces.get(*crease) {
                faces.extend(edge_faces.iter().copied());
            }
        }
        faces.sort_unstable();
        faces.dedup();
    }
    vec![UnresolvedRegion {
        id: "unresolved-1".to_string(),
        creases: state.active_creases.clone(),
        faces,
        reason: "no validated Phase 3 simple-fold rewrite matches these creases".to_string(),
    }]
}

fn boundary_vertex_flags(document: &FoldDocument) -> Vec<bool> {
    let mut flags = vec![false; document.vertices_coords.len()];
    for (edge_index, [a, b]) in document.edges_vertices.iter().copied().enumerate() {
        if document.assignment_for_edge(edge_index) == Assignment::Boundary {
            if let Some(flag) = flags.get_mut(a) {
                *flag = true;
            }
            if let Some(flag) = flags.get_mut(b) {
                *flag = true;
            }
        }
    }
    flags
}

impl SimpleFoldRule {
    fn to_forward_step(
        &self,
        index: usize,
        before_state: String,
        after_state: String,
    ) -> InstructionStep {
        let assignment = match self.assignment {
            Assignment::Mountain => "mountain",
            Assignment::Valley => "valley",
            Assignment::Flat => "flat",
            Assignment::Boundary | Assignment::Unassigned | Assignment::Cut | Assignment::Join => {
                "fold"
            }
        };
        let mut details = StepDetails::new(
            format!("step-{}", index + 1),
            format!("Make a {assignment} fold on crease {}", self.crease),
            before_state,
            after_state,
        );
        details.affected_creases = vec![self.crease];
        details.affected_faces = self.faces.clone();
        details.metadata = MoveMetadata {
            difficulty: MoveDifficulty::Simple,
            layer_mode: if self.faces.len() == 2 {
                LayerMode::SingleLayer
            } else {
                LayerMode::Unknown
            },
            confidence: 1.0,
            notes: Vec::new(),
        };
        InstructionStep::SimpleFold(details)
    }
}

fn complex_candidate_to_forward_step(
    candidate: &ComplexMoveCandidate,
    before_state: String,
    after_state: String,
) -> InstructionStep {
    let mut details = StepDetails::new(
        "complex-step",
        complex_step_label(candidate),
        before_state,
        after_state,
    );
    details.affected_creases = candidate.creases.clone();
    details.affected_faces = candidate.faces.clone();
    details.metadata = candidate.metadata.clone();
    details.metadata.confidence = details.metadata.confidence.max(0.7);
    details.metadata.notes.push(
        "accepted as an isolated local complex move; lower-level sub-folds are not decomposed"
            .to_string(),
    );
    match candidate.kind {
        ComplexMoveKind::ReverseFold => InstructionStep::ReverseFold(details),
        ComplexMoveKind::SquashFold => InstructionStep::SquashFold(details),
        ComplexMoveKind::RabbitEar => InstructionStep::RabbitEar(details),
        ComplexMoveKind::MoleculeCollapse => InstructionStep::MoleculeCollapse(details),
        ComplexMoveKind::SimultaneousCollapse => InstructionStep::SimultaneousCollapse(details),
    }
}

fn complex_step_label(candidate: &ComplexMoveCandidate) -> String {
    let center = candidate
        .center_vertex
        .map(|vertex| format!(" at vertex {vertex}"))
        .unwrap_or_default();
    format!(
        "Perform a {}{}",
        complex_kind_label(&candidate.kind),
        center
    )
}

fn complex_kind_label(kind: &ComplexMoveKind) -> &'static str {
    match kind {
        ComplexMoveKind::ReverseFold => "reverse fold",
        ComplexMoveKind::SquashFold => "squash fold",
        ComplexMoveKind::RabbitEar => "rabbit ear",
        ComplexMoveKind::MoleculeCollapse => "molecule collapse",
        ComplexMoveKind::SimultaneousCollapse => "simultaneous collapse",
    }
}

fn set_step_id(step: &mut InstructionStep, id: String) {
    match step {
        InstructionStep::PrecreaseRegion(details)
        | InstructionStep::SimpleFold(details)
        | InstructionStep::ReverseFold(details)
        | InstructionStep::SquashFold(details)
        | InstructionStep::RabbitEar(details)
        | InstructionStep::MoleculeCollapse(details)
        | InstructionStep::SimultaneousCollapse(details) => details.id = id,
        InstructionStep::ManualChoice(step) => step.id = id,
        InstructionStep::UnsupportedRegion(step) => step.id = id,
    }
}

fn trace_candidate_for_step(
    index: usize,
    step: &InstructionStep,
    plan: &SequencePlan,
) -> TraceCandidate {
    let unresolved_after = plan
        .states
        .get(index + 1)
        .map(|state| state.active_creases.len())
        .unwrap_or_else(|| plan.search.best_unresolved_creases);
    match step {
        InstructionStep::PrecreaseRegion(details)
        | InstructionStep::SimpleFold(details)
        | InstructionStep::ReverseFold(details)
        | InstructionStep::SquashFold(details)
        | InstructionStep::RabbitEar(details)
        | InstructionStep::MoleculeCollapse(details)
        | InstructionStep::SimultaneousCollapse(details) => TraceCandidate {
            step_id: details.id.clone(),
            kind: instruction_kind(step).to_string(),
            affected_creases: details.affected_creases.clone(),
            accepted: true,
            unresolved_after,
        },
        InstructionStep::ManualChoice(step) => TraceCandidate {
            step_id: step.id.clone(),
            kind: "manual_choice".to_string(),
            affected_creases: step
                .choices
                .iter()
                .flat_map(|choice| choice.affected_creases.iter().copied())
                .collect(),
            accepted: false,
            unresolved_after,
        },
        InstructionStep::UnsupportedRegion(step) => TraceCandidate {
            step_id: step.id.clone(),
            kind: "unsupported_region".to_string(),
            affected_creases: step.region.creases.clone(),
            accepted: false,
            unresolved_after,
        },
    }
}

fn instruction_kind(step: &InstructionStep) -> &'static str {
    match step {
        InstructionStep::PrecreaseRegion(_) => "precrease_region",
        InstructionStep::SimpleFold(_) => "simple_fold",
        InstructionStep::ReverseFold(_) => "reverse_fold",
        InstructionStep::SquashFold(_) => "squash_fold",
        InstructionStep::RabbitEar(_) => "rabbit_ear",
        InstructionStep::MoleculeCollapse(_) => "molecule_collapse",
        InstructionStep::SimultaneousCollapse(_) => "simultaneous_collapse",
        InstructionStep::ManualChoice(_) => "manual_choice",
        InstructionStep::UnsupportedRegion(_) => "unsupported_region",
    }
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

    fn rabbit_ear_local() -> FoldDocument {
        let mut document = FoldDocument::new(
            vec![
                vec![0.0, 0.0],
                vec![1.0, 0.0],
                vec![1.0, 1.0],
                vec![0.0, 1.0],
                vec![0.5, 0.0],
                vec![1.0, 0.5],
                vec![0.5, 1.0],
                vec![0.0, 0.5],
                vec![0.5, 0.5],
            ],
            vec![
                [0, 4],
                [4, 1],
                [1, 5],
                [5, 2],
                [2, 6],
                [6, 3],
                [3, 7],
                [7, 0],
                [4, 8],
                [5, 8],
                [6, 8],
                [7, 8],
            ],
        );
        document.edges_assignment = vec![
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Mountain,
            Assignment::Mountain,
            Assignment::Mountain,
            Assignment::Valley,
        ];
        document.faces_vertices = vec![
            vec![0, 4, 8, 7],
            vec![4, 1, 5, 8],
            vec![8, 5, 2, 6],
            vec![7, 8, 6, 3],
        ];
        document
    }

    fn squash_local() -> FoldDocument {
        let mut document = FoldDocument::new(
            vec![
                vec![0.0, 0.0],
                vec![1.0, 0.0],
                vec![1.0, 1.0],
                vec![0.0, 1.0],
                vec![0.5, 0.0],
                vec![1.0, 0.5],
                vec![0.5, 1.0],
                vec![0.0, 0.5],
                vec![0.5, 0.5],
            ],
            vec![
                [0, 4],
                [4, 1],
                [1, 5],
                [5, 2],
                [2, 6],
                [6, 3],
                [3, 7],
                [7, 0],
                [4, 8],
                [1, 8],
                [5, 8],
                [2, 8],
                [6, 8],
                [3, 8],
                [7, 8],
                [0, 8],
            ],
        );
        document.edges_assignment = vec![
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Mountain,
            Assignment::Valley,
            Assignment::Mountain,
            Assignment::Mountain,
            Assignment::Valley,
            Assignment::Mountain,
            Assignment::Mountain,
            Assignment::Valley,
        ];
        document.faces_vertices = vec![
            vec![0, 4, 8],
            vec![4, 1, 8],
            vec![1, 5, 8],
            vec![5, 2, 8],
            vec![2, 6, 8],
            vec![6, 3, 8],
            vec![3, 7, 8],
            vec![7, 0, 8],
        ];
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
    fn reference_precrease_planner_is_explicitly_not_implemented() {
        let document = two_face_valley();
        let error =
            plan_reference_precreases(&document).expect_err("reference finder is not implemented");

        assert_eq!(error.code(), "not_implemented");
    }

    #[test]
    fn simple_fold_planner_completes_single_simple_fold() {
        let target = resolve_target_state(&two_face_valley(), TargetStateOptions::default())
            .expect("target state");
        let plan = plan_folding_sequence(&target).expect("simple fold plan");

        assert_eq!(plan.status, PlanStatus::Complete);
        assert_eq!(plan.steps.len(), 1);
        assert!(plan.unresolved_regions.is_empty());
        assert_eq!(plan.search.best_unresolved_creases, 0);
        match &plan.steps[0] {
            InstructionStep::SimpleFold(details) => {
                assert_eq!(details.affected_creases, vec![4]);
                assert_eq!(details.metadata.difficulty, MoveDifficulty::Simple);
            }
            other => panic!("expected simple fold step, got {other:?}"),
        }
    }

    #[test]
    fn complex_transform_harness_keeps_unimplemented_moves_unsupported() {
        let target = resolve_target_state(&rabbit_ear_local(), TargetStateOptions::default())
            .expect("target state");
        let state = SequenceState::from_target("target", &target);
        let candidate = recognize_complex_moves(&state)
            .expect("complex candidates")
            .into_iter()
            .find(|candidate| candidate.kind == ComplexMoveKind::RabbitEar)
            .expect("rabbit-ear candidate");

        let result =
            apply_complex_transform(&state, "state-1", &candidate).expect("transform result");

        assert_eq!(result.status, ComplexTransformStatus::Unsupported);
        assert!(result.after_state.is_none());
        assert!(result.step.is_none());
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "complex_transform_not_implemented"
                && diagnostic.severity == DiagnosticSeverity::Warning
        }));
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "rabbit_ear_not_implemented")
        );
    }

    #[test]
    fn isolated_squash_transform_completes_as_complex_step() {
        let target =
            resolve_target_state(&squash_local(), TargetStateOptions::default()).expect("target");
        let plan = plan_folding_sequence(&target).expect("squash plan");

        assert_eq!(plan.status, PlanStatus::Complete);
        assert!(plan.unresolved_regions.is_empty());
        assert_eq!(plan.search.best_unresolved_creases, 0);
        assert!(
            plan.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "complex_transform_applied")
        );
        match &plan.steps[0] {
            InstructionStep::SquashFold(details) => {
                assert_eq!(details.affected_creases.len(), 8);
                assert_eq!(details.before_state, "state-1");
                assert_eq!(details.after_state, "target");
            }
            other => panic!("expected squash fold step, got {other:?}"),
        }
        assert!(
            plan.states
                .first()
                .is_some_and(|state| state.active_creases.is_empty())
        );
    }

    #[test]
    fn complex_transform_harness_rejects_invalid_candidates() {
        let target = resolve_target_state(&two_face_valley(), TargetStateOptions::default())
            .expect("target state");
        let state = SequenceState::from_target("target", &target);
        let candidate = ComplexMoveCandidate {
            kind: ComplexMoveKind::ReverseFold,
            center_vertex: Some(99),
            creases: vec![4, 99],
            faces: vec![0],
            metadata: MoveMetadata::default(),
            diagnostics: Vec::new(),
        };

        let result =
            apply_complex_transform(&state, "state-1", &candidate).expect("transform result");

        assert_eq!(result.status, ComplexTransformStatus::InvalidCandidate);
        assert!(result.after_state.is_none());
        assert!(result.step.is_none());
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "complex_candidate_crease_out_of_bounds"
                && diagnostic.severity == DiagnosticSeverity::Error
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "complex_candidate_center_out_of_bounds"
                && diagnostic.severity == DiagnosticSeverity::Error
        }));
    }

    #[test]
    fn search_budget_returns_best_partial_plan() {
        let target = resolve_target_state(&two_face_valley(), TargetStateOptions::default())
            .expect("target state");
        let plan = plan_folding_sequence_with_options(
            &target,
            SequencePlanOptions {
                max_steps: 0,
                ..SequencePlanOptions::default()
            },
        )
        .expect("partial plan");

        assert_eq!(plan.status, PlanStatus::Partial);
        assert!(plan.search.budget_exhausted);
        assert_eq!(plan.search.best_unresolved_creases, 1);
        assert!(plan.score().unresolved_creases > 0);
    }

    #[test]
    fn planner_output_is_deterministic() {
        let target = resolve_target_state(&two_face_valley(), TargetStateOptions::default())
            .expect("target state");
        let first = plan_folding_sequence(&target).expect("first plan");
        let second = plan_folding_sequence(&target).expect("second plan");

        assert_eq!(
            serde_json::to_value(first).expect("first json"),
            serde_json::to_value(second).expect("second json")
        );
    }

    #[test]
    fn trace_schema_replays_plan_score() {
        let target = resolve_target_state(&two_face_valley(), TargetStateOptions::default())
            .expect("target state");
        let plan = plan_folding_sequence(&target).expect("plan");
        let trace = trace_plan(&plan);

        assert_eq!(trace.schema_version, 1);
        assert_eq!(trace.status, plan.status);
        assert_eq!(trace.score, plan.score());
        assert_eq!(trace.candidates.len(), plan.steps.len());
        assert_eq!(
            trace.ml_decision.recommendation,
            MlRecommendation::CollectMoreTraces
        );
    }

    #[test]
    fn ml_decision_keeps_runtime_symbolic_without_trace_corpus() {
        let decision = ml_readiness_decision(0, 0);

        assert_eq!(decision.recommendation, MlRecommendation::KeepSymbolic);
        assert!(
            decision
                .reason
                .contains("ML must not affect production behavior")
        );
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
