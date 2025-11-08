use crate::cli::Args;
use crate::error::{DevCheckError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
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
    // Note: jobs field is reserved for future use to control parallelism
    pub jobs: Option<usize>,
    pub timeout: u64,
    pub workspace: bool,
    pub watch: bool,
    pub custom_checks: std::collections::HashMap<String, CustomCheckConfig>,
    pub start_time: Instant,
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Terminal,
    Json,
    Html,
    Markdown,
}

#[derive(Debug, Clone)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

/// Configuration file format (.devcheck.toml)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub devcheck: DevCheckConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DevCheckConfig {
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_true")]
    pub parallel: bool,
    #[serde(default)]
    pub fmt: CheckConfig,
    #[serde(default)]
    pub clippy: CheckConfig,
    #[serde(default)]
    pub test: CheckConfig,
    #[serde(default)]
    pub build: CheckConfig,
    #[serde(default)]
    pub custom: std::collections::HashMap<String, CustomCheckConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomCheckConfig {
    pub command: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub args: Vec<String>,
}

fn default_timeout() -> u64 {
    300
}

fn default_true() -> bool {
    true
}

impl Default for DevCheckConfig {
    fn default() -> Self {
        Self {
            timeout: 300,
            parallel: true,
            fmt: CheckConfig::default(),
            clippy: CheckConfig::default(),
            test: CheckConfig::default(),
            build: CheckConfig {
                enabled: false,
                args: vec![],
            },
            custom: std::collections::HashMap::new(),
        }
    }
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            args: vec![],
        }
    }
}

impl Config {
    pub fn from_args(args: &Args) -> Result<Self> {
        // Try to load config file
        let config_file = Self::load_config_file()?;

        // Determine which checks to run
        let (run_fmt, run_clippy, run_test, run_build) = if args.no_default {
            // If no_default is set, only run explicitly specified checks
            (args.fmt, args.clippy, args.test, args.build)
        } else if args.fmt || args.clippy || args.test || args.build {
            // If any check is explicitly specified, run only those
            (args.fmt, args.clippy, args.test, args.build)
        } else {
            // Use config file or defaults
            (
                config_file.devcheck.fmt.enabled,
                config_file.devcheck.clippy.enabled,
                config_file.devcheck.test.enabled,
                config_file.devcheck.build.enabled,
            )
        };

        let format = match args.format.as_str() {
            "terminal" => OutputFormat::Terminal,
            "json" => OutputFormat::Json,
            "html" => OutputFormat::Html,
            "markdown" | "md" => OutputFormat::Markdown,
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

        // Use timeout from args or config file
        let timeout = if args.timeout != 300 {
            args.timeout
        } else {
            config_file.devcheck.timeout
        };

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
            timeout,
            workspace: args.workspace,
            watch: args.watch,
            custom_checks: config_file.devcheck.custom.clone(),
            start_time: Instant::now(),
        })
    }

    fn load_config_file() -> Result<ConfigFile> {
        let config_path = Path::new(".devcheck.toml");

        if config_path.exists() {
            let content = fs::read_to_string(config_path).map_err(|e| {
                DevCheckError::ConfigError(format!("Failed to read .devcheck.toml: {}", e))
            })?;

            let config: ConfigFile = toml::from_str(&content).map_err(|e| {
                DevCheckError::ConfigError(format!("Failed to parse .devcheck.toml: {}", e))
            })?;

            Ok(config)
        } else {
            // Return default config if no file exists
            Ok(ConfigFile {
                devcheck: DevCheckConfig::default(),
            })
        }
    }
}
