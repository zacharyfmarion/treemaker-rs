use oristudio_cp::geometry::{LineColor, LineSegment, Point};
use oristudio_cp::model::CreasePatternModel;
use oristudio_cp::operations::point::{divide_segment_by_count, divide_segment_by_ratio};

#[test]
fn divide_segment_by_count_adds_equal_subsegments_and_splits_crossings() {
    let mut model = CreasePatternModel::default();
    model.add_line(
        Point::new(1.0, -1.0),
        Point::new(1.0, 1.0),
        LineColor::Black0,
    );
    let segment = segment(0.0, 0.0, 2.0, 0.0, LineColor::Red1);

    let added = divide_segment_by_count(&mut model, &segment, 2);

    assert_eq!(added, 2);
    assert_eq!(model.line_segments.len(), 4);
    assert!(contains_segment(
        &model,
        Point::new(1.0, -1.0),
        Point::new(1.0, 0.0),
        LineColor::Black0,
    ));
    assert!(contains_segment(
        &model,
        Point::new(1.0, 0.0),
        Point::new(1.0, 1.0),
        LineColor::Black0,
    ));
    assert!(contains_segment(
        &model,
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        LineColor::Red1,
    ));
    assert!(contains_segment(
        &model,
        Point::new(1.0, 0.0),
        Point::new(2.0, 0.0),
        LineColor::Red1,
    ));
}

#[test]
fn divide_segment_by_ratio_reverses_drag_segment_like_oriedita() {
    let mut model = CreasePatternModel::default();
    let segment = segment(0.0, 0.0, 10.0, 0.0, LineColor::Blue2);

    let added = divide_segment_by_ratio(&mut model, &segment, 1.0, 3.0);

    assert_eq!(added, 2);
    assert_eq!(model.line_segments.len(), 2);
    assert_segment(
        &model.line_segments[0],
        Point::new(10.0, 0.0),
        Point::new(2.5, 0.0),
        LineColor::Blue2,
    );
    assert_segment(
        &model.line_segments[1],
        Point::new(0.0, 0.0),
        Point::new(2.5, 0.0),
        LineColor::Blue2,
    );
}

#[test]
fn divide_segment_by_ratio_with_one_zero_adds_whole_reversed_segment() {
    let mut model = CreasePatternModel::default();
    let segment = segment(-1.0, 2.0, 3.0, 2.0, LineColor::Cyan3);

    let added = divide_segment_by_ratio(&mut model, &segment, 0.0, 4.0);

    assert_eq!(added, 1);
    assert_eq!(model.line_segments.len(), 1);
    assert_segment(
        &model.line_segments[0],
        Point::new(3.0, 2.0),
        Point::new(-1.0, 2.0),
        LineColor::Cyan3,
    );
}

#[test]
fn divide_segment_zero_inputs_are_noops() {
    let mut model = CreasePatternModel::default();
    let segment = segment(0.0, 0.0, 0.0, 0.0, LineColor::Red1);

    assert_eq!(divide_segment_by_count(&mut model, &segment, 3), 0);
    assert_eq!(divide_segment_by_ratio(&mut model, &segment, 1.0, 1.0), 0);
    assert!(model.line_segments.is_empty());
}

fn segment(ax: f64, ay: f64, bx: f64, by: f64, color: LineColor) -> LineSegment {
    LineSegment::with_color(Point::new(ax, ay), Point::new(bx, by), color)
}

fn assert_segment(segment: &LineSegment, a: Point, b: Point, color: LineColor) {
    assert_eq!(segment.a, a);
    assert_eq!(segment.b, b);
    assert_eq!(segment.color, color);
}

fn contains_segment(model: &CreasePatternModel, a: Point, b: Point, color: LineColor) -> bool {
    model
        .line_segments
        .iter()
        .any(|segment| segment.a == a && segment.b == b && segment.color == color)
}
