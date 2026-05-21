use oristudio_cp::checks::flat_foldable_boundary_check;
use oristudio_cp::geometry::{LineColor, LineSegment, Point};
use oristudio_cp::model::CreasePatternModel;

#[test]
fn flat_foldable_boundary_check_colors_empty_boundary_crossings_cyan() {
    let model = CreasePatternModel::default();
    let mut boundary = vec![segment(-1.0, 0.0, 1.0, 0.0, LineColor::Yellow7)];

    let result = flat_foldable_boundary_check(&model, &mut boundary);

    assert_eq!(result.color, LineColor::Cyan3);
    assert!(result.suitable_intersections);
    assert_eq!(result.crossing_count, 0);
    assert_eq!(boundary[0].color, LineColor::Cyan3);
}

#[test]
fn flat_foldable_boundary_check_colors_odd_crossings_magenta() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, -1.0), Point::new(0.0, 1.0), LineColor::Red1);
    let mut boundary = vec![segment(-1.0, 0.0, 1.0, 0.0, LineColor::Yellow7)];

    let result = flat_foldable_boundary_check(&model, &mut boundary);

    assert_eq!(result.color, LineColor::Magenta5);
    assert!(result.suitable_intersections);
    assert_eq!(result.crossing_count, 1);
    assert_eq!(boundary[0].color, LineColor::Magenta5);
}

#[test]
fn flat_foldable_boundary_check_leaves_invalid_boundary_crossings_yellow() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(-1.0, 0.0), Point::new(1.0, 0.0), LineColor::Red1);
    let mut boundary = vec![segment(-1.0, 0.0, 1.0, 0.0, LineColor::Yellow7)];

    let result = flat_foldable_boundary_check(&model, &mut boundary);

    assert_eq!(result.color, LineColor::Yellow7);
    assert!(!result.suitable_intersections);
    assert_eq!(result.crossing_count, 0);
    assert_eq!(boundary[0].color, LineColor::Yellow7);
}

fn segment(ax: f64, ay: f64, bx: f64, by: f64, color: LineColor) -> LineSegment {
    LineSegment::with_color(Point::new(ax, ay), Point::new(bx, by), color)
}
