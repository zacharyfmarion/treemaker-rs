use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};
use treemaker_core::{Tree, TreeError};

#[derive(Parser)]
#[command(
    name = "TreeMaker",
    version,
    about = "Headless Rust port of the TreeMaker 5.0.1 model engine"
)]
struct Cli {
    #[arg(short = 'd', long = "datadir", global = true)]
    datadir: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Inspect {
        file: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    Check {
        file: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    Optimize {
        file: PathBuf,
        #[arg(long, value_enum)]
        kind: OptimizeKind,
        #[arg(long)]
        out: PathBuf,
    },
    BuildCp {
        file: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    ExportV4 {
        file: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    RunFixtures {
        #[arg(long)]
        dir: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OptimizeKind {
    Scale,
    Edge,
    Strain,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Inspect { file, format } => {
            let tree = read_tree(&file)?;
            print_value(format, &tree.summary())?;
        }
        Command::Check { file, format } => {
            let tree = read_tree(&file)?;
            print_value(format, &tree.summary())?;
            if !tree.is_feasible() {
                std::process::exit(2);
            }
        }
        Command::Optimize { file, kind, out } => {
            let mut tree = read_tree(&file)?;
            match kind {
                OptimizeKind::Scale => tree.optimize_scale(),
                OptimizeKind::Edge => tree.optimize_edges(),
                OptimizeKind::Strain => tree.optimize_strain(),
            }
            .map_err(anyhow_from_tree_error)?;
            fs::write(&out, tree.to_tmd5_string())
                .with_context(|| format!("failed to write {}", out.display()))?;
        }
        Command::BuildCp { file, out } => {
            let mut tree = read_tree(&file)?;
            tree.build_polys_and_crease_pattern()
                .map_err(anyhow_from_tree_error)?;
            fs::write(&out, tree.to_tmd5_string())
                .with_context(|| format!("failed to write {}", out.display()))?;
        }
        Command::ExportV4 { file, out } => {
            let tree = read_tree(&file)?;
            fs::write(&out, tree.export_v4_string())
                .with_context(|| format!("failed to write {}", out.display()))?;
        }
        Command::RunFixtures { dir } => {
            let dir = dir.unwrap_or_else(|| PathBuf::from("tests/fixtures"));
            run_fixtures(&dir)?;
        }
    }
    Ok(())
}

fn read_tree(path: &Path) -> Result<Tree> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    Tree::from_tmd_str(&text).map_err(anyhow_from_tree_error)
}

fn print_value<T: serde::Serialize + std::fmt::Debug>(
    format: OutputFormat,
    value: &T,
) -> Result<()> {
    match format {
        OutputFormat::Text => println!("{value:#?}"),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(value)?),
    }
    Ok(())
}

fn run_fixtures(dir: &Path) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    let mut count = 0usize;
    for entry in entries {
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if !matches!(ext, "tmd" | "tmd4" | "tmd5") {
            continue;
        }
        let tree = read_tree(&path)?;
        let summary = tree.summary();
        println!(
            "{}: nodes={}, edges={}, paths={}, conditions={}, feasible={}",
            path.display(),
            summary.nodes,
            summary.edges,
            summary.paths,
            summary.conditions,
            summary.is_feasible
        );
        count += 1;
    }
    println!("parsed {count} fixture(s)");
    Ok(())
}

fn anyhow_from_tree_error(error: TreeError) -> anyhow::Error {
    anyhow::anyhow!("{error}")
}
