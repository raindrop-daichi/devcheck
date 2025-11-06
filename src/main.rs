mod checker;
mod cli;
mod config;
mod error;
mod report;
mod runner;
mod utils;

use anyhow::Result;
use clap::Parser;
use notify::{Event, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    let config = config::Config::from_args(&args)?;

    if config.watch {
        run_watch_mode(config).await?;
    } else {
        let exit_code = run_checks_once(&config).await?;
        std::process::exit(exit_code);
    }

    Ok(())
}

async fn run_checks_once(config: &config::Config) -> Result<i32> {
    let results = runner::run_checks(config).await?;
    let report = report::Report::from_results(results, config.start_time);
    report::display_report(&report, config)?;
    Ok(if report.has_failures() { 1 } else { 0 })
}

async fn run_watch_mode(config: config::Config) -> Result<()> {
    println!("👀 Watch mode enabled. Monitoring for changes...\n");

    // Run checks initially
    let _ = run_checks_once(&config).await?;

    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            // Only trigger on modify and create events for Rust files
            if matches!(
                event.kind,
                notify::EventKind::Modify(_) | notify::EventKind::Create(_)
            ) && event.paths.iter().any(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "rs" || e == "toml")
                    .unwrap_or(false)
            }) {
                let _ = tx.send(());
            }
        }
    })?;

    watcher.watch(std::path::Path::new("."), RecursiveMode::Recursive)?;

    let mut last_run = std::time::Instant::now();

    loop {
        if rx.recv_timeout(Duration::from_millis(100)).is_ok() {
            // Debounce: only run if at least 1 second has passed since last run
            if last_run.elapsed() > Duration::from_secs(1) {
                println!("\n📝 File change detected. Re-running checks...\n");

                // Update start time for new run
                let mut new_config = config.clone();
                new_config.start_time = std::time::Instant::now();

                let _ = run_checks_once(&new_config).await?;
                last_run = std::time::Instant::now();
            }
        }
    }
}
