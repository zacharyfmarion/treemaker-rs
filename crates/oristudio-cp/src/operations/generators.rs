use crate::geometry::{LineColor, LineSegment, Point, line_segment_rotate};
use crate::model::CreasePatternModel;
use crate::operations::arrangement::add_line_segment_like_worker;

/// Oriedita `POLYGON_SET_NO_CORNERS_29` after both polygon points are resolved.
pub fn regular_polygon_no_corners(
    model: &mut CreasePatternModel,
    p1: Point,
    p2: Point,
    corners: usize,
    color: LineColor,
) -> usize {
    let mut added = 0;
    let mut seed = LineSegment::with_color(p1, p2, color);
    add_line_segment_like_worker(model, &seed);
    added += 1;

    if corners < 2 {
        return added;
    }

    let rotation = (corners as f64 - 2.0) * 180.0 / corners as f64;
    for _ in 2..=corners {
        let rotated = line_segment_rotate(&seed, rotation);
        seed = LineSegment::with_color(rotated.b, rotated.a, color);
        add_line_segment_like_worker(model, &seed);
        added += 1;
    }

    added
}

pub fn regular_polygon(
    model: &mut CreasePatternModel,
    p1: Point,
    p2: Point,
    corners: usize,
    color: LineColor,
) -> usize {
    regular_polygon_no_corners(model, p1, p2, corners, color)
}
