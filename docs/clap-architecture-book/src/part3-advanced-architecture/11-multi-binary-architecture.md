# Multi-Binary Architecture

> **Chapter 11** | Part 3: Advanced Architecture | Estimated reading time: 18 minutes

Large-scale Rust projects often require multiple CLI binaries that share common functionality. Whether you are building a suite of DevOps tools, a family of data processing utilities, or an extensible platform with specialized frontends, understanding multi-binary architecture patterns is essential for maintainable, scalable CLI systems.

## Workspace Organization

### The Foundation: Cargo Workspaces

Cargo workspaces provide the foundation for multi-binary Rust projects. A workspace is a set of packages that share a common `Cargo.lock` and output directory, enabling coordinated development while maintaining clear boundaries between components.

```toml
# Cargo.toml (workspace root)
[workspace]
resolver = "2"
members = [
    "crates/cli-core",
    "crates/cli-auth",
    "crates/myapp",
    "crates/myapp-admin",
    "crates/myapp-worker",
]

[workspace.package]
version = "2.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/example/myapp"

[workspace.dependencies]
clap = { version = "4.5", features = ["derive", "env", "string"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"

# Shared internal crates
cli-core = { path = "crates/cli-core" }
cli-auth = { path = "crates/cli-auth" }
```

### Workspace Architecture Overview

```
+===========================================================================+
|                    MULTI-BINARY WORKSPACE ARCHITECTURE                     |
+===========================================================================+

    +-------------------------------------------------------------------+
    |                      WORKSPACE ROOT                                |
    |                                                                    |
    |   Cargo.toml (workspace)    Cargo.lock (shared)                   |
    +-------------------------------------------------------------------+
                                    |
            +-----------------------+-----------------------+
            |                       |                       |
            v                       v                       v
    +---------------+       +---------------+       +---------------+
    |  SHARED LIBS  |       |   BINARIES    |       |    TESTS      |
    +---------------+       +---------------+       +---------------+
    |               |       |               |       |               |
    | cli-core/     |<------| myapp/        |       | tests/        |
    |  - args/      |       |  main.rs      |       |  integration/ |
    |  - config/    |       |  commands/    |       |               |
    |  - logging    |       +---------------+       +---------------+
    |               |       |               |
    | cli-auth/     |<------| myapp-admin/  |
    |  - oauth      |       |  main.rs      |
    |  - creds      |       +---------------+
    +---------------+       |               |
            ^               | myapp-worker/ |
            |               |  main.rs      |
            +---------------+---------------+


    DEPENDENCY FLOW:
    ================

    Binaries depend on shared libs (never reverse):

    myapp --------+
                  |
    myapp-admin --+---> cli-core ---> external crates
                  |        |
    myapp-worker -+        +---> cli-auth ---> oauth2, jwt
                                    |
                                    v
                                external crates
```

**Diagram Description**: This architecture shows the three-tier workspace structure: shared libraries at the bottom, binaries that compose them in the middle, and workspace-level tests. The dependency flow is always downward (binaries to libs), never upward.

### Directory Layout for Production Systems

A well-organized workspace structure enables independent development while promoting code reuse:

```
myapp/
├── Cargo.toml                    # Workspace root
├── Cargo.lock                    # Shared lockfile
├── .cargo/
│   └── config.toml               # Workspace-wide cargo config
├── crates/
│   ├── cli-core/                 # Shared CLI primitives
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── args/             # Shared argument groups
│   │       │   ├── mod.rs
│   │       │   ├── verbosity.rs
│   │       │   ├── output.rs
│   │       │   └── connection.rs
│   │       ├── config/           # Configuration loading
│   │       └── logging.rs        # Unified logging setup
│   ├── cli-auth/                 # Authentication primitives
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── credentials.rs
│   │       └── oauth.rs
│   ├── myapp/                    # Primary user-facing binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/
│   ├── myapp-admin/              # Administrative binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   └── myapp-worker/             # Background worker binary
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
├── tests/                        # Workspace-level integration tests
└── docs/                         # Shared documentation
```

### Architecture Decision: Flat vs. Nested Crate Organization

| Structure | When to Use | Trade-offs |
|-----------|-------------|------------|
| **Flat** (`crates/`) | Most projects; clear visibility | More directories at one level |
| **Nested** (`bins/`, `libs/`) | Large projects with many crates | Deeper hierarchy, more navigation |
| **Single crate, multiple bins** | Simple related tools | Shared compilation, less isolation |

For most production systems, the flat `crates/` approach provides the best balance of organization and discoverability.

## Shared Argument Libraries

### Composable Argument Groups

The key to maintainable multi-binary CLIs is designing composable argument groups that binaries can selectively include. This avoids duplication while allowing binary-specific customization.

```rust
// crates/cli-core/src/args/verbosity.rs
use clap::Args;

/// Verbosity control arguments, reusable across all binaries
#[derive(Args, Clone, Debug, Default)]
pub struct VerbosityArgs {
    /// Increase verbosity level (can be repeated: -v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress all output except errors
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Enable debug output (equivalent to -vvv)
    #[arg(long, global = true, hide = true)]
    pub debug: bool,
}

impl VerbosityArgs {
    /// Convert verbosity settings to a tracing log level
    pub fn log_level(&self) -> tracing::Level {
        if self.quiet {
            return tracing::Level::ERROR;
        }
        if self.debug {
            return tracing::Level::TRACE;
        }
        match self.verbose {
            0 => tracing::Level::WARN,
            1 => tracing::Level::INFO,
            2 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        }
    }

    /// Check if any output should be shown
    pub fn is_silent(&self) -> bool {
        self.quiet && self.verbose == 0
    }
}
```

```rust
// crates/cli-core/src/args/output.rs
use clap::{Args, ValueEnum};
use std::path::PathBuf;

/// Output formatting arguments
#[derive(Args, Clone, Debug)]
pub struct OutputArgs {
    /// Output format
    #[arg(long, short = 'F', value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(long, short)]
    pub output: Option<PathBuf>,

    /// Colorize output (auto-detected if not specified)
    #[arg(long, value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,
}

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Yaml,
    Table,
    Csv,
}

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum ColorChoice {
    #[default]
    Auto,
    Always,
    Never,
}

impl OutputArgs {
    pub fn should_colorize(&self) -> bool {
        match self.color {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => atty::is(atty::Stream::Stdout),
        }
    }
}
```

### Trait-Based Behavioral Contracts

Define traits that binaries must implement, enabling polymorphic handling of common operations:

```rust
// crates/cli-core/src/lib.rs
use crate::args::{VerbosityArgs, OutputArgs};
use anyhow::Result;

/// Contract for all CLI entry points in the workspace
pub trait CliApplication: clap::Parser {
    /// Application name for logging and error messages
    const NAME: &'static str;

    /// Get verbosity configuration
    fn verbosity(&self) -> &VerbosityArgs;

    /// Get output configuration (optional for some binaries)
    fn output(&self) -> Option<&OutputArgs> {
        None
    }

    /// Run the application logic
    fn run(self) -> Result<()>;
}

/// Standard CLI initialization and execution
pub fn execute<T: CliApplication>() -> ! {
    // Initialize logging before parsing (to catch parse errors)
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("warn")
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    let app = T::parse();

    // Reconfigure logging with actual verbosity
    let level = app.verbosity().log_level();
    // ... reconfigure tracing with proper level

    match app.run() {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("{}: error: {:#}", T::NAME, e);
            std::process::exit(1)
        }
    }
}
```

### Binary-Specific Composition

Each binary composes shared argument groups with its specific requirements:

```rust
// crates/myapp/src/main.rs
use clap::{Parser, Subcommand};
use cli_core::{CliApplication, args::{VerbosityArgs, OutputArgs}};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "myapp", version, about = "Primary application interface")]
pub struct Cli {
    #[command(flatten)]
    verbosity: VerbosityArgs,

    #[command(flatten)]
    output: OutputArgs,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init(InitArgs),
    /// Run the main workflow
    Run(RunArgs),
    /// Show status information
    Status,
}

impl CliApplication for Cli {
    const NAME: &'static str = "myapp";

    fn verbosity(&self) -> &VerbosityArgs {
        &self.verbosity
    }

    fn output(&self) -> Option<&OutputArgs> {
        Some(&self.output)
    }

    fn run(self) -> Result<()> {
        match self.command {
            Commands::Init(args) => commands::init(args, &self.output),
            Commands::Run(args) => commands::run(args, &self.verbosity),
            Commands::Status => commands::status(&self.output),
        }
    }
}

fn main() -> ! {
    cli_core::execute::<Cli>()
}
```

```rust
// crates/myapp-admin/src/main.rs - Different composition
use clap::Parser;
use cli_core::{CliApplication, args::VerbosityArgs};
use cli_auth::AdminCredentials;

#[derive(Parser)]
#[command(name = "myapp-admin", version, about = "Administrative operations")]
pub struct AdminCli {
    #[command(flatten)]
    verbosity: VerbosityArgs,

    /// Admin authentication (required for all operations)
    #[command(flatten)]
    credentials: AdminCredentials,

    /// Target environment
    #[arg(long, env = "MYAPP_ENV", default_value = "production")]
    environment: String,

    #[command(subcommand)]
    command: AdminCommands,
}

// Admin binary doesn't use OutputArgs - always outputs for admin consumption
impl CliApplication for AdminCli {
    const NAME: &'static str = "myapp-admin";

    fn verbosity(&self) -> &VerbosityArgs {
        &self.verbosity
    }

    fn run(self) -> anyhow::Result<()> {
        self.credentials.verify()?;
        // ... admin command dispatch
        Ok(())
    }
}
```

## Binary Dispatch Patterns

### Pattern 1: Unified Multi-Call Binary (BusyBox Style)

A single compiled binary that behaves differently based on how it is invoked:

```rust
// crates/unified/src/main.rs
use std::path::Path;
use clap::{Parser, Subcommand};

fn main() -> anyhow::Result<()> {
    // Determine personality from argv[0]
    let binary_name = std::env::args()
        .next()
        .and_then(|s| {
            Path::new(&s)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "myapp".to_string());

    match binary_name.as_str() {
        "myapp" | "myapp.exe" => run_main_cli(),
        "myapp-admin" | "myapp-admin.exe" => run_admin_cli(),
        "myapp-worker" | "myapp-worker.exe" => run_worker_cli(),
        _ => {
            // Unknown invocation - try as subcommand
            run_with_subcommand(&binary_name)
        }
    }
}

fn run_main_cli() -> anyhow::Result<()> {
    let cli = MainCli::parse();
    cli.execute()
}

fn run_admin_cli() -> anyhow::Result<()> {
    let cli = AdminCli::parse();
    cli.execute()
}

// Installation creates symlinks:
// ln -s myapp myapp-admin
// ln -s myapp myapp-worker
```

**When to Use**: Embedded systems, container images where binary size matters, or when all tools share most dependencies.

**Architecture Decision**: This pattern trades compilation simplicity for runtime complexity. All features are compiled into every binary, increasing size but simplifying deployment.

### Pattern 2: Feature-Gated Binaries

Use Cargo features to control which binaries are built:

```toml
# crates/myapp/Cargo.toml
[package]
name = "myapp"

[[bin]]
name = "myapp"
required-features = ["cli"]

[[bin]]
name = "myapp-admin"
required-features = ["admin"]

[[bin]]
name = "myapp-worker"
required-features = ["worker"]

[features]
default = ["cli"]
cli = ["dep:dialoguer", "dep:indicatif"]
admin = ["dep:cli-auth"]
worker = ["dep:tokio/rt-multi-thread"]
full = ["cli", "admin", "worker"]
```

```rust
// src/bin/myapp.rs
#[cfg(feature = "cli")]
fn main() {
    myapp::cli::run()
}

#[cfg(not(feature = "cli"))]
fn main() {
    compile_error!("The 'cli' feature is required for this binary")
}
```

Build specific binaries:

```bash
# Build only main CLI
cargo build --bin myapp --features cli

# Build admin tool
cargo build --bin myapp-admin --features admin

# Build everything
cargo build --features full
```

### Pattern 3: Workspace with Independent Binaries

The most flexible pattern for large teams:

```rust
// Each binary is a separate crate with its own Cargo.toml
// crates/myapp-worker/Cargo.toml
[package]
name = "myapp-worker"
version.workspace = true
edition.workspace = true

[dependencies]
cli-core.workspace = true
tokio.workspace = true
clap.workspace = true

# Worker-specific dependencies not shared with other binaries
lapin = "2.3"  # RabbitMQ client
```

## Feature Gate Strategies

### Capability-Based Features

Organize features around capabilities rather than binaries:

```toml
[features]
default = []

# Capability features
networking = ["dep:reqwest", "dep:tokio"]
database = ["dep:sqlx"]
auth = ["dep:oauth2", "dep:jsonwebtoken"]
interactive = ["dep:dialoguer", "dep:indicatif"]

# Output format features
json-output = ["dep:serde_json"]
yaml-output = ["dep:serde_yaml"]
table-output = ["dep:tabled"]

# Binary profiles (combinations of capabilities)
cli-standard = ["networking", "interactive", "json-output"]
cli-minimal = ["json-output"]
admin-full = ["networking", "database", "auth"]
```

```rust
// Conditional compilation based on features
#[derive(clap::Subcommand)]
pub enum Commands {
    /// Always available
    Status,

    #[cfg(feature = "networking")]
    /// Fetch remote resources (requires networking feature)
    Fetch(FetchArgs),

    #[cfg(feature = "interactive")]
    /// Interactive configuration wizard
    Configure,

    #[cfg(feature = "database")]
    /// Database operations
    Db(DbCommands),
}
```

### When NOT To Use Multi-Binary Architecture

Multi-binary workspaces add complexity. Avoid them when:

1. **Single-purpose tools**: If your CLI does one thing well, keep it simple
2. **Tightly coupled functionality**: If binaries would share 90%+ of code, use subcommands instead
3. **Small teams**: The overhead of workspace management may outweigh benefits
4. **Rapid prototyping**: Start with a single binary; extract later when patterns emerge

**Red flags that suggest over-engineering**:
- More than 3 shared crates for 2 binaries
- Feature flags controlling which subcommands appear
- Extensive `#[cfg]` blocks making code hard to read

## Performance Considerations

### Compilation Performance

| Strategy | Clean Build | Incremental | Binary Size |
|----------|-------------|-------------|-------------|
| Single binary with features | Slower | Fast | Larger |
| Workspace with shared crates | Faster | Medium | Smaller per binary |
| Fully independent crates | Slowest | Fastest | Smallest |

**Optimization tips**:

```toml
# .cargo/config.toml - Workspace-wide build optimization
[build]
# Use all CPU cores
jobs = 8

[profile.dev]
# Faster debug builds
debug = 1
opt-level = 0

[profile.dev.package."*"]
# Optimize dependencies even in dev
opt-level = 2

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
```

### Runtime Performance

Shared libraries enable faster startup through shared page caches on Linux:

```bash
# Build shared library and linking binaries
cargo build --release

# Verify shared dependencies
ldd target/release/myapp
ldd target/release/myapp-admin
# Common .so files are loaded once in memory
```

## Summary

Multi-binary architecture enables building sophisticated CLI tool suites while maintaining code quality and development velocity.

### Key Takeaways

1. **Cargo workspaces** provide the foundation for multi-binary projects with shared dependencies and coordinated versioning
2. **Composable argument groups** eliminate duplication while allowing binary-specific customization
3. **Trait-based contracts** ensure consistent behavior across binaries while enabling polymorphism
4. **Feature gates** control binary capabilities and dependencies for optimized builds
5. **Choose the right pattern**: BusyBox for size, features for flexibility, workspace for team scalability
6. **Avoid premature extraction**: Start with subcommands; extract binaries when clear boundaries emerge

### Architecture Decisions Documented

| Decision | Recommendation | Rationale |
|----------|----------------|-----------|
| Crate organization | Flat `crates/` directory | Clear visibility, easy navigation |
| Shared arguments | Trait + flatten pattern | Type-safe, composable, testable |
| Binary dispatch | Workspace (multiple crates) | Best isolation, team scalability |
| Feature gates | Capability-based | Maps to user needs, not internals |

> **Cross-Reference**: See [Chapter 12](./12-plugin-systems.md) for extending multi-binary systems with plugins, and [Chapter 19](../part4-real-world-systems/19-performance-optimization.md) for binary size optimization techniques.

---

*Next: [Plugin Systems with Clap](./12-plugin-systems.md)*
