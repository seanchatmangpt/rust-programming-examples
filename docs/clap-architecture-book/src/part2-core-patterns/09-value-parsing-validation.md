# Value Parsing and Validation

> **Chapter 9** | Part 2: Core Patterns | Estimated reading time: 14 minutes

Clap's value parsing system transforms raw command-line strings into typed Rust values with validation. This chapter explores built-in parsers, custom parser implementation, validation pipelines, and patterns for providing excellent error messages. Mastering value parsing is essential for building robust, user-friendly CLIs.

## Value Parsing Architecture

Understanding how Clap processes argument values helps you design effective parsers:

```
┌──────────────────────────────────────────────────────────────────┐
│                    Value Parsing Pipeline                         │
├──────────────────────────────────────────────────────────────────┤
│  Raw String ──► TypedValueParser ──► Validation ──► Typed Value  │
│      │               │                    │              │        │
│   "8080"      parse as u16         range check      u16(8080)    │
│   "abc"       parse as u16  ───► ERROR (not a number)            │
│   "99999"     parse as u16  ───► ERROR (overflow)                │
└──────────────────────────────────────────────────────────────────┘
```

## Built-in Value Parsers

Clap provides parsers for common types out of the box:

### Primitive Type Parsing

```rust
use clap::Parser;
use std::path::PathBuf;
use std::net::{IpAddr, SocketAddr};

#[derive(Parser, Debug)]
#[command(name = "netutil")]
struct Cli {
    // Automatically parsed integers
    /// Port number
    #[arg(short, long)]
    port: u16,

    /// Timeout in milliseconds
    #[arg(long, default_value_t = 5000)]
    timeout: u64,

    /// Retry count (can be negative for infinite)
    #[arg(long, default_value_t = 3)]
    retries: i32,

    // Floating point
    /// Rate limit (requests per second)
    #[arg(long, default_value_t = 100.0)]
    rate: f64,

    // Boolean (flag)
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,

    // PathBuf with automatic conversion
    /// Configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    // Network types
    /// Bind address
    #[arg(long)]
    bind_ip: Option<IpAddr>,

    /// Full socket address
    #[arg(long)]
    socket: Option<SocketAddr>,

    // Character
    /// Delimiter character
    #[arg(long, default_value_t = ',')]
    delimiter: char,
}
```

### Range-Constrained Parsing

Combine parsing with range validation:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "server")]
struct Cli {
    /// Port number (1024-65535, non-privileged)
    #[arg(
        short,
        long,
        default_value_t = 8080,
        value_parser = clap::value_parser!(u16).range(1024..=65535)
    )]
    port: u16,

    /// Worker thread count (1-256)
    #[arg(
        short = 'j',
        long,
        default_value_t = 4,
        value_parser = clap::value_parser!(u8).range(1..=256)
    )]
    workers: u8,

    /// Log retention days (1-365)
    #[arg(
        long,
        default_value_t = 30,
        value_parser = clap::value_parser!(u16).range(1..=365)
    )]
    retention_days: u16,

    /// Compression level (0-9, 0 disables)
    #[arg(
        long,
        default_value_t = 6,
        value_parser = clap::value_parser!(u8).range(0..=9)
    )]
    compression: u8,
}
```

### Possible Values (Enum-like Strings)

Restrict strings to predefined options:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "formatter")]
struct Cli {
    /// Input file
    input: String,

    /// Output format
    #[arg(
        short,
        long,
        default_value = "json",
        value_parser = ["json", "yaml", "toml", "xml", "csv"]
    )]
    format: String,

    /// Log verbosity level
    #[arg(
        long,
        default_value = "info",
        value_parser = ["trace", "debug", "info", "warn", "error", "off"]
    )]
    log_level: String,

    /// Color mode
    #[arg(
        long,
        default_value = "auto",
        value_parser = ["auto", "always", "never"]
    )]
    color: String,
}
```

## Custom ValueParser Implementation

### Function-Based Parsers

The simplest approach for custom parsing:

```rust
use clap::Parser;
use std::time::Duration;

/// Parse a duration string like "30s", "5m", "2h"
fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Duration cannot be empty".to_string());
    }

    let (value_str, unit) = s.split_at(s.len() - 1);
    let value: u64 = value_str
        .parse()
        .map_err(|_| format!("Invalid number in duration: {}", value_str))?;

    match unit {
        "s" => Ok(Duration::from_secs(value)),
        "m" => Ok(Duration::from_secs(value * 60)),
        "h" => Ok(Duration::from_secs(value * 3600)),
        "d" => Ok(Duration::from_secs(value * 86400)),
        _ => Err(format!(
            "Unknown duration unit '{}'. Use s, m, h, or d",
            unit
        )),
    }
}

/// Parse a size string like "100MB", "2GB", "500KB"
fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim().to_uppercase();

    let (num_part, unit) = if s.ends_with("GB") {
        (&s[..s.len() - 2], 1024 * 1024 * 1024)
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1024 * 1024)
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], 1024)
    } else if s.ends_with("B") {
        (&s[..s.len() - 1], 1)
    } else {
        return Err(format!("Invalid size format '{}'. Use KB, MB, or GB", s));
    };

    let value: u64 = num_part
        .trim()
        .parse()
        .map_err(|_| format!("Invalid number: {}", num_part))?;

    Ok(value * unit)
}

#[derive(Parser, Debug)]
#[command(name = "cache")]
struct Cli {
    /// Cache timeout (e.g., "30s", "5m", "2h")
    #[arg(long, value_parser = parse_duration, default_value = "5m")]
    timeout: Duration,

    /// Maximum cache size (e.g., "100MB", "2GB")
    #[arg(long, value_parser = parse_size, default_value = "512MB")]
    max_size: u64,

    /// Entry TTL (e.g., "1h", "7d")
    #[arg(long, value_parser = parse_duration)]
    ttl: Option<Duration>,
}
```

### TypedValueParser Trait

For complex parsers with access to command context:

```rust
use clap::{builder::TypedValueParser, Command, Arg, Error, error::ErrorKind};
use std::ffi::OsStr;

/// Parser for semantic version strings
#[derive(Clone, Debug)]
struct SemVerParser;

#[derive(Clone, Debug, PartialEq)]
struct SemVer {
    major: u32,
    minor: u32,
    patch: u32,
    prerelease: Option<String>,
}

impl TypedValueParser for SemVerParser {
    type Value = SemVer;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, Error> {
        let value_str = value
            .to_str()
            .ok_or_else(|| {
                Error::raw(ErrorKind::InvalidUtf8, "Version must be valid UTF-8")
            })?;

        // Parse version like "1.2.3" or "1.2.3-beta.1"
        let (version_part, prerelease) = if let Some(idx) = value_str.find('-') {
            (&value_str[..idx], Some(value_str[idx + 1..].to_string()))
        } else {
            (value_str, None)
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::raw(
                ErrorKind::ValueValidation,
                format!(
                    "Version '{}' must have format MAJOR.MINOR.PATCH",
                    value_str
                ),
            ));
        }

        let parse_component = |s: &str, name: &str| -> Result<u32, Error> {
            s.parse().map_err(|_| {
                Error::raw(
                    ErrorKind::ValueValidation,
                    format!("Invalid {} version component: {}", name, s),
                )
            })
        };

        Ok(SemVer {
            major: parse_component(parts[0], "major")?,
            minor: parse_component(parts[1], "minor")?,
            patch: parse_component(parts[2], "patch")?,
            prerelease,
        })
    }
}

use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    /// Minimum version requirement
    #[arg(long, value_parser = SemVerParser)]
    min_version: Option<SemVer>,

    /// Target version
    #[arg(long, value_parser = SemVerParser, default_value = "1.0.0")]
    version: SemVer,
}
```

### Stateful Parsers with Context

Parsers that validate against external state:

```rust
use clap::{builder::TypedValueParser, Command, Arg, Error, error::ErrorKind};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::sync::Arc;

/// Parser that validates against a set of known values
#[derive(Clone)]
struct KnownValueParser {
    known_values: Arc<HashSet<String>>,
    value_type: String,
}

impl KnownValueParser {
    fn new(values: Vec<&str>, value_type: &str) -> Self {
        Self {
            known_values: Arc::new(values.into_iter().map(String::from).collect()),
            value_type: value_type.to_string(),
        }
    }
}

impl TypedValueParser for KnownValueParser {
    type Value = String;

    fn parse_ref(
        &self,
        _cmd: &Command,
        _arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, Error> {
        let value_str = value
            .to_str()
            .ok_or_else(|| Error::raw(ErrorKind::InvalidUtf8, "Value must be valid UTF-8"))?;

        if self.known_values.contains(value_str) {
            Ok(value_str.to_string())
        } else {
            let suggestions: Vec<_> = self
                .known_values
                .iter()
                .filter(|v| v.starts_with(&value_str[..1.min(value_str.len())]))
                .take(3)
                .collect();

            let mut msg = format!(
                "Unknown {}: '{}'\nValid options: {}",
                self.value_type,
                value_str,
                self.known_values.iter().take(10).cloned().collect::<Vec<_>>().join(", ")
            );

            if !suggestions.is_empty() {
                msg.push_str(&format!("\nDid you mean: {}?", suggestions.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
            }

            Err(Error::raw(ErrorKind::ValueValidation, msg))
        }
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
        Some(Box::new(
            self.known_values
                .iter()
                .map(|v| clap::builder::PossibleValue::new(v.as_str())),
        ))
    }
}

// Usage with dynamically loaded values
fn build_region_parser() -> KnownValueParser {
    // In production, these might come from an API or config file
    KnownValueParser::new(
        vec!["us-east-1", "us-west-2", "eu-west-1", "ap-northeast-1"],
        "region",
    )
}
```

## Validation Pipelines

### Chained Validation

Combine multiple validation steps:

```rust
use clap::Parser;
use std::path::PathBuf;

/// Validate that a path exists and is a file
fn validate_existing_file(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", s));
    }

    if !path.is_file() {
        return Err(format!("Path is not a file: {}", s));
    }

    // Canonicalize for consistent paths
    path.canonicalize()
        .map_err(|e| format!("Cannot resolve path '{}': {}", s, e))
}

/// Validate that a path is a writable directory
fn validate_writable_dir(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);

    if path.exists() {
        if !path.is_dir() {
            return Err(format!("Path exists but is not a directory: {}", s));
        }
        // Check if writable by attempting to create a temp file
        let test_file = path.join(".write_test");
        std::fs::write(&test_file, "")
            .map_err(|_| format!("Directory is not writable: {}", s))?;
        std::fs::remove_file(test_file).ok();
    } else {
        // Try to create the directory
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Cannot create directory '{}': {}", s, e))?;
    }

    path.canonicalize()
        .map_err(|e| format!("Cannot resolve path '{}': {}", s, e))
}

/// Validate URL format
fn validate_url(s: &str) -> Result<String, String> {
    if !s.starts_with("http://") && !s.starts_with("https://") {
        return Err(format!(
            "URL must start with http:// or https://, got: {}",
            s
        ));
    }

    // Basic URL validation
    if s.len() < 10 || !s.contains('.') {
        return Err(format!("Invalid URL format: {}", s));
    }

    Ok(s.to_string())
}

#[derive(Parser, Debug)]
#[command(name = "sync")]
struct Cli {
    /// Source file to sync
    #[arg(short, long, value_parser = validate_existing_file)]
    source: PathBuf,

    /// Destination directory
    #[arg(short, long, value_parser = validate_writable_dir)]
    dest: PathBuf,

    /// Remote endpoint URL
    #[arg(long, value_parser = validate_url)]
    endpoint: Option<String>,
}
```

### Multi-Value Parsing

Parse complex multi-value arguments:

```rust
use clap::{Parser, ArgAction};
use std::collections::HashMap;

/// Parse key=value pairs
fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid key=value format: '{}'. Expected KEY=VALUE",
            s
        ));
    }

    let key = parts[0].trim();
    let value = parts[1].trim();

    if key.is_empty() {
        return Err("Key cannot be empty".to_string());
    }

    Ok((key.to_string(), value.to_string()))
}

/// Parse host:port pairs
fn parse_host_port(s: &str) -> Result<(String, u16), String> {
    let parts: Vec<&str> = s.rsplitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid host:port format: '{}'. Expected HOST:PORT",
            s
        ));
    }

    let port: u16 = parts[0]
        .parse()
        .map_err(|_| format!("Invalid port number: {}", parts[0]))?;

    let host = parts[1].to_string();
    if host.is_empty() {
        return Err("Host cannot be empty".to_string());
    }

    Ok((host, port))
}

/// Parse comma-separated list with validation
fn parse_tag_list(s: &str) -> Result<Vec<String>, String> {
    let tags: Vec<String> = s
        .split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();

    if tags.is_empty() {
        return Err("At least one tag is required".to_string());
    }

    // Validate tag format
    for tag in &tags {
        if !tag.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(format!(
                "Invalid tag '{}'. Tags can only contain alphanumeric characters, hyphens, and underscores",
                tag
            ));
        }
    }

    Ok(tags)
}

#[derive(Parser, Debug)]
#[command(name = "deploy")]
struct Cli {
    /// Environment variables (KEY=VALUE format)
    #[arg(short = 'e', long = "env", value_parser = parse_key_value, action = ArgAction::Append)]
    env_vars: Vec<(String, String)>,

    /// Backend servers (host:port format)
    #[arg(short = 'b', long = "backend", value_parser = parse_host_port, action = ArgAction::Append)]
    backends: Vec<(String, u16)>,

    /// Resource tags (comma-separated)
    #[arg(long, value_parser = parse_tag_list)]
    tags: Option<Vec<String>>,
}

fn main() {
    let cli = Cli::parse();

    // Convert env vars to HashMap for easy access
    let env_map: HashMap<_, _> = cli.env_vars.into_iter().collect();
    println!("Environment: {:?}", env_map);

    println!("Backends: {:?}", cli.backends);
    println!("Tags: {:?}", cli.tags);
}
```

## Error Context and Messages

### Rich Error Messages

Provide context-aware, helpful error messages:

```rust
use clap::{builder::TypedValueParser, Command, Arg, Error, error::ErrorKind};
use std::ffi::OsStr;

#[derive(Clone)]
struct PortParser {
    allow_privileged: bool,
}

impl PortParser {
    fn privileged() -> Self {
        Self { allow_privileged: true }
    }

    fn unprivileged() -> Self {
        Self { allow_privileged: false }
    }
}

impl TypedValueParser for PortParser {
    type Value = u16;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, Error> {
        let value_str = value.to_str().ok_or_else(|| {
            Error::raw(ErrorKind::InvalidUtf8, "Port must be valid UTF-8")
        })?;

        // Try to parse as number
        let port: u16 = match value_str.parse() {
            Ok(p) => p,
            Err(_) => {
                // Check for common mistakes
                if value_str.contains(':') {
                    return Err(Error::raw(
                        ErrorKind::ValueValidation,
                        format!(
                            "Expected port number, got '{}'\n\
                             Hint: Use just the port number without host (e.g., 8080)",
                            value_str
                        ),
                    ));
                }

                return Err(Error::raw(
                    ErrorKind::ValueValidation,
                    format!(
                        "'{}' is not a valid port number\n\
                         Port must be a number between 0 and 65535",
                        value_str
                    ),
                ));
            }
        };

        // Check privileged port range
        if !self.allow_privileged && port < 1024 {
            return Err(Error::raw(
                ErrorKind::ValueValidation,
                format!(
                    "Port {} is in the privileged range (0-1023)\n\
                     Hint: Use a port >= 1024, or run with elevated privileges",
                    port
                ),
            ));
        }

        // Check for commonly problematic ports
        if port == 0 {
            return Err(Error::raw(
                ErrorKind::ValueValidation,
                "Port 0 is reserved and cannot be used directly\n\
                 Hint: Use a specific port number or let the system assign one",
            ));
        }

        Ok(port)
    }
}

use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    /// Server port (unprivileged)
    #[arg(short, long, value_parser = PortParser::unprivileged(), default_value = "8080")]
    port: u16,

    /// Admin port (privileged allowed)
    #[arg(long, value_parser = PortParser::privileged())]
    admin_port: Option<u16>,
}
```

### Validation with Suggestions

Provide actionable suggestions on errors:

```rust
use clap::Parser;

fn validate_identifier(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("Identifier cannot be empty".to_string());
    }

    // Check first character
    let first = s.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return Err(format!(
            "Identifier '{}' must start with a letter or underscore\n\
             Suggestion: Try '_{}'",
            s, s
        ));
    }

    // Check all characters
    for (i, c) in s.chars().enumerate() {
        if !c.is_alphanumeric() && c != '_' {
            let suggestion = s.replace(c, "_");
            return Err(format!(
                "Invalid character '{}' at position {} in '{}'\n\
                 Identifiers can only contain letters, numbers, and underscores\n\
                 Suggestion: Try '{}'",
                c, i, s, suggestion
            ));
        }
    }

    // Check reserved words
    let reserved = ["if", "else", "for", "while", "fn", "let", "mut", "pub"];
    if reserved.contains(&s) {
        return Err(format!(
            "'{}' is a reserved keyword\n\
             Suggestion: Try '{}' or 'my_{}'",
            s,
            s.to_uppercase(),
            s
        ));
    }

    Ok(s.to_string())
}

#[derive(Parser, Debug)]
struct Cli {
    /// Variable name
    #[arg(short, long, value_parser = validate_identifier)]
    name: String,
}
```

## Common Pitfalls

1. **Panicking in parsers**: Always return `Result`, never `unwrap()` or `expect()` in parser functions.

2. **Ignoring OsStr encoding**: Not all command-line arguments are valid UTF-8. Handle `to_str()` returning `None`.

3. **Unclear error messages**: "Invalid value" tells users nothing. Include the value, expected format, and suggestions.

4. **Over-validating**: Don't reject edge cases that might be valid. When in doubt, accept and warn.

5. **Not testing edge cases**: Test empty strings, Unicode, very long values, and boundary conditions.

## Pro Tips

- **Use `value_parser!` macro for common types**: `clap::value_parser!(u16).range(1..100)` is cleaner than custom parsers
- **Implement `possible_values()` on TypedValueParser**: Enables shell completion for your custom types
- **Cache expensive validation**: If validating against external resources, cache results
- **Provide `--dry-run` for destructive parsers**: Let users see what would happen without side effects
- **Log parsing decisions**: In debug mode, log why values were accepted or rejected

## Summary

Effective value parsing creates robust, user-friendly CLIs:

1. **Built-in parsers** handle common types with zero configuration
2. **Function-based parsers** provide simple custom validation
3. **TypedValueParser trait** enables advanced, context-aware parsing
4. **Rich error messages** guide users toward correct input
5. **Validation pipelines** combine multiple checks cleanly

Master these patterns to build CLIs that transform raw strings into validated, type-safe values while providing excellent error feedback.

---

*Next: [Environment and Config Integration](./10-environment-config-integration.md)*
