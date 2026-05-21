use super::{IoError, Result};
use crate::geometry::{
    Circle, LineColor, LineSegment, Point, Polygon, angle, equal, point_rotate_scaled,
};
use crate::model::{
    CreasePatternModel, GridState, TextElement, custom_color_from_hex, custom_color_hex,
    fold_angle_for_line_color, fold_assignment_for_line_color, line_color_for_fold_assignment,
};
use serde_json::{Value, json};
use treemaker_fold::FoldDocument;

const ORIEDITA_VERSION: &str = "dev";

/// Import a FOLD JSON document with Oriedita extension fields.
pub fn import_fold_json(input: &str) -> Result<CreasePatternModel> {
    let fold = serde_json::from_str::<FoldDocument>(input)?;
    import_fold_document(&fold)
}

pub fn import_fold_document(fold: &FoldDocument) -> Result<CreasePatternModel> {
    let mut model = CreasePatternModel::default();
    let edge_colors = string_array_extra(fold, "oriedita:edges_colors")?;
    let mut bounds = FoldImportBounds::default();

    for (index, edge) in fold.edges_vertices.iter().enumerate() {
        let a = vertex_point(fold, edge[0])?;
        let b = vertex_point(fold, edge[1])?;
        bounds.include(a);
        bounds.include(b);
        let mut segment = LineSegment::with_color(
            a,
            b,
            line_color_for_fold_assignment(fold.assignment_for_edge(index)),
        );

        if let Some(hex) = edge_colors.as_ref().and_then(|colors| colors.get(index))
            && !hex.is_empty()
        {
            segment = segment.with_customized_color(custom_color_from_hex(hex)?);
        }

        model.add_line_segment(segment);
    }
    normalize_imported_fold_lines(&mut model, bounds);

    import_circles(fold, &mut model)?;
    import_texts(fold, &mut model)?;
    import_grid(fold, &mut model)?;

    Ok(model)
}

/// Export a FOLD document with Oriedita extension fields.
pub fn export_fold_document(model: &CreasePatternModel, title: Option<String>) -> FoldDocument {
    let topology = export_topology(model);
    let mut assignments = Vec::new();
    let mut fold_angles = Vec::new();
    let mut edge_custom_colors = Vec::new();

    for segment in &topology.segments {
        assignments.push(fold_assignment_for_line_color(segment.color));
        fold_angles.push(Some(fold_angle_for_line_color(segment.color)));
        edge_custom_colors.push(if segment.customized == 1 {
            custom_color_hex(segment.customized_color)
        } else {
            String::new()
        });
    }

    let mut fold = FoldDocument::new(
        topology
            .vertices
            .iter()
            .map(|point| vec![point.x, point.y])
            .collect(),
        topology.edges,
    );
    fold.file_spec = Some(1.1);
    fold.file_creator = Some("oriedita".to_string());
    fold.frame_title = title;
    fold.edges_assignment = assignments;
    fold.edges_fold_angle = fold_angles;
    if topology.include_faces {
        fold.faces_vertices = topology.faces_vertices;
        fold.faces_edges = topology.faces_edges;
    }

    fold.extra.insert(
        "oriedita:version".to_string(),
        Value::String(ORIEDITA_VERSION.to_string()),
    );
    fold.extra.insert(
        "oriedita:edges_colors".to_string(),
        json!(edge_custom_colors),
    );
    export_circles(model, &mut fold);
    export_texts(model, &mut fold);
    fold.extra.insert(
        "oriedita:grid_size".to_string(),
        json!(model.grid.grid_size),
    );
    fold.extra.insert(
        "oriedita:grid_style".to_string(),
        json!(model.grid.base_state.state()),
    );

    fold
}

pub fn export_fold_json(model: &CreasePatternModel, title: Option<String>) -> Result<String> {
    Ok(serde_json::to_string_pretty(&export_fold_document(
        model, title,
    ))?)
}

#[derive(Debug, Clone)]
struct ExportTopology {
    segments: Vec<LineSegment>,
    vertices: Vec<Point>,
    edges: Vec<[usize; 2]>,
    include_faces: bool,
    faces_vertices: Vec<Vec<usize>>,
    faces_edges: Vec<Vec<usize>>,
}

fn export_topology(model: &CreasePatternModel) -> ExportTopology {
    let segments = if model.line_segments.is_empty() {
        vec![LineSegment::with_color(
            Point::new(0.0, 0.0),
            Point::new(0.0, 0.0),
            LineColor::Black0,
        )]
    } else {
        model.line_segments.clone()
    };

    let mut vertices = Vec::new();
    let mut edges = Vec::with_capacity(segments.len());
    for segment in &segments {
        let a = topology_vertex_index(&mut vertices, segment.a);
        let b = topology_vertex_index(&mut vertices, segment.b);
        edges.push([a, b]);
    }

    let (include_faces, faces_vertices, faces_edges) = topology_faces(&vertices, &edges);

    ExportTopology {
        segments,
        vertices,
        edges,
        include_faces,
        faces_vertices,
        faces_edges,
    }
}

fn topology_vertex_index(vertices: &mut Vec<Point>, point: Point) -> usize {
    if let Some(index) = vertices
        .iter()
        .position(|candidate| equal(*candidate, point))
    {
        return index;
    }

    vertices.push(point);
    vertices.len() - 1
}

fn topology_faces(
    vertices: &[Point],
    edges: &[[usize; 2]],
) -> (bool, Vec<Vec<usize>>, Vec<Vec<usize>>) {
    let mut point_linking = vec![Vec::<usize>::new(); vertices.len()];
    for edge in edges {
        if edge[0] < point_linking.len() && edge[1] < point_linking.len() {
            point_linking[edge[0]].push(edge[1]);
            point_linking[edge[1]].push(edge[0]);
        }
    }

    let mut face_point_map = vec![Vec::<usize>::new(); vertices.len()];
    let mut faces = Vec::<Vec<usize>>::new();

    for edge in edges {
        let begin = edge[0];
        let end = edge[1];

        let forward = topology_face_request(begin, end, vertices, &point_linking);
        if topology_should_add_face(&forward, begin, vertices, &faces, &face_point_map) {
            topology_add_face(forward, &mut faces, &mut face_point_map);
        }

        let reverse = topology_face_request(end, begin, vertices, &point_linking);
        if topology_should_add_face(&reverse, begin, vertices, &faces, &face_point_map) {
            topology_add_face(reverse, &mut faces, &mut face_point_map);
        }
    }

    let euler = faces.len() as isize - edges.len() as isize + vertices.len() as isize;
    let include_faces = euler == 1 || (euler - 1).abs() as f64 <= 0.005 * faces.len() as f64;
    if !include_faces {
        return (false, Vec::new(), Vec::new());
    }

    let faces_edges = faces
        .iter()
        .map(|face| topology_face_edges(face, edges))
        .collect();

    (true, faces, faces_edges)
}

fn topology_face_request(
    start: usize,
    end: usize,
    vertices: &[Point],
    point_linking: &[Vec<usize>],
) -> Vec<usize> {
    if start >= vertices.len() || end >= vertices.len() {
        return Vec::new();
    }

    let mut face = vec![start, end];
    let mut next = topology_r_point(start, end, vertices, point_linking);
    let mut added_after_seed = false;

    loop {
        let Some(next_point) = next else {
            if added_after_seed {
                // Oriedita `Face` stores a sentinel point id 0; after at least
                // one added vertex, falling off a dangling branch still returns
                // the partial face because that sentinel is "contained".
                topology_align_face(&mut face);
                return face;
            }
            return Vec::new();
        };
        if face.contains(&next_point) {
            topology_align_face(&mut face);
            return face;
        }

        face.push(next_point);
        added_after_seed = true;
        let count = face.len();
        next = topology_r_point(face[count - 2], face[count - 1], vertices, point_linking);
    }
}

fn topology_r_point(
    previous: usize,
    current: usize,
    vertices: &[Point],
    point_linking: &[Vec<usize>],
) -> Option<usize> {
    let linked_points = point_linking.get(current)?;
    if !point_linking
        .get(previous)
        .is_some_and(|linked| linked.contains(&current))
    {
        return None;
    }

    let mut result = None;
    let mut best_angle = 876.0;
    for candidate in linked_points {
        if *candidate == previous {
            continue;
        }
        let candidate_angle = angle((
            vertices[current],
            vertices[previous],
            vertices[current],
            vertices[*candidate],
        ));
        if candidate_angle <= best_angle {
            result = Some(*candidate);
            best_angle = candidate_angle;
        }
    }

    result
}

fn topology_align_face(face: &mut Vec<usize>) {
    let Some(minimum) = face.iter().copied().min() else {
        return;
    };
    while face.first().copied() != Some(minimum) {
        let first = face.remove(0);
        face.push(first);
    }
}

fn topology_should_add_face(
    face: &[usize],
    begin: usize,
    vertices: &[Point],
    faces: &[Vec<usize>],
    face_point_map: &[Vec<usize>],
) -> bool {
    if face.is_empty()
        || topology_face_area(face, vertices) <= 0.0
        || face_point_map
            .get(begin)
            .is_some_and(|existing| existing.iter().any(|index| faces[*index] == face))
    {
        return false;
    }

    true
}

fn topology_add_face(
    face: Vec<usize>,
    faces: &mut Vec<Vec<usize>>,
    face_point_map: &mut [Vec<usize>],
) {
    let face_index = faces.len();
    for point in &face {
        if let Some(entries) = face_point_map.get_mut(*point) {
            entries.push(face_index);
        }
    }
    faces.push(face);
}

fn topology_face_area(face: &[usize], vertices: &[Point]) -> f64 {
    let points = face
        .iter()
        .filter_map(|index| vertices.get(*index).copied())
        .collect::<Vec<_>>();
    Polygon::new(points).calculate_area()
}

fn topology_face_edges(face: &[usize], edges: &[[usize; 2]]) -> Vec<usize> {
    if face.is_empty() {
        return Vec::new();
    }

    let mut face_edges = Vec::with_capacity(face.len());
    let first = face[0];
    let last = face[face.len() - 1];
    face_edges.push(topology_find_edge(first, last, edges));
    for index in 1..face.len() {
        face_edges.push(topology_find_edge(face[index], face[index - 1], edges));
    }
    face_edges
}

fn topology_find_edge(a: usize, b: usize, edges: &[[usize; 2]]) -> usize {
    edges
        .iter()
        .position(|edge| (edge[0] == a && edge[1] == b) || (edge[0] == b && edge[1] == a))
        .unwrap_or(usize::MAX)
}

fn vertex_point(fold: &FoldDocument, index: usize) -> Result<Point> {
    let coords = fold
        .vertices_coords
        .get(index)
        .ok_or_else(|| IoError::InvalidField {
            field: "vertices_coords",
            message: format!("edge references missing vertex {index}"),
        })?;
    if coords.len() < 2 {
        return Err(IoError::InvalidField {
            field: "vertices_coords",
            message: format!("vertex {index} has fewer than two coordinates"),
        });
    }
    Ok(Point::new(coords[0], coords[1]))
}

#[derive(Debug, Clone, Copy)]
struct FoldImportBounds {
    min_x: f64,
    min_y: f64,
    max_y: f64,
    has_points: bool,
}

impl Default for FoldImportBounds {
    fn default() -> Self {
        Self {
            min_x: f64::MAX,
            min_y: f64::MAX,
            max_y: f64::from_bits(1),
            has_points: false,
        }
    }
}

impl FoldImportBounds {
    fn include(&mut self, point: Point) {
        self.min_x = self.min_x.min(point.x);
        self.min_y = self.min_y.min(point.y);
        self.max_y = self.max_y.max(point.y);
        self.has_points = true;
    }
}

fn normalize_imported_fold_lines(model: &mut CreasePatternModel, bounds: FoldImportBounds) {
    if !bounds.has_points {
        return;
    }

    let source_a = Point::new(bounds.min_x, bounds.min_y);
    let source_b = Point::new(bounds.min_x, bounds.max_y);
    let target_a = Point::new(-200.0, -200.0);
    let target_b = Point::new(-200.0, 200.0);
    let rotation = angle((source_a, source_b, target_a, target_b));
    let scale = target_a.distance(target_b) / source_a.distance(source_b);
    let delta = Point::new(target_a.x - source_a.x, target_a.y - source_a.y);

    for segment in &mut model.line_segments {
        segment.a = normalize_imported_fold_point(segment.a, source_a, rotation, scale, delta);
        segment.b = normalize_imported_fold_point(segment.b, source_a, rotation, scale, delta);
    }
}

fn normalize_imported_fold_point(
    point: Point,
    source_a: Point,
    rotation: f64,
    scale: f64,
    delta: Point,
) -> Point {
    point_rotate_scaled(source_a, point, rotation, scale).move_by(delta)
}

fn import_circles(fold: &FoldDocument, model: &mut CreasePatternModel) -> Result<()> {
    let Some(coords) = point_array_extra(fold, "oriedita:circles_coords")? else {
        return Ok(());
    };
    let radii = f64_array_extra(fold, "oriedita:circles_radii")?.unwrap_or_default();
    let colors = string_array_extra(fold, "oriedita:circles_colors")?.unwrap_or_default();
    let custom_colors =
        string_array_extra(fold, "oriedita:circles_custom_colors")?.unwrap_or_default();

    for (index, center) in coords.into_iter().enumerate() {
        let radius = radii.get(index).copied().unwrap_or_default();
        let color = colors
            .get(index)
            .and_then(|value| value.parse::<LineColor>().ok())
            .unwrap_or(LineColor::Black0);
        let mut circle = Circle::from_center(center, radius, color);

        if let Some(hex) = custom_colors.get(index)
            && !hex.is_empty()
        {
            circle = circle.with_customized_color(custom_color_from_hex(hex)?);
        }

        model.add_circle(circle);
    }

    Ok(())
}

fn import_texts(fold: &FoldDocument, model: &mut CreasePatternModel) -> Result<()> {
    let Some(coords) = point_array_extra(fold, "oriedita:texts_coords")? else {
        return Ok(());
    };
    let texts = string_array_extra(fold, "oriedita:texts_text")?.unwrap_or_default();

    for (index, position) in coords.into_iter().enumerate() {
        if let Some(text) = texts.get(index) {
            model.add_text(TextElement::new(position.x, position.y, text.clone()));
        }
    }

    Ok(())
}

fn import_grid(fold: &FoldDocument, model: &mut CreasePatternModel) -> Result<()> {
    model.grid.base_state = GridState::Hidden;
    if let Some(size) = integer_extra(fold, "oriedita:grid_size")? {
        model.grid.set_grid_size(size);
    }
    if let Some(style) = integer_extra(fold, "oriedita:grid_style")? {
        model.grid.base_state = GridState::from_state(style)?;
    }
    Ok(())
}

fn export_circles(model: &CreasePatternModel, fold: &mut FoldDocument) {
    if model.circles.is_empty() {
        return;
    }

    fold.extra.insert(
        "oriedita:circles_coords".to_string(),
        json!(
            model
                .circles
                .iter()
                .map(|circle| vec![circle.x, circle.y])
                .collect::<Vec<_>>()
        ),
    );
    fold.extra.insert(
        "oriedita:circles_radii".to_string(),
        json!(
            model
                .circles
                .iter()
                .map(|circle| circle.r)
                .collect::<Vec<_>>()
        ),
    );
    fold.extra.insert(
        "oriedita:circles_colors".to_string(),
        json!(
            model
                .circles
                .iter()
                .map(|circle| circle.color.to_string())
                .collect::<Vec<_>>()
        ),
    );
    fold.extra.insert(
        "oriedita:circles_custom_colors".to_string(),
        json!(
            model
                .circles
                .iter()
                .map(|circle| {
                    if circle.customized == 1 {
                        custom_color_hex(circle.customized_color)
                    } else {
                        String::new()
                    }
                })
                .collect::<Vec<_>>()
        ),
    );
}

fn export_texts(model: &CreasePatternModel, fold: &mut FoldDocument) {
    if model.texts.is_empty() {
        return;
    }

    fold.extra.insert(
        "oriedita:texts_coords".to_string(),
        json!(
            model
                .texts
                .iter()
                .map(|text| vec![text.x.0, text.y.0])
                .collect::<Vec<_>>()
        ),
    );
    fold.extra.insert(
        "oriedita:texts_text".to_string(),
        json!(
            model
                .texts
                .iter()
                .map(|text| text.text.clone())
                .collect::<Vec<_>>()
        ),
    );
}

fn string_array_extra(fold: &FoldDocument, key: &'static str) -> Result<Option<Vec<String>>> {
    let Some(value) = fold.extra.get(key) else {
        return Ok(None);
    };
    let array = value.as_array().ok_or_else(|| IoError::InvalidField {
        field: key,
        message: "expected array".to_string(),
    })?;
    array
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| IoError::InvalidField {
                    field: key,
                    message: "expected string array".to_string(),
                })
        })
        .collect::<Result<Vec<_>>>()
        .map(Some)
}

fn f64_array_extra(fold: &FoldDocument, key: &'static str) -> Result<Option<Vec<f64>>> {
    let Some(value) = fold.extra.get(key) else {
        return Ok(None);
    };
    let array = value.as_array().ok_or_else(|| IoError::InvalidField {
        field: key,
        message: "expected array".to_string(),
    })?;
    array
        .iter()
        .map(|item| {
            item.as_f64().ok_or_else(|| IoError::InvalidField {
                field: key,
                message: "expected number array".to_string(),
            })
        })
        .collect::<Result<Vec<_>>>()
        .map(Some)
}

fn point_array_extra(fold: &FoldDocument, key: &'static str) -> Result<Option<Vec<Point>>> {
    let Some(value) = fold.extra.get(key) else {
        return Ok(None);
    };
    let array = value.as_array().ok_or_else(|| IoError::InvalidField {
        field: key,
        message: "expected array".to_string(),
    })?;
    array
        .iter()
        .map(|item| {
            let coords = item.as_array().ok_or_else(|| IoError::InvalidField {
                field: key,
                message: "expected coordinate array".to_string(),
            })?;
            if coords.len() < 2 {
                return Err(IoError::InvalidField {
                    field: key,
                    message: "coordinate array has fewer than two numbers".to_string(),
                });
            }
            let x = coords[0].as_f64().ok_or_else(|| IoError::InvalidField {
                field: key,
                message: "x coordinate is not a number".to_string(),
            })?;
            let y = coords[1].as_f64().ok_or_else(|| IoError::InvalidField {
                field: key,
                message: "y coordinate is not a number".to_string(),
            })?;
            Ok(Point::new(x, y))
        })
        .collect::<Result<Vec<_>>>()
        .map(Some)
}

fn integer_extra(fold: &FoldDocument, key: &'static str) -> Result<Option<i32>> {
    let Some(value) = fold.extra.get(key) else {
        return Ok(None);
    };
    let number = value.as_i64().ok_or_else(|| IoError::InvalidField {
        field: key,
        message: "expected integer".to_string(),
    })?;
    i32::try_from(number)
        .map(Some)
        .map_err(|error| IoError::InvalidField {
            field: key,
            message: error.to_string(),
        })
}
