# Error Handling Foundations

> **Chapter 5** | Part 1: Foundations | Estimated reading time: 13 minutes

Effective error handling makes the difference between a frustrating CLI and a delightful one. When users make mistakes, the quality of error messages determines whether they can self-correct or need to search for help. This chapter establishes the foundations of Clap's error handling system and best practices for creating user-friendly CLI applications.

## Clap's Error Types

### The Error Architecture

Clap provides a comprehensive error system designed for both user-facing messages and programmatic handling. Understanding this architecture is essential for building robust CLIs.

```
┌─────────────────────────────────────────────────────────────────┐
│                    CLAP ERROR ARCHITECTURE                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   User Input                                                    │
│       │                                                         │
│       ▼                                                         │
│   ┌─────────────────┐                                          │
│   │   Clap Parser   │                                          │
│   └────────┬────────┘                                          │
│            │                                                    │
│            ▼                                                    │
│   ┌─────────────────┐     ┌──────────────────────────┐         │
│   │ Result<T, Error>│────▶│  clap::Error             │         │
│   └─────────────────┘     │  ├── kind: ErrorKind     │         │
│                           │  ├── context: Vec<...>   │         │
│                           │  ├── source: Option<...> │         │
│                           │  └── format(): String    │         │
│                           └──────────────────────────┘         │
│                                      │                          │
│                                      ▼                          │
│                           ┌──────────────────────────┐         │
│                           │  User-Friendly Output    │         │
│                           │  • Colored terminal      │         │
│                           │  • Context-aware         │         │
│                           │  • Suggests alternatives │         │
│                           └──────────────────────────┘         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Error Kinds

Clap categorizes errors by their nature using `ErrorKind`:

```rust
use clap::error::ErrorKind;

// Common error kinds and when they occur:

// User provided an invalid value for an argument
// ErrorKind::InvalidValue
// Example: --port abc (expected number)

// User provided an unknown argument
// ErrorKind::UnknownArgument
// Example: --unknwon-flag

// A required argument was not provided
// ErrorKind::MissingRequiredArgument
// Example: missing required positional argument

// Wrong number of values for an argument
// ErrorKind::WrongNumberOfValues
// Example: --pair a (expected 2 values)

// Two mutually exclusive arguments were used
// ErrorKind::ArgumentConflict
// Example: --json --yaml (exclusive formats)

// Help was requested
// ErrorKind::DisplayHelp
// Example: --help

// Version was requested
// ErrorKind::DisplayVersion
// Example: --version
```

### Working with Clap Errors

```rust
use clap::{Parser, error::ErrorKind};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    count: u32,
}

fn main() {
    // try_parse returns Result, allowing error handling
    match Cli::try_parse() {
        Ok(cli) => {
            println!("Count: {}", cli.count);
        }
        Err(error) => {
            // Programmatic error handling
            match error.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    // These are "successful" exits
                    print!("{}", error);
                    std::process::exit(0);
                }
                ErrorKind::MissingRequiredArgument => {
                    eprintln!("Error: Required argument missing");
                    eprintln!("Run with --help for usage information");
                    std::process::exit(2);
                }
                ErrorKind::InvalidValue => {
                    eprintln!("Error: Invalid value provided");
                    error.print().expect("Failed to print error");
                    std::process::exit(1);
                }
                _ => {
                    // Default error handling
                    error.exit();
                }
            }
        }
    }
}
```

### Error Propagation Patterns

Integrate Clap errors with Rust's error handling:

```rust
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// Input file to process
    #[arg(short, long)]
    input: PathBuf,

    /// Output file
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug)]
enum AppError {
    Cli(clap::Error),
    Io(std::io::Error),
    Processing(String),
}

impl From<clap::Error> for AppError {
    fn from(err: clap::Error) -> Self {
        AppError::Cli(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

fn run() -> Result<(), AppError> {
    let cli = Cli::try_parse()?;

    let content = fs::read_to_string(&cli.input)?;

    if content.is_empty() {
        return Err(AppError::Processing("Input file is empty".to_string()));
    }

    // Process content...
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        match e {
            AppError::Cli(err) => err.exit(),
            AppError::Io(err) => {
                eprintln!("IO error: {}", err);
                std::process::exit(1);
            }
            AppError::Processing(msg) => {
                eprintln!("Processing error: {}", msg);
                std::process::exit(1);
            }
        }
    }
}
```

## Custom Error Messages

### Value Validation Errors

Create descriptive errors during value parsing:

```rust
use clap::Parser;

fn parse_port(s: &str) -> Result<u16, String> {
    let port: u16 = s
        .parse()
        .map_err(|_| format!("'{}' is not a valid port number", s))?;

    if port == 0 {
        return Err("Port 0 is reserved and cannot be used".to_string());
    }

    if port < 1024 {
        return Err(format!(
            "Port {} requires root privileges. Use a port >= 1024",
            port
        ));
    }

    Ok(port)
}

fn parse_email(s: &str) -> Result<String, String> {
    if !s.contains('@') {
        return Err(format!(
            "'{}' is not a valid email address (missing @)",
            s
        ));
    }

    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return Err(format!(
            "'{}' is not a valid email address (multiple @ symbols)",
            s
        ));
    }

    if parts[0].is_empty() {
        return Err("Email local part (before @) cannot be empty".to_string());
    }

    if !parts[1].contains('.') {
        return Err(format!(
            "'{}' domain appears invalid (no TLD found)",
            parts[1]
        ));
    }

    Ok(s.to_string())
}

#[derive(Parser)]
struct ServerConfig {
    /// Port to listen on (1024-65535)
    #[arg(long, value_parser = parse_port)]
    port: u16,

    /// Admin email for notifications
    #[arg(long, value_parser = parse_email)]
    admin_email: String,
}

fn main() {
    let config = ServerConfig::parse();
    println!("Server on port {}, admin: {}", config.port, config.admin_email);
}
```

### Contextual Error Information

Add context to make errors more actionable:

```rust
use clap::{Parser, builder::TypedValueParser};
use std::path::PathBuf;

/// Custom parser that validates file exists and is readable
#[derive(Clone)]
struct ExistingFileParser;

impl TypedValueParser for ExistingFileParser {
    type Value = PathBuf;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let path = PathBuf::from(value);

        if !path.exists() {
            return Err(clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                format!(
                    "File '{}' does not exist.\n\
                     Hint: Check the path and ensure the file exists.\n\
                     Current directory: {:?}",
                    path.display(),
                    std::env::current_dir().unwrap_or_default()
                ),
            ));
        }

        if !path.is_file() {
            return Err(clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                format!(
                    "'{}' is a directory, not a file.\n\
                     Hint: Provide a path to a specific file.",
                    path.display()
                ),
            ));
        }

        // Check readability
        match std::fs::File::open(&path) {
            Ok(_) => Ok(path),
            Err(e) => Err(clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                format!(
                    "Cannot read file '{}': {}\n\
                     Hint: Check file permissions.",
                    path.display(),
                    e
                ),
            )),
        }
    }
}

#[derive(Parser)]
struct Cli {
    /// Configuration file (must exist and be readable)
    #[arg(long, value_parser = ExistingFileParser)]
    config: PathBuf,
}
```

### Error Formatting with Colors

Leverage terminal colors for visibility:

```rust
use clap::{Parser, ColorChoice};

#[derive(Parser)]
#[command(
    name = "myapp",
    // Control color output
    color = ColorChoice::Auto,
)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

// Clap automatically uses colors when:
// - stdout is a terminal (not redirected)
// - NO_COLOR environment variable is not set
// - --color=auto (default) or --color=always

// For custom error messages with colors:
fn custom_error(msg: &str) {
    // Using console crate for cross-platform colors
    // eprintln!("{}: {}", console::style("error").red().bold(), msg);

    // Or simple ANSI codes (may not work on all terminals)
    eprintln!("\x1b[1;31merror\x1b[0m: {}", msg);
}
```

## Graceful Degradation

### Fallback Values

Design CLIs that work with minimal input:

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// Configuration file
    #[arg(long, default_value = "config.toml")]
    config: PathBuf,

    /// Output directory
    #[arg(long, env = "OUTPUT_DIR", default_value = "./output")]
    output: PathBuf,

    /// Verbosity level (default: from VERBOSE env or 0)
    #[arg(short, long, env = "VERBOSE", default_value_t = 0)]
    verbose: u8,

    /// Number of threads (default: number of CPUs)
    #[arg(long)]
    threads: Option<usize>,
}

impl Cli {
    fn threads(&self) -> usize {
        self.threads.unwrap_or_else(num_cpus::get)
    }
}

fn num_cpus::get() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1)
}
```

### Optional Features

Allow partial functionality when dependencies are unavailable:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process files locally
    Process {
        files: Vec<String>,
    },

    /// Sync with remote server (requires network)
    Sync {
        #[arg(long)]
        remote: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Process { files } => {
            // Always available
            process_files(&files);
        }
        Commands::Sync { remote } => {
            // Check if network is available
            if !network_available() {
                eprintln!("Warning: Network unavailable. Running in offline mode.");
                eprintln!("Changes will be synced when connection is restored.");
                cache_sync_request(&remote);
                return;
            }
            sync_with_remote(&remote);
        }
    }
}

fn network_available() -> bool { true }
fn process_files(_files: &[String]) {}
fn sync_with_remote(_remote: &str) {}
fn cache_sync_request(_remote: &str) {}
```

### Progressive Enhancement

Enhance functionality based on available resources:

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Enable interactive mode (requires terminal)
    #[arg(long)]
    interactive: bool,

    /// Enable progress display
    #[arg(long)]
    progress: bool,
}

fn main() {
    let cli = Cli::parse();

    let is_terminal = atty::is(atty::Stream::Stdout);

    if cli.interactive && !is_terminal {
        eprintln!("Warning: --interactive ignored (not running in a terminal)");
    }

    let show_progress = cli.progress && is_terminal;

    if show_progress {
        run_with_progress();
    } else {
        run_silent();
    }
}

fn run_with_progress() {
    println!("Running with progress bar...");
}

fn run_silent() {
    println!("Running silently...");
}

mod atty {
    pub enum Stream { Stdout }
    pub fn is(_: Stream) -> bool { true }
}
```

## Error Handling Patterns

### Exit Code Conventions

Follow standard conventions for scriptability:

```rust
use clap::Parser;

/// Exit codes following Unix conventions
mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const USAGE_ERROR: i32 = 2;        // Clap default for parse errors
    pub const DATA_ERROR: i32 = 65;        // Input data incorrect
    pub const NO_INPUT: i32 = 66;          // Input file not found
    pub const CANT_CREATE: i32 = 73;       // Can't create output
    pub const IO_ERROR: i32 = 74;          // I/O error
    pub const TEMP_FAIL: i32 = 75;         // Temporary failure
    pub const CONFIG_ERROR: i32 = 78;      // Configuration error
}

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            e.print().expect("Failed to print error");
            std::process::exit(exit_codes::USAGE_ERROR);
        }
    };

    if let Some(ref config) = cli.config {
        if !config.exists() {
            eprintln!("Error: Configuration file not found: {:?}", config);
            std::process::exit(exit_codes::NO_INPUT);
        }
    }

    match run(&cli) {
        Ok(()) => std::process::exit(exit_codes::SUCCESS),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(exit_codes::GENERAL_ERROR);
        }
    }
}

fn run(_cli: &Cli) -> Result<(), String> {
    Ok(())
}
```

### Error Logging Integration

Integrate with structured logging:

```rust
use clap::Parser;
use tracing::{error, warn, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
struct Cli {
    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Quiet mode (suppress non-error output)
    #[arg(short, long)]
    quiet: bool,
}

fn setup_logging(cli: &Cli) {
    let level = if cli.quiet {
        Level::ERROR
    } else {
        match cli.verbose {
            0 => Level::WARN,
            1 => Level::INFO,
            2 => Level::DEBUG,
            _ => Level::TRACE,
        }
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set subscriber");
}

fn main() {
    let cli = Cli::parse();
    setup_logging(&cli);

    info!("Application starting");

    if let Err(e) = run() {
        error!("Application failed: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    warn!("This is a warning");
    info!("Processing...");
    Ok(())
}
```

### User-Facing vs Debug Errors

Separate user messages from technical details:

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug)]
struct DetailedError {
    user_message: String,
    technical_details: String,
    suggestion: Option<String>,
}

impl std::fmt::Display for DetailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_message)?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, "\nSuggestion: {}", suggestion)?;
        }
        Ok(())
    }
}

impl std::error::Error for DetailedError {}

fn read_config(path: &PathBuf) -> Result<String, DetailedError> {
    std::fs::read_to_string(path).map_err(|e| {
        DetailedError {
            user_message: format!("Could not read configuration from {:?}", path),
            technical_details: format!("IO error: {:?}", e),
            suggestion: Some("Ensure the file exists and you have read permissions".to_string()),
        }
    })
}

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    config: PathBuf,

    /// Show detailed error information
    #[arg(long)]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();

    match read_config(&cli.config) {
        Ok(content) => println!("Config loaded: {} bytes", content.len()),
        Err(e) => {
            eprintln!("Error: {}", e);
            if cli.debug {
                eprintln!("\nDebug info: {}", e.technical_details);
            }
            std::process::exit(1);
        }
    }
}
```

## Integration with Result and Option

### Using anyhow for Applications

For applications, `anyhow` provides ergonomic error handling:

```rust
use anyhow::{Context, Result, bail};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// Input file
    input: PathBuf,

    /// Output file
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let content = std::fs::read_to_string(&cli.input)
        .with_context(|| format!("Failed to read input file: {:?}", cli.input))?;

    if content.is_empty() {
        bail!("Input file is empty");
    }

    let processed = process(&content)
        .context("Failed to process content")?;

    let output = cli.output.unwrap_or_else(|| {
        let mut p = cli.input.clone();
        p.set_extension("out");
        p
    });

    std::fs::write(&output, processed)
        .with_context(|| format!("Failed to write output file: {:?}", output))?;

    println!("Wrote output to {:?}", output);
    Ok(())
}

fn process(content: &str) -> Result<String> {
    Ok(content.to_uppercase())
}
```

### Using thiserror for Libraries

For libraries, use `thiserror` for structured errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    NotFound { path: String },

    #[error("Invalid configuration format: {message}")]
    InvalidFormat { message: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("IO error reading configuration")]
    Io(#[from] std::io::Error),

    #[error("Parse error in configuration")]
    Parse(#[from] toml::de::Error),
}

pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    if !std::path::Path::new(path).exists() {
        return Err(ConfigError::NotFound { path: path.to_string() });
    }

    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;

    if config.name.is_empty() {
        return Err(ConfigError::MissingField { field: "name".to_string() });
    }

    Ok(config)
}

#[derive(serde::Deserialize)]
pub struct Config {
    pub name: String,
}

mod toml {
    pub mod de {
        #[derive(Debug)]
        pub struct Error;
        impl std::fmt::Display for Error {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "TOML parse error")
            }
        }
        impl std::error::Error for Error {}
    }
    pub fn from_str<T>(_s: &str) -> Result<T, de::Error> {
        Err(de::Error)
    }
}
```

## Summary

### Key Takeaways

1. **Use `try_parse()` for error control**: Access `ErrorKind` for programmatic handling
2. **Create descriptive validation errors**: Include context, hints, and suggestions
3. **Follow exit code conventions**: `0` for success, `2` for usage errors, other codes for specific failures
4. **Design for graceful degradation**: Use defaults, environment variables, and optional features
5. **Separate user and debug messages**: Show actionable info by default, details with `--debug`
6. **Integrate with Rust error ecosystem**: `anyhow` for applications, `thiserror` for libraries
7. **Log errors appropriately**: Match verbosity flags to log levels

> **Cross-Reference**: For advanced error strategies including recovery and suggestions, see [Chapter 14: Advanced Error Strategies](../part3-advanced-architecture/14-advanced-error-strategies.md). For testing error conditions, see [Chapter 15: Testing CLI Applications](../part3-advanced-architecture/15-testing-cli-applications.md).

---

*Next: [Builder Pattern Deep Dive](../part2-core-patterns/06-builder-pattern-deep-dive.md)*
