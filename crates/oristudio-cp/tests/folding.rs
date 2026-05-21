use oristudio_cp::folding::{
    ChainPermutationGenerator, HierarchyRelation, additional_estimation_from_segments,
    configure_subfaces_from_segments, equivalence_condition_candidates_from_segments,
    estimate_wireframe_from_segments, initial_hierarchy_from_segments, prepare_subface_segments,
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

fn position(permutation: &[usize], value: usize) -> usize {
    permutation
        .iter()
        .position(|digit| *digit == value)
        .expect("value should be present")
}
