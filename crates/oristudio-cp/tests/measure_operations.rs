use oristudio_cp::geometry::Point;
use oristudio_cp::operations::measure::{angle_between_three_points, length_between_points};

#[test]
fn length_between_points_uses_direct_point_distance() {
    assert_eq!(
        length_between_points(Point::new(0.0, 0.0), Point::new(3.0, 4.0)),
        5.0
    );
}

#[test]
fn angle_between_three_points_matches_oriedita_orientation() {
    assert_eq!(
        angle_between_three_points(
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
        ),
        90.0
    );
    assert_eq!(
        angle_between_three_points(
            Point::new(0.0, 1.0),
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
        ),
        270.0
    );
}
