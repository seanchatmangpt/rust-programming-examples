//! # Example 10: Plugin Architecture
//!
//! Demonstrates a plugin system with command registry, dynamic
//! command loading, and extensible CLI architecture.
//!
//! ## Run Examples:
//! ```bash
//! # View help (shows all registered commands)
//! cargo run -p plugins-architecture -- --help
//!
//! # Built-in commands
//! cargo run -p plugins-architecture -- version
//! cargo run -p plugins-architecture -- help
//!
//! # Core plugin commands
//! cargo run -p plugins-architecture -- file list /tmp
//! cargo run -p plugins-architecture -- file read example.txt
//!
//! # User plugin commands
//! cargo run -p plugins-architecture -- user list
//! cargo run -p plugins-architecture -- user create john
//!
//! # Analytics plugin commands
//! cargo run -p plugins-architecture -- stats show
//! cargo run -p plugins-architecture -- stats export --format json
//!
//! # Plugin management
//! cargo run -p plugins-architecture -- plugin list
//! cargo run -p plugins-architecture -- plugin info file
//!
//! # Verbose mode
//! cargo run -p plugins-architecture -- -v file list /home
//! ```

use clap::{Arg, ArgAction, ArgMatches, Command};
use std::collections::HashMap;

// =============================================================================
// PLUGIN TRAIT
// =============================================================================

/// Trait that all plugins must implement.
trait Plugin: Send + Sync {
    /// Unique name of the plugin.
    fn name(&self) -> &'static str;

    /// Version of the plugin.
    fn version(&self) -> &'static str;

    /// Description of the plugin.
    fn description(&self) -> &'static str;

    /// Build the clap Command for this plugin.
    fn command(&self) -> Command;

    /// Execute the plugin with parsed arguments.
    fn execute(&self, matches: &ArgMatches, ctx: &ExecutionContext) -> Result<(), PluginError>;
}

/// Context passed to plugins during execution.
#[derive(Debug)]
struct ExecutionContext {
    verbose: bool,
    quiet: bool,
    config_dir: Option<String>,
}

/// Plugin execution error.
#[derive(Debug)]
struct PluginError {
    message: String,
    exit_code: i32,
}

impl PluginError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 1,
        }
    }

    fn with_code(message: impl Into<String>, code: i32) -> Self {
        Self {
            message: message.into(),
            exit_code: code,
        }
    }
}

// =============================================================================
// PLUGIN REGISTRY
// =============================================================================

/// Registry that manages all loaded plugins.
struct PluginRegistry {
    plugins: HashMap<&'static str, Box<dyn Plugin>>,
}

impl PluginRegistry {
    fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin.
    fn register(&mut self, plugin: Box<dyn Plugin>) {
        let name = plugin.name();
        self.plugins.insert(name, plugin);
    }

    /// Get a plugin by name.
    fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// Get all registered plugins.
    fn all(&self) -> impl Iterator<Item = &dyn Plugin> {
        self.plugins.values().map(|p| p.as_ref())
    }

    /// Build the complete CLI with all plugins.
    fn build_cli(&self) -> Command {
        let mut cmd = Command::new("pluggable-cli")
            .version("1.0.0")
            .about("A CLI with plugin architecture")
            .arg_required_else_help(true)
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Enable verbose output")
                    .action(ArgAction::SetTrue)
                    .global(true),
            )
            .arg(
                Arg::new("quiet")
                    .short('q')
                    .long("quiet")
                    .help("Suppress non-essential output")
                    .action(ArgAction::SetTrue)
                    .global(true),
            )
            .arg(
                Arg::new("config-dir")
                    .long("config-dir")
                    .help("Configuration directory")
                    .global(true),
            );

        // Add plugin commands
        for plugin in self.all() {
            cmd = cmd.subcommand(plugin.command());
        }

        // Add plugin management subcommand
        cmd = cmd.subcommand(
            Command::new("plugin")
                .about("Plugin management")
                .subcommand(
                    Command::new("list")
                        .about("List all loaded plugins")
                )
                .subcommand(
                    Command::new("info")
                        .about("Show plugin information")
                        .arg(Arg::new("name").required(true).help("Plugin name"))
                )
        );

        cmd
    }
}

// =============================================================================
// CORE PLUGINS
// =============================================================================

/// File operations plugin.
struct FilePlugin;

impl Plugin for FilePlugin {
    fn name(&self) -> &'static str {
        "file"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "File system operations"
    }

    fn command(&self) -> Command {
        Command::new("file")
            .about("File operations")
            .subcommand(
                Command::new("list")
                    .about("List files in directory")
                    .arg(Arg::new("path").required(true).help("Directory path"))
                    .arg(
                        Arg::new("all")
                            .short('a')
                            .long("all")
                            .help("Show hidden files")
                            .action(ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("long")
                            .short('l')
                            .long("long")
                            .help("Long format")
                            .action(ArgAction::SetTrue),
                    ),
            )
            .subcommand(
                Command::new("read")
                    .about("Read file contents")
                    .arg(Arg::new("file").required(true).help("File path"))
                    .arg(
                        Arg::new("lines")
                            .short('n')
                            .long("lines")
                            .help("Number of lines to read")
                            .value_parser(clap::value_parser!(usize)),
                    ),
            )
            .subcommand(
                Command::new("stat")
                    .about("Show file information")
                    .arg(Arg::new("file").required(true).help("File path")),
            )
    }

    fn execute(&self, matches: &ArgMatches, ctx: &ExecutionContext) -> Result<(), PluginError> {
        match matches.subcommand() {
            Some(("list", sub_m)) => {
                let path = sub_m.get_one::<String>("path").unwrap();
                let all = sub_m.get_flag("all");
                let long = sub_m.get_flag("long");

                if ctx.verbose {
                    println!("[file] Listing directory: {}", path);
                }
                println!("Listing {} (all: {}, long: {})", path, all, long);
                Ok(())
            }
            Some(("read", sub_m)) => {
                let file = sub_m.get_one::<String>("file").unwrap();
                let lines = sub_m.get_one::<usize>("lines");

                if ctx.verbose {
                    println!("[file] Reading file: {}", file);
                }
                if let Some(n) = lines {
                    println!("Reading {} lines from {}", n, file);
                } else {
                    println!("Reading entire file: {}", file);
                }
                Ok(())
            }
            Some(("stat", sub_m)) => {
                let file = sub_m.get_one::<String>("file").unwrap();
                println!("File info: {}", file);
                Ok(())
            }
            _ => Err(PluginError::new("Unknown file subcommand")),
        }
    }
}

/// User management plugin.
struct UserPlugin;

impl Plugin for UserPlugin {
    fn name(&self) -> &'static str {
        "user"
    }

    fn version(&self) -> &'static str {
        "1.2.0"
    }

    fn description(&self) -> &'static str {
        "User management operations"
    }

    fn command(&self) -> Command {
        Command::new("user")
            .about("User management")
            .subcommand(
                Command::new("list")
                    .about("List all users")
                    .arg(
                        Arg::new("format")
                            .short('f')
                            .long("format")
                            .help("Output format")
                            .value_parser(["table", "json", "csv"])
                            .default_value("table"),
                    ),
            )
            .subcommand(
                Command::new("create")
                    .about("Create a new user")
                    .arg(Arg::new("username").required(true).help("Username"))
                    .arg(
                        Arg::new("email")
                            .short('e')
                            .long("email")
                            .help("Email address"),
                    )
                    .arg(
                        Arg::new("admin")
                            .long("admin")
                            .help("Create as admin")
                            .action(ArgAction::SetTrue),
                    ),
            )
            .subcommand(
                Command::new("delete")
                    .about("Delete a user")
                    .arg(Arg::new("username").required(true).help("Username"))
                    .arg(
                        Arg::new("force")
                            .short('f')
                            .long("force")
                            .help("Skip confirmation")
                            .action(ArgAction::SetTrue),
                    ),
            )
    }

    fn execute(&self, matches: &ArgMatches, ctx: &ExecutionContext) -> Result<(), PluginError> {
        match matches.subcommand() {
            Some(("list", sub_m)) => {
                let format = sub_m.get_one::<String>("format").unwrap();
                if ctx.verbose {
                    println!("[user] Listing users in {} format", format);
                }
                println!("Listing users (format: {})", format);
                Ok(())
            }
            Some(("create", sub_m)) => {
                let username = sub_m.get_one::<String>("username").unwrap();
                let email = sub_m.get_one::<String>("email");
                let admin = sub_m.get_flag("admin");

                if ctx.verbose {
                    println!("[user] Creating user: {}", username);
                }

                println!("Creating user: {}", username);
                if let Some(e) = email {
                    println!("  Email: {}", e);
                }
                if admin {
                    println!("  Admin: yes");
                }
                Ok(())
            }
            Some(("delete", sub_m)) => {
                let username = sub_m.get_one::<String>("username").unwrap();
                let force = sub_m.get_flag("force");

                if !force {
                    println!("Would delete user: {} (use --force to confirm)", username);
                } else {
                    println!("Deleted user: {}", username);
                }
                Ok(())
            }
            _ => Err(PluginError::new("Unknown user subcommand")),
        }
    }
}

/// Analytics/stats plugin.
struct AnalyticsPlugin;

impl Plugin for AnalyticsPlugin {
    fn name(&self) -> &'static str {
        "stats"
    }

    fn version(&self) -> &'static str {
        "0.9.0"
    }

    fn description(&self) -> &'static str {
        "Analytics and statistics"
    }

    fn command(&self) -> Command {
        Command::new("stats")
            .about("Analytics and statistics")
            .subcommand(
                Command::new("show")
                    .about("Show current statistics")
                    .arg(
                        Arg::new("period")
                            .short('p')
                            .long("period")
                            .help("Time period")
                            .value_parser(["hour", "day", "week", "month"])
                            .default_value("day"),
                    ),
            )
            .subcommand(
                Command::new("export")
                    .about("Export statistics")
                    .arg(
                        Arg::new("format")
                            .short('f')
                            .long("format")
                            .help("Export format")
                            .value_parser(["json", "csv", "xlsx"])
                            .default_value("json"),
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .help("Output file"),
                    ),
            )
            .subcommand(
                Command::new("clear")
                    .about("Clear statistics")
                    .arg(
                        Arg::new("confirm")
                            .long("confirm")
                            .help("Confirm clearing")
                            .action(ArgAction::SetTrue)
                            .required(true),
                    ),
            )
    }

    fn execute(&self, matches: &ArgMatches, ctx: &ExecutionContext) -> Result<(), PluginError> {
        match matches.subcommand() {
            Some(("show", sub_m)) => {
                let period = sub_m.get_one::<String>("period").unwrap();
                if ctx.verbose {
                    println!("[stats] Showing stats for period: {}", period);
                }
                println!("Statistics for {}", period);
                println!("  Active users: 42");
                println!("  Requests: 12,345");
                println!("  Errors: 23");
                Ok(())
            }
            Some(("export", sub_m)) => {
                let format = sub_m.get_one::<String>("format").unwrap();
                let output = sub_m.get_one::<String>("output");

                let filename = output
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("stats.{}", format));

                println!("Exporting stats to {} (format: {})", filename, format);
                Ok(())
            }
            Some(("clear", _)) => {
                println!("Statistics cleared");
                Ok(())
            }
            _ => Err(PluginError::new("Unknown stats subcommand")),
        }
    }
}

// =============================================================================
// MAIN APPLICATION
// =============================================================================

fn main() {
    // Create and populate plugin registry
    let mut registry = PluginRegistry::new();

    // Register core plugins
    registry.register(Box::new(FilePlugin));
    registry.register(Box::new(UserPlugin));
    registry.register(Box::new(AnalyticsPlugin));

    // In a real application, you might also:
    // - Load plugins from a directory
    // - Load plugins from configuration
    // - Support dynamic library plugins (.so/.dll)

    // Build and parse CLI
    let cli = registry.build_cli();
    let matches = cli.get_matches();

    // Create execution context
    let ctx = ExecutionContext {
        verbose: matches.get_flag("verbose"),
        quiet: matches.get_flag("quiet"),
        config_dir: matches.get_one::<String>("config-dir").cloned(),
    };

    if ctx.verbose {
        println!("Verbose mode enabled");
        if let Some(dir) = &ctx.config_dir {
            println!("Config directory: {}", dir);
        }
    }

    // Route to appropriate handler
    let result = match matches.subcommand() {
        Some(("plugin", sub_m)) => handle_plugin_command(sub_m, &registry),
        Some((name, sub_m)) => {
            // Find and execute the matching plugin
            if let Some(plugin) = registry.get(name) {
                plugin.execute(sub_m, &ctx)
            } else {
                Err(PluginError::new(format!("Unknown command: {}", name)))
            }
        }
        None => {
            println!("No command specified. Use --help for usage.");
            Ok(())
        }
    };

    // Handle errors
    if let Err(e) = result {
        eprintln!("Error: {}", e.message);
        std::process::exit(e.exit_code);
    }
}

/// Handle plugin management commands.
fn handle_plugin_command(matches: &ArgMatches, registry: &PluginRegistry) -> Result<(), PluginError> {
    match matches.subcommand() {
        Some(("list", _)) => {
            println!("Loaded plugins:\n");
            for plugin in registry.all() {
                println!("  {} v{}", plugin.name(), plugin.version());
                println!("    {}", plugin.description());
            }
            Ok(())
        }
        Some(("info", sub_m)) => {
            let name = sub_m.get_one::<String>("name").unwrap();
            if let Some(plugin) = registry.get(name) {
                println!("Plugin: {}", plugin.name());
                println!("Version: {}", plugin.version());
                println!("Description: {}", plugin.description());
                println!("\nCommands:");
                // Show command help
                let cmd = plugin.command();
                for subcmd in cmd.get_subcommands() {
                    println!("  {} - {}",
                             subcmd.get_name(),
                             subcmd.get_about().map(|s| s.to_string()).unwrap_or_default());
                }
                Ok(())
            } else {
                Err(PluginError::new(format!("Plugin not found: {}", name)))
            }
        }
        _ => Err(PluginError::new("Unknown plugin subcommand")),
    }
}

// =============================================================================
// KEY CONCEPTS:
// =============================================================================
//
// 1. PLUGIN TRAIT:
//    Define a trait that all plugins must implement.
//    Includes name, version, description, command builder, and executor.
//
// 2. PLUGIN REGISTRY:
//    Central registry that manages all loaded plugins.
//    Provides registration, lookup, and CLI building.
//
// 3. DYNAMIC CLI BUILDING:
//    Build the complete CLI by iterating over registered plugins.
//    Each plugin contributes its own Command structure.
//
// 4. EXECUTION CONTEXT:
//    Pass global state (verbose, config, etc.) to plugins.
//    Allows plugins to access application-wide settings.
//
// 5. COMMAND ROUTING:
//    Route subcommands to the appropriate plugin.
//    Plugin receives its ArgMatches and executes.
//
// EXTENSION POINTS:
//
// For real plugin systems, consider:
// - Loading plugins from shared libraries (.so/.dll)
// - Plugin configuration files
// - Plugin dependencies and load order
// - Plugin lifecycle (init, shutdown)
// - Plugin isolation and sandboxing
// - Plugin updates and versioning
// - Plugin discovery from directories
//
// DESIGN PATTERNS:
//
// - Command Pattern: Each plugin command is a self-contained unit
// - Registry Pattern: Central plugin management
// - Strategy Pattern: Plugins provide different implementations
// - Factory Pattern: Plugins create their own Commands
