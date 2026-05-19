use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use sha2::{Digest, Sha256};
use treemaker_flatfold::{NormalizeOptions, normalize_fold};
use treemaker_fold::{Assignment, FoldDocument};

mod support;
use support::repo_root;

#[test]
fn flat_folder_normalization_matches_js_oracle_when_enabled() {
    let Some(mut oracle) = env::var_os("FLATFOLDER_ORACLE").map(PathBuf::from) else {
        eprintln!("skipping Flat-Folder oracle parity; set FLATFOLDER_ORACLE to enable");
        return;
    };
    let root = repo_root();
    if oracle.is_relative() {
        oracle = root.join(oracle);
    }
    for fixture in ["kabuto.fold", "bad_twist.fold"] {
        let path = root.join("tests/fixtures/flat-folder").join(fixture);
        let record = run_flat_folder_oracle(&oracle, &root, "normalize", &path);
        assert_eq!(record["status"].as_str(), Some("ok"), "{fixture}");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        let document: FoldDocument =
            serde_json::from_str(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        let normalized = normalize_fold(&document, NormalizeOptions::default())
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        assert_normalize_record(
            &normalized.document,
            &normalized.vertex_vertices,
            &record,
            fixture,
        );
    }
}

fn run_flat_folder_oracle(oracle: &Path, root: &Path, command: &str, file: &Path) -> Value {
    let output = Command::new(oracle)
        .current_dir(root)
        .arg(command)
        .arg(file)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", oracle.display()));
    assert!(
        output.status.success(),
        "oracle failed for {}\nstdout:\n{}\nstderr:\n{}",
        file.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("oracle stdout utf8");
    serde_json::from_str(stdout.trim()).unwrap_or_else(|err| panic!("{stdout}: {err}"))
}

fn assert_normalize_record(
    document: &FoldDocument,
    vertex_vertices: &[Vec<usize>],
    record: &Value,
    fixture: &str,
) {
    let normalize = &record["normalize"];
    assert_eq!(
        document.vertices_coords.len(),
        normalize["vertices"].as_u64().expect("vertices") as usize,
        "{fixture}"
    );
    assert_eq!(
        document.edges_vertices.len(),
        normalize["edges"].as_u64().expect("edges") as usize,
        "{fixture}"
    );
    assert_eq!(
        document.faces_vertices.len(),
        normalize["faces"].as_u64().expect("faces") as usize,
        "{fixture}"
    );
    assert_eq!(
        assignment_counts(&document.edges_assignment),
        normalize["assignments"],
        "{fixture}"
    );
    assert_hash(
        &document.edges_vertices,
        normalize,
        "edges_vertices_hash",
        fixture,
    );
    assert_hash(
        &document.edges_faces,
        normalize,
        "edges_faces_hash",
        fixture,
    );
    assert_hash(
        &document.faces_vertices,
        normalize,
        "faces_vertices_hash",
        fixture,
    );
    assert_hash(
        &document.faces_edges,
        normalize,
        "faces_edges_hash",
        fixture,
    );
    assert_hash(vertex_vertices, normalize, "vertex_vertices_hash", fixture);
}

fn assignment_counts(assignments: &[Assignment]) -> Value {
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for assignment in assignments {
        *counts.entry(assignment.as_str()).or_default() += 1;
    }
    serde_json::to_value(counts).expect("assignment counts json")
}

fn assert_hash<T: serde::Serialize + ?Sized>(
    actual: &T,
    normalize: &Value,
    key: &str,
    fixture: &str,
) {
    let actual_hash = sha256_json(actual);
    let expected = normalize[key].as_str().expect(key);
    assert_eq!(actual_hash, expected, "{fixture} {key}");
}

fn sha256_json<T: serde::Serialize + ?Sized>(value: &T) -> String {
    let json = serde_json::to_string(value).expect("hash json");
    format!("{:x}", Sha256::digest(json.as_bytes()))
}
