use crate::checker::{BuildChecker, CheckResult, Checker, ClippyChecker, FmtChecker, TestChecker};
use crate::config::Config;
use anyhow::Result;
use tokio::task::JoinSet;

pub async fn run_checks(config: &Config) -> Result<Vec<CheckResult>> {
    let mut tasks = JoinSet::new();

    // Spawn tasks for enabled checks
    if config.run_fmt {
        let config = config.clone();
        tasks.spawn(async move {
            let checker = FmtChecker;
            checker.run(&config).await
        });
    }

    if config.run_clippy {
        let config = config.clone();
        tasks.spawn(async move {
            let checker = ClippyChecker;
            checker.run(&config).await
        });
    }

    if config.run_test {
        let config = config.clone();
        tasks.spawn(async move {
            let checker = TestChecker;
            checker.run(&config).await
        });
    }

    if config.run_build {
        let config = config.clone();
        tasks.spawn(async move {
            let checker = BuildChecker;
            checker.run(&config).await
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

    // Sort results by name for consistent output
    results.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(results)
}
