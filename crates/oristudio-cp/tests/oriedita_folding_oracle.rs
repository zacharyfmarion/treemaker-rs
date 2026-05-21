use oristudio_cp::folding::{
    AdditionalEstimation, AdditionalEstimationError, ChainPermutationGenerator,
    EquivalenceConditionSet, InitialHierarchy, InitialHierarchyError, SubFaceConfiguration,
    additional_estimation_from_segments, configure_subfaces_from_segments,
    equivalence_condition_candidates_from_segments, estimate_wireframe_from_segments,
    initial_hierarchy_from_segments, prepare_subface_segments,
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

#[test]
fn additional_estimation_matches_oriedita_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    for starting_face in [1, 0] {
        let segments = square_with_diagonal();
        let estimation = additional_estimation_from_segments(&segments, starting_face)
            .expect("Rust additional estimation should not have an initial hierarchy error")
            .expect("Rust additional estimation should succeed");
        let mut args = vec![
            "additional-estimation-summary".to_string(),
            starting_face.to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);
        let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

        assert_eq!(
            additional_estimation_summary(Ok(&estimation)),
            run_oracle(&oracle, &oracle_args)
        );
    }
}

#[test]
fn chain_permutation_generator_matches_oriedita_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    for case in [
        PermutationCase {
            digits: 3,
            guides: &[],
            top: &[],
            bottom: &[],
            limit: 8,
        },
        PermutationCase {
            digits: 4,
            guides: &[(1, 2), (2, 3)],
            top: &[],
            bottom: &[],
            limit: 8,
        },
        PermutationCase {
            digits: 4,
            guides: &[(1, 3)],
            top: &[2, 4],
            bottom: &[1, 3],
            limit: 8,
        },
    ] {
        let mut generator = ChainPermutationGenerator::new(case.digits);
        for (upper, lower) in case.guides {
            generator.add_guide(*upper, *lower).expect("valid guide");
        }
        generator
            .set_top_indices(case.top.iter().copied())
            .expect("valid top indices");
        generator
            .set_bottom_indices(case.bottom.iter().copied())
            .expect("valid bottom indices");
        generator.initialize();

        let mut args = vec![
            "chain-permutation-summary".to_string(),
            case.digits.to_string(),
            case.guides.len().to_string(),
        ];
        for (upper, lower) in case.guides {
            args.push(upper.to_string());
            args.push(lower.to_string());
        }
        args.push(joined_or_dash(case.top));
        args.push(joined_or_dash(case.bottom));
        args.push(case.limit.to_string());
        let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

        assert_eq!(
            chain_permutation_summary(generator, case.limit),
            run_oracle(&oracle, &oracle_args)
        );
    }
}

#[test]
fn chain_permutation_temp_guides_match_oriedita_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    let mut generator = ChainPermutationGenerator::new(4);
    generator.add_guide(1, 3).expect("valid guide");
    generator.initialize();

    let args = [
        "chain-permutation-temp-summary",
        "4",
        "1",
        "1",
        "3",
        "-",
        "-",
        "2",
        "2",
        "1",
        "2",
        "4",
    ];

    assert_eq!(
        chain_permutation_temp_summary(generator, 2, 2, 1, 2, 4),
        run_oracle(&oracle, &args)
    );
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

fn additional_estimation_summary(
    estimation: Result<&AdditionalEstimation, AdditionalEstimationError>,
) -> String {
    match estimation {
        Ok(estimation) => {
            let mut output = String::new();
            output.push_str(&format!(
                "additional|{}|{}\n",
                estimation.hierarchy.faces_total,
                estimation.hierarchy.relations.len()
            ));
            for relation in &estimation.hierarchy.relations {
                output.push_str(&format!(
                    "relation|{}|{}\n",
                    relation.upper_face, relation.lower_face
                ));
            }
            output
        }
        Err(error) => format!("additional_error|{error:?}\n"),
    }
}

struct PermutationCase<'a> {
    digits: usize,
    guides: &'a [(usize, usize)],
    top: &'a [usize],
    bottom: &'a [usize],
    limit: usize,
}

fn chain_permutation_summary(mut generator: ChainPermutationGenerator, limit: usize) -> String {
    let mut output = String::new();
    output.push_str(&format!("permutations|{}\n", generator.count()));
    if limit == 0 {
        return output;
    }
    push_chain_permutation(&mut output, 0, 0, &generator);
    for step in 1..limit {
        let changed = generator
            .next(generator.num_digits())
            .expect("valid generator advance");
        if changed == 0 {
            output.push_str(&format!("end|{}|0|{}\n", step, generator.count()));
            return output;
        }
        push_chain_permutation(&mut output, step, changed, &generator);
    }
    output
}

fn chain_permutation_temp_summary(
    mut generator: ChainPermutationGenerator,
    steps_before_temp: usize,
    temp_upper: usize,
    temp_lower: usize,
    steps_after_temp: usize,
    limit_after_clear: usize,
) -> String {
    let mut output = String::new();
    output.push_str(&format!("permutations|{}\n", generator.count()));
    push_chain_permutation(&mut output, 0, 0, &generator);
    for step in 1..=steps_before_temp {
        let changed = generator
            .next(generator.num_digits())
            .expect("valid generator advance");
        if changed == 0 {
            output.push_str(&format!("end|{}|0|{}\n", step, generator.count()));
            return output;
        }
        push_chain_permutation(&mut output, step, changed, &generator);
    }

    generator
        .add_guide(temp_upper, temp_lower)
        .expect("valid temp guide");
    output.push_str(&format!("temp|{}|{}\n", temp_upper, temp_lower));
    for step in 1..=steps_after_temp {
        let changed = generator
            .next(generator.num_digits())
            .expect("valid generator advance");
        if changed == 0 {
            output.push_str(&format!("end_temp|{}|0|{}\n", step, generator.count()));
            return output;
        }
        push_chain_permutation(&mut output, step, changed, &generator);
    }

    generator.clear_temp_guide();
    output.push_str("clear_temp\n");
    for step in 1..=limit_after_clear {
        let changed = generator
            .next(generator.num_digits())
            .expect("valid generator advance");
        if changed == 0 {
            output.push_str(&format!("end_clear|{}|0|{}\n", step, generator.count()));
            return output;
        }
        push_chain_permutation(&mut output, step, changed, &generator);
    }
    output
}

fn push_chain_permutation(
    output: &mut String,
    step: usize,
    changed: usize,
    generator: &ChainPermutationGenerator,
) {
    output.push_str(&format!(
        "permutation|{}|{}|{}|{}\n",
        step,
        changed,
        generator.count(),
        joined_ids(&generator.current_permutation())
    ));
}

fn joined_ids(ids: &[usize]) -> String {
    ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn joined_or_dash(ids: &[usize]) -> String {
    if ids.is_empty() {
        "-".to_string()
    } else {
        joined_ids(ids)
    }
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
