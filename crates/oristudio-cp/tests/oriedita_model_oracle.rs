use oristudio_cp::model::CustomLineType;
use std::path::{Path, PathBuf};
use std::process::Command;

struct CustomLineTypeCase {
    custom_type: CustomLineType,
    line_color_number: i32,
}

#[test]
fn custom_line_types_match_oriedita_model_oracle() {
    let Ok(oracle) = std::env::var("ORIEDITA_MODEL_ORACLE")
        .or_else(|_| std::env::var("ORIEDITA_GEOMETRY_ORACLE"))
    else {
        eprintln!("skipping Oriedita model oracle test: ORIEDITA_MODEL_ORACLE is not set");
        return;
    };
    let oracle = resolve_oracle_path(&oracle);

    let cases = [
        CustomLineTypeCase {
            custom_type: CustomLineType::Any,
            line_color_number: 8,
        },
        CustomLineTypeCase {
            custom_type: CustomLineType::Edge,
            line_color_number: 0,
        },
        CustomLineTypeCase {
            custom_type: CustomLineType::MountainAndValley,
            line_color_number: 2,
        },
        CustomLineTypeCase {
            custom_type: CustomLineType::Mountain,
            line_color_number: 1,
        },
        CustomLineTypeCase {
            custom_type: CustomLineType::Valley,
            line_color_number: 2,
        },
        CustomLineTypeCase {
            custom_type: CustomLineType::Aux,
            line_color_number: 3,
        },
    ];

    for case in cases {
        let output = run_oracle(&oracle, &case);
        let expected = format!(
            "{},{},{},{}",
            case.custom_type.number(),
            case.custom_type.number_for_line_color(),
            case.custom_type.line_color().number(),
            case.custom_type.matches(
                oristudio_cp::geometry::LineColor::from_number(case.line_color_number)
                    .expect("test line color exists")
            )
        );
        assert_eq!(output, expected);
    }
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

fn run_oracle(oracle: &Path, case: &CustomLineTypeCase) -> String {
    let output = Command::new(oracle)
        .arg("custom-line-type")
        .arg(case.custom_type.number().to_string())
        .arg(case.line_color_number.to_string())
        .output()
        .unwrap_or_else(|err| panic!("failed to run Oriedita model oracle {oracle:?}: {err}"));

    assert!(
        output.status.success(),
        "Oriedita model oracle failed with status {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout)
        .expect("oracle stdout should be valid UTF-8")
        .trim()
        .to_owned()
}
