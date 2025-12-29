# Build Instructions

This document provides instructions for building the Clap Architecture Book locally.

## Prerequisites

### Required Tools

1. **Rust and Cargo** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustc --version  # Verify installation
   ```

2. **mdbook** (0.5+)
   ```bash
   cargo install mdbook
   mdbook --version  # Verify installation
   ```

## Building the Book

### Quick Build

From the repository root:

```bash
cd docs/clap-architecture-book
mdbook build
```

The HTML output will be generated in `docs/clap-architecture-book/book/`.

### Development Server

For live preview with auto-reload:

```bash
cd docs/clap-architecture-book
mdbook serve
```

This starts a local server at `http://localhost:3000` with automatic rebuilding on file changes.

### Custom Port

```bash
mdbook serve -p 8080
```

## Building Code Examples

The accompanying code examples are in the `clap-examples/` directory:

```bash
cd clap-examples
cargo build --workspace      # Build all examples
cargo test --workspace       # Run all tests
cargo run -p hello-world     # Run specific example
```

### Example Projects

| Package Name | Description |
|--------------|-------------|
| `hello-world` | Basic Clap introduction |
| `arguments-basic` | Argument parsing fundamentals |
| `subcommands` | Subcommand architecture |
| `groups-and-conflicts` | Argument groups and conflicts |
| `custom-parsers` | Custom value parsers |
| `builder-pattern` | Builder API patterns |
| `derive-macros` | Derive macro usage |
| `environment-config` | Environment variables and config |
| `error-handling` | Error handling strategies |
| `plugins-architecture` | Plugin system design |
| `testing` | CLI testing patterns |
| `cloudctl` | Real-world project example |

## Build Options

### Clean Build

```bash
mdbook clean
mdbook build
```

### Build with Verbose Output

```bash
RUST_LOG=mdbook=debug mdbook build
```

### Test Code Examples in Book

```bash
mdbook test
```

Note: This runs any Rust code blocks marked as testable.

## Output Formats

### HTML (Default)

```bash
mdbook build
# Output: book/
```

### Print Version

The HTML build includes a print-friendly version at `book/print.html`.

## Troubleshooting

### Common Issues

**Issue: `mdbook: command not found`**
```bash
# Ensure cargo bin is in PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

**Issue: Build warnings about unclosed HTML tags**

These warnings occur when generic type notation like `<T>` appears in markdown headings or emphasis. The build still succeeds.

**Issue: Missing chapters**

If `create-missing = true` is set in book.toml, missing chapters are auto-generated. Check `src/SUMMARY.md` for expected chapters.

### Verify Installation

```bash
mdbook --version      # Should be 0.5.x
rustc --version       # Should be 1.70+
cargo --version       # Should match rustc
```

## Continuous Integration

For CI/CD pipelines:

```bash
#!/bin/bash
set -e

# Install mdbook
cargo install mdbook

# Build book
cd docs/clap-architecture-book
mdbook build

# Optional: Run book tests
mdbook test

# Verify examples compile
cd ../../clap-examples
cargo build --workspace
cargo test --workspace
```

## Additional Resources

- [mdbook Documentation](https://rust-lang.github.io/mdBook/)
- [Clap Documentation](https://docs.rs/clap/)
- [Rust Book](https://doc.rust-lang.org/book/)
