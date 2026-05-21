//! Construction/drawing commands ported from Oriedita handlers.

use crate::geometry::{Epsilon, LineSegment, find_line_symmetry_line_segment};
use crate::model::CreasePatternModel;
use crate::operations::arrangement::{
    add_line_segment_like_worker, divide_line_segment_with_new_lines,
};
use crate::operations::selection::unselect_all;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawCreaseTarget {
    FoldLine,
    AuxLine,
}

/// Oriedita free/restricted draw-crease insertion after endpoints are resolved.
pub fn draw_crease_segment(
    model: &mut CreasePatternModel,
    segment: &LineSegment,
    target: DrawCreaseTarget,
) -> bool {
    if !Epsilon::HIGH.gt0(segment.determine_length()) {
        return false;
    }

    match target {
        DrawCreaseTarget::FoldLine => add_line_segment_like_worker(model, segment),
        DrawCreaseTarget::AuxLine => model.add_aux_line_segment(segment.clone()),
    }
    true
}

/// Oriedita `DRAW_CREASE_SYMMETRIC_12` mutation after the mirror axis is known.
pub fn mirror_selected_lines(model: &mut CreasePatternModel, axis: &LineSegment) -> usize {
    let selected: Vec<_> = model
        .line_segments
        .iter()
        .filter(|segment| segment.selected == 2)
        .cloned()
        .collect();
    if selected.is_empty() {
        return 0;
    }

    let original_end = model.line_segments.len();
    for segment in &selected {
        let mirrored =
            find_line_symmetry_line_segment(segment, axis).with_line_color(segment.color);
        model.add_line_segment(mirrored);
    }
    let added_end = model.line_segments.len();
    divide_line_segment_with_new_lines(model, original_end, added_end);
    unselect_all(model);
    selected.len()
}
