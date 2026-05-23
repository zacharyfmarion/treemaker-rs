use oristudio_cp::geometry::{Circle, LineColor, LineSegment, Point};
use oristudio_cp::model::CreasePatternModel;
use oristudio_cp::operations::transform::{
    LengthenColorMode, OperationFrame, OperationFrameMode, copy_selected_lines,
    copy_selected_lines_by_points, extend_to_intersection_point_2, lengthen_crease,
    move_selected_lines, move_selected_lines_by_points, operation_frame_drag,
    operation_frame_press, operation_frame_release, operation_frame_reset, translate_model,
};

#[test]
fn translate_model_moves_lines_and_circles_like_foldlineset() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(1.0, 0.0), LineColor::Red1);
    model.add_circle(Circle::new(2.0, 3.0, 4.0, LineColor::Blue2));

    translate_model(&mut model, 5.0, -2.0);

    assert_segment(
        &model.line_segments[0],
        Point::new(5.0, -2.0),
        Point::new(6.0, -2.0),
        LineColor::Red1,
    );
    assert_eq!(model.circles[0].determine_center(), Point::new(7.0, 1.0));
}

#[test]
fn move_selected_lines_deletes_originals_appends_moved_lines_and_unselects() {
    let mut model = model_from_segments(&[
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1).with_selected(2),
        segment(0.0, 2.0, 1.0, 2.0, LineColor::Blue2),
    ]);

    let moved = move_selected_lines(&mut model, Point::new(0.0, 1.0));

    assert_eq!(moved, 1);
    assert_eq!(model.line_segments.len(), 2);
    assert_segment(
        &model.line_segments[0],
        Point::new(0.0, 2.0),
        Point::new(1.0, 2.0),
        LineColor::Blue2,
    );
    assert_segment(
        &model.line_segments[1],
        Point::new(0.0, 1.0),
        Point::new(1.0, 1.0),
        LineColor::Red1,
    );
    assert!(
        model
            .line_segments
            .iter()
            .all(|segment| segment.selected == 0)
    );
}

#[test]
fn copy_selected_lines_keeps_originals_and_appends_unselected_copies() {
    let mut model = model_from_segments(&[
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1).with_selected(2),
        segment(0.0, 2.0, 1.0, 2.0, LineColor::Blue2),
    ]);

    let copied = copy_selected_lines(&mut model, Point::new(0.0, 1.0));

    assert_eq!(copied, 1);
    assert_eq!(model.line_segments.len(), 3);
    assert_segment(
        &model.line_segments[2],
        Point::new(0.0, 1.0),
        Point::new(1.0, 1.0),
        LineColor::Red1,
    );
    assert!(
        model
            .line_segments
            .iter()
            .all(|segment| segment.selected == 0)
    );
}

#[test]
fn four_point_selected_move_and_copy_apply_oriedita_scale_rotate_translate() {
    let original_a = Point::new(0.0, 0.0);
    let original_b = Point::new(1.0, 0.0);
    let target_a = Point::new(0.0, 0.0);
    let target_b = Point::new(0.0, 2.0);

    let mut move_model =
        model_from_segments(&[segment(1.0, 0.0, 1.0, 1.0, LineColor::Red1).with_selected(2)]);
    let moved =
        move_selected_lines_by_points(&mut move_model, original_a, original_b, target_a, target_b);
    assert_eq!(moved, 1);
    assert_segment_close(
        &move_model.line_segments[0],
        Point::new(0.0, 2.0),
        Point::new(-2.0, 2.0),
        LineColor::Red1,
    );

    let mut copy_model =
        model_from_segments(&[segment(1.0, 0.0, 1.0, 1.0, LineColor::Red1).with_selected(2)]);
    let copied =
        copy_selected_lines_by_points(&mut copy_model, original_a, original_b, target_a, target_b);
    assert_eq!(copied, 1);
    assert_eq!(copy_model.line_segments.len(), 2);
    assert_segment_close(
        &copy_model.line_segments[1],
        Point::new(0.0, 2.0),
        Point::new(-2.0, 2.0),
        LineColor::Red1,
    );
}

#[test]
fn extend_to_intersection_point_extends_from_b_to_nearest_forward_hit() {
    let model = model_from_segments(&[
        segment(5.0, -1.0, 5.0, 1.0, LineColor::Blue2),
        segment(10.0, -1.0, 10.0, 1.0, LineColor::Black0),
    ]);
    let source = segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1);

    let result = extend_to_intersection_point_2(&model, &source);

    assert_segment(
        &result,
        Point::new(1.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Red1,
    );
}

#[test]
fn extend_to_intersection_point_uses_collinear_endpoint_hits() {
    let model = model_from_segments(&[segment(5.0, 0.0, 7.0, 0.0, LineColor::Blue2)]);
    let source = segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1);

    let result = extend_to_intersection_point_2(&model, &source);

    assert_segment(
        &result,
        Point::new(1.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Red1,
    );
}

#[test]
fn lengthen_crease_extends_selected_candidate_to_target_line() {
    let mut model = model_from_segments(&[
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1),
        segment(2.0, -1.0, 2.0, 1.0, LineColor::Black0),
    ]);

    let added = lengthen_crease(
        &mut model,
        segment(0.5, -1.0, 0.5, 1.0, LineColor::Magenta5),
        Point::new(2.0, 0.25),
        1.0,
        LengthenColorMode::Current(LineColor::Blue2),
    );

    assert_eq!(added, 1);
    assert!(
        model
            .line_segments
            .iter()
            .any(|segment| segment.a == Point::new(2.0, 0.0)
                && segment.b == Point::new(1.0, 0.0)
                && segment.color == LineColor::Blue2)
    );
}

#[test]
fn lengthen_crease_same_color_uses_original_color_and_same_line_mode() {
    let mut model = model_from_segments(&[
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1),
        segment(2.0, -1.0, 2.0, 1.0, LineColor::Black0),
    ]);

    let added = lengthen_crease(
        &mut model,
        segment(0.5, -1.0, 0.5, 1.0, LineColor::Magenta5),
        Point::new(0.25, 0.0),
        1.0,
        LengthenColorMode::SameAsOriginal,
    );

    assert_eq!(added, 1);
    assert!(
        model
            .line_segments
            .iter()
            .any(|segment| segment.a == Point::new(1.0, 0.0)
                && segment.b == Point::new(2.0, 0.0)
                && segment.color == LineColor::Red1)
    );
}

#[test]
fn operation_frame_create_snaps_to_model_point_and_deactivates_tiny_frame() {
    let model = model_from_segments(&[segment(2.0, 2.0, 3.0, 2.0, LineColor::Black0)]);
    let mut frame = OperationFrame::default();

    let state = operation_frame_press(&model, &mut frame, Point::new(2.1, 2.1), 0.5);
    assert_eq!(state.mode, OperationFrameMode::Create1);
    assert_eq!(frame.points, [Point::new(2.0, 2.0); 4]);

    operation_frame_release(&model, &mut frame, &state, Point::new(2.2, 2.0), 0.5);
    assert!(!frame.active);
}

#[test]
fn operation_frame_create_drag_and_release_builds_rectangle() {
    let model = CreasePatternModel::default();
    let mut frame = OperationFrame::default();

    let mut state = operation_frame_press(&model, &mut frame, Point::new(1.0, 1.0), 0.5);
    operation_frame_drag(&model, &mut frame, &mut state, Point::new(4.0, 3.0), 0.5);
    operation_frame_release(&model, &mut frame, &state, Point::new(4.0, 3.0), 0.5);

    assert!(frame.active);
    assert_eq!(frame.points[0], Point::new(1.0, 1.0));
    assert_eq!(frame.points[1], Point::new(1.0, 3.0));
    assert_eq!(frame.points[2], Point::new(4.0, 3.0));
    assert_eq!(frame.points[3], Point::new(4.0, 1.0));
}

#[test]
fn operation_frame_moves_box_sides_and_corner_like_oriedita() {
    let model = CreasePatternModel::default();

    let mut box_frame = active_frame();
    let mut state = operation_frame_press(&model, &mut box_frame, Point::new(1.0, 1.0), 0.5);
    assert_eq!(state.mode, OperationFrameMode::MoveBox4);
    operation_frame_drag(
        &model,
        &mut box_frame,
        &mut state,
        Point::new(3.0, 4.0),
        0.5,
    );
    assert_eq!(
        box_frame.points,
        [
            Point::new(2.0, 3.0),
            Point::new(2.0, 5.0),
            Point::new(4.0, 5.0),
            Point::new(4.0, 3.0),
        ]
    );

    let mut side_frame = active_frame();
    let mut state = operation_frame_press(&model, &mut side_frame, Point::new(-0.1, 1.0), 0.5);
    assert_eq!(state.mode, OperationFrameMode::MoveSides3);
    operation_frame_drag(
        &model,
        &mut side_frame,
        &mut state,
        Point::new(0.5, 1.0),
        0.5,
    );
    assert_eq!(
        side_frame.points,
        [
            Point::new(0.5, 0.0),
            Point::new(0.5, 2.0),
            Point::new(2.0, 2.0),
            Point::new(2.0, 0.0),
        ]
    );

    let mut corner_frame = active_frame();
    let mut state = operation_frame_press(&model, &mut corner_frame, Point::new(2.0, 2.0), 0.5);
    assert_eq!(state.mode, OperationFrameMode::MovePoints2);
    operation_frame_drag(
        &model,
        &mut corner_frame,
        &mut state,
        Point::new(5.0, 4.0),
        0.5,
    );
    assert_eq!(state.mode, OperationFrameMode::Create1);
    assert_eq!(
        corner_frame.points,
        [
            Point::new(0.0, 0.0),
            Point::new(0.0, 4.0),
            Point::new(5.0, 4.0),
            Point::new(5.0, 0.0),
        ]
    );

    operation_frame_reset(&mut corner_frame);
    assert!(!corner_frame.active);
}

fn model_from_segments(segments: &[LineSegment]) -> CreasePatternModel {
    let mut model = CreasePatternModel::default();
    for segment in segments {
        model.add_line_segment(segment.clone());
    }
    model
}

fn active_frame() -> OperationFrame {
    OperationFrame {
        active: true,
        points: [
            Point::new(0.0, 0.0),
            Point::new(0.0, 2.0),
            Point::new(2.0, 2.0),
            Point::new(2.0, 0.0),
        ],
    }
}

fn segment(ax: f64, ay: f64, bx: f64, by: f64, color: LineColor) -> LineSegment {
    LineSegment::with_color(Point::new(ax, ay), Point::new(bx, by), color)
}

fn assert_segment(segment: &LineSegment, a: Point, b: Point, color: LineColor) {
    assert_eq!(segment.a, a);
    assert_eq!(segment.b, b);
    assert_eq!(segment.color, color);
}

fn assert_segment_close(segment: &LineSegment, a: Point, b: Point, color: LineColor) {
    assert!((segment.a.x - a.x).abs() < 1e-12);
    assert!((segment.a.y - a.y).abs() < 1e-12);
    assert!((segment.b.x - b.x).abs() < 1e-12);
    assert!((segment.b.y - b.y).abs() < 1e-12);
    assert_eq!(segment.color, color);
}
