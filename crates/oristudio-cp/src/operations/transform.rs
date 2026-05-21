//! Transform operations ported from Oriedita selected-line move/copy handlers.

use crate::geometry::{
    Epsilon, Intersection, LineSegment, Point, StraightLine, StraightLineIntersection, angle,
    determine_line_segment_intersection_sweet_with_tolerances, find_intersection_straight_lines,
    point_rotate_scaled,
};
use crate::model::CreasePatternModel;
use crate::operations::arrangement::divide_line_segment_with_new_lines;
use crate::operations::selection::{delete_selected_lines, unselect_all};

/// Oriedita `FoldLineSet.move(dx, dy)` for the editable model's FoldLineSet data.
pub fn translate_model(model: &mut CreasePatternModel, dx: f64, dy: f64) {
    let delta = Point::new(dx, dy);
    for segment in &mut model.line_segments {
        *segment = translate_segment(segment, delta);
    }
    for circle in &mut model.circles {
        *circle = circle.with_center(circle.determine_center().move_by(delta));
    }
}

/// Oriedita `CREASE_MOVE_21` final mutation after selected lines and delta are known.
pub fn move_selected_lines(model: &mut CreasePatternModel, delta: Point) -> usize {
    if !Epsilon::HIGH.gt0(delta.distance(Point::origin())) {
        return 0;
    }

    let mut selected = selected_line_segments(model);
    let moved_count = selected.len();
    if moved_count == 0 {
        return 0;
    }

    delete_selected_lines(model);
    translate_segments(&mut selected, delta);
    append_and_split(model, selected);
    unselect_all(model);
    moved_count
}

/// Oriedita `CREASE_COPY_22` final mutation after selected lines and delta are known.
pub fn copy_selected_lines(model: &mut CreasePatternModel, delta: Point) -> usize {
    if !Epsilon::HIGH.gt0(delta.distance(Point::origin())) {
        return 0;
    }

    let mut selected = selected_line_segments(model);
    let copied_count = selected.len();
    if copied_count == 0 {
        return 0;
    }

    translate_segments(&mut selected, delta);
    for segment in &mut selected {
        *segment = segment.with_selected(0);
    }
    append_and_split(model, selected);
    unselect_all(model);
    copied_count
}

/// Oriedita `CREASE_MOVE_4P_31` final mutation once all four points are known.
pub fn move_selected_lines_by_points(
    model: &mut CreasePatternModel,
    original_a: Point,
    original_b: Point,
    target_a: Point,
    target_b: Point,
) -> usize {
    let mut selected = selected_line_segments(model);
    let moved_count = selected.len();
    if moved_count == 0 {
        return 0;
    }

    delete_selected_lines(model);
    transform_segments_by_points(&mut selected, original_a, original_b, target_a, target_b);
    append_and_split(model, selected);
    unselect_all(model);
    moved_count
}

/// Oriedita `CREASE_COPY_4P_32` final mutation once all four points are known.
pub fn copy_selected_lines_by_points(
    model: &mut CreasePatternModel,
    original_a: Point,
    original_b: Point,
    target_a: Point,
    target_b: Point,
) -> usize {
    let mut selected = selected_line_segments(model);
    let copied_count = selected.len();
    if copied_count == 0 {
        return 0;
    }

    transform_segments_by_points(&mut selected, original_a, original_b, target_a, target_b);
    for segment in &mut selected {
        *segment = segment.with_selected(0);
    }
    append_and_split(model, selected);
    unselect_all(model);
    copied_count
}

/// Oriedita `FoldLineSet.move(ta, tb, tc, td)` line-segment transform.
pub fn transform_segments_by_points(
    segments: &mut [LineSegment],
    original_a: Point,
    original_b: Point,
    target_a: Point,
    target_b: Point,
) {
    let rotation = angle((original_a, original_b, target_a, target_b));
    let scale = target_a.distance(target_b) / original_a.distance(original_b);
    let delta = original_a.delta(target_a);

    for segment in segments {
        let new_a = point_rotate_scaled(original_a, segment.a, rotation, scale).move_by(delta);
        let new_b = point_rotate_scaled(original_a, segment.b, rotation, scale).move_by(delta);
        *segment = segment.with_coordinates(new_a, new_b);
    }
}

/// Oriedita `OritaCalc.extendToIntersectionPoint_2`.
pub fn extend_to_intersection_point_2(
    model: &CreasePatternModel,
    segment: &LineSegment,
) -> LineSegment {
    let mut add_segment = segment.clone();
    let mut intersection_point = Point::new(1_000_000.0, 1_000_000.0);
    let mut intersection_distance = intersection_point.distance(add_segment.a);
    let straight_line = StraightLine::from_points(add_segment.a, add_segment.b);

    for existing in &model.line_segments {
        let straight_intersection = straight_line.line_segment_intersect_reverse_detail(existing);
        let segment_intersection = determine_line_segment_intersection_sweet_with_tolerances(
            segment,
            existing,
            Epsilon::UNKNOWN_1EN5,
            Epsilon::UNKNOWN_1EN5,
        );

        if straight_intersection.is_intersecting()
            && !segment_intersection.is_endpoint_intersection()
        {
            intersection_point = find_intersection_straight_lines(
                straight_line,
                StraightLine::from_segment(existing),
            );
            if should_extend_to(
                add_segment.a,
                add_segment.b,
                intersection_point,
                intersection_distance,
            ) {
                intersection_distance = intersection_point.distance(add_segment.a);
                add_segment = add_segment.with_b(intersection_point);
            }
        }

        if straight_intersection == StraightLineIntersection::Included3
            && segment_intersection != Intersection::ParallelEqual31
        {
            for point in [existing.a, existing.b] {
                if should_extend_to(add_segment.a, add_segment.b, point, intersection_distance) {
                    intersection_distance = point.distance(add_segment.a);
                    add_segment = add_segment.with_b(point);
                }
            }
        }
    }

    add_segment.with_a(segment.b)
}

fn selected_line_segments(model: &CreasePatternModel) -> Vec<LineSegment> {
    model
        .line_segments
        .iter()
        .filter(|segment| segment.selected == 2)
        .cloned()
        .collect()
}

fn append_and_split(model: &mut CreasePatternModel, segments: Vec<LineSegment>) {
    let original_end = model.line_segments.len();
    model.line_segments.extend(segments);
    let added_end = model.line_segments.len();
    divide_line_segment_with_new_lines(model, original_end, added_end);
}

fn translate_segments(segments: &mut [LineSegment], delta: Point) {
    for segment in segments {
        *segment = translate_segment(segment, delta);
    }
}

fn translate_segment(segment: &LineSegment, delta: Point) -> LineSegment {
    segment.with_coordinates(segment.a.move_by(delta), segment.b.move_by(delta))
}

fn should_extend_to(origin: Point, direction: Point, point: Point, current_distance: f64) -> bool {
    if point.distance(origin) <= Epsilon::UNKNOWN_1EN5 {
        return false;
    }
    if point.distance(origin) >= current_distance {
        return false;
    }

    let angle = angle((origin, direction, origin, point));
    !(1.0..=359.0).contains(&angle)
}
