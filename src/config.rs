use crate::cli::Args;
use crate::error::{DevCheckError, Result};
use std::time::Instant;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub run_fmt: bool,
    pub run_clippy: bool,
    pub run_test: bool,
    pub run_build: bool,
    pub format: OutputFormat,
    pub quiet: bool,
    pub verbose: bool,
    pub color: ColorMode,
    pub manifest_path: Option<String>,
    pub jobs: Option<usize>,
    pub timeout: u64,
    pub start_time: Instant,
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Terminal,
    Json,
}

#[derive(Debug, Clone)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl Config {
    pub fn from_args(args: &Args) -> Result<Self> {
        // Determine which checks to run
        let (run_fmt, run_clippy, run_test, run_build) = if args.no_default {
            // If no_default is set, only run explicitly specified checks
            (args.fmt, args.clippy, args.test, args.build)
        } else if args.fmt || args.clippy || args.test || args.build {
            // If any check is explicitly specified, run only those
            (args.fmt, args.clippy, args.test, args.build)
        } else {
            // Default: run fmt, clippy, and test (but not build)
            (true, true, true, false)
        };

        let format = match args.format.as_str() {
            "terminal" => OutputFormat::Terminal,
            "json" => OutputFormat::Json,
            _ => {
                return Err(DevCheckError::ConfigError(format!(
                    "invalid format: {}",
                    args.format
                )))
            }
        };

        let color = match args.color.as_str() {
            "auto" => ColorMode::Auto,
            "always" => ColorMode::Always,
            "never" => ColorMode::Never,
            _ => {
                return Err(DevCheckError::ConfigError(format!(
                    "invalid color mode: {}",
                    args.color
                )))
            }
        };

        // Verify cargo is available
        if which::which("cargo").is_err() {
            return Err(DevCheckError::CargoNotFound);
        }

        Ok(Config {
            run_fmt,
            run_clippy,
            run_test,
            run_build,
            format,
            quiet: args.quiet,
            verbose: args.verbose,
            color,
            manifest_path: args.manifest_path.clone(),
            jobs: args.jobs,
            timeout: args.timeout,
            start_time: Instant::now(),
        })
    }
}
