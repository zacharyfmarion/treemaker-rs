//! Construction/drawing commands ported from Oriedita handlers.

use crate::geometry::{
    Epsilon, LineColor, LineSegment, ParallelJudgement, Point, StraightLine, distance,
    find_intersection_segments, find_line_symmetry_line_segment, find_projection,
    get_segment_with_length, is_line_segment_parallel_with_precision, is_point_within_line_span,
    move_parallel,
};
use crate::model::CreasePatternModel;
use crate::operations::arrangement::{
    add_line_segment_like_worker, divide_line_segment_with_new_lines,
};
use crate::operations::selection::unselect_all;
use crate::operations::transform::extend_to_intersection_point_2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawCreaseTarget {
    FoldLine,
    AuxLine,
}

/// Oriedita free/restricted draw-crease insertion after endpoints are resolved.
pub fn draw_crease_segment(
    model: &mut CreasePatternModel,
    segment: &LineSegment,
    target: DrawCreaseTarget,
) -> bool {
    if !Epsilon::HIGH.gt0(segment.determine_length()) {
        return false;
    }

    match target {
        DrawCreaseTarget::FoldLine => add_line_segment_like_worker(model, segment),
        DrawCreaseTarget::AuxLine => model.add_aux_line_segment(segment.clone()),
    }
    true
}

/// Oriedita `DRAW_CREASE_SYMMETRIC_12` mutation after the mirror axis is known.
pub fn mirror_selected_lines(model: &mut CreasePatternModel, axis: &LineSegment) -> usize {
    let selected: Vec<_> = model
        .line_segments
        .iter()
        .filter(|segment| segment.selected == 2)
        .cloned()
        .collect();
    if selected.is_empty() {
        return 0;
    }

    let original_end = model.line_segments.len();
    for segment in &selected {
        let mirrored =
            find_line_symmetry_line_segment(segment, axis).with_line_color(segment.color);
        model.add_line_segment(mirrored);
    }
    let added_end = model.line_segments.len();
    divide_line_segment_with_new_lines(model, original_end, added_end);
    unselect_all(model);
    selected.len()
}

/// Oriedita `PARALLEL_DRAW_40` final mutation after all three inputs are resolved.
pub fn parallel_draw(
    model: &mut CreasePatternModel,
    target_point: Point,
    parallel_segment: &LineSegment,
    destination_segment: &LineSegment,
    color: LineColor,
) -> bool {
    let guide = LineSegment::new(
        target_point,
        Point::new(
            target_point.x + parallel_segment.determine_bx() - parallel_segment.determine_ax(),
            target_point.y + parallel_segment.determine_by() - parallel_segment.determine_ay(),
        ),
    );
    let Some(result) = additional_intersection(&guide, destination_segment, color) else {
        return false;
    };
    add_line_segment_like_worker(model, &result);
    true
}

/// Oriedita `PARALLEL_DRAW_WIDTH_51` indicator generation after width is resolved.
pub fn parallel_width_indicators(selected_segment: &LineSegment, width: f64) -> [LineSegment; 2] {
    [
        move_parallel(selected_segment, width).with_line_color(LineColor::Purple8),
        move_parallel(selected_segment, -width).with_line_color(LineColor::Purple8),
    ]
}

/// Oriedita `PARALLEL_DRAW_WIDTH_51` final mutation after an indicator is chosen.
pub fn commit_parallel_width_indicator(
    model: &mut CreasePatternModel,
    indicator: &LineSegment,
    color: LineColor,
) -> bool {
    let segment = indicator.with_line_color(color);
    if !Epsilon::HIGH.gt0(segment.determine_length()) {
        return false;
    }
    add_line_segment_like_worker(model, &segment);
    true
}

/// Oriedita `PERPENDICULAR_DRAW_9` immediate projection branch.
pub fn perpendicular_projection(
    model: &mut CreasePatternModel,
    target_point: Point,
    perpendicular_segment: &LineSegment,
    color: LineColor,
) -> bool {
    let result = LineSegment::with_color(
        target_point,
        find_projection(
            StraightLine::from_segment(perpendicular_segment),
            target_point,
        ),
        color,
    );
    if !Epsilon::HIGH.gt0(result.determine_length()) {
        return false;
    }
    add_line_segment_like_worker(model, &result);
    true
}

/// Oriedita `PERPENDICULAR_DRAW_9` indicator branch after the base line is resolved.
pub fn perpendicular_indicator(
    model: &CreasePatternModel,
    target_point: Point,
    perpendicular_segment: &LineSegment,
) -> Option<LineSegment> {
    if !is_point_within_line_span(target_point, perpendicular_segment) {
        return None;
    }

    let moved = move_parallel(perpendicular_segment, 1.0);
    let seed = LineSegment::with_color(
        target_point,
        find_projection(StraightLine::from_segment(&moved), target_point),
        LineColor::Purple8,
    );
    let indicator = full_extend_until_hit(model, &seed);
    Some(full_extend_until_hit(
        model,
        &indicator.with_coordinates(indicator.b, indicator.a),
    ))
}

/// Add the chosen perpendicular indicator itself.
pub fn commit_perpendicular_indicator(
    model: &mut CreasePatternModel,
    indicator: &LineSegment,
    color: LineColor,
) -> bool {
    let result = indicator.with_line_color(color);
    if !Epsilon::HIGH.gt0(result.determine_length()) {
        return false;
    }
    add_line_segment_like_worker(model, &result);
    true
}

/// Oriedita `PERPENDICULAR_DRAW_9` destination branch after an indicator is resolved.
pub fn perpendicular_draw_to_destination(
    model: &mut CreasePatternModel,
    target_point: Point,
    indicator: &LineSegment,
    destination_segment: &LineSegment,
    color: LineColor,
) -> bool {
    let guide = LineSegment::new(
        target_point,
        Point::new(
            target_point.x + indicator.determine_bx() - indicator.determine_ax(),
            target_point.y + indicator.determine_by() - indicator.determine_ay(),
        ),
    );
    let Some(result) = additional_intersection(&guide, destination_segment, color) else {
        return false;
    };
    add_line_segment_like_worker(model, &result);
    true
}

fn additional_intersection(
    guide: &LineSegment,
    destination: &LineSegment,
    color: LineColor,
) -> Option<LineSegment> {
    let parallel = is_line_segment_parallel_with_precision(
        StraightLine::from_segment(guide),
        StraightLine::from_segment(destination),
        Epsilon::UNKNOWN_1EN7,
    );

    let cross_point = match parallel {
        ParallelJudgement::ParallelNotEqual => return None,
        ParallelJudgement::ParallelEqual => {
            if distance(guide.a, destination.a) > distance(guide.a, destination.b) {
                destination.b
            } else {
                destination.a
            }
        }
        ParallelJudgement::NotParallel => find_intersection_segments(guide, destination),
    };

    let result = LineSegment::with_color(cross_point, guide.a, color);
    if Epsilon::HIGH.gt0(result.determine_length()) {
        Some(result)
    } else {
        None
    }
}

fn full_extend_until_hit(model: &CreasePatternModel, segment: &LineSegment) -> LineSegment {
    let temp = get_segment_with_length(segment, 0.5);
    let point = temp.a;
    let temp = extend_to_intersection_point_2(model, &temp);
    LineSegment::with_color(
        point,
        temp.determine_furthest_endpoint(point),
        segment.color,
    )
}
