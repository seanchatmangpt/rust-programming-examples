# Derive Macro Mastery

> **Chapter 7** | Part 2: Core Patterns | Estimated reading time: 14 minutes

The derive macro approach represents the modern, declarative way to define CLIs in Clap. By leveraging Rust's powerful procedural macro system, derive macros transform struct and enum definitions into fully-featured command-line interfaces. This chapter provides a comprehensive reference for mastering derive-based CLI development.

## Attribute Syntax Architecture

Clap's derive macros use three primary attributes that correspond to different abstraction levels:

```
+===========================================================================+
|                    DERIVE ATTRIBUTE HIERARCHY                              |
+===========================================================================+

    ATTRIBUTE LEVEL          PURPOSE                      RUST CONSTRUCT
    ===============          =======                      ==============

    #[derive(Parser)]        Entry point                  struct / enum
          |
          v
    #[command(...)]     ---> Command-level config         struct Cli { }
          |                  - name, version, about
          |                  - help_template
          |                  - propagate_version
          |
          +-----------------+------------------+
          |                 |                  |
          v                 v                  v
    #[arg(...)]        #[group(...)]    #[command(subcommand)]
          |                 |                  |
    Argument config    Group config      Subcommand enum
    - short, long      - required        #[derive(Subcommand)]
    - value_parser     - multiple        enum Commands { }
    - default_value    - conflicts_with
    - env


    EXPANSION FLOW:
    ===============

    Your Code                     Generated Code
    =========                     ==============

    #[derive(Parser)]             impl Parser for Cli {
    struct Cli {                      fn parse() -> Self { ... }
        #[arg(short)]                 fn try_parse() -> Result<Self> { ... }
        verbose: bool,            }
    }
                    ==>           impl CommandFactory for Cli {
                                      fn command() -> Command { ... }
                                  }

                                  impl FromArgMatches for Cli {
                                      fn from_arg_matches(...) -> Result<Self> { ... }
                                  }
```

**Diagram Description**: This hierarchy shows how derive attributes map to different levels of CLI configuration: Parser for the entry point, command for metadata, arg for individual arguments, group for relationships, and subcommand for nested commands. The expansion flow shows what traits are automatically implemented.

### Command-Level Attributes

The `#[command]` attribute configures the overall command behavior:

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "dataforge",
    version = "2.5.0",
    author = "DataForge Team <team@dataforge.io>",
    about = "Transform and analyze data at scale",
    long_about = "DataForge is a high-performance data processing toolkit \
                  designed for ETL pipelines, data validation, and \
                  real-time stream processing.",
    after_help = "EXAMPLES:\n    \
                  dataforge transform input.csv --format parquet\n    \
                  dataforge validate schema.json data.json",
    after_long_help = "For complete documentation, visit https://docs.dataforge.io",
    arg_required_else_help = true,
    propagate_version = true,
    subcommand_required = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration file path
    #[arg(short, long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Transform data between formats
    Transform(TransformArgs),
    /// Validate data against schema
    Validate(ValidateArgs),
}

#[derive(clap::Args, Debug)]
struct TransformArgs {
    /// Input file
    input: PathBuf,
    /// Output format
    #[arg(short, long, value_parser = ["json", "csv", "parquet", "avro"])]
    format: String,
}

#[derive(clap::Args, Debug)]
struct ValidateArgs {
    /// Schema file
    schema: PathBuf,
    /// Data file to validate
    data: PathBuf,
}
```

### Comprehensive Argument Attributes

The `#[arg]` attribute provides fine-grained control over individual arguments:

```rust
use clap::{Parser, ArgAction, ValueHint};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "processor")]
struct Cli {
    // Positional argument (no short/long)
    /// Input file to process
    #[arg(value_name = "INPUT", value_hint = ValueHint::FilePath)]
    input: PathBuf,

    // Short and long flags
    /// Enable verbose output
    #[arg(short = 'v', long = "verbose", action = ArgAction::SetTrue)]
    verbose: bool,

    // Required option with custom name
    /// Output destination
    #[arg(short, long, required = true, value_name = "FILE")]
    output: PathBuf,

    // Option with default value
    /// Number of worker threads
    #[arg(short = 'j', long = "jobs", default_value_t = 4)]
    jobs: usize,

    // Option with environment variable fallback
    /// API endpoint URL
    #[arg(long, env = "PROCESSOR_API_URL", hide_env_values = true)]
    api_url: Option<String>,

    // Multiple values (collected into Vec)
    /// Files to exclude from processing
    #[arg(short = 'x', long = "exclude", action = ArgAction::Append)]
    excludes: Vec<String>,

    // Option that requires another
    /// Encryption key (requires --encrypt)
    #[arg(long, requires = "encrypt")]
    key: Option<String>,

    /// Enable encryption
    #[arg(long)]
    encrypt: bool,

    // Mutually exclusive options
    /// Silent mode (no output)
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,

    // Hidden argument (for scripts/power users)
    /// Internal debug flag
    #[arg(long, hide = true)]
    debug_internal: bool,

    // Value with restricted choices
    /// Log level
    #[arg(
        long,
        value_parser = ["trace", "debug", "info", "warn", "error"],
        default_value = "info"
    )]
    log_level: String,

    // Numeric range constraint
    /// Port number (1024-65535)
    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1024..=65535))]
    port: Option<u16>,
}
```

## Flattening and Nesting Strategies

Complex CLIs benefit from decomposing arguments into logical groups using `flatten`:

```rust
use clap::{Args, Parser};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "deploy")]
struct Cli {
    /// Target environment
    #[arg(short, long)]
    env: String,

    // Flatten common argument groups
    #[command(flatten)]
    auth: AuthOptions,

    #[command(flatten)]
    output: OutputOptions,

    #[command(flatten)]
    network: NetworkOptions,
}

/// Authentication configuration
#[derive(Args, Debug)]
struct AuthOptions {
    /// API token for authentication
    #[arg(long, env = "DEPLOY_TOKEN", hide_env_values = true)]
    token: Option<String>,

    /// Named credential profile
    #[arg(long, default_value = "default")]
    profile: String,

    /// Path to credentials file
    #[arg(long, value_name = "PATH")]
    credentials_file: Option<PathBuf>,
}

/// Output formatting configuration
#[derive(Args, Debug)]
struct OutputOptions {
    /// Output format
    #[arg(short, long, value_parser = ["json", "yaml", "table"])]
    format: Option<String>,

    /// Suppress non-essential output
    #[arg(short, long)]
    quiet: bool,

    /// Enable colorized output
    #[arg(long, default_value_t = true)]
    color: bool,
}

/// Network configuration
#[derive(Args, Debug)]
struct NetworkOptions {
    /// Request timeout in seconds
    #[arg(long, default_value_t = 30)]
    timeout: u64,

    /// Maximum retry attempts
    #[arg(long, default_value_t = 3)]
    retries: u32,

    /// HTTP proxy URL
    #[arg(long, env = "HTTP_PROXY")]
    proxy: Option<String>,
}
```

## Advanced Subcommand Patterns

### Nested Subcommand Hierarchies

Build deep command trees with nested enums:

```rust
use clap::{Parser, Subcommand, Args};

#[derive(Parser, Debug)]
#[command(name = "cloud", about = "Cloud infrastructure management")]
struct Cli {
    #[command(subcommand)]
    command: CloudCommands,
}

#[derive(Subcommand, Debug)]
enum CloudCommands {
    /// Manage compute resources
    Compute {
        #[command(subcommand)]
        action: ComputeCommands,
    },
    /// Manage storage resources
    Storage {
        #[command(subcommand)]
        action: StorageCommands,
    },
    /// Manage networking
    Network {
        #[command(subcommand)]
        action: NetworkCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ComputeCommands {
    /// List all instances
    List {
        /// Filter by region
        #[arg(short, long)]
        region: Option<String>,
    },
    /// Create a new instance
    Create(CreateInstanceArgs),
    /// Terminate an instance
    Delete {
        /// Instance ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Args, Debug)]
struct CreateInstanceArgs {
    /// Instance name
    #[arg(short, long)]
    name: String,
    /// Instance type/size
    #[arg(short = 't', long, default_value = "small")]
    instance_type: String,
    /// Disk size in GB
    #[arg(long, default_value_t = 20)]
    disk_size: u32,
}

#[derive(Subcommand, Debug)]
enum StorageCommands {
    /// List storage buckets
    List,
    /// Create a bucket
    Create { name: String },
    /// Delete a bucket
    Delete { name: String },
}

#[derive(Subcommand, Debug)]
enum NetworkCommands {
    /// List VPCs
    List,
    /// Create a VPC
    Create { cidr: String },
}
```

### Optional Subcommands with Default Actions

```rust
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "status")]
struct Cli {
    /// Show extended information
    #[arg(short, long)]
    extended: bool,

    #[command(subcommand)]
    command: Option<StatusCommands>,
}

#[derive(Subcommand, Debug)]
enum StatusCommands {
    /// Show service status
    Services,
    /// Show resource usage
    Resources,
    /// Show active connections
    Connections,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(StatusCommands::Services) => show_services(),
        Some(StatusCommands::Resources) => show_resources(),
        Some(StatusCommands::Connections) => show_connections(),
        None => {
            // Default action when no subcommand provided
            show_summary(cli.extended);
        }
    }
}

fn show_summary(extended: bool) { /* ... */ }
fn show_services() { /* ... */ }
fn show_resources() { /* ... */ }
fn show_connections() { /* ... */ }
```

## Documentation Comments as Help Text

Clap derives help text from documentation comments, enabling self-documenting CLIs:

```rust
use clap::Parser;
use std::path::PathBuf;

/// High-performance log analyzer for distributed systems
///
/// Analyzes log files from multiple sources, correlates events,
/// and generates reports. Supports structured (JSON) and
/// unstructured log formats.
#[derive(Parser, Debug)]
#[command(name = "logalyzer", version, author)]
struct Cli {
    /// Log files to analyze
    ///
    /// Supports glob patterns like "logs/*.log" and
    /// recursive patterns like "logs/**/*.json"
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Time window for analysis
    ///
    /// Accepts formats like "1h", "30m", "7d", or
    /// ISO 8601 duration strings
    #[arg(short, long, default_value = "1h")]
    window: String,

    /// Correlation patterns to detect
    ///
    /// Provide regex patterns for event correlation.
    /// Multiple patterns can be specified.
    #[arg(short = 'p', long = "pattern", action = clap::ArgAction::Append)]
    patterns: Vec<String>,

    /// Output report format
    #[arg(short, long, value_parser = ["text", "json", "html"])]
    format: Option<String>,
}
```

## Custom Type Derivation

Implement `ValueEnum` for type-safe enum arguments:

```rust
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,

    /// Compression algorithm
    #[arg(short, long, value_enum)]
    compression: Option<Compression>,

    /// Log level
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,
}

#[derive(ValueEnum, Clone, Debug, Default)]
enum OutputFormat {
    /// Display as formatted table
    #[default]
    Table,
    /// Output as JSON
    Json,
    /// Output as YAML
    Yaml,
    /// Output as CSV
    Csv,
    /// Compact single-line output
    #[value(name = "oneline")]
    OneLine,
}

#[derive(ValueEnum, Clone, Debug)]
enum Compression {
    /// No compression
    None,
    /// Gzip compression (fast, moderate ratio)
    Gzip,
    /// Zstd compression (balanced)
    Zstd,
    /// Lz4 compression (very fast)
    Lz4,
    /// Brotli compression (high ratio)
    Brotli,
}

#[derive(ValueEnum, Clone, Debug)]
#[value(rename_all = "UPPER")]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
```

## Testing Derive Definitions

Robust testing catches configuration errors before runtime:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli_structure() {
        // Validates all attributes and relationships
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_minimal_args() {
        let cli = Cli::parse_from(["app", "input.txt", "-o", "output.txt"]);
        assert_eq!(cli.input, PathBuf::from("input.txt"));
    }

    #[test]
    fn parse_with_all_options() {
        let cli = Cli::parse_from([
            "app",
            "input.txt",
            "-o", "output.txt",
            "-v",
            "--format", "json",
            "-j", "8",
        ]);
        assert!(cli.verbose);
        assert_eq!(cli.jobs, 8);
    }

    #[test]
    fn test_default_values() {
        let cli = Cli::try_parse_from(["app", "input.txt", "-o", "out.txt"])
            .expect("should parse");
        assert_eq!(cli.jobs, 4); // Check default
        assert_eq!(cli.log_level, "info");
    }

    #[test]
    fn test_conflicting_args_rejected() {
        let result = Cli::try_parse_from([
            "app",
            "input.txt",
            "-o", "output.txt",
            "-v",
            "-q",  // Conflicts with -v
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_required_arg_enforcement() {
        let result = Cli::try_parse_from(["app", "input.txt"]);
        // Should fail because -o/--output is required
        assert!(result.is_err());
    }

    #[test]
    fn test_subcommand_parsing() {
        let cli = Cli::parse_from([
            "app",
            "compute",
            "create",
            "--name", "web-server",
            "--instance-type", "large",
        ]);
        // Verify subcommand structure
        match cli.command {
            Commands::Compute { action: ComputeCommands::Create(args) } => {
                assert_eq!(args.name, "web-server");
                assert_eq!(args.instance_type, "large");
            }
            _ => panic!("Wrong subcommand parsed"),
        }
    }
}
```

## Common Pitfalls

1. **Forgetting `#[command(flatten)]` vs `#[arg(flatten)]`**: Use `#[command(flatten)]` for nested `Args` structs and `#[command(subcommand)]` for enums.

2. **Conflicting short flags**: When flattening multiple structs, short flags can collide. Audit all flattened structs for uniqueness.

3. **Missing `value_enum` attribute**: Using a `ValueEnum` type without `#[arg(value_enum)]` causes cryptic compilation errors.

4. **Doc comment formatting**: The first line becomes `help`, subsequent paragraphs become `long_help`. A blank line separates them.

5. **Default value type mismatch**: Use `default_value_t` for typed defaults and `default_value` for string defaults.

## Pro Tips

- **Use `#[command(flatten)]` liberally**: Decompose CLIs into logical groups for better organization and reuse
- **Leverage `#[arg(skip)]`**: Skip fields that shouldn't come from CLI (computed values, runtime state)
- **Enable `#[command(propagate_version = true)]`**: All subcommands inherit the version string
- **Add `#[command(infer_subcommands = true)]`**: Allow unique prefix matching for subcommands (`com` matches `compute`)
- **Use `#[arg(value_hint = ValueHint::FilePath)]`**: Enables shell completion for file paths
- **Implement `Default` on your CLI struct**: Enables partial parsing and default fallbacks

## Summary

Derive macros transform Clap CLI development by enabling:

1. **Declarative syntax** that's readable and maintainable
2. **Type-safe arguments** with compile-time validation
3. **Automatic help generation** from documentation comments
4. **Composable structures** through flatten and subcommand patterns
5. **Comprehensive testing** via `CommandFactory` and `parse_from`

Master these patterns to build expressive, self-documenting CLIs that leverage Rust's type system for correctness.

---

*Next: [Argument Groups and Conflicts](./08-argument-groups-conflicts.md)*
