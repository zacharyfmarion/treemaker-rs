use oristudio_cp::geometry::{LineColor, LineSegment, Point};
use oristudio_cp::model::CreasePatternModel;
use oristudio_cp::operations::construction::{
    DrawCreaseTarget, axiom7_draw_to_destination, axiom7_indicator,
    commit_parallel_width_indicator, double_symmetric_draw, draw_crease_angle_restricted_5,
    draw_crease_segment, fishbone_draw, inward, mirror_selected_lines, parallel_draw,
    parallel_width_indicators, perpendicular_indicator, perpendicular_projection,
    square_bisector_from_lines_to_destination, square_bisector_from_points_to_destination,
    square_bisector_parallel_between_destinations, square_bisector_parallel_indicator,
    symmetric_draw,
};

#[test]
fn draw_crease_segment_inserts_and_splits_fold_lines() {
    let mut model = CreasePatternModel::default();
    model.add_line(
        Point::new(1.0, -1.0),
        Point::new(1.0, 1.0),
        LineColor::Black0,
    );
    let segment = segment(0.0, 0.0, 2.0, 0.0, LineColor::Red1);

    assert!(draw_crease_segment(
        &mut model,
        &segment,
        DrawCreaseTarget::FoldLine
    ));

    assert_eq!(model.line_segments.len(), 4);
    assert!(contains_segment(
        &model.line_segments,
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        LineColor::Red1,
    ));
    assert!(contains_segment(
        &model.line_segments,
        Point::new(1.0, 0.0),
        Point::new(2.0, 0.0),
        LineColor::Red1,
    ));
}

#[test]
fn draw_crease_segment_aux_lines_append_without_foldline_splitting() {
    let mut model = CreasePatternModel::default();
    let segment = segment(0.0, 0.0, 2.0, 0.0, LineColor::Yellow7);

    assert!(draw_crease_segment(
        &mut model,
        &segment,
        DrawCreaseTarget::AuxLine
    ));

    assert!(model.line_segments.is_empty());
    assert_eq!(model.aux_line_segments, vec![segment]);
}

#[test]
fn draw_crease_segment_ignores_degenerate_segments() {
    let mut model = CreasePatternModel::default();
    let segment = segment(0.0, 0.0, 0.0, 0.0, LineColor::Red1);

    assert!(!draw_crease_segment(
        &mut model,
        &segment,
        DrawCreaseTarget::FoldLine
    ));
    assert!(model.is_empty());
}

#[test]
fn mirror_selected_lines_reflects_across_axis_and_unselects() {
    let mut model = CreasePatternModel::default();
    model.add_line_segment(segment(1.0, 0.0, 1.0, 1.0, LineColor::Red1).with_selected(2));
    model.add_line_segment(segment(3.0, 0.0, 3.0, 1.0, LineColor::Blue2));
    let axis = segment(0.0, 0.0, 0.0, 1.0, LineColor::Black0);

    let mirrored = mirror_selected_lines(&mut model, &axis);

    assert_eq!(mirrored, 1);
    assert_eq!(model.line_segments.len(), 3);
    assert!(contains_segment(
        &model.line_segments,
        Point::new(-1.0, 0.0),
        Point::new(-1.0, 1.0),
        LineColor::Red1,
    ));
    assert!(
        model
            .line_segments
            .iter()
            .all(|segment| segment.selected == 0)
    );
}

#[test]
fn parallel_draw_adds_parallel_segment_to_destination() {
    let mut model = model_from_segments(&[segment(2.0, -1.0, 2.0, 1.0, LineColor::Black0)]);
    let parallel = segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1);
    let destination = segment(2.0, -1.0, 2.0, 1.0, LineColor::Black0);

    assert!(parallel_draw(
        &mut model,
        Point::new(0.0, 0.5),
        &parallel,
        &destination,
        LineColor::Blue2,
    ));
    assert!(contains_segment(
        &model.line_segments,
        Point::new(2.0, 0.5),
        Point::new(0.0, 0.5),
        LineColor::Blue2,
    ));
}

#[test]
fn parallel_width_indicators_offset_selected_segment() {
    let mut model = CreasePatternModel::default();
    let selected = segment(0.0, 0.0, 2.0, 0.0, LineColor::Red1);
    let indicators = parallel_width_indicators(&selected, 1.0);

    assert_eq!(indicators[0].color, LineColor::Purple8);
    assert!(commit_parallel_width_indicator(
        &mut model,
        &indicators[0],
        LineColor::Blue2,
    ));
    assert_eq!(model.line_segments.len(), 1);
    assert_eq!(model.line_segments[0].color, LineColor::Blue2);
}

#[test]
fn perpendicular_projection_adds_short_projection_when_target_outside_span() {
    let mut model = CreasePatternModel::default();
    let base = segment(0.0, 0.0, 1.0, 0.0, LineColor::Black0);

    assert!(perpendicular_projection(
        &mut model,
        Point::new(2.0, 1.0),
        &base,
        LineColor::Red1,
    ));
    assert!(contains_segment(
        &model.line_segments,
        Point::new(2.0, 1.0),
        Point::new(2.0, 0.0),
        LineColor::Red1,
    ));
}

#[test]
fn perpendicular_indicator_extends_across_existing_hits() {
    let model = model_from_segments(&[
        segment(-1.0, -2.0, 1.0, -2.0, LineColor::Black0),
        segment(-1.0, 2.0, 1.0, 2.0, LineColor::Black0),
    ]);
    let base = segment(-1.0, 0.0, 1.0, 0.0, LineColor::Red1);

    let indicator = perpendicular_indicator(&model, Point::new(0.0, 0.0), &base)
        .expect("point on span should produce indicator");

    assert_eq!(indicator.color, LineColor::Purple8);
    assert!((indicator.a.x - 0.0).abs() < 1e-12);
    assert!((indicator.a.y + 2.0).abs() < 1e-12);
    assert!((indicator.b.x - 0.0).abs() < 1e-12);
    assert!((indicator.b.y - 2.0).abs() < 1e-12);
}

#[test]
fn symmetric_draw_reflects_source_ray_across_mirror_line() {
    let mut model = model_from_segments(&[segment(0.0, 2.0, 2.0, 2.0, LineColor::Black0)]);
    let source = segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1);
    let mirror = segment(0.0, 0.0, 1.0, 1.0, LineColor::Blue2);

    assert!(symmetric_draw(
        &mut model,
        &source,
        &mirror,
        LineColor::Red1,
    ));
    assert!(
        model
            .line_segments
            .iter()
            .any(|segment| segment.color == LineColor::Red1
                && (segment.a.x - 0.0).abs() < 1e-12
                && (segment.a.y - 1.0).abs() < 1e-12
                && (segment.b.x - 0.0).abs() < 1e-12
                && (segment.b.y - 2.0).abs() < 1e-12)
    );
}

#[test]
fn double_symmetric_draw_reflects_far_endpoint_across_drag_axis() {
    let mut model = model_from_segments(&[
        segment(0.0, 1.0, 2.0, 1.0, LineColor::Red1),
        segment(-3.0, 0.0, -3.0, 2.0, LineColor::Black0),
    ]);
    let drag_axis = segment(0.0, 0.0, 0.0, 2.0, LineColor::Black0);

    assert_eq!(double_symmetric_draw(&mut model, &drag_axis), 1);
    assert!(contains_segment(
        &model.line_segments,
        Point::new(-2.0, 1.0),
        Point::new(-3.0, 1.0),
        LineColor::Red1,
    ));
}

#[test]
fn inward_connects_triangle_vertices_to_incenter() {
    let mut model = CreasePatternModel::default();

    assert_eq!(
        inward(
            &mut model,
            Point::new(0.0, 0.0),
            Point::new(4.0, 0.0),
            Point::new(0.0, 3.0),
            LineColor::Blue2,
        ),
        3
    );

    assert!(contains_segment(
        &model.line_segments,
        Point::new(0.0, 0.0),
        Point::new(1.0, 1.0),
        LineColor::Blue2,
    ));
    assert!(contains_segment(
        &model.line_segments,
        Point::new(4.0, 0.0),
        Point::new(1.0, 1.0),
        LineColor::Blue2,
    ));
    assert!(contains_segment(
        &model.line_segments,
        Point::new(0.0, 3.0),
        Point::new(1.0, 1.0),
        LineColor::Blue2,
    ));
}

#[test]
fn square_bisector_points_draws_to_destination() {
    let destination = segment(2.0, -1.0, 2.0, 3.0, LineColor::Black0);
    let mut model = model_from_segments(std::slice::from_ref(&destination));

    assert!(square_bisector_from_points_to_destination(
        &mut model,
        Point::new(0.0, 0.0),
        Point::new(4.0, 0.0),
        Point::new(0.0, 3.0),
        &destination,
        LineColor::Red1,
    ));

    assert!(contains_segment_close(
        &model.line_segments,
        Point::new(2.0, 2.0 / 3.0),
        Point::new(4.0, 0.0),
        LineColor::Red1,
    ));
}

#[test]
fn square_bisector_nonparallel_lines_draw_to_destination() {
    let first = segment(0.0, 0.0, 4.0, 0.0, LineColor::Black0);
    let second = segment(0.0, 0.0, 0.0, 4.0, LineColor::Black0);
    let destination = segment(2.0, -1.0, 2.0, 3.0, LineColor::Black0);
    let mut model = model_from_segments(&[first.clone(), second.clone(), destination.clone()]);

    assert!(square_bisector_from_lines_to_destination(
        &mut model,
        &first,
        &second,
        &destination,
        LineColor::Blue2,
    ));

    assert!(contains_segment_close(
        &model.line_segments,
        Point::new(2.0, 2.0),
        Point::new(0.0, 0.0),
        LineColor::Blue2,
    ));
}

#[test]
fn square_bisector_parallel_indicator_and_destination_commit() {
    let first = segment(-2.0, 0.0, 2.0, 0.0, LineColor::Black0);
    let second = segment(-2.0, 2.0, 2.0, 2.0, LineColor::Black0);
    let left = segment(-3.0, -1.0, -3.0, 3.0, LineColor::Black0);
    let right = segment(3.0, -1.0, 3.0, 3.0, LineColor::Black0);
    let mut model = model_from_segments(&[first.clone(), second.clone(), left, right]);

    let indicator = square_bisector_parallel_indicator(&model, &first, &second)
        .expect("parallel lines should produce indicator");
    assert_eq!(indicator.color, LineColor::Purple8);
    assert!(same_segment_close(
        &indicator,
        Point::new(-3.0, 1.0),
        Point::new(3.0, 1.0),
        LineColor::Purple8,
    ));

    let first_destination = segment(-1.0, -1.0, -1.0, 3.0, LineColor::Black0);
    let second_destination = segment(1.0, -1.0, 1.0, 3.0, LineColor::Black0);
    assert!(square_bisector_parallel_between_destinations(
        &mut model,
        &indicator,
        &first_destination,
        &second_destination,
        LineColor::Red1,
    ));
    assert!(contains_segment(
        &model.line_segments,
        Point::new(-1.0, 1.0),
        Point::new(1.0, 1.0),
        LineColor::Red1,
    ));
}

#[test]
fn fishbone_draw_adds_alternating_perpendicular_ribs() {
    let mut model = model_from_segments(&[
        segment(-1.0, -2.0, 3.0, -2.0, LineColor::Black0),
        segment(-1.0, 2.0, 3.0, 2.0, LineColor::Black0),
    ]);
    let drag = segment(0.0, 0.0, 2.0, 0.0, LineColor::Black0);

    assert_eq!(
        fishbone_draw(&mut model, &drag, 1.0, LineColor::Red1, 0.5),
        6
    );
    assert!(contains_segment_close(
        &model.line_segments,
        Point::new(2.0, -1.0),
        Point::new(2.0, -2.0),
        LineColor::Red1,
    ));
    assert!(contains_segment_close(
        &model.line_segments,
        Point::new(1.0, 1.0),
        Point::new(1.0, 2.0),
        LineColor::Blue2,
    ));
    assert!(contains_segment_close(
        &model.line_segments,
        Point::new(0.0, -1.0),
        Point::new(0.0, -2.0),
        LineColor::Red1,
    ));
}

#[test]
fn axiom7_indicator_extends_fold_line_and_clips_to_destination() {
    let target_segment = segment(4.0, -2.0, 4.0, 2.0, LineColor::Black0);
    let perpendicular_segment = segment(0.0, 0.0, 1.0, 0.0, LineColor::Black0);
    let top = segment(0.0, 3.0, 4.0, 3.0, LineColor::Black0);
    let bottom = segment(0.0, -3.0, 4.0, -3.0, LineColor::Black0);
    let mut model = model_from_segments(&[target_segment.clone(), top, bottom]);

    let indicator = axiom7_indicator(
        &model,
        Point::new(0.0, 0.0),
        &target_segment,
        &perpendicular_segment,
    )
    .expect("resolved Axiom 7 inputs should produce an indicator");
    assert_eq!(indicator.color, LineColor::Purple8);
    assert!(same_segment_close(
        &indicator,
        Point::new(2.0, -3.0),
        Point::new(2.0, 3.0),
        LineColor::Purple8,
    ));

    let destination = segment(0.0, 1.0, 4.0, 1.0, LineColor::Black0);
    assert!(axiom7_draw_to_destination(
        &mut model,
        &indicator,
        &destination,
        LineColor::Blue2,
    ));
    assert!(contains_segment_close(
        &model.line_segments,
        Point::new(2.0, 1.0),
        indicator.a,
        LineColor::Blue2,
    ));
}

#[test]
fn angle_restricted_5_snaps_to_angle_system_and_nearby_line() {
    let mut model = model_from_segments(&[segment(2.0, -1.0, 2.0, 1.0, LineColor::Black0)]);

    assert!(draw_crease_angle_restricted_5(
        &mut model,
        Point::new(0.0, 0.0),
        Point::new(2.0, 0.2),
        4,
        [40.0, 60.0, 80.0, 30.0, 50.0, 100.0],
        0.5,
        LineColor::Red1,
    ));
    assert!(contains_segment_close(
        &model.line_segments,
        Point::new(0.0, 0.0),
        Point::new(2.0, 0.0),
        LineColor::Red1,
    ));
}

fn segment(ax: f64, ay: f64, bx: f64, by: f64, color: LineColor) -> LineSegment {
    LineSegment::with_color(Point::new(ax, ay), Point::new(bx, by), color)
}

fn model_from_segments(segments: &[LineSegment]) -> CreasePatternModel {
    let mut model = CreasePatternModel::default();
    for segment in segments {
        model.add_line_segment(segment.clone());
    }
    model
}

fn contains_segment(segments: &[LineSegment], a: Point, b: Point, color: LineColor) -> bool {
    segments
        .iter()
        .any(|segment| segment.a == a && segment.b == b && segment.color == color)
}

fn contains_segment_close(segments: &[LineSegment], a: Point, b: Point, color: LineColor) -> bool {
    segments
        .iter()
        .any(|segment| same_segment_close(segment, a, b, color))
}

fn contains_point_close(actual: Point, expected: Point) -> bool {
    (actual.x - expected.x).abs() < 1e-12 && (actual.y - expected.y).abs() < 1e-12
}

fn same_segment_close(segment: &LineSegment, a: Point, b: Point, color: LineColor) -> bool {
    segment.color == color
        && ((contains_point_close(segment.a, a) && contains_point_close(segment.b, b))
            || (contains_point_close(segment.a, b) && contains_point_close(segment.b, a)))
}
