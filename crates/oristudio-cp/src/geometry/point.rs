use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Immutable point used by Oriedita geometry.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub const fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn mid(p: Self, q: Self) -> Self {
        Self::new((p.x + q.x) / 2.0, (p.y + q.y) / 2.0)
    }

    pub fn weighted(a: f64, p: Self, b: f64, q: Self) -> Self {
        Self::new(a * p.x + b * q.x, a * p.y + b * q.y)
    }

    pub fn with_x(self, x: f64) -> Self {
        Self::new(x, self.y)
    }

    pub fn with_y(self, y: f64) -> Self {
        Self::new(self.x, y)
    }

    pub fn distance(self, p: Self) -> f64 {
        self.distance_squared(p).sqrt()
    }

    pub fn distance_squared(self, p: Self) -> f64 {
        let x1 = p.x - self.x;
        let y1 = p.y - self.y;
        x1 * x1 + y1 * y1
    }

    pub fn delta(self, point: Self) -> Self {
        Self::new(point.x - self.x, point.y - self.y)
    }

    pub fn move_by(self, add_point: Self) -> Self {
        Self::new(self.x + add_point.x, self.y + add_point.y)
    }
}

impl Default for Point {
    fn default() -> Self {
        Self::origin()
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x.total_cmp(&other.x) == Ordering::Equal
            && self.y.total_cmp(&other.y) == Ordering::Equal
    }
}

impl Eq for Point {}

#[cfg(test)]
mod tests {
    use super::Point;

    #[test]
    fn exact_equality_preserves_signed_zero_like_java_double_compare() {
        assert_ne!(Point::new(0.0, 0.0), Point::new(-0.0, 0.0));
    }

    #[test]
    fn weighted_constructor_matches_oriedita() {
        let p = Point::weighted(0.25, Point::new(2.0, 4.0), 0.75, Point::new(6.0, 8.0));
        assert_eq!(p, Point::new(5.0, 7.0));
    }
}
