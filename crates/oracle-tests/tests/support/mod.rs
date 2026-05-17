use std::env;
use std::path::{Path, PathBuf};

use serde_json::Value;
use treemaker_core::{CPStatus, OwnerRef, Tree};

pub const FIXTURE_DIR: &str = "tests/fixtures";

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

pub fn oracle_binary() -> Option<PathBuf> {
    env::var_os("TREEMAKER_CPP_ORACLE").map(PathBuf::from)
}

pub fn approx_eq(actual: f64, expected: f64, tol: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: actual {actual:.10}, expected {expected:.10}, tol {tol}"
    );
}

pub fn as_usize(record: &Value, key: &str) -> usize {
    record[key].as_u64().expect(key) as usize
}

pub fn as_f64(record: &Value, key: &str) -> f64 {
    record[key].as_f64().expect(key)
}

pub fn as_bool(record: &Value, key: &str) -> bool {
    record[key].as_bool().expect(key)
}

pub fn as_usize_array(record: &Value, key: &str) -> Vec<usize> {
    record[key]
        .as_array()
        .expect(key)
        .iter()
        .map(|value| value.as_u64().expect(key) as usize)
        .collect()
}

pub fn as_point(value: &Value) -> (f64, f64) {
    let array = value.as_array().expect("point");
    (
        array[0].as_f64().expect("point x"),
        array[1].as_f64().expect("point y"),
    )
}

pub fn assert_owner_eq(actual: &OwnerRef, expected: &Value, file: &str) {
    let kind = expected["kind"].as_str().expect("owner kind");
    let index = expected["index"].as_u64().expect("owner index") as usize;
    match (actual, kind, index) {
        (OwnerRef::Tree, "tree", 0)
        | (OwnerRef::Node(_), "node", _)
        | (OwnerRef::Path(_), "path", _)
        | (OwnerRef::Poly(_), "poly", _) => {}
        _ => panic!("{file}: owner kind mismatch: actual {actual:?}, expected {expected:?}"),
    }
    match actual {
        OwnerRef::Tree => assert_eq!(index, 0, "{file}"),
        OwnerRef::Node(id) | OwnerRef::Path(id) | OwnerRef::Poly(id) => {
            assert_eq!(*id, index, "{file}")
        }
    }
}

pub fn oracle_cp_status(status: &str) -> CPStatus {
    match status {
        "HAS_FULL_CP" => CPStatus::HasFullCp,
        "EDGES_TOO_SHORT" => CPStatus::EdgesTooShort,
        "POLYS_NOT_VALID" => CPStatus::PolysNotValid,
        "POLYS_NOT_FILLED" => CPStatus::PolysNotFilled,
        "POLYS_MULTIPLE_IBPS" => CPStatus::PolysMultipleIbps,
        "VERTICES_LACK_DEPTH" => CPStatus::VerticesLackDepth,
        "FACETS_NOT_VALID" => CPStatus::FacetsNotValid,
        "NOT_LOCAL_ROOT_CONNECTABLE" => CPStatus::NotLocalRootConnectable,
        other => panic!("unknown CP status {other}"),
    }
}

pub fn weighted_rms_strain_percent(tree: &Tree) -> f64 {
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
