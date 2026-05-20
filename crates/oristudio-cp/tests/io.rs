use oristudio_cp::geometry::{Circle, LineColor, LineSegment, Point, RgbColor};
use oristudio_cp::io::{cp, dxf, fold, obj};
use oristudio_cp::model::{CreasePatternModel, GridState, TextElement};

#[test]
fn cp_import_and_export_preserve_oriedita_assignment_numbers() {
    let input = "\
1 200.0 200.0 200.0 -200.0
3 200.0 200.0 0.0 0.0
2 0.0 0.0 -200.0 -200.0
4 1.5 2.25 3.5 4.75
";

    let model = cp::import_cp_str(input).expect("valid cp");
    assert_eq!(model.line_segments.len(), 4);
    assert_eq!(model.line_segments[0].color, LineColor::Black0);
    assert_eq!(model.line_segments[1].color, LineColor::Red1);
    assert_eq!(model.line_segments[2].color, LineColor::Blue2);
    assert_eq!(model.line_segments[3].color, LineColor::Cyan3);

    assert_eq!(cp::export_cp_string(&model), input);
}

#[test]
fn fold_import_reads_edges_and_oriedita_extensions() {
    let input = r##"{
      "file_spec": 1.1,
      "vertices_coords": [[0, 0], [10, 0], [10, 10]],
      "edges_vertices": [[0, 1], [1, 2]],
      "edges_assignment": ["B", "M"],
      "oriedita:edges_colors": ["", "ffff33"],
      "oriedita:circles_coords": [[5, 5]],
      "oriedita:circles_radii": [2],
      "oriedita:circles_colors": ["3"],
      "oriedita:circles_custom_colors": ["64c8c8"],
      "oriedita:texts_coords": [[1, 2]],
      "oriedita:texts_text": ["note"],
      "oriedita:grid_size": 16,
      "oriedita:grid_style": 2
    }"##;

    let model = fold::import_fold_json(input).expect("valid fold");
    assert_eq!(model.line_segments.len(), 2);
    assert_eq!(model.line_segments[0].color, LineColor::Black0);
    assert_eq!(model.line_segments[1].color, LineColor::Red1);
    assert_eq!(model.line_segments[1].customized, 1);
    assert_eq!(
        model.line_segments[1].customized_color,
        RgbColor::new(255, 255, 51)
    );
    assert_eq!(model.circles.len(), 1);
    assert_eq!(model.circles[0].color, LineColor::Cyan3);
    assert_eq!(
        model.circles[0].customized_color,
        RgbColor::new(100, 200, 200)
    );
    assert_eq!(model.texts[0].text, "note");
    assert_eq!(model.grid.grid_size, 16);
    assert_eq!(model.grid.base_state, GridState::Full);
}

#[test]
fn fold_export_round_trips_canonical_model_data() {
    let mut model = CreasePatternModel::default();
    model.add_line_segment(
        LineSegment::from_coordinates(0.0, 0.0, 10.0, 0.0)
            .with_line_color(LineColor::Blue2)
            .with_customized_color(RgbColor::new(1, 2, 3)),
    );
    model.add_line(
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
        LineColor::Cyan3,
    );
    model.add_circle(
        Circle::new(5.0, 5.0, 2.0, LineColor::Magenta5)
            .with_customized_color(RgbColor::new(100, 200, 200)),
    );
    model.add_text(TextElement::new(3.0, 4.0, "hello"));
    model.grid.set_grid_size(12);
    model.grid.base_state = GridState::Hidden;

    let json = fold::export_fold_json(&model, Some("fold".to_string())).expect("serializes");
    let imported = fold::import_fold_json(&json).expect("imports exported fold");

    assert_eq!(model.canonical(1.0e-9), imported.canonical(1.0e-9));
}

#[test]
fn dxf_export_uses_oriedita_layers_and_coordinate_transform() {
    let mut model = CreasePatternModel::default();
    model.add_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1);

    let output = dxf::export_dxf_string(&model);
    assert!(output.contains("MountainLine"));
    assert!(output.contains("\n  62\n1\n"));
    assert!(output.contains("\n  10\n604\n"));
    assert!(output.contains("\n  20\n604\n"));
}

#[test]
fn obj_import_matches_oriedita_face_edge_and_dummy_line_behavior() {
    let input = "\
v 0 0 0
v 10 0 0
v 0 10 0
f 1 2 3
";

    let model = obj::import_obj_str(input).expect("valid obj");
    assert_eq!(model.line_segments.len(), 4);
    assert_eq!(model.line_segments[0].color, LineColor::None);
    assert_eq!(model.line_segments[1].a, Point::new(0.0, 10.0));
    assert_eq!(model.line_segments[1].b, Point::new(0.0, 0.0));
}
