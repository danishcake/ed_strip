use std::path::{Path, PathBuf};

use clap::{ArgAction, Parser};
use ed_strip::errors::{EdStripResult, StrippingError};
use ed_strip::strip_process::{find_files, process_file};
use ed_strip::type_hints::{load_type_hints_file, TypeHints};
use log::debug;
use rayon::prelude::*;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory to read from
    #[arg(short, long)]
    input_dir: PathBuf,

    /// Directory to output to
    #[arg(short, long)]
    output_dir: PathBuf,

    /// Glob to use. Should not be expanded by shell
    #[arg(short, long, default_value_t = String::from("**/*.*"))]
    glob: String,

    /// Number of concurrent stripping jobs. Defaults to number of available cores
    #[arg(short, long, default_value_t = 0)]
    jobs: usize,

    /// An optional JSON file containing type hints
    #[arg(short = 't', long = "type-hints")]
    type_hints_path: Option<PathBuf>,

    /// Increase verbosity to debug
    #[arg(short = 'v', long = "verbose", action = ArgAction::SetTrue)]
    verbose: bool,

    /// Decrease verbosity to error
    /// Takes precedent over warn
    #[arg(short = 'q', long = "quiet", action = ArgAction::SetTrue)]
    quiet: bool,
}

/// Output the stripping result for a single job
/// Returns a tuple containing the number of jobs (1), and the number of successful jobs (1 or 0)
fn report_result(result: Result<(), StrippingError>, path: &Path) -> (i32, i32) {
    match &result {
        Ok(()) => {
            log::info!("{}: OK", path.to_string_lossy());
            (1, 1)
        }
        Err(e) => {
            log::warn!("{}: {}", path.to_string_lossy(), e);
            (1, 0)
        }
    }
}

fn main() -> EdStripResult<()> {
    let args = Args::parse();

    // Initialise logging.
    // Default to verbosity flag value unless RUST_LOG says differently
    if std::env::var("RUST_LOG").is_err() {
        let level_filter: log::LevelFilter = if args.quiet {
            log::LevelFilter::Warn
        } else if args.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        };

        env_logger::Builder::new().filter(None, level_filter).init();
    } else {
        env_logger::init();
    }

    // Parse the arguments

    let input_dir = std::path::absolute(args.input_dir)
        .map_err(|e: std::io::Error| -> StrippingError { e.into() })?;
    let output_dir = std::path::absolute(args.output_dir)
        .map_err(|e: std::io::Error| -> StrippingError { e.into() })?;

    // Load the type hints
    let type_hints: TypeHints = if let Some(type_hints_path) = args.type_hints_path {
        load_type_hints_file(&type_hints_path)?
    } else {
        debug!("No type hints specified");
        Vec::new()
    };

    // Find files
    let files = find_files(&input_dir, &args.glob)?;

    // Initialise threadpool
    debug!("Initialising threadpool with {} workers", args.jobs);
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.jobs)
        .build_global()?;

    // Strip each file
    let (total_jobs, passed_jobs) = files
        .par_bridge()
        .map(|path| {
            match path {
                Ok(path) => {
                    let result = process_file(&input_dir, &output_dir, &type_hints, &path);
                    report_result(result, &path)
                }
                Err(e) => {
                    // Error unwrapping path - probably permissions problem
                    log::warn!("Glob error: {}", e);
                    (1, 0)
                }
            }
        })
        .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1));

    log::info!("{}/{} jobs passed", passed_jobs, total_jobs);
    if passed_jobs != total_jobs {
        log::warn!("{} jobs failed", total_jobs - passed_jobs);
    }
    std::process::exit(total_jobs - passed_jobs);
}
