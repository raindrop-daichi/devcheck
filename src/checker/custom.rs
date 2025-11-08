use super::{CheckResult, CheckStatus, Checker};
use crate::config::{Config, CustomCheckConfig};
use anyhow::Result;
use std::time::Instant;
use tokio::process::Command;

pub struct CustomChecker {
    name: String,
    config: CustomCheckConfig,
}

impl CustomChecker {
    pub fn new(name: String, config: CustomCheckConfig) -> Self {
        Self { name, config }
    }
}

#[async_trait::async_trait]
impl Checker for CustomChecker {
    fn name(&self) -> &str {
        &self.name
    }

    async fn run(&self, config: &Config) -> Result<CheckResult> {
        let start = Instant::now();

        let mut cmd = Command::new(&self.config.command);

        for arg in &self.config.args {
            cmd.arg(arg);
        }

        let output =
            tokio::time::timeout(std::time::Duration::from_secs(config.timeout), cmd.output())
                .await
                .map_err(|_| {
                    anyhow::anyhow!("Check timed out after {} seconds", config.timeout)
                })??;
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
            vec![format!("Custom check '{}' failed", self.name)]
        } else {
            vec![]
        };

        Ok(CheckResult {
            name: self.name.clone(),
            status,
            duration,
            output: combined_output,
            errors,
        })
    }
}
