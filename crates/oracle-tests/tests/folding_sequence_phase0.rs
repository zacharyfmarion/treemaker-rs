use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use treemaker_flatfold::{SolutionLimit, SolveOptions, solve_flat_fold};
use treemaker_fold::{FoldDocument, validate_basic};

mod support;
use support::repo_root;

#[derive(Debug, Deserialize)]
struct Manifest {
    schema_version: u32,
    phase: String,
    not_implemented_policy: String,
    fixtures: Vec<Fixture>,
}

#[derive(Debug, Deserialize)]
struct Fixture {
    id: String,
    file: String,
    expected_artifact: String,
    category: String,
    expected_v1_status: String,
    target_state: String,
    visual_checks: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ExpectedArtifact {
    schema_version: u32,
    fixture_id: String,
    status: String,
    expected_v1_status: String,
    steps: Vec<Value>,
    states: Vec<Value>,
    diagnostics: Vec<Diagnostic>,
    unresolved_regions: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct Diagnostic {
    severity: String,
    code: String,
    message: String,
}

#[test]
fn phase0_manifest_is_complete_and_reviewable() {
    let root = fixture_root();
    let manifest = read_manifest(&root);
    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.phase, "phase0");
    assert!(
        manifest.not_implemented_policy.contains("not_implemented"),
        "manifest should document the not implemented policy"
    );
    assert!(
        manifest.fixtures.len() >= 6,
        "Phase 0 should include the canonical fixture categories"
    );

    let mut ids = BTreeSet::new();
    for fixture in &manifest.fixtures {
        assert!(ids.insert(fixture.id.as_str()), "duplicate fixture id");
        assert!(
            matches!(
                fixture.category.as_str(),
                "simple_fold"
                    | "multi_layer_simple_fold"
                    | "complex_local_move"
                    | "treemaker_base_proxy"
                    | "unsupported_simultaneous_collapse"
            ),
            "{}: unexpected fixture category {}",
            fixture.id,
            fixture.category
        );
        assert!(
            matches!(
                fixture.expected_v1_status.as_str(),
                "complete" | "partial" | "unsupported"
            ),
            "{}: unexpected expected_v1_status {}",
            fixture.id,
            fixture.expected_v1_status
        );
        assert!(
            fixture.visual_checks.len() >= 3,
            "{}: fixtures should include concrete visual checks",
            fixture.id
        );
        assert!(
            root.join(&fixture.file).exists(),
            "{} missing FOLD file",
            fixture.id
        );
        assert!(
            root.join(&fixture.expected_artifact).exists(),
            "{} missing expected artifact",
            fixture.id
        );
    }
}

#[test]
fn phase0_fold_fixtures_parse_and_solve_target_states() {
    let root = fixture_root();
    let manifest = read_manifest(&root);

    for fixture in &manifest.fixtures {
        let fold = read_fold(&root.join(&fixture.file));
        validate_basic(&fold).unwrap_or_else(|err| panic!("{}: {err}", fixture.id));
        assert!(
            fold.frame_classes
                .iter()
                .any(|class| class == "creasePattern"),
            "{}: expected creasePattern frame class",
            fixture.id
        );
        assert!(
            !fold.faces_vertices.is_empty(),
            "{}: expected explicit faces for visual review",
            fixture.id
        );
        assert_eq!(
            fold.edges_assignment.len(),
            fold.edges_vertices.len(),
            "{}: assignments should cover every edge",
            fixture.id
        );

        if fixture.target_state == "flat_foldable" {
            let solved = solve_flat_fold(
                &fold,
                SolveOptions {
                    solution_limit: SolutionLimit::Count(10),
                    ..SolveOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("{}: flat-fold solve failed: {err}", fixture.id));
            assert_eq!(
                solved.analysis.normalized.document.faces_vertices.len(),
                solved.analysis.faces_flip.len(),
                "{}: each normalized face should have a flip flag",
                fixture.id
            );
            assert!(
                solved.analysis.overlap.is_some(),
                "{}: target-state analysis should include overlap graph",
                fixture.id
            );
            assert!(
                !solved.states.is_empty(),
                "{}: solver should report at least one state marker",
                fixture.id
            );
        } else {
            panic!(
                "{}: unsupported Phase 0 target_state {}",
                fixture.id, fixture.target_state
            );
        }
    }
}

#[test]
fn phase0_expected_artifacts_are_explicitly_not_implemented() {
    let root = fixture_root();
    let manifest = read_manifest(&root);

    for fixture in &manifest.fixtures {
        let artifact = read_expected_artifact(&root.join(&fixture.expected_artifact));
        assert_eq!(artifact.schema_version, 1, "{}", fixture.id);
        assert_eq!(artifact.fixture_id, fixture.id, "{}", fixture.id);
        assert_eq!(
            artifact.expected_v1_status, fixture.expected_v1_status,
            "{}",
            fixture.id
        );
        assert_eq!(
            artifact.status, "not_implemented",
            "{}: Phase 0 must not include fake planner output",
            fixture.id
        );
        assert!(
            artifact.steps.is_empty(),
            "{}: not_implemented artifacts should not include steps",
            fixture.id
        );
        assert!(
            artifact.states.is_empty(),
            "{}: not_implemented artifacts should not include state frames",
            fixture.id
        );
        assert!(
            artifact.unresolved_regions.is_empty(),
            "{}: unresolved regions are planner output and should wait",
            fixture.id
        );
        assert!(
            artifact.diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == "info"
                    && diagnostic.code == "not_implemented"
                    && diagnostic.message.contains("not implemented")
            }),
            "{}: expected a clear not_implemented diagnostic",
            fixture.id
        );
    }
}

fn fixture_root() -> PathBuf {
    repo_root().join("tests/fixtures/folding-sequence")
}

fn read_manifest(root: &Path) -> Manifest {
    let path = root.join("manifest.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

fn read_fold(path: &Path) -> FoldDocument {
    let text = fs::read_to_string(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

fn read_expected_artifact(path: &Path) -> ExpectedArtifact {
    let text = fs::read_to_string(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}
