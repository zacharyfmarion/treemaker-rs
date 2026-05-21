//! Point and line-division commands ported from Oriedita handlers.

use crate::geometry::{Epsilon, LineSegment, Point};
use crate::model::CreasePatternModel;
use crate::operations::arrangement::divide_line_segment_with_new_lines;

/// Oriedita `LINE_SEGMENT_DIVISION_27` mutation after endpoints are known.
pub fn divide_segment_by_count(
    model: &mut CreasePatternModel,
    segment: &LineSegment,
    division_count: usize,
) -> usize {
    if division_count == 0 || !Epsilon::HIGH.gt0(segment.determine_length()) {
        return 0;
    }

    for index in 0..division_count {
        let count = division_count as f64;
        let i = index as f64;
        let a = Point::new(
            ((count - i) * segment.determine_ax() + i * segment.determine_bx()) / count,
            ((count - i) * segment.determine_ay() + i * segment.determine_by()) / count,
        );
        let b = Point::new(
            ((count - i - 1.0) * segment.determine_ax() + (i + 1.0) * segment.determine_bx())
                / count,
            ((count - i - 1.0) * segment.determine_ay() + (i + 1.0) * segment.determine_by())
                / count,
        );
        add_line_segment_like_worker(model, &segment.with_coordinates(a, b));
    }

    division_count
}

/// Oriedita `LINE_SEGMENT_RATIO_SET_28` mutation after endpoints and ratio are known.
pub fn divide_segment_by_ratio(
    model: &mut CreasePatternModel,
    segment: &LineSegment,
    ratio_s: f64,
    ratio_t: f64,
) -> usize {
    if !Epsilon::HIGH.gt0(segment.determine_length()) {
        return 0;
    }

    let drag_segment = segment.with_coordinates(segment.b, segment.a);
    if (ratio_s == 0.0 && ratio_t != 0.0) || (ratio_s != 0.0 && ratio_t == 0.0) {
        add_line_segment_like_worker(model, &drag_segment);
        return 1;
    }

    if ratio_s != 0.0 && ratio_t != 0.0 {
        let nx = (ratio_t * drag_segment.determine_bx() + ratio_s * drag_segment.determine_ax())
            / (ratio_s + ratio_t);
        let ny = (ratio_t * drag_segment.determine_by() + ratio_s * drag_segment.determine_ay())
            / (ratio_s + ratio_t);
        let division = Point::new(nx, ny);
        add_line_segment_like_worker(
            model,
            &drag_segment.with_coordinates(drag_segment.a, division),
        );
        add_line_segment_like_worker(
            model,
            &drag_segment.with_coordinates(drag_segment.b, division),
        );
        return 2;
    }

    0
}

fn add_line_segment_like_worker(model: &mut CreasePatternModel, segment: &LineSegment) {
    let original_end = model.line_segments.len();
    model.add_line_segment(segment.clone());
    divide_line_segment_with_new_lines(model, original_end, original_end + 1);
}
