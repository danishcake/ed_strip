use std::path::{Path, PathBuf};

use clap::Parser;
use ed_strip::errors::{EdStripResult, StrippingError};
use ed_strip::strip_process::{find_files, process_file};
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
    // Initialise logging. Default to Info unless RUST_LOG says differently
    if std::env::var("RUST_LOG").is_err() {
        env_logger::Builder::new()
            .filter(None, log::LevelFilter::Info)
            .init();
    } else {
        env_logger::init();
    }
    // Parse the arguments
    let args = Args::parse();
    let input_dir = std::path::absolute(args.input_dir)
        .map_err(|e: std::io::Error| -> StrippingError { e.into() })?;
    let output_dir = std::path::absolute(args.output_dir)
        .map_err(|e: std::io::Error| -> StrippingError { e.into() })?;

    // Find files
    let files = find_files(&input_dir, &args.glob)?;

    // Initialise threadpool
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.jobs)
        .build_global()?;

    // Strip each file
    let (total_jobs, passed_jobs) = files
        .par_bridge()
        .map(|path| {
            match path {
                Ok(path) => {
                    let result = process_file(&input_dir, &output_dir, &path);
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
