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
