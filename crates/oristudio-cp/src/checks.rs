//! Oriedita-compatible crease-pattern diagnostic helpers.

use crate::geometry::{
    Epsilon, Intersection, LineColor, LineSegment,
    determine_line_segment_intersection_with_precision, equal_with_radius,
    find_intersection_segments, find_line_symmetry_line_segment,
};
use crate::model::CreasePatternModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlatFoldableBoundaryCheck {
    pub color: LineColor,
    pub suitable_intersections: bool,
    pub crossing_count: usize,
}

/// Oriedita `FLAT_FOLDABLE_CHECK_63` result coloring once the boundary loop is resolved.
pub fn flat_foldable_boundary_check(
    model: &CreasePatternModel,
    boundary: &mut [LineSegment],
) -> FlatFoldableBoundaryCheck {
    let mut suitable_intersections = true;
    let mut ordered_crossings = Vec::new();

    for boundary_segment in boundary.iter() {
        let mut segment_crossings = Vec::<(f64, LineSegment)>::new();
        for segment in &model.line_segments {
            let intersection = determine_line_segment_intersection_with_precision(
                segment,
                boundary_segment,
                Epsilon::UNKNOWN_1EN4,
            );
            if intersection != Intersection::NoIntersection0
                && intersection != Intersection::Intersects1
            {
                suitable_intersections = false;
            }

            if intersection == Intersection::Intersects1 && segment.color.number() < 3 {
                segment_crossings.push((
                    boundary_segment
                        .a
                        .distance(find_intersection_segments(segment, boundary_segment)),
                    segment.clone(),
                ));
            }
        }

        segment_crossings.sort_by(|left, right| left.0.total_cmp(&right.0));
        ordered_crossings.extend(segment_crossings.into_iter().map(|(_, segment)| segment));
    }

    let color = if suitable_intersections {
        flat_foldable_boundary_color(&ordered_crossings)
    } else {
        LineColor::Yellow7
    };

    if suitable_intersections {
        for segment in boundary {
            *segment = segment.with_line_color(color);
        }
    }

    FlatFoldableBoundaryCheck {
        color,
        suitable_intersections,
        crossing_count: ordered_crossings.len(),
    }
}

fn flat_foldable_boundary_color(crossings: &[LineSegment]) -> LineColor {
    if !crossings.len().is_multiple_of(2) {
        return LineColor::Magenta5;
    }
    if crossings.is_empty() {
        return LineColor::Cyan3;
    }

    let mut moved = crossings[0].clone();
    for crossing in crossings.iter().skip(1) {
        moved = find_line_symmetry_line_segment(&moved, crossing);
    }

    if equal_with_radius(crossings[0].a, moved.a, Epsilon::UNKNOWN_1EN4)
        && equal_with_radius(crossings[0].b, moved.b, Epsilon::UNKNOWN_1EN4)
    {
        LineColor::Cyan3
    } else {
        LineColor::Magenta5
    }
}
