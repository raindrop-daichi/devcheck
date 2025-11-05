mod checker;
mod cli;
mod config;
mod error;
mod report;
mod runner;
mod utils;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    let config = config::Config::from_args(&args)?;

    let results = runner::run_checks(&config).await?;

    let report = report::Report::from_results(results, config.start_time);

    report::display_report(&report, &config)?;

    std::process::exit(if report.has_failures() { 1 } else { 0 });
}
