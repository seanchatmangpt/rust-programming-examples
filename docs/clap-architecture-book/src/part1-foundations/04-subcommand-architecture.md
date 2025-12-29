# Subcommand Architecture

> **Chapter 4** | Part 1: Foundations | Estimated reading time: 14 minutes

Well-designed subcommand hierarchies are the backbone of complex CLI applications. From git's sprawling command tree to cargo's focused toolchain, the structure of subcommands determines how users discover features, how code is organized, and how the application scales. This chapter covers architectural patterns for organizing commands at any scale.

## Hierarchical Command Design

### Understanding Command Hierarchies

Modern CLIs typically organize functionality into hierarchical subcommands. This structure serves multiple purposes: discoverability, documentation, and logical grouping.

```
┌─────────────────────────────────────────────────────────────────┐
│                    COMMAND HIERARCHY MODELS                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   FLAT MODEL                    DEEP MODEL                      │
│   ───────────                   ──────────                      │
│   app init                      app project create              │
│   app build                     app project build               │
│   app test                      app project test                │
│   app deploy                    app project deploy              │
│   app config-set                app config set                  │
│   app config-get                app config get                  │
│   app user-add                  app user add                    │
│   app user-remove               app user remove                 │
│                                                                 │
│   Pros:                         Pros:                           │
│   • Simple discovery            • Logical grouping              │
│   • Faster typing               • Namespace collision avoided   │
│   • Fewer concepts              • Scalable organization         │
│                                                                 │
│   Cons:                         Cons:                           │
│   • Naming collisions           • More typing required          │
│   • Hard to scale               • Harder to discover            │
│   • Cluttered help              • More complex implementation   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Command Naming Conventions

Choose naming patterns that match user mental models:

**Verb-Based Commands**: Action-oriented, imperative style
```
app add <item>
app remove <item>
app list
app update <item>
```

**Noun-Based Namespaces**: Resource-oriented, RESTful style
```
app users list
app users add <name>
app projects create <name>
app projects delete <id>
```

**Hybrid Approach**: Combines both (like git)
```
app remote add <name> <url>
app remote remove <name>
app branch create <name>
app branch delete <name>
```

### Discoverability Principles

Design for users who don't read documentation:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "devtool",
    about = "Development toolkit for project management",
    // Show subcommands in help
    subcommand_required = true,
    // Suggest similar commands on typos
    arg_required_else_help = true,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project (aliases: i, new)
    #[command(alias = "i", alias = "new")]
    Init {
        /// Project name
        name: String,
    },

    /// Build the project (aliases: b)
    #[command(alias = "b")]
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },

    /// Run tests (aliases: t)
    #[command(alias = "t")]
    Test {
        /// Test pattern to match
        pattern: Option<String>,
    },
}
```

## Nested Subcommands

### Two-Level Nesting

The most common pattern organizes commands into logical groups:

```rust
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cloudctl", about = "Cloud resource management")]
struct Cli {
    /// Global verbosity flag
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage compute instances
    Instance(InstanceArgs),

    /// Manage storage volumes
    Storage(StorageArgs),

    /// Manage network resources
    Network(NetworkArgs),
}

// Instance subcommands
#[derive(Args)]
struct InstanceArgs {
    #[command(subcommand)]
    command: InstanceCommands,
}

#[derive(Subcommand)]
enum InstanceCommands {
    /// List all instances
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
    },
    /// Create a new instance
    Create {
        /// Instance name
        name: String,
        /// Instance type
        #[arg(short = 't', long, default_value = "small")]
        instance_type: String,
    },
    /// Delete an instance
    Delete {
        /// Instance ID
        id: String,
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },
    /// SSH into an instance
    Ssh {
        /// Instance ID
        id: String,
    },
}

// Storage subcommands
#[derive(Args)]
struct StorageArgs {
    #[command(subcommand)]
    command: StorageCommands,
}

#[derive(Subcommand)]
enum StorageCommands {
    /// List volumes
    List,
    /// Create a volume
    Create { name: String, size_gb: u32 },
    /// Delete a volume
    Delete { id: String },
    /// Attach volume to instance
    Attach { volume_id: String, instance_id: String },
}

// Network subcommands
#[derive(Args)]
struct NetworkArgs {
    #[command(subcommand)]
    command: NetworkCommands,
}

#[derive(Subcommand)]
enum NetworkCommands {
    /// List networks
    List,
    /// Create a network
    Create { name: String, cidr: String },
}
```

### Deep Nesting Patterns

For complex applications, three or more levels may be appropriate:

```rust
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "k8s-tool")]
struct Cli {
    #[command(subcommand)]
    command: TopLevel,
}

#[derive(Subcommand)]
enum TopLevel {
    /// Cluster operations
    Cluster(ClusterArgs),
}

#[derive(Args)]
struct ClusterArgs {
    #[command(subcommand)]
    command: ClusterCommands,
}

#[derive(Subcommand)]
enum ClusterCommands {
    /// Node management
    Node(NodeArgs),
    /// Namespace management
    Namespace(NamespaceArgs),
}

#[derive(Args)]
struct NodeArgs {
    #[command(subcommand)]
    command: NodeCommands,
}

#[derive(Subcommand)]
enum NodeCommands {
    /// List nodes
    List,
    /// Drain a node
    Drain { name: String },
    /// Cordon a node
    Cordon { name: String },
    /// Uncordon a node
    Uncordon { name: String },
}

#[derive(Args)]
struct NamespaceArgs {
    #[command(subcommand)]
    command: NamespaceCommands,
}

#[derive(Subcommand)]
enum NamespaceCommands {
    List,
    Create { name: String },
    Delete { name: String },
}

// Usage: k8s-tool cluster node drain worker-1
```

### Shared Arguments Across Levels

Use global arguments and flattening for shared options:

```rust
use clap::{Args, Parser, Subcommand};

/// Common output options shared across commands
#[derive(Args, Debug, Clone)]
struct OutputOptions {
    /// Output format
    #[arg(long, global = true, default_value = "table")]
    format: String,

    /// Suppress headers in output
    #[arg(long, global = true)]
    no_headers: bool,
}

/// Common filtering options
#[derive(Args, Debug, Clone)]
struct FilterOptions {
    /// Filter by label (key=value)
    #[arg(long, short = 'l')]
    label: Vec<String>,

    /// Filter by namespace
    #[arg(long, short = 'n')]
    namespace: Option<String>,
}

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    output: OutputOptions,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List resources
    List {
        #[command(flatten)]
        filter: FilterOptions,
    },
    /// Get resource details
    Get {
        /// Resource name
        name: String,
        #[command(flatten)]
        filter: FilterOptions,
    },
}

fn main() {
    let cli = Cli::parse();

    // Output options available everywhere
    println!("Format: {}", cli.output.format);

    match cli.command {
        Commands::List { filter } => {
            println!("Listing with {} labels", filter.label.len());
        }
        Commands::Get { name, filter } => {
            println!("Getting {} in {:?}", name, filter.namespace);
        }
    }
}
```

## Command Routing Patterns

### Match-Based Routing

The standard approach using Rust's pattern matching:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { name: String },
    Remove { name: String },
    List,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { name } => {
            println!("Adding: {}", name);
            // add_item(&name);
        }
        Commands::Remove { name } => {
            println!("Removing: {}", name);
            // remove_item(&name);
        }
        Commands::List => {
            println!("Listing all items");
            // list_items();
        }
    }
}
```

### Trait-Based Dispatch

For more complex applications, use traits for command execution:

```rust
use clap::{Parser, Subcommand};

/// Trait for executable commands
trait Executable {
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize repository
    Init(InitCommand),
    /// Clone repository
    Clone(CloneCommand),
    /// Show status
    Status(StatusCommand),
}

#[derive(clap::Args)]
struct InitCommand {
    /// Directory to initialize
    #[arg(default_value = ".")]
    path: String,
}

impl Executable for InitCommand {
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Initializing repository in: {}", self.path);
        // Actual initialization logic
        Ok(())
    }
}

#[derive(clap::Args)]
struct CloneCommand {
    /// Repository URL
    url: String,
    /// Destination directory
    dest: Option<String>,
}

impl Executable for CloneCommand {
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dest = self.dest.as_deref().unwrap_or(".");
        println!("Cloning {} to {}", self.url, dest);
        Ok(())
    }
}

#[derive(clap::Args)]
struct StatusCommand {
    /// Show verbose status
    #[arg(short, long)]
    verbose: bool,
}

impl Executable for StatusCommand {
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Status (verbose: {})", self.verbose);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(cmd) => cmd.execute(),
        Commands::Clone(cmd) => cmd.execute(),
        Commands::Status(cmd) => cmd.execute(),
    }
}
```

### Command Registry Pattern

For plugin systems or large applications, use a registry:

```rust
use clap::{ArgMatches, Command};
use std::collections::HashMap;

type CommandHandler = Box<dyn Fn(&ArgMatches) -> Result<(), String>>;

struct CommandRegistry {
    handlers: HashMap<String, CommandHandler>,
    commands: Vec<Command>,
}

impl CommandRegistry {
    fn new() -> Self {
        CommandRegistry {
            handlers: HashMap::new(),
            commands: Vec::new(),
        }
    }

    fn register<F>(&mut self, name: &str, command: Command, handler: F)
    where
        F: Fn(&ArgMatches) -> Result<(), String> + 'static,
    {
        self.commands.push(command);
        self.handlers.insert(name.to_string(), Box::new(handler));
    }

    fn build_cli(&self, base: Command) -> Command {
        let mut cmd = base;
        for subcmd in &self.commands {
            cmd = cmd.subcommand(subcmd.clone());
        }
        cmd
    }

    fn dispatch(&self, matches: &ArgMatches) -> Result<(), String> {
        if let Some((name, sub_matches)) = matches.subcommand() {
            if let Some(handler) = self.handlers.get(name) {
                return handler(sub_matches);
            }
        }
        Err("No command specified".to_string())
    }
}

fn main() {
    let mut registry = CommandRegistry::new();

    // Register commands dynamically
    registry.register(
        "hello",
        Command::new("hello")
            .about("Say hello")
            .arg(clap::Arg::new("name").required(true)),
        |matches| {
            let name: &String = matches.get_one("name").unwrap();
            println!("Hello, {}!", name);
            Ok(())
        },
    );

    registry.register(
        "goodbye",
        Command::new("goodbye").about("Say goodbye"),
        |_matches| {
            println!("Goodbye!");
            Ok(())
        },
    );

    let cli = registry.build_cli(Command::new("app").version("1.0"));
    let matches = cli.get_matches();

    if let Err(e) = registry.dispatch(&matches) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

## Scaling Subcommand Systems

### Module Organization

Organize code to mirror command structure:

```
src/
├── main.rs              # Entry point, CLI definition
├── cli/
│   ├── mod.rs           # CLI struct, top-level routing
│   ├── instance.rs      # Instance command definitions
│   ├── storage.rs       # Storage command definitions
│   └── network.rs       # Network command definitions
├── commands/
│   ├── mod.rs           # Command implementations
│   ├── instance/
│   │   ├── mod.rs
│   │   ├── list.rs      # instance list implementation
│   │   ├── create.rs    # instance create implementation
│   │   └── delete.rs    # instance delete implementation
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── list.rs
│   │   └── create.rs
│   └── network/
│       └── mod.rs
└── lib.rs               # Shared types and utilities
```

**main.rs**:
```rust
mod cli;
mod commands;

use clap::Parser;
use cli::Cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    commands::dispatch(cli)
}
```

**cli/mod.rs**:
```rust
mod instance;
mod storage;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cloudctl")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Instance(instance::InstanceArgs),
    Storage(storage::StorageArgs),
}
```

**commands/mod.rs**:
```rust
mod instance;
mod storage;

use crate::cli::{Cli, Commands};

pub fn dispatch(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Instance(args) => instance::dispatch(args),
        Commands::Storage(args) => storage::dispatch(args),
    }
}
```

### Code Splitting Strategies

For large CLIs, consider feature flags:

```toml
# Cargo.toml
[features]
default = ["instance", "storage", "network"]
instance = []
storage = []
network = []
full = ["instance", "storage", "network"]
```

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[cfg(feature = "instance")]
    Instance(InstanceArgs),

    #[cfg(feature = "storage")]
    Storage(StorageArgs),

    #[cfg(feature = "network")]
    Network(NetworkArgs),
}

#[cfg(feature = "instance")]
#[derive(clap::Args)]
struct InstanceArgs {
    // ...
}

#[cfg(feature = "storage")]
#[derive(clap::Args)]
struct StorageArgs {
    // ...
}

#[cfg(feature = "network")]
#[derive(clap::Args)]
struct NetworkArgs {
    // ...
}
```

## Real-World Examples

### Git-like CLI Structure

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mygit", about = "A git-like version control system")]
struct Cli {
    /// Run as if started in <path>
    #[arg(short = 'C', long, global = true)]
    directory: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create an empty repository
    Init {
        /// Directory for the new repository
        #[arg(default_value = ".")]
        directory: String,
    },

    /// Clone a repository
    Clone {
        /// Repository URL
        repository: String,
        /// Destination directory
        directory: Option<String>,
    },

    /// Add file contents to the index
    Add {
        /// Files to add
        #[arg(required = true)]
        files: Vec<String>,
    },

    /// Record changes to the repository
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: Option<String>,
        /// Amend previous commit
        #[arg(long)]
        amend: bool,
    },

    /// Show the working tree status
    Status {
        /// Show short format
        #[arg(short, long)]
        short: bool,
    },

    /// Manage remote repositories
    #[command(subcommand)]
    Remote(RemoteCommands),

    /// List, create, or delete branches
    #[command(subcommand)]
    Branch(BranchCommands),
}

#[derive(Subcommand)]
enum RemoteCommands {
    /// Add a remote
    Add { name: String, url: String },
    /// Remove a remote
    Remove { name: String },
    /// List remotes
    #[command(name = "-v")]
    Verbose,
}

#[derive(Subcommand)]
enum BranchCommands {
    /// List branches
    List,
    /// Create a branch
    Create { name: String },
    /// Delete a branch
    Delete {
        name: String,
        #[arg(short = 'D', long)]
        force: bool,
    },
}
```

### Cargo-like Tool Structure

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mytool", about = "Project build tool")]
struct Cli {
    /// Path to Manifest.toml
    #[arg(long, global = true)]
    manifest_path: Option<PathBuf>,

    /// Use verbose output
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Do not print output
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile the current package
    Build {
        /// Build artifacts in release mode
        #[arg(long)]
        release: bool,
        /// Package to build
        #[arg(short, long)]
        package: Option<String>,
    },

    /// Run the main binary
    Run {
        /// Build in release mode
        #[arg(long)]
        release: bool,
        /// Arguments to pass to the binary
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Execute unit and integration tests
    Test {
        /// Test only the specified test target
        #[arg(long)]
        test: Option<String>,
        /// Run ignored tests
        #[arg(long)]
        ignored: bool,
    },

    /// Check for errors without building
    Check,

    /// Remove generated artifacts
    Clean,

    /// Create a new package
    New {
        /// Package name
        name: String,
        /// Create a library package
        #[arg(long)]
        lib: bool,
    },
}
```

## Summary

### Key Takeaways

1. **Choose hierarchy depth wisely**: Flat for simple tools, deep for complex systems with clear resource boundaries
2. **Follow naming conventions**: Verb-based for actions, noun-based for resources, hybrid for complex domains
3. **Design for discoverability**: Use aliases, helpful descriptions, and `arg_required_else_help`
4. **Use enums for type-safe routing**: Compiler ensures all commands are handled
5. **Organize code to mirror commands**: File structure should reflect command hierarchy
6. **Share common arguments with `#[command(flatten)]`**: Reduces duplication while maintaining type safety
7. **Consider feature flags for large CLIs**: Reduce binary size by compiling only needed commands

> **Cross-Reference**: For plugin-based dynamic subcommands, see [Chapter 12: Plugin Systems with Clap](../part3-advanced-architecture/12-plugin-systems.md). For testing subcommand routing, see [Chapter 15: Testing CLI Applications](../part3-advanced-architecture/15-testing-cli-applications.md).

---

*Next: [Error Handling Foundations](./05-error-handling-foundations.md)*
