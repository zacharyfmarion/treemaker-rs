use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use treemaker_core::{
    Condition, ConditionKind, Crease, Edge, Facet, Node, OptimizationKind, OwnerRef,
    Path as TreePath, Point, Poly, TmFloat, Tree, Vertex,
};

mod support;
use support::{
    approx_eq, as_f64, assert_polygon_contents_record, assert_tree_poly_record, oracle_binary,
    repo_root, run_oracle_json, run_oracle_json_args,
};

#[derive(Clone)]
struct NodeSpec {
    label: &'static str,
    loc: Point,
    pinned: bool,
}

#[derive(Clone)]
struct EdgeSpec {
    a: usize,
    b: usize,
    length: TmFloat,
    pinned: bool,
    stiffness: TmFloat,
}

#[derive(Clone)]
struct FamilySpec {
    slug: &'static str,
    has_symmetry: bool,
    symmetry_angle: TmFloat,
    nodes: Vec<NodeSpec>,
    edges: Vec<EdgeSpec>,
    conditions: Vec<ConditionKind>,
    check_optimizer: bool,
}

#[test]
fn generated_tree_families_match_cpp_oracle_when_enabled() {
    let Some(mut oracle) = oracle_binary() else {
        eprintln!(
            "skipping generated-family C++ oracle parity; set TREEMAKER_CPP_ORACLE to enable"
        );
        return;
    };

    let root = repo_root();
    if oracle.is_relative() {
        oracle = root.join(oracle);
    }
    let tmp = temp_dir("generated-families");
    fs::create_dir_all(&tmp).expect("temp dir");

    for spec in generated_families() {
        let raw_tree = build_tree(&spec);
        let raw_path = tmp.join(format!("{}-raw.tmd5", spec.slug));
        fs::write(&raw_path, raw_tree.to_tmd5_string()).expect("raw seed");

        if spec.check_optimizer {
            let raw_arg = raw_path.to_string_lossy();
            let record =
                run_oracle_json_args(&oracle, &root, &["optimize", "scale", raw_arg.as_ref()]);
            let mut rust_tree = build_tree(&spec);
            let report = rust_tree.optimize_scale().expect(spec.slug);
            assert_eq!(
                report.kind,
                OptimizationKind::Scale,
                "{} optimizer kind",
                spec.slug
            );
            assert_eq!(
                report.converged,
                record["converged"].as_bool().expect("converged"),
                "{} optimizer convergence",
                spec.slug
            );
            approx_eq(rust_tree.scale, as_f64(&record, "scale"), 1.0e-3, spec.slug);
        }

        let mut seed = build_tree(&spec);
        seed.optimize_scale().expect(spec.slug);
        let seed_path = tmp.join(format!("{}-optimized.tmd5", spec.slug));
        fs::write(&seed_path, seed.to_tmd5_string()).expect("optimized seed");

        let mut tree_polys =
            Tree::from_tmd_str(&fs::read_to_string(&seed_path).expect("seed text"))
                .expect(spec.slug);
        tree_polys.build_tree_polys().expect(spec.slug);
        let tree_poly_record = run_oracle_json(&oracle, &root, "build-tree-polys", &seed_path);
        assert_tree_poly_record(&tree_polys, &tree_poly_record, spec.slug);

        let mut contents = Tree::from_tmd_str(&fs::read_to_string(&seed_path).expect("seed text"))
            .expect(spec.slug);
        contents
            .build_polygon_contents_for_oracle_tests()
            .expect(spec.slug);
        let contents_record = run_oracle_json(&oracle, &root, "build-polygon-contents", &seed_path);
        assert_polygon_contents_record(&contents, &contents_record, spec.slug);
    }

    let _ = fs::remove_dir_all(tmp);
}

fn temp_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("treemaker-{label}-{}-{stamp}", process::id()))
}

fn generated_families() -> Vec<FamilySpec> {
    vec![
        FamilySpec {
            slug: "deep-trunk",
            has_symmetry: false,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.48, 0.12),
                ns("h0", 0.48, 0.30),
                ns("h1", 0.42, 0.48),
                ns("h2", 0.56, 0.64),
                ns("h3", 0.50, 0.80),
                ns("t0", 0.12, 0.08),
                ns("t1", 0.82, 0.10),
                ns("t2", 0.16, 0.42),
                ns("t3", 0.84, 0.48),
                ns("t4", 0.18, 0.76),
                ns("t5", 0.50, 0.96),
                ns("t6", 0.88, 0.82),
            ],
            edges: vec![
                es(1, 2, 0.30),
                es(2, 3, 0.27),
                es(3, 4, 0.24),
                es(4, 5, 0.22),
                es(1, 6, 0.44),
                es(1, 7, 0.48),
                es(3, 8, 0.34),
                es(4, 9, 0.32),
                es(5, 10, 0.36),
                es(5, 11, 0.24),
                es(5, 12, 0.34),
            ],
            conditions: Vec::new(),
            check_optimizer: true,
        },
        FamilySpec {
            slug: "many-terminal-star",
            has_symmetry: false,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.50, 0.50),
                ns("t0", 0.10, 0.12),
                ns("t1", 0.34, 0.06),
                ns("t2", 0.66, 0.06),
                ns("t3", 0.90, 0.12),
                ns("t4", 0.94, 0.50),
                ns("t5", 0.88, 0.88),
                ns("t6", 0.50, 0.94),
                ns("t7", 0.12, 0.86),
                ns("t8", 0.06, 0.48),
            ],
            edges: vec![
                es(1, 2, 0.44),
                es(1, 3, 0.36),
                es(1, 4, 0.36),
                es(1, 5, 0.44),
                es(1, 6, 0.38),
                es(1, 7, 0.44),
                es(1, 8, 0.38),
                es(1, 9, 0.44),
                es(1, 10, 0.40),
            ],
            conditions: Vec::new(),
            check_optimizer: true,
        },
        FamilySpec {
            slug: "balanced-binary",
            has_symmetry: false,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.50, 0.50),
                ns("l", 0.34, 0.56),
                ns("r", 0.66, 0.56),
                ns("ll", 0.24, 0.76),
                ns("lr", 0.44, 0.78),
                ns("rl", 0.56, 0.78),
                ns("rr", 0.76, 0.76),
                ns("t0", 0.10, 0.92),
                ns("t1", 0.32, 0.96),
                ns("t2", 0.46, 0.96),
                ns("t3", 0.58, 0.96),
                ns("t4", 0.70, 0.96),
                ns("t5", 0.92, 0.92),
            ],
            edges: vec![
                es(1, 2, 0.30),
                es(1, 3, 0.30),
                es(2, 4, 0.22),
                es(2, 5, 0.22),
                es(3, 6, 0.22),
                es(3, 7, 0.22),
                es(4, 8, 0.26),
                es(4, 9, 0.20),
                es(5, 10, 0.18),
                es(6, 11, 0.18),
                es(7, 12, 0.20),
                es(7, 13, 0.26),
            ],
            conditions: Vec::new(),
            check_optimizer: true,
        },
        FamilySpec {
            slug: "asymmetric-bush",
            has_symmetry: false,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.42, 0.30),
                ns("h0", 0.34, 0.46),
                ns("h1", 0.58, 0.54),
                ns("h2", 0.42, 0.70),
                ns("t0", 0.12, 0.22),
                ns("t1", 0.78, 0.20),
                ns("t2", 0.14, 0.58),
                ns("t3", 0.86, 0.50),
                ns("t4", 0.24, 0.90),
                ns("t5", 0.58, 0.92),
            ],
            edges: vec![
                es(1, 2, 0.25),
                es(1, 3, 0.32),
                es(2, 4, 0.26),
                es(1, 5, 0.36),
                es(1, 6, 0.40),
                es(2, 7, 0.28),
                es(3, 8, 0.30),
                es(4, 9, 0.24),
                es(4, 10, 0.22),
            ],
            conditions: Vec::new(),
            check_optimizer: true,
        },
        FamilySpec {
            slug: "comb-tree",
            has_symmetry: false,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.20, 0.18),
                ns("s0", 0.32, 0.30),
                ns("s1", 0.44, 0.42),
                ns("s2", 0.56, 0.54),
                ns("s3", 0.68, 0.66),
                ns("t0", 0.10, 0.42),
                ns("t1", 0.22, 0.58),
                ns("t2", 0.34, 0.74),
                ns("t3", 0.50, 0.90),
                ns("t4", 0.88, 0.84),
            ],
            edges: vec![
                es(1, 2, 0.22),
                es(2, 3, 0.22),
                es(3, 4, 0.22),
                es(4, 5, 0.22),
                es(2, 6, 0.26),
                es(3, 7, 0.24),
                es(4, 8, 0.22),
                es(5, 9, 0.20),
                es(5, 10, 0.28),
            ],
            conditions: Vec::new(),
            check_optimizer: false,
        },
        FamilySpec {
            slug: "mirrored-pairs",
            has_symmetry: true,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.50, 0.44),
                ns("t0", 0.16, 0.18),
                ns("t1", 0.84, 0.18),
                ns("t2", 0.18, 0.82),
                ns("t3", 0.82, 0.82),
            ],
            edges: vec![
                es(1, 2, 0.92),
                es(1, 3, 0.92),
                es(1, 4, 0.82),
                es(1, 5, 0.82),
            ],
            conditions: vec![
                ConditionKind::NodesPaired { node1: 2, node2: 3 },
                ConditionKind::NodesPaired { node1: 4, node2: 5 },
            ],
            check_optimizer: true,
        },
        FamilySpec {
            slug: "node-fixed-edge-fixed",
            has_symmetry: false,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.50, 0.50),
                nsp("t0", 0.06, 0.50),
                ns("t1", 0.84, 0.22),
                ns("t2", 0.70, 0.86),
            ],
            edges: vec![esp(1, 2, 0.44), es(1, 3, 0.52), es(1, 4, 0.48)],
            conditions: vec![
                ConditionKind::NodeFixed {
                    node: 2,
                    x_fixed: true,
                    y_fixed: true,
                    x_fix_value: 0.06,
                    y_fix_value: 0.50,
                },
                ConditionKind::EdgeLengthFixed { edge: 1 },
            ],
            check_optimizer: false,
        },
        FamilySpec {
            slug: "corner-and-edge",
            has_symmetry: false,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.46, 0.42),
                ns("corner", 0.00, 0.00),
                ns("edge", 1.00, 0.42),
                ns("free", 0.30, 0.92),
            ],
            edges: vec![es(1, 2, 0.60), es(1, 3, 0.48), es(1, 4, 0.46)],
            conditions: vec![
                ConditionKind::NodeOnCorner { node: 2 },
                ConditionKind::NodeOnEdge { node: 3 },
            ],
            check_optimizer: false,
        },
        FamilySpec {
            slug: "collinear-same-strain",
            has_symmetry: false,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.50, 0.52),
                ns("a", 0.18, 0.20),
                ns("b", 0.50, 0.20),
                ns("c", 0.82, 0.20),
                ns("top", 0.50, 0.88),
            ],
            edges: vec![
                es(1, 2, 0.42),
                ess(1, 3, 0.34, 1.5),
                ess(1, 4, 0.42, 1.5),
                es(1, 5, 0.36),
            ],
            conditions: vec![
                ConditionKind::NodesCollinear {
                    node1: 2,
                    node2: 3,
                    node3: 4,
                },
                ConditionKind::EdgesSameStrain { edge1: 2, edge2: 3 },
            ],
            check_optimizer: false,
        },
        FamilySpec {
            slug: "path-constraints",
            has_symmetry: false,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.50, 0.50),
                ns("east", 0.90, 0.50),
                ns("north", 0.50, 0.90),
                ns("west", 0.10, 0.50),
                ns("south", 0.50, 0.10),
            ],
            edges: vec![
                es(1, 2, 0.42),
                es(1, 3, 0.42),
                es(1, 4, 0.42),
                es(1, 5, 0.42),
            ],
            conditions: vec![
                ConditionKind::PathActive { node1: 2, node2: 3 },
                ConditionKind::PathAngleFixed {
                    node1: 2,
                    node2: 3,
                    angle: 90.0,
                },
                ConditionKind::PathAngleQuant {
                    node1: 3,
                    node2: 4,
                    quant: 4,
                    quant_offset: 0.0,
                },
            ],
            check_optimizer: false,
        },
        FamilySpec {
            slug: "node-combo-symmetric",
            has_symmetry: true,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.50, 0.46),
                ns("left", 0.18, 0.24),
                ns("right", 0.82, 0.24),
                ns("center", 0.50, 0.88),
            ],
            edges: vec![es(1, 2, 0.42), es(1, 3, 0.42), es(1, 4, 0.46)],
            conditions: vec![
                ConditionKind::NodeSymmetric { node: 4 },
                ConditionKind::NodeCombo {
                    node: 2,
                    to_symmetry_line: false,
                    to_paper_edge: false,
                    to_paper_corner: false,
                    x_fixed: true,
                    x_fix_value: 0.18,
                    y_fixed: false,
                    y_fix_value: 0.0,
                },
            ],
            check_optimizer: false,
        },
    ]
}

fn ns(label: &'static str, x: TmFloat, y: TmFloat) -> NodeSpec {
    NodeSpec {
        label,
        loc: Point { x, y },
        pinned: false,
    }
}

fn nsp(label: &'static str, x: TmFloat, y: TmFloat) -> NodeSpec {
    NodeSpec {
        label,
        loc: Point { x, y },
        pinned: true,
    }
}

fn es(a: usize, b: usize, length: TmFloat) -> EdgeSpec {
    EdgeSpec {
        a,
        b,
        length,
        pinned: false,
        stiffness: 1.0,
    }
}

fn esp(a: usize, b: usize, length: TmFloat) -> EdgeSpec {
    EdgeSpec {
        a,
        b,
        length,
        pinned: true,
        stiffness: 1.0,
    }
}

fn ess(a: usize, b: usize, length: TmFloat, stiffness: TmFloat) -> EdgeSpec {
    EdgeSpec {
        a,
        b,
        length,
        pinned: false,
        stiffness,
    }
}

fn build_tree(spec: &FamilySpec) -> Tree {
    let n = spec.nodes.len();
    let mut adjacency = vec![Vec::<(usize, usize)>::new(); n + 1];
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for (i, node) in spec.nodes.iter().enumerate() {
        nodes.push(Node {
            index: i + 1,
            label: node.label.to_string(),
            loc: node.loc,
            depth: -999.0,
            elevation: 0.0,
            is_leaf: false,
            is_sub: false,
            is_border: false,
            is_pinned: node.pinned,
            is_polygon: false,
            is_junction: false,
            is_conditioned: false,
            owned_vertices: Vec::new(),
            edges: Vec::new(),
            leaf_paths: Vec::new(),
            owner: OwnerRef::Tree,
        });
    }

    for (i, edge) in spec.edges.iter().enumerate() {
        let edge_id = i + 1;
        edges.push(Edge {
            index: edge_id,
            label: format!("e{edge_id}"),
            length: edge.length,
            strain: 0.0,
            stiffness: edge.stiffness,
            is_pinned: edge.pinned,
            is_conditioned: false,
            nodes: vec![edge.a, edge.b],
        });
        adjacency[edge.a].push((edge.b, edge_id));
        adjacency[edge.b].push((edge.a, edge_id));
        nodes[edge.a - 1].edges.push(edge_id);
        nodes[edge.b - 1].edges.push(edge_id);
    }

    let leaf = |node_id: usize, adjacency: &[Vec<(usize, usize)>]| adjacency[node_id].len() == 1;
    for node_id in 1..=n {
        nodes[node_id - 1].is_leaf = leaf(node_id, &adjacency);
    }

    let mut paths = Vec::new();
    for a in 1..=n {
        for b in a + 1..=n {
            let (path_nodes, path_edges) = tree_path(a, b, &adjacency);
            let path_id = paths.len() + 1;
            let min_tree_length = path_edges
                .iter()
                .map(|edge_id| edges[*edge_id - 1].length)
                .sum::<TmFloat>();
            let is_leaf = leaf(a, &adjacency) && leaf(b, &adjacency);
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

    let conditions = spec
        .conditions
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, kind)| Condition {
            index: i + 1,
            is_feasible: true,
            kind,
        })
        .collect();
    let owned_paths = (1..=paths.len()).collect();

    Tree {
        source_version: "5.0".to_string(),
        paper_width: 1.0,
        paper_height: 1.0,
        scale: 0.1,
        has_symmetry: spec.has_symmetry,
        sym_loc: Point { x: 0.5, y: 0.0 },
        sym_angle: spec.symmetry_angle,
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
        conditions,
        owned_nodes: (1..=n).collect(),
        owned_edges: (1..=spec.edges.len()).collect(),
        owned_paths,
        owned_polys: Vec::new(),
    }
}

fn tree_path(
    start: usize,
    end: usize,
    adjacency: &[Vec<(usize, usize)>],
) -> (Vec<usize>, Vec<usize>) {
    let mut parent: HashMap<usize, (usize, usize)> = HashMap::new();
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
