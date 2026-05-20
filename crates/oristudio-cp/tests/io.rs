use oristudio_cp::CreasePatternDocument;
use oristudio_cp::geometry::{Circle, LineColor, LineSegment, Point, RgbColor};
use oristudio_cp::io::{cp, dxf, fold, obj, ori};
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

#[test]
fn ori_import_reads_oriedita_save_json() {
    let input = r##"{
      "@version": "v1.1",
      "lineSegments": [{
        "a": "-200.0,-200.0",
        "b": "200.0,-200.0",
        "active": "ACTIVE_A_1",
        "color": "RED_1",
        "customized": 1,
        "customizedColor": "ff010203",
        "selected": 2
      }],
      "circles": [{
        "x": 25.0,
        "y": -50.0,
        "r": 12.5,
        "color": "CYAN_3",
        "customized": 1,
        "customizedColor": "ff64c8c8"
      }],
      "texts": [{"x": 1.0, "y": 2.0, "text": "note"}],
      "title": "_",
      "points": ["3.0,4.0"],
      "auxLineSegments": [{
        "a": "0.0,0.0",
        "b": "1.0,1.0",
        "active": "ACTIVE_BOTH_3",
        "color": "YELLOW_7",
        "customized": 0,
        "customizedColor": "ff64c8c8",
        "selected": 0
      }],
      "gridModel": {
        "intervalGridSize": 5,
        "gridSize": 16,
        "gridXA": 2.0,
        "gridXB": 1.0,
        "gridXC": 4.0,
        "gridYA": 1.0,
        "gridYB": 0.0,
        "gridYC": 1.0,
        "gridAngle": 45.0,
        "baseState": "FULL",
        "verticalScalePosition": 3,
        "horizontalScalePosition": 2,
        "drawDiagonalGridlines": true
      },
      "canvasModel": {"mouseMode": "DRAW_CREASE_FREE_1"}
    }"##;

    let document = ori::import_ori_json(input).expect("valid ori");
    let model = &document.crease_pattern;

    assert_eq!(document.title.as_deref(), Some("_"));
    assert_eq!(model.line_segments.len(), 1);
    assert_eq!(model.line_segments[0].color, LineColor::Red1);
    assert_eq!(
        model.line_segments[0].active,
        oristudio_cp::geometry::ActiveState::ActiveA1
    );
    assert_eq!(model.line_segments[0].selected, 2);
    assert_eq!(
        model.line_segments[0].customized_color,
        RgbColor::new(1, 2, 3)
    );
    assert_eq!(model.circles[0].color, LineColor::Cyan3);
    assert_eq!(model.texts[0].text, "note");
    assert_eq!(model.points[0], Point::new(3.0, 4.0));
    assert_eq!(model.aux_line_segments[0].color, LineColor::Yellow7);
    assert_eq!(model.grid.interval_grid_size, 5);
    assert_eq!(model.grid.grid_size, 16);
    assert_eq!(model.grid.grid_angle, 45.0);
    assert_eq!(model.grid.base_state, GridState::Full);
    assert_eq!(
        document.metadata.get("oriedita:ori:canvasModel"),
        Some(&serde_json::json!({"mouseMode": "DRAW_CREASE_FREE_1"}))
    );
}

#[test]
fn ori_export_round_trips_canonical_model_data_and_metadata() {
    let mut document = CreasePatternDocument {
        title: Some("model".to_string()),
        ..CreasePatternDocument::default()
    };
    document.crease_pattern.add_line_segment(
        LineSegment::from_coordinates(0.0, 0.0, 10.0, 0.0)
            .with_line_color(LineColor::Blue2)
            .with_customized_color(RgbColor::new(10, 20, 30)),
    );
    document
        .crease_pattern
        .add_aux_line_segment(LineSegment::with_color(
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
            LineColor::Orange4,
        ));
    document
        .crease_pattern
        .add_circle(Circle::new(5.0, 5.0, 2.0, LineColor::Magenta5));
    document
        .crease_pattern
        .add_text(TextElement::new(3.0, 4.0, "hello"));
    document.crease_pattern.add_point(Point::new(-1.0, -2.0));
    document.crease_pattern.grid.base_state = GridState::Hidden;
    document.metadata.insert(
        "oriedita:ori:applicationModel".to_string(),
        serde_json::Value::Null,
    );

    let json = ori::export_ori_json(&document).expect("serializes ori");
    let exported: serde_json::Value = serde_json::from_str(&json).expect("json");

    assert_eq!(exported["@version"], "v1.1");
    assert_eq!(exported["lineSegments"][0]["color"], "BLUE_2");
    assert_eq!(exported["lineSegments"][0]["customizedColor"], "ff0a141e");
    assert!(exported.get("applicationModel").is_some());

    let imported = ori::import_ori_json(&json).expect("imports exported ori");
    assert_eq!(document.canonical(1.0e-9), imported.canonical(1.0e-9));
    assert_eq!(
        imported.metadata.get("oriedita:ori:applicationModel"),
        Some(&serde_json::Value::Null)
    );
}

#[test]
fn ori_import_has_explicit_unknown_version_policy() {
    let input = r#"{"@version":"v99","lineSegments":[]}"#;

    assert!(ori::import_ori_json(input).is_err());
    assert!(ori::import_ori_json_with_unknown_version(input, true).is_ok());
}
