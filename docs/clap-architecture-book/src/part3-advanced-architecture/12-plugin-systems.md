# Plugin Systems with Clap

> **Chapter 12** | Part 3: Advanced Architecture | Estimated reading time: 20 minutes

Extensible CLI applications allow users and third parties to add functionality without modifying the core codebase. This chapter explores plugin architectures that integrate with Clap, from simple external command discovery to sophisticated dynamic loading systems.

## Dynamic Subcommand Loading

### The External Command Pattern (Git-style)

The most proven plugin architecture follows Git's model: plugins are separate executables with a naming convention. When a user runs `myapp foo`, and `foo` is not a built-in subcommand, the CLI searches for `myapp-foo` in `PATH` and executes it.

```rust
use clap::{Command, ArgMatches, error::ErrorKind};
use std::process::{Command as ProcessCommand, exit};
use std::ffi::OsString;

fn main() {
    let matches = build_cli().get_matches();

    if let Some((subcommand, sub_matches)) = matches.subcommand() {
        match subcommand {
            "init" => cmd_init(sub_matches),
            "build" => cmd_build(sub_matches),
            _ => {
                // Unknown subcommand - try external
                if !try_external_command(subcommand, sub_matches) {
                    eprintln!("error: unknown command '{}'", subcommand);
                    eprintln!("See 'myapp --help' for available commands");
                    exit(1);
                }
            }
        }
    }
}

fn try_external_command(name: &str, _matches: &ArgMatches) -> bool {
    let external_name = format!("myapp-{}", name);

    // Collect remaining arguments to pass through
    let args: Vec<OsString> = std::env::args_os()
        .skip(1)  // Skip program name
        .skip(1)  // Skip subcommand name
        .collect();

    // Attempt to execute the external command
    match ProcessCommand::new(&external_name).args(&args).status() {
        Ok(status) => {
            exit(status.code().unwrap_or(1));
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            false  // Command not found - not a plugin
        }
        Err(e) => {
            eprintln!("error: failed to execute '{}': {}", external_name, e);
            exit(1);
        }
    }
}

fn build_cli() -> Command {
    Command::new("myapp")
        .version("1.0")
        .about("Extensible application with plugin support")
        .subcommand_required(false)
        .allow_external_subcommands(true)  // Critical for plugin support
        .subcommand(Command::new("init").about("Initialize project"))
        .subcommand(Command::new("build").about("Build the project"))
}
```

### Plugin Discovery and Registration

For a richer experience, discover plugins at startup and register them as proper subcommands with help text:

```rust
use clap::Command;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone)]
struct DiscoveredPlugin {
    name: String,
    path: PathBuf,
    description: Option<String>,
}

fn discover_plugins() -> Vec<DiscoveredPlugin> {
    let mut plugins = Vec::new();

    // Search in PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            plugins.extend(scan_directory_for_plugins(&dir));
        }
    }

    // Search in application-specific plugin directories
    if let Some(home) = dirs::home_dir() {
        let plugin_dir = home.join(".myapp").join("plugins");
        plugins.extend(scan_directory_for_plugins(&plugin_dir));
    }

    // Search in XDG data directories
    if let Some(data_dir) = dirs::data_dir() {
        let plugin_dir = data_dir.join("myapp").join("plugins");
        plugins.extend(scan_directory_for_plugins(&plugin_dir));
    }

    // Deduplicate by name (first found wins)
    let mut seen = std::collections::HashSet::new();
    plugins.retain(|p| seen.insert(p.name.clone()));

    plugins
}

fn scan_directory_for_plugins(dir: &Path) -> Vec<DiscoveredPlugin> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    entries
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?;

            // Match pattern: myapp-<name> or myapp-<name>.exe
            let plugin_name = name
                .strip_prefix("myapp-")?
                .strip_suffix(".exe")
                .or(Some(name.strip_prefix("myapp-")?))
                .map(String::from)?;

            // Verify it's executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let meta = fs::metadata(&path).ok()?;
                if meta.permissions().mode() & 0o111 == 0 {
                    return None;  // Not executable
                }
            }

            // Try to get description from plugin
            let description = get_plugin_description(&path);

            Some(DiscoveredPlugin {
                name: plugin_name,
                path,
                description,
            })
        })
        .collect()
}

fn get_plugin_description(path: &Path) -> Option<String> {
    // Run plugin with --help and parse first line
    let output = std::process::Command::new(path)
        .arg("--myapp-plugin-info")
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.lines().next().map(String::from)
    } else {
        None
    }
}

fn build_cli_with_plugins() -> Command {
    let mut cmd = Command::new("myapp")
        .version("1.0")
        .about("Extensible application")
        .allow_external_subcommands(true);

    // Add built-in commands
    cmd = cmd.subcommand(Command::new("init").about("Initialize project"));
    cmd = cmd.subcommand(Command::new("build").about("Build project"));

    // Discover and register plugins
    for plugin in discover_plugins() {
        let mut subcmd = Command::new(&plugin.name)
            .about(plugin.description.as_deref().unwrap_or("External plugin"));

        // Mark as external for dispatch
        subcmd = subcmd.hide(false);  // Show in help

        cmd = cmd.subcommand(subcmd);
    }

    cmd
}
```

### Architecture Decision: Eager vs. Lazy Plugin Discovery

| Approach | Startup Time | Help Accuracy | Implementation |
|----------|--------------|---------------|----------------|
| **Eager** (discover at startup) | Slower | Complete | Complex |
| **Lazy** (discover on unknown command) | Fast | Incomplete help | Simple |
| **Cached** (discover + cache) | Fast after first run | Complete | Most complex |

For most production systems, cached discovery provides the best balance:

```rust
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, Duration};

#[derive(Serialize, Deserialize)]
struct PluginCache {
    last_updated: SystemTime,
    plugins: Vec<CachedPlugin>,
}

#[derive(Serialize, Deserialize)]
struct CachedPlugin {
    name: String,
    path: PathBuf,
    description: Option<String>,
}

const CACHE_TTL: Duration = Duration::from_secs(3600);  // 1 hour

fn load_plugins_cached() -> Vec<DiscoveredPlugin> {
    let cache_path = dirs::cache_dir()
        .map(|d| d.join("myapp").join("plugins.json"));

    // Try to load from cache
    if let Some(ref path) = cache_path {
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Ok(cache) = serde_json::from_str::<PluginCache>(&contents) {
                if cache.last_updated.elapsed().unwrap_or(CACHE_TTL) < CACHE_TTL {
                    return cache.plugins.into_iter()
                        .map(|p| DiscoveredPlugin {
                            name: p.name,
                            path: p.path,
                            description: p.description,
                        })
                        .collect();
                }
            }
        }
    }

    // Cache miss or expired - do full discovery
    let plugins = discover_plugins();

    // Update cache
    if let Some(path) = cache_path {
        let _ = std::fs::create_dir_all(path.parent().unwrap());
        let cache = PluginCache {
            last_updated: SystemTime::now(),
            plugins: plugins.iter()
                .map(|p| CachedPlugin {
                    name: p.name.clone(),
                    path: p.path.clone(),
                    description: p.description.clone(),
                })
                .collect(),
        };
        let _ = std::fs::write(&path, serde_json::to_string(&cache).unwrap());
    }

    plugins
}
```

## Plugin Trait Design

### The Plugin Interface Contract

For in-process plugins (loaded as dynamic libraries), define a clear trait interface:

```rust
// plugin-api/src/lib.rs - Shared between host and plugins
use std::error::Error;

/// Semantic version for API compatibility checking
pub const API_VERSION: &str = "1.0.0";

/// Plugin metadata returned during registration
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub author: Option<&'static str>,
}

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// The core plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Return plugin metadata
    fn info(&self) -> PluginInfo;

    /// Called when the plugin is loaded
    fn on_load(&mut self) -> PluginResult<()> {
        Ok(())  // Default: no-op
    }

    /// Called when the plugin is about to be unloaded
    fn on_unload(&mut self) -> PluginResult<()> {
        Ok(())  // Default: no-op
    }

    /// Register subcommands with the CLI
    fn register_commands(&self, registry: &mut CommandRegistry) -> PluginResult<()>;

    /// Execute a command registered by this plugin
    fn execute(&self, command: &str, args: &[String]) -> PluginResult<i32>;
}

/// Registry for plugins to register their subcommands
pub struct CommandRegistry {
    commands: Vec<RegisteredCommand>,
}

#[derive(Clone)]
pub struct RegisteredCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub options: Vec<CommandOption>,
}

#[derive(Clone)]
pub struct CommandOption {
    pub short: Option<char>,
    pub long: String,
    pub description: String,
    pub takes_value: bool,
    pub required: bool,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    pub fn register(&mut self, cmd: RegisteredCommand) {
        self.commands.push(cmd);
    }

    pub fn commands(&self) -> &[RegisteredCommand] {
        &self.commands
    }
}

/// Macro for plugins to declare their entry point
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:expr) => {
        #[no_mangle]
        pub extern "C" fn _plugin_api_version() -> &'static str {
            $crate::API_VERSION
        }

        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut dyn $crate::Plugin {
            let plugin = $constructor;
            let boxed: Box<dyn $crate::Plugin> = Box::new(plugin);
            Box::into_raw(boxed)
        }

        #[no_mangle]
        pub extern "C" fn _plugin_destroy(plugin: *mut dyn $crate::Plugin) {
            if !plugin.is_null() {
                unsafe { drop(Box::from_raw(plugin)); }
            }
        }
    };
}
```

### Dynamic Library Loading

Load plugins from shared libraries at runtime:

```rust
use libloading::{Library, Symbol};
use std::path::Path;
use std::collections::HashMap;

pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
}

struct LoadedPlugin {
    _library: Library,  // Keep library alive
    instance: *mut dyn Plugin,
    destroy_fn: Symbol<'static, extern "C" fn(*mut dyn Plugin)>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self { plugins: HashMap::new() }
    }

    pub fn load_plugin(&mut self, path: &Path) -> PluginResult<PluginInfo> {
        // SAFETY: Loading dynamic libraries is inherently unsafe.
        // We verify API version compatibility before trusting the plugin.
        unsafe {
            let library = Library::new(path)
                .map_err(|e| format!("Failed to load plugin: {}", e))?;

            // Check API version
            let version_fn: Symbol<extern "C" fn() -> &'static str> =
                library.get(b"_plugin_api_version")
                    .map_err(|_| "Plugin missing version function")?;

            let plugin_api_version = version_fn();
            if !is_compatible_version(plugin_api_version, API_VERSION) {
                return Err(format!(
                    "Plugin API version {} incompatible with host {}",
                    plugin_api_version, API_VERSION
                ).into());
            }

            // Create plugin instance
            let create_fn: Symbol<extern "C" fn() -> *mut dyn Plugin> =
                library.get(b"_plugin_create")
                    .map_err(|_| "Plugin missing create function")?;

            let destroy_fn: Symbol<extern "C" fn(*mut dyn Plugin)> =
                library.get(b"_plugin_destroy")
                    .map_err(|_| "Plugin missing destroy function")?;

            let instance = create_fn();
            if instance.is_null() {
                return Err("Plugin creation returned null".into());
            }

            // Initialize plugin
            (*instance).on_load()?;

            let info = (*instance).info();
            let name = info.name.to_string();

            // Store in registry
            // Transmute to 'static lifetime - safe because we control the Library lifetime
            let destroy_fn: Symbol<'static, extern "C" fn(*mut dyn Plugin)> =
                std::mem::transmute(destroy_fn);

            self.plugins.insert(name.clone(), LoadedPlugin {
                _library: library,
                instance,
                destroy_fn,
            });

            Ok(info)
        }
    }

    pub fn execute_command(&self, plugin_name: &str, command: &str, args: &[String])
        -> PluginResult<i32>
    {
        let plugin = self.plugins.get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not loaded", plugin_name))?;

        unsafe { (*plugin.instance).execute(command, args) }
    }

    pub fn unload_plugin(&mut self, name: &str) -> PluginResult<()> {
        if let Some(plugin) = self.plugins.remove(name) {
            unsafe {
                (*plugin.instance).on_unload()?;
                (plugin.destroy_fn)(plugin.instance);
            }
        }
        Ok(())
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        for (_, plugin) in self.plugins.drain() {
            unsafe {
                let _ = (*plugin.instance).on_unload();
                (plugin.destroy_fn)(plugin.instance);
            }
        }
    }
}

fn is_compatible_version(plugin: &str, host: &str) -> bool {
    // Semantic versioning: major version must match
    let plugin_major = plugin.split('.').next().unwrap_or("0");
    let host_major = host.split('.').next().unwrap_or("0");
    plugin_major == host_major
}
```

## Command Registry Patterns

### Integrating Plugins with Clap

Bridge plugin-registered commands into Clap's command structure:

```rust
use clap::Command;

fn build_cli_with_dynamic_plugins(manager: &PluginManager) -> Command {
    let mut cmd = Command::new("myapp")
        .version("1.0")
        .about("Application with dynamic plugins");

    // Add built-in commands
    cmd = cmd
        .subcommand(Command::new("plugin")
            .about("Manage plugins")
            .subcommand(Command::new("list").about("List installed plugins"))
            .subcommand(Command::new("install").about("Install a plugin")
                .arg(clap::arg!(<PATH> "Path to plugin")))
            .subcommand(Command::new("uninstall").about("Remove a plugin")
                .arg(clap::arg!(<NAME> "Plugin name"))));

    // Register plugin commands
    for (name, plugin) in &manager.plugins {
        let mut registry = CommandRegistry::new();

        unsafe {
            if let Err(e) = (*plugin.instance).register_commands(&mut registry) {
                eprintln!("Warning: Plugin '{}' failed to register: {}", name, e);
                continue;
            }
        }

        for registered_cmd in registry.commands() {
            let mut subcmd = Command::new(&registered_cmd.name)
                .about(&registered_cmd.description);

            for opt in &registered_cmd.options {
                let mut arg = clap::Arg::new(&opt.long)
                    .long(&opt.long)
                    .help(&opt.description);

                if let Some(short) = opt.short {
                    arg = arg.short(short);
                }

                if opt.takes_value {
                    arg = arg.num_args(1);
                }

                if opt.required {
                    arg = arg.required(true);
                }

                subcmd = subcmd.arg(arg);
            }

            cmd = cmd.subcommand(subcmd);
        }
    }

    cmd
}
```

### Versioning Plugin APIs

Maintain backward compatibility with semantic versioning:

```rust
/// Version information for compatibility checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ApiVersion {
    pub const CURRENT: Self = Self { major: 1, minor: 2, patch: 0 };

    /// Check if this version is compatible with another
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // Major version must match (breaking changes)
        // Minor version of host must be >= plugin (features)
        self.major == other.major && self.minor >= other.minor
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(Self {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }
}
```

## When NOT To Use Plugin Systems

Plugin architectures add significant complexity. Avoid them when:

1. **Functionality is predictable**: If you know all commands upfront, use static subcommands
2. **Security is paramount**: Dynamic code loading increases attack surface
3. **Single-user tools**: Plugins shine for shared/enterprise tools, not personal utilities
4. **Startup time is critical**: Plugin discovery adds latency
5. **ABI stability is difficult**: Rust has no stable ABI; dynamic linking is fragile

**Warning signs you may be over-engineering**:
- Fewer than 3 potential plugins in the roadmap
- All plugins would be developed by the same team
- Plugin API changes with every release

### Alternative: Configuration-Driven Extensibility

Sometimes configuration achieves extensibility goals without plugin complexity:

```rust
use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Execute a custom command defined in config
    Run {
        /// Name of the custom command
        name: String,
        /// Arguments to pass
        args: Vec<String>,
    },
    // ... other built-in commands
}

#[derive(Deserialize)]
struct Config {
    /// User-defined command aliases
    commands: HashMap<String, CustomCommand>,
}

#[derive(Deserialize)]
struct CustomCommand {
    /// Shell command to execute
    run: String,
    /// Description for help text
    description: Option<String>,
    /// Working directory
    cwd: Option<String>,
}

fn execute_custom_command(name: &str, args: &[String], config: &Config)
    -> anyhow::Result<i32>
{
    let cmd = config.commands.get(name)
        .ok_or_else(|| anyhow::anyhow!("Unknown command: {}", name))?;

    let mut process = std::process::Command::new("sh");
    process.arg("-c").arg(&cmd.run);
    process.args(args);

    if let Some(ref cwd) = cmd.cwd {
        process.current_dir(cwd);
    }

    let status = process.status()?;
    Ok(status.code().unwrap_or(1))
}
```

## Performance Considerations

### Startup Time Impact

| Plugin Strategy | Cold Start | Warm Start | Memory Overhead |
|-----------------|------------|------------|-----------------|
| No plugins | ~5ms | ~5ms | Baseline |
| External commands (no discovery) | ~5ms | ~5ms | None |
| External commands (eager discovery) | ~50-200ms | ~5ms with cache | Minimal |
| Dynamic libraries (lazy) | ~10ms | ~10ms | Per-plugin |
| Dynamic libraries (preloaded) | ~50-100ms | ~20ms | Significant |

### Lazy Loading Strategy

Only load plugins when their commands are invoked:

```rust
use std::sync::OnceLock;

struct LazyPlugin {
    path: PathBuf,
    info: PluginInfo,
    instance: OnceLock<Box<dyn Plugin>>,
}

impl LazyPlugin {
    fn get_or_load(&self) -> PluginResult<&dyn Plugin> {
        self.instance.get_or_try_init(|| {
            // Perform actual loading only when needed
            load_plugin_from_path(&self.path)
        }).map(|p| p.as_ref())
    }
}
```

## Summary

Plugin systems transform static CLI tools into extensible platforms. The right architecture depends on your extensibility requirements, security constraints, and performance budget.

### Key Takeaways

1. **External command pattern** (Git-style) provides simple, battle-tested extensibility with minimal code
2. **Dynamic library loading** enables rich plugin APIs but requires careful ABI management
3. **Plugin discovery** can be eager, lazy, or cached - choose based on startup time requirements
4. **Version your API** from day one using semantic versioning for compatibility
5. **Consider alternatives**: Configuration-driven extensibility may be simpler than true plugins
6. **Security implications** of loading external code are significant; sandbox when possible

### Architecture Decisions Documented

| Decision | Recommendation | Rationale |
|----------|----------------|-----------|
| Plugin type | External commands for most cases | Simplest, most portable, secure |
| Discovery | Cached with TTL | Balance startup time and accuracy |
| API versioning | Semantic versioning, major must match | Predictable compatibility |
| Loading | Lazy by default | Minimize startup impact |

> **Cross-Reference**: See [Chapter 11](./11-multi-binary-architecture.md) for organizing plugin codebases as workspaces, and [Chapter 19](../part4-real-world-systems/19-performance-optimization.md) for measuring plugin startup impact.

---

*Next: [Configuration Layering Patterns](./13-configuration-layering.md)*
