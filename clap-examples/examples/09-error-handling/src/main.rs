//! # Example 09: Error Handling
//!
//! Demonstrates custom error types, suggestions, error recovery,
//! and user-friendly error messages in CLI applications.
//!
//! ## Run Examples:
//! ```bash
//! # Successful operations
//! cargo run -p error-handling -- file read existing.txt
//! cargo run -p error-handling -- user get admin
//!
//! # Trigger validation errors (clap-level)
//! cargo run -p error-handling -- file read   # missing filename
//! cargo run -p error-handling -- file read f.txt --retries 100  # out of range
//!
//! # Trigger application errors
//! cargo run -p error-handling -- file read nonexistent.txt
//! cargo run -p error-handling -- user get unknown_user
//! cargo run -p error-handling -- network fetch invalid://url
//!
//! # Error with suggestions
//! cargo run -p error-handling -- user get admn  # typo -> suggests "admin"
//!
//! # Chain of errors (context)
//! cargo run -p error-handling -- config load bad.toml
//!
//! # Verbose error output
//! cargo run -p error-handling -- --verbose file read missing.txt
//!
//! # Custom exit codes
//! cargo run -p error-handling -- file read missing.txt; echo "Exit code: $?"
//! ```

use clap::{Parser, Subcommand, Args, CommandFactory};
use thiserror::Error;
use anyhow::{Context, Result, bail, anyhow};
use std::path::PathBuf;

// =============================================================================
// CUSTOM ERROR TYPES
// =============================================================================

/// Application-specific errors using thiserror.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("File not found: {path}")]
    FileNotFound {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Permission denied: {path}")]
    PermissionDenied {
        path: PathBuf,
    },

    #[error("User not found: '{username}'")]
    UserNotFound {
        username: String,
        suggestions: Vec<String>,
    },

    #[error("Invalid URL: {url}")]
    InvalidUrl {
        url: String,
        reason: String,
    },

    #[error("Configuration error: {message}")]
    ConfigError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Network error: {message}")]
    NetworkError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Operation timed out after {seconds}s")]
    Timeout {
        seconds: u64,
        operation: String,
    },

    #[error("Rate limited: retry after {retry_after} seconds")]
    RateLimited {
        retry_after: u64,
    },

    #[error("Validation failed: {0}")]
    ValidationError(String),
}

impl AppError {
    /// Get the appropriate exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::FileNotFound { .. } => 2,
            AppError::PermissionDenied { .. } => 3,
            AppError::UserNotFound { .. } => 4,
            AppError::InvalidUrl { .. } => 5,
            AppError::ConfigError { .. } => 6,
            AppError::NetworkError { .. } => 7,
            AppError::Timeout { .. } => 8,
            AppError::RateLimited { .. } => 9,
            AppError::ValidationError(_) => 10,
        }
    }

    /// Check if this error has suggestions for the user.
    pub fn suggestions(&self) -> Option<&[String]> {
        match self {
            AppError::UserNotFound { suggestions, .. } if !suggestions.is_empty() => {
                Some(suggestions)
            }
            _ => None,
        }
    }
}

// =============================================================================
// CLI DEFINITION
// =============================================================================

#[derive(Parser, Debug)]
#[command(name = "error-demo")]
#[command(version, about = "Demonstrates error handling patterns")]
struct Cli {
    /// Enable verbose error output (show full error chain)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// File operations
    #[command(subcommand)]
    File(FileCommands),

    /// User operations
    #[command(subcommand)]
    User(UserCommands),

    /// Network operations
    #[command(subcommand)]
    Network(NetworkCommands),

    /// Configuration operations
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand, Debug)]
enum FileCommands {
    /// Read a file
    Read {
        /// File path to read
        path: PathBuf,

        /// Number of retries on failure
        #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(1..=10))]
        retries: u8,
    },

    /// Write to a file
    Write {
        /// File path to write
        path: PathBuf,

        /// Content to write
        content: String,

        /// Force overwrite
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
enum UserCommands {
    /// Get user information
    Get {
        /// Username to look up
        username: String,
    },

    /// Create a new user
    Create(CreateUserArgs),
}

#[derive(Args, Debug)]
struct CreateUserArgs {
    /// Username
    username: String,

    /// Email address
    #[arg(short, long)]
    email: String,
}

#[derive(Subcommand, Debug)]
enum NetworkCommands {
    /// Fetch a URL
    Fetch {
        /// URL to fetch
        url: String,

        /// Request timeout in seconds
        #[arg(long, default_value_t = 30)]
        timeout: u64,
    },

    /// Check connectivity
    Ping {
        /// Host to ping
        host: String,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Load configuration
    Load {
        /// Config file path
        path: PathBuf,
    },

    /// Validate configuration
    Validate {
        /// Config file path
        path: PathBuf,
    },
}

// =============================================================================
// SIMULATED OPERATIONS
// =============================================================================

/// Simulated known users for demonstration
const KNOWN_USERS: &[&str] = &["admin", "alice", "bob", "charlie", "david"];

fn read_file(path: &PathBuf, _retries: u8) -> Result<String> {
    // Simulate file reading
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    if filename.contains("nonexistent") || filename.contains("missing") {
        return Err(AppError::FileNotFound {
            path: path.clone(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
        }.into());
    }

    if filename.contains("protected") {
        return Err(AppError::PermissionDenied {
            path: path.clone(),
        }.into());
    }

    Ok(format!("Contents of {:?}", path))
}

fn write_file(path: &PathBuf, content: &str, force: bool) -> Result<()> {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    if filename.contains("readonly") {
        return Err(AppError::PermissionDenied {
            path: path.clone(),
        }.into());
    }

    if !force && filename.contains("exists") {
        bail!("File already exists. Use --force to overwrite.");
    }

    println!("Would write {} bytes to {:?}", content.len(), path);
    Ok(())
}

fn get_user(username: &str) -> Result<()> {
    // Check if user exists
    if KNOWN_USERS.contains(&username) {
        println!("User found: {}", username);
        return Ok(());
    }

    // Find similar usernames for suggestions
    let suggestions: Vec<String> = KNOWN_USERS
        .iter()
        .filter(|u| similar(u, username))
        .map(|s| s.to_string())
        .collect();

    Err(AppError::UserNotFound {
        username: username.to_string(),
        suggestions,
    }.into())
}

fn create_user(args: &CreateUserArgs) -> Result<()> {
    // Validate email format
    if !args.email.contains('@') {
        return Err(AppError::ValidationError(
            format!("Invalid email format: '{}'", args.email)
        ).into());
    }

    // Check for duplicate username
    if KNOWN_USERS.contains(&args.username.as_str()) {
        bail!("Username '{}' already exists", args.username);
    }

    println!("Would create user: {} <{}>", args.username, args.email);
    Ok(())
}

fn fetch_url(url: &str, timeout: u64) -> Result<()> {
    // Validate URL format
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(AppError::InvalidUrl {
            url: url.to_string(),
            reason: "URL must start with http:// or https://".to_string(),
        }.into());
    }

    // Simulate timeout
    if url.contains("slow") {
        return Err(AppError::Timeout {
            seconds: timeout,
            operation: format!("fetching {}", url),
        }.into());
    }

    // Simulate rate limiting
    if url.contains("ratelimit") {
        return Err(AppError::RateLimited {
            retry_after: 60,
        }.into());
    }

    println!("Would fetch: {} (timeout: {}s)", url, timeout);
    Ok(())
}

fn ping_host(host: &str) -> Result<()> {
    if host.contains("unreachable") {
        return Err(AppError::NetworkError {
            message: format!("Host {} is unreachable", host),
            source: None,
        }.into());
    }

    println!("Pinging {}...", host);
    Ok(())
}

fn load_config(path: &PathBuf) -> Result<()> {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    if filename.contains("bad") || filename.contains("invalid") {
        return Err(AppError::ConfigError {
            message: "Invalid TOML syntax".to_string(),
            source: Some(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "expected '=' at line 3, column 5"
            ))),
        }.into());
    }

    if filename.contains("missing") {
        // Use anyhow context for chained errors
        std::fs::read_to_string(path)
            .with_context(|| format!("Failed to load config from {:?}", path))?;
    }

    println!("Loaded config from {:?}", path);
    Ok(())
}

/// Simple similarity check for suggestions
fn similar(a: &str, b: &str) -> bool {
    if a == b {
        return false; // Not similar if exactly same
    }

    // Levenshtein-like: allow 1-2 character differences
    let min_len = a.len().min(b.len());
    let max_len = a.len().max(b.len());

    if max_len - min_len > 2 {
        return false;
    }

    let mut diffs = 0;
    for (ca, cb) in a.chars().zip(b.chars()) {
        if ca != cb {
            diffs += 1;
        }
    }

    diffs + (max_len - min_len) <= 2
}

// =============================================================================
// ERROR DISPLAY
// =============================================================================

fn display_error(error: &anyhow::Error, verbose: bool) {
    eprintln!("error: {}", error);

    // Check for suggestions
    if let Some(app_err) = error.downcast_ref::<AppError>() {
        if let Some(suggestions) = app_err.suggestions() {
            eprintln!("\nDid you mean one of these?");
            for suggestion in suggestions {
                eprintln!("    {}", suggestion);
            }
        }
    }

    // Show error chain in verbose mode
    if verbose {
        let chain: Vec<_> = error.chain().skip(1).collect();
        if !chain.is_empty() {
            eprintln!("\nCaused by:");
            for (i, cause) in chain.into_iter().enumerate() {
                eprintln!("    {}: {}", i, cause);
            }
        }
    }
}

fn get_exit_code(error: &anyhow::Error) -> i32 {
    if let Some(app_err) = error.downcast_ref::<AppError>() {
        app_err.exit_code()
    } else {
        1 // Generic error
    }
}

// =============================================================================
// MAIN
// =============================================================================

fn main() {
    // Custom error handling for clap parse errors
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Clap provides nice error formatting
            e.exit();
        }
    };

    // Run the application and handle errors
    if let Err(e) = run(cli) {
        display_error(&e, cli_was_verbose());
        std::process::exit(get_exit_code(&e));
    }
}

// Helper to check verbosity (need to parse twice for this pattern)
fn cli_was_verbose() -> bool {
    std::env::args().any(|a| a == "-v" || a == "--verbose")
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::File(cmd) => match cmd {
            FileCommands::Read { path, retries } => {
                let content = read_file(&path, retries)?;
                if !cli.quiet {
                    println!("{}", content);
                }
            }
            FileCommands::Write { path, content, force } => {
                write_file(&path, &content, force)?;
            }
        },

        Commands::User(cmd) => match cmd {
            UserCommands::Get { username } => {
                get_user(&username)?;
            }
            UserCommands::Create(args) => {
                create_user(&args)?;
            }
        },

        Commands::Network(cmd) => match cmd {
            NetworkCommands::Fetch { url, timeout } => {
                fetch_url(&url, timeout)?;
            }
            NetworkCommands::Ping { host } => {
                ping_host(&host)?;
            }
        },

        Commands::Config(cmd) => match cmd {
            ConfigCommands::Load { path } => {
                load_config(&path)?;
            }
            ConfigCommands::Validate { path } => {
                load_config(&path)?;
                println!("Configuration is valid");
            }
        },
    }

    Ok(())
}

// =============================================================================
// KEY CONCEPTS:
// =============================================================================
//
// 1. THISERROR CRATE:
//    #[derive(Error)] for custom error types
//    #[error("...")] for Display implementation
//    #[source] to chain underlying errors
//
// 2. ANYHOW CRATE:
//    Result<T> = anyhow::Result<T> for easy error handling
//    .context() to add context to errors
//    bail!() for quick error returns
//    Error chain traversal with .chain()
//
// 3. CLAP ERROR HANDLING:
//    Cli::try_parse() for custom error handling
//    e.exit() for clap's formatted errors
//    Custom validation in value_parser
//
// 4. EXIT CODES:
//    Use specific exit codes for different errors
//    Standard: 0 = success, 1 = general error, 2+ = specific
//    Document exit codes in --help
//
// 5. USER-FRIENDLY MESSAGES:
//    Show suggestions for typos
//    Hide internal details unless --verbose
//    Include actionable information
//
// 6. ERROR CHAIN:
//    Use #[source] to link to underlying errors
//    Use .context() to add high-level context
//    Show full chain with --verbose
//
// BEST PRACTICES:
//
// - Define custom error types for your domain
// - Use exit codes consistently
// - Provide suggestions for common mistakes
// - Show context without overwhelming users
// - Support --verbose for debugging
// - Never expose internal errors to users by default
// - Always include enough info to fix the problem
