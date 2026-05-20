//! Arrangement and cleanup helpers ported from Oriedita `FoldLineSet` workers.

use crate::geometry::{
    Intersection, determine_line_segment_intersection,
    determine_line_segment_intersection_with_precision,
};
use crate::model::CreasePatternModel;

/// Oriedita sentinel used by `FoldLineSet.removeOverlappingLines()`.
const DEFAULT_PRECISION_SENTINEL: f64 = -9999.9;

/// Remove duplicate overlapping line segments in Oriedita's order.
///
/// This ports the observable behavior of `FoldLineSet.removeOverlappingLines`:
/// when two line segments classify as `PARALLEL_EQUAL_31`, the later line is
/// removed and the earlier line survives. The upstream implementation uses
/// spatial acceleration; this first Rust port intentionally keeps the same
/// pair-order semantics with a direct scan so correctness is visible.
pub fn remove_overlapping_lines(model: &mut CreasePatternModel) {
    remove_overlapping_lines_with_precision(model, DEFAULT_PRECISION_SENTINEL);
}

/// Remove duplicate overlapping line segments with Oriedita's optional radius.
pub fn remove_overlapping_lines_with_precision(model: &mut CreasePatternModel, radius: f64) {
    let mut remove = vec![false; model.line_segments.len()];

    let len = model.line_segments.len();
    for i in 0..len.saturating_sub(1) {
        for (j, remove_j) in remove.iter_mut().enumerate().take(len).skip(i + 1) {
            let intersection = if radius <= DEFAULT_PRECISION_SENTINEL {
                determine_line_segment_intersection(
                    &model.line_segments[i],
                    &model.line_segments[j],
                )
            } else {
                determine_line_segment_intersection_with_precision(
                    &model.line_segments[i],
                    &model.line_segments[j],
                    radius,
                )
            };

            if intersection == Intersection::ParallelEqual31 {
                *remove_j = true;
            }
        }
    }

    model.line_segments = model
        .line_segments
        .iter()
        .enumerate()
        .filter_map(|(index, segment)| (!remove[index]).then_some(segment.clone()))
        .collect();
}
