use super::circle::{Circle, CircleIntersection};
use super::epsilon::Epsilon;
use super::line_segment::{Intersection, LineSegment};
use super::point::Point;
use super::straight_line::StraightLine;
use serde::{Deserialize, Serialize};

pub fn find_projection(line: StraightLine, p: Point) -> Point {
    let t1 = line.orthogonalize(p);
    find_intersection_straight_lines(line, t1)
}

pub fn find_projection_points(p0: Point, p1: Point, p: Point) -> Point {
    find_projection(StraightLine::from_points(p0, p1), p)
}

pub fn find_projection_segment(segment: &LineSegment, p: Point) -> Point {
    find_projection_points(segment.a, segment.b, p)
}

pub fn equal(p1: Point, p2: Point) -> bool {
    equal_with_radius(p1, p2, Epsilon::POINT)
}

pub fn equal_with_radius(p1: Point, p2: Point, r: f64) -> bool {
    if r <= 0.0 && p1 == p2 {
        return true;
    }
    if r > 0.0 {
        return distance(p1, p2) <= r;
    }
    false
}

pub fn distance(p0: Point, p1: Point) -> f64 {
    p0.distance(p1)
}

pub trait AngleInput {
    fn angle(self) -> f64;
}

impl AngleInput for (Point, Point) {
    fn angle(self) -> f64 {
        let (a, b) = self;
        let x = b.x - a.x;
        let y = b.y - a.y;
        let length = (x * x + y * y).sqrt();
        if length <= 0.0 {
            return -10000.0;
        }

        let mut c = x / length;
        if c > 1.0 {
            c = 1.0;
        }

        let mut ret = c.acos();
        if y < 0.0 {
            ret = -ret;
        }
        ret = 180.0 * ret / std::f64::consts::PI;
        if ret < 0.0 {
            ret += 360.0;
        }
        ret
    }
}

impl AngleInput for &LineSegment {
    fn angle(self) -> f64 {
        angle((self.a, self.b))
    }
}

impl AngleInput for (LineSegment, LineSegment) {
    fn angle(self) -> f64 {
        let (s1, s2) = self;
        angle_between_0_360(angle((s2.a, s2.b)) - angle((s1.a, s1.b)))
    }
}

impl AngleInput for (&LineSegment, &LineSegment) {
    fn angle(self) -> f64 {
        let (s1, s2) = self;
        angle_between_0_360(angle((s2.a, s2.b)) - angle((s1.a, s1.b)))
    }
}

impl AngleInput for (Point, Point, Point, Point) {
    fn angle(self) -> f64 {
        let (a, b, c, d) = self;
        angle_between_0_360(angle((c, d)) - angle((a, b)))
    }
}

pub fn angle<T: AngleInput>(input: T) -> f64 {
    input.angle()
}

pub fn is_inside(p1: Point, pa: Point, p2: Point) -> i32 {
    let u1 = StraightLine::from_points(p1, p2).orthogonalize(p1);
    let u2 = StraightLine::from_points(p1, p2).orthogonalize(p2);
    let product = u1.assignment_calculation(pa) * u2.assignment_calculation(pa);
    if product == 0.0 {
        return 1;
    }
    if product < 0.0 {
        return 2;
    }
    0
}

pub fn is_inside_sweet(p1: Point, pa: Point, p2: Point) -> i32 {
    let u1 = StraightLine::from_points(p1, p2).orthogonalize(p1);
    let u2 = StraightLine::from_points(p1, p2).orthogonalize(p2);
    if u1.calculate_distance(pa) < Epsilon::SWEET_DISTANCE
        || u2.calculate_distance(pa) < Epsilon::SWEET_DISTANCE
    {
        return 1;
    }
    if u1.assignment_calculation(pa) * u2.assignment_calculation(pa) < 0.0 {
        return 2;
    }
    0
}

pub fn determine_closest_line_segment_endpoint(p: Point, segment: &LineSegment, r: f64) -> i32 {
    if r > distance(p, segment.a) {
        return 1;
    }
    if r > distance(p, segment.b) {
        return 2;
    }
    if r > determine_line_segment_distance(p, segment) {
        return 3;
    }
    0
}

pub fn determine_line_segment_distance(p0: Point, segment: &LineSegment) -> f64 {
    determine_line_segment_distance_points(p0, segment.a, segment.b)
}

pub fn determine_line_segment_distance_points(p0: Point, p1: Point, p2: Point) -> f64 {
    if distance(p1, p2) == 0.0 {
        return distance(p0, p1);
    }

    let t = StraightLine::from_points(p1, p2);
    let u = t.orthogonalize(p0);

    if is_inside(p1, find_intersection_straight_lines(t, u), p2) >= 1 {
        return t.calculate_distance(p0);
    }
    distance(p0, p1).min(distance(p0, p2))
}

pub fn determine_line_segment_intersection(s1: &LineSegment, s2: &LineSegment) -> Intersection {
    determine_line_segment_intersection_with_precision(s1, s2, Epsilon::UNKNOWN_001)
}

pub fn determine_line_segment_intersection_with_precision(
    s1: &LineSegment,
    s2: &LineSegment,
    precision: f64,
) -> Intersection {
    determine_line_segment_intersection_with_tolerances(s1, s2, precision, precision)
}

pub fn determine_line_segment_intersection_with_tolerances(
    s1: &LineSegment,
    s2: &LineSegment,
    rhit: f64,
    rhei: f64,
) -> Intersection {
    determine_line_segment_intersection_impl(s1, s2, rhit, rhei, false)
}

pub fn determine_line_segment_intersection_sweet(
    s1: &LineSegment,
    s2: &LineSegment,
) -> Intersection {
    determine_line_segment_intersection_sweet_with_tolerances(
        s1,
        s2,
        Epsilon::UNKNOWN_001,
        Epsilon::UNKNOWN_001,
    )
}

pub fn determine_line_segment_intersection_sweet_with_tolerances(
    s1: &LineSegment,
    s2: &LineSegment,
    rhit: f64,
    rhei: f64,
) -> Intersection {
    determine_line_segment_intersection_impl(s1, s2, rhit, rhei, true)
}

fn determine_line_segment_intersection_impl(
    s1: &LineSegment,
    s2: &LineSegment,
    rhit: f64,
    rhei: f64,
    sweet: bool,
) -> Intersection {
    let x1max = s1.determine_ax().max(s1.determine_bx());
    let x1min = s1.determine_ax().min(s1.determine_bx());
    let y1max = s1.determine_ay().max(s1.determine_by());
    let y1min = s1.determine_ay().min(s1.determine_by());
    let x2max = s2.determine_ax().max(s2.determine_bx());
    let x2min = s2.determine_ax().min(s2.determine_bx());
    let y2max = s2.determine_ay().max(s2.determine_by());
    let y2min = s2.determine_ay().min(s2.determine_by());

    if x1max + rhit + Epsilon::POINT < x2min {
        return Intersection::NoIntersection0;
    }
    if x1min - rhit - Epsilon::POINT > x2max {
        return Intersection::NoIntersection0;
    }
    if y1max + rhit + Epsilon::POINT < y2min {
        return Intersection::NoIntersection0;
    }
    if y1min - rhit - Epsilon::POINT > y2max {
        return Intersection::NoIntersection0;
    }

    let p1 = s1.a;
    let p2 = s1.b;
    let p3 = s2.a;
    let p4 = s2.b;

    let t1 = StraightLine::from_points(p1, p2);
    let t2 = StraightLine::from_points(p3, p4);

    if p1 == p2 && p3 == p4 {
        if p1 == p3 {
            return Intersection::IntersectAtPoint4;
        }
        return Intersection::NoIntersection0;
    }

    if p1 == p2 {
        if is_inside(p3, p1, p4) >= 1 && t2.assignment_calculation(p1) == 0.0 {
            return Intersection::IntersectAtPointS1_5;
        }
        return Intersection::NoIntersection0;
    }

    if p3 == p4 {
        if is_inside(p1, p3, p2) >= 1 && t1.assignment_calculation(p3) == 0.0 {
            return Intersection::IntersectAtPointS2_6;
        }
        return Intersection::NoIntersection0;
    }

    match is_line_segment_parallel_with_precision(t1, t2, rhei) {
        ParallelJudgement::NotParallel => {
            let pk = find_intersection_straight_lines(t1, t2);
            let s1_inside = if sweet {
                is_inside_sweet(p1, pk, p2)
            } else {
                is_inside(p1, pk, p2)
            };
            let s2_inside = if sweet {
                is_inside_sweet(p3, pk, p4)
            } else {
                is_inside(p3, pk, p4)
            };
            if s1_inside >= 1 && s2_inside >= 1 {
                if equal_with_radius(p1, p3, rhit) {
                    return Intersection::IntersectsLShapeS1StartS2Start21;
                }
                if equal_with_radius(p1, p4, rhit) {
                    return Intersection::IntersectsLShapeS1StartS2End22;
                }
                if equal_with_radius(p2, p3, rhit) {
                    return Intersection::IntersectsLShapeS1EndS2Start23;
                }
                if equal_with_radius(p2, p4, rhit) {
                    return Intersection::IntersectsLShapeS1EndS2End24;
                }
                if equal_with_radius(p1, pk, rhit) {
                    return Intersection::IntersectsTShapeS1VerticalBar25;
                }
                if equal_with_radius(p2, pk, rhit) {
                    return Intersection::IntersectsTShapeS1VerticalBar26;
                }
                if equal_with_radius(p3, pk, rhit) {
                    return Intersection::IntersectsTShapeS2VerticalBar27;
                }
                if equal_with_radius(p4, pk, rhit) {
                    return Intersection::IntersectsTShapeS2VerticalBar28;
                }
                return Intersection::Intersects1;
            }
            Intersection::NoIntersection0
        }
        ParallelJudgement::ParallelNotEqual => Intersection::NoIntersection0,
        ParallelJudgement::ParallelEqual => {
            if (equal_with_radius(p1, p3, rhit) && equal_with_radius(p2, p4, rhit))
                || (equal_with_radius(p1, p4, rhit) && equal_with_radius(p2, p3, rhit))
            {
                return Intersection::ParallelEqual31;
            }

            parallel_equal_intersection(p1, p2, p3, p4, rhit)
        }
    }
}

fn parallel_equal_intersection(
    p1: Point,
    p2: Point,
    p3: Point,
    p4: Point,
    rhit: f64,
) -> Intersection {
    if equal_with_radius(p1, p3, rhit) {
        if is_inside(p1, p4, p2) == 2 {
            return Intersection::ParallelStartOfS1ContainsStartOfS2_321;
        }
        if is_inside(p3, p2, p4) == 2 {
            return Intersection::ParallelStartOfS2ContainsStartOfS1_322;
        }
        if is_inside(p2, p1, p4) == 2 {
            return Intersection::ParallelStartOfS1IntersectsStartOfS2_323;
        }
    }

    if equal_with_radius(p1, p4, rhit) {
        if is_inside(p1, p3, p2) == 2 {
            return Intersection::ParallelStartOfS1ContainsEndOfS2_331;
        }
        if is_inside(p4, p2, p3) == 2 {
            return Intersection::ParallelEndOfS2ContainsStartOfS1_332;
        }
        if is_inside(p2, p1, p3) == 2 {
            return Intersection::ParallelStartOfS1IntersectsEndOfS2_333;
        }
    }

    if equal_with_radius(p2, p3, rhit) {
        if is_inside(p2, p4, p1) == 2 {
            return Intersection::ParallelEndOfS1ContainsStartOfS2_341;
        }
        if is_inside(p3, p1, p4) == 2 {
            return Intersection::ParallelStartOfS2ContainsEndOfS1_342;
        }
        if is_inside(p1, p2, p4) == 2 {
            return Intersection::ParallelEndOfS1IntersectsStartOfS2_343;
        }
    }

    if equal_with_radius(p2, p4, rhit) {
        if is_inside(p2, p3, p1) == 2 {
            return Intersection::ParallelEndOfS1ContainsEndOfS2_351;
        }
        if is_inside(p4, p1, p3) == 2 {
            return Intersection::ParallelEndOfS2ContainsEndOfS1_352;
        }
        if is_inside(p1, p2, p3) == 2 {
            return Intersection::ParallelEndOfS1IntersectsEndOfS2_353;
        }
    }

    if is_inside(p1, p3, p4) == 2 && is_inside(p3, p4, p2) == 2 {
        return Intersection::ParallelS1IncludesS2_361;
    }
    if is_inside(p1, p4, p3) == 2 && is_inside(p4, p3, p2) == 2 {
        return Intersection::ParallelS1IncludesS2_362;
    }
    if is_inside(p3, p1, p2) == 2 && is_inside(p1, p2, p4) == 2 {
        return Intersection::ParallelS2IncludesS1_363;
    }
    if is_inside(p3, p2, p1) == 2 && is_inside(p2, p1, p4) == 2 {
        return Intersection::ParallelS2IncludesS1_364;
    }
    if is_inside(p1, p3, p2) == 2 && is_inside(p3, p2, p4) == 2 {
        return Intersection::ParallelS1EndOverlapsS2Start371;
    }
    if is_inside(p1, p4, p2) == 2 && is_inside(p4, p2, p3) == 2 {
        return Intersection::ParallelS1EndOverlapsS2End372;
    }
    if is_inside(p3, p1, p4) == 2 && is_inside(p1, p4, p2) == 2 {
        return Intersection::ParallelS1StartOverlapsS2End373;
    }
    if is_inside(p4, p1, p3) == 2 && is_inside(p1, p3, p2) == 2 {
        return Intersection::ParallelS1StartOverlapsS2Start374;
    }

    Intersection::NoIntersection0
}

pub fn get_segment_with_length(segment: &LineSegment, length: f64) -> LineSegment {
    let scale_factor = length / segment.determine_length();
    let new_x =
        segment.determine_ax() + (segment.determine_bx() - segment.determine_ax()) * scale_factor;
    let new_y =
        segment.determine_ay() + (segment.determine_by() - segment.determine_ay()) * scale_factor;
    LineSegment::with_color(segment.a, Point::new(new_x, new_y), segment.color)
}

pub fn is_line_segment_parallel(t1: StraightLine, t2: StraightLine) -> ParallelJudgement {
    is_line_segment_parallel_with_precision(t1, t2, Epsilon::PARALLEL_FOR_EDIT)
}

pub fn is_line_segment_parallel_segments(s1: &LineSegment, s2: &LineSegment) -> ParallelJudgement {
    is_line_segment_parallel(
        StraightLine::from_segment(s1),
        StraightLine::from_segment(s2),
    )
}

pub fn is_line_segment_parallel_with_precision(
    t1: StraightLine,
    t2: StraightLine,
    r: f64,
) -> ParallelJudgement {
    let a1 = t1.a;
    let b1 = t1.b;
    let c1 = t1.c;
    let a2 = t2.a;
    let b2 = t2.b;
    let c2 = t2.c;

    if r <= 0.0 && a1 * b2 - a2 * b1 == 0.0 {
        if (a1 * a1 + b1 * b1) * c2 * c2 == (a2 * a2 + b2 * b2) * c1 * c1 {
            return ParallelJudgement::ParallelEqual;
        }
        return ParallelJudgement::ParallelNotEqual;
    }

    if r > 0.0 && (a1 * b2 - a2 * b1).abs() < r {
        let kyori_t = t2.calculate_distance(t1.find_projection(Point::origin()));
        if kyori_t < r {
            return ParallelJudgement::ParallelEqual;
        }
        return ParallelJudgement::ParallelNotEqual;
    }

    ParallelJudgement::NotParallel
}

pub fn find_intersection_straight_lines(t1: StraightLine, t2: StraightLine) -> Point {
    t1.find_intersection(t2)
}

pub fn line_segment_to_straight_line(segment: &LineSegment) -> StraightLine {
    StraightLine::from_segment(segment)
}

pub fn find_intersection_segments(s1: &LineSegment, s2: &LineSegment) -> Point {
    find_intersection_straight_lines(
        line_segment_to_straight_line(s1),
        line_segment_to_straight_line(s2),
    )
}

pub fn move_parallel(segment: &LineSegment, d: f64) -> LineSegment {
    let ta = StraightLine::from_points(segment.a, segment.b).orthogonalize(segment.a);
    let tb = StraightLine::from_points(segment.a, segment.b).orthogonalize(segment.b);
    let td = StraightLine::from_points(segment.a, segment.b).translate(d);

    LineSegment::new(
        find_intersection_straight_lines(ta, td),
        find_intersection_straight_lines(tb, td),
    )
}

pub fn point_rotate(a: Point, b: Point, d: f64) -> Point {
    point_rotate_scaled(a, b, d, 1.0)
}

pub fn point_rotate_scaled(a: Point, b: Point, d: f64, r: f64) -> Point {
    let mcd = (d * std::f64::consts::PI / 180.0).cos();
    let msd = (d * std::f64::consts::PI / 180.0).sin();
    Point::new(
        r * (mcd * (b.x - a.x) - msd * (b.y - a.y)) + a.x,
        r * (msd * (b.x - a.x) + mcd * (b.y - a.y)) + a.y,
    )
}

pub fn line_segment_rotate(segment: &LineSegment, d: f64) -> LineSegment {
    line_segment_rotate_scaled(segment, d, 1.0)
}

pub fn line_segment_rotate_scaled(segment: &LineSegment, d: f64, r: f64) -> LineSegment {
    LineSegment::new(segment.a, point_rotate_scaled(segment.a, segment.b, d, r))
}

pub fn change_length(segment: &LineSegment, length_multiplier: f64) -> LineSegment {
    let bx1 = length_multiplier * (segment.determine_bx() - segment.determine_ax())
        + segment.determine_ax();
    let by1 = length_multiplier * (segment.determine_by() - segment.determine_ay())
        + segment.determine_ay();
    LineSegment::new(segment.a, Point::new(bx1, by1))
}

pub fn find_line_symmetry_line_segment(s0: &LineSegment, axis: &LineSegment) -> LineSegment {
    LineSegment::new(
        find_line_symmetry_point(axis.a, axis.b, s0.a),
        find_line_symmetry_point(axis.a, axis.b, s0.b),
    )
}

pub fn find_line_symmetry_point(t1: Point, t2: Point, p: Point) -> Point {
    let s1 = StraightLine::from_points(t1, t2);
    let s2 = s1.orthogonalize(p);
    let p1 = find_intersection_straight_lines(s1, s2);
    Point::new(2.0 * p1.x - p.x, 2.0 * p1.y - p.y)
}

pub fn angle_between_m180_180(mut value: f64) -> f64 {
    while value <= -180.0 {
        value += 360.0;
    }
    while value > 180.0 {
        value -= 360.0;
    }
    value
}

pub fn angle_between_0_360(mut value: f64) -> f64 {
    while value < 0.0 {
        value += 360.0;
    }
    while value >= 360.0 {
        value -= 360.0;
    }
    value
}

pub fn angle_between_0_kmax(mut value: f64, kmax: f64) -> f64 {
    while value < 0.0 {
        value += kmax;
    }
    while value >= kmax {
        value -= kmax;
    }
    value
}

pub fn center(ta: Point, tb: Point, tc: Point) -> Point {
    let xa = ta.x;
    let ya = ta.y;
    let xb = tb.x;
    let yb = tb.y;
    let xc = tc.x;
    let yc = tc.y;

    let a = ((xc - xb) * (xc - xb) + (yc - yb) * (yc - yb)).sqrt();
    let b = ((xa - xc) * (xa - xc) + (ya - yc) * (ya - yc)).sqrt();
    let c = ((xb - xa) * (xb - xa) + (yb - ya) * (yb - ya)).sqrt();
    let xd = (c * xc + b * xb) / (b + c);
    let yd = (c * yc + b * yb) / (b + c);
    let xe = (c * xc + a * xa) / (a + c);
    let ye = (c * yc + a * ya) / (a + c);
    let g = xd - xa;
    let h = yd - ya;
    let k = xe - xb;
    let l = ye - yb;
    let p = g * ya - h * xa;
    let q = k * yb - l * xb;
    Point::new(
        (g * q - k * p) / (h * k - g * l),
        (l * p - h * q) / (g * l - h * k),
    )
}

pub fn internal_division_ratio(a: Point, b: Point, s: f64, t: f64) -> Point {
    let error_point = Point::new(-10000.0, -10000.0);
    if distance(a, b) < Epsilon::UNKNOWN_1EN6 {
        return error_point;
    }

    if s == 0.0 {
        if t == 0.0 { error_point } else { a }
    } else if t == 0.0 {
        b
    } else {
        Point::new((t * a.x + s * b.x) / (s + t), (t * a.y + s * b.y) / (s + t))
    }
}

pub fn mid_point(a: Point, b: Point) -> Point {
    Point::mid(a, b)
}

pub fn circle_to_circle_intersection(c1: Circle, c2: Circle) -> CircleIntersection {
    let center_dist = c1.determine_center().distance(c2.determine_center());
    let tangent_dist = c1.r + c2.r;

    if (center_dist - tangent_dist).abs() < Epsilon::UNKNOWN_1EN6 {
        CircleIntersection::Tangent
    } else if center_dist < tangent_dist {
        CircleIntersection::Intersect
    } else {
        CircleIntersection::NoIntersection
    }
}

pub fn circle_to_circle_no_intersection_wo_tooru_straight_line(
    e1: Circle,
    e2: Circle,
) -> StraightLine {
    let a = 2.0 * e1.x - 2.0 * e2.x;
    let b = 2.0 * e1.y - 2.0 * e2.y;
    let c = e2.x * e2.x - e1.x * e1.x + e2.y * e2.y - e1.y * e1.y + e1.r * e1.r - e2.r * e2.r;
    StraightLine::new(a, b, c)
}

pub fn circle_to_circle_no_intersection_wo_musubu_line_segment(
    e1: Circle,
    e2: Circle,
) -> LineSegment {
    let t0 = circle_to_circle_no_intersection_wo_tooru_straight_line(e1, e2);
    let t1 = StraightLine::from_points(e1.determine_center(), e2.determine_center());
    let intersection_t0t1 = find_intersection_straight_lines(t0, t1);
    let length_a = t0.calculate_distance(e1.determine_center());
    let length_b = (e1.r * e1.r - length_a * length_a).sqrt();
    let denominator = (t0.b * t0.b + t0.a * t0.a).sqrt();

    LineSegment::from_coordinates(
        intersection_t0t1.x + t0.b * length_b / denominator,
        intersection_t0t1.y - t0.a * length_b / denominator,
        intersection_t0t1.x - t0.b * length_b / denominator,
        intersection_t0t1.y + t0.a * length_b / denominator,
    )
}

pub fn circle_to_straight_line_no_intersect_wo_connect_line_segment(
    e1: Circle,
    t0: StraightLine,
) -> LineSegment {
    let intersection = find_projection(t0, e1.determine_center());
    let length_a = t0.calculate_distance(e1.determine_center());
    let length_b = (e1.r * e1.r - length_a * length_a).sqrt();
    let denominator = (t0.b * t0.b + t0.a * t0.a).sqrt();

    LineSegment::from_coordinates(
        intersection.x + t0.b * length_b / denominator,
        intersection.y - t0.a * length_b / denominator,
        intersection.x - t0.b * length_b / denominator,
        intersection.y + t0.a * length_b / denominator,
    )
}

pub fn distance_circumference(p0: Point, e0: Circle) -> f64 {
    (distance(p0, e0.determine_center()) - e0.r).abs()
}

pub fn min4(d1: f64, d2: f64, d3: f64, d4: f64) -> f64 {
    d1.min(d2).min(d3).min(d4)
}

pub fn bisection(t1: Point, t2: Point, length: f64) -> LineSegment {
    let tm = Point::new((t1.x + t2.x) / 2.0, (t1.y + t2.y) / 2.0);
    let bai = length / distance(t1, t2);
    let s1 = line_segment_rotate_scaled(&LineSegment::new(tm, t1), 90.0, bai);
    let s2 = line_segment_rotate_scaled(&LineSegment::new(tm, t2), 90.0, bai);
    LineSegment::new(s1.b, s2.b)
}

pub fn is_line_segment_overlapping(s1: &LineSegment, s2: &LineSegment) -> bool {
    determine_line_segment_intersection_with_precision(s1, s2, Epsilon::UNKNOWN_1EN4)
        .is_segment_overlapping()
}

pub fn line_segment_x_kousa_decide(s1: &LineSegment, s2: &LineSegment) -> bool {
    determine_line_segment_intersection_with_precision(s1, s2, Epsilon::UNKNOWN_1EN4)
        == Intersection::Intersects1
}

pub fn line_segment_change_length(s: &LineSegment, new_length: f64) -> LineSegment {
    let dx = s.determine_bx() - s.determine_ax();
    let dy = s.determine_by() - s.determine_ay();
    let new_dx = dx / s.determine_length() * new_length;
    let new_dy = dy / s.determine_length() * new_length;
    LineSegment::new(s.a, Point::new(s.a.x + new_dx, s.a.y + new_dy))
}

pub fn is_point_within_line_span(p0: Point, s0: &LineSegment) -> bool {
    if distance(p0, s0.a) < Epsilon::UNKNOWN_1EN7 || distance(p0, s0.b) < Epsilon::UNKNOWN_1EN7 {
        return true;
    }
    let temp = LineSegment::new(p0, s0.determine_closest_endpoint(p0));
    is_line_segment_parallel_segments(&temp, s0) == ParallelJudgement::ParallelEqual
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParallelJudgement {
    NotParallel,
    ParallelNotEqual,
    ParallelEqual,
}
