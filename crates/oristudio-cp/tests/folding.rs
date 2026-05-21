use oristudio_cp::folding::estimate_wireframe_from_segments;
use oristudio_cp::geometry::{LineColor, LineSegment, Point};

#[test]
fn wireframe_fold_builds_faces_and_face_positions() {
    let segments = square_with_diagonal();

    let folded = estimate_wireframe_from_segments(&segments, 1).expect("folded wireframe");

    assert_eq!(folded.points.len(), 4);
    assert_eq!(folded.lines.len(), 5);
    assert_eq!(folded.faces.len(), 2);
    assert_eq!(folded.starting_face, 0);
    assert_eq!(folded.face_positions[0], 1);
    assert!(folded.face_positions.contains(&2));
}

#[test]
fn wireframe_fold_returns_none_without_faces() {
    let segments = vec![LineSegment::with_color(
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        LineColor::Black0,
    )];

    assert!(estimate_wireframe_from_segments(&segments, 1).is_none());
}

fn square_with_diagonal() -> Vec<LineSegment> {
    vec![
        LineSegment::with_color(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            LineColor::Black0,
        ),
        LineSegment::with_color(
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            LineColor::Black0,
        ),
        LineSegment::with_color(
            Point::new(1.0, 1.0),
            Point::new(0.0, 1.0),
            LineColor::Black0,
        ),
        LineSegment::with_color(
            Point::new(0.0, 1.0),
            Point::new(0.0, 0.0),
            LineColor::Black0,
        ),
        LineSegment::with_color(Point::new(0.0, 0.0), Point::new(1.0, 1.0), LineColor::Red1),
    ]
}
