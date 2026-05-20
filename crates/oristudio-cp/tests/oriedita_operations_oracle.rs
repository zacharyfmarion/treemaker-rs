use oristudio_cp::geometry::{LineColor, LineSegment, Point};
use oristudio_cp::model::CreasePatternModel;
use oristudio_cp::operations::arrangement::{divide_intersections, intersect_divide_pair};
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn intersect_divide_pair_matches_oriedita_operations_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    for (i, j, segments) in [
        (
            0,
            1,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(5.0, -5.0, 5.0, 5.0, LineColor::Blue2),
            ],
        ),
        (
            0,
            1,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(5.0, 0.0, 5.0, 5.0, LineColor::Blue2),
            ],
        ),
        (
            0,
            1,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(5.0, 0.0, 15.0, 0.0, LineColor::Blue2),
            ],
        ),
        (
            1,
            0,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(5.0, 0.0, 15.0, 0.0, LineColor::Blue2),
            ],
        ),
    ] {
        let mut model = model_from_segments(&segments);
        let added = intersect_divide_pair(&mut model, i, j);

        let mut args = vec![
            "intersect-divide-pair".to_string(),
            i.to_string(),
            j.to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);

        let rust_summary = format!("added|{added}\n{}", line_segment_set_summary(&model));
        assert_eq!(rust_summary, run_oracle(&oracle, &args));
    }
}

#[test]
fn divide_intersections_matches_oriedita_operations_oracle_for_crossing_fixture() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let segments = vec![
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
        segment(5.0, -5.0, 5.0, 5.0, LineColor::Blue2),
    ];
    let mut model = model_from_segments(&segments);
    divide_intersections(&mut model);

    let mut args = vec!["intersect-divide".to_string(), segments.len().to_string()];
    push_segment_args(&mut args, &segments);

    assert_eq!(line_segment_set_summary(&model), run_oracle(&oracle, &args));
}

fn operations_oracle() -> Option<PathBuf> {
    std::env::var("ORIEDITA_OPERATIONS_ORACLE")
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

fn run_oracle(oracle: &Path, args: &[String]) -> String {
    let output = Command::new(oracle)
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("failed to run Oriedita operations oracle {oracle:?}: {err}"));

    assert!(
        output.status.success(),
        "Oriedita operations oracle failed with status {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("oracle stdout should be valid UTF-8")
}

fn segment(ax: f64, ay: f64, bx: f64, by: f64, color: LineColor) -> LineSegment {
    LineSegment::with_color(Point::new(ax, ay), Point::new(bx, by), color)
}

fn model_from_segments(segments: &[LineSegment]) -> CreasePatternModel {
    let mut model = CreasePatternModel::default();
    for segment in segments {
        model.add_line_segment(segment.clone());
    }
    model
}

fn push_segment_args(args: &mut Vec<String>, segments: &[LineSegment]) {
    for segment in segments {
        args.push(segment.a.x.to_string());
        args.push(segment.a.y.to_string());
        args.push(segment.b.x.to_string());
        args.push(segment.b.y.to_string());
        args.push(segment.color.number().to_string());
    }
}

fn line_segment_set_summary(model: &CreasePatternModel) -> String {
    let mut output = String::new();
    output.push_str(&format!("lines|{}\n", model.line_segments.len()));
    for segment in &model.line_segments {
        output.push_str(&format!(
            "line|{}|{}|{}|{}|{}\n",
            java_double_string(segment.a.x),
            java_double_string(segment.a.y),
            java_double_string(segment.b.x),
            java_double_string(segment.b.y),
            segment.color.number()
        ));
    }
    output
}

fn java_double_string(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}
