# Advanced Error Handling Strategies

> **Chapter 14** | Part 3: Advanced Architecture | Estimated reading time: 18 minutes

Error handling is where CLI applications distinguish themselves. Users encountering errors should receive actionable guidance, not cryptic messages. This chapter explores sophisticated error handling patterns that transform frustrating failures into helpful interactions.

## Error Recovery Patterns

### Graceful Degradation

When errors occur, provide fallback behavior rather than immediate failure:

```rust
use std::path::PathBuf;
use anyhow::{Context, Result};

struct ConfigLoader {
    primary_path: PathBuf,
    fallback_paths: Vec<PathBuf>,
}

impl ConfigLoader {
    fn load(&self) -> Result<Config> {
        // Try primary path first
        match self.load_from(&self.primary_path) {
            Ok(config) => return Ok(config),
            Err(e) => {
                eprintln!(
                    "Warning: Could not load config from {:?}: {}",
                    self.primary_path, e
                );
                eprintln!("Trying fallback locations...");
            }
        }

        // Try fallbacks in order
        for path in &self.fallback_paths {
            match self.load_from(path) {
                Ok(config) => {
                    eprintln!("Loaded config from {:?}", path);
                    return Ok(config);
                }
                Err(_) => continue,
            }
        }

        // All failed - use defaults with warning
        eprintln!("Warning: No config file found, using defaults");
        Ok(Config::default())
    }

    fn load_from(&self, path: &PathBuf) -> Result<Config> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {:?}", path))?;
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse {:?}", path))
    }
}
```

### Interactive Error Recovery

For terminal applications, offer users a chance to correct mistakes:

```rust
use dialoguer::{Confirm, Input, Select};
use std::path::PathBuf;

#[derive(Debug)]
enum RecoveryAction {
    Retry,
    UseAlternative(String),
    Abort,
}

fn recover_file_not_found(path: &PathBuf, is_interactive: bool) -> Result<RecoveryAction> {
    if !is_interactive {
        return Ok(RecoveryAction::Abort);
    }

    eprintln!("Error: File not found: {:?}", path);

    // Suggest similar files
    let suggestions = find_similar_files(path);

    if !suggestions.is_empty() {
        let items: Vec<String> = suggestions
            .iter()
            .map(|p| p.display().to_string())
            .chain(std::iter::once("Enter a different path".to_string()))
            .chain(std::iter::once("Abort".to_string()))
            .collect();

        let selection = Select::new()
            .with_prompt("Did you mean one of these?")
            .items(&items)
            .default(0)
            .interact()?;

        if selection < suggestions.len() {
            return Ok(RecoveryAction::UseAlternative(
                suggestions[selection].display().to_string()
            ));
        } else if selection == suggestions.len() {
            let new_path: String = Input::new()
                .with_prompt("Enter file path")
                .interact_text()?;
            return Ok(RecoveryAction::UseAlternative(new_path));
        }
    } else {
        if Confirm::new()
            .with_prompt("Would you like to enter a different path?")
            .interact()?
        {
            let new_path: String = Input::new()
                .with_prompt("Enter file path")
                .interact_text()?;
            return Ok(RecoveryAction::UseAlternative(new_path));
        }
    }

    Ok(RecoveryAction::Abort)
}

fn find_similar_files(path: &PathBuf) -> Vec<PathBuf> {
    let parent = path.parent().unwrap_or(std::path::Path::new("."));
    let filename = path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    std::fs::read_dir(parent)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            strsim::jaro_winkler(&name, &filename) > 0.7
        })
        .map(|e| e.path())
        .take(5)
        .collect()
}
```

### Retry with Backoff

For transient errors (network, busy files), implement smart retries:

```rust
use std::time::Duration;
use std::thread::sleep;

#[derive(Clone)]
struct RetryPolicy {
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
    backoff_factor: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_factor: 2.0,
        }
    }
}

fn retry_with_backoff<T, E, F>(policy: &RetryPolicy, mut operation: F) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut delay = policy.initial_delay;
    let mut last_error = None;

    for attempt in 1..=policy.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                eprintln!(
                    "Attempt {}/{} failed: {}",
                    attempt, policy.max_attempts, e
                );
                last_error = Some(e);

                if attempt < policy.max_attempts {
                    eprintln!("Retrying in {:?}...", delay);
                    sleep(delay);
                    delay = Duration::from_secs_f64(
                        (delay.as_secs_f64() * policy.backoff_factor)
                            .min(policy.max_delay.as_secs_f64())
                    );
                }
            }
        }
    }

    Err(last_error.unwrap())
}

// Usage
fn fetch_with_retry(url: &str) -> Result<String, reqwest::Error> {
    let policy = RetryPolicy::default();

    retry_with_backoff(&policy, || {
        reqwest::blocking::get(url)?.text()
    })
}
```

## Suggestion Engines

### Did-You-Mean for Commands

Suggest similar valid commands when users make typos:

```rust
use strsim::levenshtein;

struct CommandSuggester {
    commands: Vec<&'static str>,
    max_distance: usize,
}

impl CommandSuggester {
    fn new(commands: Vec<&'static str>) -> Self {
        Self {
            commands,
            max_distance: 3,
        }
    }

    fn suggest(&self, unknown: &str) -> Vec<&'static str> {
        let mut suggestions: Vec<_> = self.commands
            .iter()
            .map(|cmd| (cmd, levenshtein(unknown, cmd)))
            .filter(|(_, dist)| *dist <= self.max_distance)
            .collect();

        suggestions.sort_by_key(|(_, dist)| *dist);
        suggestions.into_iter().map(|(cmd, _)| *cmd).take(3).collect()
    }

    fn format_suggestion(&self, unknown: &str) -> Option<String> {
        let suggestions = self.suggest(unknown);

        if suggestions.is_empty() {
            None
        } else if suggestions.len() == 1 {
            Some(format!("Did you mean '{}'?", suggestions[0]))
        } else {
            let formatted: Vec<_> = suggestions.iter()
                .map(|s| format!("'{}'", s))
                .collect();
            Some(format!("Did you mean one of: {}?", formatted.join(", ")))
        }
    }
}

// Integrate with Clap
fn handle_unknown_command(cmd: &str) {
    let suggester = CommandSuggester::new(vec![
        "init", "build", "run", "test", "deploy",
        "install", "uninstall", "update", "config",
    ]);

    eprintln!("error: '{}' is not a valid command", cmd);

    if let Some(suggestion) = suggester.format_suggestion(cmd) {
        eprintln!("\n{}", suggestion);
    }

    eprintln!("\nRun 'myapp --help' for a list of commands.");
}
```

### Argument Value Suggestions

Suggest valid values for enumerated arguments:

```rust
use clap::{Error, error::ErrorKind};

fn suggest_enum_value<T>(input: &str, valid_values: &[T]) -> String
where
    T: AsRef<str> + std::fmt::Display,
{
    let valid_strs: Vec<&str> = valid_values.iter()
        .map(|v| v.as_ref())
        .collect();

    let suggestions: Vec<_> = valid_strs.iter()
        .filter(|v| {
            v.contains(input) ||
            input.contains(*v) ||
            levenshtein(input, v) <= 2
        })
        .take(3)
        .collect();

    if suggestions.is_empty() {
        format!(
            "Invalid value '{}'. Valid values are: {}",
            input,
            valid_strs.join(", ")
        )
    } else {
        format!(
            "Invalid value '{}'. Did you mean: {}?",
            input,
            suggestions.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", ")
        )
    }
}

// Custom value parser with suggestions
fn parse_log_level(s: &str) -> Result<LogLevel, String> {
    match s.to_lowercase().as_str() {
        "trace" => Ok(LogLevel::Trace),
        "debug" => Ok(LogLevel::Debug),
        "info" => Ok(LogLevel::Info),
        "warn" | "warning" => Ok(LogLevel::Warn),
        "error" => Ok(LogLevel::Error),
        _ => Err(suggest_enum_value(s, &["trace", "debug", "info", "warn", "error"])),
    }
}
```

## Context Preservation

### Rich Error Context with anyhow

Build comprehensive error context as errors propagate:

```rust
use anyhow::{Context, Result, bail};
use std::path::Path;

fn process_file(path: &Path) -> Result<ProcessedData> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let parsed = parse_content(&content)
        .with_context(|| format!("Failed to parse content from: {}", path.display()))?;

    validate_data(&parsed)
        .with_context(|| format!("Validation failed for: {}", path.display()))?;

    transform_data(parsed)
        .with_context(|| format!("Transformation failed for: {}", path.display()))
}

fn parse_content(content: &str) -> Result<ParsedContent> {
    let lines: Vec<_> = content.lines().collect();

    if lines.is_empty() {
        bail!("File is empty");
    }

    let header = parse_header(lines[0])
        .with_context(|| format!("Invalid header on line 1"))?;

    let records: Result<Vec<_>> = lines[1..]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            parse_record(line)
                .with_context(|| format!("Invalid record on line {}", i + 2))
        })
        .collect();

    Ok(ParsedContent {
        header,
        records: records?,
    })
}

// Display full error chain
fn display_error(err: &anyhow::Error) {
    eprintln!("Error: {}", err);

    let chain: Vec<_> = err.chain().skip(1).collect();
    if !chain.is_empty() {
        eprintln!("\nCaused by:");
        for (i, cause) in chain.iter().enumerate() {
            eprintln!("    {}: {}", i + 1, cause);
        }
    }

    // Include backtrace in debug mode
    if std::env::var("RUST_BACKTRACE").is_ok() {
        eprintln!("\nBacktrace:\n{:?}", err.backtrace());
    }
}
```

### Structured Error Types with thiserror

Create domain-specific error types with rich context:

```rust
use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error")]
    Config(#[from] ConfigError),

    #[error("I/O error accessing {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Parse error in {file} at line {line}")]
    Parse {
        file: PathBuf,
        line: usize,
        message: String,
    },

    #[error("Network error connecting to {host}:{port}")]
    Network {
        host: String,
        port: u16,
        #[source]
        source: std::io::Error,
    },

    #[error("Validation failed: {0}")]
    Validation(String),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    #[error("Invalid value for '{field}': {message}")]
    InvalidValue {
        field: &'static str,
        message: String,
    },

    #[error("Config file not found: {0}")]
    NotFound(PathBuf),

    #[error("Config parse error")]
    Parse(#[from] toml::de::Error),
}

impl AppError {
    /// Get a user-friendly short description
    pub fn user_message(&self) -> String {
        match self {
            Self::Config(ConfigError::NotFound(path)) => {
                format!(
                    "Config file not found at {}. Run 'myapp init' to create one.",
                    path.display()
                )
            }
            Self::Io { path, source } => {
                format!("Cannot access {}: {}", path.display(), source)
            }
            Self::Network { host, port, .. } => {
                format!("Cannot connect to {}:{}. Check your network connection.", host, port)
            }
            _ => self.to_string(),
        }
    }

    /// Get exit code for this error type
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) => 78,      // EX_CONFIG
            Self::Io { .. } => 74,      // EX_IOERR
            Self::Parse { .. } => 65,   // EX_DATAERR
            Self::Network { .. } => 69, // EX_UNAVAILABLE
            Self::Validation(_) => 65,  // EX_DATAERR
        }
    }
}
```

## Error Serialization

### JSON Error Output

For machine-readable error output (CI/CD, tooling):

```rust
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct JsonError {
    error: bool,
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<ErrorLocation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    causes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    suggestions: Vec<String>,
}

#[derive(Serialize)]
struct ErrorLocation {
    file: Option<String>,
    line: Option<usize>,
    column: Option<usize>,
}

impl From<&AppError> for JsonError {
    fn from(err: &AppError) -> Self {
        let (code, location, suggestions) = match err {
            AppError::Config(ConfigError::MissingField(field)) => (
                "CONFIG_MISSING_FIELD".to_string(),
                None,
                vec![format!("Add '{}' to your config file", field)],
            ),
            AppError::Parse { file, line, message } => (
                "PARSE_ERROR".to_string(),
                Some(ErrorLocation {
                    file: Some(file.display().to_string()),
                    line: Some(*line),
                    column: None,
                }),
                vec![format!("Check syntax near line {}", line)],
            ),
            AppError::Network { host, port, .. } => (
                "NETWORK_ERROR".to_string(),
                None,
                vec![
                    format!("Verify {}:{} is reachable", host, port),
                    "Check firewall settings".to_string(),
                ],
            ),
            _ => (
                "UNKNOWN_ERROR".to_string(),
                None,
                vec![],
            ),
        };

        // Collect error chain
        let causes: Vec<String> = std::iter::successors(
            err.source(),
            |e| e.source()
        )
        .map(|e| e.to_string())
        .collect();

        JsonError {
            error: true,
            code,
            message: err.user_message(),
            location,
            causes,
            suggestions,
        }
    }
}

fn output_error(err: &AppError, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            eprintln!("error: {}", err.user_message());
        }
        OutputFormat::Json => {
            let json_err = JsonError::from(err);
            eprintln!("{}", serde_json::to_string_pretty(&json_err).unwrap());
        }
        OutputFormat::Yaml => {
            let json_err = JsonError::from(err);
            eprintln!("{}", serde_yaml::to_string(&json_err).unwrap());
        }
    }
}
```

### Error Reports for Bug Reporting

Generate comprehensive error reports users can share:

```rust
use std::collections::HashMap;

#[derive(Serialize)]
struct ErrorReport {
    timestamp: String,
    version: String,
    os: String,
    arch: String,
    error: JsonError,
    environment: HashMap<String, String>,
    recent_commands: Vec<String>,
    config_summary: ConfigSummary,
}

#[derive(Serialize)]
struct ConfigSummary {
    config_file_exists: bool,
    profile: String,
    // Redacted sensitive values
}

fn generate_error_report(err: &AppError) -> ErrorReport {
    // Collect environment (filter sensitive vars)
    let safe_vars = ["PATH", "HOME", "SHELL", "TERM", "USER"];
    let environment: HashMap<String, String> = std::env::vars()
        .filter(|(k, _)| safe_vars.contains(&k.as_str()) || k.starts_with("MYAPP_"))
        .filter(|(k, _)| !k.contains("SECRET") && !k.contains("TOKEN") && !k.contains("KEY"))
        .collect();

    ErrorReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        error: JsonError::from(err),
        environment,
        recent_commands: vec![], // Could read from history
        config_summary: ConfigSummary {
            config_file_exists: std::path::Path::new("config.toml").exists(),
            profile: std::env::var("MYAPP_PROFILE").unwrap_or_else(|_| "default".to_string()),
        },
    }
}

fn create_bug_report_file(err: &AppError) -> Result<PathBuf, std::io::Error> {
    let report = generate_error_report(err);
    let filename = format!("myapp-error-{}.json", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    let path = std::env::temp_dir().join(&filename);

    std::fs::write(&path, serde_json::to_string_pretty(&report)?)?;

    eprintln!("\nError report saved to: {}", path.display());
    eprintln!("Please include this file when reporting issues.");

    Ok(path)
}
```

## Integration with Error Aggregators

### Sentry Integration

Send errors to Sentry for production monitoring:

```rust
use sentry::{ClientOptions, IntoDsn};

fn init_error_reporting() -> Option<sentry::ClientInitGuard> {
    let dsn = std::env::var("SENTRY_DSN").ok()?;

    let guard = sentry::init((dsn, ClientOptions {
        release: Some(env!("CARGO_PKG_VERSION").into()),
        environment: std::env::var("MYAPP_ENV").ok().map(Into::into),
        attach_stacktrace: true,
        ..Default::default()
    }));

    Some(guard)
}

fn capture_error(err: &AppError) {
    // Add context
    sentry::configure_scope(|scope| {
        scope.set_tag("error_type", error_type_tag(err));
        scope.set_extra("exit_code", err.exit_code().into());
    });

    // Capture based on severity
    match err {
        AppError::Config(_) => {
            // Config errors are usually user mistakes, not bugs
            sentry::capture_message(
                &err.user_message(),
                sentry::Level::Warning
            );
        }
        _ => {
            sentry::capture_error(err);
        }
    }
}

fn error_type_tag(err: &AppError) -> &'static str {
    match err {
        AppError::Config(_) => "config",
        AppError::Io { .. } => "io",
        AppError::Parse { .. } => "parse",
        AppError::Network { .. } => "network",
        AppError::Validation(_) => "validation",
    }
}
```

### Custom Telemetry

For self-hosted error tracking:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

static TELEMETRY_ENABLED: AtomicBool = AtomicBool::new(false);

fn init_telemetry(opt_in: bool) {
    TELEMETRY_ENABLED.store(opt_in, Ordering::Relaxed);
}

async fn report_error_telemetry(err: &AppError) {
    if !TELEMETRY_ENABLED.load(Ordering::Relaxed) {
        return;
    }

    // Anonymous error report
    let payload = serde_json::json!({
        "v": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "code": err.exit_code(),
        "type": error_type_tag(err),
        // No PII, no file paths, no error messages
    });

    // Fire and forget - don't block on telemetry
    tokio::spawn(async move {
        let _ = reqwest::Client::new()
            .post("https://telemetry.example.com/cli-errors")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;
    });
}
```

## When NOT To Use Advanced Error Handling

Complex error handling adds code and cognitive overhead. Avoid it when:

1. **Simple scripts**: A `panic!` with a message may suffice
2. **Internal tools**: Users can read stack traces
3. **Prototypes**: Get it working first, polish errors later
4. **Performance-critical paths**: Error construction has cost

**Warning signs of over-engineering errors**:
- More error types than commands
- Error formatting code larger than business logic
- Spending more time on error messages than features

### Simpler Alternative: anyhow Only

For many CLIs, `anyhow` provides sufficient error handling:

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;

    let result = process(&config)
        .context("Processing failed")?;

    output(result)
        .context("Failed to write output")?;

    Ok(())
}
```

## Summary

Error handling is a user experience investment. The patterns in this chapter transform error messages from obstacles into guides.

### Key Takeaways

1. **Graceful degradation**: Provide fallbacks when possible
2. **Interactive recovery**: Let users correct mistakes in terminals
3. **Suggestion engines**: Use Levenshtein distance for "did you mean"
4. **Rich context**: Chain errors with `anyhow::Context`
5. **Structured errors**: Use `thiserror` for domain-specific types
6. **Machine-readable output**: JSON errors for tooling integration
7. **Error telemetry**: Opt-in reporting for production debugging

### Architecture Decisions Documented

| Decision | Recommendation | Rationale |
|----------|----------------|-----------|
| Error library | `anyhow` for apps, `thiserror` for libs | Balance flexibility and type safety |
| Suggestions | Levenshtein distance <= 3 | Catches most typos without noise |
| Exit codes | Follow sysexits.h conventions | Unix compatibility |
| Telemetry | Opt-in, anonymous, non-blocking | Respect privacy, don't slow CLI |

> **Cross-Reference**: See [Chapter 5](../part1-foundations/05-error-handling-foundations.md) for Clap's built-in error types, and [Chapter 18](../part4-real-world-systems/18-case-study-interactive-clis.md) for interactive error recovery patterns.

---

*Next: [Testing CLI Applications](./15-testing-cli-applications.md)*
