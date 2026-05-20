use oristudio_cp::CreasePatternDocument;
use oristudio_cp::geometry::{ActiveState, Circle, LineColor, LineSegment, Point, RgbColor};
use oristudio_cp::io::{dxf, obj, orh};
use oristudio_cp::model::{CreasePatternModel, GridMetadata, GridState};
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn orh_import_matches_oriedita_io_oracle() {
    let Some(oracle) = io_oracle() else {
        eprintln!("skipping Oriedita IO oracle test: ORIEDITA_IO_ORACLE is not set");
        return;
    };
    let input = "\
<タイトル>
タイトル,orh model
<線分集合>
番号,1
色,1
<tpp>0</tpp>
<tpp_color_R>10</tpp_color_R>
<tpp_color_G>20</tpp_color_G>
<tpp_color_B>30</tpp_color_B>
iactive,ACTIVE_BOTH_3
選択,2
座標,0.0,0.0,10.0,0.0
<円集合>
番号,1
中心と半径と色,5.0,5.0,2.0,3
<tpp>1</tpp>
<tpp_color_R>40</tpp_color_R>
<tpp_color_G>50</tpp_color_G>
<tpp_color_B>60</tpp_color_B>
<Kousi>
<i_kitei_jyoutai>2</i_kitei_jyoutai>
<nyuuryoku_kitei>12.6</nyuuryoku_kitei>
<memori_kankaku>6</memori_kankaku>
<a_to_heikouna_memori_iti>4</a_to_heikouna_memori_iti>
<b_to_heikouna_memori_iti>5</b_to_heikouna_memori_iti>
<d_kousi_x_a>2</d_kousi_x_a>
<d_kousi_x_b>1.5</d_kousi_x_b>
<d_kousi_x_c>4</d_kousi_x_c>
<d_kousi_y_a>1</d_kousi_y_a>
<d_kousi_y_b>0</d_kousi_y_b>
<d_kousi_y_c>1</d_kousi_y_c>
<d_kousi_kakudo>45</d_kousi_kakudo>
</Kousi>
";
    let path = write_temp("orh-oracle", ".orh", input.as_bytes());

    let oracle_summary = run_oracle(&oracle, &["orh-import-summary", path.to_str().unwrap()]);
    let document = orh::import_orh_str(input).expect("Rust ORH import should succeed");
    let rust_summary = document_summary(&document, Some(&document.crease_pattern.grid));

    let _ = std::fs::remove_file(path);
    assert_eq!(rust_summary, oracle_summary);
}

#[test]
fn orh_and_dxf_exports_match_oriedita_io_oracle() {
    let Some(oracle) = io_oracle() else {
        eprintln!("skipping Oriedita IO oracle test: ORIEDITA_IO_ORACLE is not set");
        return;
    };

    let document = oracle_fixture_document();
    assert_eq!(
        orh::export_orh_string(&document),
        run_oracle(&oracle, &["orh-export-fixture"])
    );
    assert_eq!(
        dxf::export_dxf_string(&document.crease_pattern),
        run_oracle(&oracle, &["dxf-export-fixture"])
    );
}

#[test]
fn obj_import_matches_oriedita_io_oracle() {
    let Some(oracle) = io_oracle() else {
        eprintln!("skipping Oriedita IO oracle test: ORIEDITA_IO_ORACLE is not set");
        return;
    };
    let input = "\
v 0 0 0
v 10 0 0
v 0 10 0
f 1 2 3
";
    let path = write_temp("obj-oracle", ".obj", input.as_bytes());

    let oracle_summary = run_oracle(&oracle, &["obj-import-summary", path.to_str().unwrap()]);
    let model = obj::import_obj_str(input).expect("Rust OBJ import should succeed");
    let rust_summary = model_summary(None, &model, None);

    let _ = std::fs::remove_file(path);
    assert_eq!(rust_summary, oracle_summary);
}

fn io_oracle() -> Option<PathBuf> {
    std::env::var("ORIEDITA_IO_ORACLE")
        .or_else(|_| std::env::var("ORIEDITA_GEOMETRY_ORACLE"))
        .ok()
        .map(|oracle| resolve_oracle_path(&oracle))
}

fn resolve_oracle_path(oracle: &str) -> PathBuf {
    let path = PathBuf::from(oracle);
    if path.is_absolute() || path.exists() {
        return path;
    }

    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_oracle(oracle: &Path, args: &[&str]) -> String {
    let output = Command::new(oracle)
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("failed to run Oriedita IO oracle {oracle:?}: {err}"));

    assert!(
        output.status.success(),
        "Oriedita IO oracle failed with status {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("oracle stdout should be valid UTF-8")
}

fn write_temp(prefix: &str, extension: &str, bytes: &[u8]) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}{extension}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::write(&path, bytes).expect("write oracle fixture");
    path
}

fn oracle_fixture_document() -> CreasePatternDocument {
    let mut document = CreasePatternDocument {
        title: Some("oracle".to_string()),
        ..CreasePatternDocument::default()
    };
    document.crease_pattern.add_line_segment(
        LineSegment::with_color(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            LineColor::Blue2,
        )
        .with_customized_color(RgbColor::new(1, 2, 3)),
    );
    document
        .crease_pattern
        .add_circle(Circle::new(5.0, 5.0, 2.0, LineColor::Magenta5));
    document
        .crease_pattern
        .add_aux_line_segment(LineSegment::with_color(
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
            LineColor::Orange4,
        ));
    document.crease_pattern.grid.base_state = GridState::Hidden;
    document.crease_pattern.grid.set_grid_size(24);
    document
}

fn document_summary(document: &CreasePatternDocument, grid: Option<&GridMetadata>) -> String {
    model_summary(document.title.as_deref(), &document.crease_pattern, grid)
}

fn model_summary(
    title: Option<&str>,
    model: &CreasePatternModel,
    grid: Option<&GridMetadata>,
) -> String {
    let mut output = String::new();
    output.push_str(&format!("title|{}\n", title.unwrap_or_default()));
    output.push_str(&format!("lines|{}\n", model.line_segments.len()));
    for segment in &model.line_segments {
        push_segment(&mut output, "line", segment);
    }
    output.push_str(&format!("circles|{}\n", model.circles.len()));
    for circle in &model.circles {
        output.push_str(&format!(
            "circle|{}|{}|{}|{}|{}|{}|{}|{}\n",
            java_double_string(circle.x),
            java_double_string(circle.y),
            java_double_string(circle.r),
            circle.color.number(),
            circle.customized,
            circle.customized_color.red,
            circle.customized_color.green,
            circle.customized_color.blue
        ));
    }
    output.push_str(&format!("aux|{}\n", model.aux_line_segments.len()));
    for segment in &model.aux_line_segments {
        push_segment(&mut output, "auxline", segment);
    }
    if let Some(grid) = grid {
        output.push_str(&format!(
            "grid|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
            grid.interval_grid_size,
            grid.grid_size,
            java_double_string(grid.grid_xa),
            java_double_string(grid.grid_xb),
            java_double_string(grid.grid_xc),
            java_double_string(grid.grid_ya),
            java_double_string(grid.grid_yb),
            java_double_string(grid.grid_yc),
            java_double_string(grid.grid_angle),
            grid.base_state.state(),
            grid.vertical_scale_position,
            grid.horizontal_scale_position,
            grid.draw_diagonal_gridlines
        ));
    } else {
        output.push_str("grid|null\n");
    }
    output
}

fn push_segment(output: &mut String, prefix: &str, segment: &LineSegment) {
    output.push_str(&format!(
        "{prefix}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
        java_double_string(segment.a.x),
        java_double_string(segment.a.y),
        java_double_string(segment.b.x),
        java_double_string(segment.b.y),
        segment.color.number(),
        active_state_name(segment.active),
        segment.selected,
        segment.customized,
        segment.customized_color.red,
        segment.customized_color.green,
        segment.customized_color.blue
    ));
}

fn active_state_name(active: ActiveState) -> &'static str {
    match active {
        ActiveState::Inactive0 => "INACTIVE_0",
        ActiveState::ActiveA1 => "ACTIVE_A_1",
        ActiveState::ActiveB2 => "ACTIVE_B_2",
        ActiveState::ActiveBoth3 => "ACTIVE_BOTH_3",
    }
}

fn java_double_string(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}
