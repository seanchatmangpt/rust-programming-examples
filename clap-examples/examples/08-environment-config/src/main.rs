//! # Example 08: Environment Variables and Configuration
//!
//! Demonstrates environment variable support, config file loading,
//! and configuration layering (CLI > Env > Config > Defaults).
//!
//! ## Run Examples:
//! ```bash
//! # Using defaults
//! cargo run -p environment-config
//!
//! # Using environment variables
//! APP_HOST=0.0.0.0 APP_PORT=3000 cargo run -p environment-config
//!
//! # CLI args override environment
//! APP_PORT=3000 cargo run -p environment-config -- --port 8080
//!
//! # Using a config file (create app.toml first)
//! cargo run -p environment-config -- --config app.toml
//!
//! # Full override chain: CLI > Env > Config > Defaults
//! APP_LOG_LEVEL=warn cargo run -p environment-config -- \
//!     --config app.toml --port 9000
//!
//! # Show resolved configuration
//! cargo run -p environment-config -- --show-config
//!
//! # With prefix for all env vars
//! MYAPP_HOST=localhost MYAPP_PORT=4000 cargo run -p environment-config
//! ```
//!
//! ## Create a sample config file (app.toml):
//! ```toml
//! host = "127.0.0.1"
//! port = 5000
//! log_level = "debug"
//! workers = 8
//!
//! [database]
//! url = "postgres://localhost/mydb"
//! pool_size = 10
//!
//! [features]
//! enable_cache = true
//! enable_metrics = true
//! ```

use clap::{Parser, ValueEnum};
use serde::Deserialize;
use std::path::PathBuf;
use std::fs;

// =============================================================================
// CONFIGURATION STRUCTURE
// =============================================================================

/// File-based configuration (loaded from TOML)
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct FileConfig {
    host: Option<String>,
    port: Option<u16>,
    log_level: Option<String>,
    workers: Option<usize>,
    database: DatabaseConfig,
    features: FeatureFlags,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct DatabaseConfig {
    url: Option<String>,
    pool_size: Option<u32>,
    timeout_seconds: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct FeatureFlags {
    enable_cache: Option<bool>,
    enable_metrics: Option<bool>,
    enable_tracing: Option<bool>,
}

// =============================================================================
// CLI DEFINITION WITH ENV SUPPORT
// =============================================================================

/// Application with environment and config file support.
///
/// Configuration priority (highest to lowest):
/// 1. Command-line arguments
/// 2. Environment variables
/// 3. Configuration file
/// 4. Default values
#[derive(Parser, Debug)]
#[command(name = "myapp")]
#[command(version, about)]
struct Cli {
    // =========================================================================
    // CONFIG FILE
    // =========================================================================

    /// Path to configuration file (TOML format)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Show resolved configuration and exit
    #[arg(long)]
    show_config: bool,

    // =========================================================================
    // SERVER SETTINGS (with env support)
    // =========================================================================

    /// Server host address
    ///
    /// Can also be set via APP_HOST or MYAPP_HOST environment variable.
    #[arg(long, env = "APP_HOST", default_value = "localhost")]
    host: String,

    /// Server port
    ///
    /// Can also be set via APP_PORT or MYAPP_PORT environment variable.
    #[arg(short, long, env = "APP_PORT", default_value_t = 8080)]
    port: u16,

    /// Number of worker threads
    ///
    /// Defaults to number of CPU cores if not specified.
    #[arg(short, long, env = "APP_WORKERS")]
    workers: Option<usize>,

    // =========================================================================
    // LOGGING (with env support)
    // =========================================================================

    /// Log level
    #[arg(long, env = "APP_LOG_LEVEL", value_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,

    /// Enable JSON logging format
    #[arg(long, env = "APP_LOG_JSON")]
    log_json: bool,

    // =========================================================================
    // DATABASE (with env support)
    // =========================================================================

    /// Database connection URL
    ///
    /// Can also be set via DATABASE_URL environment variable.
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Database connection pool size
    #[arg(long, env = "DATABASE_POOL_SIZE", default_value_t = 5)]
    database_pool_size: u32,

    /// Database connection timeout (seconds)
    #[arg(long, env = "DATABASE_TIMEOUT", default_value_t = 30)]
    database_timeout: u64,

    // =========================================================================
    // FEATURE FLAGS (with env support)
    // =========================================================================

    /// Enable caching
    #[arg(long, env = "ENABLE_CACHE")]
    enable_cache: bool,

    /// Enable metrics endpoint
    #[arg(long, env = "ENABLE_METRICS")]
    enable_metrics: bool,

    /// Enable distributed tracing
    #[arg(long, env = "ENABLE_TRACING")]
    enable_tracing: bool,

    // =========================================================================
    // SECRETS (env-only, no CLI for security)
    // =========================================================================
    // Note: These should ONLY come from env vars, never CLI (visible in ps)

    /// API key (env only for security)
    #[arg(skip)]
    api_key: Option<String>,

    /// Secret token (env only for security)
    #[arg(skip)]
    secret_token: Option<String>,
}

#[derive(ValueEnum, Clone, Debug, Default)]
enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

// =============================================================================
// RESOLVED CONFIGURATION
// =============================================================================

/// Final resolved configuration after merging all sources.
#[derive(Debug)]
struct ResolvedConfig {
    // Server
    host: String,
    port: u16,
    workers: usize,

    // Logging
    log_level: LogLevel,
    log_json: bool,

    // Database
    database_url: Option<String>,
    database_pool_size: u32,
    database_timeout: u64,

    // Features
    enable_cache: bool,
    enable_metrics: bool,
    enable_tracing: bool,

    // Secrets
    api_key: Option<String>,
    secret_token: Option<String>,

    // Source tracking (for debugging)
    sources: ConfigSources,
}

#[derive(Debug, Default)]
struct ConfigSources {
    host: &'static str,
    port: &'static str,
    workers: &'static str,
    log_level: &'static str,
    database_url: &'static str,
}

// =============================================================================
// CONFIGURATION MERGING
// =============================================================================

impl ResolvedConfig {
    fn from_cli_and_file(cli: Cli, file_config: Option<FileConfig>) -> Self {
        let file = file_config.unwrap_or_default();
        let mut sources = ConfigSources::default();

        // Determine number of workers
        let workers = if let Some(w) = cli.workers {
            sources.workers = "cli/env";
            w
        } else if let Some(w) = file.workers {
            sources.workers = "config file";
            w
        } else {
            sources.workers = "default (cpu count)";
            num_cpus()
        };

        // Host - CLI/env takes precedence, then file, then default
        // Note: clap already handles CLI > env > default
        // We just need to apply file config for values not set via CLI/env
        let host = if std::env::var("APP_HOST").is_ok() || std::env::var("MYAPP_HOST").is_ok() {
            sources.host = "env";
            cli.host.clone()
        } else if cli.host != "localhost" {
            sources.host = "cli";
            cli.host.clone()
        } else if let Some(h) = file.host {
            sources.host = "config file";
            h
        } else {
            sources.host = "default";
            cli.host.clone()
        };

        // Port
        let port = if std::env::var("APP_PORT").is_ok() || std::env::var("MYAPP_PORT").is_ok() {
            sources.port = "env";
            cli.port
        } else if cli.port != 8080 {
            sources.port = "cli";
            cli.port
        } else if let Some(p) = file.port {
            sources.port = "config file";
            p
        } else {
            sources.port = "default";
            cli.port
        };

        // Database URL
        let database_url = if cli.database_url.is_some() {
            sources.database_url = "cli/env";
            cli.database_url.clone()
        } else if file.database.url.is_some() {
            sources.database_url = "config file";
            file.database.url.clone()
        } else {
            sources.database_url = "not set";
            None
        };

        // Load secrets from environment only
        let api_key = std::env::var("API_KEY").ok();
        let secret_token = std::env::var("SECRET_TOKEN").ok();

        // Merge feature flags (CLI/env > file > default)
        let enable_cache = cli.enable_cache ||
            file.features.enable_cache.unwrap_or(false);
        let enable_metrics = cli.enable_metrics ||
            file.features.enable_metrics.unwrap_or(false);
        let enable_tracing = cli.enable_tracing ||
            file.features.enable_tracing.unwrap_or(false);

        ResolvedConfig {
            host,
            port,
            workers,
            log_level: cli.log_level,
            log_json: cli.log_json,
            database_url,
            database_pool_size: file.database.pool_size.unwrap_or(cli.database_pool_size),
            database_timeout: file.database.timeout_seconds.unwrap_or(cli.database_timeout),
            enable_cache,
            enable_metrics,
            enable_tracing,
            api_key,
            secret_token,
            sources,
        }
    }
}

fn num_cpus() -> usize {
    // Simplified - in real code, use the num_cpus crate
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4)
}

// =============================================================================
// MAIN
// =============================================================================

fn main() {
    let cli = Cli::parse();

    // Load config file if specified
    let file_config = if let Some(config_path) = &cli.config {
        match fs::read_to_string(config_path) {
            Ok(content) => {
                match toml::from_str::<FileConfig>(&content) {
                    Ok(config) => {
                        println!("Loaded config from: {:?}", config_path);
                        Some(config)
                    }
                    Err(e) => {
                        eprintln!("Error parsing config file: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading config file {:?}: {}", config_path, e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    let show_config = cli.show_config;

    // Merge all configuration sources
    let config = ResolvedConfig::from_cli_and_file(cli, file_config);

    if show_config {
        println!("\n=== Resolved Configuration ===\n");
        println!("Server:");
        println!("  host: {} (from: {})", config.host, config.sources.host);
        println!("  port: {} (from: {})", config.port, config.sources.port);
        println!("  workers: {} (from: {})", config.workers, config.sources.workers);

        println!("\nLogging:");
        println!("  level: {:?}", config.log_level);
        println!("  json: {}", config.log_json);

        println!("\nDatabase:");
        match &config.database_url {
            Some(url) => println!("  url: {} (from: {})", mask_password(url), config.sources.database_url),
            None => println!("  url: not configured"),
        }
        println!("  pool_size: {}", config.database_pool_size);
        println!("  timeout: {}s", config.database_timeout);

        println!("\nFeatures:");
        println!("  cache: {}", config.enable_cache);
        println!("  metrics: {}", config.enable_metrics);
        println!("  tracing: {}", config.enable_tracing);

        println!("\nSecrets:");
        println!("  api_key: {}", if config.api_key.is_some() { "[set]" } else { "[not set]" });
        println!("  secret_token: {}", if config.secret_token.is_some() { "[set]" } else { "[not set]" });
    } else {
        println!("Starting server on {}:{}...", config.host, config.port);
        println!("Workers: {}", config.workers);
        println!("Log level: {:?}", config.log_level);

        if config.enable_metrics {
            println!("Metrics enabled at /metrics");
        }
    }
}

/// Mask password in database URL for safe display
fn mask_password(url: &str) -> String {
    // Simple masking - replace password between :// and @
    if let Some(at_pos) = url.find('@') {
        if let Some(scheme_end) = url.find("://") {
            let scheme = &url[..scheme_end + 3];
            let rest = &url[at_pos..];
            if let Some(colon) = url[scheme_end + 3..at_pos].find(':') {
                let user = &url[scheme_end + 3..scheme_end + 3 + colon];
                return format!("{}{}:****{}", scheme, user, rest);
            }
        }
    }
    url.to_string()
}

// =============================================================================
// KEY CONCEPTS:
// =============================================================================
//
// 1. ENVIRONMENT VARIABLES:
//    #[arg(env = "VAR_NAME")] - Read from env var
//    Clap uses this priority: CLI arg > env var > default
//
// 2. CONFIGURATION LAYERING:
//    Common pattern: CLI > Environment > Config File > Defaults
//    Implement custom merging logic for config files
//
// 3. SECRETS HANDLING:
//    #[arg(skip)] - Don't accept from CLI (for security)
//    Read secrets only from environment variables
//    Never log or display secret values
//
// 4. CONFIG FILE LOADING:
//    Use serde for deserialization
//    Handle missing/invalid files gracefully
//    Support multiple formats (TOML, YAML, JSON)
//
// 5. DEFAULT VALUES:
//    default_value = "string" - for String types
//    default_value_t = value - for types with Display
//    Can also use Default trait with serde
//
// 6. SOURCE TRACKING:
//    Track where each config value came from
//    Useful for debugging configuration issues
//
// BEST PRACTICES:
//
// - Never accept secrets via CLI (visible in process list)
// - Use env vars with clear naming conventions (APP_*, MYAPP_*)
// - Provide sensible defaults for development
// - Validate configuration early and fail fast
// - Log configuration sources (but not secret values)
// - Support both --config and CONFIG_FILE env var
// - Use TOML for config files (human-readable, supports comments)
