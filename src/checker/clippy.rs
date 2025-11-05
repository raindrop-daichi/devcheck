use super::{CheckResult, CheckStatus, Checker};
use crate::config::Config;
use anyhow::Result;
use std::time::Instant;
use tokio::process::Command;

pub struct ClippyChecker;

#[async_trait::async_trait]
impl Checker for ClippyChecker {
    fn name(&self) -> &str {
        "clippy"
    }

    async fn run(&self, config: &Config) -> Result<CheckResult> {
        let start = Instant::now();

        let mut cmd = Command::new("cargo");
        cmd.arg("clippy");

        if config.workspace {
            cmd.arg("--workspace");
        }

        cmd.arg("--").arg("-D").arg("warnings");

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
            // Try to extract error count from output
            let error_lines: Vec<String> = combined_output
                .lines()
                .filter(|line| line.contains("error:") || line.contains("warning:"))
                .take(5)
                .map(|s| s.to_string())
                .collect();

            if error_lines.is_empty() {
                vec!["Clippy checks failed".to_string()]
            } else {
                error_lines
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
