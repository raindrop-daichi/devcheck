use crate::checker::{CheckResult, CheckStatus};
use crate::config::{Config, OutputFormat};
use anyhow::Result;
use colored::*;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Serialize)]
pub struct Report {
    pub version: String,
    pub timestamp: String,
    pub duration_secs: f64,
    pub summary: Summary,
    pub checks: Vec<CheckResult>,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
}

impl Report {
    pub fn from_results(results: Vec<CheckResult>, start_time: Instant) -> Self {
        let duration = start_time.elapsed();
        let total = results.len();
        let passed = results
            .iter()
            .filter(|r| r.status == CheckStatus::Passed)
            .count();
        let failed = results
            .iter()
            .filter(|r| r.status == CheckStatus::Failed)
            .count();

        Report {
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_secs: duration.as_secs_f64(),
            summary: Summary {
                total,
                passed,
                failed,
            },
            checks: results,
        }
    }

    pub fn has_failures(&self) -> bool {
        self.summary.failed > 0
    }
}

pub fn display_report(report: &Report, config: &Config) -> Result<()> {
    match config.format {
        OutputFormat::Terminal => display_terminal(report, config),
        OutputFormat::Json => display_json(report),
    }
}

fn display_terminal(report: &Report, config: &Config) -> Result<()> {
    if !config.quiet {
        println!(
            "\n{}",
            "╭─────────────────────────────────────╮".bright_blue()
        );
        println!(
            "{}",
            "│  devcheck - Rust Quality Checker    │".bright_blue()
        );
        println!(
            "{}",
            "╰─────────────────────────────────────╯".bright_blue()
        );
        println!();
    }

    // Display individual check results
    for check in &report.checks {
        let symbol = match check.status {
            CheckStatus::Passed => "✓".green(),
            CheckStatus::Failed => "✗".red(),
            CheckStatus::Skipped => "○".yellow(),
        };

        let status_str = match check.status {
            CheckStatus::Passed => "PASSED".green(),
            CheckStatus::Failed => "FAILED".red(),
            CheckStatus::Skipped => "SKIPPED".yellow(),
        };

        println!(
            "{} cargo {:<8} [{:>6.1}s] {}",
            symbol,
            check.name,
            check.duration.as_secs_f64(),
            status_str
        );
    }

    println!(
        "\n{}",
        "───────────────────────────────────────".bright_black()
    );

    // Display summary
    if !config.quiet {
        println!("\n{}:", "Summary".bold());
        println!("  Total:   {} checks", report.summary.total);
        println!(
            "  Passed:  {} checks",
            report.summary.passed.to_string().green()
        );
        if report.summary.failed > 0 {
            println!(
                "  Failed:  {} checks",
                report.summary.failed.to_string().red()
            );
        } else {
            println!("  Failed:  {} checks", report.summary.failed);
        }
        println!("  Time:    {:.1}s", report.duration_secs);
    }

    // Display failed check details
    if report.has_failures() {
        println!("\n{}:", "Failed Checks".red().bold());
        for check in &report.checks {
            if check.status == CheckStatus::Failed {
                println!("  {} cargo {}", "✗".red(), check.name);
                for error in &check.errors {
                    println!("    {}", error.dimmed());
                }

                if config.verbose && !check.output.is_empty() {
                    println!("\n    {}:", "Output".dimmed());
                    for line in check.output.lines().take(20) {
                        println!("    {}", line.dimmed());
                    }
                }
            }
        }
    }

    println!();
    Ok(())
}

fn display_json(report: &Report) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}
