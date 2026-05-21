use oristudio_cp::folding::{estimate_wireframe_from_segments, prepare_subface_segments};
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

#[test]
fn subface_preparation_removes_points_duplicates_and_splits_crossings() {
    let segments = vec![
        LineSegment::with_color(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1),
        LineSegment::with_color(
            Point::new(5.0, -5.0),
            Point::new(5.0, 5.0),
            LineColor::Blue2,
        ),
        LineSegment::with_color(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1),
        LineSegment::with_color(
            Point::new(2.0, 2.0),
            Point::new(2.0, 2.0),
            LineColor::Black0,
        ),
    ];

    let prepared = prepare_subface_segments(&segments);

    assert_eq!(prepared.len(), 4);
    assert!(prepared.iter().all(|segment| segment.a != segment.b));
    assert_eq!(
        prepared
            .iter()
            .filter(|segment| segment.color == LineColor::Red1)
            .count(),
        2
    );
    assert_eq!(
        prepared
            .iter()
            .filter(|segment| segment.color == LineColor::Blue2)
            .count(),
        2
    );
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
