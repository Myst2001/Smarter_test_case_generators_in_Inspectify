mod cobertura;
mod compare;
mod runner;
mod stats;

use chrono::Local;
use clap::Parser;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use runner::{GeneratorMode, RunConfig};
use stats::{AggregateStats, RunStats};

#[derive(Parser, Debug)]
#[command(
    name = "coverage_runner",
    about = "Run Rust analyzer under cargo-llvm-cov and collect line/branch coverage statistics"
)]
struct Cli {
    /// Directory where results will be saved (default: src/Coverage/results in crate)
    #[arg(long)]
    output_dir: Option<String>,

    /// Number of iterations (inputs) to run
    #[arg(long, default_value_t = 200)]
    iterations: usize,

    /// Generator mode: new, old, or compare
    #[arg(long, default_value = "new")]
    mode: String,

    /// Optional RNG seed for reproducibility (0 = random)
    #[arg(long, default_value_t = 0u64)]
    seed: u64,
}

fn make_rng(seed: u64) -> SmallRng {
    if seed == 0 {
        SmallRng::from_os_rng()
    } else {
        SmallRng::seed_from_u64(seed)
    }
}

fn timestamp_label() -> String {
    Local::now().format("%Y-%m-%dT%H-%M-%S").to_string()
}

fn run_batch(
    iterations: usize,
    config: &RunConfig,
    rng: &mut SmallRng,
    label: &str,
) -> Vec<RunStats> {
    let mut results: Vec<RunStats> = Vec::with_capacity(iterations);

    for i in 0..iterations {
        print!("\r[{}] Running iteration {}/{} ...", label, i + 1, iterations);
        use std::io::Write;
        let _ = std::io::stdout().flush();

        match runner::run_iteration(config, i, rng) {
            Ok(run_stats) => results.push(run_stats),
            Err(e) => {
                eprintln!(
                    "\n[{}] Warning: iteration {} failed: {}",
                    label, i, e
                );
            }
        }
    }
    println!(
        "\r[{}] Done. {} iterations completed.         ",
        label,
        results.len()
    );
    results
}

/// Default output dir: Coverage/results (relative to crate manifest)
fn default_output_dir() -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("Coverage")
        .join("results")
        .to_string_lossy()
        .into_owned()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let output_dir = cli.output_dir.unwrap_or_else(default_output_dir);

    let seed = cli.seed;
    let effective_seed = if seed == 0 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42)
    } else {
        seed
    };

    let ts = timestamp_label();

    match cli.mode.as_str() {
        "new" => {
            let run_dir = std::path::PathBuf::from(&output_dir)
                .join(format!("{}_{}", ts, "new"));
            std::fs::create_dir_all(&run_dir)?;
            println!("Output directory: {}", run_dir.display());
            println!("Seed: {}", effective_seed);

            let config = RunConfig {
                output_dir: run_dir.clone(),
                mode: GeneratorMode::New,
            };
            let mut rng = make_rng(effective_seed);
            let runs = run_batch(cli.iterations, &config, &mut rng, "NEW");
            let agg = stats::aggregate(&runs);

            stats::write_csv(&run_dir.join("results.csv"), &runs)?;
            stats::write_summary(&run_dir.join("summary.txt"), "NEW", &agg)?;

            print_summary("NEW", &agg);
            println!("Results saved to: {}", run_dir.display());
        }

        "old" => {
            let run_dir = std::path::PathBuf::from(&output_dir)
                .join(format!("{}_{}", ts, "old"));
            std::fs::create_dir_all(&run_dir)?;
            println!("Output directory: {}", run_dir.display());
            println!("Seed: {}", effective_seed);

            let config = RunConfig {
                output_dir: run_dir.clone(),
                mode: GeneratorMode::Old,
            };
            let mut rng = make_rng(effective_seed);
            let runs = run_batch(cli.iterations, &config, &mut rng, "OLD");
            let agg = stats::aggregate(&runs);

            stats::write_csv(&run_dir.join("results.csv"), &runs)?;
            stats::write_summary(&run_dir.join("summary.txt"), "OLD", &agg)?;

            print_summary("OLD", &agg);
            println!("Results saved to: {}", run_dir.display());
        }

        "compare" => {
            let run_dir = std::path::PathBuf::from(&output_dir)
                .join(format!("{}_{}", ts, "compare"));
            std::fs::create_dir_all(&run_dir)?;
            println!("Output directory: {}", run_dir.display());
            println!(
                "Seed: {} (same seed used for both generators)",
                effective_seed
            );

            // Run NEW generator
            let new_dir = run_dir.join("new");
            std::fs::create_dir_all(&new_dir)?;
            let new_config = RunConfig {
                output_dir: new_dir.clone(),
                mode: GeneratorMode::New,
            };
            let mut rng_new = make_rng(effective_seed);
            let new_runs = run_batch(cli.iterations, &new_config, &mut rng_new, "NEW");
            let new_agg = stats::aggregate(&new_runs);
            stats::write_csv(&new_dir.join("results.csv"), &new_runs)?;
            stats::write_summary(&new_dir.join("summary.txt"), "NEW", &new_agg)?;

            // Run OLD generator (same seed for fairness)
            let old_dir = run_dir.join("old");
            std::fs::create_dir_all(&old_dir)?;
            let old_config = RunConfig {
                output_dir: old_dir.clone(),
                mode: GeneratorMode::Old,
            };
            let mut rng_old = make_rng(effective_seed);
            let old_runs = run_batch(cli.iterations, &old_config, &mut rng_old, "OLD");
            let old_agg = stats::aggregate(&old_runs);
            stats::write_csv(&old_dir.join("results.csv"), &old_runs)?;
            stats::write_summary(&old_dir.join("summary.txt"), "OLD", &old_agg)?;

            print_summary("NEW", &new_agg);
            print_summary("OLD", &old_agg);

            let report = compare::compare("NEW", new_agg, "OLD", old_agg);
            compare::print_comparison(&report);
            compare::write_comparison(&run_dir.join("comparison.txt"), &report)?;

            println!("Results saved to: {}", run_dir.display());
        }

        other => {
            eprintln!(
                "Unknown mode: '{}'. Use 'new', 'old', or 'compare'.",
                other
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_summary(label: &str, agg: &AggregateStats) {
    println!();
    println!("=== {} Summary ===", label);
    println!("  Iterations:        {}", agg.n);
    println!("  Mean line rate:    {:.4}", agg.mean_line_rate);
    println!("  Mean branch rate:  {:.4}", agg.mean_branch_rate);
    println!("  Max line rate:     {:.4}", agg.max_line_rate);
    println!("  Max branch rate:   {:.4}", agg.max_branch_rate);
    println!("  Std dev (line):    {:.6}", agg.var_line_rate.sqrt());
}
