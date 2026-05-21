use oristudio_cp::geometry::{LineColor, LineSegment, Point};
use oristudio_cp::model::CreasePatternModel;
use oristudio_cp::operations::construction::{
    DrawCreaseTarget, draw_crease_segment, mirror_selected_lines,
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

fn segment(ax: f64, ay: f64, bx: f64, by: f64, color: LineColor) -> LineSegment {
    LineSegment::with_color(Point::new(ax, ay), Point::new(bx, by), color)
}

fn contains_segment(segments: &[LineSegment], a: Point, b: Point, color: LineColor) -> bool {
    segments
        .iter()
        .any(|segment| segment.a == a && segment.b == b && segment.color == color)
}
