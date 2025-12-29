//! # Example 07: Derive Macros - Complete Reference
//!
//! Comprehensive showcase of all major derive macro attributes.
//!
//! ## Run Examples:
//! ```bash
//! # View all help options
//! cargo run -p derive-macros -- --help
//! cargo run -p derive-macros -- user --help
//! cargo run -p derive-macros -- user create --help
//!
//! # User commands
//! cargo run -p derive-macros -- user create "john_doe" --email "john@example.com" --role admin
//! cargo run -p derive-macros -- user list --limit 50 --sort-by name
//! cargo run -p derive-macros -- user delete john_doe --force
//!
//! # Project commands
//! cargo run -p derive-macros -- project create my-project --visibility public
//! cargo run -p derive-macros -- project build my-project --release --target linux
//!
//! # System commands
//! cargo run -p derive-macros -- system status
//! cargo run -p derive-macros -- system config set theme dark
//!
//! # Global options
//! cargo run -p derive-macros -- --verbose --no-color user list
//! ```

use clap::{Parser, Subcommand, Args, ValueEnum, CommandFactory};
use std::path::PathBuf;

// =============================================================================
// MAIN CLI STRUCT - #[derive(Parser)]
// =============================================================================

/// A comprehensive CLI demonstrating all derive macro features.
///
/// This long description appears when using `--help`. It supports
/// multiple paragraphs and Markdown-like formatting.
///
/// ## Features
/// - User management
/// - Project management
/// - System configuration
#[derive(Parser, Debug)]
#[command(
    name = "derive-demo",
    author = "Clap Architecture Book <book@example.com>",
    version = "2.0.0",
    about = "Comprehensive derive macro demonstration",
    long_about = None,  // Use doc comment above
    // Styling and behavior
    propagate_version = true,
    subcommand_required = true,
    arg_required_else_help = true,
    // Help customization
    help_template = "{before-help}{name} {version}\n{author-with-newline}{about-with-newline}\n{usage-heading} {usage}\n\n{all-args}{after-help}",
    after_help = "For more information, see: https://example.com/docs",
    after_long_help = "Full documentation available at: https://example.com/docs\n\nReport bugs to: bugs@example.com"
)]
struct Cli {
    // Global arguments (available to all subcommands)
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Configuration file path
    #[arg(long, env = "APP_CONFIG", global = true)]
    config: Option<PathBuf>,

    /// Output format for all commands
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, global = true)]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

// =============================================================================
// TOP-LEVEL SUBCOMMANDS - #[derive(Subcommand)]
// =============================================================================

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage users
    #[command(subcommand)]
    User(UserCommands),

    /// Manage projects
    #[command(subcommand)]
    Project(ProjectCommands),

    /// System administration
    #[command(subcommand)]
    System(SystemCommands),

    /// Show version information
    #[command(name = "version", alias = "ver")]
    Version,

    /// Generate shell completions
    #[command(hide = true)]  // Hidden from help
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

// =============================================================================
// USER SUBCOMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
enum UserCommands {
    /// Create a new user
    Create(UserCreateArgs),

    /// List all users
    List(UserListArgs),

    /// Get user details
    Get {
        /// Username to look up
        username: String,

        /// Include activity history
        #[arg(long)]
        with_activity: bool,
    },

    /// Update user information
    Update {
        /// Username to update
        username: String,

        #[command(flatten)]
        updates: UserUpdateArgs,
    },

    /// Delete a user
    Delete {
        /// Username to delete
        username: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

/// Arguments for creating a user - demonstrates #[derive(Args)]
#[derive(Args, Debug)]
struct UserCreateArgs {
    /// Username (alphanumeric, 3-20 characters)
    #[arg(value_parser = validate_username)]
    username: String,

    /// User's email address
    #[arg(short, long)]
    email: String,

    /// User's role
    #[arg(short, long, value_enum, default_value_t = Role::User)]
    role: Role,

    /// User's display name
    #[arg(long)]
    display_name: Option<String>,

    /// Account is active
    #[arg(long, default_value_t = true)]
    active: bool,

    /// Additional tags for the user
    #[arg(long, value_delimiter = ',')]
    tags: Vec<String>,
}

/// Arguments for listing users
#[derive(Args, Debug)]
struct UserListArgs {
    /// Maximum number of results
    #[arg(short, long, default_value_t = 20, value_parser = clap::value_parser!(u32).range(1..=100))]
    limit: u32,

    /// Number of results to skip
    #[arg(long, default_value_t = 0)]
    offset: u32,

    /// Field to sort by
    #[arg(long, value_enum, default_value_t = SortField::CreatedAt)]
    sort_by: SortField,

    /// Sort in descending order
    #[arg(long)]
    desc: bool,

    /// Filter by role
    #[arg(long, value_enum)]
    role: Option<Role>,

    /// Filter by active status
    #[arg(long)]
    active_only: bool,

    /// Search query
    #[arg(short = 's', long)]
    search: Option<String>,
}

/// Arguments for updating a user (all optional)
#[derive(Args, Debug)]
struct UserUpdateArgs {
    /// New email address
    #[arg(long)]
    email: Option<String>,

    /// New role
    #[arg(long, value_enum)]
    role: Option<Role>,

    /// New display name
    #[arg(long)]
    display_name: Option<String>,

    /// Set active status
    #[arg(long)]
    active: Option<bool>,
}

// =============================================================================
// PROJECT SUBCOMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
enum ProjectCommands {
    /// Create a new project
    Create {
        /// Project name
        name: String,

        /// Project visibility
        #[arg(long, value_enum, default_value_t = Visibility::Private)]
        visibility: Visibility,

        /// Project description
        #[arg(short, long)]
        description: Option<String>,

        /// Initialize with template
        #[arg(long)]
        template: Option<String>,
    },

    /// Build a project
    Build {
        /// Project name
        name: String,

        /// Build in release mode
        #[arg(short, long)]
        release: bool,

        /// Target platform
        #[arg(long, value_enum)]
        target: Option<Platform>,

        /// Build features
        #[arg(long, value_delimiter = ',')]
        features: Vec<String>,

        /// Extra build flags
        #[arg(last = true)]
        extra_args: Vec<String>,
    },

    /// Run a project
    Run {
        /// Project name
        name: String,

        /// Arguments to pass to the program
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// List all projects
    List(ProjectListArgs),
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Filtering Options")]
struct ProjectListArgs {
    /// Filter by visibility
    #[arg(long, value_enum)]
    visibility: Option<Visibility>,

    /// Filter by owner
    #[arg(long)]
    owner: Option<String>,

    /// Include archived projects
    #[arg(long)]
    include_archived: bool,
}

// =============================================================================
// SYSTEM SUBCOMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
enum SystemCommands {
    /// Show system status
    Status,

    /// System configuration
    #[command(subcommand)]
    Config(ConfigCommands),

    /// View system logs
    Logs {
        /// Number of lines to show
        #[arg(short = 'n', long, default_value_t = 50)]
        lines: u32,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Filter by level
        #[arg(long, value_enum)]
        level: Option<LogLevel>,
    },

    /// Run diagnostics
    Diagnose {
        /// Components to check
        #[arg(value_enum)]
        components: Vec<Component>,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },

    /// List all configuration
    List,

    /// Reset to defaults
    Reset {
        /// Reset specific key (or all if not specified)
        key: Option<String>,
    },
}

// =============================================================================
// ENUMS WITH #[derive(ValueEnum)]
// =============================================================================

#[derive(ValueEnum, Clone, Debug, Default)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Yaml,
    Table,
}

#[derive(ValueEnum, Clone, Debug, Default)]
enum Role {
    Admin,
    #[default]
    User,
    Guest,
    #[value(name = "super-admin", alias = "superadmin")]
    SuperAdmin,
}

#[derive(ValueEnum, Clone, Debug)]
enum SortField {
    #[value(name = "created-at", alias = "created")]
    CreatedAt,
    #[value(name = "updated-at", alias = "updated")]
    UpdatedAt,
    Name,
    Email,
}

#[derive(ValueEnum, Clone, Debug)]
enum Visibility {
    Public,
    Private,
    Internal,
}

#[derive(ValueEnum, Clone, Debug)]
enum Platform {
    Linux,
    MacOS,
    Windows,
    #[value(name = "all")]
    All,
}

#[derive(ValueEnum, Clone, Debug)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(ValueEnum, Clone, Debug)]
enum Component {
    Database,
    Cache,
    Network,
    Storage,
    #[value(name = "all")]
    All,
}

#[derive(ValueEnum, Clone, Debug)]
enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

// =============================================================================
// CUSTOM VALIDATORS
// =============================================================================

fn validate_username(s: &str) -> Result<String, String> {
    if s.len() < 3 {
        return Err("Username must be at least 3 characters".into());
    }
    if s.len() > 20 {
        return Err("Username must be at most 20 characters".into());
    }
    if !s.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Username must contain only alphanumeric characters and underscores".into());
    }
    Ok(s.to_string())
}

// =============================================================================
// MAIN FUNCTION
// =============================================================================

fn main() {
    let cli = Cli::parse();

    // Display global settings
    if cli.verbose > 0 {
        println!("[verbosity: {}]", cli.verbose);
    }
    if cli.no_color {
        println!("[colors disabled]");
    }
    if let Some(config) = &cli.config {
        println!("[config: {:?}]", config);
    }
    println!("[format: {:?}]\n", cli.format);

    // Dispatch to command handlers
    match cli.command {
        Commands::User(cmd) => handle_user(cmd),
        Commands::Project(cmd) => handle_project(cmd),
        Commands::System(cmd) => handle_system(cmd),
        Commands::Version => {
            // Use Clap's built-in version
            let mut cmd = Cli::command();
            println!("{} {}", cmd.get_name(), cmd.get_version().unwrap_or("unknown"));
        }
        Commands::Completions { shell } => {
            println!("Generating completions for {:?}...", shell);
        }
    }
}

fn handle_user(cmd: UserCommands) {
    match cmd {
        UserCommands::Create(args) => {
            println!("Creating user: {}", args.username);
            println!("  Email: {}", args.email);
            println!("  Role: {:?}", args.role);
            if let Some(name) = &args.display_name {
                println!("  Display name: {}", name);
            }
            println!("  Active: {}", args.active);
            if !args.tags.is_empty() {
                println!("  Tags: {:?}", args.tags);
            }
        }
        UserCommands::List(args) => {
            println!("Listing users:");
            println!("  Limit: {}, Offset: {}", args.limit, args.offset);
            println!("  Sort by: {:?} (desc: {})", args.sort_by, args.desc);
            if let Some(role) = &args.role {
                println!("  Filter role: {:?}", role);
            }
            if args.active_only {
                println!("  Active only: true");
            }
            if let Some(search) = &args.search {
                println!("  Search: {}", search);
            }
        }
        UserCommands::Get { username, with_activity } => {
            println!("Getting user: {} (activity: {})", username, with_activity);
        }
        UserCommands::Update { username, updates } => {
            println!("Updating user: {}", username);
            if let Some(email) = &updates.email { println!("  New email: {}", email); }
            if let Some(role) = &updates.role { println!("  New role: {:?}", role); }
            if let Some(name) = &updates.display_name { println!("  New name: {}", name); }
            if let Some(active) = updates.active { println!("  Active: {}", active); }
        }
        UserCommands::Delete { username, force } => {
            println!("Deleting user: {} (force: {})", username, force);
        }
    }
}

fn handle_project(cmd: ProjectCommands) {
    match cmd {
        ProjectCommands::Create { name, visibility, description, template } => {
            println!("Creating project: {}", name);
            println!("  Visibility: {:?}", visibility);
            if let Some(desc) = description { println!("  Description: {}", desc); }
            if let Some(tmpl) = template { println!("  Template: {}", tmpl); }
        }
        ProjectCommands::Build { name, release, target, features, extra_args } => {
            println!("Building project: {}", name);
            println!("  Release: {}", release);
            if let Some(t) = target { println!("  Target: {:?}", t); }
            if !features.is_empty() { println!("  Features: {:?}", features); }
            if !extra_args.is_empty() { println!("  Extra args: {:?}", extra_args); }
        }
        ProjectCommands::Run { name, args } => {
            println!("Running project: {} with args: {:?}", name, args);
        }
        ProjectCommands::List(args) => {
            println!("Listing projects:");
            if let Some(vis) = &args.visibility { println!("  Visibility: {:?}", vis); }
            if let Some(owner) = &args.owner { println!("  Owner: {}", owner); }
            println!("  Include archived: {}", args.include_archived);
        }
    }
}

fn handle_system(cmd: SystemCommands) {
    match cmd {
        SystemCommands::Status => println!("System status: OK"),
        SystemCommands::Config(cfg) => match cfg {
            ConfigCommands::Get { key } => println!("Getting config: {}", key),
            ConfigCommands::Set { key, value } => println!("Setting {} = {}", key, value),
            ConfigCommands::List => println!("Listing all config"),
            ConfigCommands::Reset { key } => {
                if let Some(k) = key {
                    println!("Resetting config: {}", k);
                } else {
                    println!("Resetting all config to defaults");
                }
            }
        },
        SystemCommands::Logs { lines, follow, level } => {
            println!("Showing {} log lines (follow: {})", lines, follow);
            if let Some(l) = level { println!("  Level filter: {:?}", l); }
        }
        SystemCommands::Diagnose { components } => {
            println!("Running diagnostics for: {:?}", components);
        }
    }
}

// =============================================================================
// KEY CONCEPTS REFERENCE:
// =============================================================================
//
// #[derive(Parser)] - Main CLI struct
//   #[command(...)] - Command-level attributes:
//     name, author, version, about, long_about
//     propagate_version, subcommand_required
//     arg_required_else_help, help_template
//     after_help, before_help, after_long_help
//
//   #[arg(...)] - Argument attributes:
//     short, long, value_name, help
//     default_value, default_value_t
//     value_parser, value_enum
//     action, global, env
//     requires, conflicts_with
//     value_delimiter, last, trailing_var_arg
//
// #[derive(Subcommand)] - Enum for subcommands
//   #[command(subcommand)] - Nested subcommands
//   #[command(name = "...")] - Override name
//   #[command(alias = "...")] - Command aliases
//   #[command(hide = true)] - Hide from help
//
// #[derive(Args)] - Reusable argument groups
//   #[command(flatten)] - Include in parent
//   #[command(next_help_heading)] - Group in help
//
// #[derive(ValueEnum)] - Enum for values
//   #[value(name = "...")] - Override value name
//   #[value(alias = "...")] - Value aliases
//   #[default] - Default value
