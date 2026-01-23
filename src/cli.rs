use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "devcheck")]
#[command(about = "A CLI tool for parallel execution of Rust quality checks", long_about = None)]
pub struct Args {
    /// Run cargo fmt check
    #[arg(long)]
    pub fmt: bool,

    /// Run cargo clippy
    #[arg(long)]
    pub clippy: bool,

    /// Run cargo test
    #[arg(long)]
    pub test: bool,

    /// Run cargo build
    #[arg(long)]
    pub build: bool,

    /// Disable all default checks (use with specific check flags)
    #[arg(long)]
    pub no_default: bool,

    /// Output format (terminal, json, html, or markdown)
    #[arg(long, default_value = "terminal")]
    pub format: String,

    /// Quiet mode (errors only)
    #[arg(long, short)]
    pub quiet: bool,

    /// Verbose output
    #[arg(long, short)]
    pub verbose: bool,

    /// Color output (auto, always, never)
    #[arg(long, default_value = "auto")]
    pub color: String,

    /// Path to Cargo.toml
    #[arg(long)]
    pub manifest_path: Option<String>,

    /// Run checks on all workspace members
    #[arg(long)]
    pub workspace: bool,

    /// Watch mode - re-run checks when files change
    #[arg(long, short)]
    pub watch: bool,

    /// Number of parallel jobs
    #[arg(long, short)]
    pub jobs: Option<usize>,

    /// Timeout in seconds for each check
    #[arg(long, default_value = "300")]
    pub timeout: u64,
}
