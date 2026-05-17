use std::collections::{HashMap, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};

use proptest::prelude::*;
use treemaker_core::{
    Condition, Crease, Edge, Facet, Node, OwnerRef, Path as TreePath, Point, Poly, TmFloat, Tree,
    TreeError, Vertex,
};

#[derive(Debug, Clone)]
struct TreeSpec {
    locs: Vec<Point>,
    parents: Vec<usize>,
    lengths: Vec<TmFloat>,
    pinned: Vec<bool>,
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        failure_persistence: None,
        .. ProptestConfig::default()
    })]

    #[test]
    fn valid_random_trees_roundtrip_and_operations_do_not_panic(spec in tree_specs()) {
        let tree = build_tree(&spec);
        let text = tree.to_tmd5_string();
        let parsed = Tree::from_tmd_str(&text).expect("valid generated tree parses");
        let reparsed = Tree::from_tmd_str(&parsed.to_tmd5_string()).expect("generated tree roundtrips");
        prop_assert_eq!(parsed.summary(), reparsed.summary());

        let mut optimized = parsed.clone();
        let optimize = catch_unwind(AssertUnwindSafe(|| optimized.optimize_scale()));
        prop_assert!(optimize.is_ok(), "optimize_scale panicked");
        if let Err(error) = optimize.unwrap() {
            prop_assert!(matches!(
                error,
                TreeError::OptimizerConvergence(_) | TreeError::InvalidOperation(_)
            ));
        }

        let mut built = parsed;
        let build = catch_unwind(AssertUnwindSafe(|| built.build_polys_and_crease_pattern()));
        prop_assert!(build.is_ok(), "build_polys_and_crease_pattern panicked");
        if let Err(error) = build.unwrap() {
            prop_assert!(matches!(error, TreeError::InvalidOperation(_)));
        }
    }

    #[test]
    fn malformed_fixture_mutations_return_structured_errors(
        fixture_index in 0usize..fixture_texts().len(),
        mutation in 0usize..5,
    ) {
        let fixture = fixture_texts()[fixture_index];
        let mutated = mutate_fixture(fixture, mutation);
        let parsed = catch_unwind(AssertUnwindSafe(|| Tree::from_tmd_str(&mutated)));
        prop_assert!(parsed.is_ok(), "parser panicked");
        match parsed.unwrap() {
            Ok(_) => prop_assert!(false, "malformed mutation unexpectedly parsed"),
            Err(error) => {
                let structured = matches!(
                    error,
                    TreeError::Parse { .. }
                        | TreeError::BadReference { .. }
                        | TreeError::UnsupportedVersion(_)
                        | TreeError::UnsupportedOperation(_)
                );
                prop_assert!(structured, "{error:?}");
            }
        }
    }
}

fn tree_specs() -> impl Strategy<Value = TreeSpec> {
    (2usize..=8)
        .prop_flat_map(|node_count| {
            (
                Just(node_count),
                prop::collection::vec(0usize..8, node_count - 1),
                prop::collection::vec((0u16..=1000, 0u16..=1000), node_count),
                prop::collection::vec(1u16..=95, node_count - 1),
                prop::collection::vec(any::<bool>(), node_count),
            )
        })
        .prop_map(|(node_count, raw_parents, raw_locs, raw_lengths, pinned)| {
            let parents = (2..=node_count)
                .map(|node| (raw_parents[node - 2] % (node - 1)) + 1)
                .collect::<Vec<_>>();
            let locs = raw_locs
                .into_iter()
                .map(|(x, y)| Point {
                    x: 0.05 + f64::from(x) * 0.0009,
                    y: 0.05 + f64::from(y) * 0.0009,
                })
                .collect::<Vec<_>>();
            let lengths = raw_lengths
                .into_iter()
                .map(|length| 0.05 + f64::from(length) * 0.01)
                .collect::<Vec<_>>();
            TreeSpec {
                locs,
                parents,
                lengths,
                pinned,
            }
        })
}

fn build_tree(spec: &TreeSpec) -> Tree {
    let node_count = spec.locs.len();
    let mut adjacency = vec![Vec::<(usize, usize)>::new(); node_count + 1];
    let mut nodes = spec
        .locs
        .iter()
        .enumerate()
        .map(|(i, loc)| Node {
            index: i + 1,
            label: format!("n{}", i + 1),
            loc: *loc,
            depth: -999.0,
            elevation: 0.0,
            is_leaf: false,
            is_sub: false,
            is_border: false,
            is_pinned: spec.pinned[i],
            is_polygon: false,
            is_junction: false,
            is_conditioned: false,
            owned_vertices: Vec::new(),
            edges: Vec::new(),
            leaf_paths: Vec::new(),
            owner: OwnerRef::Tree,
        })
        .collect::<Vec<_>>();
    let mut edges = Vec::new();

    for (i, parent) in spec.parents.iter().copied().enumerate() {
        let node = i + 2;
        let edge_id = i + 1;
        edges.push(Edge {
            index: edge_id,
            label: format!("e{edge_id}"),
            length: spec.lengths[i],
            strain: 0.0,
            stiffness: 1.0,
            is_pinned: false,
            is_conditioned: false,
            nodes: vec![parent, node],
        });
        adjacency[parent].push((node, edge_id));
        adjacency[node].push((parent, edge_id));
        nodes[parent - 1].edges.push(edge_id);
        nodes[node - 1].edges.push(edge_id);
    }

    for node_id in 1..=node_count {
        nodes[node_id - 1].is_leaf = adjacency[node_id].len() == 1;
    }

    let mut paths = Vec::new();
    for a in 1..=node_count {
        for b in a + 1..=node_count {
            let (path_nodes, path_edges) = tree_path(a, b, &adjacency);
            let path_id = paths.len() + 1;
            let min_tree_length = path_edges
                .iter()
                .map(|edge_id| edges[*edge_id - 1].length)
                .sum::<TmFloat>();
            let is_leaf = adjacency[a].len() == 1 && adjacency[b].len() == 1;
            if is_leaf {
                nodes[a - 1].leaf_paths.push(path_id);
                nodes[b - 1].leaf_paths.push(path_id);
            }
            paths.push(TreePath {
                index: path_id,
                min_tree_length,
                min_paper_length: min_tree_length * 0.1,
                act_tree_length: 0.0,
                act_paper_length: 0.0,
                is_leaf,
                is_sub: false,
                is_feasible: false,
                is_active: false,
                is_border: false,
                is_polygon: false,
                is_conditioned: false,
                fwd_poly: None,
                bkd_poly: None,
                nodes: path_nodes,
                edges: path_edges,
                outset_path: None,
                front_reduction: 0.0,
                back_reduction: 0.0,
                min_depth: -999.0,
                min_depth_dist: -999.0,
                owned_vertices: Vec::new(),
                owned_creases: Vec::new(),
                owner: OwnerRef::Tree,
            });
        }
    }

    let owned_paths = (1..=paths.len()).collect();
    Tree {
        source_version: "5.0".to_string(),
        paper_width: 1.0,
        paper_height: 1.0,
        scale: 0.1,
        has_symmetry: false,
        sym_loc: Point { x: 0.5, y: 0.0 },
        sym_angle: 90.0,
        is_feasible: false,
        is_polygon_valid: false,
        is_polygon_filled: false,
        is_vertex_depth_valid: false,
        is_facet_data_valid: false,
        is_local_root_connectable: false,
        needs_cleanup: true,
        nodes,
        edges,
        paths,
        polys: Vec::<Poly>::new(),
        vertices: Vec::<Vertex>::new(),
        creases: Vec::<Crease>::new(),
        facets: Vec::<Facet>::new(),
        conditions: Vec::<Condition>::new(),
        owned_nodes: (1..=node_count).collect(),
        owned_edges: (1..=spec.parents.len()).collect(),
        owned_paths,
        owned_polys: Vec::new(),
    }
}

fn tree_path(
    start: usize,
    end: usize,
    adjacency: &[Vec<(usize, usize)>],
) -> (Vec<usize>, Vec<usize>) {
    let mut parent = HashMap::<usize, (usize, usize)>::new();
    let mut queue = VecDeque::from([start]);
    parent.insert(start, (0, 0));
    while let Some(node) = queue.pop_front() {
        if node == end {
            break;
        }
        for (next, edge_id) in adjacency[node].iter().copied() {
            if parent.contains_key(&next) {
                continue;
            }
            parent.insert(next, (node, edge_id));
            queue.push_back(next);
        }
    }

    let mut nodes = vec![end];
    let mut edges = Vec::new();
    let mut cur = end;
    while cur != start {
        let (prev, edge_id) = parent[&cur];
        edges.push(edge_id);
        nodes.push(prev);
        cur = prev;
    }
    nodes.reverse();
    edges.reverse();
    (nodes, edges)
}

fn fixture_texts() -> Vec<&'static str> {
    vec![
        include_str!("../../../tests/fixtures/minimal_v3.tmd"),
        include_str!("../../../tests/fixtures/minimal_cp_v4.tmd4"),
        include_str!("../../../tests/fixtures/minimal_cp_v5.tmd5"),
        include_str!("../../../tests/fixtures/tmModelTester_1.tmd5"),
        include_str!("../../../tests/fixtures/tmModelTester_5.tmd5"),
    ]
}

fn mutate_fixture(text: &str, mutation: usize) -> String {
    let normalized = text.replace('\r', "\n");
    let mut lines = normalized.lines().map(str::to_string).collect::<Vec<_>>();
    match mutation {
        0 => lines[0] = "not_tree".to_string(),
        1 => lines[1] = "99.0".to_string(),
        2 => lines[2] = "not-a-number".to_string(),
        3 => {
            if let Some(index) = lines.iter().position(|line| line == "edge") {
                lines[index] = "node".to_string();
            }
        }
        _ => lines.truncate((lines.len() / 2).max(1)),
    }
    lines.join("\n")
}
