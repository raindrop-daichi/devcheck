use crate::checker::{BuildChecker, CheckResult, Checker, ClippyChecker, FmtChecker, TestChecker};
use crate::config::{Config, OutputFormat};
use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::task::JoinSet;

pub async fn run_checks(config: &Config) -> Result<Vec<CheckResult>> {
    let mut tasks = JoinSet::new();

    // Create progress bars if not in quiet mode and using terminal output
    let multi_progress = if !config.quiet && matches!(config.format, OutputFormat::Terminal) {
        Some(MultiProgress::new())
    } else {
        None
    };

    // Spawn tasks for enabled checks
    if config.run_fmt {
        let config = config.clone();
        let pb = create_progress_bar(&multi_progress, "fmt");
        tasks.spawn(async move {
            let checker = FmtChecker;
            let result = checker.run(&config).await;
            if let Some(pb) = pb {
                pb.finish_and_clear();
            }
            result
        });
    }

    if config.run_clippy {
        let config = config.clone();
        let pb = create_progress_bar(&multi_progress, "clippy");
        tasks.spawn(async move {
            let checker = ClippyChecker;
            let result = checker.run(&config).await;
            if let Some(pb) = pb {
                pb.finish_and_clear();
            }
            result
        });
    }

    if config.run_test {
        let config = config.clone();
        let pb = create_progress_bar(&multi_progress, "test");
        tasks.spawn(async move {
            let checker = TestChecker;
            let result = checker.run(&config).await;
            if let Some(pb) = pb {
                pb.finish_and_clear();
            }
            result
        });
    }

    if config.run_build {
        let config = config.clone();
        let pb = create_progress_bar(&multi_progress, "build");
        tasks.spawn(async move {
            let checker = BuildChecker;
            let result = checker.run(&config).await;
            if let Some(pb) = pb {
                pb.finish_and_clear();
            }
            result
        });
    }

    // Collect results
    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(check_result)) => results.push(check_result),
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(e.into()),
        }
    }

    // Clear progress bars
    if let Some(mp) = multi_progress {
        mp.clear().ok();
    }

    // Sort results by name for consistent output
    results.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(results)
}

fn create_progress_bar(
    multi_progress: &Option<MultiProgress>,
    check_name: &str,
) -> Option<ProgressBar> {
    multi_progress.as_ref().map(|mp| {
        let pb = mp.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.blue} {msg}")
                .unwrap(),
        );
        pb.set_message(format!("Running cargo {}...", check_name));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    })
}
