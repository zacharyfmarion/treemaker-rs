use crate::fold_graph::{FacePositions, FoldGraph};
use crate::geometry::{
    Epsilon, LineColor, LineSegment, Point, Polygon, PolygonIntersection, equal, equal_with_radius,
};
use crate::model::CreasePatternModel;
use crate::operations::arrangement::divide_intersections;
use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubFace {
    pub face_ids: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubFaceConfiguration {
    pub subfaces: Vec<SubFace>,
    pub reduced_subface_indices: Vec<usize>,
    pub face_id_count_max: usize,
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

/// Oriedita `FoldedFigure_Configurator.configureSubFaces()` for the folded
/// wireframe and its subdivided subface arrangement, without hierarchy solving.
pub fn configure_subfaces_from_segments(
    segments: &[LineSegment],
    starting_face_id: i32,
) -> Option<SubFaceConfiguration> {
    let folded = estimate_wireframe_from_segments(segments, starting_face_id)?;
    let folded_segments = folded_wireframe_segments(&folded);
    let prepared_segments = prepare_subface_segments(&folded_segments);
    let subface_graph = FoldGraph::from_segments(&prepared_segments, true);
    if subface_graph.faces.is_empty() {
        return None;
    }

    Some(configure_subfaces(&folded, &subface_graph))
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

fn configure_subfaces(folded: &FoldedWireframe, subface_graph: &FoldGraph) -> SubFaceConfiguration {
    let face_polygons = folded
        .faces
        .iter()
        .map(|face| {
            Polygon::new(
                face.iter()
                    .filter_map(|point| folded.points.get(*point).copied())
                    .collect(),
            )
        })
        .collect::<Vec<_>>();

    let mut frequency = vec![0usize; face_polygons.len()];
    let mut subfaces = Vec::with_capacity(subface_graph.faces.len());
    for subface in &subface_graph.faces {
        let inside_point = subface_polygon(subface_graph, subface).inside_point_find();
        let mut face_ids = Vec::new();
        for (face_index, polygon) in face_polygons.iter().enumerate() {
            if polygon.inside(inside_point) == PolygonIntersection::Inside {
                face_ids.push(face_index);
                frequency[face_index] += 1;
            }
        }
        subfaces.push(SubFace { face_ids });
    }

    let face_id_count_max = subfaces
        .iter()
        .map(|subface| subface.face_ids.len())
        .max()
        .unwrap_or(0);
    let reduced_subface_indices = reduce_subface_set(&subfaces, &frequency);

    SubFaceConfiguration {
        subfaces,
        reduced_subface_indices,
        face_id_count_max,
    }
}

fn folded_wireframe_segments(folded: &FoldedWireframe) -> Vec<LineSegment> {
    folded
        .lines
        .iter()
        .filter_map(|line| {
            let a = folded.points.get(line.begin).copied()?;
            let b = folded.points.get(line.end).copied()?;
            Some(LineSegment::with_color(a, b, line.color))
        })
        .collect()
}

fn subface_polygon(graph: &FoldGraph, face: &[usize]) -> Polygon {
    Polygon::new(
        face.iter()
            .filter_map(|point| graph.points.get(*point).copied())
            .collect(),
    )
}

fn reduce_subface_set(subfaces: &[SubFace], frequency: &[usize]) -> Vec<usize> {
    let mut sorted = (0..subfaces.len()).collect::<Vec<_>>();
    sorted.sort_by(|a, b| {
        subfaces[*b]
            .face_ids
            .len()
            .cmp(&subfaces[*a].face_ids.len())
            .then_with(|| a.cmp(b))
    });

    let mut reduced_indices: Vec<usize> = Vec::new();
    let mut face_to_reduced = HashMap::<usize, Vec<usize>>::new();
    for subface_index in sorted {
        let subface = &subfaces[subface_index];
        if subface.face_ids.is_empty() {
            continue;
        }

        let mut ids = subface.face_ids.clone();
        ids.sort_by(|a, b| {
            frequency
                .get(*a)
                .copied()
                .unwrap_or_default()
                .cmp(&frequency.get(*b).copied().unwrap_or_default())
        });

        let mut is_not_subset = !face_to_reduced.contains_key(&ids[0]);
        if !is_not_subset && let Some(candidates) = face_to_reduced.get(&ids[0]) {
            is_not_subset = !candidates.iter().any(|candidate| {
                let reduced = &subfaces[reduced_indices[*candidate]];
                ids.iter().skip(1).all(|id| reduced.face_ids.contains(id))
            });
        }

        if is_not_subset {
            let reduced_index = reduced_indices.len();
            reduced_indices.push(subface_index);
            for id in ids {
                face_to_reduced.entry(id).or_default().push(reduced_index);
            }
        }
    }

    reduced_indices
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
