use crate::geometry::{
    Circle, Epsilon, LineColor, LineSegment, Point, StraightLine, angle, distance,
    internal_division_ratio,
};
use crate::model::CreasePatternModel;

/// Add the circle produced by Oriedita's restricted circle draw after the UI has
/// resolved both snapped points.
pub fn draw(model: &mut CreasePatternModel, center: Point, radius_point: Point) -> bool {
    model.add_circle(Circle::from_center(
        center,
        distance(center, radius_point),
        LineColor::Cyan3,
    ));
    true
}

/// Add the circle produced by Oriedita's free circle draw after point snapping.
pub fn free(model: &mut CreasePatternModel, center: Point, radius_point: Point) -> bool {
    if center == radius_point {
        return false;
    }

    draw(model, center, radius_point)
}

/// Add Oriedita's circumcircle for three non-collinear points.
pub fn through_three_points(
    model: &mut CreasePatternModel,
    p1: Point,
    p2: Point,
    p3: Point,
) -> bool {
    let sen1 = LineSegment::new(p1, p2);
    let sen2 = LineSegment::new(p2, p3);
    let sen3 = LineSegment::new(p3, p1);

    if is_flat_angle(angle((&sen1, &sen2)))
        || is_flat_angle(angle((&sen2, &sen3)))
        || is_flat_angle(angle((&sen3, &sen1)))
    {
        return false;
    }

    let t1 = StraightLine::from_segment(&sen1)
        .orthogonalize(internal_division_ratio(sen1.a, sen1.b, 1.0, 1.0));
    let t2 = StraightLine::from_segment(&sen2)
        .orthogonalize(internal_division_ratio(sen2.a, sen2.b, 1.0, 1.0));
    let center = t1.find_intersection(t2);
    model.add_circle(Circle::from_center(
        center,
        distance(p1, center),
        LineColor::Cyan3,
    ));
    true
}

fn is_flat_angle(value: f64) -> bool {
    value.abs() < Epsilon::UNKNOWN_1EN6
        || (value - 180.0).abs() < Epsilon::UNKNOWN_1EN6
        || (value - 360.0).abs() < Epsilon::UNKNOWN_1EN6
}
