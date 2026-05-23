//! Non-mutating measurement commands ported from Oriedita handlers.

use crate::geometry::{Point, angle};

/// Oriedita display-length measurement after both points have been selected.
pub fn length_between_points(a: Point, b: Point) -> f64 {
    a.distance(b)
}

/// Oriedita display-angle measurement after three points have been selected.
pub fn angle_between_three_points(a: Point, center: Point, b: Point) -> f64 {
    angle((center, a, center, b))
}
