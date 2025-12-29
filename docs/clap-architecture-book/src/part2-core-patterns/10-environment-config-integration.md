# Environment and Config Integration

> **Chapter 10** | Part 2: Core Patterns | Estimated reading time: 14 minutes

Modern CLIs must integrate seamlessly with environment variables, configuration files, and multiple deployment contexts. This chapter covers patterns for building CLIs that work naturally in containers, CI/CD pipelines, and development environments while maintaining clear precedence rules and excellent user experience.

## Configuration Source Architecture

A well-designed CLI accepts configuration from multiple sources with predictable precedence:

```
┌──────────────────────────────────────────────────────────────────┐
│                 Configuration Precedence Hierarchy               │
├──────────────────────────────────────────────────────────────────┤
│  Priority │  Source                │  Use Case                   │
├──────────────────────────────────────────────────────────────────┤
│  1 (High) │  Command-line args     │  One-time overrides        │
│  2        │  Environment variables │  Container/CI secrets      │
│  3        │  Local config file     │  Project-specific settings │
│  4        │  User config file      │  Personal preferences      │
│  5 (Low)  │  Default values        │  Sensible fallbacks        │
└──────────────────────────────────────────────────────────────────┘
```

## Environment Variable Binding

### Basic Environment Variable Support

Clap provides built-in environment variable support:

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "myapp", about = "Production-ready CLI application")]
struct Cli {
    /// API endpoint URL
    #[arg(long, env = "MYAPP_API_URL", default_value = "https://api.example.com")]
    api_url: String,

    /// Authentication token
    #[arg(long, env = "MYAPP_TOKEN")]
    token: Option<String>,

    /// Database connection string
    #[arg(long, env = "MYAPP_DATABASE_URL")]
    database_url: Option<String>,

    /// Log level
    #[arg(
        long,
        env = "MYAPP_LOG_LEVEL",
        default_value = "info",
        value_parser = ["trace", "debug", "info", "warn", "error"]
    )]
    log_level: String,

    /// Enable debug mode
    #[arg(long, env = "MYAPP_DEBUG")]
    debug: bool,

    /// Worker thread count
    #[arg(long, env = "MYAPP_WORKERS", default_value_t = 4)]
    workers: usize,

    /// Configuration file path
    #[arg(short, long, env = "MYAPP_CONFIG")]
    config: Option<PathBuf>,
}
```

### Secure Environment Variable Handling

Hide sensitive values from help and error output:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "secure-app")]
struct Cli {
    /// API authentication token (hidden from help output)
    #[arg(
        long,
        env = "SECURE_APP_TOKEN",
        hide_env_values = true,  // Don't show env value in help
        hide = false             // Still show the argument exists
    )]
    token: Option<String>,

    /// Database password
    #[arg(
        long,
        env = "SECURE_APP_DB_PASSWORD",
        hide_env_values = true
    )]
    db_password: Option<String>,

    /// Encryption key
    #[arg(
        long,
        env = "SECURE_APP_ENCRYPTION_KEY",
        hide_env_values = true,
        value_name = "KEY"
    )]
    encryption_key: Option<String>,

    /// Public configuration (safe to display)
    #[arg(long, env = "SECURE_APP_REGION", default_value = "us-east-1")]
    region: String,
}

impl Cli {
    /// Validate that required secrets are present
    pub fn validate_secrets(&self) -> Result<(), String> {
        if self.token.is_none() {
            return Err(
                "Authentication token required. Set SECURE_APP_TOKEN or use --token".to_string()
            );
        }
        Ok(())
    }
}
```

### Environment Variable Naming Conventions

Follow consistent naming patterns for discoverability:

```rust
use clap::Parser;

/// Application with hierarchical environment variable naming
#[derive(Parser, Debug)]
#[command(name = "cloudctl")]
struct Cli {
    // Global settings: CLOUDCTL_*
    #[arg(long, env = "CLOUDCTL_PROFILE", default_value = "default")]
    profile: String,

    #[arg(long, env = "CLOUDCTL_REGION")]
    region: Option<String>,

    #[arg(long, env = "CLOUDCTL_OUTPUT", default_value = "table")]
    output: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Manage compute resources
    Compute(ComputeArgs),
    /// Manage storage resources
    Storage(StorageArgs),
}

#[derive(clap::Args, Debug)]
struct ComputeArgs {
    // Subcommand-specific: CLOUDCTL_COMPUTE_*
    #[arg(long, env = "CLOUDCTL_COMPUTE_DEFAULT_TYPE", default_value = "t3.micro")]
    instance_type: String,

    #[arg(long, env = "CLOUDCTL_COMPUTE_SSH_KEY")]
    ssh_key: Option<String>,

    #[command(subcommand)]
    action: ComputeAction,
}

#[derive(clap::Subcommand, Debug)]
enum ComputeAction {
    List,
    Create { name: String },
}

#[derive(clap::Args, Debug)]
struct StorageArgs {
    // Subcommand-specific: CLOUDCTL_STORAGE_*
    #[arg(long, env = "CLOUDCTL_STORAGE_DEFAULT_CLASS", default_value = "standard")]
    storage_class: String,

    #[command(subcommand)]
    action: StorageAction,
}

#[derive(clap::Subcommand, Debug)]
enum StorageAction {
    List,
    Create { name: String },
}
```

## Configuration File Integration

### Using the config Crate

Integrate with Rust's popular `config` crate for file-based configuration:

```rust
use clap::Parser;
use config::{Config, ConfigError, File, Environment};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "server")]
struct CliArgs {
    /// Configuration file path
    #[arg(short, long, env = "SERVER_CONFIG")]
    config: Option<PathBuf>,

    /// Override: server port
    #[arg(short, long, env = "SERVER_PORT")]
    port: Option<u16>,

    /// Override: log level
    #[arg(long, env = "SERVER_LOG_LEVEL")]
    log_level: Option<String>,

    /// Override: enable TLS
    #[arg(long, env = "SERVER_TLS")]
    tls: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_host")]
    host: String,
    #[serde(default = "default_log_level")]
    log_level: String,
    #[serde(default)]
    tls: TlsConfig,
    #[serde(default)]
    database: DatabaseConfig,
}

#[derive(Debug, Deserialize, Default)]
struct TlsConfig {
    enabled: bool,
    cert_path: Option<String>,
    key_path: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct DatabaseConfig {
    url: Option<String>,
    pool_size: Option<u32>,
}

fn default_port() -> u16 { 8080 }
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_log_level() -> String { "info".to_string() }

impl ServerConfig {
    /// Load configuration with proper precedence
    pub fn load(cli: &CliArgs) -> Result<Self, ConfigError> {
        let mut builder = Config::builder();

        // Layer 1: Default configuration file locations
        builder = builder
            .add_source(File::with_name("/etc/server/config").required(false))
            .add_source(File::with_name("~/.config/server/config").required(false))
            .add_source(File::with_name("./config").required(false));

        // Layer 2: Explicit config file from CLI
        if let Some(ref path) = cli.config {
            builder = builder.add_source(
                File::from(path.clone()).required(true)
            );
        }

        // Layer 3: Environment variables (prefix SERVER_)
        builder = builder.add_source(
            Environment::with_prefix("SERVER")
                .separator("_")
                .try_parsing(true)
        );

        // Build base config
        let mut config: ServerConfig = builder.build()?.try_deserialize()?;

        // Layer 4: CLI overrides (highest priority)
        if let Some(port) = cli.port {
            config.port = port;
        }
        if let Some(ref log_level) = cli.log_level {
            config.log_level = log_level.clone();
        }
        if let Some(tls) = cli.tls {
            config.tls.enabled = tls;
        }

        Ok(config)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CliArgs::parse();
    let config = ServerConfig::load(&cli)?;

    println!("Server configuration:");
    println!("  Host: {}:{}", config.host, config.port);
    println!("  Log level: {}", config.log_level);
    println!("  TLS enabled: {}", config.tls.enabled);

    Ok(())
}
```

### Using Figment for Advanced Layering

The `figment` crate provides sophisticated configuration merging:

```rust
use clap::Parser;
use figment::{Figment, providers::{Env, Format, Toml, Serialized}};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug, Serialize)]
#[command(name = "pipeline")]
struct CliArgs {
    /// Config file path
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<PathBuf>,

    /// Pipeline name
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// Parallelism level
    #[arg(short = 'j', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    parallelism: Option<usize>,

    /// Dry run mode
    #[arg(long)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    dry_run: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct PipelineConfig {
    name: String,
    parallelism: usize,
    dry_run: bool,
    stages: Vec<StageConfig>,
    notifications: NotificationConfig,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct StageConfig {
    name: String,
    command: String,
    timeout_seconds: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct NotificationConfig {
    slack_webhook: Option<String>,
    email: Option<String>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            parallelism: 4,
            dry_run: false,
            stages: Vec::new(),
            notifications: NotificationConfig::default(),
        }
    }
}

impl PipelineConfig {
    pub fn load(cli: CliArgs) -> Result<Self, figment::Error> {
        let mut figment = Figment::new()
            // Base defaults
            .merge(Serialized::defaults(PipelineConfig::default()))
            // System-wide config
            .merge(Toml::file("/etc/pipeline/config.toml").nested())
            // User config
            .merge(Toml::file("~/.config/pipeline/config.toml").nested())
            // Project config
            .merge(Toml::file("pipeline.toml").nested());

        // Explicit config file
        if let Some(ref path) = cli.config {
            figment = figment.merge(Toml::file(path).nested());
        }

        // Environment variables
        figment = figment.merge(
            Env::prefixed("PIPELINE_")
                .split("__")
                .map(|key| key.as_str().replace("_", ".").into())
        );

        // CLI arguments (highest priority)
        figment = figment.merge(Serialized::defaults(cli));

        figment.extract()
    }
}
```

## Multi-Format Configuration Support

### TOML, YAML, and JSON Support

Support multiple configuration file formats:

```rust
use serde::Deserialize;
use std::path::Path;
use std::fs;

#[derive(Debug, Deserialize)]
struct AppConfig {
    server: ServerSettings,
    database: DatabaseSettings,
    logging: LoggingSettings,
}

#[derive(Debug, Deserialize)]
struct ServerSettings {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct DatabaseSettings {
    url: String,
    pool_size: u32,
}

#[derive(Debug, Deserialize)]
struct LoggingSettings {
    level: String,
    format: String,
}

#[derive(Debug)]
enum ConfigFormat {
    Toml,
    Yaml,
    Json,
}

impl ConfigFormat {
    fn detect(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "toml" => Some(ConfigFormat::Toml),
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "json" => Some(ConfigFormat::Json),
            _ => None,
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;

        let format = ConfigFormat::detect(path)
            .ok_or_else(|| format!("Unknown config format: {:?}", path.extension()))?;

        let config = match format {
            ConfigFormat::Toml => toml::from_str(&content)?,
            ConfigFormat::Yaml => serde_yaml::from_str(&content)?,
            ConfigFormat::Json => serde_json::from_str(&content)?,
        };

        Ok(config)
    }

    /// Find and load config from standard locations
    pub fn discover() -> Result<Self, Box<dyn std::error::Error>> {
        let candidates = [
            "config.toml",
            "config.yaml",
            "config.yml",
            "config.json",
            ".config/app/config.toml",
        ];

        for candidate in &candidates {
            let path = Path::new(candidate);
            if path.exists() {
                return Self::load(path);
            }
        }

        Err("No configuration file found".into())
    }
}
```

## Profile Management

### Environment-Based Profiles

Support development, staging, and production configurations:

```rust
use clap::{Parser, ValueEnum};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "app")]
struct Cli {
    /// Environment profile
    #[arg(
        short,
        long,
        env = "APP_PROFILE",
        value_enum,
        default_value = "development"
    )]
    profile: Profile,

    /// Configuration directory
    #[arg(long, env = "APP_CONFIG_DIR", default_value = "./config")]
    config_dir: PathBuf,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum Profile {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    api_url: String,
    debug: bool,
    log_level: String,
    database_url: String,
    cache_ttl: u64,
}

impl AppConfig {
    pub fn load_for_profile(
        config_dir: &PathBuf,
        profile: &Profile,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Load base configuration
        let base_path = config_dir.join("base.toml");
        let base_content = std::fs::read_to_string(&base_path)?;
        let mut config: AppConfig = toml::from_str(&base_content)?;

        // Load profile-specific overrides
        let profile_name = match profile {
            Profile::Development => "development",
            Profile::Staging => "staging",
            Profile::Production => "production",
        };

        let profile_path = config_dir.join(format!("{}.toml", profile_name));
        if profile_path.exists() {
            let profile_content = std::fs::read_to_string(&profile_path)?;
            let profile_config: PartialConfig = toml::from_str(&profile_content)?;

            // Apply overrides
            if let Some(url) = profile_config.api_url {
                config.api_url = url;
            }
            if let Some(debug) = profile_config.debug {
                config.debug = debug;
            }
            if let Some(level) = profile_config.log_level {
                config.log_level = level;
            }
            if let Some(url) = profile_config.database_url {
                config.database_url = url;
            }
            if let Some(ttl) = profile_config.cache_ttl {
                config.cache_ttl = ttl;
            }
        }

        // Load local overrides (not committed to version control)
        let local_path = config_dir.join("local.toml");
        if local_path.exists() {
            let local_content = std::fs::read_to_string(&local_path)?;
            let local_config: PartialConfig = toml::from_str(&local_content)?;
            // Apply local overrides...
        }

        Ok(config)
    }
}

#[derive(Debug, Deserialize, Default)]
struct PartialConfig {
    api_url: Option<String>,
    debug: Option<bool>,
    log_level: Option<String>,
    database_url: Option<String>,
    cache_ttl: Option<u64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = AppConfig::load_for_profile(&cli.config_dir, &cli.profile)?;

    println!("Loaded configuration for {:?}:", cli.profile);
    println!("  API URL: {}", config.api_url);
    println!("  Debug: {}", config.debug);
    println!("  Log Level: {}", config.log_level);

    Ok(())
}
```

## Config File Discovery Patterns

### XDG Base Directory Compliance

Follow platform conventions for config file locations:

```rust
use std::path::PathBuf;

/// Configuration file discovery following XDG and platform conventions
pub struct ConfigDiscovery {
    app_name: String,
}

impl ConfigDiscovery {
    pub fn new(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
        }
    }

    /// Get user config directory
    pub fn user_config_dir(&self) -> Option<PathBuf> {
        #[cfg(target_os = "linux")]
        {
            std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .ok()
                .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
                .map(|p| p.join(&self.app_name))
        }

        #[cfg(target_os = "macos")]
        {
            dirs::home_dir().map(|h| {
                h.join("Library")
                    .join("Application Support")
                    .join(&self.app_name)
            })
        }

        #[cfg(target_os = "windows")]
        {
            std::env::var("APPDATA")
                .map(PathBuf::from)
                .ok()
                .map(|p| p.join(&self.app_name))
        }
    }

    /// Get system config directory
    pub fn system_config_dir(&self) -> PathBuf {
        #[cfg(target_family = "unix")]
        {
            PathBuf::from("/etc").join(&self.app_name)
        }

        #[cfg(target_os = "windows")]
        {
            std::env::var("ProgramData")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData"))
                .join(&self.app_name)
        }
    }

    /// Search for config file in standard locations
    pub fn find_config(&self, filename: &str) -> Option<PathBuf> {
        let candidates = vec![
            // Current directory
            PathBuf::from(filename),
            // Project-local
            PathBuf::from(format!(".{}", self.app_name)).join(filename),
            // User config
            self.user_config_dir().map(|d| d.join(filename)),
            // System config
            Some(self.system_config_dir().join(filename)),
        ];

        candidates
            .into_iter()
            .flatten()
            .find(|p| p.exists())
    }

    /// Get all config locations for documentation
    pub fn all_locations(&self) -> Vec<String> {
        let mut locations = vec![
            format!("./{}", "config.toml"),
            format!(".{}/config.toml", self.app_name),
        ];

        if let Some(user_dir) = self.user_config_dir() {
            locations.push(format!("{}/config.toml", user_dir.display()));
        }

        locations.push(format!("{}/config.toml", self.system_config_dir().display()));

        locations
    }
}
```

## Common Pitfalls

1. **Unclear precedence**: Document which source wins when values conflict. Users get frustrated when environment variables seem ignored.

2. **Missing environment variable documentation**: Always document available environment variables in `--help` or README.

3. **Secrets in config files**: Never commit secrets. Use environment variables or secret management for credentials.

4. **Platform-specific paths**: Use cross-platform path handling (`dirs` crate) instead of hardcoded paths.

5. **No config validation**: Validate configuration early. A typo in a config file shouldn't crash the application mid-run.

## Pro Tips

- **Use `env!()` for compile-time defaults**: `env!("CARGO_PKG_VERSION")` embeds version at build time
- **Provide `--config-dump`**: Let users see the final merged configuration for debugging
- **Support `--config-check`**: Validate configuration without running the application
- **Implement config file generation**: `myapp init` creates a template config file
- **Log configuration sources**: In debug mode, show where each value came from
- **Consider `dotenvy`**: Load `.env` files automatically during development

## Summary

Robust configuration integration makes CLIs deployable across environments:

1. **Environment variables** enable container and CI/CD integration
2. **Configuration files** support complex, versioned settings
3. **Clear precedence** prevents user confusion
4. **Profile management** simplifies multi-environment deployments
5. **Platform-aware discovery** follows OS conventions

Master these patterns to build CLIs that adapt seamlessly from development laptops to production Kubernetes clusters.

---

*Next: [Multi-Binary Architecture](../part3-advanced-architecture/11-multi-binary-architecture.md)*
