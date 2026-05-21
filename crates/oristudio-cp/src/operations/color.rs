//! Line color/type operations ported from Oriedita handlers and `FoldLineSet`.

use crate::geometry::{
    LineColor, LineSegment, determine_line_segment_distance, is_line_segment_overlapping,
};
use crate::model::{CreasePatternModel, CustomLineType};
use crate::operations::arrangement::divide_line_segment_with_new_lines;

/// Oriedita `FoldLineSet.setColor(Collection<LineSegment>, LineColor)`.
pub fn set_line_color_for_segments(
    model: &mut CreasePatternModel,
    lines: &[LineSegment],
    color: LineColor,
) -> usize {
    let mut changed = 0;
    let mut aux = Vec::new();

    for index in 0..model.line_segments.len() {
        if !lines.contains(&model.line_segments[index]) {
            continue;
        }

        if model.line_segments[index].color == color {
            continue;
        }

        changed += 1;
        if model.line_segments[index].color == LineColor::Cyan3 {
            aux.push(model.line_segments[index].clone());
        } else {
            let segment = model.line_segments[index].clone();
            model.line_segments[index] = segment.with_line_color(color);
        }
    }

    replace_aux_lines(model, color, &aux);
    changed
}

/// Convenience wrapper for index-based callers.
pub fn set_line_color_for_indices(
    model: &mut CreasePatternModel,
    indices: &[usize],
    color: LineColor,
) -> usize {
    let lines: Vec<_> = indices
        .iter()
        .filter_map(|index| model.line_segments.get(*index).cloned())
        .collect();
    set_line_color_for_segments(model, &lines, color)
}

/// Oriedita selected-line mountain command without the UI box-selection step.
pub fn make_mountain(model: &mut CreasePatternModel, indices: &[usize]) -> usize {
    set_line_color_for_indices(model, indices, LineColor::Red1)
}

/// Oriedita selected-line valley command without the UI box-selection step.
pub fn make_valley(model: &mut CreasePatternModel, indices: &[usize]) -> usize {
    set_line_color_for_indices(model, indices, LineColor::Blue2)
}

/// Oriedita selected-line edge command without the UI box-selection step.
pub fn make_edge(model: &mut CreasePatternModel, indices: &[usize]) -> usize {
    set_line_color_for_indices(model, indices, LineColor::Black0)
}

/// Oriedita `CREASE_MAKE_AUX_60` selected-line mutation.
pub fn make_aux(model: &mut CreasePatternModel, indices: &[usize]) -> usize {
    set_line_color_for_indices(model, indices, LineColor::Cyan3)
}

/// Oriedita `REPLACE_LINE_TYPE_SELECT_72` mutation over explicit line indices.
pub fn replace_line_type_for_indices(
    model: &mut CreasePatternModel,
    indices: &[usize],
    from: CustomLineType,
    to: CustomLineType,
) -> usize {
    let lines = lines_matching_type_for_indices(model, indices, from);
    set_line_color_for_segments(model, &lines, to.line_color())
}

/// Replace currently selected lines matching one Oriedita custom line type.
pub fn replace_selected_line_type(
    model: &mut CreasePatternModel,
    from: CustomLineType,
    to: CustomLineType,
) -> usize {
    let indices = selected_indices(model);
    replace_line_type_for_indices(model, &indices, from, to)
}

/// Oriedita `DELETE_LINE_TYPE_SELECT_73` mutation over explicit line indices.
pub fn delete_line_type_for_indices(
    model: &mut CreasePatternModel,
    indices: &[usize],
    line_type: CustomLineType,
) -> usize {
    let lines = lines_matching_type_for_indices(model, indices, line_type);
    delete_lines_by_value(model, &lines)
}

/// Delete currently selected lines matching one Oriedita custom line type.
pub fn delete_selected_line_type(
    model: &mut CreasePatternModel,
    line_type: CustomLineType,
) -> usize {
    let indices = selected_indices(model);
    delete_line_type_for_indices(model, &indices, line_type)
}

/// Oriedita `CREASE_TOGGLE_MV_58` selected-line mutation.
pub fn toggle_mountain_valley(model: &mut CreasePatternModel, indices: &[usize]) -> usize {
    let mut changed = 0;
    for index in indices {
        let Some(segment) = model.line_segments.get(*index).cloned() else {
            continue;
        };

        let color = match segment.color {
            LineColor::Red1 => LineColor::Blue2,
            LineColor::Blue2 => LineColor::Red1,
            _ => continue,
        };
        model.line_segments[*index] = segment.with_line_color(color);
        changed += 1;
    }
    changed
}

/// Oriedita `CREASE_ADVANCE_TYPE_30` release behavior for one line.
pub fn advance_line_type(model: &mut CreasePatternModel, index: usize) -> bool {
    let Some(segment) = model.line_segments.get(index).cloned() else {
        return false;
    };

    let next = match (segment.color, segment.selected) {
        (LineColor::Black0, 0) => segment.with_selected(2),
        (LineColor::Black0, 2) => segment.with_line_color(LineColor::Red1).with_selected(0),
        (LineColor::Red1, 0) => segment.with_line_color(LineColor::Blue2),
        (LineColor::Blue2, 0) => segment.with_line_color(LineColor::Black0),
        _ => segment,
    };
    model.line_segments[index] = next;
    true
}

/// Oriedita `CREASE_MAKE_MV_34` mutation over overlapping lines.
pub fn alternate_mountain_valley_along(
    model: &mut CreasePatternModel,
    guide: &LineSegment,
    start_color: LineColor,
) -> usize {
    let mut overlapping: Vec<_> = model
        .line_segments
        .iter()
        .enumerate()
        .filter(|(_, segment)| is_line_segment_overlapping(segment, guide))
        .map(|(index, segment)| (index, determine_line_segment_distance(guide.a, segment)))
        .collect();
    overlapping.sort_by(|left, right| {
        left.1
            .partial_cmp(&right.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut color = start_color;
    let mut changed = 0;
    for (index, _) in overlapping {
        let segment = model.line_segments[index].clone();
        model.line_segments[index] = segment.with_line_color(color);
        changed += 1;
        color = match color {
            LineColor::Red1 => LineColor::Blue2,
            LineColor::Blue2 => LineColor::Red1,
            other => other,
        };
    }
    changed
}

fn replace_aux_lines(model: &mut CreasePatternModel, color: LineColor, aux: &[LineSegment]) {
    for segment in aux {
        let replacement = segment.with_line_color(color);
        let Some(index) = model
            .line_segments
            .iter()
            .position(|candidate| candidate == segment)
        else {
            continue;
        };

        model.line_segments.remove(index);
        let original_end = model.line_segments.len();
        model.add_line_segment(replacement);
        divide_line_segment_with_new_lines(model, original_end, original_end + 1);
    }
}

fn selected_indices(model: &CreasePatternModel) -> Vec<usize> {
    model
        .line_segments
        .iter()
        .enumerate()
        .filter(|(_, segment)| segment.selected == 2)
        .map(|(index, _)| index)
        .collect()
}

fn lines_matching_type_for_indices(
    model: &CreasePatternModel,
    indices: &[usize],
    line_type: CustomLineType,
) -> Vec<LineSegment> {
    indices
        .iter()
        .filter_map(|index| model.line_segments.get(*index))
        .filter(|segment| line_type.matches(segment.color))
        .cloned()
        .collect()
}

fn delete_lines_by_value(model: &mut CreasePatternModel, lines: &[LineSegment]) -> usize {
    let mut deleted = 0;
    for line in lines {
        if let Some(index) = model
            .line_segments
            .iter()
            .position(|candidate| candidate == line)
        {
            model.line_segments.remove(index);
            deleted += 1;
        }
    }
    deleted
}
