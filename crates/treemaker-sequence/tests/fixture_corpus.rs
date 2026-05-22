use std::path::{Path, PathBuf};

use serde::Deserialize;
use treemaker_fold::FoldDocument;
use treemaker_sequence::{
    PlanStatus, SequenceState, TargetStateOptions, plan_folding_sequence, resolve_target_state,
};

#[derive(Debug, Deserialize)]
struct Manifest {
    fixtures: Vec<Fixture>,
}

#[derive(Debug, Deserialize)]
struct Fixture {
    id: String,
    file: String,
    expected_v1_status: String,
    target_state: String,
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

fn read_fold(path: impl AsRef<Path>) -> FoldDocument {
    let path = path.as_ref();
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn phase0_manifest_expected_v1_status_matches_planner() {
    let root = repo_root().join("tests/fixtures/folding-sequence");
    let manifest_text = std::fs::read_to_string(root.join("manifest.json")).expect("manifest.json");
    let manifest: Manifest = serde_json::from_str(&manifest_text).expect("manifest");

    for fixture in manifest.fixtures {
        if fixture.target_state != "flat_foldable" {
            continue;
        }
        let document = read_fold(root.join(&fixture.file));
        let target = resolve_target_state(&document, TargetStateOptions::default())
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.id));
        let plan = plan_folding_sequence(&target)
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.id));

        assert_eq!(
            plan.status,
            expected_status(&fixture.expected_v1_status),
            "{}",
            fixture.id
        );
        for state in &plan.states {
            SequenceState::validate(state).unwrap_or_else(|error| {
                panic!(
                    "{} state {} did not validate: {error}",
                    fixture.id, state.id
                )
            });
        }
    }
}

#[test]
fn checked_in_flat_folder_fixtures_are_planner_smoke_tested() {
    let root = repo_root().join("tests/fixtures/flat-folder");
    let mut checked = 0usize;
    let mut rejected = 0usize;
    for path in fold_files_under(&root) {
        let document = read_fold(&path);
        let Ok(target) = resolve_target_state(&document, TargetStateOptions::default()) else {
            rejected += 1;
            continue;
        };
        let plan = plan_folding_sequence(&target)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));

        assert!(
            matches!(
                plan.status,
                PlanStatus::Complete | PlanStatus::Partial | PlanStatus::Unsupported
            ),
            "{}",
            path.display()
        );
        checked += 1;
    }
    assert!(
        checked >= 1,
        "expected at least one solvable flat-folder smoke fixture"
    );
    assert!(
        checked + rejected >= 2,
        "expected checked-in flat-folder smoke fixtures"
    );
}

#[test]
fn optional_external_oriedita_fold_corpus_smoke_test() {
    let Some(root) = std::env::var_os("ORIEDITA_FOLD_CORPUS").map(PathBuf::from) else {
        eprintln!("skipping Oriedita/FOLD corpus smoke; set ORIEDITA_FOLD_CORPUS to enable");
        return;
    };
    let mut checked = 0usize;
    for path in fold_files_under(&root) {
        let document = read_fold(&path);
        let target = resolve_target_state(&document, TargetStateOptions::default())
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let _plan = plan_folding_sequence(&target)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        checked += 1;
    }
    assert!(checked > 0, "external corpus did not contain .fold files");
}

fn expected_status(raw: &str) -> PlanStatus {
    match raw {
        "complete" => PlanStatus::Complete,
        "partial" => PlanStatus::Partial,
        "unsupported" => PlanStatus::Unsupported,
        other => panic!("unsupported expected_v1_status {other}"),
    }
}

fn fold_files_under(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_fold_files(root, &mut out);
    out.sort();
    out
}

fn collect_fold_files(path: &Path, out: &mut Vec<PathBuf>) {
    let Ok(metadata) = std::fs::metadata(path) else {
        return;
    };
    if metadata.is_file() {
        if path.extension().and_then(|extension| extension.to_str()) == Some("fold") {
            out.push(path.to_path_buf());
        }
        return;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect_fold_files(&entry.path(), out);
    }
}
