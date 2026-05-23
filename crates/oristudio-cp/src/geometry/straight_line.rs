use super::epsilon::Epsilon;
use super::line_segment::LineSegment;
use super::point::Point;
use serde::{Deserialize, Serialize};

/// Infinite line in the form `a * x + b * y + c = 0`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StraightLine {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl StraightLine {
    pub fn new(a: f64, b: f64, c: f64) -> Self {
        let mut tmp_a = a;
        let mut tmp_b = b;
        let mut tmp_c = c;

        if tmp_a < 0.0 {
            tmp_a = -tmp_a;
            tmp_b = -tmp_b;
            tmp_c = -tmp_c;
        }
        if -Epsilon::PARALLEL_FOR_EDIT < tmp_a && tmp_a < Epsilon::PARALLEL_FOR_EDIT && tmp_b < 0.0
        {
            tmp_a = -tmp_a;
            tmp_b = -tmp_b;
            tmp_c = -tmp_c;
        }

        Self {
            a: tmp_a,
            b: tmp_b,
            c: tmp_c,
        }
    }

    pub fn from_points(p1: Point, p2: Point) -> Self {
        Self::from_coordinates(p1.x, p1.y, p2.x, p2.y)
    }

    pub fn from_segment(s: &LineSegment) -> Self {
        Self::from_coordinates(
            s.determine_ax(),
            s.determine_ay(),
            s.determine_bx(),
            s.determine_by(),
        )
    }

    pub fn from_coordinates(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self::new(y2 - y1, x1 - x2, y1 * x2 - x1 * y2)
    }

    pub fn translate(self, d: f64) -> Self {
        Self::new(
            self.a,
            self.b,
            self.c + d * (self.a * self.a + self.b * self.b).sqrt(),
        )
    }

    pub fn calculate_distance(self, p: Point) -> f64 {
        self.assignment_calculation(p).abs() / (self.a * self.a + self.b * self.b).sqrt()
    }

    pub fn calculate_distance_squared(self, p: Point) -> f64 {
        let assignment = self.assignment_calculation(p);
        assignment * assignment / (self.a * self.a + self.b * self.b)
    }

    pub fn orthogonalize(self, p: Point) -> Self {
        let c = -self.b * p.x + self.a * p.y;
        Self::new(self.b, -self.a, c)
    }

    pub fn same_side(self, p1: Point, p2: Point) -> i32 {
        let dd = self.assignment_calculation(p1) * self.assignment_calculation(p2);
        if dd > 0.0 {
            1
        } else if dd < 0.0 {
            -1
        } else {
            0
        }
    }

    pub fn assignment_calculation(self, p: Point) -> f64 {
        self.a * p.x + self.b * p.y + self.c
    }

    pub fn line_segment_intersect_reverse_detail(
        self,
        s0: &LineSegment,
    ) -> StraightLineIntersection {
        let d_a2 = self.calculate_distance_squared(s0.a);
        let d_b2 = self.calculate_distance_squared(s0.b);

        if Epsilon::HIGH.le0(d_a2) && Epsilon::HIGH.le0(d_b2) {
            return StraightLineIntersection::Included3;
        }

        if Epsilon::HIGH.le0(d_a2) && Epsilon::HIGH.gt0(d_b2) {
            return StraightLineIntersection::IntersectTA21;
        }

        if Epsilon::HIGH.gt0(d_a2) && Epsilon::HIGH.le0(d_b2) {
            return StraightLineIntersection::IntersectTB22;
        }

        let d_a = self.assignment_calculation(s0.a);
        let d_b = self.assignment_calculation(s0.b);

        if d_a * d_b > 0.0 {
            return StraightLineIntersection::None0;
        }

        if d_a * d_b < 0.0 {
            return StraightLineIntersection::IntersectX1;
        }

        StraightLineIntersection::Included3
    }

    pub fn find_intersection(self, t2: Self) -> Point {
        Point::new(
            (self.b * t2.c - t2.b * self.c) / (self.a * t2.b - t2.a * self.b),
            (t2.a * self.c - self.a * t2.c) / (self.a * t2.b - t2.a * self.b),
        )
    }

    pub fn find_projection(self, p: Point) -> Point {
        let t1 = self.orthogonalize(p);
        self.find_intersection(t1)
    }
}

impl Default for StraightLine {
    fn default() -> Self {
        Self::from_coordinates(0.0, 0.0, 1.0, 1.0)
    }
}

/// Oriedita straight-line versus segment intersection classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum StraightLineIntersection {
    None0 = 0,
    IntersectX1 = 1,
    IntersectTA21 = 21,
    IntersectTB22 = 22,
    Included3 = 3,
}

impl StraightLineIntersection {
    pub const fn code(self) -> i32 {
        self as i32
    }

    pub const fn is_intersecting(self) -> bool {
        matches!(
            self,
            Self::IntersectX1 | Self::IntersectTA21 | Self::IntersectTB22
        )
    }
}
