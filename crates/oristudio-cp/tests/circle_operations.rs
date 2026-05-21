use oristudio_cp::geometry::{Circle, LineColor, LineSegment, Point, RgbColor};
use oristudio_cp::model::CreasePatternModel;
use oristudio_cp::operations::circle::{
    CircleInversionOutput, change_custom_color_for_indices, concentric, concentric_select,
    concentric_select_candidates, concentric_two_circle_select, draw, free, invert_circle,
    invert_line_segment, organize, separate, through_three_points,
};

#[test]
fn draw_adds_cyan_circle_with_radius_between_points() {
    let mut model = CreasePatternModel::default();

    assert!(draw(&mut model, Point::new(1.0, 2.0), Point::new(4.0, 6.0)));

    assert_eq!(model.circles.len(), 1);
    assert_eq!(model.circles[0].determine_center(), Point::new(1.0, 2.0));
    assert_eq!(model.circles[0].r, 5.0);
    assert_eq!(model.circles[0].color, LineColor::Cyan3);
}

#[test]
fn restricted_draw_preserves_oriedita_zero_radius_circle() {
    let mut model = CreasePatternModel::default();

    assert!(draw(&mut model, Point::new(1.0, 2.0), Point::new(1.0, 2.0)));

    assert_eq!(model.circles.len(), 1);
    assert_eq!(model.circles[0].r, 0.0);
}

#[test]
fn free_draw_ignores_equal_points() {
    let mut model = CreasePatternModel::default();

    assert!(!free(
        &mut model,
        Point::new(1.0, 2.0),
        Point::new(1.0, 2.0)
    ));

    assert!(model.circles.is_empty());
}

#[test]
fn through_three_points_adds_circumcircle() {
    let mut model = CreasePatternModel::default();

    assert!(through_three_points(
        &mut model,
        Point::new(1.0, 0.0),
        Point::new(0.0, 1.0),
        Point::new(-1.0, 0.0)
    ));

    assert_eq!(model.circles.len(), 1);
    assert!(model.circles[0].x.abs() < 1e-12);
    assert!(model.circles[0].y.abs() < 1e-12);
    assert!((model.circles[0].r - 1.0).abs() < 1e-12);
    assert_eq!(model.circles[0].color, LineColor::Cyan3);
}

#[test]
fn through_three_points_ignores_collinear_points() {
    let mut model = CreasePatternModel::default();

    assert!(!through_three_points(
        &mut model,
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        Point::new(2.0, 0.0)
    ));

    assert!(model.circles.is_empty());
}

#[test]
fn separate_and_concentric_modes_add_cyan_circles() {
    let mut model = CreasePatternModel::default();

    assert!(separate(
        &mut model,
        Point::new(10.0, 10.0),
        Point::new(1.0, 1.0),
        Point::new(4.0, 5.0)
    ));
    assert!(concentric(
        &mut model,
        Circle::new(2.0, 3.0, 7.0, LineColor::Magenta5),
        Point::new(0.0, 0.0),
        Point::new(0.0, 2.0)
    ));

    assert_eq!(model.circles.len(), 2);
    assert_eq!(model.circles[0].determine_center(), Point::new(10.0, 10.0));
    assert_eq!(model.circles[0].r, 5.0);
    assert_eq!(model.circles[1].determine_center(), Point::new(2.0, 3.0));
    assert_eq!(model.circles[1].r, 9.0);
    assert!(
        model
            .circles
            .iter()
            .all(|circle| circle.color == LineColor::Cyan3)
    );
}

#[test]
fn concentric_select_variants_match_resolved_indicator_semantics() {
    let target = Circle::new(0.0, 0.0, 5.0, LineColor::Cyan3);
    let reference1 = Circle::new(10.0, 0.0, 2.0, LineColor::Cyan3);
    let reference2 = Circle::new(12.0, 0.0, 4.0, LineColor::Cyan3);

    let candidates = concentric_select_candidates(target, reference1, reference2);
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].r, 7.0);
    assert_eq!(candidates[1].r, 3.0);
    assert!(
        candidates
            .iter()
            .all(|circle| circle.color == LineColor::Magenta5)
    );

    let mut model = CreasePatternModel::default();
    assert!(concentric_select(
        &mut model, target, reference1, reference2, 1
    ));
    assert_eq!(model.circles.len(), 1);
    assert_eq!(model.circles[0].r, 3.0);
    assert_eq!(model.circles[0].color, LineColor::Cyan3);
    assert!(!concentric_select(
        &mut model, target, reference1, reference2, 2
    ));
}

#[test]
fn two_circle_concentric_select_adds_pair_unless_tangent() {
    let mut model = CreasePatternModel::default();
    let circle1 = Circle::new(0.0, 0.0, 1.0, LineColor::Cyan3);
    let circle2 = Circle::new(5.0, 0.0, 1.0, LineColor::Cyan3);

    assert_eq!(
        concentric_two_circle_select(&mut model, circle1, circle2),
        2
    );
    assert_eq!(model.circles.len(), 2);
    assert_eq!(model.circles[0].r, 2.5);
    assert_eq!(model.circles[1].r, 2.5);

    assert_eq!(
        concentric_two_circle_select(
            &mut model,
            circle1,
            Circle::new(2.0, 0.0, 1.0, LineColor::Cyan3)
        ),
        0
    );
    assert_eq!(model.circles.len(), 2);
}

#[test]
fn inversion_modes_append_oriedita_result_shapes() {
    let mut model = CreasePatternModel::default();
    let inversion = Circle::new(0.0, 0.0, 1.0, LineColor::Cyan3);

    assert_eq!(
        invert_circle(
            &mut model,
            Circle::new(2.0, 0.0, 0.5, LineColor::Magenta5),
            inversion
        ),
        CircleInversionOutput::Circle
    );
    assert_eq!(model.circles.len(), 1);
    assert_eq!(model.circles[0].color, LineColor::Cyan3);

    assert_eq!(
        invert_circle(
            &mut model,
            Circle::new(1.0, 0.0, 1.0, LineColor::Magenta5),
            inversion
        ),
        CircleInversionOutput::LineSegment
    );
    assert_eq!(model.line_segments.len(), 1);
    assert_eq!(model.line_segments[0].color, LineColor::Cyan3);

    let subject = LineSegment::with_color(
        Point::new(2.0, -1.0),
        Point::new(2.0, 1.0),
        LineColor::Black0,
    );
    assert_eq!(
        invert_line_segment(&mut model, &subject, inversion),
        CircleInversionOutput::Circle
    );
    assert_eq!(model.circles.len(), 2);

    let through_center = LineSegment::with_color(
        Point::new(-1.0, 0.0),
        Point::new(1.0, 0.0),
        LineColor::Black0,
    );
    assert_eq!(
        invert_line_segment(&mut model, &through_center, inversion),
        CircleInversionOutput::None
    );
    assert_eq!(model.circles.len(), 2);
}

#[test]
fn organize_prunes_zero_radius_circles_like_oriedita_worker() {
    let mut model = CreasePatternModel::default();
    model.add_circle(Circle::new(0.0, 0.0, 2.0, LineColor::Cyan3));
    model.add_circle(Circle::new(2.0, 0.0, 0.0, LineColor::Cyan3));
    model.add_circle(Circle::new(9.0, 9.0, 0.0, LineColor::Cyan3));
    model.add_line(
        Point::new(2.0, -1.0),
        Point::new(2.0, 1.0),
        LineColor::Black0,
    );

    assert_eq!(organize(&mut model), 1);
    assert_eq!(model.circles.len(), 2);
    assert_eq!(model.circles[0].r, 2.0);
    assert_eq!(model.circles[1].determine_center(), Point::new(2.0, 0.0));
}

#[test]
fn change_custom_color_updates_circles_and_cyan_lines_with_oriedita_value_lookup() {
    let mut model = CreasePatternModel::default();
    model.add_circle(Circle::new(0.0, 0.0, 1.0, LineColor::Cyan3));
    model.add_circle(Circle::new(3.0, 0.0, 1.0, LineColor::Cyan3));
    let duplicate =
        LineSegment::with_color(Point::new(0.0, 0.0), Point::new(1.0, 0.0), LineColor::Cyan3);
    model.add_line_segment(duplicate.clone());
    model.add_line_segment(duplicate);
    model.add_line_segment(LineSegment::with_color(
        Point::new(0.0, 1.0),
        Point::new(1.0, 1.0),
        LineColor::Red1,
    ));

    let changed =
        change_custom_color_for_indices(&mut model, &[1], &[1, 2], RgbColor::new(10, 20, 30));

    assert_eq!(changed, 2);
    assert_eq!(model.circles[0].customized, 0);
    assert_eq!(model.circles[1].customized, 1);
    assert_eq!(model.circles[1].customized_color, RgbColor::new(10, 20, 30));
    assert_eq!(model.line_segments[0].customized, 1);
    assert_eq!(model.line_segments[1].customized, 0);
    assert_eq!(model.line_segments[2].customized, 0);
}
