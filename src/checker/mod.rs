mod build;
mod clippy;
mod fmt;
mod test;

pub use build::BuildChecker;
pub use clippy::ClippyChecker;
pub use fmt::FmtChecker;
pub use test::TestChecker;

use crate::config::Config;
use anyhow::Result;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    #[serde(serialize_with = "serialize_duration")]
    pub duration: Duration,
    pub output: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum CheckStatus {
    Passed,
    Failed,
    Skipped,
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_f64(duration.as_secs_f64())
}

#[async_trait::async_trait]
pub trait Checker: Send + Sync {
    fn name(&self) -> &str;
    async fn run(&self, config: &Config) -> Result<CheckResult>;
}
