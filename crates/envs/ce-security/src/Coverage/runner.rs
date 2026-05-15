use std::path::PathBuf;
use std::process::Command;

use rand::Rng;

/// Find workspace root for cargo commands (parent of target/ dir)
fn workspace_root_for_cargo() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        // exe is target/release/coverage_runner or target/debug/coverage_runner
        if let Some(release_or_debug) = exe.parent() {
            if release_or_debug
                .file_name()
                .map(|n| n == "release" || n == "debug")
                .unwrap_or(false)
            {
                if let Some(target_dir) = release_or_debug.parent() {
                    if target_dir.file_name().map(|n| n == "target").unwrap_or(false) {
                        if let Some(ws) = target_dir.parent() {
                            return ws.to_path_buf();
                        }
                    }
                }
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

use crate::cobertura::{self, CoverageReport};
use crate::stats::RunStats;

pub struct RunConfig {
    pub output_dir: PathBuf,
    pub mode: GeneratorMode,
}

#[derive(Clone, Copy, Debug)]
pub enum GeneratorMode {
    New,
    Old,
}

pub fn run_iteration<R: Rng>(
    config: &RunConfig,
    iter: usize,
    rng: &mut R,
) -> Result<RunStats, Box<dyn std::error::Error>> {
    // 1. Create per-iteration directory
    let iter_dir = config.output_dir.join(format!("iter_{:05}", iter));
    std::fs::create_dir_all(&iter_dir)?;

    // 2. Generate input based on mode
    let input = match config.mode {
        GeneratorMode::New => ce_security::generator::generate_input(rng),
        GeneratorMode::Old => ce_security::generator_old::generate_input(rng),
    };

    // 3. Serialize and write input.json
    let input_json = serde_json::to_string(&input)?;
    let input_path = iter_dir.join("input.json");
    std::fs::write(&input_path, &input_json)?;

    // 4. Build coverage output path
    let coverage_xml_path = iter_dir.join("coverage.xml");

    // 5. Run cargo llvm-cov on the analyzer binary (-p ce-security for workspace)
    // Ensure we run from workspace root so cargo finds the workspace
    let workspace_root = workspace_root_for_cargo();

    // 5a. Clear stale profile data so this iteration's report is isolated.
    // cargo-llvm-cov accumulates .profraw / .profdata files in
    // target/llvm-cov-target/ by default; if we do not clear them, each
    // iteration's Cobertura report will include coverage inherited from
    // earlier iterations in the same batch. We delete the profile files
    // directly (instead of `cargo llvm-cov clean`) so the compiled
    // analyzer binary stays cached and we avoid a rebuild per iteration.
    let llvm_cov_target = workspace_root.join("target").join("llvm-cov-target");
    if llvm_cov_target.exists() {
        if let Ok(entries) = std::fs::read_dir(&llvm_cov_target) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "profraw" || ext == "profdata" {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }

    let status = Command::new("cargo")
        .current_dir(&workspace_root)
        .args([
            "llvm-cov",
            "run",
            "--no-clean",
            "-p",
            "ce-security",
            "--bin",
            "analyzer",
            "--cobertura",
            "--output-path",
            &coverage_xml_path.to_string_lossy(),
            "--",
            "--input",
            &input_path.to_string_lossy(),
        ])
        .status()?;

    if !status.success() {
        return Err(format!(
            "cargo llvm-cov exited with non-zero status on iteration {}",
            iter
        )
        .into());
    }

    // 6. Filter Cobertura to analysis.rs only (Rust-only)
    let filtered_coverage_xml_path = iter_dir.join("coverage_analysis.xml");
    let report: CoverageReport = if coverage_xml_path.exists() {
        cobertura::filter_analysis_file(
            &coverage_xml_path,
            &filtered_coverage_xml_path,
            "crates/envs/ce-security/src/analysis.rs",
        )
        .unwrap_or_default()
    } else {
        CoverageReport::default()
    };

    // 7. Return stats
    Ok(RunStats {
        iteration: iter,
        line_rate: report.line_rate,
        branch_rate: report.branch_rate,
        lines_valid: report.lines_valid,
        lines_covered: report.lines_covered,
    })
}
