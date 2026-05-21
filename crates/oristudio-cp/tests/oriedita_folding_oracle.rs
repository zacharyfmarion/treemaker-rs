use oristudio_cp::folding::{
    EquivalenceConditionSet, InitialHierarchy, InitialHierarchyError, SubFaceConfiguration,
    configure_subfaces_from_segments, equivalence_condition_candidates_from_segments,
    estimate_wireframe_from_segments, initial_hierarchy_from_segments, prepare_subface_segments,
};
use oristudio_cp::geometry::{LineColor, LineSegment, Point};
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn wireframe_folding_matches_oriedita_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    for starting_face in [1, 0] {
        let segments = square_with_diagonal();
        let folded = estimate_wireframe_from_segments(&segments, starting_face)
            .expect("Rust wireframe folding should succeed");
        let mut args = vec![
            "wireframe-folding-summary".to_string(),
            starting_face.to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);
        let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

        assert_eq!(
            wireframe_summary(&folded),
            run_oracle(&oracle, &oracle_args)
        );
    }
}

#[test]
fn subface_arrangement_preparation_matches_oriedita_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    for segments in [
        vec![
            segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
            segment(5.0, -5.0, 5.0, 5.0, LineColor::Blue2),
            segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
            segment(2.0, 2.0, 2.0, 2.0, LineColor::Black0),
        ],
        vec![
            segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
            segment(5.0, 0.0, 5.0, 5.0, LineColor::Blue2),
            segment(10.0, 0.0, 0.0, 0.0, LineColor::Red1),
        ],
        vec![
            segment(0.0, 0.0, 1.0, 0.0, LineColor::Black0),
            segment(0.0, 0.00005, 1.0, 0.00005, LineColor::Black0),
        ],
    ] {
        let prepared = prepare_subface_segments(&segments);
        let mut args = vec![
            "split-subface-arrangement".to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);
        let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

        assert_eq!(
            line_segment_summary(&prepared),
            run_oracle(&oracle, &oracle_args)
        );
    }
}

#[test]
fn subface_configuration_matches_oriedita_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    for starting_face in [1, 0] {
        let segments = square_with_diagonal();
        let configuration = configure_subfaces_from_segments(&segments, starting_face)
            .expect("Rust subface configuration should succeed");
        let mut args = vec![
            "subface-configuration-summary".to_string(),
            starting_face.to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);
        let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

        assert_eq!(
            subface_configuration_summary(&configuration),
            run_oracle(&oracle, &oracle_args)
        );
    }
}

#[test]
fn initial_hierarchy_matches_oriedita_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    for (starting_face, segments) in [
        (1, square_with_diagonal()),
        (1, square_with_blue_diagonal()),
        (0, square_with_diagonal()),
    ] {
        let hierarchy = initial_hierarchy_from_segments(&segments, starting_face)
            .expect("Rust initial hierarchy should not have a parity error")
            .expect("Rust initial hierarchy should succeed");
        let mut args = vec![
            "initial-hierarchy-summary".to_string(),
            starting_face.to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);
        let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

        assert_eq!(
            initial_hierarchy_summary(Ok(&hierarchy)),
            run_oracle(&oracle, &oracle_args)
        );
    }
}

#[test]
fn equivalence_condition_candidates_match_oriedita_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    for (starting_face, segments) in [(1, quartered_square()), (0, quartered_square())] {
        let conditions = equivalence_condition_candidates_from_segments(&segments, starting_face)
            .expect("Rust equivalence condition generation should not have a parity error")
            .expect("Rust equivalence condition generation should succeed");
        let mut args = vec![
            "equivalence-candidates-summary".to_string(),
            starting_face.to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);
        let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

        assert_eq!(
            equivalence_condition_summary(&conditions),
            run_oracle(&oracle, &oracle_args)
        );
    }
}

fn folding_oracle() -> Option<PathBuf> {
    std::env::var("ORIEDITA_GEOMETRY_ORACLE")
        .or_else(|_| std::env::var("ORIEDITA_ORACLE"))
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
        .unwrap_or_else(|err| panic!("failed to run Oriedita folding oracle {oracle:?}: {err}"));

    assert!(
        output.status.success(),
        "Oriedita folding oracle failed with status {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("oracle stdout should be valid UTF-8")
}

fn wireframe_summary(folded: &oristudio_cp::folding::FoldedWireframe) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "wireframe|{}|{}|{}|{}\n",
        folded.points.len(),
        folded.lines.len(),
        folded.faces.len(),
        folded.starting_face
    ));
    for (index, point) in folded.points.iter().enumerate() {
        output.push_str(&format!(
            "vertex|{}|{}|{}\n",
            index,
            java_double_string(point.x),
            java_double_string(point.y)
        ));
    }
    for (index, line) in folded.lines.iter().enumerate() {
        output.push_str(&format!(
            "edge|{}|{}|{}|{}\n",
            index,
            line.begin,
            line.end,
            line.color.number()
        ));
    }
    for (index, face) in folded.faces.iter().enumerate() {
        let points = face
            .iter()
            .map(|point| point.to_string())
            .collect::<Vec<_>>()
            .join(",");
        output.push_str(&format!("face|{}|{}\n", index, points));
        output.push_str(&format!(
            "face_position|{}|{}\n",
            index, folded.face_positions[index]
        ));
    }
    output
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

fn line_segment_summary(segments: &[LineSegment]) -> String {
    let mut output = String::new();
    output.push_str(&format!("lines|{}\n", segments.len()));
    for segment in segments {
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

fn subface_configuration_summary(configuration: &SubFaceConfiguration) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "subfaces|{}|{}\n",
        configuration.subfaces.len(),
        configuration.face_id_count_max
    ));
    for (index, subface) in configuration.subfaces.iter().enumerate() {
        output.push_str(&format!(
            "subface|{}|{}\n",
            index,
            joined_ids(&subface.face_ids)
        ));
    }
    output.push_str(&format!(
        "reduced|{}\n",
        configuration.reduced_subface_indices.len()
    ));
    for (index, subface_index) in configuration.reduced_subface_indices.iter().enumerate() {
        output.push_str(&format!(
            "reduced_subface|{}|{}|{}\n",
            index,
            subface_index,
            joined_ids(&configuration.subfaces[*subface_index].face_ids)
        ));
    }
    output
}

fn initial_hierarchy_summary(
    hierarchy: Result<&InitialHierarchy, InitialHierarchyError>,
) -> String {
    match hierarchy {
        Ok(hierarchy) => {
            let mut output = String::new();
            output.push_str(&format!(
                "hierarchy|{}|{}\n",
                hierarchy.faces_total,
                hierarchy.relations.len()
            ));
            for relation in &hierarchy.relations {
                output.push_str(&format!(
                    "relation|{}|{}\n",
                    relation.upper_face, relation.lower_face
                ));
            }
            output
        }
        Err(InitialHierarchyError::SameParityAdjacentFaces {
            line,
            first_face,
            second_face,
        }) => format!("hierarchy_error|same_parity|{line}|{first_face}|{second_face}\n"),
    }
}

fn equivalence_condition_summary(conditions: &EquivalenceConditionSet) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "equivalence|{}|{}\n",
        conditions.triple_conditions.len(),
        conditions.quadruple_conditions.len()
    ));
    for condition in &conditions.triple_conditions {
        output.push_str(&format!(
            "triple|{}|{}|{}|{}\n",
            condition.a, condition.b, condition.c, condition.d
        ));
    }
    for condition in &conditions.quadruple_conditions {
        output.push_str(&format!(
            "quad|{}|{}|{}|{}\n",
            condition.a, condition.b, condition.c, condition.d
        ));
    }
    output
}

fn joined_ids(ids: &[usize]) -> String {
    ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn segment(ax: f64, ay: f64, bx: f64, by: f64, color: LineColor) -> LineSegment {
    LineSegment::with_color(Point::new(ax, ay), Point::new(bx, by), color)
}

fn square_with_diagonal() -> Vec<LineSegment> {
    vec![
        LineSegment::with_color(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            LineColor::Black0,
        ),
        LineSegment::with_color(
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            LineColor::Black0,
        ),
        LineSegment::with_color(
            Point::new(1.0, 1.0),
            Point::new(0.0, 1.0),
            LineColor::Black0,
        ),
        LineSegment::with_color(
            Point::new(0.0, 1.0),
            Point::new(0.0, 0.0),
            LineColor::Black0,
        ),
        LineSegment::with_color(Point::new(0.0, 0.0), Point::new(1.0, 1.0), LineColor::Red1),
    ]
}

fn square_with_blue_diagonal() -> Vec<LineSegment> {
    let mut segments = square_with_diagonal();
    if let Some(diagonal) = segments.last_mut() {
        *diagonal = diagonal.with_line_color(LineColor::Blue2);
    }
    segments
}

fn quartered_square() -> Vec<LineSegment> {
    vec![
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Black0),
        segment(1.0, 0.0, 1.0, 1.0, LineColor::Black0),
        segment(1.0, 1.0, 0.0, 1.0, LineColor::Black0),
        segment(0.0, 1.0, 0.0, 0.0, LineColor::Black0),
        segment(0.5, 0.5, 0.0, 0.0, LineColor::Red1),
        segment(0.5, 0.5, 1.0, 0.0, LineColor::Blue2),
        segment(0.5, 0.5, 1.0, 1.0, LineColor::Red1),
        segment(0.5, 0.5, 0.0, 1.0, LineColor::Blue2),
    ]
}

fn java_double_string(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}
