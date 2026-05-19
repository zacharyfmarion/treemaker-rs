use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde_json::Value;
use sha2::{Digest, Sha256};
use treemaker_flatfold::{
    ConstraintSummary, FlatFoldError, SolutionLimit, SolveOptions, SolveResult, solve_flat_fold,
};
use treemaker_fold::FoldDocument;
use walkdir::WalkDir;

mod support;
use support::repo_root;

const LOCAL_CORPUS_DEFAULT: &str = "/Users/zacharymarion/Documents/datasets/create-pattern-detector/synthetic/cp_training_mix_v1/folds";

#[test]
fn flat_folder_external_corpus_matches_js_oracle_when_enabled() {
    let Some(mut oracle) = env::var_os("FLATFOLDER_ORACLE").map(PathBuf::from) else {
        eprintln!("skipping Flat-Folder corpus parity; set FLATFOLDER_ORACLE to enable");
        return;
    };
    let Some(corpus_dir) = corpus_dir() else {
        eprintln!(
            "skipping Flat-Folder corpus parity; set FLATFOLDER_CORPUS_DIR or create {LOCAL_CORPUS_DEFAULT}"
        );
        return;
    };
    let root = repo_root();
    if oracle.is_relative() {
        oracle = root.join(oracle);
    }
    let limit = solve_limit();
    let limit_arg = limit_arg(&limit);
    let started = Instant::now();
    let mut scan = scan_corpus(&corpus_dir);
    let unique_hashes = scan.cases.len();
    if let Some(max_cases) = max_cases() {
        scan.cases.truncate(max_cases);
    }
    let mut stats = CorpusStats {
        total_symlinks: scan.total_symlinks,
        total_fold_entries: scan.total_fold_entries,
        unique_hashes,
        duplicates: scan.duplicates,
        ..CorpusStats::default()
    };
    let mut mismatches = Vec::new();

    for (index, case) in scan.cases.iter().enumerate() {
        if index > 0 && index % 100 == 0 {
            eprintln!(
                "flat-folder corpus: checked {}/{} unique cases",
                index,
                scan.cases.len()
            );
        }
        let text = std::fs::read_to_string(&case.path)
            .unwrap_or_else(|err| panic!("{}: {err}", case.path.display()));
        let rust = run_rust_solver(&text, &limit);
        let oracle_record = run_flat_folder_oracle(&oracle, &root, "solve", &case.path, &limit_arg);
        let oracle_status = oracle_record["status"]
            .as_str()
            .unwrap_or("oracle-error")
            .to_string();
        stats.bump(&oracle_status);
        if rust.status != oracle_status {
            mismatches.push(format!(
                "{}: status rust={} oracle={}",
                case.id.display(),
                rust.status,
                oracle_status
            ));
            continue;
        }
        if let Some(solved) = rust.solved.as_ref()
            && oracle_status == "ok"
        {
            compare_ok_case(&case.id, solved, &oracle_record, &mut mismatches);
        }
    }

    eprintln!(
        "flat-folder corpus {}: fold_entries={} symlinks={} unique_hashes={} checked={} duplicates={} solved={} assignment_conflicts={} precision_failures={} invalid_inputs={} unsatisfied_components={} oracle_errors={} unimplemented={} elapsed={:.2?}",
        corpus_dir.display(),
        stats.total_fold_entries,
        stats.total_symlinks,
        stats.unique_hashes,
        scan.cases.len(),
        stats.duplicates,
        stats.solved,
        stats.assignment_conflicts,
        stats.precision_failures,
        stats.invalid_inputs,
        stats.unsatisfied_components,
        stats.oracle_errors,
        stats.unimplemented,
        started.elapsed()
    );

    if !mismatches.is_empty() {
        let shown = mismatches.iter().take(20).cloned().collect::<Vec<_>>();
        panic!(
            "Flat-Folder corpus mismatches: {} total\n{}",
            mismatches.len(),
            shown.join("\n")
        );
    }
}

fn corpus_dir() -> Option<PathBuf> {
    if let Some(raw) = env::var_os("FLATFOLDER_CORPUS_DIR") {
        return Some(PathBuf::from(raw));
    }
    let local = PathBuf::from(LOCAL_CORPUS_DEFAULT);
    local.exists().then_some(local)
}

fn solve_limit() -> SolutionLimit {
    match env::var("FLATFOLDER_SOLVE_LIMIT").as_deref() {
        Ok("all") => SolutionLimit::All,
        Ok("1") => SolutionLimit::Count(1),
        Ok("10") | Err(_) => SolutionLimit::Count(10),
        Ok("100") => SolutionLimit::Count(100),
        Ok("1000") => SolutionLimit::Count(1000),
        Ok(other) => panic!("FLATFOLDER_SOLVE_LIMIT must be all|1|10|100|1000, got {other}"),
    }
}

fn limit_arg(limit: &SolutionLimit) -> String {
    match limit {
        SolutionLimit::All => "all".to_string(),
        SolutionLimit::Count(count) => count.to_string(),
    }
}

fn max_cases() -> Option<usize> {
    env::var("FLATFOLDER_CORPUS_MAX_CASES").ok().map(|raw| {
        raw.parse::<usize>()
            .unwrap_or_else(|err| panic!("FLATFOLDER_CORPUS_MAX_CASES={raw}: {err}"))
    })
}

#[derive(Debug)]
struct CorpusScan {
    total_fold_entries: usize,
    total_symlinks: usize,
    duplicates: usize,
    cases: Vec<CorpusCase>,
}

#[derive(Debug)]
struct CorpusCase {
    id: PathBuf,
    path: PathBuf,
}

fn scan_corpus(root: &Path) -> CorpusScan {
    let mut total_fold_entries = 0usize;
    let mut total_symlinks = 0usize;
    let mut duplicates = 0usize;
    let mut seen_hashes = BTreeSet::<String>::new();
    let mut cases = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(true)
        .sort_by_file_name()
        .into_iter()
    {
        let entry = entry.unwrap_or_else(|err| panic!("walk {}: {err}", root.display()));
        if !entry.file_type().is_file() || entry.path().extension() != Some(OsStr::new("fold")) {
            continue;
        }
        total_fold_entries += 1;
        if std::fs::symlink_metadata(entry.path())
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
        {
            total_symlinks += 1;
        }
        let bytes = std::fs::read(entry.path())
            .unwrap_or_else(|err| panic!("{}: {err}", entry.path().display()));
        let hash = format!("{:x}", Sha256::digest(&bytes));
        if !seen_hashes.insert(hash) {
            duplicates += 1;
            continue;
        }
        cases.push(CorpusCase {
            id: entry.path().to_path_buf(),
            path: entry.path().to_path_buf(),
        });
    }

    CorpusScan {
        total_fold_entries,
        total_symlinks,
        duplicates,
        cases,
    }
}

#[derive(Debug)]
struct RustRun {
    status: String,
    solved: Option<SolveResult>,
}

fn run_rust_solver(text: &str, limit: &SolutionLimit) -> RustRun {
    let document = match serde_json::from_str::<FoldDocument>(text) {
        Ok(document) => document,
        Err(error) => {
            return RustRun {
                status: "invalid-input".to_string(),
                solved: None,
            }
            .with_context(error.to_string());
        }
    };
    match solve_flat_fold(
        &document,
        SolveOptions {
            solution_limit: limit.clone(),
            ..SolveOptions::default()
        },
    ) {
        Ok(solved) => RustRun {
            status: "ok".to_string(),
            solved: Some(solved),
        },
        Err(error) => RustRun {
            status: rust_status(&error).to_string(),
            solved: None,
        }
        .with_context(error.to_string()),
    }
}

impl RustRun {
    fn with_context(self, _context: String) -> Self {
        self
    }
}

fn rust_status(error: &FlatFoldError) -> &'static str {
    match error {
        FlatFoldError::InvalidInput(_) => "invalid-input",
        FlatFoldError::PrecisionFailure(_) => "precision-failure",
        FlatFoldError::AssignmentConflict(_) => "assignment-conflict",
        FlatFoldError::UnsatisfiedComponent(_) => "unsatisfied-component",
        FlatFoldError::Unimplemented(_) => "unimplemented",
    }
}

fn compare_ok_case(
    case_id: &Path,
    solved: &SolveResult,
    oracle: &Value,
    mismatches: &mut Vec<String>,
) {
    compare_constraints(
        case_id,
        &solved.constraints,
        &oracle["constraints"],
        mismatches,
    );
    let solve = &oracle["solve"];
    compare_usize_array(
        case_id,
        "component_sizes",
        &solved.component_sizes,
        solve,
        mismatches,
    );
    compare_usize_array(
        case_id,
        "solution_counts",
        &solved.solution_counts,
        solve,
        mismatches,
    );
    if solved.states != solve["states"].as_str().unwrap_or_default() {
        mismatches.push(format!(
            "{}: states rust={} oracle={}",
            case_id.display(),
            solved.states,
            solve["states"].as_str().unwrap_or_default()
        ));
    }
    if solved.face_orders.len() != solve["face_orders"].as_u64().expect("face_orders") as usize {
        mismatches.push(format!(
            "{}: face_orders rust={} oracle={}",
            case_id.display(),
            solved.face_orders.len(),
            solve["face_orders"].as_u64().expect("face_orders")
        ));
    }
    let rust_hash = sha256_json(&solved.face_orders);
    let oracle_hash = solve["face_orders_hash"]
        .as_str()
        .expect("face_orders_hash");
    if rust_hash != oracle_hash {
        let first_diff = solve["face_orders_data"].as_array().and_then(|expected| {
            solved
                .face_orders
                .iter()
                .zip(expected)
                .enumerate()
                .find(|(_, (actual, expected))| {
                    serde_json::to_value(actual).expect("face order json") != **expected
                })
                .map(|(index, (actual, expected))| {
                    format!(" diff_at={index} rust={actual:?} oracle={expected}")
                })
        });
        mismatches.push(format!(
            "{}: face_orders_hash rust={} oracle={} first_rust={:?}{}",
            case_id.display(),
            rust_hash,
            oracle_hash,
            solved.face_orders.iter().take(20).collect::<Vec<_>>(),
            first_diff.unwrap_or_default()
        ));
    }
}

fn compare_constraints(
    case_id: &Path,
    actual: &ConstraintSummary,
    expected: &Value,
    mismatches: &mut Vec<String>,
) {
    compare_count(
        case_id,
        "variables",
        actual.variables,
        expected["variables"].as_u64().expect("variables") as usize,
        mismatches,
    );
    compare_count(
        case_id,
        "taco_taco",
        actual.taco_taco,
        expected["taco_taco"].as_u64().expect("taco_taco") as usize,
        mismatches,
    );
    compare_count(
        case_id,
        "taco_tortilla",
        actual.taco_tortilla,
        expected["taco_tortilla"].as_u64().expect("taco_tortilla") as usize,
        mismatches,
    );
    compare_count(
        case_id,
        "tortilla_tortilla",
        actual.tortilla_tortilla,
        expected["tortilla_tortilla"]
            .as_u64()
            .expect("tortilla_tortilla") as usize,
        mismatches,
    );
    compare_count(
        case_id,
        "transitivity",
        actual.transitivity,
        expected["transitivity"].as_u64().expect("transitivity") as usize,
        mismatches,
    );
    compare_count(
        case_id,
        "reduced_transitivity",
        actual.reduced_transitivity,
        expected["reduced_transitivity"]
            .as_u64()
            .expect("reduced_transitivity") as usize,
        mismatches,
    );
}

fn compare_usize_array(
    case_id: &Path,
    key: &str,
    actual: &[usize],
    record: &Value,
    mismatches: &mut Vec<String>,
) {
    let expected = record[key]
        .as_array()
        .expect(key)
        .iter()
        .map(|value| value.as_u64().expect(key) as usize)
        .collect::<Vec<_>>();
    if actual != expected {
        mismatches.push(format!(
            "{}: {key} rust={actual:?} oracle={expected:?}",
            case_id.display()
        ));
    }
}

fn compare_count(
    case_id: &Path,
    key: &str,
    actual: usize,
    expected: usize,
    mismatches: &mut Vec<String>,
) {
    if actual != expected {
        mismatches.push(format!(
            "{}: {key} rust={actual} oracle={expected}",
            case_id.display()
        ));
    }
}

fn run_flat_folder_oracle(
    oracle: &Path,
    root: &Path,
    command: &str,
    file: &Path,
    limit: &str,
) -> Value {
    let output = Command::new(oracle)
        .current_dir(root)
        .arg(command)
        .arg(file)
        .arg("--limit")
        .arg(limit)
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

fn sha256_json<T: serde::Serialize + ?Sized>(value: &T) -> String {
    let json = serde_json::to_string(value).expect("hash json");
    format!("{:x}", Sha256::digest(json.as_bytes()))
}

#[derive(Debug, Default)]
struct CorpusStats {
    total_fold_entries: usize,
    total_symlinks: usize,
    unique_hashes: usize,
    duplicates: usize,
    solved: usize,
    assignment_conflicts: usize,
    precision_failures: usize,
    invalid_inputs: usize,
    unsatisfied_components: usize,
    oracle_errors: usize,
    unimplemented: usize,
}

impl CorpusStats {
    fn bump(&mut self, status: &str) {
        match status {
            "ok" => self.solved += 1,
            "assignment-conflict" => self.assignment_conflicts += 1,
            "precision-failure" => self.precision_failures += 1,
            "invalid-input" => self.invalid_inputs += 1,
            "unsatisfied-component" => self.unsatisfied_components += 1,
            "unimplemented" => self.unimplemented += 1,
            _ => self.oracle_errors += 1,
        }
    }
}
