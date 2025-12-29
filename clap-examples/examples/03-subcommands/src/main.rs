//! # Example 03: Subcommands
//!
//! Demonstrates nested subcommand structures similar to git, cargo, or kubectl.
//!
//! ## Run Examples:
//! ```bash
//! # View help
//! cargo run -p subcommands -- --help
//! cargo run -p subcommands -- repo --help
//! cargo run -p subcommands -- repo create --help
//!
//! # Repository commands
//! cargo run -p subcommands -- repo create my-project --public
//! cargo run -p subcommands -- repo clone https://github.com/user/repo
//! cargo run -p subcommands -- repo list --limit 10
//!
//! # Config commands (nested subcommands)
//! cargo run -p subcommands -- config get user.name
//! cargo run -p subcommands -- config set user.email "me@example.com"
//! cargo run -p subcommands -- config list --global
//!
//! # Auth commands
//! cargo run -p subcommands -- auth login --provider github
//! cargo run -p subcommands -- auth logout
//! cargo run -p subcommands -- auth status
//!
//! # Global flags apply to all commands
//! cargo run -p subcommands -- --verbose repo list
//! cargo run -p subcommands -- -q config get key
//! ```

use clap::{Parser, Subcommand, Args};

// =============================================================================
// TOP-LEVEL CLI STRUCTURE
// =============================================================================

/// A Git-like CLI demonstrating subcommand patterns.
///
/// This example shows how to structure a complex CLI with nested subcommands,
/// global flags, and subcommand-specific arguments.
#[derive(Parser, Debug)]
#[command(name = "mycli")]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    // Global flags that apply to all subcommands
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress all output
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Configuration file to use
    #[arg(long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

// =============================================================================
// FIRST-LEVEL SUBCOMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
enum Commands {
    /// Repository management commands
    #[command(subcommand)]
    Repo(RepoCommands),

    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Authentication commands
    #[command(subcommand)]
    Auth(AuthCommands),

    /// Initialize a new project (standalone command)
    Init {
        /// Project name
        name: String,

        /// Template to use
        #[arg(short, long, default_value = "default")]
        template: String,

        /// Skip git initialization
        #[arg(long)]
        no_git: bool,
    },

    /// Show system status (no additional args)
    Status,
}

// =============================================================================
// REPOSITORY SUBCOMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
enum RepoCommands {
    /// Create a new repository
    Create(RepoCreateArgs),

    /// Clone an existing repository
    Clone {
        /// Repository URL to clone
        url: String,

        /// Local directory name (optional)
        #[arg(short, long)]
        directory: Option<String>,

        /// Clone depth (shallow clone)
        #[arg(long)]
        depth: Option<u32>,
    },

    /// List repositories
    List {
        /// Maximum number of results
        #[arg(short, long, default_value_t = 20)]
        limit: u32,

        /// Filter by visibility
        #[arg(long)]
        visibility: Option<Visibility>,

        /// Sort order
        #[arg(long, default_value = "updated")]
        sort: String,
    },

    /// Delete a repository
    Delete {
        /// Repository name
        name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

/// Arguments for creating a repository (using Args derive)
#[derive(Args, Debug)]
struct RepoCreateArgs {
    /// Repository name
    name: String,

    /// Repository description
    #[arg(short, long)]
    description: Option<String>,

    /// Create as public repository
    #[arg(long, conflicts_with = "private")]
    public: bool,

    /// Create as private repository
    #[arg(long, conflicts_with = "public")]
    private: bool,

    /// Initialize with README
    #[arg(long)]
    readme: bool,

    /// License template (e.g., mit, apache-2.0)
    #[arg(long)]
    license: Option<String>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum Visibility {
    Public,
    Private,
    All,
}

// =============================================================================
// CONFIGURATION SUBCOMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Get a configuration value
    Get {
        /// Configuration key (e.g., user.name)
        key: String,

        /// Use global config
        #[arg(long)]
        global: bool,
    },

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,

        /// Set in global config
        #[arg(long)]
        global: bool,
    },

    /// Remove a configuration value
    Unset {
        /// Configuration key
        key: String,

        /// Remove from global config
        #[arg(long)]
        global: bool,
    },

    /// List all configuration values
    List {
        /// Show only global config
        #[arg(long)]
        global: bool,

        /// Show only local config
        #[arg(long)]
        local: bool,
    },

    /// Edit configuration in default editor
    Edit {
        /// Edit global config
        #[arg(long)]
        global: bool,
    },
}

// =============================================================================
// AUTHENTICATION SUBCOMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
enum AuthCommands {
    /// Login to a provider
    Login {
        /// Authentication provider
        #[arg(long, default_value = "github")]
        provider: String,

        /// Use web-based authentication
        #[arg(long)]
        web: bool,

        /// Token for non-interactive login
        #[arg(long)]
        token: Option<String>,
    },

    /// Logout from current session
    Logout {
        /// Provider to logout from (default: all)
        #[arg(long)]
        provider: Option<String>,
    },

    /// Show authentication status
    Status,

    /// Refresh authentication token
    Refresh,
}

// =============================================================================
// COMMAND EXECUTION
// =============================================================================

fn main() {
    let cli = Cli::parse();

    // Handle global flags
    if cli.verbose && !cli.quiet {
        println!("Verbose mode enabled");
    }
    if let Some(config) = &cli.config {
        println!("Using config file: {}", config);
    }

    // Route to appropriate command handler
    match &cli.command {
        Commands::Repo(repo_cmd) => handle_repo(repo_cmd, &cli),
        Commands::Config(config_cmd) => handle_config(config_cmd, &cli),
        Commands::Auth(auth_cmd) => handle_auth(auth_cmd, &cli),
        Commands::Init { name, template, no_git } => {
            handle_init(name, template, *no_git, &cli);
        }
        Commands::Status => handle_status(&cli),
    }
}

fn handle_repo(cmd: &RepoCommands, cli: &Cli) {
    if cli.verbose {
        println!("[repo] Executing repository command...");
    }

    match cmd {
        RepoCommands::Create(args) => {
            println!("Creating repository: {}", args.name);
            if let Some(desc) = &args.description {
                println!("  Description: {}", desc);
            }
            println!("  Visibility: {}", if args.private { "private" } else { "public" });
            if args.readme {
                println!("  Initializing with README");
            }
            if let Some(license) = &args.license {
                println!("  License: {}", license);
            }
        }
        RepoCommands::Clone { url, directory, depth } => {
            println!("Cloning: {}", url);
            if let Some(dir) = directory {
                println!("  Into: {}", dir);
            }
            if let Some(d) = depth {
                println!("  Depth: {}", d);
            }
        }
        RepoCommands::List { limit, visibility, sort } => {
            println!("Listing repositories (limit: {}, sort: {})", limit, sort);
            if let Some(vis) = visibility {
                println!("  Visibility filter: {:?}", vis);
            }
        }
        RepoCommands::Delete { name, force } => {
            if *force {
                println!("Force deleting repository: {}", name);
            } else {
                println!("Deleting repository: {} (would prompt for confirmation)", name);
            }
        }
    }
}

fn handle_config(cmd: &ConfigCommands, cli: &Cli) {
    if cli.verbose {
        println!("[config] Executing configuration command...");
    }

    match cmd {
        ConfigCommands::Get { key, global } => {
            let scope = if *global { "global" } else { "local" };
            println!("Getting {} config: {}", scope, key);
        }
        ConfigCommands::Set { key, value, global } => {
            let scope = if *global { "global" } else { "local" };
            println!("Setting {} config: {} = {}", scope, key, value);
        }
        ConfigCommands::Unset { key, global } => {
            let scope = if *global { "global" } else { "local" };
            println!("Unsetting {} config: {}", scope, key);
        }
        ConfigCommands::List { global, local } => {
            let scope = match (*global, *local) {
                (true, false) => "global",
                (false, true) => "local",
                _ => "all",
            };
            println!("Listing {} configuration", scope);
        }
        ConfigCommands::Edit { global } => {
            let scope = if *global { "global" } else { "local" };
            println!("Opening {} config in editor", scope);
        }
    }
}

fn handle_auth(cmd: &AuthCommands, cli: &Cli) {
    if cli.verbose {
        println!("[auth] Executing authentication command...");
    }

    match cmd {
        AuthCommands::Login { provider, web, token } => {
            println!("Logging in to: {}", provider);
            if *web {
                println!("  Using web-based authentication");
            }
            if token.is_some() {
                println!("  Using provided token");
            }
        }
        AuthCommands::Logout { provider } => {
            if let Some(p) = provider {
                println!("Logging out from: {}", p);
            } else {
                println!("Logging out from all providers");
            }
        }
        AuthCommands::Status => {
            println!("Checking authentication status...");
        }
        AuthCommands::Refresh => {
            println!("Refreshing authentication token...");
        }
    }
}

fn handle_init(name: &str, template: &str, no_git: bool, cli: &Cli) {
    if cli.verbose {
        println!("[init] Initializing new project...");
    }

    println!("Initializing project: {}", name);
    println!("  Template: {}", template);
    if no_git {
        println!("  Skipping git initialization");
    }
}

fn handle_status(cli: &Cli) {
    if cli.verbose {
        println!("[status] Checking system status...");
    }
    println!("System status: OK");
}

// =============================================================================
// KEY CONCEPTS:
// =============================================================================
//
// 1. SUBCOMMAND STRUCTURE:
//    #[command(subcommand)] on a field marks it as containing subcommands.
//    #[derive(Subcommand)] on an enum defines the available subcommands.
//
// 2. NESTED SUBCOMMANDS:
//    Subcommands can themselves have subcommands (e.g., repo create, config set).
//    Use #[command(subcommand)] recursively.
//
// 3. GLOBAL ARGUMENTS:
//    #[arg(global = true)] makes an argument available to all subcommands.
//    Must be defined at the parent level.
//
// 4. ARGS DERIVE:
//    #[derive(Args)] groups arguments into a struct for reuse.
//    Useful when subcommands share common argument patterns.
//
// 5. VALUEENUM:
//    #[derive(ValueEnum)] creates enumerated string arguments.
//    Clap generates valid values list automatically.
//
// 6. COMMAND ROUTING:
//    Use match statements to dispatch to appropriate handlers.
//    Pattern matches enum variants with their captured arguments.
//
// 7. VERSION PROPAGATION:
//    #[command(propagate_version = true)] shows version in subcommand help.
//
// 8. CONFLICTS:
//    #[arg(conflicts_with = "other")] prevents using both options together.
