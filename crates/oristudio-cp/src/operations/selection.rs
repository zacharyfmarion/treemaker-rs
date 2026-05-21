//! Selection operations ported from Oriedita `FoldLineSet` and selection handlers.

use crate::geometry::{
    LineSegment, Point, Polygon, PolygonIntersection, equal, is_line_segment_overlapping,
    line_segment_x_kousa_decide,
};
use crate::model::CreasePatternModel;

const SELECTED: i32 = 2;
const UNSELECTED: i32 = 0;

/// Oriedita `FoldLineSet.select_all`.
pub fn select_all(model: &mut CreasePatternModel) -> usize {
    set_all(model, SELECTED)
}

/// Oriedita `FoldLineSet.unselect_all`.
pub fn unselect_all(model: &mut CreasePatternModel) -> usize {
    set_all(model, UNSELECTED)
}

/// Set the selected flag for zero-based line indices.
pub fn select_indices(model: &mut CreasePatternModel, indices: &[usize]) -> usize {
    set_indices(model, indices, SELECTED)
}

/// Clear the selected flag for zero-based line indices.
pub fn unselect_indices(model: &mut CreasePatternModel, indices: &[usize]) -> usize {
    set_indices(model, indices, UNSELECTED)
}

/// Oriedita `FoldLineSet.select(Polygon)`/box-line selection behavior.
pub fn select_box(model: &mut CreasePatternModel, polygon: &Polygon) -> usize {
    select_by_boundary_or_inside(model, polygon, SELECTED)
}

/// Box-line unselection using Oriedita's `lineSegmentsInside` predicate.
pub fn unselect_box(model: &mut CreasePatternModel, polygon: &Polygon) -> usize {
    select_by_boundary_or_inside(model, polygon, UNSELECTED)
}

/// Oriedita `FoldLineSet.select_Takakukei(polygon, "select")`.
pub fn select_polygon(model: &mut CreasePatternModel, polygon: &Polygon) -> usize {
    select_by_polygon_intersection(model, polygon, SELECTED)
}

/// Oriedita `FoldLineSet.select_Takakukei(polygon, "unselectAction")`.
pub fn unselect_polygon(model: &mut CreasePatternModel, polygon: &Polygon) -> usize {
    select_by_polygon_intersection(model, polygon, UNSELECTED)
}

/// Oriedita `FoldLineSet.select_lX(selection, "select_lX")`.
pub fn select_intersecting_line(model: &mut CreasePatternModel, selection: &LineSegment) -> usize {
    select_by_intersecting_line(model, selection, SELECTED)
}

/// Oriedita `FoldLineSet.select_lX(selection, "unselect_lX")`.
pub fn unselect_intersecting_line(
    model: &mut CreasePatternModel,
    selection: &LineSegment,
) -> usize {
    select_by_intersecting_line(model, selection, UNSELECTED)
}

/// Oriedita `FoldLineSet.selectProbablyConnected` without quadtree acceleration.
pub fn select_connected_from_point(model: &mut CreasePatternModel, point: Point) -> usize {
    let mut active_points = vec![point];
    let mut new_active_points = Vec::new();
    let mut processed_points = Vec::new();
    let mut connected_indices = Vec::new();

    while !active_points.is_empty() {
        for active_point in active_points.drain(..) {
            processed_points.push(active_point);

            for (index, line) in model.line_segments.iter().enumerate() {
                if equal(line.a, active_point) {
                    push_unique_usize(&mut connected_indices, index);
                    if !processed_points.contains(&line.b) {
                        push_unique_point(&mut new_active_points, line.b);
                    }
                }

                if equal(line.b, active_point) {
                    push_unique_usize(&mut connected_indices, index);
                    if !processed_points.contains(&line.a) {
                        push_unique_point(&mut new_active_points, line.a);
                    }
                }
            }
        }

        active_points.append(&mut new_active_points);
    }

    set_indices(model, &connected_indices, SELECTED)
}

/// Oriedita `FoldLineSet.delSelectedLineSegmentFast`.
pub fn delete_selected_lines(model: &mut CreasePatternModel) -> usize {
    let original_total = model.line_segments.len();
    model
        .line_segments
        .retain(|segment| segment.selected != SELECTED);
    original_total - model.line_segments.len()
}

fn set_all(model: &mut CreasePatternModel, selected: i32) -> usize {
    let mut changed = 0;
    for segment in &mut model.line_segments {
        if segment.selected != selected {
            changed += 1;
        }
        *segment = segment.with_selected(selected);
    }
    changed
}

fn set_indices(model: &mut CreasePatternModel, indices: &[usize], selected: i32) -> usize {
    let mut changed = 0;
    for index in indices {
        let Some(segment) = model.line_segments.get_mut(*index) else {
            continue;
        };

        if segment.selected != selected {
            changed += 1;
        }
        *segment = segment.with_selected(selected);
    }
    changed
}

fn select_by_boundary_or_inside(
    model: &mut CreasePatternModel,
    polygon: &Polygon,
    selected: i32,
) -> usize {
    let indices: Vec<_> = model
        .line_segments
        .iter()
        .enumerate()
        .filter(|(_, segment)| polygon.totu_boundary_inside_line_segment(segment))
        .map(|(index, _)| index)
        .collect();
    set_indices(model, &indices, selected)
}

fn select_by_polygon_intersection(
    model: &mut CreasePatternModel,
    polygon: &Polygon,
    selected: i32,
) -> usize {
    let indices: Vec<_> = model
        .line_segments
        .iter()
        .enumerate()
        .filter(|(_, segment)| {
            matches!(
                polygon.inside_outside_check(segment),
                PolygonIntersection::Border
                    | PolygonIntersection::BorderInside
                    | PolygonIntersection::Inside
            )
        })
        .map(|(index, _)| index)
        .collect();
    set_indices(model, &indices, selected)
}

fn select_by_intersecting_line(
    model: &mut CreasePatternModel,
    selection: &LineSegment,
    selected: i32,
) -> usize {
    let indices: Vec<_> = model
        .line_segments
        .iter()
        .enumerate()
        .filter(|(_, segment)| {
            is_line_segment_overlapping(segment, selection)
                || line_segment_x_kousa_decide(segment, selection)
        })
        .map(|(index, _)| index)
        .collect();
    set_indices(model, &indices, selected)
}

fn push_unique_usize(values: &mut Vec<usize>, value: usize) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn push_unique_point(values: &mut Vec<Point>, value: Point) {
    if !values.contains(&value) {
        values.push(value);
    }
}
