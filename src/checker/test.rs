use super::{CheckResult, CheckStatus, Checker};
use crate::config::Config;
use anyhow::Result;
use std::time::Instant;
use tokio::process::Command;

pub struct TestChecker;

#[async_trait::async_trait]
impl Checker for TestChecker {
    fn name(&self) -> &str {
        "test"
    }

    async fn run(&self, config: &Config) -> Result<CheckResult> {
        let start = Instant::now();

        let mut cmd = Command::new("cargo");
        cmd.arg("test");

        if config.workspace {
            cmd.arg("--workspace");
        }

        if let Some(ref path) = config.manifest_path {
            cmd.arg("--manifest-path").arg(path);
        }

        let output = cmd.output().await?;
        let duration = start.elapsed();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined_output = format!("{}{}", stdout, stderr);

        let status = if output.status.success() {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        };

        let errors = if status == CheckStatus::Failed {
            // Try to extract failed test names
            let failed_tests: Vec<String> = combined_output
                .lines()
                .filter(|line| line.contains("test ") && line.contains("FAILED"))
                .take(10)
                .map(|s| s.trim().to_string())
                .collect();

            if failed_tests.is_empty() {
                vec!["Some tests failed".to_string()]
            } else {
                failed_tests
            }
        } else {
            vec![]
        };

        Ok(CheckResult {
            name: self.name().to_string(),
            status,
            duration,
            output: combined_output,
            errors,
        })
    }
}
