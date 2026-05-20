use super::epsilon::Epsilon;
use super::line_color::LineColor;
use super::line_segment::{LineSegment, RgbColor};
use super::point::Point;
use super::straight_line::StraightLine;
use serde::{Deserialize, Serialize};

/// Oriedita circle carrier.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub color: LineColor,
    pub customized: i32,
    pub customized_color: RgbColor,
}

impl Circle {
    pub const fn new(x: f64, y: f64, r: f64, color: LineColor) -> Self {
        Self {
            x,
            y,
            r,
            color,
            customized: 0,
            customized_color: RgbColor::new(100, 200, 200),
        }
    }

    pub fn from_center(center: Point, r: f64, color: LineColor) -> Self {
        Self::new(center.x, center.y, r, color)
    }

    pub fn from_diameter(segment: &LineSegment, color: LineColor) -> Self {
        Self::new(
            (segment.determine_ax() + segment.determine_bx()) / 2.0,
            (segment.determine_ay() + segment.determine_by()) / 2.0,
            segment.determine_length() / 2.0,
            color,
        )
    }

    pub fn determine_center(self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn with_center(self, center: Point) -> Self {
        Self {
            x: center.x,
            y: center.y,
            ..self
        }
    }

    pub fn with_radius(self, r: f64) -> Self {
        Self { r, ..self }
    }

    pub fn with_color(self, color: LineColor) -> Self {
        Self { color, ..self }
    }

    pub fn with_customized_color(self, customized_color: RgbColor) -> Self {
        Self {
            customized: 1,
            customized_color,
            ..self
        }
    }

    pub fn turn_around_point(self, point: Point) -> Point {
        let x1 = point.x - self.x;
        let y1 = point.y - self.y;
        let d1 = (x1 * x1 + y1 * y1).sqrt();

        if (d1 - self.r).abs() < Epsilon::UNKNOWN_1EN7 {
            return point;
        }

        let d2 = self.r * self.r / d1;
        let x2 = d2 * x1 / d1;
        let y2 = d2 * y1 / d1;
        Point::new(x2 + self.x, y2 + self.y)
    }

    pub fn turn_around_circle(self, circle: Self) -> Self {
        let x1 = circle.x - self.x;
        let y1 = circle.y - self.y;
        let d1 = (x1 * x1 + y1 * y1).sqrt();
        let da1 = d1 - circle.r;
        let db1 = d1 + circle.r;

        let (xa1, ya1, xb1, yb1) = if d1 < Epsilon::UNKNOWN_1EN6 {
            (da1, 0.0, db1, 0.0)
        } else {
            (da1 * x1 / d1, da1 * y1 / d1, db1 * x1 / d1, db1 * y1 / d1)
        };

        let a = self.turn_around_point(Point::new(xa1 + self.x, ya1 + self.y));
        let b = self.turn_around_point(Point::new(xb1 + self.x, yb1 + self.y));
        Self::from_diameter(&LineSegment::new(a, b), LineColor::Magenta5)
    }

    pub fn turn_around_circle_to_line_segment(self, circle: Self) -> LineSegment {
        let x1 = circle.x - self.x;
        let y1 = circle.y - self.y;
        let th = self.turn_around_point(Point::new(x1 * 2.0 + self.x, y1 * 2.0 + self.y));
        let tha = Point::new(th.x + 3.0 * y1, th.y - 3.0 * x1);
        let thb = Point::new(th.x - 3.0 * y1, th.y + 3.0 * x1);
        LineSegment::with_color(tha, thb, LineColor::Cyan3)
    }

    pub fn turn_around_line_segment_to_circle(self, segment: &LineSegment) -> Self {
        let line = StraightLine::from_segment(segment);
        let t0 = line.find_projection(self.determine_center());
        Self::from_diameter(
            &LineSegment::new(self.turn_around_point(t0), self.determine_center()),
            LineColor::Magenta5,
        )
    }
}

impl Default for Circle {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, LineColor::Black0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CircleIntersection {
    NoIntersection,
    Tangent,
    Intersect,
}
