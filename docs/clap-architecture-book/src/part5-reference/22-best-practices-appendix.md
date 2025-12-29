# Best Practices & Appendices

> **Chapter 22** | Part 5: Reference & Appendices | Estimated reading time: 16 minutes

This appendix consolidates best practices from throughout the book into actionable checklists, troubleshooting guides, and quick reference materials for ongoing use.

## Design Checklist

Use this checklist when designing a new CLI application or reviewing an existing one.

### Pre-Design Phase

- [ ] **Define target users**: Developer? Operator? End-user? All three?
- [ ] **Identify use cases**: What are the primary workflows?
- [ ] **Review similar tools**: What conventions do they follow?
- [ ] **Plan extensibility**: Will the CLI grow over time?
- [ ] **Consider automation**: Will it be used in scripts and pipelines?

### Architecture Decisions

- [ ] **Choose derive vs builder**: Derive for static CLIs, builder for dynamic
- [ ] **Plan subcommand hierarchy**: Flat, nested, or hybrid?
- [ ] **Design configuration layering**: CLI > env > config file > defaults
- [ ] **Plan error strategy**: Custom errors vs Clap defaults
- [ ] **Consider multi-binary**: Separate binaries or single with subcommands?

### Argument Design

- [ ] **Use clear, verb-based command names**: `create`, `delete`, `list`
- [ ] **Group related options logically**: Authentication options together
- [ ] **Provide sensible defaults**: Most common use case works out of box
- [ ] **Make destructive operations explicit**: Require `--force` or `--yes`
- [ ] **Support both short and long flags**: `-v` and `--verbose`
- [ ] **Use consistent naming**: `--input-file`, not `--inputFile` or `--input_file`

### Help and Documentation

- [ ] **Write clear, concise help text**: One sentence per argument
- [ ] **Include examples in `long_help`**: Show real usage patterns
- [ ] **Document environment variables**: Show in help output
- [ ] **Provide version information**: `--version` with meaningful output
- [ ] **Generate man pages**: Use `clap_mangen` for distribution
- [ ] **Generate shell completions**: Use `clap_complete`

### Error Handling

- [ ] **Provide specific error messages**: "Port 70000 is out of range (1-65535)"
- [ ] **Use appropriate exit codes**: 0 success, 1 general error, 2 usage error
- [ ] **Include suggestions**: "Did you mean '--verbose'?"
- [ ] **Support debug output**: `-vvv` for increasing verbosity
- [ ] **Log errors appropriately**: stderr for errors, stdout for output

## Performance Checklist

### Startup Time Optimization

- [ ] **Profile startup**: `hyperfine --warmup 3 './myapp --help'`
- [ ] **Minimize dependencies**: Use `default-features = false`
- [ ] **Defer expensive initialization**: Lazy-load resources after parsing
- [ ] **Consider compile-time vs runtime**: Fewer features = faster startup

```toml
# Optimize dependencies
[dependencies]
clap = { version = "4", default-features = false, features = ["derive", "std"] }
```

### Binary Size Optimization

- [ ] **Enable LTO**: Link-time optimization reduces size
- [ ] **Strip symbols**: Remove debug symbols from release builds
- [ ] **Use size optimization**: `opt-level = "z"` for size
- [ ] **Audit dependencies**: `cargo bloat` shows size contributors

```toml
# Cargo.toml profile for small binaries
[profile.release]
lto = true
codegen-units = 1
opt-level = "z"
strip = true
panic = "abort"
```

### Memory Optimization

- [ ] **Use borrowed types**: `&str` instead of `String` where possible
- [ ] **Avoid unnecessary cloning**: Pass references, not owned values
- [ ] **Stream large inputs**: Process iteratively, not all at once
- [ ] **Consider arenas**: For many small allocations

## Security Checklist

### Credential Handling

- [ ] **Never log credentials**: Mask in debug output
- [ ] **Use `secrecy` crate**: For sensitive string handling
- [ ] **Clear memory**: Zero sensitive data when done
- [ ] **Support credential helpers**: Git-credential-style patterns
- [ ] **Avoid environment variables for secrets**: They're visible in /proc

```rust
use secrecy::{Secret, ExposeSecret};

#[derive(Parser)]
struct Cli {
    #[arg(long, env = "API_TOKEN")]
    api_token: Secret<String>,
}

fn use_token(cli: &Cli) {
    // Only expose when absolutely necessary
    let token = cli.api_token.expose_secret();
    // Use token...
}
```

### Input Validation

- [ ] **Validate all user input**: Never trust external data
- [ ] **Sanitize paths**: Prevent directory traversal attacks
- [ ] **Set reasonable limits**: Maximum file sizes, string lengths
- [ ] **Use typed parsing**: Let Clap catch type errors early
- [ ] **Validate UTF-8**: Use `String` for text, `OsString` for paths

```rust
// Path validation example
fn validate_path(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    let canonical = path.canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;

    if !canonical.starts_with("/allowed/directory") {
        return Err("Path must be within /allowed/directory".to_string());
    }
    Ok(canonical)
}
```

### Execution Safety

- [ ] **Require confirmation**: `--yes` or `--force` for destructive operations
- [ ] **Support dry-run**: `--dry-run` to preview changes
- [ ] **Log actions**: Create audit trails for sensitive operations
- [ ] **Handle signals gracefully**: Catch Ctrl+C, clean up resources
- [ ] **Avoid shell injection**: Never pass user input directly to shell

```rust
use ctrlc;

fn main() {
    ctrlc::set_handler(move || {
        eprintln!("\nInterrupted. Cleaning up...");
        // Cleanup code
        std::process::exit(130);  // 128 + SIGINT
    }).expect("Error setting Ctrl-C handler");
}
```

## Accessibility Checklist

### Terminal Accessibility

- [ ] **Don't rely on color alone**: Use text indicators too
- [ ] **Support NO_COLOR**: Respect the environment variable
- [ ] **Ensure contrast**: Test with various terminal themes
- [ ] **Work without mouse**: All interactions via keyboard
- [ ] **Support screen readers**: Structured, semantic output

```rust
#[derive(Parser)]
struct Cli {
    /// Disable colored output
    #[arg(long, env = "NO_COLOR")]
    no_color: bool,

    /// Plain text output (no formatting)
    #[arg(long)]
    plain: bool,
}

fn should_use_color(cli: &Cli) -> bool {
    !cli.no_color && !cli.plain && atty::is(atty::Stream::Stdout)
}
```

### Documentation Accessibility

- [ ] **Use simple language**: Avoid jargon where possible
- [ ] **Provide examples**: Show, don't just tell
- [ ] **Document keyboard shortcuts**: For interactive features
- [ ] **Alternative formats**: Man pages, web docs, `--help`

### Internationalization Preparation

- [ ] **Externalize strings**: Prepare for future translation
- [ ] **Handle Unicode**: Support non-ASCII input and output
- [ ] **Consider locales**: Date, time, number formatting

## Common Anti-Patterns

### Anti-Pattern 1: Flag Overloading

```rust
// BAD: One flag does too much
#[arg(long)]
mode: String,  // "fast", "slow", "safe", "debug", "verbose"...

// GOOD: Separate concerns
#[arg(long, value_enum)]
speed: Speed,

#[arg(long)]
safe_mode: bool,

#[arg(short, long, action = ArgAction::Count)]
verbose: u8,
```

### Anti-Pattern 2: Inconsistent Naming

```rust
// BAD: Mixed conventions
#[arg(long = "inputFile")]     // camelCase
input_file: PathBuf,

#[arg(long = "output-dir")]    // kebab-case
output_dir: PathBuf,

#[arg(long = "MAX_SIZE")]      // SCREAMING
max_size: usize,

// GOOD: Consistent kebab-case (Clap default)
#[arg(long)]
input_file: PathBuf,  // Becomes --input-file

#[arg(long)]
output_dir: PathBuf,  // Becomes --output-dir

#[arg(long)]
max_size: usize,      // Becomes --max-size
```

### Anti-Pattern 3: Silent Failures

```rust
// BAD: Silently uses default
#[arg(long, default_value = "/tmp")]
output_dir: PathBuf,

// GOOD: Document defaults clearly
/// Output directory [default: /tmp]
#[arg(long, default_value = "/tmp")]
output_dir: PathBuf,

// BETTER: Warn about significant defaults
fn main() {
    let cli = Cli::parse();
    if !cli.output_dir_set {
        eprintln!("Note: Using default output directory: /tmp");
    }
}
```

### Anti-Pattern 4: Monolithic Commands

```rust
// BAD: Everything in one command
myapp --create-user --delete-user --list-users --export --import

// GOOD: Subcommand structure
myapp user create
myapp user delete
myapp user list
myapp data export
myapp data import
```

### Anti-Pattern 5: Ignoring Exit Codes

```rust
// BAD: Always exits 0
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

// GOOD: Meaningful exit codes
fn main() {
    match run() {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
```

## Troubleshooting Guide

### Compilation Errors

| Error Message | Cause | Solution |
|---------------|-------|----------|
| `cannot find derive macro Parser` | Missing feature | Add `features = ["derive"]` |
| `cannot find attribute command` | Wrong Clap version | Use Clap 4+ |
| `conflicting implementations` | Duplicate impls | Remove custom ValueParser |
| `expected struct, found enum` | Wrong derive | Use `#[derive(Subcommand)]` for enums |

### Runtime Issues

| Symptom | Cause | Solution |
|---------|-------|----------|
| Arguments not recognized | Missing short/long | Add `#[arg(short, long)]` |
| Required arg not enforced | Option type | Use `T` not `Option<T>` |
| Env var not read | Missing feature | Add `features = ["env"]` |
| Help shows wrong name | Cargo.toml mismatch | Check `[package] name` |

### Testing Issues

| Symptom | Cause | Solution |
|---------|-------|----------|
| `try_parse_from` panics | Using `parse_from` | Switch to `try_parse_from` |
| Tests pass, binary fails | Different env | Check env vars in tests |
| Snapshot tests fail | Version change | Update snapshots after upgrade |

## Quick Reference Cards

### The Good CLI Checklist

1. **Discoverable**: `--help` explains everything
2. **Predictable**: Consistent behavior and naming
3. **Composable**: Works well with pipes and scripts
4. **Forgiving**: Helpful errors and suggestions
5. **Efficient**: Fast startup, reasonable resources
6. **Accessible**: Works for all users
7. **Secure**: Safe credential handling
8. **Tested**: Comprehensive coverage

### Exit Code Reference

| Code | Meaning | When to Use |
|------|---------|-------------|
| `0` | Success | Operation completed successfully |
| `1` | General error | Unspecified failure |
| `2` | Usage error | Invalid arguments or options |
| `64-78` | BSD sysexits | Specific error categories |
| `126` | Cannot execute | Permission denied |
| `127` | Command not found | Missing dependency |
| `128+N` | Signal N | Terminated by signal |
| `130` | SIGINT (Ctrl+C) | User interrupted |

### Testing Patterns Summary

| Test Type | Tool | Purpose |
|-----------|------|---------|
| Unit tests | `try_parse_from` | Argument parsing |
| Integration | `assert_cmd` | End-to-end behavior |
| Snapshot | `insta` | Help output stability |
| Fuzzing | `cargo-fuzz` | Edge case discovery |

## Resources and Further Reading

### Official Resources

- [Clap Documentation](https://docs.rs/clap) - Complete API reference
- [Clap Derive Tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html) - Official derive guide
- [Clap GitHub](https://github.com/clap-rs/clap) - Source and issues
- [Clap CHANGELOG](https://github.com/clap-rs/clap/blob/master/CHANGELOG.md) - Version history

### Companion Crates

| Crate | Purpose | Documentation |
|-------|---------|---------------|
| `clap_complete` | Shell completions | [docs.rs](https://docs.rs/clap_complete) |
| `clap_mangen` | Man page generation | [docs.rs](https://docs.rs/clap_mangen) |
| `clap_lex` | Low-level tokenization | [docs.rs](https://docs.rs/clap_lex) |

### Ecosystem Crates

| Crate | Purpose |
|-------|---------|
| `indicatif` | Progress bars and spinners |
| `dialoguer` | Interactive prompts |
| `console` | Terminal styling |
| `anyhow` | Ergonomic error handling |
| `thiserror` | Custom error types |
| `config` | Configuration file parsing |
| `figment` | Layered configuration |
| `tracing` | Structured logging |

### Learning Resources

- [Command Line Interface Guidelines](https://clig.dev/) - Modern CLI design
- [The Art of Unix Programming](http://www.catb.org/~esr/writings/taoup/html/) - CLI philosophy
- [12 Factor CLI Apps](https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46) - Best practices
- [Rust CLI Book](https://rust-cli.github.io/book/) - Rust-specific guide

## See Also

- [Chapter 1: Understanding Clap's Philosophy](../part1-foundations/01-clap-philosophy.md) - Foundational concepts
- [Chapter 15: Testing CLI Applications](../part3-advanced-architecture/15-testing-cli-applications.md) - Testing strategies
- [Chapter 19: Performance Optimization](../part4-real-world-systems/19-performance-optimization.md) - Performance deep dive
- [Chapter 20: API Quick Reference](./20-api-quick-reference.md) - API lookup

---

*Return to [Introduction](../introduction.md) | [API Quick Reference](./20-api-quick-reference.md)*
