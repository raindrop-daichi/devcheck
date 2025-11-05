use super::{CheckResult, CheckStatus, Checker};
use crate::config::Config;
use anyhow::Result;
use std::time::Instant;
use tokio::process::Command;

pub struct FmtChecker;

#[async_trait::async_trait]
impl Checker for FmtChecker {
    fn name(&self) -> &str {
        "fmt"
    }

    async fn run(&self, config: &Config) -> Result<CheckResult> {
        let start = Instant::now();

        let mut cmd = Command::new("cargo");
        cmd.arg("fmt");

        if config.workspace {
            cmd.arg("--all");
        }

        cmd.arg("--").arg("--check");

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
            vec!["Code formatting issues found".to_string()]
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
