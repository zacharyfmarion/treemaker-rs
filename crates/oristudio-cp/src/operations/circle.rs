use crate::geometry::{
    Circle, CircleIntersection, Epsilon, LineColor, LineSegment, Point, RgbColor, StraightLine,
    angle, circle_to_circle_intersection, determine_line_segment_distance, distance,
    internal_division_ratio,
};
use crate::model::CreasePatternModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircleInversionOutput {
    None,
    Circle,
    LineSegment,
}

pub fn change_custom_color_for_indices(
    model: &mut CreasePatternModel,
    circle_indices: &[usize],
    aux_line_indices: &[usize],
    color: RgbColor,
) -> usize {
    let mut changed = 0;
    for &index in circle_indices {
        if let Some(circle) = model.circles.get_mut(index) {
            *circle = circle.with_customized_color(color);
            changed += 1;
        }
    }

    let selected_aux_lines: Vec<LineSegment> = aux_line_indices
        .iter()
        .filter_map(|&index| model.line_segments.get(index).cloned())
        .collect();
    for selected in selected_aux_lines {
        if selected.color != LineColor::Cyan3 {
            continue;
        }
        if let Some(line) = model
            .line_segments
            .iter_mut()
            .find(|line| **line == selected)
        {
            *line = line.with_customized_color(color);
            changed += 1;
        }
    }

    changed
}

pub fn change_color(
    model: &mut CreasePatternModel,
    circle_indices: &[usize],
    aux_line_indices: &[usize],
    color: RgbColor,
) -> usize {
    change_custom_color_for_indices(model, circle_indices, aux_line_indices, color)
}

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

/// Add Oriedita's separate circle from a resolved center and radius segment.
pub fn separate(
    model: &mut CreasePatternModel,
    center: Point,
    radius_a: Point,
    radius_b: Point,
) -> bool {
    model.add_circle(Circle::from_center(
        center,
        distance(radius_a, radius_b),
        LineColor::Cyan3,
    ));
    true
}

/// Add a concentric circle using the selected circle plus resolved radius delta.
pub fn concentric(
    model: &mut CreasePatternModel,
    original: Circle,
    radius_a: Point,
    radius_b: Point,
) -> bool {
    model.add_circle(Circle::from_center(
        original.determine_center(),
        original.r + distance(radius_a, radius_b),
        LineColor::Cyan3,
    ));
    true
}

/// Add the pair produced by Oriedita's two-circle concentric select mode.
pub fn concentric_two_circle_select(
    model: &mut CreasePatternModel,
    circle1: Circle,
    circle2: Circle,
) -> usize {
    if circle_to_circle_intersection(circle1, circle2) == CircleIntersection::Tangent {
        return 0;
    }

    let center_line_length = distance(circle1.determine_center(), circle2.determine_center());
    let concentric_offset = (center_line_length - circle1.r - circle2.r) / 2.0;
    model.add_circle(Circle::from_center(
        circle1.determine_center(),
        circle1.r + concentric_offset,
        LineColor::Cyan3,
    ));
    model.add_circle(Circle::from_center(
        circle2.determine_center(),
        circle2.r + concentric_offset,
        LineColor::Cyan3,
    ));
    2
}

/// Return the selectable indicator circles from Oriedita's concentric-select mode.
pub fn concentric_select_candidates(
    target: Circle,
    reference1: Circle,
    reference2: Circle,
) -> Vec<Circle> {
    let delta_r = (reference2.r - reference1.r).abs();
    if Epsilon::HIGH.eq0(delta_r) {
        return Vec::new();
    }

    let outer_r = target.r + delta_r;
    let inner_r = target.r - delta_r;
    let mut candidates = vec![Circle::from_center(
        target.determine_center(),
        outer_r,
        LineColor::Magenta5,
    )];
    if Epsilon::HIGH.gt0(inner_r) {
        candidates.push(Circle::from_center(
            target.determine_center(),
            inner_r,
            LineColor::Magenta5,
        ));
    }
    candidates
}

/// Add one resolved concentric-select indicator as the final cyan circle.
pub fn concentric_select(
    model: &mut CreasePatternModel,
    target: Circle,
    reference1: Circle,
    reference2: Circle,
    candidate_index: usize,
) -> bool {
    let candidates = concentric_select_candidates(target, reference1, reference2);
    let Some(candidate) = candidates.get(candidate_index) else {
        return false;
    };

    model.add_circle(candidate.with_color(LineColor::Cyan3));
    true
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

/// Invert a circle through another circle, appending Oriedita's resulting object.
pub fn invert_circle(
    model: &mut CreasePatternModel,
    subject: Circle,
    inversion: Circle,
) -> CircleInversionOutput {
    if (distance(subject.determine_center(), inversion.determine_center()) - subject.r).abs()
        < Epsilon::UNKNOWN_1EN7
    {
        model.add_line_segment(inversion.turn_around_circle_to_line_segment(subject));
        return CircleInversionOutput::LineSegment;
    }

    model.add_circle(
        inversion
            .turn_around_circle(subject)
            .with_color(LineColor::Cyan3),
    );
    CircleInversionOutput::Circle
}

/// Invert a line segment through a circle, appending Oriedita's resulting circle.
pub fn invert_line_segment(
    model: &mut CreasePatternModel,
    subject: &LineSegment,
    inversion: Circle,
) -> CircleInversionOutput {
    if StraightLine::from_segment(subject).calculate_distance(inversion.determine_center())
        < Epsilon::UNKNOWN_1EN7
    {
        return CircleInversionOutput::None;
    }

    model.add_circle(
        inversion
            .turn_around_line_segment_to_circle(subject)
            .with_color(LineColor::Cyan3),
    );
    CircleInversionOutput::Circle
}

/// Apply Oriedita's zero-radius circle pruning worker.
pub fn organize(model: &mut CreasePatternModel) -> usize {
    let mut deleted = 0;
    for index in (0..model.circles.len()).rev() {
        if organize_circle(model, index) {
            deleted += 1;
        }
    }
    deleted
}

pub fn organize_circle(model: &mut CreasePatternModel, index: usize) -> bool {
    let ies3 = determine_circle_state(model, index, 3);
    let ies4 = determine_circle_state(model, index, 4);
    let ies5 = determine_circle_state(model, index, 5);

    if ies3 == 100000 || ies3 == 2 || (ies3 == 1 && ies4 >= 1) || (ies3 == 1 && ies5 >= 1) {
        return false;
    }

    if index >= model.circles.len() {
        return false;
    }

    model.circles.remove(index);
    true
}

pub fn determine_circle_state(model: &CreasePatternModel, index: usize, mode: i32) -> i32 {
    let Some(circle) = model.circles.get(index) else {
        return 100000;
    };
    let er0 = circle.r;
    let ec0 = circle.determine_center();

    let mut ir1 = 0;
    let mut ir2 = 0;
    let mut ir3 = 0;
    let mut ir4 = 0;
    let mut ir5 = 0;

    if er0 < Epsilon::UNKNOWN_1EN7 {
        for (other_index, other) in model.circles.iter().enumerate() {
            if other_index == index {
                continue;
            }

            let er1 = other.r;
            let ec1 = other.determine_center();
            if er1 < Epsilon::UNKNOWN_1EN7 {
                if distance(ec0, ec1) < Epsilon::UNKNOWN_1EN7 {
                    ir1 += 1;
                }
            } else if distance(ec0, ec1) < Epsilon::UNKNOWN_1EN7 {
                ir2 += 1;
            } else if (distance(ec0, ec1) - er1).abs() < Epsilon::UNKNOWN_1EN7 {
                ir3 += 1;
            }
        }

        for segment in &model.line_segments {
            if determine_line_segment_distance(ec0, segment) < Epsilon::UNKNOWN_1EN6 {
                if segment.color.number() <= 2 {
                    ir4 += 1;
                } else if segment.color == LineColor::Cyan3 {
                    ir5 += 1;
                }
            }
        }

        ir1 = ir1.min(2);
        ir2 = ir2.min(2);
        ir3 = ir3.min(2);
        ir4 = ir4.min(2);
        ir5 = ir5.min(2);

        return match mode {
            0 => ir1 + ir2 * 10 + ir3 * 100 + ir4 * 1000 + ir5 * 10000,
            1 => ir1,
            2 => ir2,
            3 => ir3,
            4 => ir4,
            5 => ir5,
            _ => 100000,
        };
    }

    100000
}

fn is_flat_angle(value: f64) -> bool {
    value.abs() < Epsilon::UNKNOWN_1EN6
        || (value - 180.0).abs() < Epsilon::UNKNOWN_1EN6
        || (value - 360.0).abs() < Epsilon::UNKNOWN_1EN6
}
