use oristudio_cp::geometry::{LineColor, LineSegment, Point};
use oristudio_cp::model::CreasePatternModel;
use oristudio_cp::operations::arrangement::{
    divide_intersections, divide_intersections_fast, divide_line_segment_with_new_lines,
    intersect_divide_pair, remove_overlapping_lines, remove_overlapping_lines_with_precision,
};
use std::collections::BTreeSet;

#[test]
fn intersect_divide_pair_splits_crossing_segments() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);
    model.add_line(
        Point::new(5.0, -5.0),
        Point::new(5.0, 5.0),
        LineColor::Blue2,
    );

    let added = intersect_divide_pair(&mut model, 0, 1);

    assert_eq!(added, 2);
    assert_eq!(model.line_segments.len(), 4);
    assert_segment(
        &model.line_segments[0],
        Point::new(0.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Red1,
    );
    assert_segment(
        &model.line_segments[1],
        Point::new(5.0, -5.0),
        Point::new(5.0, 0.0),
        LineColor::Blue2,
    );
    assert_segment(
        &model.line_segments[2],
        Point::new(10.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Red1,
    );
    assert_segment(
        &model.line_segments[3],
        Point::new(5.0, 5.0),
        Point::new(5.0, 0.0),
        LineColor::Blue2,
    );
}

#[test]
fn intersect_divide_pair_splits_t_shape_owner_segment() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);
    model.add_line(Point::new(5.0, 0.0), Point::new(5.0, 5.0), LineColor::Blue2);

    let added = intersect_divide_pair(&mut model, 0, 1);

    assert_eq!(added, 1);
    assert_eq!(model.line_segments.len(), 3);
    assert_segment(
        &model.line_segments[0],
        Point::new(0.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Red1,
    );
    assert_segment(
        &model.line_segments[1],
        Point::new(5.0, 0.0),
        Point::new(5.0, 5.0),
        LineColor::Blue2,
    );
    assert_segment(
        &model.line_segments[2],
        Point::new(10.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Red1,
    );
}

#[test]
fn intersect_divide_pair_uses_later_color_for_overlap_piece() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);
    model.add_line(
        Point::new(5.0, 0.0),
        Point::new(15.0, 0.0),
        LineColor::Blue2,
    );

    let added = intersect_divide_pair(&mut model, 0, 1);

    assert_eq!(added, 1);
    assert_eq!(model.line_segments.len(), 3);
    assert_segment(
        &model.line_segments[0],
        Point::new(0.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Red1,
    );
    assert_segment(
        &model.line_segments[1],
        Point::new(10.0, 0.0),
        Point::new(15.0, 0.0),
        LineColor::Blue2,
    );
    assert_segment(
        &model.line_segments[2],
        Point::new(10.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Blue2,
    );
}

#[test]
fn divide_intersections_arranges_crossing_segments() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);
    model.add_line(
        Point::new(5.0, -5.0),
        Point::new(5.0, 5.0),
        LineColor::Blue2,
    );

    divide_intersections(&mut model);

    assert_eq!(model.line_segments.len(), 4);
    assert!(contains_segment(
        &model,
        Point::new(0.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Red1,
    ));
    assert!(contains_segment(
        &model,
        Point::new(10.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Red1,
    ));
    assert!(contains_segment(
        &model,
        Point::new(5.0, -5.0),
        Point::new(5.0, 0.0),
        LineColor::Blue2,
    ));
    assert!(contains_segment(
        &model,
        Point::new(5.0, 5.0),
        Point::new(5.0, 0.0),
        LineColor::Blue2,
    ));
}

#[test]
fn divide_intersections_fast_splits_new_and_existing_crossing_lines() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);
    model.add_line(
        Point::new(5.0, -5.0),
        Point::new(5.0, 5.0),
        LineColor::Blue2,
    );
    let mut to_delete = BTreeSet::new();

    let intersection = divide_intersections_fast(&mut model, 1, 0, &mut to_delete);

    assert_eq!(
        intersection,
        oristudio_cp::geometry::Intersection::Intersects1
    );
    assert!(to_delete.is_empty());
    assert_eq!(model.line_segments.len(), 4);
    assert_segment(
        &model.line_segments[0],
        Point::new(0.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Red1,
    );
    assert_segment(
        &model.line_segments[1],
        Point::new(5.0, -5.0),
        Point::new(5.0, 0.0),
        LineColor::Blue2,
    );
    assert_segment(
        &model.line_segments[2],
        Point::new(5.0, 0.0),
        Point::new(5.0, 5.0),
        LineColor::Blue2,
    );
    assert_segment(
        &model.line_segments[3],
        Point::new(5.0, 0.0),
        Point::new(10.0, 0.0),
        LineColor::Red1,
    );
}

#[test]
fn divide_intersections_fast_preserves_cyan_auxiliary_split_rules() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);
    model.add_line(Point::new(5.0, 0.0), Point::new(5.0, 5.0), LineColor::Cyan3);
    let mut to_delete = BTreeSet::new();

    let intersection = divide_intersections_fast(&mut model, 1, 0, &mut to_delete);

    assert_eq!(
        intersection,
        oristudio_cp::geometry::Intersection::NoIntersection0
    );
    assert!(to_delete.is_empty());
    assert_eq!(model.line_segments.len(), 2);
    assert_segment(
        &model.line_segments[0],
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        LineColor::Red1,
    );
    assert_segment(
        &model.line_segments[1],
        Point::new(5.0, 0.0),
        Point::new(5.0, 5.0),
        LineColor::Cyan3,
    );
}

#[test]
fn divide_intersections_fast_splits_parallel_overlap_with_new_line_color() {
    let mut model = CreasePatternModel::default();
    model.add_line(
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        LineColor::Blue2,
    );
    model.add_line(Point::new(5.0, 0.0), Point::new(15.0, 0.0), LineColor::Red1);
    let mut to_delete = BTreeSet::new();

    let intersection = divide_intersections_fast(&mut model, 1, 0, &mut to_delete);

    assert_eq!(
        intersection,
        oristudio_cp::geometry::Intersection::ParallelS1StartOverlapsS2End373
    );
    assert!(to_delete.is_empty());
    assert_eq!(model.line_segments.len(), 3);
    assert_segment(
        &model.line_segments[0],
        Point::new(0.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Blue2,
    );
    assert_segment(
        &model.line_segments[1],
        Point::new(10.0, 0.0),
        Point::new(15.0, 0.0),
        LineColor::Red1,
    );
    assert_segment(
        &model.line_segments[2],
        Point::new(5.0, 0.0),
        Point::new(10.0, 0.0),
        LineColor::Red1,
    );
}

#[test]
fn divide_line_segment_with_new_lines_splits_inserted_line_against_existing_lines() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);
    model.add_line(
        Point::new(5.0, -5.0),
        Point::new(5.0, 5.0),
        LineColor::Blue2,
    );

    divide_line_segment_with_new_lines(&mut model, 1, 2);

    assert_eq!(model.line_segments.len(), 4);
    assert!(contains_segment(
        &model,
        Point::new(0.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Red1,
    ));
    assert!(contains_segment(
        &model,
        Point::new(5.0, 0.0),
        Point::new(10.0, 0.0),
        LineColor::Red1,
    ));
    assert!(contains_segment(
        &model,
        Point::new(5.0, -5.0),
        Point::new(5.0, 0.0),
        LineColor::Blue2,
    ));
    assert!(contains_segment(
        &model,
        Point::new(5.0, 0.0),
        Point::new(5.0, 5.0),
        LineColor::Blue2,
    ));
}

#[test]
fn divide_line_segment_with_new_lines_deletes_existing_exact_duplicate() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);
    model.add_line(
        Point::new(10.0, 0.0),
        Point::new(0.0, 0.0),
        LineColor::Blue2,
    );

    divide_line_segment_with_new_lines(&mut model, 1, 2);

    assert_eq!(model.line_segments.len(), 1);
    assert_segment(
        &model.line_segments[0],
        Point::new(10.0, 0.0),
        Point::new(0.0, 0.0),
        LineColor::Blue2,
    );
}

#[test]
fn overlapping_line_removal_keeps_first_matching_segment() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);
    model.add_line(
        Point::new(10.0, 0.0),
        Point::new(0.0, 0.0),
        LineColor::Blue2,
    );
    model.add_line(
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
        LineColor::Cyan3,
    );

    remove_overlapping_lines(&mut model);

    assert_eq!(model.line_segments.len(), 2);
    assert_eq!(model.line_segments[0].color, LineColor::Red1);
    assert_eq!(model.line_segments[0].a, Point::new(0.0, 0.0));
    assert_eq!(model.line_segments[0].b, Point::new(10.0, 0.0));
    assert_eq!(model.line_segments[1].color, LineColor::Cyan3);
}

#[test]
fn overlapping_line_removal_uses_requested_precision() {
    let mut model = CreasePatternModel::default();
    model.add_line_segment(LineSegment::from_coordinates(0.0, 0.0, 10.0, 0.0));
    model.add_line_segment(LineSegment::from_coordinates(0.0001, 0.0, 10.0001, 0.0));

    remove_overlapping_lines_with_precision(&mut model, 0.001);

    assert_eq!(model.line_segments.len(), 1);
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
