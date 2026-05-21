use oristudio_cp::folding::{
    ChainPermutationGenerator, HierarchyRelation, InitialHierarchy, SubFacePermutationSearch,
    SubFaceSwapper, additional_estimation_from_segments, configure_subfaces_from_segments,
    equivalence_condition_candidates_from_segments, estimate_wireframe_from_segments,
    folding_estimate_from_segments, initial_hierarchy_from_segments, overlap_search_from_segments,
    overlap_search_from_segments_with_swap, possible_overlap_search_for_ordered_subfaces,
    possible_overlap_search_for_subfaces, possible_overlap_search_for_subfaces_with_swap,
    prepare_subface_segments, prioritize_subfaces, two_colored_subface_segments_from_segments,
};
use oristudio_cp::geometry::{LineColor, LineSegment, Point};

#[test]
fn wireframe_fold_builds_faces_and_face_positions() {
    let segments = square_with_diagonal();

    let folded = estimate_wireframe_from_segments(&segments, 1).expect("folded wireframe");

    assert_eq!(folded.points.len(), 4);
    assert_eq!(folded.lines.len(), 5);
    assert_eq!(folded.faces.len(), 2);
    assert_eq!(folded.starting_face, 0);
    assert_eq!(folded.face_positions[0], 1);
    assert!(folded.face_positions.contains(&2));
}

#[test]
fn wireframe_fold_returns_none_without_faces() {
    let segments = vec![LineSegment::with_color(
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        LineColor::Black0,
    )];

    assert!(estimate_wireframe_from_segments(&segments, 1).is_none());
}

#[test]
fn subface_preparation_removes_points_duplicates_and_splits_crossings() {
    let segments = vec![
        LineSegment::with_color(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1),
        LineSegment::with_color(
            Point::new(5.0, -5.0),
            Point::new(5.0, 5.0),
            LineColor::Blue2,
        ),
        LineSegment::with_color(Point::new(0.0, 0.0), Point::new(10.0, 0.0), LineColor::Red1),
        LineSegment::with_color(
            Point::new(2.0, 2.0),
            Point::new(2.0, 2.0),
            LineColor::Black0,
        ),
    ];

    let prepared = prepare_subface_segments(&segments);

    assert_eq!(prepared.len(), 4);
    assert!(prepared.iter().all(|segment| segment.a != segment.b));
    assert_eq!(
        prepared
            .iter()
            .filter(|segment| segment.color == LineColor::Red1)
            .count(),
        2
    );
    assert_eq!(
        prepared
            .iter()
            .filter(|segment| segment.color == LineColor::Blue2)
            .count(),
        2
    );
}

#[test]
fn subface_configuration_maps_subfaces_to_folded_faces() {
    let segments = square_with_diagonal();

    let configuration =
        configure_subfaces_from_segments(&segments, 1).expect("subface configuration");

    assert!(!configuration.subfaces.is_empty());
    assert_eq!(configuration.face_id_count_max, 2);
    assert!(
        configuration
            .subfaces
            .iter()
            .any(|subface| subface.face_ids == vec![0, 1])
    );
    assert!(!configuration.reduced_subface_indices.is_empty());
}

#[test]
fn initial_hierarchy_uses_mountain_valley_and_face_parity() {
    let segments = square_with_diagonal();

    let hierarchy = initial_hierarchy_from_segments(&segments, 1)
        .expect("hierarchy should not fail")
        .expect("hierarchy");

    assert_eq!(hierarchy.faces_total, 2);
    assert_eq!(
        hierarchy.relations,
        vec![HierarchyRelation {
            upper_face: 0,
            lower_face: 1,
        }]
    );
}

#[test]
fn equivalence_condition_candidates_are_exposed() {
    let segments = quartered_square();

    let conditions = equivalence_condition_candidates_from_segments(&segments, 1)
        .expect("condition generation should not fail")
        .expect("condition set");

    assert!(
        !conditions.triple_conditions.is_empty() || !conditions.quadruple_conditions.is_empty()
    );
}

#[test]
fn additional_estimation_produces_hierarchy_relations() {
    let segments = square_with_diagonal();

    let estimation = additional_estimation_from_segments(&segments, 1)
        .expect("additional estimation should not fail")
        .expect("additional estimation");

    assert_eq!(estimation.hierarchy.faces_total, 2);
    assert_eq!(estimation.hierarchy.relations.len(), 1);
}

#[test]
fn chain_permutation_generator_honors_pair_guides() {
    let mut generator = ChainPermutationGenerator::new(4);
    generator.add_guide(1, 2).expect("valid guide");
    generator.add_guide(2, 3).expect("valid guide");
    generator.initialize();

    for permutation in collect_permutations(generator, 16) {
        let one = position(&permutation, 1);
        let two = position(&permutation, 2);
        let three = position(&permutation, 3);
        assert!(one < two);
        assert!(two < three);
    }
}

#[test]
fn chain_permutation_generator_applies_top_and_bottom_constraints() {
    let mut generator = ChainPermutationGenerator::new(4);
    generator.set_top_indices([2, 3]).expect("valid top set");
    generator
        .set_bottom_indices([1, 4])
        .expect("valid bottom set");
    generator.initialize();

    for permutation in collect_permutations(generator, 16) {
        assert!([2, 3].contains(&permutation[0]));
        assert!([1, 4].contains(&permutation[3]));
    }
}

#[test]
fn chain_permutation_generator_supports_temporary_guides() {
    let mut generator = ChainPermutationGenerator::new(3);
    generator.initialize();
    generator.next(3).expect("advance before temp guide");
    generator.add_guide(2, 1).expect("valid temporary guide");
    generator.next(3).expect("advance with temp guide");

    let temp_permutation = generator.current_permutation();
    assert!(position(&temp_permutation, 2) < position(&temp_permutation, 1));

    generator.clear_temp_guide();
    generator
        .next(3)
        .expect("advance after clearing temp guide");
    assert_eq!(generator.current_permutation().len(), 3);
}

#[test]
fn subface_permutation_search_builds_transitive_reduced_guides() {
    let hierarchy = InitialHierarchy {
        faces_total: 4,
        relations: vec![
            HierarchyRelation {
                upper_face: 0,
                lower_face: 1,
            },
            HierarchyRelation {
                upper_face: 1,
                lower_face: 2,
            },
            HierarchyRelation {
                upper_face: 0,
                lower_face: 2,
            },
        ],
    };
    let mut search = SubFacePermutationSearch::new(vec![0, 1, 2, 3]);
    search.set_guide_map(&hierarchy, None).expect("guide map");

    for ordering in collect_subface_orderings(search, 12) {
        assert!(position(&ordering, 0) < position(&ordering, 1));
        assert!(position(&ordering, 1) < position(&ordering, 2));
    }
}

#[test]
fn subface_overlap_search_advances_past_hierarchy_contradictions() {
    let hierarchy = InitialHierarchy {
        faces_total: 3,
        relations: vec![HierarchyRelation {
            upper_face: 2,
            lower_face: 0,
        }],
    };
    let mut search = SubFacePermutationSearch::new(vec![0, 1, 2]);
    search.set_guide_map(&hierarchy, None).expect("guide map");

    assert!(
        search
            .possible_overlapping_search(&hierarchy)
            .expect("subface search should be supported")
    );

    let ordering = search.current_ordering();
    assert!(position(&ordering, 2) < position(&ordering, 0));
}

#[test]
fn subface_priority_prefers_new_pair_information_then_face_count() {
    let hierarchy = InitialHierarchy {
        faces_total: 4,
        relations: Vec::new(),
    };
    let subfaces = vec![
        oristudio_cp::folding::SubFace {
            face_ids: vec![0, 1],
        },
        oristudio_cp::folding::SubFace {
            face_ids: vec![1, 2, 3],
        },
        oristudio_cp::folding::SubFace {
            face_ids: vec![0, 1, 2, 3],
        },
    ];

    let priority = prioritize_subfaces(&subfaces, &[0, 1, 2], &hierarchy);

    assert_eq!(priority.ordered_subface_indices, vec![2, 1, 0]);
    assert_eq!(priority.valid_count, 1);
}

#[test]
fn worker_overlap_search_composes_valid_subface_orders() {
    let hierarchy = InitialHierarchy {
        faces_total: 3,
        relations: vec![HierarchyRelation {
            upper_face: 2,
            lower_face: 0,
        }],
    };
    let subfaces = vec![oristudio_cp::folding::SubFace {
        face_ids: vec![0, 1, 2],
    }];

    let search = possible_overlap_search_for_subfaces(&subfaces, &[0], &hierarchy, None)
        .expect("worker search should be supported");

    assert!(search.found);
    assert_eq!(search.priority.valid_count, 1);
    assert!(
        search
            .hierarchy
            .relations
            .iter()
            .any(|relation| relation.upper_face == 2 && relation.lower_face == 0)
    );
}

#[test]
fn subface_swapper_moves_recorded_dead_end_toward_front() {
    let mut swapper = SubFaceSwapper::new();
    let mut order = vec![0, 1, 2, 3];
    let counters = vec![0, 0, 0, 0];

    swapper.visit(order[0]);
    swapper.record(4);
    swapper.process(&mut order, 4, &counters);

    assert_eq!(order, vec![0, 3, 1, 2]);
    assert!(swapper.should_estimate(2));
}

#[test]
fn worker_overlap_search_with_swap_runs_realtime_search_path() {
    let hierarchy = InitialHierarchy {
        faces_total: 7,
        relations: Vec::new(),
    };
    let subfaces = vec![
        oristudio_cp::folding::SubFace {
            face_ids: vec![0, 1, 2, 3],
        },
        oristudio_cp::folding::SubFace {
            face_ids: vec![4, 5, 6],
        },
    ];
    let conditions = oristudio_cp::folding::EquivalenceConditionSet {
        triple_conditions: vec![
            oristudio_cp::folding::EquivalenceCondition {
                a: 4,
                b: 5,
                c: 4,
                d: 6,
            },
            oristudio_cp::folding::EquivalenceCondition {
                a: 5,
                b: 4,
                c: 5,
                d: 6,
            },
            oristudio_cp::folding::EquivalenceCondition {
                a: 6,
                b: 4,
                c: 6,
                d: 5,
            },
        ],
        quadruple_conditions: Vec::new(),
    };

    let search = possible_overlap_search_for_subfaces_with_swap(
        &subfaces,
        &[0, 1],
        &hierarchy,
        Some(&conditions),
    )
    .expect("worker search should be supported");

    assert!(!search.found);
    assert_eq!(search.priority.valid_count, 2);
}

#[test]
fn worker_overlap_search_promotes_final_aea_error_subface() {
    let hierarchy = InitialHierarchy {
        faces_total: 3,
        relations: vec![HierarchyRelation {
            upper_face: 2,
            lower_face: 0,
        }],
    };
    let subfaces = vec![
        oristudio_cp::folding::SubFace {
            face_ids: vec![0, 1],
        },
        oristudio_cp::folding::SubFace {
            face_ids: vec![1, 2],
        },
        oristudio_cp::folding::SubFace {
            face_ids: vec![0, 1, 2],
        },
    ];

    let search = possible_overlap_search_for_ordered_subfaces(&subfaces, 2, &hierarchy, None, true)
        .expect("worker search should be supported");

    assert!(search.found);
    assert_eq!(search.priority.valid_count, 3);
    assert_eq!(search.priority.ordered_subface_indices, vec![1, 2, 0]);
}

#[test]
fn overlap_search_from_segments_runs_folded_worker_pipeline() {
    let search = overlap_search_from_segments(&square_with_diagonal(), 1)
        .expect("overlap search should not fail")
        .expect("overlap search result");

    assert!(search.found);
    assert_eq!(search.hierarchy.faces_total, 2);
    assert!(!search.hierarchy.relations.is_empty());
}

#[test]
fn overlap_search_from_segments_with_swap_runs_initial_worker_pipeline() {
    let search = overlap_search_from_segments_with_swap(&square_with_diagonal(), 1)
        .expect("overlap search should not fail")
        .expect("overlap search result");

    assert!(search.found);
    assert_eq!(search.hierarchy.faces_total, 2);
    assert!(!search.hierarchy.relations.is_empty());
}

#[test]
fn folding_estimate_runs_ordered_stages_to_first_solution() {
    let estimate = folding_estimate_from_segments(
        &square_with_diagonal(),
        1,
        oristudio_cp::folding::EstimationOrder::Order5,
    )
    .expect("folding estimate");

    assert_eq!(
        estimate.estimation_step,
        oristudio_cp::folding::EstimationStep::Step5
    );
    assert_eq!(
        estimate.display_style,
        oristudio_cp::folding::DisplayStyle::Paper5
    );
    assert_eq!(estimate.discovered_fold_cases, 1);
    assert!(!estimate.find_another_overlap_valid);
    assert!(estimate.overlap.as_ref().is_some_and(|search| search.found));
}

#[test]
fn two_colored_subface_segments_keep_development_coordinates() {
    let prepared = two_colored_subface_segments_from_segments(&two_square_strip(), 1)
        .expect("two-colored subface preparation");

    assert!(!prepared.is_empty());
    assert!(
        prepared
            .iter()
            .any(|segment| segment.a.x == 10.0 || segment.b.x == 10.0)
    );
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

fn two_square_strip() -> Vec<LineSegment> {
    vec![
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Black0),
        segment(10.0, 0.0, 20.0, 0.0, LineColor::Black0),
        segment(20.0, 0.0, 20.0, 10.0, LineColor::Black0),
        segment(20.0, 10.0, 10.0, 10.0, LineColor::Black0),
        segment(10.0, 10.0, 0.0, 10.0, LineColor::Black0),
        segment(0.0, 10.0, 0.0, 0.0, LineColor::Black0),
        segment(10.0, 0.0, 10.0, 10.0, LineColor::Red1),
    ]
}

fn quartered_square() -> Vec<LineSegment> {
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
        LineSegment::with_color(Point::new(0.5, 0.5), Point::new(0.0, 0.0), LineColor::Red1),
        LineSegment::with_color(Point::new(0.5, 0.5), Point::new(1.0, 0.0), LineColor::Blue2),
        LineSegment::with_color(Point::new(0.5, 0.5), Point::new(1.0, 1.0), LineColor::Red1),
        LineSegment::with_color(Point::new(0.5, 0.5), Point::new(0.0, 1.0), LineColor::Blue2),
    ]
}

fn collect_permutations(mut generator: ChainPermutationGenerator, limit: usize) -> Vec<Vec<usize>> {
    let mut permutations = Vec::new();
    for step in 0..limit {
        if step > 0 && generator.next(generator.num_digits()).expect("advance") == 0 {
            break;
        }
        permutations.push(generator.current_permutation());
    }
    permutations
}

fn collect_subface_orderings(
    mut search: SubFacePermutationSearch,
    limit: usize,
) -> Vec<Vec<usize>> {
    let mut permutations = Vec::new();
    for step in 0..limit {
        if step > 0 && search.next(search.face_ids().len()).expect("advance") == 0 {
            break;
        }
        permutations.push(search.current_ordering());
    }
    permutations
}

fn position(permutation: &[usize], value: usize) -> usize {
    permutation
        .iter()
        .position(|digit| *digit == value)
        .expect("value should be present")
}

fn segment(ax: f64, ay: f64, bx: f64, by: f64, color: LineColor) -> LineSegment {
    LineSegment::with_color(Point::new(ax, ay), Point::new(bx, by), color)
}
