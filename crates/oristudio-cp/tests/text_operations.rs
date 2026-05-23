use oristudio_cp::geometry::Point;
use oristudio_cp::model::{CreasePatternModel, TextElement};
use oristudio_cp::operations::text::{
    TextSelectionState, text_create_or_select_pressed, text_delete_at, text_delete_box,
    text_drag_selected,
};

#[test]
fn text_press_creates_selects_and_drags_text() {
    let mut model = CreasePatternModel::default();
    let mut state = TextSelectionState::default();

    text_create_or_select_pressed(&mut model, &mut state, Point::new(10.0, 10.0));
    assert_eq!(model.texts.len(), 1);
    assert_eq!(state.selected, Some(0));
    assert!(state.is_selected);

    text_drag_selected(&mut model, &mut state, Point::new(15.0, 12.0));
    assert_eq!(model.texts[0].position(), Point::new(15.0, 12.0));
    assert!(state.dirty);

    text_create_or_select_pressed(&mut model, &mut state, Point::new(15.0, 12.0));
    assert_eq!(model.texts.len(), 1);
    assert_eq!(state.selected, Some(0));
    assert!(state.is_selected);
}

#[test]
fn text_delete_at_removes_nearest_bounded_text() {
    let mut model = CreasePatternModel::default();
    model.add_text(TextElement::new(10.0, 10.0, "a"));
    model.add_text(TextElement::new(50.0, 50.0, "b"));
    let mut state = TextSelectionState {
        selected: Some(0),
        is_selected: true,
        dirty: true,
        selection_start: None,
    };

    assert!(text_delete_at(
        &mut model,
        &mut state,
        Point::new(11.0, 10.0)
    ));
    assert_eq!(model.texts.len(), 1);
    assert_eq!(model.texts[0].text, "b");
    assert!(!state.is_selected);
    assert_eq!(state.selected, None);
    assert!(!state.dirty);
}

#[test]
fn text_delete_box_removes_intersecting_default_bounds() {
    let mut model = CreasePatternModel::default();
    model.add_text(TextElement::new(10.0, 10.0, "a"));
    model.add_text(TextElement::new(60.0, 60.0, "b"));
    let mut state = TextSelectionState {
        selected: Some(1),
        is_selected: true,
        dirty: false,
        selection_start: None,
    };

    let deleted = text_delete_box(
        &mut model,
        &mut state,
        Point::new(0.0, 0.0),
        Point::new(40.0, 40.0),
    );

    assert_eq!(deleted, 1);
    assert_eq!(model.texts.len(), 1);
    assert_eq!(model.texts[0].text, "b");
    assert_eq!(state.selected, Some(0));
    assert!(state.is_selected);
}
