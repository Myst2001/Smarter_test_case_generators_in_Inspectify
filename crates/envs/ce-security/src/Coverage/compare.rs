use std::fmt::Write as FmtWrite;

use crate::stats::AggregateStats;

#[derive(Debug)]
pub struct CompareReport {
    pub label_a: String,
    pub label_b: String,
    pub stats_a: AggregateStats,
    pub stats_b: AggregateStats,
    /// Which generator has a higher mean line rate.
    pub winner_line_rate: String,
    /// Which generator has a higher mean branch rate.
    pub winner_branch_rate: String,
    /// Which generator has lower variance in line rate (more consistent).
    pub winner_variance: String,
    /// Which generator reaches 90% of its own max faster.
    pub winner_convergence: String,
}

/// Returns the number of iterations needed to reach 90% of the final cumulative
/// max. If the cumulative max vec is empty, returns `n`.
fn convergence_iter(stats: &AggregateStats) -> usize {
    if stats.cumulative_max.is_empty() {
        return stats.n;
    }
    let final_max = *stats.cumulative_max.last().unwrap();
    let threshold = final_max * 0.90;
    stats
        .cumulative_max
        .iter()
        .position(|&v| v >= threshold)
        .map(|i| i + 1)
        .unwrap_or(stats.n)
}

pub fn compare(
    label_a: &str,
    stats_a: AggregateStats,
    label_b: &str,
    stats_b: AggregateStats,
) -> CompareReport {
    let winner_line_rate = if stats_a.mean_line_rate >= stats_b.mean_line_rate {
        label_a.to_string()
    } else {
        label_b.to_string()
    };

    let winner_branch_rate = if stats_a.mean_branch_rate >= stats_b.mean_branch_rate {
        label_a.to_string()
    } else {
        label_b.to_string()
    };

    // Lower variance = more consistent = better for benchmark stability
    let winner_variance = if stats_a.var_line_rate <= stats_b.var_line_rate {
        label_a.to_string()
    } else {
        label_b.to_string()
    };

    let conv_a = convergence_iter(&stats_a);
    let conv_b = convergence_iter(&stats_b);
    let winner_convergence = if conv_a <= conv_b {
        label_a.to_string()
    } else {
        label_b.to_string()
    };

    CompareReport {
        label_a: label_a.to_string(),
        label_b: label_b.to_string(),
        stats_a,
        stats_b,
        winner_line_rate,
        winner_branch_rate,
        winner_variance,
        winner_convergence,
    }
}

pub fn write_comparison(
    path: &std::path::Path,
    report: &CompareReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = String::new();
    format_report(&mut out, report)?;
    std::fs::write(path, out)?;
    Ok(())
}

pub fn print_comparison(report: &CompareReport) {
    let mut out = String::new();
    format_report(&mut out, report).unwrap();
    print!("{}", out);
}

fn format_report(
    out: &mut String,
    report: &CompareReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let conv_a = convergence_iter(&report.stats_a);
    let conv_b = convergence_iter(&report.stats_b);

    writeln!(
        out,
        "=== Coverage Comparison: {} vs {} ===",
        report.label_a, report.label_b
    )?;
    writeln!(out)?;
    writeln!(
        out,
        "{:<30} {:>12} {:>12}",
        "Metric", report.label_a, report.label_b
    )?;
    writeln!(out, "{}", "-".repeat(56))?;
    writeln!(
        out,
        "{:<30} {:>12.4} {:>12.4}",
        "Mean line rate",
        report.stats_a.mean_line_rate,
        report.stats_b.mean_line_rate,
    )?;
    writeln!(
        out,
        "{:<30} {:>12.4} {:>12.4}",
        "Mean branch rate",
        report.stats_a.mean_branch_rate,
        report.stats_b.mean_branch_rate,
    )?;
    writeln!(
        out,
        "{:<30} {:>12.4} {:>12.4}",
        "Max line rate",
        report.stats_a.max_line_rate,
        report.stats_b.max_line_rate,
    )?;
    writeln!(
        out,
        "{:<30} {:>12.4} {:>12.4}",
        "Max branch rate",
        report.stats_a.max_branch_rate,
        report.stats_b.max_branch_rate,
    )?;
    writeln!(
        out,
        "{:<30} {:>12.6} {:>12.6}",
        "Variance (line rate)",
        report.stats_a.var_line_rate,
        report.stats_b.var_line_rate,
    )?;
    writeln!(
        out,
        "{:<30} {:>12.6} {:>12.6}",
        "Std dev (line rate)",
        report.stats_a.var_line_rate.sqrt(),
        report.stats_b.var_line_rate.sqrt(),
    )?;
    writeln!(
        out,
        "{:<30} {:>12} {:>12}",
        "Iterations (n)",
        report.stats_a.n,
        report.stats_b.n,
    )?;
    writeln!(
        out,
        "{:<30} {:>12} {:>12}",
        "Iters to 90% max coverage",
        conv_a,
        conv_b,
    )?;
    writeln!(out)?;
    writeln!(out, "=== Winners ===")?;
    writeln!(
        out,
        "  Higher mean line rate:     {}",
        report.winner_line_rate
    )?;
    writeln!(
        out,
        "  Higher mean branch rate:   {}",
        report.winner_branch_rate
    )?;
    writeln!(
        out,
        "  Lower variance (stable):   {}",
        report.winner_variance
    )?;
    writeln!(
        out,
        "  Faster convergence:        {}",
        report.winner_convergence
    )?;

    Ok(())
}
