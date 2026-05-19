use std::collections::{BTreeMap, BTreeSet};

use crate::avl::AvlSet;
use crate::math::{self, Point};
use crate::{FlatFoldError, NormalizeOptions, NormalizedFold, PaperSide, Result};
use treemaker_fold::{Assignment, FoldDocument};

type Edge = [usize; 2];
type SplitGraph = (Vec<Point>, Vec<Edge>, Vec<Vec<usize>>);
type SplitGraphWithEps = (Vec<Point>, Vec<Edge>, Vec<Vec<usize>>, usize);
type FaceAdjacency = (Vec<Vec<usize>>, Vec<Vec<usize>>);

pub(crate) fn normalize_document(
    document: &FoldDocument,
    options: NormalizeOptions,
) -> Result<NormalizedFold> {
    let mut vertices = document_points(document)?;
    let mut edges = document_edges(document, vertices.len())?;
    let mut assignments = document_assignments(document, edges.len())?;
    let mut faces = if document.faces_vertices.is_empty() {
        None
    } else {
        Some(document.faces_vertices.clone())
    };
    let mut vertex_vertices: Vec<Vec<usize>> = Vec::new();

    if let Some(existing_faces) = faces.as_mut() {
        sort_faces(existing_faces, &vertices);
        vertex_vertices = vertex_vertices_from_faces(&vertices, existing_faces);
    } else {
        let lines = edges
            .iter()
            .map(|[a, b]| [vertices[*a], vertices[*b]])
            .collect::<Vec<_>>();
        let (next_vertices, next_edges, edge_lines, _) = lines_to_vertices_edges(&lines)?;
        vertices = next_vertices;
        edges = next_edges;
        assignments = edge_lines
            .iter()
            .map(|lines| {
                lines
                    .iter()
                    .find_map(|line| {
                        let assignment = assignments[*line];
                        (assignment != Assignment::Flat).then_some(assignment)
                    })
                    .unwrap_or(Assignment::Flat)
            })
            .collect();
    }

    if faces.is_none() {
        if options.side == PaperSide::Front {
            flip_assignments(&mut assignments);
        } else {
            flip_y(&mut vertices);
        }
        let (vv, fv) = vertices_edges_to_vertices_faces(&vertices, &edges)?;
        vertex_vertices = vv;
        faces = Some(fv);
    } else if let Some(faces_ref) = faces.as_mut() {
        if !faces_ref.is_empty() && polygon_area_for_face(&vertices, &faces_ref[0]) < 0.0 {
            flip_assignments(&mut assignments);
            reverse_faces(faces_ref);
        }
        if options.side == PaperSide::Back {
            flip_assignments(&mut assignments);
            reverse_faces(faces_ref);
            flip_y(&mut vertices);
        }
    }

    let mut faces = faces.unwrap_or_default();
    let (mut edges_faces, mut faces_edges) = edges_faces_from_faces(&edges, &faces)?;
    if faces.len() > 1 {
        let filtered = faces
            .iter()
            .enumerate()
            .filter_map(|(face_index, face)| {
                let is_hole = faces_edges[face_index]
                    .iter()
                    .all(|edge| assignments[*edge] == Assignment::Boundary);
                (!is_hole).then(|| face.clone())
            })
            .collect::<Vec<_>>();
        if filtered.len() != faces.len() {
            faces = filtered;
            (edges_faces, faces_edges) = edges_faces_from_faces(&edges, &faces)?;
        }
    }
    for (edge_index, faces_for_edge) in edges_faces.iter().enumerate() {
        if faces_for_edge.len() == 1 {
            assignments[edge_index] = Assignment::Boundary;
        }
    }

    let mut normalized = document.clone();
    normalized.vertices_coords = vertices.iter().map(|[x, y]| vec![*x, *y]).collect();
    normalized.edges_vertices = edges;
    normalized.edges_assignment = assignments;
    normalized.edges_fold_angle.clear();
    normalized.faces_vertices = faces;
    normalized.edges_faces = edges_faces;
    normalized.faces_edges = faces_edges;

    Ok(NormalizedFold {
        document: normalized,
        vertex_vertices,
    })
}

fn document_points(document: &FoldDocument) -> Result<Vec<Point>> {
    if document.vertices_coords.is_empty() {
        return Err(FlatFoldError::InvalidInput(
            "FOLD document must contain vertices_coords".to_string(),
        ));
    }
    document
        .vertices_coords
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let x = point.first().copied().ok_or_else(|| {
                FlatFoldError::InvalidInput(format!("vertex {index} is missing x coordinate"))
            })?;
            let y = point.get(1).copied().ok_or_else(|| {
                FlatFoldError::InvalidInput(format!("vertex {index} is missing y coordinate"))
            })?;
            Ok([x, y])
        })
        .collect()
}

fn document_edges(document: &FoldDocument, vertex_count: usize) -> Result<Vec<Edge>> {
    if document.edges_vertices.is_empty() {
        return Err(FlatFoldError::InvalidInput(
            "FOLD document must contain edges_vertices".to_string(),
        ));
    }
    let mut out = Vec::with_capacity(document.edges_vertices.len());
    for (edge_index, [a, b]) in document.edges_vertices.iter().copied().enumerate() {
        if a >= vertex_count || b >= vertex_count {
            return Err(FlatFoldError::InvalidInput(format!(
                "edge {edge_index} references vertex outside 0..{vertex_count}"
            )));
        }
        out.push(if a < b { [a, b] } else { [b, a] });
    }
    Ok(out)
}

fn document_assignments(document: &FoldDocument, edge_count: usize) -> Result<Vec<Assignment>> {
    if document.edges_assignment.is_empty() {
        return Ok(vec![Assignment::Unassigned; edge_count]);
    }
    if document.edges_assignment.len() != edge_count {
        return Err(FlatFoldError::InvalidInput(format!(
            "edges_assignment length {} does not match edges_vertices length {edge_count}",
            document.edges_assignment.len()
        )));
    }
    Ok(document.edges_assignment.clone())
}

fn lines_to_vertices_edges(lines: &[[Point; 2]]) -> Result<SplitGraphWithEps> {
    let d = math::min_line_length(lines);
    let target_repeat = 3usize;
    let max_steps = 25usize;
    let mut best_repeat = 0usize;
    let mut best_i = 1usize;
    let mut last_vertices = 0usize;
    let mut last_edges = 0usize;
    let mut repeat = 0usize;
    for i in 3..(max_steps + 3) {
        let eps = d / 2f64.powi(i as i32);
        if eps < math::FLOAT_EPS {
            break;
        }
        let (vertices, edges, _) = lines_eps_to_vertices_edges(lines, eps)?;
        if vertices.is_empty() {
            last_vertices = 0;
            last_edges = 0;
            repeat = 0;
            continue;
        }
        repeat = if vertices.len() == last_vertices && edges.len() == last_edges {
            repeat + 1
        } else {
            1
        };
        last_vertices = vertices.len();
        last_edges = edges.len();
        if repeat <= best_repeat {
            continue;
        }
        best_repeat = repeat;
        best_i = i;
        if best_repeat == target_repeat {
            break;
        }
    }
    let eps_i = best_i - best_repeat + 1;
    let eps = d / 2f64.powi(eps_i as i32);
    let (vertices, edges, edge_lines) = lines_eps_to_vertices_edges(lines, eps)?;
    Ok((vertices, edges, edge_lines, eps_i))
}

fn lines_eps_to_vertices_edges(lines: &[[Point; 2]], eps: f64) -> Result<SplitGraph> {
    let mut vertices = vec![[f64::NEG_INFINITY, f64::NEG_INFINITY]];
    let mut vertex_lines: Vec<Vec<usize>> = vec![Vec::new()];
    let mut vertex_point: Vec<Option<usize>> = vec![None];
    let mut line_vertices: Vec<Edge> = Vec::with_capacity(lines.len());
    let mut line_units: Vec<Point> = Vec::with_capacity(lines.len());
    let mut line_angles: Vec<f64> = Vec::with_capacity(lines.len());
    let mut line_distances: Vec<f64> = Vec::with_capacity(lines.len());
    let mut q = AvlSet::default();

    for (line_index, [mut p, mut q_point]) in lines.iter().copied().enumerate() {
        if point_comp(p, q_point, eps) > 0 {
            std::mem::swap(&mut p, &mut q_point);
        }
        let vi = insert_sweep_point(
            &mut vertices,
            &mut vertex_lines,
            &mut vertex_point,
            &mut q,
            p,
            eps,
        );
        let vj = insert_sweep_point(
            &mut vertices,
            &mut vertex_lines,
            &mut vertex_point,
            &mut q,
            q_point,
            eps,
        );
        line_vertices.push([vi, vj]);
        let d = math::sub(vertices[vj], vertices[vi]);
        let unit = math::unit(d)?;
        line_units.push(unit);
        line_angles.push(if d[1] < eps { 0.0 } else { math::angle(d) });
        line_distances.push(math::dot(math::perp(unit), vertices[vj]));
        vertex_lines[vi].push(line_index);
    }

    let bbox = math::bounding_box(&vertices[1..])?;
    let height = bbox[1][1] - bbox[0][1];
    let width = bbox[1][0] - bbox[0][0];
    let scale = height.max(width);

    let mut segment_vertices: Vec<[Option<usize>; 2]> = vec![[None, None]];
    let mut segment_units: Vec<Point> = vec![[-1.0, 0.0]];
    let mut segment_angles = vec![f64::INFINITY];
    let mut segment_distances: Vec<Option<f64>> = vec![None];
    let mut segment_lines: Vec<Vec<usize>> = vec![Vec::new()];
    let mut t = AvlSet::default();
    let mut curr: usize;
    let mut points = Vec::new();

    while q.len() > 0 {
        let Some(vi) = q.remove_next(0, |a, b| point_comp(vertices[a], vertices[b], eps)) else {
            break;
        };
        curr = vi;
        let v = vertices[vi];
        let mut entering = Vec::new();
        segment_vertices[0][0] = Some(vi);
        segment_distances[0] = Some(math::dot(math::perp(segment_units[0]), v));
        let should_remove_horizontal = t
            .prev(0, |a, b| {
                segment_comp(
                    a,
                    b,
                    curr,
                    &vertices,
                    &segment_vertices,
                    &segment_units,
                    &segment_angles,
                    &segment_distances,
                    eps,
                )
            })
            .is_some_and(|si| segment_angles[si] == 0.0);
        if should_remove_horizontal
            && let Some(si) = t.remove_prev(0, |a, b| {
                segment_comp(
                    a,
                    b,
                    curr,
                    &vertices,
                    &segment_vertices,
                    &segment_units,
                    &segment_angles,
                    &segment_distances,
                    eps,
                )
            })
        {
            entering.push(si);
        }
        loop {
            let next = t.next(0, |a, b| {
                segment_comp(
                    a,
                    b,
                    curr,
                    &vertices,
                    &segment_vertices,
                    &segment_units,
                    &segment_angles,
                    &segment_distances,
                    eps,
                )
            });
            let Some(si) = next else {
                break;
            };
            if !on_line(vi, si, &vertices, &segment_units, &segment_distances, eps) {
                break;
            }
            if let Some(removed) = t.remove_next(0, |a, b| {
                segment_comp(
                    a,
                    b,
                    curr,
                    &vertices,
                    &segment_vertices,
                    &segment_units,
                    &segment_angles,
                    &segment_distances,
                    eps,
                )
            }) {
                entering.push(removed);
            }
        }

        if vertex_lines[vi].is_empty() && entering.len() < 2 {
            if entering.is_empty() {
                continue;
            }
            let si = entering[0];
            let mut ends = false;
            for line_index in &segment_lines[si] {
                if point_comp(vertices[line_vertices[*line_index][1]], vertices[vi], eps) <= 0 {
                    ends = true;
                    break;
                }
            }
            if !ends {
                t.insert(si, |a, b| {
                    segment_comp(
                        a,
                        b,
                        curr,
                        &vertices,
                        &segment_vertices,
                        &segment_units,
                        &segment_angles,
                        &segment_distances,
                        eps,
                    )
                });
                continue;
            }
        }

        if entering.len() == 1 {
            let s0 = entering[0];
            let mut all_parallel = true;
            for line_index in &vertex_lines[vi] {
                if !on_line(
                    line_vertices[*line_index][1],
                    s0,
                    &vertices,
                    &segment_units,
                    &segment_distances,
                    eps,
                ) {
                    all_parallel = false;
                    break;
                }
            }
            if all_parallel {
                t.insert(s0, |a, b| {
                    segment_comp(
                        a,
                        b,
                        curr,
                        &vertices,
                        &segment_vertices,
                        &segment_units,
                        &segment_angles,
                        &segment_distances,
                        eps,
                    )
                });
                let passing = vertex_lines[vi].clone();
                segment_lines[s0].extend(passing);
                continue;
            }
        }

        vertex_point[vi] = Some(points.len());
        points.push(v);

        for si in entering {
            segment_vertices[si][1] = Some(vi);
            let lines_for_segment = segment_lines[si].clone();
            for line_index in lines_for_segment {
                if point_comp(vertices[line_vertices[line_index][1]], vertices[vi], eps) <= 0 {
                    continue;
                }
                vertex_lines[vi].push(line_index);
            }
        }
        let vi_point = vertices[vi];
        vertex_lines[vi].sort_by(|i, j| {
            let dj = math::distsq(vi_point, vertices[line_vertices[*j][1]]);
            let di = math::distsq(vi_point, vertices[line_vertices[*i][1]]);
            if dj == di {
                i.cmp(j)
            } else if dj < di {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });
        let starting_lines = vertex_lines[vi].clone();
        for line_index in starting_lines {
            let si = segment_vertices.len();
            segment_vertices.push([Some(vi), Some(line_vertices[line_index][1])]);
            segment_units.push(line_units[line_index]);
            segment_angles.push(line_angles[line_index]);
            segment_distances.push(Some(line_distances[line_index]));
            segment_lines.push(vec![line_index]);
            let existing = t.insert(si, |a, b| {
                segment_comp(
                    a,
                    b,
                    curr,
                    &vertices,
                    &segment_vertices,
                    &segment_units,
                    &segment_angles,
                    &segment_distances,
                    eps,
                )
            });
            if let Some(sj) = existing {
                segment_vertices.pop();
                segment_units.pop();
                segment_angles.pop();
                segment_distances.pop();
                segment_lines.pop();
                segment_lines[sj].push(line_index);
            } else {
                segment_vertices[si][1] = None;
            }
        }

        for sentinel_angle in [-1.0, f64::INFINITY] {
            segment_angles[0] = sentinel_angle;
            let left = t.prev(0, |a, b| {
                segment_comp(
                    a,
                    b,
                    curr,
                    &vertices,
                    &segment_vertices,
                    &segment_units,
                    &segment_angles,
                    &segment_distances,
                    eps,
                )
            });
            let right = t.next(0, |a, b| {
                segment_comp(
                    a,
                    b,
                    curr,
                    &vertices,
                    &segment_vertices,
                    &segment_units,
                    &segment_angles,
                    &segment_distances,
                    eps,
                )
            });
            let (Some(left), Some(right)) = (left, right) else {
                continue;
            };
            let Some(vl) = segment_vertices[left][0] else {
                continue;
            };
            let Some(vr) = segment_vertices[right][0] else {
                continue;
            };
            let x = line_intersect(
                vertices[vl],
                math::add(vertices[vl], math::mul(segment_units[left], scale)),
                vertices[vr],
                math::add(vertices[vr], math::mul(segment_units[right], scale)),
                eps,
            );
            let Some(x) = x else {
                continue;
            };
            let c = point_comp(x, v, eps);
            if c == 0 || (c < 0 && (x[1] - v[1]) < eps) {
                continue;
            }
            let vx = vertices.len();
            vertices.push(x);
            let vj = q.insert(vx, |a, b| point_comp(vertices[a], vertices[b], eps));
            if vj.is_some() {
                vertices.pop();
            } else {
                vertex_lines.push(Vec::new());
                vertex_point.push(None);
            }
        }
    }

    if t.len() != 0 {
        return Ok((Vec::new(), Vec::new(), Vec::new()));
    }

    let mut compact_index = vec![None; vertex_point.len()];
    for (vi, point_index) in vertex_point.iter().enumerate() {
        compact_index[vi] = *point_index;
    }

    let mut combined: BTreeMap<Vec<u16>, (Edge, Vec<usize>)> = BTreeMap::new();
    for si in 1..segment_vertices.len() {
        let [Some(a), Some(b)] = segment_vertices[si] else {
            return Err(FlatFoldError::PrecisionFailure(
                "sweep segment was left open".to_string(),
            ));
        };
        let Some(mut pa) = compact_index.get(a).copied().flatten() else {
            continue;
        };
        let Some(mut pb) = compact_index.get(b).copied().flatten() else {
            continue;
        };
        if pb < pa {
            std::mem::swap(&mut pa, &mut pb);
        }
        let key = math::encode(&[pa, pb]);
        let entry = combined.entry(key).or_insert(([pa, pb], Vec::new()));
        entry.1.extend(segment_lines[si].iter().copied());
    }
    let mut edges = Vec::with_capacity(combined.len());
    let mut edge_lines = Vec::with_capacity(combined.len());
    for (_, (edge, mut lines)) in combined {
        lines.sort_unstable();
        edges.push(edge);
        edge_lines.push(lines);
    }
    Ok((points, edges, edge_lines))
}

fn insert_sweep_point(
    vertices: &mut Vec<Point>,
    vertex_lines: &mut Vec<Vec<usize>>,
    vertex_point: &mut Vec<Option<usize>>,
    queue: &mut AvlSet,
    point: Point,
    eps: f64,
) -> usize {
    let index = vertices.len();
    vertices.push(point);
    vertex_lines.push(Vec::new());
    vertex_point.push(None);
    if let Some(existing) = queue.insert(index, |a, b| point_comp(vertices[a], vertices[b], eps)) {
        vertices.pop();
        vertex_lines.pop();
        vertex_point.pop();
        existing
    } else {
        index
    }
}

fn point_comp(a: Point, b: Point, eps: f64) -> i32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    if dy.abs() > eps {
        if dy < 0.0 { -1 } else { 1 }
    } else if dx.abs() > eps {
        if dx < 0.0 { -1 } else { 1 }
    } else {
        0
    }
}

fn line_intersect(a: Point, b: Point, c: Point, d: Point, eps: f64) -> Option<Point> {
    let dx12 = a[0] - b[0];
    let dx34 = c[0] - d[0];
    let dy12 = a[1] - b[1];
    let dy34 = c[1] - d[1];
    let denom = dx12 * dy34 - dx34 * dy12;
    if denom.abs() < eps * eps {
        return None;
    }
    let ab = a[0] * b[1] - a[1] * b[0];
    let cd = c[0] * d[1] - c[1] * d[0];
    Some([
        (ab * dx34 - cd * dx12) / denom,
        (ab * dy34 - cd * dy12) / denom,
    ])
}

fn point_seg_dist(
    vi: usize,
    si: usize,
    vertices: &[Point],
    segment_units: &[Point],
    segment_distances: &[Option<f64>],
) -> f64 {
    segment_distances[si].unwrap_or(0.0) - math::dot(math::perp(segment_units[si]), vertices[vi])
}

fn on_line(
    vi: usize,
    si: usize,
    vertices: &[Point],
    segment_units: &[Point],
    segment_distances: &[Option<f64>],
    eps: f64,
) -> bool {
    point_seg_dist(vi, si, vertices, segment_units, segment_distances).abs() < eps
}

#[allow(clippy::too_many_arguments)]
fn segment_comp(
    si: usize,
    sj: usize,
    curr: usize,
    vertices: &[Point],
    segment_vertices: &[[Option<usize>; 2]],
    segment_units: &[Point],
    segment_angles: &[f64],
    segment_distances: &[Option<f64>],
    eps: f64,
) -> i32 {
    let dj = point_seg_dist(curr, sj, vertices, segment_units, segment_distances);
    if dj.abs() < eps {
        let pi = segment_vertices[si][1];
        if pi.is_some_and(|pi| on_line(pi, sj, vertices, segment_units, segment_distances, eps)) {
            return 0;
        }
        if segment_angles[sj] - segment_angles[si] > 0.0 {
            1
        } else {
            -1
        }
    } else if -dj > 0.0 {
        1
    } else {
        -1
    }
}

fn vertices_edges_to_vertices_faces(vertices: &[Point], edges: &[Edge]) -> Result<FaceAdjacency> {
    let mut adj = vec![Vec::new(); vertices.len()];
    for [a, b] in edges {
        adj[*a].push(*b);
        adj[*b].push(*a);
    }
    let mut vertex_vertices = Vec::with_capacity(vertices.len());
    for (i, vertex) in vertices.iter().copied().enumerate() {
        let mut adjacent = adj[i]
            .iter()
            .copied()
            .map(|vi| (vi, math::angle(math::sub(vertices[vi], vertex))))
            .collect::<Vec<_>>();
        adjacent.sort_by(|a, b| a.1.total_cmp(&b.1));
        vertex_vertices.push(
            adjacent
                .into_iter()
                .map(|(vi, _)| vi)
                .collect::<Vec<usize>>(),
        );
    }

    let mut faces = Vec::new();
    let mut seen: BTreeSet<Vec<u16>> = BTreeSet::new();
    for (v1, adjacent) in vertex_vertices.iter().enumerate() {
        for v2 in adjacent {
            let key = math::encode(&[v1, *v2]);
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);
            let mut face = vec![v1];
            let mut i = v1;
            let mut j = *v2;
            let mut guard = 0usize;
            while j != v1 {
                guard += 1;
                if guard > edges.len() + vertices.len() + 1 {
                    return Err(FlatFoldError::InvalidInput(
                        "face walk did not close".to_string(),
                    ));
                }
                face.push(j);
                let Some(next) = previous_in_list(&vertex_vertices[j], i) else {
                    return Err(FlatFoldError::InvalidInput(
                        "face walk reached a missing adjacency".to_string(),
                    ));
                };
                i = j;
                j = next;
                seen.insert(math::encode(&[i, j]));
            }
            if face.len() > 2 {
                faces.push(face);
            }
        }
    }
    sort_faces(&mut faces, vertices);
    faces.pop();
    Ok((vertex_vertices, faces))
}

fn vertex_vertices_from_faces(vertices: &[Point], faces: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut next = vec![OrderedMap::default(); vertices.len()];
    let mut prev = vec![OrderedMap::default(); vertices.len()];
    for face in faces {
        if face.is_empty() {
            continue;
        }
        let mut v1 = face[face.len() - 2];
        let mut v2 = face[face.len() - 1];
        for v3 in face {
            next[v2].set(v1, *v3);
            prev[v2].set(*v3, v1);
            v1 = v2;
            v2 = *v3;
        }
    }
    let mut out = vec![Vec::new(); vertices.len()];
    for i in 0..vertices.len() {
        let Some(v0) = next[i].first_key() else {
            continue;
        };
        let mut v = v0;
        if let Some(v1) = prev[i].get(v0) {
            v = v1;
            let mut v_next = prev[i].get(v);
            while v_next.is_some() && v_next != Some(v0) {
                let Some(next_value) = v_next else {
                    break;
                };
                v = next_value;
                v_next = prev[i].get(v);
            }
        }
        let start = v;
        out[i].push(start);
        let mut current = next[i].get(v);
        while let Some(value) = current {
            if value == start {
                break;
            }
            out[i].push(value);
            current = next[i].get(value);
        }
    }
    out
}

#[derive(Debug, Clone, Default)]
struct OrderedMap {
    entries: Vec<(usize, usize)>,
}

impl OrderedMap {
    fn set(&mut self, key: usize, value: usize) {
        if let Some((_, existing)) = self.entries.iter_mut().find(|(k, _)| *k == key) {
            *existing = value;
        } else {
            self.entries.push((key, value));
        }
    }

    fn get(&self, key: usize) -> Option<usize> {
        self.entries
            .iter()
            .find_map(|(k, value)| (*k == key).then_some(*value))
    }

    fn first_key(&self) -> Option<usize> {
        self.entries.first().map(|(key, _)| *key)
    }
}

fn previous_in_list(values: &[usize], value: usize) -> Option<usize> {
    values.iter().position(|v| *v == value).map(|index| {
        if index == 0 {
            values[values.len() - 1]
        } else {
            values[index - 1]
        }
    })
}

fn edges_faces_from_faces(edges: &[Edge], faces: &[Vec<usize>]) -> Result<FaceAdjacency> {
    let mut edge_map = BTreeMap::new();
    for (edge_index, [a, b]) in edges.iter().copied().enumerate() {
        edge_map.insert(math::encode(&[a, b]), edge_index);
    }
    let mut edges_faces = vec![[None, None]; edges.len()];
    for (face_index, face) in faces.iter().enumerate() {
        for (offset, v1) in face.iter().copied().enumerate() {
            let v2 = face[(offset + 1) % face.len()];
            let key = math::encode_order_pair(v1, v2);
            let Some(edge_index) = edge_map.get(&key).copied() else {
                return Err(FlatFoldError::InvalidInput(format!(
                    "face {face_index} references missing edge [{v1}, {v2}]"
                )));
            };
            let slot = if v2 < v1 { 0 } else { 1 };
            edges_faces[edge_index][slot] = Some(face_index);
        }
    }
    let edges_faces = edges_faces
        .into_iter()
        .map(|faces| match faces {
            [Some(a), Some(b)] => vec![a, b],
            [Some(a), None] => vec![a],
            [None, Some(b)] => vec![b],
            [None, None] => Vec::new(),
        })
        .collect::<Vec<_>>();
    let mut faces_edges = Vec::with_capacity(faces.len());
    for face in faces {
        let mut face_edges = Vec::with_capacity(face.len());
        for (offset, v1) in face.iter().copied().enumerate() {
            let v2 = face[(offset + 1) % face.len()];
            let key = math::encode_order_pair(v1, v2);
            let Some(edge_index) = edge_map.get(&key).copied() else {
                return Err(FlatFoldError::InvalidInput(format!(
                    "face references missing edge [{v1}, {v2}]"
                )));
            };
            face_edges.push(edge_index);
        }
        faces_edges.push(face_edges);
    }
    Ok((edges_faces, faces_edges))
}

fn sort_faces(faces: &mut [Vec<usize>], vertices: &[Point]) {
    faces.sort_by(|a, b| {
        let area_a = polygon_area_for_face(vertices, a);
        let area_b = polygon_area_for_face(vertices, b);
        area_b.total_cmp(&area_a)
    });
}

fn polygon_area_for_face(vertices: &[Point], face: &[usize]) -> f64 {
    let points = face.iter().map(|vi| vertices[*vi]).collect::<Vec<_>>();
    math::polygon_area2(&points)
}

fn flip_assignments(assignments: &mut [Assignment]) {
    for assignment in assignments {
        *assignment = match *assignment {
            Assignment::Mountain => Assignment::Valley,
            Assignment::Valley => Assignment::Mountain,
            other => other,
        };
    }
}

fn flip_y(vertices: &mut [Point]) {
    for vertex in vertices {
        vertex[1] = -vertex[1] + 1.0;
    }
}

fn reverse_faces(faces: &mut [Vec<usize>]) {
    for face in faces {
        face.reverse();
    }
}
