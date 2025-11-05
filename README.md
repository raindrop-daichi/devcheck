# devcheck

A CLI tool for parallel execution of Rust quality checks (fmt/clippy/test) with formatted summary output.

## Installation

```bash
cargo install --path .
```

## Usage

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

## Features

- 🚀 **Parallel Execution**: Runs multiple checks simultaneously for faster feedback
- 🎨 **Beautiful Output**: Color-coded, formatted terminal output
- 📊 **Summary Reports**: Clear overview of passed/failed checks
- 🔧 **Flexible**: Run all checks or select specific ones
- 📦 **CI/CD Ready**: JSON output format for automation

## Design

See [DESIGN.md](DESIGN.md) for detailed design documentation.

## License

MIT