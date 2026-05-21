use oristudio_cp::geometry::{LineColor, Point};
use oristudio_cp::model::CreasePatternModel;
use oristudio_cp::operations::circle::{draw, free, through_three_points};

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
