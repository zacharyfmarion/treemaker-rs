use crate::fold_graph::{FacePositions, FoldGraph};
use crate::geometry::{Epsilon, LineColor, LineSegment, Point, equal, equal_with_radius};
use crate::model::CreasePatternModel;
use crate::operations::arrangement::divide_intersections;

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

/// Oriedita `LineSegmentSetWorker.split_arrangement_for_SubFace_generation()`.
///
/// This is the folded-model preprocessing pass before subface generation:
/// remove point-like line segments, remove duplicate endpoint-identical
/// segments with Oriedita's `UNKNOWN_001` tolerance, divide all intersections,
/// and run the point/duplicate cleanup again.
pub fn prepare_subface_segments(segments: &[LineSegment]) -> Vec<LineSegment> {
    let mut model = CreasePatternModel {
        line_segments: segments.to_vec(),
        ..CreasePatternModel::default()
    };
    remove_point_segments(&mut model.line_segments);
    remove_line_segment_set_duplicates(&mut model.line_segments);
    divide_intersections(&mut model);
    remove_point_segments(&mut model.line_segments);
    remove_line_segment_set_duplicates(&mut model.line_segments);
    model.line_segments
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

fn remove_point_segments(segments: &mut Vec<LineSegment>) {
    segments.retain(|segment| !equal(segment.a, segment.b));
}

fn remove_line_segment_set_duplicates(segments: &mut Vec<LineSegment>) {
    let mut remove = vec![false; segments.len()];
    for i in 0..segments.len() {
        let si = &segments[i];
        for j in (i + 1)..segments.len() {
            let sj = &segments[j];
            if (equal_with_radius(si.a, sj.a, Epsilon::UNKNOWN_001)
                && equal_with_radius(si.b, sj.b, Epsilon::UNKNOWN_001))
                || (equal_with_radius(si.a, sj.b, Epsilon::UNKNOWN_001)
                    && equal_with_radius(si.b, sj.a, Epsilon::UNKNOWN_001))
            {
                remove[j] = true;
            }
        }
    }

    *segments = segments
        .iter()
        .enumerate()
        .filter_map(|(index, segment)| (!remove[index]).then_some(segment.clone()))
        .collect();
}
