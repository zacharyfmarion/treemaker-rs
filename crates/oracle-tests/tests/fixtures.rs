use treemaker_core::{Tree, TreeError};

const FIXTURES: &[(&str, &str)] = &[
    (
        "tmModelTester_1.tmd5",
        include_str!("../../../tests/fixtures/tmModelTester_1.tmd5"),
    ),
    (
        "tmModelTester_2.tmd5",
        include_str!("../../../tests/fixtures/tmModelTester_2.tmd5"),
    ),
    (
        "tmModelTester_3.tmd5",
        include_str!("../../../tests/fixtures/tmModelTester_3.tmd5"),
    ),
    (
        "tmModelTester_4.tmd5",
        include_str!("../../../tests/fixtures/tmModelTester_4.tmd5"),
    ),
    (
        "tmModelTester_5.tmd5",
        include_str!("../../../tests/fixtures/tmModelTester_5.tmd5"),
    ),
    (
        "minimal_v3.tmd",
        include_str!("../../../tests/fixtures/minimal_v3.tmd"),
    ),
    (
        "minimal_cp_v4.tmd4",
        include_str!("../../../tests/fixtures/minimal_cp_v4.tmd4"),
    ),
    (
        "minimal_cp_v5.tmd5",
        include_str!("../../../tests/fixtures/minimal_cp_v5.tmd5"),
    ),
];

#[test]
fn all_model_tester_fixtures_parse() {
    for (name, text) in FIXTURES {
        let tree = Tree::from_tmd_str(text).unwrap_or_else(|err| panic!("{name}: {err}"));
        let summary = tree.summary();
        assert!(summary.nodes > 0, "{name}");
        assert!(summary.edges > 0, "{name}");
        assert!(summary.paths > 0, "{name}");
    }
}

#[test]
fn v4_export_reparses() {
    for (name, text) in FIXTURES {
        let tree = Tree::from_tmd_str(text).unwrap_or_else(|err| panic!("{name}: {err}"));
        let exported = tree.export_v4_string();
        let reparsed =
            Tree::from_tmd_str(&exported).unwrap_or_else(|err| panic!("{name} export: {err}"));
        assert_eq!(tree.summary().nodes, reparsed.summary().nodes, "{name}");
        assert_eq!(tree.summary().edges, reparsed.summary().edges, "{name}");
        assert_eq!(tree.summary().paths, reparsed.summary().paths, "{name}");
    }
}

#[test]
fn unported_algorithms_fail_loudly() {
    let mut tree = Tree::from_tmd_str(FIXTURES[0].1).unwrap();
    assert!(tree.optimize_scale().unwrap().converged);
    let mut edge_tree = Tree::from_tmd_str(FIXTURES[3].1).unwrap();
    assert!(edge_tree.optimize_edges().unwrap().converged);
    let mut strain_tree = Tree::from_tmd_str(FIXTURES[4].1).unwrap();
    assert!(strain_tree.optimize_strain().unwrap().converged);
    assert!(matches!(
        strain_tree.build_polys_and_crease_pattern(),
        Err(TreeError::UnsupportedOperation(_))
    ));
}
