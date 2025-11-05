use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum DevCheckError {
    #[error("cargo command not found - please install Rust and cargo")]
    CargoNotFound,

    #[error("failed to execute {command}: {source}")]
    ExecutionError {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("check timed out after {timeout} seconds")]
    Timeout { timeout: u64 },

    #[error("invalid configuration: {0}")]
    ConfigError(String),

    #[error("Cargo.toml not found in {0}")]
    ManifestNotFound(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, DevCheckError>;
