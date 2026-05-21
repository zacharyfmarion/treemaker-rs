use oristudio_cp::folding::{
    AdditionalEstimation, AdditionalEstimationError, ChainPermutationGenerator,
    EquivalenceConditionSet, HierarchyRelation, InitialHierarchy, InitialHierarchyError, SubFace,
    SubFaceConfiguration, SubFacePermutationSearch, additional_estimation_from_segments,
    configure_subfaces_from_segments, equivalence_condition_candidates_from_segments,
    estimate_wireframe_from_segments, initial_hierarchy_from_segments,
    overlap_search_from_segments, possible_overlap_search_for_subfaces, prepare_subface_segments,
    prioritize_subfaces,
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

#[test]
fn subface_guide_permutations_match_oriedita_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    for case in [
        SubFaceGuideCase {
            faces_total: 4,
            face_ids: &[0, 1, 2, 3],
            relations: &[(0, 1), (1, 2), (0, 2)],
            limit: 10,
        },
        SubFaceGuideCase {
            faces_total: 5,
            face_ids: &[1, 2, 4],
            relations: &[(1, 4), (2, 4)],
            limit: 8,
        },
    ] {
        let hierarchy = InitialHierarchy {
            faces_total: case.faces_total,
            relations: case
                .relations
                .iter()
                .map(|(upper_face, lower_face)| HierarchyRelation {
                    upper_face: *upper_face,
                    lower_face: *lower_face,
                })
                .collect(),
        };
        let mut search = SubFacePermutationSearch::new(case.face_ids.to_vec());
        search.set_guide_map(&hierarchy, None).expect("guide map");

        let mut args = vec![
            "subface-guide-permutation-summary".to_string(),
            case.faces_total.to_string(),
            case.face_ids.len().to_string(),
        ];
        for face_id in case.face_ids {
            args.push(face_id.to_string());
        }
        args.push(case.relations.len().to_string());
        for (upper_face, lower_face) in case.relations {
            args.push(upper_face.to_string());
            args.push(lower_face.to_string());
        }
        args.push(case.limit.to_string());
        let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

        assert_eq!(
            subface_guide_summary(search, case.limit),
            run_oracle(&oracle, &oracle_args)
        );
    }
}

#[test]
fn subface_overlap_search_matches_oriedita_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    for case in [
        SubFaceOverlapCase {
            faces_total: 3,
            face_ids: &[0, 1, 2],
            relations: &[(2, 0)],
            triple_conditions: &[],
            quadruple_conditions: &[],
        },
        SubFaceOverlapCase {
            faces_total: 3,
            face_ids: &[0, 1, 2],
            relations: &[(1, 0)],
            triple_conditions: &[(0, 1, 0, 2)],
            quadruple_conditions: &[],
        },
    ] {
        let hierarchy = InitialHierarchy {
            faces_total: case.faces_total,
            relations: case
                .relations
                .iter()
                .map(|(upper_face, lower_face)| HierarchyRelation {
                    upper_face: *upper_face,
                    lower_face: *lower_face,
                })
                .collect(),
        };
        let conditions = EquivalenceConditionSet {
            triple_conditions: case
                .triple_conditions
                .iter()
                .map(|(a, b, c, d)| oristudio_cp::folding::EquivalenceCondition {
                    a: *a,
                    b: *b,
                    c: *c,
                    d: *d,
                })
                .collect(),
            quadruple_conditions: case
                .quadruple_conditions
                .iter()
                .map(|(a, b, c, d)| oristudio_cp::folding::EquivalenceCondition {
                    a: *a,
                    b: *b,
                    c: *c,
                    d: *d,
                })
                .collect(),
        };
        let mut search = SubFacePermutationSearch::new(case.face_ids.to_vec());
        search
            .set_guide_map(&hierarchy, Some(&conditions))
            .expect("guide map");

        let mut args = vec![
            "subface-overlap-search-summary".to_string(),
            case.faces_total.to_string(),
            case.face_ids.len().to_string(),
        ];
        for face_id in case.face_ids {
            args.push(face_id.to_string());
        }
        args.push(case.relations.len().to_string());
        for (upper_face, lower_face) in case.relations {
            args.push(upper_face.to_string());
            args.push(lower_face.to_string());
        }
        args.push(case.triple_conditions.len().to_string());
        for (a, b, c, d) in case.triple_conditions {
            args.push(a.to_string());
            args.push(b.to_string());
            args.push(c.to_string());
            args.push(d.to_string());
        }
        args.push(case.quadruple_conditions.len().to_string());
        for (a, b, c, d) in case.quadruple_conditions {
            args.push(a.to_string());
            args.push(b.to_string());
            args.push(c.to_string());
            args.push(d.to_string());
        }
        let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

        assert_eq!(
            subface_overlap_summary(search, &hierarchy),
            run_oracle(&oracle, &oracle_args)
        );
    }
}

#[test]
fn subface_priority_matches_oriedita_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    for case in [
        SubFacePriorityCase {
            faces_total: 4,
            subfaces: &[&[0usize, 1], &[1usize, 2, 3], &[0usize, 1, 2, 3]],
            relations: &[],
        },
        SubFacePriorityCase {
            faces_total: 5,
            subfaces: &[&[0usize, 2, 4], &[1usize, 2], &[0usize, 1, 4]],
            relations: &[(0, 4)],
        },
    ] {
        let hierarchy = InitialHierarchy {
            faces_total: case.faces_total,
            relations: case
                .relations
                .iter()
                .map(|(upper_face, lower_face)| HierarchyRelation {
                    upper_face: *upper_face,
                    lower_face: *lower_face,
                })
                .collect(),
        };
        let subfaces = case
            .subfaces
            .iter()
            .map(|face_ids| SubFace {
                face_ids: face_ids.to_vec(),
            })
            .collect::<Vec<_>>();
        let reduced = (0..subfaces.len()).collect::<Vec<_>>();
        let priority = prioritize_subfaces(&subfaces, &reduced, &hierarchy);

        let mut args = vec![
            "subface-priority-summary".to_string(),
            case.faces_total.to_string(),
            case.subfaces.len().to_string(),
        ];
        for face_ids in case.subfaces {
            args.push(face_ids.len().to_string());
            for face_id in *face_ids {
                args.push(face_id.to_string());
            }
        }
        args.push(case.relations.len().to_string());
        for (upper_face, lower_face) in case.relations {
            args.push(upper_face.to_string());
            args.push(lower_face.to_string());
        }
        let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

        assert_eq!(
            subface_priority_summary(&priority, &subfaces),
            run_oracle(&oracle, &oracle_args)
        );
    }
}

#[test]
fn worker_overlap_search_matches_oriedita_no_swap_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    let case = WorkerOverlapCase {
        faces_total: 3,
        subfaces: &[&[0usize, 1, 2]],
        relations: &[(2, 0)],
        triple_conditions: &[],
        quadruple_conditions: &[],
    };
    let hierarchy = InitialHierarchy {
        faces_total: case.faces_total,
        relations: case
            .relations
            .iter()
            .map(|(upper_face, lower_face)| HierarchyRelation {
                upper_face: *upper_face,
                lower_face: *lower_face,
            })
            .collect(),
    };
    let conditions = EquivalenceConditionSet {
        triple_conditions: Vec::new(),
        quadruple_conditions: Vec::new(),
    };
    let subfaces = case
        .subfaces
        .iter()
        .map(|face_ids| SubFace {
            face_ids: face_ids.to_vec(),
        })
        .collect::<Vec<_>>();
    let reduced = (0..subfaces.len()).collect::<Vec<_>>();
    let search =
        possible_overlap_search_for_subfaces(&subfaces, &reduced, &hierarchy, Some(&conditions))
            .expect("worker overlap search");

    let mut args = vec![
        "worker-overlap-search-summary".to_string(),
        case.faces_total.to_string(),
        case.subfaces.len().to_string(),
    ];
    for face_ids in case.subfaces {
        args.push(face_ids.len().to_string());
        for face_id in *face_ids {
            args.push(face_id.to_string());
        }
    }
    args.push(case.relations.len().to_string());
    for (upper_face, lower_face) in case.relations {
        args.push(upper_face.to_string());
        args.push(lower_face.to_string());
    }
    args.push(case.triple_conditions.len().to_string());
    args.push(case.quadruple_conditions.len().to_string());
    let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

    assert_eq!(
        worker_overlap_summary(&search),
        run_oracle(&oracle, &oracle_args)
    );
}

#[test]
fn worker_overlap_from_segments_matches_oriedita_no_swap_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    for starting_face in [1, 0] {
        let segments = square_with_diagonal();
        let search = overlap_search_from_segments(&segments, starting_face)
            .expect("worker overlap search should not fail")
            .expect("worker overlap search");
        let mut args = vec![
            "worker-overlap-from-segments-summary".to_string(),
            starting_face.to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);
        let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

        assert_eq!(
            worker_overlap_summary(&search),
            run_oracle(&oracle, &oracle_args)
        );
    }
}

#[test]
fn subface_swapper_matches_oriedita_oracle() {
    let Some(oracle) = folding_oracle() else {
        eprintln!("skipping Oriedita folding oracle test: ORIEDITA_GEOMETRY_ORACLE is not set");
        return;
    };

    let counters = [0usize, 0, 2, 0];
    let actions = [
        SwapperAction::Visit(0),
        SwapperAction::Record(3),
        SwapperAction::Process(4),
        SwapperAction::Estimate(1),
        SwapperAction::Record(2),
        SwapperAction::Process(4),
    ];
    let mut args = vec![
        "subface-swapper-summary".to_string(),
        counters.len().to_string(),
    ];
    for counter in counters {
        args.push(counter.to_string());
    }
    args.push(actions.len().to_string());
    for action in actions {
        match action {
            SwapperAction::Visit(index) => {
                args.push("visit".to_string());
                args.push(index.to_string());
            }
            SwapperAction::Record(index) => {
                args.push("record".to_string());
                args.push(index.to_string());
            }
            SwapperAction::Process(max) => {
                args.push("process".to_string());
                args.push(max.to_string());
            }
            SwapperAction::Estimate(index) => {
                args.push("estimate".to_string());
                args.push(index.to_string());
            }
        }
    }
    let oracle_args = args.iter().map(String::as_str).collect::<Vec<_>>();

    assert_eq!(
        subface_swapper_summary(&[0, 0, 2, 0], &actions),
        run_oracle(&oracle, &oracle_args)
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

struct SubFaceGuideCase<'a> {
    faces_total: usize,
    face_ids: &'a [usize],
    relations: &'a [(usize, usize)],
    limit: usize,
}

struct SubFaceOverlapCase<'a> {
    faces_total: usize,
    face_ids: &'a [usize],
    relations: &'a [(usize, usize)],
    triple_conditions: &'a [(usize, usize, usize, usize)],
    quadruple_conditions: &'a [(usize, usize, usize, usize)],
}

struct SubFacePriorityCase<'a> {
    faces_total: usize,
    subfaces: &'a [&'a [usize]],
    relations: &'a [(usize, usize)],
}

struct WorkerOverlapCase<'a> {
    faces_total: usize,
    subfaces: &'a [&'a [usize]],
    relations: &'a [(usize, usize)],
    triple_conditions: &'a [(usize, usize, usize, usize)],
    quadruple_conditions: &'a [(usize, usize, usize, usize)],
}

#[derive(Clone, Copy)]
enum SwapperAction {
    Visit(usize),
    Record(usize),
    Process(usize),
    Estimate(usize),
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

fn subface_guide_summary(mut search: SubFacePermutationSearch, limit: usize) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "subface_permutations|{}\n",
        search.permutation_count()
    ));
    if limit == 0 {
        return output;
    }
    push_subface_permutation(&mut output, 0, 0, &search);
    for step in 1..limit {
        let changed = search.next(search.face_ids().len()).expect("valid advance");
        if changed == 0 {
            output.push_str(&format!("end|{}|0|{}\n", step, search.permutation_count()));
            return output;
        }
        push_subface_permutation(&mut output, step, changed, &search);
    }
    output
}

fn push_subface_permutation(
    output: &mut String,
    step: usize,
    changed: usize,
    search: &SubFacePermutationSearch,
) {
    output.push_str(&format!(
        "subface_permutation|{}|{}|{}|{}\n",
        step,
        changed,
        search.permutation_count(),
        joined_ids(&search.current_ordering())
    ));
}

fn subface_overlap_summary(
    mut search: SubFacePermutationSearch,
    hierarchy: &InitialHierarchy,
) -> String {
    match search.possible_overlapping_search(hierarchy) {
        Ok(found) => format!(
            "subface_overlap|{}|{}|{}\n",
            if found { 1000 } else { 0 },
            search.permutation_count(),
            joined_ids(&search.current_ordering())
        ),
        Err(error) => format!("subface_overlap_error|{error:?}\n"),
    }
}

fn subface_priority_summary(
    priority: &oristudio_cp::folding::SubFacePriority,
    subfaces: &[SubFace],
) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "subface_priority|{}|{}\n",
        priority.valid_count,
        priority.ordered_subface_indices.len()
    ));
    for (rank, subface_index) in priority.ordered_subface_indices.iter().enumerate() {
        output.push_str(&format!(
            "priority_subface|{}|{}|{}\n",
            rank,
            subface_index,
            joined_ids(&subfaces[*subface_index].face_ids)
        ));
    }
    output
}

fn worker_overlap_summary(search: &oristudio_cp::folding::WorkerOverlapSearch) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "worker_overlap|{}|{}|{}|{}\n",
        if search.found { 1000 } else { 0 },
        search.priority.valid_count,
        search.priority.ordered_subface_indices.len(),
        search.hierarchy.relations.len()
    ));
    for relation in &search.hierarchy.relations {
        output.push_str(&format!(
            "relation|{}|{}\n",
            relation.upper_face, relation.lower_face
        ));
    }
    output
}

fn subface_swapper_summary(counters: &[usize], actions: &[SwapperAction]) -> String {
    let mut swapper = oristudio_cp::folding::SubFaceSwapper::new();
    let mut order = (0..counters.len()).collect::<Vec<_>>();
    let mut output = String::new();
    push_swapper_summary(&mut output, "initial", &swapper, &order);
    for action in actions {
        match *action {
            SwapperAction::Visit(index) => {
                if let Some(item) = order.get(index).copied() {
                    swapper.visit(item);
                }
                push_swapper_summary(&mut output, &format!("visit|{index}"), &swapper, &order);
            }
            SwapperAction::Record(index) => {
                swapper.record(index + 1);
                push_swapper_summary(&mut output, &format!("record|{index}"), &swapper, &order);
            }
            SwapperAction::Process(max) => {
                swapper.process(&mut order, max, counters);
                push_swapper_summary(&mut output, &format!("process|{max}"), &swapper, &order);
            }
            SwapperAction::Estimate(index) => {
                let estimate = swapper.should_estimate(index + 1);
                output.push_str(&format!("estimate|{}|{}\n", index, estimate));
                push_swapper_summary(
                    &mut output,
                    &format!("after_estimate|{index}"),
                    &swapper,
                    &order,
                );
            }
        }
    }
    output
}

fn push_swapper_summary(
    output: &mut String,
    label: &str,
    swapper: &oristudio_cp::folding::SubFaceSwapper,
    order: &[usize],
) {
    output.push_str(&format!(
        "swapper|{}|{}|{}\n",
        label,
        swapper.visited_count(),
        joined_ids(order)
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
