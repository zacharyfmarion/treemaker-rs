//! Construction/drawing commands ported from Oriedita handlers.

use crate::geometry::{Epsilon, LineSegment};
use crate::model::CreasePatternModel;
use crate::operations::arrangement::add_line_segment_like_worker;

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
