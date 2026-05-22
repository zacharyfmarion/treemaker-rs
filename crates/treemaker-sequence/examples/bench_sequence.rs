use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use treemaker_fold::{FoldDocument, validate_basic};
use treemaker_sequence::{
    SequencePlanOptions, SolutionLimit, TargetStateOptions, plan_folding_sequence_with_options,
    resolve_target_state,
};

#[derive(Debug)]
struct Config {
    repeat: usize,
    max_steps: usize,
    max_states: usize,
    solution_limit: usize,
    files: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args(env::args().skip(1))?;
    println!(
        "file\trepeat\tvertices\tedges\tfaces\ttarget_ms\tplan_ms\ttotal_ms\tstatus\tsteps\tstates_explored\tbranches_pruned\trepeated_states\tbest_unresolved_creases\ttarget_solves\ttarget_solve_cache_hits\tduplicate_candidates_pruned"
    );
    for file in &config.files {
        let document = read_fold(file)?;
        validate_basic(&document)?;
        for repeat in 0..config.repeat {
            let target_start = Instant::now();
            let target = resolve_target_state(
                &document,
                TargetStateOptions {
                    solution_limit: SolutionLimit::Count(config.solution_limit),
                    ..TargetStateOptions::default()
                },
            )?;
            let target_elapsed = target_start.elapsed();

            let plan_start = Instant::now();
            let plan = plan_folding_sequence_with_options(
                &target,
                SequencePlanOptions {
                    max_steps: config.max_steps,
                    max_states: config.max_states,
                },
            )?;
            let plan_elapsed = plan_start.elapsed();
            let total_elapsed = target_elapsed + plan_elapsed;

            println!(
                "{}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                file.display(),
                repeat + 1,
                document.vertices_coords.len(),
                document.edges_vertices.len(),
                document.faces_vertices.len(),
                millis(target_elapsed),
                millis(plan_elapsed),
                millis(total_elapsed),
                plan.status,
                plan.steps.len(),
                plan.search.states_explored,
                plan.search.branches_pruned,
                plan.search.repeated_states,
                plan.search.best_unresolved_creases,
                plan.search.target_solves,
                plan.search.target_solve_cache_hits,
                plan.search.duplicate_candidates_pruned
            );
        }
    }
    Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Config, Box<dyn std::error::Error>> {
    let mut repeat = 1usize;
    let mut max_steps = 64usize;
    let mut max_states = 1024usize;
    let mut solution_limit = 10usize;
    let mut files = Vec::new();
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repeat" => repeat = parse_next_usize(&mut args, "--repeat")?,
            "--max-steps" => max_steps = parse_next_usize(&mut args, "--max-steps")?,
            "--max-states" => max_states = parse_next_usize(&mut args, "--max-states")?,
            "--solution-limit" => solution_limit = parse_next_usize(&mut args, "--solution-limit")?,
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown option {value}").into());
            }
            value => files.push(PathBuf::from(value)),
        }
    }
    if files.is_empty() {
        print_usage();
        return Err("at least one FOLD file path is required".into());
    }
    Ok(Config {
        repeat,
        max_steps,
        max_states,
        solution_limit,
        files,
    })
}

fn parse_next_usize(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    option: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let value = args
        .next()
        .ok_or_else(|| format!("{option} requires a value"))?;
    value
        .parse::<usize>()
        .map_err(|error| format!("invalid {option} value {value}: {error}").into())
}

fn print_usage() {
    eprintln!(
        "usage: cargo run -p treemaker-sequence --release --example bench_sequence -- [--repeat N] [--max-steps N] [--max-states N] [--solution-limit N] <file.fold>..."
    );
}

fn read_fold(path: &Path) -> Result<FoldDocument, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|error| {
        format!(
            "failed to parse {} as a FOLD JSON document: {error}",
            path.display()
        )
        .into()
    })
}

fn millis(duration: std::time::Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}
