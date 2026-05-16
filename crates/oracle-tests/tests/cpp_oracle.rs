use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use treemaker_core::Tree;

const FIXTURE_DIR: &str = "tests/fixtures";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn oracle_binary() -> Option<PathBuf> {
    env::var_os("TREEMAKER_CPP_ORACLE").map(PathBuf::from)
}

fn approx_eq(actual: f64, expected: f64, tol: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: actual {actual:.10}, expected {expected:.10}, tol {tol}"
    );
}

fn as_usize(record: &Value, key: &str) -> usize {
    record[key].as_u64().expect(key) as usize
}

fn as_f64(record: &Value, key: &str) -> f64 {
    record[key].as_f64().expect(key)
}

fn as_bool(record: &Value, key: &str) -> bool {
    record[key].as_bool().expect(key)
}

fn as_usize_array(record: &Value, key: &str) -> Vec<usize> {
    record[key]
        .as_array()
        .expect(key)
        .iter()
        .map(|value| value.as_u64().expect(key) as usize)
        .collect()
}

fn as_point(value: &Value) -> (f64, f64) {
    let array = value.as_array().expect("point");
    (
        array[0].as_f64().expect("point x"),
        array[1].as_f64().expect("point y"),
    )
}

fn weighted_rms_strain_percent(tree: &Tree) -> f64 {
    if tree.edges.is_empty() {
        return 0.0;
    }
    let ss = tree
        .edges
        .iter()
        .map(|edge| edge.stiffness * edge.strain.powi(2))
        .sum::<f64>()
        / tree.edges.len() as f64;
    100.0 * ss.sqrt()
}

#[test]
fn cpp_oracle_matches_rust_parse_and_stable_optimizer_cases_when_enabled() {
    let Some(mut oracle) = oracle_binary() else {
        eprintln!("skipping C++ oracle parity; set TREEMAKER_CPP_ORACLE to enable");
        return;
    };

    let root = repo_root();
    if oracle.is_relative() {
        oracle = root.join(oracle);
    }
    let output = Command::new(&oracle)
        .current_dir(&root)
        .args(["run-fixtures", "--fixture-dir", FIXTURE_DIR])
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", oracle.display()));

    assert!(
        output.status.success(),
        "oracle failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("oracle stdout utf8");
    let records: Vec<Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|err| panic!("{line}: {err}")))
        .collect();
    assert_eq!(records.len(), 14);

    for record in records
        .iter()
        .filter(|record| record["case"].as_str() == Some("summary"))
    {
        let file = record["file"].as_str().expect("file");
        let text = std::fs::read_to_string(root.join(FIXTURE_DIR).join(file)).expect(file);
        let tree = Tree::from_tmd_str(&text).expect(file);
        let summary = tree.summary();

        approx_eq(
            summary.paper_width,
            as_f64(record, "paper_width"),
            1.0e-12,
            file,
        );
        approx_eq(
            summary.paper_height,
            as_f64(record, "paper_height"),
            1.0e-12,
            file,
        );
        approx_eq(summary.scale, as_f64(record, "scale"), 1.0e-10, file);
        assert_eq!(
            summary.has_symmetry,
            record["has_symmetry"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(
            summary.is_feasible,
            record["is_feasible"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(
            tree.is_polygon_valid,
            record["is_polygon_valid"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(
            tree.is_polygon_filled,
            record["is_polygon_filled"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(
            tree.is_vertex_depth_valid,
            record["is_vertex_depth_valid"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(
            tree.is_facet_data_valid,
            record["is_facet_data_valid"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(
            tree.is_local_root_connectable,
            record["is_local_root_connectable"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(summary.nodes, as_usize(record, "nodes"), "{file}");
        assert_eq!(summary.edges, as_usize(record, "edges"), "{file}");
        assert_eq!(summary.paths, as_usize(record, "paths"), "{file}");
        assert_eq!(summary.polys, as_usize(record, "polys"), "{file}");
        assert_eq!(summary.vertices, as_usize(record, "vertices"), "{file}");
        assert_eq!(summary.creases, as_usize(record, "creases"), "{file}");
        assert_eq!(summary.facets, as_usize(record, "facets"), "{file}");
        assert_eq!(summary.conditions, as_usize(record, "conditions"), "{file}");
        assert_eq!(summary.leaf_nodes, as_usize(record, "leaf_nodes"), "{file}");
        assert_eq!(summary.leaf_paths, as_usize(record, "leaf_paths"), "{file}");
        assert_eq!(
            summary.feasible_paths,
            as_usize(record, "feasible_paths"),
            "{file}"
        );
        assert_eq!(
            summary.active_paths,
            as_usize(record, "active_paths"),
            "{file}"
        );
        assert_eq!(
            summary.border_nodes,
            as_usize(record, "border_nodes"),
            "{file}"
        );
        assert_eq!(
            summary.border_paths,
            as_usize(record, "border_paths"),
            "{file}"
        );
        assert_eq!(
            summary.polygon_nodes,
            as_usize(record, "polygon_nodes"),
            "{file}"
        );
        assert_eq!(
            summary.polygon_paths,
            as_usize(record, "polygon_paths"),
            "{file}"
        );
        assert_eq!(
            summary.pinned_nodes,
            as_usize(record, "pinned_nodes"),
            "{file}"
        );
        assert_eq!(
            summary.pinned_edges,
            as_usize(record, "pinned_edges"),
            "{file}"
        );
        assert_eq!(
            summary.conditioned_nodes,
            as_usize(record, "conditioned_nodes"),
            "{file}"
        );
        assert_eq!(
            summary.conditioned_edges,
            as_usize(record, "conditioned_edges"),
            "{file}"
        );
        assert_eq!(
            summary.conditioned_paths,
            as_usize(record, "conditioned_paths"),
            "{file}"
        );
    }

    for record in records
        .iter()
        .filter(|record| record["case"].as_str() == Some("optimize"))
    {
        let file = record["file"].as_str().expect("file");
        let kind = record["kind"].as_str().expect("kind");
        if kind == "scale" && matches!(file, "tmModelTester_2.tmd5" | "tmModelTester_3.tmd5") {
            assert!(record["converged"].as_bool().unwrap(), "{file}");
            assert!(record["is_feasible"].as_bool().unwrap(), "{file}");
            continue;
        }

        let text = std::fs::read_to_string(root.join(FIXTURE_DIR).join(file)).expect(file);
        let mut tree = Tree::from_tmd_str(&text).expect(file);

        let report = match kind {
            "scale" => tree.optimize_scale().expect(file),
            "edge" => tree.optimize_edges().expect(file),
            "strain" => tree.optimize_strain().expect(file),
            other => panic!("unknown oracle optimization kind {other}"),
        };

        assert_eq!(
            report.converged,
            record["converged"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(
            tree.is_feasible(),
            record["is_feasible"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(
            tree.is_polygon_valid,
            record["is_polygon_valid"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(
            tree.is_polygon_filled,
            record["is_polygon_filled"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(
            tree.is_vertex_depth_valid,
            record["is_vertex_depth_valid"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(
            tree.is_facet_data_valid,
            record["is_facet_data_valid"].as_bool().unwrap(),
            "{file}"
        );
        assert_eq!(
            tree.is_local_root_connectable,
            record["is_local_root_connectable"].as_bool().unwrap(),
            "{file}"
        );
        approx_eq(tree.scale, as_f64(record, "scale"), 5.0e-7, file);

        let max_edge_strain = tree
            .edges
            .iter()
            .map(|edge| edge.strain)
            .fold(0.0_f64, f64::max);
        approx_eq(
            max_edge_strain,
            as_f64(record, "max_edge_strain"),
            5.0e-7,
            file,
        );
        approx_eq(
            weighted_rms_strain_percent(&tree),
            as_f64(record, "weighted_rms_strain_percent"),
            5.0e-7,
            file,
        );

        let summary = tree.summary();
        assert_eq!(summary.nodes, as_usize(record, "nodes"), "{file}");
        assert_eq!(summary.edges, as_usize(record, "edges"), "{file}");
        assert_eq!(summary.paths, as_usize(record, "paths"), "{file}");
        assert_eq!(summary.polys, as_usize(record, "polys"), "{file}");
        assert_eq!(summary.vertices, as_usize(record, "vertices"), "{file}");
        assert_eq!(summary.creases, as_usize(record, "creases"), "{file}");
        assert_eq!(summary.facets, as_usize(record, "facets"), "{file}");
        assert_eq!(
            summary.feasible_paths,
            as_usize(record, "feasible_paths"),
            "{file}"
        );
        assert_eq!(
            summary.active_paths,
            as_usize(record, "active_paths"),
            "{file}"
        );
        assert_eq!(
            summary.border_nodes,
            as_usize(record, "border_nodes"),
            "{file}"
        );
        assert_eq!(
            summary.border_paths,
            as_usize(record, "border_paths"),
            "{file}"
        );
        assert_eq!(
            summary.polygon_nodes,
            as_usize(record, "polygon_nodes"),
            "{file}"
        );
        assert_eq!(
            summary.polygon_paths,
            as_usize(record, "polygon_paths"),
            "{file}"
        );
        assert_eq!(
            summary.pinned_nodes,
            as_usize(record, "pinned_nodes"),
            "{file}"
        );
        assert_eq!(
            summary.pinned_edges,
            as_usize(record, "pinned_edges"),
            "{file}"
        );
        assert_eq!(
            summary.conditioned_nodes,
            as_usize(record, "conditioned_nodes"),
            "{file}"
        );
        assert_eq!(
            summary.conditioned_edges,
            as_usize(record, "conditioned_edges"),
            "{file}"
        );
        assert_eq!(
            summary.conditioned_paths,
            as_usize(record, "conditioned_paths"),
            "{file}"
        );
    }
}

#[test]
fn cpp_oracle_matches_rust_build_tree_polys_when_enabled() {
    let Some(mut oracle) = oracle_binary() else {
        eprintln!("skipping C++ oracle parity; set TREEMAKER_CPP_ORACLE to enable");
        return;
    };

    let root = repo_root();
    if oracle.is_relative() {
        oracle = root.join(oracle);
    }

    for file in ["tmModelTester_1.tmd5", "tmModelTester_4.tmd5"] {
        let output = Command::new(&oracle)
            .current_dir(&root)
            .args([
                "build-tree-polys",
                &root.join(FIXTURE_DIR).join(file).to_string_lossy(),
            ])
            .output()
            .unwrap_or_else(|err| panic!("failed to run {}: {err}", oracle.display()));

        assert!(
            output.status.success(),
            "oracle failed for {file}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8(output.stdout).expect("oracle stdout utf8");
        let record: Value = serde_json::from_str(stdout.trim())
            .unwrap_or_else(|err| panic!("{file}: {stdout}: {err}"));
        let text = std::fs::read_to_string(root.join(FIXTURE_DIR).join(file)).expect(file);
        let mut tree = Tree::from_tmd_str(&text).expect(file);
        tree.build_tree_polys().expect(file);

        assert_eq!(tree.is_feasible, as_bool(&record, "is_feasible"), "{file}");
        assert_eq!(
            tree.is_polygon_valid,
            as_bool(&record, "is_polygon_valid"),
            "{file}"
        );
        assert_eq!(
            tree.is_polygon_filled,
            as_bool(&record, "is_polygon_filled"),
            "{file}"
        );
        assert_eq!(tree.nodes.len(), as_usize(&record, "nodes"), "{file}");
        assert_eq!(tree.paths.len(), as_usize(&record, "paths"), "{file}");
        assert_eq!(tree.polys.len(), as_usize(&record, "polys"), "{file}");
        assert_eq!(
            tree.owned_polys.len(),
            as_usize(&record, "owned_polys"),
            "{file}"
        );
        assert_eq!(tree.vertices.len(), as_usize(&record, "vertices"), "{file}");
        assert_eq!(tree.creases.len(), as_usize(&record, "creases"), "{file}");
        assert_eq!(tree.facets.len(), as_usize(&record, "facets"), "{file}");
        assert_eq!(
            tree.owned_polys,
            as_usize_array(&record, "owned_poly_ids"),
            "{file}"
        );

        let summary = tree.summary();
        assert_eq!(
            summary.polygon_nodes,
            as_usize(&record, "polygon_nodes"),
            "{file}"
        );
        assert_eq!(
            summary.polygon_paths,
            as_usize(&record, "polygon_paths"),
            "{file}"
        );
        assert_eq!(
            summary.border_nodes,
            as_usize(&record, "border_nodes"),
            "{file}"
        );
        assert_eq!(
            summary.border_paths,
            as_usize(&record, "border_paths"),
            "{file}"
        );
        assert_eq!(
            summary.active_paths,
            as_usize(&record, "active_paths"),
            "{file}"
        );
        assert_eq!(
            summary.feasible_paths,
            as_usize(&record, "feasible_paths"),
            "{file}"
        );

        let polys = record["polys_detail"].as_array().expect("polys_detail");
        assert_eq!(tree.polys.len(), polys.len(), "{file}");
        for poly_record in polys {
            let index = as_usize(poly_record, "index");
            let poly = &tree.polys[index - 1];
            assert_eq!(
                poly.is_sub_poly,
                as_bool(poly_record, "is_sub_poly"),
                "{file}"
            );
            let (cx, cy) = as_point(&poly_record["centroid"]);
            approx_eq(poly.centroid.x, cx, 1.0e-9, file);
            approx_eq(poly.centroid.y, cy, 1.0e-9, file);
            assert_eq!(
                poly.ring_nodes,
                as_usize_array(poly_record, "ring_nodes"),
                "{file}"
            );
            assert_eq!(
                poly.ring_paths,
                as_usize_array(poly_record, "ring_paths"),
                "{file}"
            );
            assert_eq!(
                poly.cross_paths,
                as_usize_array(poly_record, "cross_paths"),
                "{file}"
            );
            assert_eq!(
                poly.inset_nodes,
                as_usize_array(poly_record, "inset_nodes"),
                "{file}"
            );
            assert_eq!(
                poly.spoke_paths,
                as_usize_array(poly_record, "spoke_paths"),
                "{file}"
            );
            assert_eq!(
                poly.ridge_path.unwrap_or(0),
                as_usize(poly_record, "ridge_path"),
                "{file}"
            );
            assert_eq!(
                poly.owned_nodes,
                as_usize_array(poly_record, "owned_nodes"),
                "{file}"
            );
            assert_eq!(
                poly.owned_paths,
                as_usize_array(poly_record, "owned_paths"),
                "{file}"
            );
            assert_eq!(
                poly.owned_polys,
                as_usize_array(poly_record, "owned_polys"),
                "{file}"
            );

            let node_locs = poly_record["node_locs"].as_array().expect("node_locs");
            assert_eq!(poly.node_locs.len(), node_locs.len(), "{file}");
            for (loc, expected) in poly.node_locs.iter().zip(node_locs) {
                let (x, y) = as_point(expected);
                approx_eq(loc.x, x, 1.0e-9, file);
                approx_eq(loc.y, y, 1.0e-9, file);
            }
        }

        let path_sides = record["polygon_path_sides"]
            .as_array()
            .expect("polygon_path_sides");
        let rust_path_sides: Vec<_> = tree.paths.iter().filter(|path| path.is_polygon).collect();
        assert_eq!(rust_path_sides.len(), path_sides.len(), "{file}");
        for side in path_sides {
            let index = as_usize(side, "index");
            let path = &tree.paths[index - 1];
            assert!(path.is_polygon, "{file}");
            assert_eq!(path.nodes, as_usize_array(side, "nodes"), "{file}");
            assert_eq!(path.is_border, as_bool(side, "is_border"), "{file}");
            assert_eq!(
                path.fwd_poly.unwrap_or(0),
                as_usize(side, "fwd_poly"),
                "{file}"
            );
            assert_eq!(
                path.bkd_poly.unwrap_or(0),
                as_usize(side, "bkd_poly"),
                "{file}"
            );
        }
    }
}

#[test]
fn cpp_oracle_matches_rust_polygon_contents_when_enabled() {
    let Some(mut oracle) = oracle_binary() else {
        eprintln!("skipping C++ oracle parity; set TREEMAKER_CPP_ORACLE to enable");
        return;
    };

    let root = repo_root();
    if oracle.is_relative() {
        oracle = root.join(oracle);
    }

    for file in ["tmModelTester_1.tmd5", "tmModelTester_4.tmd5"] {
        let output = Command::new(&oracle)
            .current_dir(&root)
            .args([
                "build-polygon-contents",
                &root.join(FIXTURE_DIR).join(file).to_string_lossy(),
            ])
            .output()
            .unwrap_or_else(|err| panic!("failed to run {}: {err}", oracle.display()));

        assert!(
            output.status.success(),
            "oracle failed for {file}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8(output.stdout).expect("oracle stdout utf8");
        let record: Value = serde_json::from_str(stdout.trim())
            .unwrap_or_else(|err| panic!("{file}: {stdout}: {err}"));
        let text = std::fs::read_to_string(root.join(FIXTURE_DIR).join(file)).expect(file);
        let mut tree = Tree::from_tmd_str(&text).expect(file);
        tree.build_polygon_contents_for_oracle_tests().expect(file);

        assert_eq!(tree.is_feasible, as_bool(&record, "is_feasible"), "{file}");
        assert_eq!(
            tree.is_polygon_valid,
            as_bool(&record, "is_polygon_valid"),
            "{file}"
        );
        assert_eq!(
            tree.is_polygon_filled,
            as_bool(&record, "is_polygon_filled"),
            "{file}"
        );
        assert_eq!(tree.nodes.len(), as_usize(&record, "nodes"), "{file}");
        assert_eq!(tree.paths.len(), as_usize(&record, "paths"), "{file}");
        assert_eq!(tree.polys.len(), as_usize(&record, "polys"), "{file}");
        assert_eq!(
            tree.owned_polys.len(),
            as_usize(&record, "owned_polys"),
            "{file}"
        );
        assert_eq!(
            tree.owned_polys,
            as_usize_array(&record, "owned_poly_ids"),
            "{file}"
        );

        let summary = tree.summary();
        assert_eq!(
            summary.polygon_nodes,
            as_usize(&record, "polygon_nodes"),
            "{file}"
        );
        assert_eq!(
            summary.polygon_paths,
            as_usize(&record, "polygon_paths"),
            "{file}"
        );
        assert_eq!(
            summary.border_nodes,
            as_usize(&record, "border_nodes"),
            "{file}"
        );
        assert_eq!(
            summary.border_paths,
            as_usize(&record, "border_paths"),
            "{file}"
        );
        assert_eq!(
            summary.active_paths,
            as_usize(&record, "active_paths"),
            "{file}"
        );
        assert_eq!(
            summary.feasible_paths,
            as_usize(&record, "feasible_paths"),
            "{file}"
        );

        let polys = record["polys_detail"].as_array().expect("polys_detail");
        assert_eq!(tree.polys.len(), polys.len(), "{file}");
        for poly_record in polys {
            let index = as_usize(poly_record, "index");
            let poly = &tree.polys[index - 1];
            assert_eq!(
                poly.is_sub_poly,
                as_bool(poly_record, "is_sub_poly"),
                "{file}"
            );
            let (cx, cy) = as_point(&poly_record["centroid"]);
            approx_eq(poly.centroid.x, cx, 1.0e-7, file);
            approx_eq(poly.centroid.y, cy, 1.0e-7, file);
            assert_eq!(
                poly.ring_nodes,
                as_usize_array(poly_record, "ring_nodes"),
                "{file}"
            );
            assert_eq!(
                poly.ring_paths,
                as_usize_array(poly_record, "ring_paths"),
                "{file}"
            );
            assert_eq!(
                poly.cross_paths,
                as_usize_array(poly_record, "cross_paths"),
                "{file}"
            );
            assert_eq!(
                poly.inset_nodes,
                as_usize_array(poly_record, "inset_nodes"),
                "{file}"
            );
            assert_eq!(
                poly.spoke_paths,
                as_usize_array(poly_record, "spoke_paths"),
                "{file}"
            );
            assert_eq!(
                poly.ridge_path.unwrap_or(0),
                as_usize(poly_record, "ridge_path"),
                "{file}"
            );
            assert_eq!(
                poly.owned_nodes,
                as_usize_array(poly_record, "owned_nodes"),
                "{file}"
            );
            assert_eq!(
                poly.owned_paths,
                as_usize_array(poly_record, "owned_paths"),
                "{file}"
            );
            assert_eq!(
                poly.owned_polys,
                as_usize_array(poly_record, "owned_polys"),
                "{file}"
            );

            let node_locs = poly_record["node_locs"].as_array().expect("node_locs");
            assert_eq!(poly.node_locs.len(), node_locs.len(), "{file}");
            for (loc, expected) in poly.node_locs.iter().zip(node_locs) {
                let (x, y) = as_point(expected);
                approx_eq(loc.x, x, 1.0e-7, file);
                approx_eq(loc.y, y, 1.0e-7, file);
            }
        }

        let nodes = record["nodes_detail"].as_array().expect("nodes_detail");
        assert_eq!(tree.nodes.len(), nodes.len(), "{file}");
        for node_record in nodes {
            let index = as_usize(node_record, "index");
            let node = &tree.nodes[index - 1];
            let (x, y) = as_point(&node_record["loc"]);
            approx_eq(node.loc.x, x, 1.0e-7, file);
            approx_eq(node.loc.y, y, 1.0e-7, file);
            approx_eq(
                node.elevation,
                as_f64(node_record, "elevation"),
                1.0e-7,
                file,
            );
            assert_eq!(node.is_sub, as_bool(node_record, "is_sub"), "{file}");
            assert_eq!(
                node.is_junction,
                as_bool(node_record, "is_junction"),
                "{file}"
            );
            assert_eq!(
                node.leaf_paths,
                as_usize_array(node_record, "leaf_paths"),
                "{file}"
            );
        }

        let paths = record["paths_detail"].as_array().expect("paths_detail");
        assert_eq!(tree.paths.len(), paths.len(), "{file}");
        for path_record in paths {
            let index = as_usize(path_record, "index");
            let path = &tree.paths[index - 1];
            assert_eq!(path.nodes, as_usize_array(path_record, "nodes"), "{file}");
            approx_eq(
                path.min_tree_length,
                as_f64(path_record, "min_tree_length"),
                1.0e-7,
                file,
            );
            approx_eq(
                path.min_paper_length,
                as_f64(path_record, "min_paper_length"),
                1.0e-7,
                file,
            );
            approx_eq(
                path.act_tree_length,
                as_f64(path_record, "act_tree_length"),
                1.0e-7,
                file,
            );
            approx_eq(
                path.act_paper_length,
                as_f64(path_record, "act_paper_length"),
                1.0e-7,
                file,
            );
            assert_eq!(path.is_leaf, as_bool(path_record, "is_leaf"), "{file}");
            assert_eq!(path.is_sub, as_bool(path_record, "is_sub"), "{file}");
            assert_eq!(
                path.is_feasible,
                as_bool(path_record, "is_feasible"),
                "{file}"
            );
            assert_eq!(path.is_active, as_bool(path_record, "is_active"), "{file}");
            assert_eq!(path.is_border, as_bool(path_record, "is_border"), "{file}");
            assert_eq!(
                path.is_polygon,
                as_bool(path_record, "is_polygon"),
                "{file}"
            );
            assert_eq!(
                path.fwd_poly.unwrap_or(0),
                as_usize(path_record, "fwd_poly"),
                "{file}"
            );
            assert_eq!(
                path.bkd_poly.unwrap_or(0),
                as_usize(path_record, "bkd_poly"),
                "{file}"
            );
            assert_eq!(
                path.outset_path.unwrap_or(0),
                as_usize(path_record, "outset_path"),
                "{file}"
            );
            approx_eq(
                path.front_reduction,
                as_f64(path_record, "front_reduction"),
                1.0e-7,
                file,
            );
            approx_eq(
                path.back_reduction,
                as_f64(path_record, "back_reduction"),
                1.0e-7,
                file,
            );
        }
    }
}
