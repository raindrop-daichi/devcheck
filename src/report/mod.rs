use crate::checker::{CheckResult, CheckStatus};
use crate::config::{ColorMode, Config, OutputFormat};
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
    // Set color mode based on config
    match config.color {
        ColorMode::Always => colored::control::set_override(true),
        ColorMode::Never => colored::control::set_override(false),
        ColorMode::Auto => {
            // Let colored crate auto-detect
            colored::control::unset_override();
        }
    }

    match config.format {
        OutputFormat::Terminal => display_terminal(report, config),
        OutputFormat::Json => display_json(report),
        OutputFormat::Html => display_html(report),
        OutputFormat::Markdown => display_markdown(report),
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
        };

        let status_str = match check.status {
            CheckStatus::Passed => "PASSED".green(),
            CheckStatus::Failed => "FAILED".red(),
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

fn display_html(report: &Report) -> Result<()> {
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>devcheck Report</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        .container {{
            background: white;
            border-radius: 8px;
            padding: 30px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            border-bottom: 3px solid #007acc;
            padding-bottom: 10px;
        }}
        .summary {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin: 30px 0;
        }}
        .stat {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 6px;
            text-align: center;
        }}
        .stat-value {{
            font-size: 2em;
            font-weight: bold;
            margin: 10px 0;
        }}
        .stat-label {{
            color: #666;
            font-size: 0.9em;
        }}
        .checks {{
            margin-top: 30px;
        }}
        .check {{
            border: 1px solid #e1e4e8;
            border-radius: 6px;
            padding: 15px;
            margin: 10px 0;
        }}
        .check-header {{
            display: flex;
            align-items: center;
            font-weight: 500;
        }}
        .status {{
            padding: 4px 12px;
            border-radius: 4px;
            font-size: 0.85em;
            margin-left: auto;
        }}
        .passed {{
            background: #d4edda;
            color: #155724;
        }}
        .failed {{
            background: #f8d7da;
            color: #721c24;
        }}
        .duration {{
            color: #666;
            font-size: 0.9em;
            margin-left: 10px;
        }}
        .errors {{
            margin-top: 10px;
            padding: 10px;
            background: #fff3cd;
            border-left: 4px solid #ffc107;
            font-family: monospace;
            font-size: 0.9em;
        }}
        .footer {{
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #e1e4e8;
            color: #666;
            font-size: 0.9em;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔍 devcheck Report</h1>
        
        <div class="summary">
            <div class="stat">
                <div class="stat-value">{}</div>
                <div class="stat-label">Total Checks</div>
            </div>
            <div class="stat">
                <div class="stat-value" style="color: #28a745;">{}</div>
                <div class="stat-label">Passed</div>
            </div>
            <div class="stat">
                <div class="stat-value" style="color: #dc3545;">{}</div>
                <div class="stat-label">Failed</div>
            </div>
            <div class="stat">
                <div class="stat-value">{:.2}s</div>
                <div class="stat-label">Duration</div>
            </div>
        </div>

        <div class="checks">
            <h2>Check Results</h2>
{}"#,
        report.summary.total,
        report.summary.passed,
        report.summary.failed,
        report.duration_secs,
        report
            .checks
            .iter()
            .map(|check| {
                let status_class = match check.status {
                    CheckStatus::Passed => "passed",
                    CheckStatus::Failed => "failed",
                };
                let status_text = format!("{:?}", check.status).to_uppercase();
                let errors_html = if !check.errors.is_empty() {
                    format!(
                        r#"
                <div class="errors">
                    {}
                </div>"#,
                        check
                            .errors
                            .iter()
                            .map(|e| format!("<div>{}</div>", html_escape(e)))
                            .collect::<Vec<_>>()
                            .join("")
                    )
                } else {
                    String::new()
                };
                format!(
                    r#"
            <div class="check">
                <div class="check-header">
                    <span>{}</span>
                    <span class="duration">{:.2}s</span>
                    <span class="status {}">{}</span>
                </div>
                {}
            </div>"#,
                    html_escape(&check.name),
                    check.duration.as_secs_f64(),
                    status_class,
                    status_text,
                    errors_html
                )
            })
            .collect::<Vec<_>>()
            .join("")
    );

    let html = format!(
        r#"{}
        </div>

        <div class="footer">
            Generated by devcheck v{} at {}
        </div>
    </div>
</body>
</html>"#,
        html, report.version, report.timestamp
    );

    println!("{}", html);
    Ok(())
}

fn display_markdown(report: &Report) -> Result<()> {
    let mut output = String::new();

    output.push_str("# devcheck Report\n\n");

    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Total Checks**: {}\n", report.summary.total));
    output.push_str(&format!("- **Passed**: ✅ {}\n", report.summary.passed));
    output.push_str(&format!("- **Failed**: ❌ {}\n", report.summary.failed));
    output.push_str(&format!("- **Duration**: {:.2}s\n", report.duration_secs));
    output.push_str(&format!("- **Timestamp**: {}\n", report.timestamp));

    output.push_str("\n## Check Results\n\n");

    for check in &report.checks {
        let symbol = match check.status {
            CheckStatus::Passed => "✅",
            CheckStatus::Failed => "❌",
        };
        let status = format!("{:?}", check.status).to_uppercase();

        output.push_str(&format!("### {} {} - {}\n\n", symbol, check.name, status));
        output.push_str(&format!(
            "- **Duration**: {:.2}s\n",
            check.duration.as_secs_f64()
        ));

        if !check.errors.is_empty() {
            output.push_str("\n**Errors:**\n\n");
            for error in &check.errors {
                output.push_str(&format!("- {}\n", error));
            }
        }

        if !check.output.is_empty() {
            output.push_str("\n<details>\n<summary>View Output</summary>\n\n```\n");
            output.push_str(&check.output);
            output.push_str("\n```\n\n</details>\n");
        }

        output.push('\n');
    }

    output.push_str(&format!(
        "\n---\n\n*Generated by devcheck v{}*\n",
        report.version
    ));

    println!("{}", output);
    Ok(())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
        .replace('\n', "<br>")
        .replace('\r', "")
}
