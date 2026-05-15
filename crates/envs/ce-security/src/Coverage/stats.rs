use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RunStats {
    pub iteration: usize,
    pub line_rate: f64,
    pub branch_rate: f64,
    pub lines_valid: u64,
    pub lines_covered: u64,
}

#[derive(Debug, Clone)]
pub struct AggregateStats {
    pub n: usize,
    pub mean_line_rate: f64,
    pub mean_branch_rate: f64,
    pub max_line_rate: f64,
    pub max_branch_rate: f64,
    pub var_line_rate: f64,
    pub var_branch_rate: f64,
    /// Cumulative max line-rate after k iterations (for convergence analysis).
    pub cumulative_max: Vec<f64>,
}

pub fn aggregate(runs: &[RunStats]) -> AggregateStats {
    let n = runs.len();
    if n == 0 {
        return AggregateStats {
            n: 0,
            mean_line_rate: 0.0,
            mean_branch_rate: 0.0,
            max_line_rate: 0.0,
            max_branch_rate: 0.0,
            var_line_rate: 0.0,
            var_branch_rate: 0.0,
            cumulative_max: Vec::new(),
        };
    }

    let sum_line: f64 = runs.iter().map(|r| r.line_rate).sum();
    let sum_branch: f64 = runs.iter().map(|r| r.branch_rate).sum();
    let mean_line_rate = sum_line / n as f64;
    let mean_branch_rate = sum_branch / n as f64;

    let max_line_rate = runs
        .iter()
        .map(|r| r.line_rate)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_branch_rate = runs
        .iter()
        .map(|r| r.branch_rate)
        .fold(f64::NEG_INFINITY, f64::max);

    let var_line_rate = if n > 1 {
        runs.iter()
            .map(|r| {
                let diff = r.line_rate - mean_line_rate;
                diff * diff
            })
            .sum::<f64>()
            / (n - 1) as f64
    } else {
        0.0
    };

    let var_branch_rate = if n > 1 {
        runs.iter()
            .map(|r| {
                let diff = r.branch_rate - mean_branch_rate;
                diff * diff
            })
            .sum::<f64>()
            / (n - 1) as f64
    } else {
        0.0
    };

    // Build cumulative max line-rate
    let mut cumulative_max = Vec::with_capacity(n);
    let mut running_max = f64::NEG_INFINITY;
    for r in runs {
        if r.line_rate > running_max {
            running_max = r.line_rate;
        }
        cumulative_max.push(running_max);
    }

    AggregateStats {
        n,
        mean_line_rate,
        mean_branch_rate,
        max_line_rate,
        max_branch_rate,
        var_line_rate,
        var_branch_rate,
        cumulative_max,
    }
}

pub fn write_csv(
    path: &std::path::Path,
    runs: &[RunStats],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    for run in runs {
        wtr.serialize(run)?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn write_summary(
    path: &std::path::Path,
    label: &str,
    stats: &AggregateStats,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write as FmtWrite;
    let mut out = String::new();

    writeln!(out, "=== Coverage Summary: {} ===", label)?;
    writeln!(out, "Iterations: {}", stats.n)?;
    writeln!(out, "Mean line rate:    {:.4}", stats.mean_line_rate)?;
    writeln!(out, "Mean branch rate:  {:.4}", stats.mean_branch_rate)?;
    writeln!(out, "Max line rate:     {:.4}", stats.max_line_rate)?;
    writeln!(out, "Max branch rate:   {:.4}", stats.max_branch_rate)?;
    writeln!(out, "Variance (line):   {:.6}", stats.var_line_rate)?;
    writeln!(out, "Variance (branch): {:.6}", stats.var_branch_rate)?;
    writeln!(out, "Std dev (line):    {:.6}", stats.var_line_rate.sqrt())?;
    writeln!(out, "Std dev (branch):  {:.6}", stats.var_branch_rate.sqrt())?;

    if !stats.cumulative_max.is_empty() {
        let final_max = *stats.cumulative_max.last().unwrap();
        writeln!(out, "Final cumulative max line rate: {:.4}", final_max)?;

        // Convergence: how many iterations to reach 90% of final max
        let threshold = final_max * 0.90;
        let converge_at = stats
            .cumulative_max
            .iter()
            .position(|&v| v >= threshold)
            .map(|i| i + 1)
            .unwrap_or(stats.n);
        writeln!(
            out,
            "Iterations to reach 90% of max coverage: {}",
            converge_at
        )?;
    }

    std::fs::write(path, out)?;
    Ok(())
}
