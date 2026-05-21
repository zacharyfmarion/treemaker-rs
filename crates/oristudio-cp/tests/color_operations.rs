use oristudio_cp::geometry::{LineColor, LineSegment, Point};
use oristudio_cp::model::{CreasePatternModel, CustomLineType};
use oristudio_cp::operations::color::{
    advance_line_type, alternate_mountain_valley_along, delete_line_type_for_indices,
    delete_selected_line_type, make_mountain, replace_line_type_for_indices,
    replace_selected_line_type, set_line_color_for_indices, toggle_mountain_valley,
};

#[test]
fn set_line_color_for_indices_changes_non_aux_lines_in_place() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);

    let changed = set_line_color_for_indices(&mut model, &[0], LineColor::Blue2);

    assert_eq!(changed, 1);
    assert_eq!(model.line_segments[0].color, LineColor::Blue2);
}

#[test]
fn set_line_color_for_indices_replaces_aux_lines_with_insertion_splitting() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);
    model.add_line(
        Point::new(5.0, -5.0),
        Point::new(5.0, 5.0),
        LineColor::Cyan3,
    );

    let changed = make_mountain(&mut model, &[1]);

    assert_eq!(changed, 1);
    assert_eq!(model.line_segments.len(), 4);
    assert!(
        model
            .line_segments
            .iter()
            .all(|segment| segment.color == LineColor::Red1)
    );
}

#[test]
fn set_line_color_for_indices_changes_equal_duplicates_like_oriedita_hashset() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);

    let changed = set_line_color_for_indices(&mut model, &[1], LineColor::Blue2);

    assert_eq!(changed, 2);
    assert_eq!(model.line_segments[0].color, LineColor::Blue2);
    assert_eq!(model.line_segments[1].color, LineColor::Blue2);
}

#[test]
fn replace_line_type_for_indices_filters_by_custom_type() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(1.0, 0.0), LineColor::Red1);
    model.add_line(Point::new(0.0, 1.0), Point::new(1.0, 1.0), LineColor::Blue2);
    model.add_line(
        Point::new(0.0, 2.0),
        Point::new(1.0, 2.0),
        LineColor::Black0,
    );

    let changed = replace_line_type_for_indices(
        &mut model,
        &[0, 1, 2],
        CustomLineType::MountainAndValley,
        CustomLineType::Aux,
    );

    assert_eq!(changed, 2);
    assert_eq!(model.line_segments[0].color, LineColor::Cyan3);
    assert_eq!(model.line_segments[1].color, LineColor::Cyan3);
    assert_eq!(model.line_segments[2].color, LineColor::Black0);
}

#[test]
fn replace_selected_line_type_uses_selected_flags() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(1.0, 0.0), LineColor::Red1);
    model.add_line(Point::new(0.0, 1.0), Point::new(1.0, 1.0), LineColor::Blue2);
    model.line_segments[0] = model.line_segments[0].with_selected(2);

    let changed =
        replace_selected_line_type(&mut model, CustomLineType::Mountain, CustomLineType::Valley);

    assert_eq!(changed, 1);
    assert_eq!(model.line_segments[0].color, LineColor::Blue2);
    assert_eq!(model.line_segments[1].color, LineColor::Blue2);
}

#[test]
fn delete_line_type_for_indices_removes_matching_lines_by_value_order() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(1.0, 0.0), LineColor::Red1);
    model.add_line(Point::new(0.0, 1.0), Point::new(1.0, 1.0), LineColor::Blue2);
    model.add_line(
        Point::new(0.0, 2.0),
        Point::new(1.0, 2.0),
        LineColor::Black0,
    );

    let deleted =
        delete_line_type_for_indices(&mut model, &[0, 1, 2], CustomLineType::MountainAndValley);

    assert_eq!(deleted, 2);
    assert_eq!(model.line_segments.len(), 1);
    assert_eq!(model.line_segments[0].color, LineColor::Black0);
}

#[test]
fn delete_selected_line_type_uses_selected_flags() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(1.0, 0.0), LineColor::Red1);
    model.add_line(Point::new(0.0, 1.0), Point::new(1.0, 1.0), LineColor::Blue2);
    model.line_segments[0] = model.line_segments[0].with_selected(2);

    let deleted = delete_selected_line_type(&mut model, CustomLineType::MountainAndValley);

    assert_eq!(deleted, 1);
    assert_eq!(model.line_segments.len(), 1);
    assert_eq!(model.line_segments[0].color, LineColor::Blue2);
}

#[test]
fn toggle_mountain_valley_changes_only_red_and_blue() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(1.0, 0.0), LineColor::Red1);
    model.add_line(Point::new(0.0, 1.0), Point::new(1.0, 1.0), LineColor::Blue2);
    model.add_line(
        Point::new(0.0, 2.0),
        Point::new(1.0, 2.0),
        LineColor::Black0,
    );

    let changed = toggle_mountain_valley(&mut model, &[0, 1, 2]);

    assert_eq!(changed, 2);
    assert_eq!(model.line_segments[0].color, LineColor::Blue2);
    assert_eq!(model.line_segments[1].color, LineColor::Red1);
    assert_eq!(model.line_segments[2].color, LineColor::Black0);
}

#[test]
fn advance_line_type_matches_oriedita_click_cycle() {
    let mut model = CreasePatternModel::default();
    model.add_line(
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        LineColor::Black0,
    );

    assert!(advance_line_type(&mut model, 0));
    assert_eq!(model.line_segments[0].color, LineColor::Black0);
    assert_eq!(model.line_segments[0].selected, 2);
    assert!(advance_line_type(&mut model, 0));
    assert_eq!(model.line_segments[0].color, LineColor::Red1);
    assert_eq!(model.line_segments[0].selected, 0);
    assert!(advance_line_type(&mut model, 0));
    assert_eq!(model.line_segments[0].color, LineColor::Blue2);
    assert!(advance_line_type(&mut model, 0));
    assert_eq!(model.line_segments[0].color, LineColor::Black0);
}

#[test]
fn alternate_mountain_valley_along_overlapping_lines_by_distance() {
    let mut model = CreasePatternModel::default();
    model.add_line(
        Point::new(10.0, 0.0),
        Point::new(20.0, 0.0),
        LineColor::Black0,
    );
    model.add_line(
        Point::new(0.0, 0.0),
        Point::new(5.0, 0.0),
        LineColor::Black0,
    );
    let guide =
        LineSegment::with_color(Point::new(0.0, 0.0), Point::new(20.0, 0.0), LineColor::Red1);

    let changed = alternate_mountain_valley_along(&mut model, &guide, LineColor::Red1);

    assert_eq!(changed, 2);
    assert_eq!(model.line_segments[1].color, LineColor::Red1);
    assert_eq!(model.line_segments[0].color, LineColor::Blue2);
}
