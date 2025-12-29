# Configuration Layering Patterns

> **Chapter 13** | Part 3: Advanced Architecture | Estimated reading time: 18 minutes

Production CLI applications rarely rely on command-line arguments alone. Users expect to configure behavior through configuration files, environment variables, and sensible defaults. This chapter explores sophisticated configuration layering strategies that integrate seamlessly with Clap while maintaining type safety and clear precedence rules.

## Multi-Source Configuration Hierarchies

### The Standard Precedence Model

```
+===========================================================================+
|                    CONFIGURATION LAYERING ARCHITECTURE                     |
+===========================================================================+

    USER ACTION: myapp --port 9000

         |
         v
    +-----------------------------------------------------------------------+
    |  LAYER 1: CLI ARGUMENTS                    Priority: HIGHEST          |
    |  --port 9000                               (Explicit user intent)     |
    +-----------------------------------------------------------------------+
         |
         | Merge (CLI wins if present)
         v
    +-----------------------------------------------------------------------+
    |  LAYER 2: ENVIRONMENT VARIABLES            Priority: HIGH             |
    |  MYAPP_PORT=8080                           (Deployment config)        |
    |  MYAPP_DEBUG=true                                                     |
    +-----------------------------------------------------------------------+
         |
         | Merge (Env wins if CLI absent)
         v
    +-----------------------------------------------------------------------+
    |  LAYER 3: LOCAL CONFIG FILE                Priority: MEDIUM           |
    |  ./.myapp.toml                             (Project-specific)         |
    |  [server]                                                             |
    |  port = 3000                                                          |
    +-----------------------------------------------------------------------+
         |
         | Merge (Local wins if env absent)
         v
    +-----------------------------------------------------------------------+
    |  LAYER 4: USER CONFIG FILE                 Priority: LOW              |
    |  ~/.config/myapp/config.toml               (Personal preferences)     |
    +-----------------------------------------------------------------------+
         |
         | Merge (User wins if local absent)
         v
    +-----------------------------------------------------------------------+
    |  LAYER 5: SYSTEM CONFIG FILE               Priority: LOWER            |
    |  /etc/myapp/config.toml                    (Organization defaults)    |
    +-----------------------------------------------------------------------+
         |
         | Merge (System wins if user absent)
         v
    +-----------------------------------------------------------------------+
    |  LAYER 6: BUILT-IN DEFAULTS                Priority: LOWEST           |
    |  (Compiled into binary)                    (Fallback values)          |
    |  port = 8080, debug = false                                           |
    +-----------------------------------------------------------------------+
         |
         v
    +-----------------------+
    |   EFFECTIVE CONFIG    |      Result: port = 9000 (from CLI)
    |   (After all merges)  |              debug = true (from env)
    +-----------------------+


    RESOLUTION EXAMPLE:
    ===================

    Source          port    debug   log_level
    ------          ----    -----   ---------
    CLI args        9000    -       -
    Environment     8080    true    -
    Local file      3000    -       debug
    User file       -       false   info
    System file     -       -       warn
    Defaults        8080    false   info

    RESULT:         9000    true    debug
                    ^       ^       ^
                    |       |       |
                    CLI     Env     Local
```

**Diagram Description**: This diagram shows the six-layer configuration precedence model. Each layer can set values that override lower layers. The example shows how different values are resolved from multiple sources, with CLI taking highest priority.

The industry-standard configuration precedence, from highest to lowest priority:

```
1. Command-line arguments    (explicit user intent)
2. Environment variables     (deployment configuration)
3. Local config file         (./.myapp.toml - project-specific)
4. User config file          (~/.config/myapp/config.toml)
5. System config file        (/etc/myapp/config.toml)
6. Built-in defaults         (compiled into binary)
```

This hierarchy respects user intent while enabling organizational defaults. A developer's local override always wins, but absent that, team or system-wide configurations apply.

### Implementing with Figment

Figment is the Rust ecosystem's premier configuration library, designed for layered configuration:

```rust
use clap::Parser;
use figment::{Figment, providers::{Env, Format, Serialized, Toml}};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "myapp", version, about)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, env = "MYAPP_CONFIG")]
    config: Option<PathBuf>,

    /// Log level
    #[arg(short, long, env = "MYAPP_LOG_LEVEL")]
    log_level: Option<String>,

    /// Server port
    #[arg(short, long, env = "MYAPP_PORT")]
    port: Option<u16>,

    /// Enable debug mode
    #[arg(long, env = "MYAPP_DEBUG")]
    debug: bool,

    /// Database connection string
    #[arg(long, env = "MYAPP_DATABASE_URL")]
    database_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct Config {
    log_level: String,
    port: u16,
    debug: bool,
    database_url: String,

    #[serde(default)]
    server: ServerConfig,

    #[serde(default)]
    database: DatabaseConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct ServerConfig {
    host: String,
    workers: usize,
    timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            workers: num_cpus::get(),
            timeout_secs: 30,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct DatabaseConfig {
    pool_size: u32,
    idle_timeout_secs: u64,
}

fn load_config(cli: &Cli) -> Result<Config, figment::Error> {
    // Build layered configuration
    let mut figment = Figment::new()
        // Layer 6: Built-in defaults (lowest priority)
        .merge(Serialized::defaults(Config {
            log_level: "info".to_string(),
            port: 8080,
            debug: false,
            database_url: String::new(),
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
        }));

    // Layer 5: System config (if exists)
    figment = figment.merge(Toml::file("/etc/myapp/config.toml").nested());

    // Layer 4: User config
    if let Some(config_dir) = dirs::config_dir() {
        let user_config = config_dir.join("myapp").join("config.toml");
        figment = figment.merge(Toml::file(user_config).nested());
    }

    // Layer 3: Local config
    figment = figment.merge(Toml::file(".myapp.toml").nested());

    // Layer 2: Environment variables
    figment = figment.merge(Env::prefixed("MYAPP_").split("__"));

    // Layer 1: CLI arguments (highest priority)
    // Only merge non-None values to preserve lower-priority settings
    let cli_overrides = CliOverrides {
        log_level: cli.log_level.clone(),
        port: cli.port,
        debug: if cli.debug { Some(true) } else { None },
        database_url: cli.database_url.clone(),
    };
    figment = figment.merge(Serialized::defaults(cli_overrides));

    // Optional: explicit config file overrides all file-based config
    if let Some(ref config_path) = cli.config {
        figment = figment.merge(Toml::file(config_path).nested());
    }

    figment.extract()
}

/// Wrapper for CLI values that should only override when present
#[derive(Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    log_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    debug: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    database_url: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = load_config(&cli)?;

    println!("Effective configuration:");
    println!("  Log level: {}", config.log_level);
    println!("  Port: {}", config.port);
    println!("  Debug: {}", config.debug);
    println!("  Server workers: {}", config.server.workers);

    Ok(())
}
```

### Architecture Decision: Option Types in CLI Structs

To properly implement layering, CLI argument types must distinguish between "not provided" and "provided with default":

| Approach | Behavior | Use Case |
|----------|----------|----------|
| `port: u16` with `default_value` | Always has value, can't detect absence | Simple tools |
| `port: Option<u16>` | `None` means use config file | Layered config |
| `port: Option<Option<u16>>` | `None`=absent, `Some(None)`=explicit empty | Rare, complex |

For configuration layering, **always use `Option<T>`** for arguments that can come from config files.

## Configuration Merging Strategies

### Deep Merge vs. Replace

Different strategies for combining nested configuration:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AppConfig {
    server: ServerConfig,
    features: HashMap<String, bool>,
    plugins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ServerConfig {
    host: String,
    port: u16,
    tls: Option<TlsConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TlsConfig {
    cert_path: String,
    key_path: String,
    ca_path: Option<String>,
}

/// Merge strategies for configuration values
#[derive(Debug, Clone, Copy)]
enum MergeStrategy {
    /// New value completely replaces old
    Replace,
    /// Collections are concatenated
    Append,
    /// Maps are merged recursively
    DeepMerge,
}

trait Mergeable {
    fn merge(&mut self, other: Self, strategy: MergeStrategy);
}

impl Mergeable for AppConfig {
    fn merge(&mut self, other: Self, strategy: MergeStrategy) {
        // Server config: deep merge
        self.server.merge(other.server, MergeStrategy::DeepMerge);

        // Features map: always deep merge (add new, update existing)
        for (key, value) in other.features {
            self.features.insert(key, value);
        }

        // Plugins list: strategy-dependent
        match strategy {
            MergeStrategy::Replace => self.plugins = other.plugins,
            MergeStrategy::Append => self.plugins.extend(other.plugins),
            MergeStrategy::DeepMerge => {
                // Append but deduplicate
                for plugin in other.plugins {
                    if !self.plugins.contains(&plugin) {
                        self.plugins.push(plugin);
                    }
                }
            }
        }
    }
}

impl Mergeable for ServerConfig {
    fn merge(&mut self, other: Self, _strategy: MergeStrategy) {
        // For structs, overwrite fields if other has non-default values
        if other.host != String::default() {
            self.host = other.host;
        }
        if other.port != 0 {
            self.port = other.port;
        }
        // Optional fields: Some overwrites, None preserves
        if other.tls.is_some() {
            self.tls = other.tls;
        }
    }
}
```

### Custom Figment Provider for Clap

Create a dedicated provider that handles Clap integration cleanly:

```rust
use figment::{Provider, Metadata, Profile, Error};
use figment::value::{Map, Value, Dict};
use clap::Parser;

/// A Figment provider that extracts configuration from Clap
struct ClapProvider<T> {
    cli: T,
    profile: Profile,
}

impl<T: Serialize> ClapProvider<T> {
    fn new(cli: T) -> Self {
        Self {
            cli,
            profile: Profile::Default,
        }
    }

    fn profile(mut self, profile: impl Into<Profile>) -> Self {
        self.profile = profile.into();
        self
    }
}

impl<T: Serialize> Provider for ClapProvider<T> {
    fn metadata(&self) -> Metadata {
        Metadata::named("CLI arguments")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        // Serialize CLI struct to figment Value
        let value = Value::serialize(&self.cli)?;

        // Filter out None values to avoid overriding lower-priority sources
        let filtered = filter_none_values(value);

        let mut map = Map::new();
        if let Value::Dict(_, dict) = filtered {
            map.insert(self.profile.clone(), dict);
        }

        Ok(map)
    }
}

fn filter_none_values(value: Value) -> Value {
    match value {
        Value::Dict(tag, dict) => {
            let filtered: Dict = dict
                .into_iter()
                .filter_map(|(k, v)| {
                    match &v {
                        Value::Empty(_, _) => None,  // Skip None/null values
                        _ => Some((k, filter_none_values(v))),
                    }
                })
                .collect();
            Value::Dict(tag, filtered)
        }
        other => other,
    }
}

// Usage:
fn load_with_clap_provider(cli: Cli) -> Result<Config, figment::Error> {
    Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("MYAPP_"))
        .merge(ClapProvider::new(cli))  // CLI has highest priority
        .extract()
}
```

## Profile-Based Configuration

### Environment Profiles

Support distinct configurations for different environments:

```toml
# config.toml
[default]
log_level = "info"
debug = false

[default.server]
host = "127.0.0.1"
port = 8080
workers = 4

[default.database]
pool_size = 10

# Development overrides
[development]
log_level = "debug"
debug = true

[development.database]
pool_size = 2

# Production overrides
[production]
log_level = "warn"

[production.server]
host = "0.0.0.0"
workers = 16

[production.database]
pool_size = 50
```

```rust
use clap::{Parser, ValueEnum};

#[derive(Parser)]
struct Cli {
    /// Configuration profile to use
    #[arg(long, short = 'P', env = "MYAPP_PROFILE", default_value = "default")]
    profile: ConfigProfile,
}

#[derive(Clone, Debug, ValueEnum, Default)]
enum ConfigProfile {
    #[default]
    Default,
    Development,
    Production,
    Staging,
    Test,
}

impl ConfigProfile {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Development => "development",
            Self::Production => "production",
            Self::Staging => "staging",
            Self::Test => "test",
        }
    }
}

fn load_profiled_config(cli: &Cli) -> Result<Config, figment::Error> {
    let profile_name = cli.profile.as_str();

    Figment::new()
        // Start with default profile
        .merge(Toml::file("config.toml").select("default"))
        // Overlay requested profile (may not exist - that's OK)
        .merge(Toml::file("config.toml").select(profile_name))
        // Environment variables can still override
        .merge(Env::prefixed("MYAPP_"))
        .extract()
}
```

### Profile Inheritance

Implement explicit profile inheritance for complex scenarios:

```rust
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct ProfiledConfig {
    #[serde(default)]
    profiles: HashMap<String, ProfileConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct ProfileConfig {
    /// Parent profile to inherit from
    inherits: Option<String>,

    #[serde(flatten)]
    config: PartialConfig,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
struct PartialConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    log_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    debug: Option<bool>,
}

fn resolve_profile(
    profiles: &HashMap<String, ProfileConfig>,
    name: &str,
    visited: &mut Vec<String>,
) -> Result<PartialConfig, String> {
    // Detect cycles
    if visited.contains(&name.to_string()) {
        return Err(format!("Circular profile inheritance detected: {:?}", visited));
    }
    visited.push(name.to_string());

    let profile = profiles.get(name)
        .ok_or_else(|| format!("Profile '{}' not found", name))?;

    // Start with parent config if inherited
    let mut config = if let Some(ref parent_name) = profile.inherits {
        resolve_profile(profiles, parent_name, visited)?
    } else {
        PartialConfig::default()
    };

    // Override with this profile's values
    if let Some(ref level) = profile.config.log_level {
        config.log_level = Some(level.clone());
    }
    if let Some(port) = profile.config.port {
        config.port = Some(port);
    }
    if let Some(debug) = profile.config.debug {
        config.debug = Some(debug);
    }

    Ok(config)
}
```

## Validation at Boundaries

### Type-Safe Configuration Composition

Validate configuration after all layers are merged:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
enum ConfigError {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    #[error("Invalid value for {field}: {reason}")]
    InvalidValue { field: &'static str, reason: String },

    #[error("Conflicting configuration: {0}")]
    Conflict(String),

    #[error("Configuration parse error: {0}")]
    Parse(#[from] figment::Error),
}

/// Raw configuration as parsed from sources
#[derive(Debug, Deserialize)]
struct RawConfig {
    log_level: Option<String>,
    port: Option<u16>,
    database_url: Option<String>,
    server: Option<RawServerConfig>,
}

#[derive(Debug, Deserialize)]
struct RawServerConfig {
    host: Option<String>,
    workers: Option<usize>,
    timeout_secs: Option<u64>,
}

/// Validated configuration with all fields guaranteed present
#[derive(Debug, Clone)]
pub struct ValidatedConfig {
    pub log_level: LogLevel,
    pub port: u16,
    pub database_url: String,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl RawConfig {
    /// Validate and convert to ValidatedConfig
    pub fn validate(self) -> Result<ValidatedConfig, ConfigError> {
        // Validate log level
        let log_level = match self.log_level.as_deref() {
            Some("trace") => LogLevel::Trace,
            Some("debug") => LogLevel::Debug,
            Some("info") | None => LogLevel::Info,  // default
            Some("warn") => LogLevel::Warn,
            Some("error") => LogLevel::Error,
            Some(other) => return Err(ConfigError::InvalidValue {
                field: "log_level",
                reason: format!("unknown level '{}', expected trace/debug/info/warn/error", other),
            }),
        };

        // Validate port
        let port = self.port.unwrap_or(8080);
        if port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "port",
                reason: "port cannot be 0".to_string(),
            });
        }

        // Validate database URL (required in production)
        let database_url = self.database_url
            .ok_or(ConfigError::MissingField("database_url"))?;
        if database_url.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "database_url",
                reason: "cannot be empty".to_string(),
            });
        }

        // Validate server config with defaults
        let raw_server = self.server.unwrap_or_default();
        let server = ServerConfig {
            host: raw_server.host.unwrap_or_else(|| "127.0.0.1".to_string()),
            workers: raw_server.workers.unwrap_or_else(num_cpus::get),
            timeout_secs: raw_server.timeout_secs.unwrap_or(30),
        };

        if server.workers == 0 {
            return Err(ConfigError::InvalidValue {
                field: "server.workers",
                reason: "must be at least 1".to_string(),
            });
        }

        Ok(ValidatedConfig {
            log_level,
            port,
            database_url,
            server,
        })
    }
}

impl Default for RawServerConfig {
    fn default() -> Self {
        Self {
            host: None,
            workers: None,
            timeout_secs: None,
        }
    }
}
```

### Compile-Time Configuration Validation

Use const generics and associated types for stronger guarantees:

```rust
use std::marker::PhantomData;

/// Marker traits for configuration states
trait ConfigState {}

struct Unchecked;
struct Validated;

impl ConfigState for Unchecked {}
impl ConfigState for Validated {}

/// Configuration wrapper with state tracking
struct AppConfig<S: ConfigState> {
    inner: ConfigData,
    _state: PhantomData<S>,
}

struct ConfigData {
    port: u16,
    workers: usize,
    database_url: String,
}

impl AppConfig<Unchecked> {
    fn new(data: ConfigData) -> Self {
        Self {
            inner: data,
            _state: PhantomData,
        }
    }

    /// Validate and transition to Validated state
    fn validate(self) -> Result<AppConfig<Validated>, ConfigError> {
        // Perform validation
        if self.inner.port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "port",
                reason: "cannot be zero".to_string(),
            });
        }

        Ok(AppConfig {
            inner: self.inner,
            _state: PhantomData,
        })
    }
}

impl AppConfig<Validated> {
    /// Only available on validated configs
    pub fn port(&self) -> u16 {
        self.inner.port
    }

    pub fn workers(&self) -> usize {
        self.inner.workers
    }

    pub fn database_url(&self) -> &str {
        &self.inner.database_url
    }
}

// Functions that require validated config
fn start_server(config: &AppConfig<Validated>) {
    println!("Starting server on port {}", config.port());
}
```

## When NOT To Use Configuration Layering

Complex configuration layering adds cognitive overhead. Avoid it when:

1. **Single-source tools**: If your CLI only needs flags, skip config files
2. **Ephemeral operations**: One-shot commands don't benefit from persistent config
3. **Strict reproducibility**: Multiple config sources can make debugging harder
4. **Simple deployment**: If environment is controlled, env vars may suffice

**Warning signs of over-configuration**:
- More than 5 configuration sources
- Users confused about which config applies
- Debug output needed to show "effective config"
- Config inheritance chains deeper than 2 levels

### Alternative: Single Config File with Overrides

For simpler needs, a single file with CLI overrides may suffice:

```rust
#[derive(Parser)]
struct Cli {
    /// Path to config file (required)
    #[arg(long, default_value = "config.toml")]
    config: PathBuf,

    /// Override any config value (key=value)
    #[arg(long = "set", value_parser = parse_override)]
    overrides: Vec<(String, String)>,
}

fn parse_override(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Format: key=value".to_string());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
```

## Performance Considerations

### Startup Time Impact

| Configuration Strategy | Typical Parse Time | Recommendation |
|------------------------|-------------------|----------------|
| CLI only | ~1ms | Use for simple tools |
| CLI + 1 TOML file | ~3-5ms | Common case |
| Full hierarchy (6 sources) | ~10-20ms | Enterprise tools |
| With validation | +2-5ms | Always worth it |

### Lazy Configuration Loading

For tools where most commands don't need all config:

```rust
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        load_config().expect("Configuration error")
    })
}

// Commands that need config call get_config()
// Commands that don't need config skip the overhead
```

## Summary

Configuration layering transforms simple CLI tools into flexible, deployment-ready applications. The key is balancing flexibility with simplicity.

### Key Takeaways

1. **Standard precedence**: CLI > env > local file > user file > system file > defaults
2. **Use Option<T>** in CLI structs to detect argument absence
3. **Figment** provides battle-tested layering with clean Clap integration
4. **Profile-based config** enables environment-specific settings
5. **Validate after merge** to catch conflicts and missing values
6. **Type-state pattern** ensures only validated configs reach business logic

### Architecture Decisions Documented

| Decision | Recommendation | Rationale |
|----------|----------------|-----------|
| CLI field types | `Option<T>` for layered fields | Detect absence vs. default |
| Config library | Figment | Best Rust layering support |
| Profile handling | Explicit selection via flag | Clear, debuggable |
| Validation | Post-merge, type-state pattern | Strong guarantees |

> **Cross-Reference**: See [Chapter 10](../part2-core-patterns/10-environment-config-integration.md) for environment variable basics, and [Chapter 14](./14-advanced-error-strategies.md) for configuration error handling.

---

*Next: [Advanced Error Strategies](./14-advanced-error-strategies.md)*
