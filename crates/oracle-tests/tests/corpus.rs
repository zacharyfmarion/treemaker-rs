use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};
use treemaker_core::{CPStatus, Tree, TreeSummary};
use walkdir::WalkDir;

mod support;
use support::{oracle_binary, repo_root, run_oracle_json};

#[test]
fn external_corpus_roundtrips_and_optionally_matches_cpp_oracle() {
    let Some(mut corpus_dir) = env::var_os("TREEMAKER_CORPUS_DIR").map(PathBuf::from) else {
        eprintln!("skipping external corpus parity; set TREEMAKER_CORPUS_DIR to enable");
        return;
    };

    let root = repo_root();
    if corpus_dir.is_relative() {
        corpus_dir = root.join(corpus_dir);
    }
    let oracle = oracle_binary().map(|oracle| {
        if oracle.is_relative() {
            root.join(oracle)
        } else {
            oracle
        }
    });
    let paths = corpus_paths(&corpus_dir);
    assert!(
        !paths.is_empty(),
        "no .tmd/.tmd4/.tmd5 files under {}",
        corpus_dir.display()
    );

    let mut hashes = HashSet::new();
    let mut unique = 0usize;
    let mut duplicates = 0usize;
    let mut oracle_checked = 0usize;

    for path in paths {
        let bytes = fs::read(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        let hash = format!("{:x}", Sha256::digest(&bytes));
        if !hashes.insert(hash) {
            duplicates += 1;
            continue;
        }
        unique += 1;

        let text =
            String::from_utf8(bytes).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        let tree =
            Tree::from_tmd_str(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        let summary = tree.summary();
        let roundtrip = Tree::from_tmd_str(&tree.to_tmd5_string())
            .unwrap_or_else(|err| panic!("{} roundtrip: {err}", path.display()));
        assert_summary_matches(&summary, &roundtrip.summary(), &path);

        if let Some(oracle) = &oracle {
            let record = run_oracle_json(oracle, &root, "summary", &path);
            assert_oracle_summary(&summary, &record, &path);
            oracle_checked += 1;
        }
    }

    eprintln!(
        "corpus {}: unique={unique}, duplicates={duplicates}, oracle_checked={oracle_checked}",
        corpus_dir.display()
    );
}

fn corpus_paths(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(dir).follow_links(false) {
        let entry = entry.unwrap_or_else(|err| panic!("{}: {err}", dir.display()));
        if entry.file_type().is_file() && is_treemaker_file(entry.path()) {
            paths.push(entry.into_path());
        }
    }
    paths.sort();
    paths
}

fn is_treemaker_file(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return false;
    };
    matches!(&ext.to_ascii_lowercase()[..], "tmd" | "tmd4" | "tmd5")
}

fn assert_summary_matches(actual: &TreeSummary, expected: &TreeSummary, path: &Path) {
    assert_float(
        actual.paper_width,
        expected.paper_width,
        path,
        "paper_width",
    );
    assert_float(
        actual.paper_height,
        expected.paper_height,
        path,
        "paper_height",
    );
    assert_float(actual.scale, expected.scale, path, "scale");
    assert_eq!(
        actual.has_symmetry,
        expected.has_symmetry,
        "{}",
        path.display()
    );
    assert_eq!(
        actual.is_feasible,
        expected.is_feasible,
        "{}",
        path.display()
    );
    assert_eq!(actual.cp_status, expected.cp_status, "{}", path.display());
    assert_eq!(actual.nodes, expected.nodes, "{}", path.display());
    assert_eq!(actual.edges, expected.edges, "{}", path.display());
    assert_eq!(actual.paths, expected.paths, "{}", path.display());
    assert_eq!(actual.polys, expected.polys, "{}", path.display());
    assert_eq!(actual.vertices, expected.vertices, "{}", path.display());
    assert_eq!(actual.creases, expected.creases, "{}", path.display());
    assert_eq!(actual.facets, expected.facets, "{}", path.display());
    assert_eq!(actual.conditions, expected.conditions, "{}", path.display());
    assert_eq!(actual.leaf_nodes, expected.leaf_nodes, "{}", path.display());
    assert_eq!(actual.leaf_paths, expected.leaf_paths, "{}", path.display());
    assert_eq!(
        actual.feasible_paths,
        expected.feasible_paths,
        "{}",
        path.display()
    );
    assert_eq!(
        actual.active_paths,
        expected.active_paths,
        "{}",
        path.display()
    );
    assert_eq!(
        actual.border_nodes,
        expected.border_nodes,
        "{}",
        path.display()
    );
    assert_eq!(
        actual.border_paths,
        expected.border_paths,
        "{}",
        path.display()
    );
    assert_eq!(
        actual.polygon_nodes,
        expected.polygon_nodes,
        "{}",
        path.display()
    );
    assert_eq!(
        actual.polygon_paths,
        expected.polygon_paths,
        "{}",
        path.display()
    );
    assert_eq!(
        actual.pinned_nodes,
        expected.pinned_nodes,
        "{}",
        path.display()
    );
    assert_eq!(
        actual.pinned_edges,
        expected.pinned_edges,
        "{}",
        path.display()
    );
    assert_eq!(
        actual.conditioned_nodes,
        expected.conditioned_nodes,
        "{}",
        path.display()
    );
    assert_eq!(
        actual.conditioned_edges,
        expected.conditioned_edges,
        "{}",
        path.display()
    );
    assert_eq!(
        actual.conditioned_paths,
        expected.conditioned_paths,
        "{}",
        path.display()
    );
    assert_eq!(
        actual.conditions_by_tag,
        expected.conditions_by_tag,
        "{}",
        path.display()
    );
}

fn assert_oracle_summary(summary: &TreeSummary, record: &Value, path: &Path) {
    assert_oracle_float(summary.paper_width, record, "paper_width", path);
    assert_oracle_float(summary.paper_height, record, "paper_height", path);
    assert_oracle_float(summary.scale, record, "scale", path);
    assert_eq!(
        summary.has_symmetry,
        record["has_symmetry"].as_bool().expect("has_symmetry"),
        "{}",
        path.display()
    );
    assert_eq!(
        summary.is_feasible,
        record["is_feasible"].as_bool().expect("is_feasible"),
        "{}",
        path.display()
    );
    assert_eq!(
        cp_status_oracle_name(&summary.cp_status),
        record["cp_status"].as_str().expect("cp_status"),
        "{}",
        path.display()
    );
    for (key, actual) in [
        ("nodes", summary.nodes),
        ("edges", summary.edges),
        ("paths", summary.paths),
        ("polys", summary.polys),
        ("vertices", summary.vertices),
        ("creases", summary.creases),
        ("facets", summary.facets),
        ("conditions", summary.conditions),
        ("leaf_nodes", summary.leaf_nodes),
        ("leaf_paths", summary.leaf_paths),
        ("feasible_paths", summary.feasible_paths),
        ("active_paths", summary.active_paths),
        ("border_nodes", summary.border_nodes),
        ("border_paths", summary.border_paths),
        ("polygon_nodes", summary.polygon_nodes),
        ("polygon_paths", summary.polygon_paths),
        ("pinned_nodes", summary.pinned_nodes),
        ("pinned_edges", summary.pinned_edges),
        ("conditioned_nodes", summary.conditioned_nodes),
        ("conditioned_edges", summary.conditioned_edges),
        ("conditioned_paths", summary.conditioned_paths),
    ] {
        assert_eq!(
            actual as u64,
            record[key].as_u64().expect(key),
            "{} {key}",
            path.display()
        );
    }
}

fn assert_float(actual: f64, expected: f64, path: &Path, key: &str) {
    assert!(
        (actual - expected).abs() <= 1.0e-9,
        "{} {key}: actual {actual:.10}, expected {expected:.10}",
        path.display()
    );
}

fn assert_oracle_float(actual: f64, record: &Value, key: &str, path: &Path) {
    let expected = record[key].as_f64().expect(key);
    assert!(
        (actual - expected).abs() <= 1.0e-7,
        "{} {key}: rust {actual:.10}, oracle {expected:.10}",
        path.display()
    );
}

fn cp_status_oracle_name(status: &CPStatus) -> &'static str {
    match status {
        CPStatus::HasFullCp => "HAS_FULL_CP",
        CPStatus::EdgesTooShort => "EDGES_TOO_SHORT",
        CPStatus::PolysNotValid => "POLYS_NOT_VALID",
        CPStatus::PolysNotFilled => "POLYS_NOT_FILLED",
        CPStatus::PolysMultipleIbps => "POLYS_MULTIPLE_IBPS",
        CPStatus::VerticesLackDepth => "VERTICES_LACK_DEPTH",
        CPStatus::FacetsNotValid => "FACETS_NOT_VALID",
        CPStatus::NotLocalRootConnectable => "NOT_LOCAL_ROOT_CONNECTABLE",
    }
}
