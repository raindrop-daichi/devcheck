# devcheck

A CLI tool for parallel execution of Rust quality checks (fmt/clippy/test) with formatted summary output.

## Installation

```bash
cargo install --path .
```

## Usage

### Basic Usage

```bash
# Run all default checks (fmt, clippy, test)
devcheck

# Run specific checks
devcheck --fmt
devcheck --clippy
devcheck --test
devcheck --fmt --clippy

# JSON output for CI/CD
devcheck --format json

# Verbose output
devcheck --verbose

# Quiet mode (errors only)
devcheck --quiet
```

### Workspace Support

```bash
# Run checks on all workspace members
devcheck --workspace

# Run specific checks on workspace
devcheck --workspace --fmt --clippy
```

### Configuration File

Create a `.devcheck.toml` file in your project root to customize default behavior:

```toml
[devcheck]
timeout = 300
parallel = true

[devcheck.fmt]
enabled = true
args = []

[devcheck.clippy]
enabled = true
args = ["--", "-D", "warnings"]

[devcheck.test]
enabled = true
args = []

[devcheck.build]
enabled = false
args = []
```

See `.devcheck.toml.example` for a complete example.

## Features

- 🚀 **Parallel Execution**: Runs multiple checks simultaneously for faster feedback
- 🎨 **Beautiful Output**: Color-coded, formatted terminal output with progress indicators
- 📊 **Summary Reports**: Clear overview of passed/failed checks
- 🔧 **Flexible**: Run all checks or select specific ones
- 📦 **CI/CD Ready**: JSON output format for automation
- ⚙️ **Configuration File**: Customize behavior with `.devcheck.toml`
- 🏢 **Workspace Support**: Run checks across all workspace members
- 📈 **Progress Bars**: Real-time progress indicators for running checks

## Development Status

### Phase 1: Complete ✅
- Basic fmt/clippy/test execution
- Parallel execution
- Terminal output
- JSON output

### Phase 2: Complete ✅
- Configuration file support (`.devcheck.toml`)
- Workspace support
- Progress bars

### Phase 3: Planned 🔄
- Custom check addition functionality
- Watch mode (file change detection)
- HTML/Markdown report generation

## Design

See [DESIGN.md](DESIGN.md) for detailed design documentation.

## License

MIT