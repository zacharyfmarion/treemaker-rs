use crate::fold_graph::{FacePositions, FoldGraph};
use crate::geometry::{LineColor, LineSegment, Point};
use crate::model::CreasePatternModel;

#[derive(Debug, Clone, PartialEq)]
pub struct FoldedWireframe {
    pub points: Vec<Point>,
    pub lines: Vec<FoldedWireframeLine>,
    pub faces: Vec<Vec<usize>>,
    pub starting_face: usize,
    pub face_positions: Vec<usize>,
    pub next_faces: Vec<Option<usize>>,
    pub associated_lines: Vec<Option<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoldedWireframeLine {
    pub begin: usize,
    pub end: usize,
    pub color: LineColor,
}

/// Oriedita `WireFrame_Worker.folding()`: fold the line-set topology around a
/// starting face without solving layer overlap.
pub fn estimate_wireframe(
    model: &CreasePatternModel,
    starting_face_id: i32,
) -> Option<FoldedWireframe> {
    if model.line_segments.is_empty() {
        return None;
    }

    estimate_wireframe_from_segments(&model.line_segments, starting_face_id)
}

pub fn estimate_wireframe_from_segments(
    segments: &[LineSegment],
    starting_face_id: i32,
) -> Option<FoldedWireframe> {
    if segments.is_empty() {
        return None;
    }

    let graph = FoldGraph::from_segments(segments, true);
    if graph.faces.is_empty() {
        return None;
    }

    let face_positions = graph.face_positions(starting_face_id);
    Some(wireframe_from_graph(
        &graph,
        &face_positions,
        graph.folded_points(&face_positions),
    ))
}

/// Oriedita `WireFrame_Worker.getFacePositions()`: compute face adjacency
/// depth without moving vertices. This is used by Oriedita's two-colored CP
/// path before later subface/hierarchy stages.
pub fn face_position_wireframe_from_segments(
    segments: &[LineSegment],
    starting_face_id: i32,
) -> Option<FoldedWireframe> {
    if segments.is_empty() {
        return None;
    }

    let graph = FoldGraph::from_segments(segments, true);
    if graph.faces.is_empty() {
        return None;
    }

    let face_positions = graph.face_positions(starting_face_id);
    Some(wireframe_from_graph(
        &graph,
        &face_positions,
        graph.points.clone(),
    ))
}

fn wireframe_from_graph(
    graph: &FoldGraph,
    face_positions: &FacePositions,
    points: Vec<Point>,
) -> FoldedWireframe {
    FoldedWireframe {
        points,
        lines: graph
            .lines
            .iter()
            .map(|line| FoldedWireframeLine {
                begin: line.begin,
                end: line.end,
                color: line.color,
            })
            .collect(),
        faces: graph.faces.clone(),
        starting_face: face_positions.starting_face,
        face_positions: face_positions.face_position.clone(),
        next_faces: face_positions.next_face.clone(),
        associated_lines: face_positions.associated_line.clone(),
    }
}
