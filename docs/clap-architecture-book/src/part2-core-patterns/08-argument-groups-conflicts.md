# Argument Groups and Conflicts

> **Chapter 8** | Part 2: Core Patterns | Estimated reading time: 12 minutes

Production CLIs often require sophisticated argument relationships: options that must appear together, flags that are mutually exclusive, and inputs that depend on context. Clap's argument groups and conflict system provides declarative, compile-time validation for these complex relationships. This chapter explores patterns for managing argument interdependencies.

## Argument Relationship Architecture

Understanding the types of relationships between arguments is fundamental to effective CLI design:

```
┌────────────────────────────────────────────────────────────────┐
│                  Argument Relationship Types                    │
├────────────────────────────────────────────────────────────────┤
│  CONFLICTS     │  A and B cannot appear together              │
│  REQUIRES      │  A needs B to also be present                │
│  REQUIRED_IF   │  A is required when B has specific value     │
│  GROUP         │  Set of args treated as logical unit         │
│  OVERRIDES     │  A's presence supersedes B's value           │
└────────────────────────────────────────────────────────────────┘
```

## Mutual Exclusion Patterns

### Simple Conflicts

The most common pattern prevents contradictory options:

```rust
use clap::{Parser, ArgAction};

#[derive(Parser, Debug)]
#[command(name = "logger", about = "Application logging control")]
struct Cli {
    /// Enable verbose output (detailed logs)
    #[arg(short, long, conflicts_with_all = ["quiet", "silent"])]
    verbose: bool,

    /// Reduce output (warnings and errors only)
    #[arg(short, long, conflicts_with_all = ["verbose", "silent"])]
    quiet: bool,

    /// Suppress all output
    #[arg(short, long, conflicts_with_all = ["verbose", "quiet"])]
    silent: bool,

    /// Log file path
    #[arg(long)]
    log_file: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let level = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "warn"
    } else if cli.silent {
        "off"
    } else {
        "info"
    };

    println!("Log level: {}", level);
}
```

### Exclusive Groups

For sets of mutually exclusive options, use `ArgGroup`:

```rust
use clap::{Parser, Args, ArgGroup};

#[derive(Parser, Debug)]
#[command(name = "export")]
struct Cli {
    /// Data source to export
    #[arg(short, long)]
    source: String,

    #[command(flatten)]
    output_mode: OutputMode,

    #[command(flatten)]
    format_options: FormatOptions,
}

/// Output destination - exactly one must be specified
#[derive(Args, Debug)]
#[group(required = true, multiple = false, id = "output")]
struct OutputMode {
    /// Write to file
    #[arg(short, long, value_name = "PATH")]
    file: Option<String>,

    /// Write to stdout
    #[arg(long)]
    stdout: bool,

    /// Upload to remote storage
    #[arg(long, value_name = "URL")]
    remote: Option<String>,

    /// Store in clipboard
    #[arg(long)]
    clipboard: bool,
}

/// Output format - at most one can be specified
#[derive(Args, Debug)]
#[group(required = false, multiple = false, id = "format")]
struct FormatOptions {
    /// Export as JSON
    #[arg(long)]
    json: bool,

    /// Export as CSV
    #[arg(long)]
    csv: bool,

    /// Export as XML
    #[arg(long)]
    xml: bool,

    /// Export as YAML
    #[arg(long)]
    yaml: bool,
}

fn main() {
    let cli = Cli::parse();

    // Determine output destination
    let destination = if let Some(ref path) = cli.output_mode.file {
        format!("file: {}", path)
    } else if cli.output_mode.stdout {
        "stdout".to_string()
    } else if let Some(ref url) = cli.output_mode.remote {
        format!("remote: {}", url)
    } else if cli.output_mode.clipboard {
        "clipboard".to_string()
    } else {
        unreachable!("group(required = true) ensures one is set")
    };

    // Determine format (with default)
    let format = if cli.format_options.json {
        "json"
    } else if cli.format_options.csv {
        "csv"
    } else if cli.format_options.xml {
        "xml"
    } else if cli.format_options.yaml {
        "yaml"
    } else {
        "json" // default when none specified
    };

    println!("Exporting {} from {} as {}", destination, cli.source, format);
}
```

## Dependency Chains

### Requires Relationships

Ensure dependent options appear together:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "deploy")]
struct Cli {
    /// Deployment environment
    #[arg(short, long)]
    environment: String,

    /// Enable SSL/TLS
    #[arg(long)]
    ssl: bool,

    /// SSL certificate path (required when --ssl is used)
    #[arg(long, requires = "ssl", value_name = "PATH")]
    ssl_cert: Option<String>,

    /// SSL key path (required when --ssl is used)
    #[arg(long, requires = "ssl", value_name = "PATH")]
    ssl_key: Option<String>,

    /// Enable database replication
    #[arg(long)]
    replicate: bool,

    /// Primary database host (required for replication)
    #[arg(long, requires = "replicate")]
    primary_host: Option<String>,

    /// Replica count (required for replication)
    #[arg(long, requires = "replicate", default_value_t = 2)]
    replica_count: u32,
}
```

### Conditional Requirements

Make arguments required based on other argument values:

```rust
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "auth")]
struct Cli {
    /// Authentication method
    #[arg(short, long, value_enum)]
    method: AuthMethod,

    /// Username (required for password auth)
    #[arg(long, required_if_eq("method", "password"))]
    username: Option<String>,

    /// Password (required for password auth)
    #[arg(long, required_if_eq("method", "password"))]
    password: Option<String>,

    /// API key (required for apikey auth)
    #[arg(long, required_if_eq("method", "apikey"))]
    api_key: Option<String>,

    /// OAuth client ID (required for oauth auth)
    #[arg(long, required_if_eq("method", "oauth"))]
    client_id: Option<String>,

    /// OAuth client secret (required for oauth auth)
    #[arg(long, required_if_eq("method", "oauth"))]
    client_secret: Option<String>,

    /// OAuth redirect URI (required for oauth auth)
    #[arg(long, required_if_eq("method", "oauth"))]
    redirect_uri: Option<String>,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum AuthMethod {
    Password,
    Apikey,
    Oauth,
    Certificate,
}
```

### Bidirectional Dependencies

Some options must always appear together:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "proxy")]
struct Cli {
    /// Target server
    #[arg(short, long)]
    target: String,

    /// Proxy username (requires password)
    #[arg(long, requires = "proxy_password")]
    proxy_user: Option<String>,

    /// Proxy password (requires username)
    #[arg(long, requires = "proxy_user")]
    proxy_password: Option<String>,

    /// Source IP binding (requires source port)
    #[arg(long, requires = "source_port")]
    source_ip: Option<String>,

    /// Source port binding (requires source IP)
    #[arg(long, requires = "source_ip")]
    source_port: Option<u16>,
}
```

## Advanced Group Patterns

### Nested and Composite Groups

Build complex validation rules with nested groups:

```rust
use clap::{Parser, Args, ArgGroup};

#[derive(Parser, Debug)]
#[command(name = "backup")]
struct Cli {
    /// Source path to backup
    #[arg(short, long)]
    source: String,

    #[command(flatten)]
    destination: DestinationOptions,

    #[command(flatten)]
    schedule: ScheduleOptions,
}

/// Backup destination configuration
#[derive(Args, Debug)]
#[group(required = true, id = "dest_group")]
struct DestinationOptions {
    /// Local backup directory
    #[arg(long, value_name = "PATH")]
    local_path: Option<String>,

    #[command(flatten)]
    cloud: CloudOptions,
}

/// Cloud storage options
#[derive(Args, Debug)]
#[group(id = "cloud_group")]
struct CloudOptions {
    /// S3 bucket for backup
    #[arg(long)]
    s3_bucket: Option<String>,

    /// Azure blob container
    #[arg(long)]
    azure_container: Option<String>,

    /// Google Cloud Storage bucket
    #[arg(long)]
    gcs_bucket: Option<String>,
}

/// Backup scheduling options
#[derive(Args, Debug)]
#[group(id = "schedule_group")]
struct ScheduleOptions {
    /// Run backup immediately
    #[arg(long, conflicts_with_all = ["cron", "interval"])]
    now: bool,

    /// Cron expression for scheduling
    #[arg(long, conflicts_with_all = ["now", "interval"])]
    cron: Option<String>,

    /// Interval in hours between backups
    #[arg(long, conflicts_with_all = ["now", "cron"])]
    interval: Option<u32>,
}
```

### Dynamic Group Validation

For complex validation that goes beyond static attributes:

```rust
use clap::{Parser, Args, error::ErrorKind};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "transfer")]
struct Cli {
    #[command(flatten)]
    transfer: TransferOptions,
}

#[derive(Args, Debug)]
struct TransferOptions {
    /// Source path
    #[arg(short, long)]
    source: PathBuf,

    /// Destination path
    #[arg(short, long)]
    dest: PathBuf,

    /// Enable compression
    #[arg(long)]
    compress: bool,

    /// Compression level (1-9)
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=9))]
    compression_level: Option<u8>,

    /// Enable encryption
    #[arg(long)]
    encrypt: bool,

    /// Encryption algorithm
    #[arg(long, value_parser = ["aes256", "chacha20"])]
    cipher: Option<String>,
}

impl TransferOptions {
    /// Validate complex interdependencies after parsing
    pub fn validate(&self) -> Result<(), String> {
        // compression_level requires compress
        if self.compression_level.is_some() && !self.compress {
            return Err(
                "--compression-level requires --compress to be enabled".to_string()
            );
        }

        // cipher requires encrypt
        if self.cipher.is_some() && !self.encrypt {
            return Err("--cipher requires --encrypt to be enabled".to_string());
        }

        // source and dest cannot be the same
        if self.source == self.dest {
            return Err("Source and destination cannot be the same".to_string());
        }

        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.transfer.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    println!("Transfer configuration validated: {:?}", cli.transfer);
}
```

## Default Value Strategies

### Context-Aware Defaults

Defaults that depend on other arguments:

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "build")]
struct Cli {
    /// Project directory
    #[arg(short, long, default_value = ".")]
    project: PathBuf,

    /// Build profile
    #[arg(long, default_value = "release")]
    profile: String,

    /// Output directory (defaults to <project>/target/<profile>)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Number of parallel jobs (defaults to CPU count)
    #[arg(short = 'j', long)]
    jobs: Option<usize>,
}

impl Cli {
    /// Resolve computed defaults after parsing
    pub fn resolved_output(&self) -> PathBuf {
        self.output.clone().unwrap_or_else(|| {
            self.project.join("target").join(&self.profile)
        })
    }

    pub fn resolved_jobs(&self) -> usize {
        self.jobs.unwrap_or_else(|| num_cpus::get())
    }
}

fn main() {
    let cli = Cli::parse();

    println!("Project: {:?}", cli.project);
    println!("Profile: {}", cli.profile);
    println!("Output: {:?}", cli.resolved_output());
    println!("Jobs: {}", cli.resolved_jobs());
}
```

### Override Semantics

Handle multiple sources with clear precedence:

```rust
use clap::{Parser, ArgAction};

#[derive(Parser, Debug)]
#[command(name = "config")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, env = "APP_CONFIG")]
    config: Option<String>,

    /// Override log level (takes precedence over config file)
    #[arg(long, env = "APP_LOG_LEVEL", overrides_with = "log_level")]
    log_level: Option<String>,

    /// Override port (last value wins when specified multiple times)
    #[arg(
        short,
        long,
        env = "APP_PORT",
        action = ArgAction::Set,
        overrides_with = "port"
    )]
    port: Option<u16>,

    /// Add to include paths (accumulates, doesn't override)
    #[arg(short = 'I', long = "include", action = ArgAction::Append)]
    includes: Vec<String>,
}
```

## Group Validation Rules

### At-Least-One Patterns

Ensure at least one option from a group is provided:

```rust
use clap::{Parser, Args, ArgGroup};

#[derive(Parser, Debug)]
#[command(name = "notify")]
struct Cli {
    /// Message to send
    #[arg(short, long)]
    message: String,

    #[command(flatten)]
    channels: NotificationChannels,
}

/// At least one notification channel must be specified
#[derive(Args, Debug)]
#[group(required = true, multiple = true, id = "channels")]
struct NotificationChannels {
    /// Send via email
    #[arg(long)]
    email: Option<String>,

    /// Send via SMS
    #[arg(long)]
    sms: Option<String>,

    /// Send via Slack
    #[arg(long)]
    slack: Option<String>,

    /// Send via webhook
    #[arg(long)]
    webhook: Option<String>,
}
```

### Exactly-One Patterns

Enforce precisely one option:

```rust
use clap::{Parser, Args, ArgGroup};

#[derive(Parser, Debug)]
#[command(name = "convert")]
struct Cli {
    /// Input file
    #[arg(short, long)]
    input: String,

    #[command(flatten)]
    output_format: OutputFormat,
}

/// Exactly one output format must be chosen
#[derive(Args, Debug)]
#[group(required = true, multiple = false, id = "format")]
struct OutputFormat {
    #[arg(long)]
    to_json: bool,

    #[arg(long)]
    to_yaml: bool,

    #[arg(long)]
    to_xml: bool,

    #[arg(long)]
    to_csv: bool,
}
```

## Common Pitfalls

1. **Circular dependencies**: `a requires b, b requires a` can create impossible requirements. Use groups instead.

2. **Forgotten group IDs**: When using `conflicts_with` on groups, ensure the group has an explicit `id`.

3. **Overly strict validation**: Don't prevent valid use cases. Test with real user workflows.

4. **Missing error context**: When using post-parse validation, provide helpful error messages.

5. **Group vs Flatten confusion**: `#[group]` defines validation rules; `#[command(flatten)]` composes structs.

## Pro Tips

- **Test conflict combinations**: Write tests for all invalid argument combinations to ensure good error messages
- **Use `requires_all` for multi-dependencies**: When an option needs several others, use `requires_all` instead of multiple `requires`
- **Document relationships in help**: Add notes about required combinations in `long_help`
- **Consider defaults carefully**: Sometimes a sensible default eliminates the need for complex requirements
- **Validate early, fail fast**: Run validation immediately after parsing to provide quick feedback

## Summary

Effective argument relationship management creates intuitive, error-resistant CLIs:

1. **Conflicts** prevent contradictory combinations at parse time
2. **Requirements** ensure dependent options appear together
3. **Groups** treat related arguments as logical units
4. **Conditional requirements** adapt to context dynamically
5. **Post-parse validation** handles complex cross-field rules

Master these patterns to build CLIs that guide users toward valid configurations while providing clear feedback when constraints are violated.

---

*Next: [Value Parsing and Validation](./09-value-parsing-validation.md)*
