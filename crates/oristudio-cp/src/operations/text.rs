//! Text annotation operations ported from Oriedita's text handler.

use crate::geometry::Point;
use crate::model::{CreasePatternModel, TextElement};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextSelectionState {
    pub selected: Option<usize>,
    pub is_selected: bool,
    pub dirty: bool,
    pub selection_start: Option<Point>,
}

/// Oriedita `MouseHandlerText` create/select press with identity-camera semantics.
pub fn text_create_or_select_pressed(
    model: &mut CreasePatternModel,
    state: &mut TextSelectionState,
    point: Point,
) {
    if state.is_selected {
        if !try_select_text(model, state, point) {
            state.is_selected = false;
            if state.dirty {
                state.dirty = false;
            }
        }
    } else {
        select_or_create_text(model, state, point);
    }
    state.selection_start = Some(point);
}

/// Oriedita `MouseHandlerText` drag movement for the selected annotation.
pub fn text_drag_selected(
    model: &mut CreasePatternModel,
    state: &mut TextSelectionState,
    point: Point,
) {
    if !state.is_selected {
        return;
    }
    let Some(selected) = state.selected else {
        return;
    };
    let Some(start) = state.selection_start else {
        return;
    };
    let Some(text) = model.texts.get_mut(selected) else {
        state.is_selected = false;
        state.selected = None;
        return;
    };

    text.x.0 += point.x - start.x;
    text.y.0 += point.y - start.y;
    state.dirty = true;
    state.selection_start = Some(point);
}

/// Oriedita point delete for the nearest text whose default bounds contain the point.
pub fn text_delete_at(
    model: &mut CreasePatternModel,
    state: &mut TextSelectionState,
    point: Point,
) -> bool {
    let Some(index) = find_nearest_text(model, state, point) else {
        return false;
    };

    model.texts.remove(index);
    reconcile_deleted_text(state, index);
    state.dirty = false;
    true
}

/// Oriedita box delete for text annotations with identity-camera/default-bounds semantics.
pub fn text_delete_box(
    model: &mut CreasePatternModel,
    state: &mut TextSelectionState,
    first: Point,
    second: Point,
) -> usize {
    let selection = TextBounds::from_points(first, second);
    let mut deleted = Vec::new();
    for (index, text) in model.texts.iter().enumerate() {
        let bounds = text_delete_bounds(text);
        if selection.contains_rect(bounds)
            || selection.intersects(bounds)
            || bounds.contains_rect(selection)
        {
            deleted.push(index);
        }
    }

    for index in deleted.iter().rev() {
        model.texts.remove(*index);
        reconcile_deleted_text(state, *index);
    }
    if !deleted.is_empty() {
        state.dirty = false;
    }
    deleted.len()
}

pub fn text_reset(state: &mut TextSelectionState) {
    state.selection_start = None;
}

fn select_or_create_text(
    model: &mut CreasePatternModel,
    state: &mut TextSelectionState,
    point: Point,
) {
    if !try_select_text(model, state, point) {
        if state.is_selected && state.dirty {
            state.dirty = false;
        }
        model.add_text(TextElement::new(point.x, point.y, ""));
        state.selected = Some(model.texts.len() - 1);
    }
    state.is_selected = true;
}

fn try_select_text(
    model: &CreasePatternModel,
    state: &mut TextSelectionState,
    point: Point,
) -> bool {
    let Some(index) = find_nearest_text(model, state, point) else {
        return false;
    };
    if state.is_selected && state.selected != Some(index) && state.dirty {
        state.dirty = false;
    }
    state.selected = Some(index);
    true
}

fn find_nearest_text(
    model: &CreasePatternModel,
    state: &TextSelectionState,
    point: Point,
) -> Option<usize> {
    let mut min_distance = 100_000_000.0;
    let mut nearest = None;
    let click = IntPoint::from_point(point);
    for (index, text) in model.texts.iter().enumerate() {
        let bounds =
            text_selection_bounds(text, state.is_selected && state.selected == Some(index));
        if bounds.contains_point(click) {
            let distance = point.distance(text.position());
            if distance < min_distance {
                min_distance = distance;
                nearest = Some(index);
            }
        }
    }
    nearest
}

fn reconcile_deleted_text(state: &mut TextSelectionState, deleted_index: usize) {
    if state.selected == Some(deleted_index) {
        state.selected = None;
        state.is_selected = false;
    } else if let Some(selected) = state.selected
        && deleted_index < selected
    {
        state.selected = Some(selected - 1);
    }
}

fn text_selection_bounds(text: &TextElement, selected: bool) -> TextBounds {
    let base = default_text_bounds();
    let selection_radius = if selected { 7 } else { 1 };
    let position = text.position();
    TextBounds {
        x: position.x as i32 - 3 - selection_radius,
        y: position.y as i32 - 10 - selection_radius,
        width: base.width + 8 + selection_radius * 5,
        height: base.height + 10 + selection_radius * 5,
    }
}

fn text_delete_bounds(text: &TextElement) -> TextBounds {
    let base = default_text_bounds();
    let position = text.position();
    TextBounds {
        x: position.x as i32,
        y: position.y as i32,
        width: base.width,
        height: base.height,
    }
}

fn default_text_bounds() -> TextBounds {
    TextBounds {
        x: 0,
        y: 0,
        width: 25,
        height: 3,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IntPoint {
    x: i32,
    y: i32,
}

impl IntPoint {
    fn from_point(point: Point) -> Self {
        Self {
            x: point.x as i32,
            y: point.y as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TextBounds {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl TextBounds {
    fn from_points(first: Point, second: Point) -> Self {
        let mut min_x = first.x;
        let mut max_x = second.x;
        if min_x > max_x {
            std::mem::swap(&mut min_x, &mut max_x);
        }
        let mut min_y = first.y;
        let mut max_y = second.y;
        if min_y > max_y {
            std::mem::swap(&mut min_y, &mut max_y);
        }
        Self {
            x: min_x as i32,
            y: min_y as i32,
            width: (max_x - min_x) as i32,
            height: (max_y - min_y) as i32,
        }
    }

    fn contains_point(self, point: IntPoint) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x + self.width
            && point.y < self.y + self.height
    }

    fn contains_rect(self, other: Self) -> bool {
        self.contains_point(IntPoint {
            x: other.x,
            y: other.y,
        }) && self.contains_point(IntPoint {
            x: other.x + other.width - 1,
            y: other.y + other.height - 1,
        })
    }

    fn intersects(self, other: Self) -> bool {
        self.x < other.x + other.width
            && other.x < self.x + self.width
            && self.y < other.y + other.height
            && other.y < self.y + self.height
    }
}
