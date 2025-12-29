# Declarative vs Derive Architecture

> **Chapter 2** | Part 1: Foundations | Estimated reading time: 14 minutes

Clap offers two primary approaches to defining command-line interfaces: the builder pattern (imperative) and derive macros (declarative). Each approach has distinct strengths, and understanding when to use each is fundamental to designing maintainable CLI architectures.

## Builder Pattern Fundamentals

The builder pattern is Clap's original API, offering maximum flexibility through method chaining. It constructs CLI definitions programmatically at runtime.

### The Imperative Approach

```rust
use clap::{Arg, ArgAction, Command};

fn build_cli() -> Command {
    Command::new("filemanager")
        .version("2.1.0")
        .author("CLI Team <team@example.com>")
        .about("Manage files with precision")
        .arg(
            Arg::new("input")
                .help("Input file path")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file path")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(ArgAction::Count),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .help("Output format")
                .value_parser(["json", "yaml", "toml"])
                .default_value("json"),
        )
}

fn main() {
    let matches = build_cli().get_matches();

    let input: &String = matches.get_one("input").unwrap();
    let output: Option<&String> = matches.get_one("output");
    let verbosity: u8 = *matches.get_one("verbose").unwrap_or(&0);
    let format: &String = matches.get_one("format").unwrap();

    println!("Processing {} -> {:?} (format: {}, verbosity: {})",
             input, output, format, verbosity);
}
```

### Builder Architecture Pattern

The builder pattern follows a predictable flow that can be visualized as:

```
┌─────────────────────────────────────────────────────────────────┐
│                     BUILDER PATTERN FLOW                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Command::new("app")                                           │
│         │                                                       │
│         ├──▶ .version() / .author() / .about()                  │
│         │         │                                             │
│         │         ▼                                             │
│         ├──▶ .arg(Arg::new("name")...)   ◀─── Repeated         │
│         │         │                                             │
│         │         ▼                                             │
│         ├──▶ .subcommand(Command::new...)  ◀─── Nested         │
│         │         │                                             │
│         │         ▼                                             │
│         └──▶ .get_matches()                                     │
│                   │                                             │
│                   ▼                                             │
│              ArgMatches  ───▶  Runtime value extraction         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Dynamic CLI Construction

The builder pattern excels when CLI structure is determined at runtime:

```rust
use clap::{Arg, Command};

fn build_dynamic_cli(plugins: &[PluginConfig]) -> Command {
    let mut cmd = Command::new("extensible-app")
        .version("1.0.0")
        .about("An extensible CLI application");

    // Add arguments from configuration
    for plugin in plugins {
        let mut arg = Arg::new(&plugin.name)
            .long(&plugin.flag_name)
            .help(&plugin.description);

        if plugin.takes_value {
            arg = arg.value_name(&plugin.value_name);
        }

        if let Some(ref default) = plugin.default_value {
            arg = arg.default_value(default);
        }

        cmd = cmd.arg(arg);
    }

    cmd
}

struct PluginConfig {
    name: String,
    flag_name: String,
    description: String,
    takes_value: bool,
    value_name: String,
    default_value: Option<String>,
}
```

### Builder Advantages

1. **Runtime Flexibility**: Construct commands based on runtime conditions, configuration files, or discovered plugins
2. **Conditional Logic**: Add or modify arguments based on environment or feature flags
3. **Plugin Architecture**: Build CLI structure dynamically from external modules
4. **Fine-Grained Control**: Direct access to every aspect of argument configuration

## Derive Macro Approach

The derive macro approach transforms Rust struct definitions into CLI parsers at compile time, providing type safety and reduced boilerplate.

### The Declarative Approach

```rust
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "filemanager", version, author, about = "Manage files with precision")]
struct Cli {
    /// Input file path
    input: PathBuf,

    /// Output file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Enable verbose output (can be repeated: -v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = Format::Json)]
    format: Format,
}

#[derive(Clone, Debug, ValueEnum)]
enum Format {
    Json,
    Yaml,
    Toml,
}

fn main() {
    let cli = Cli::parse();

    println!("Processing {:?} -> {:?} (format: {:?}, verbosity: {})",
             cli.input, cli.output, cli.format, cli.verbose);
}
```

### Macro Expansion Visualization

Understanding what the derive macro generates helps debug issues and appreciate the abstraction:

```rust
// This derive definition...
#[derive(Parser)]
#[command(name = "example")]
struct Args {
    /// A required positional argument
    input: String,

    /// An optional flag
    #[arg(short, long)]
    verbose: bool,
}

// ...expands approximately to:
impl clap::Parser for Args {
    fn parse() -> Self {
        Self::parse_from(std::env::args_os())
    }

    fn parse_from<I, T>(itr: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::try_parse_from(itr).unwrap_or_else(|e| e.exit())
    }

    fn try_parse_from<I, T>(itr: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let matches = <Self as clap::CommandFactory>::command()
            .try_get_matches_from(itr)?;
        <Self as clap::FromArgMatches>::from_arg_matches(&matches)
            .map_err(|e| e.format(&mut <Self as clap::CommandFactory>::command()))
    }
}

impl clap::CommandFactory for Args {
    fn command() -> clap::Command {
        clap::Command::new("example")
            .arg(
                clap::Arg::new("input")
                    .help("A required positional argument")
                    .required(true)
                    .index(1)
                    .value_parser(clap::value_parser!(String)),
            )
            .arg(
                clap::Arg::new("verbose")
                    .help("An optional flag")
                    .short('v')
                    .long("verbose")
                    .action(clap::ArgAction::SetTrue),
            )
    }
}

impl clap::FromArgMatches for Args {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        Ok(Args {
            input: matches.get_one::<String>("input").cloned().unwrap(),
            verbose: matches.get_flag("verbose"),
        })
    }
}
```

### Derive Advantages

1. **Type Safety**: Rust's type system ensures arguments are parsed correctly at compile time
2. **Self-Documenting**: The struct definition is the single source of truth
3. **Reduced Boilerplate**: Doc comments become help text automatically
4. **Refactoring Safety**: Compiler catches breaking changes
5. **IDE Support**: Full autocomplete and type hints

## When to Use Each

### Decision Framework

```
┌─────────────────────────────────────────────────────────────────┐
│                    APPROACH SELECTION GUIDE                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Is CLI structure known at compile time?                       │
│        │                                                        │
│        ├── YES ──▶ Use DERIVE (preferred)                       │
│        │                                                        │
│        └── NO ───▶ Is it partially dynamic?                     │
│                         │                                       │
│                         ├── Partially ──▶ Use HYBRID            │
│                         │                                       │
│                         └── Fully ──────▶ Use BUILDER           │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│   Additional considerations:                                    │
│                                                                 │
│   • Plugin system needed?      ──▶ Builder or Hybrid            │
│   • Config-driven arguments?   ──▶ Builder                      │
│   • Maximum type safety?       ──▶ Derive                       │
│   • Team familiarity?          ──▶ Consider learning curve      │
│   • Binary size critical?      ──▶ Builder (slightly smaller)   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Use Builder When

**Plugin-Based Architecture**: When commands come from dynamically loaded modules:

```rust
use clap::Command;

trait Plugin {
    fn name(&self) -> &str;
    fn register_commands(&self, cmd: Command) -> Command;
}

fn build_with_plugins(plugins: &[Box<dyn Plugin>]) -> Command {
    let mut cmd = Command::new("plugin-host");

    for plugin in plugins {
        cmd = plugin.register_commands(cmd);
    }

    cmd
}
```

**Configuration-Driven CLIs**: When argument definitions come from external files:

```rust
use clap::{Arg, Command};
use serde::Deserialize;

#[derive(Deserialize)]
struct ArgConfig {
    name: String,
    short: Option<char>,
    long: Option<String>,
    help: String,
    required: bool,
}

fn build_from_config(config_path: &str) -> Command {
    let config: Vec<ArgConfig> = load_config(config_path);
    let mut cmd = Command::new("configurable");

    for arg_cfg in config {
        let mut arg = Arg::new(&arg_cfg.name).help(&arg_cfg.help);

        if let Some(s) = arg_cfg.short {
            arg = arg.short(s);
        }
        if let Some(ref l) = arg_cfg.long {
            arg = arg.long(l);
        }
        if arg_cfg.required {
            arg = arg.required(true);
        }

        cmd = cmd.arg(arg);
    }

    cmd
}

fn load_config(_path: &str) -> Vec<ArgConfig> {
    // Load from YAML/TOML/JSON
    vec![]
}
```

### Use Derive When

**Standard Application CLIs**: Most applications with fixed command structures:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "myapp", version, about)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init {
        /// Project name
        name: String,
        /// Use default template
        #[arg(long)]
        default: bool,
    },
    /// Build the project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Run the project
    Run {
        /// Arguments to pass to the binary
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}
```

### Hybrid Approaches

The most powerful pattern combines both approaches:

```rust
use clap::{Args, Command, FromArgMatches, Parser};

// Static core CLI with derive
#[derive(Parser)]
#[command(name = "hybrid-app")]
struct Cli {
    /// Global verbosity level
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<StaticCommands>,
}

#[derive(clap::Subcommand)]
enum StaticCommands {
    /// Built-in init command
    Init { name: String },
}

// Dynamic extension via builder
fn augment_with_plugins(plugins: &[PluginDef]) -> Command {
    let mut cmd = Cli::command();

    for plugin in plugins {
        let subcmd = Command::new(&plugin.name)
            .about(&plugin.description);
        cmd = cmd.subcommand(subcmd);
    }

    cmd
}

struct PluginDef {
    name: String,
    description: String,
}

fn main() {
    let plugins = discover_plugins();
    let cmd = augment_with_plugins(&plugins);
    let matches = cmd.get_matches();

    // Handle static commands via derive
    if let Ok(cli) = Cli::from_arg_matches(&matches) {
        if let Some(cmd) = cli.command {
            match cmd {
                StaticCommands::Init { name } => println!("Init: {}", name),
            }
            return;
        }
    }

    // Handle dynamic plugin commands
    if let Some((name, sub_matches)) = matches.subcommand() {
        handle_plugin_command(name, sub_matches);
    }
}

fn discover_plugins() -> Vec<PluginDef> { vec![] }
fn handle_plugin_command(_name: &str, _matches: &clap::ArgMatches) {}
```

## Comparison Matrix

| Aspect | Builder Pattern | Derive Macros |
|--------|----------------|---------------|
| **Type Safety** | Runtime (via `get_one`) | Compile-time |
| **Flexibility** | Maximum | Constrained to struct layout |
| **Readability** | Moderate (method chains) | High (declarative) |
| **Compile Time** | Faster | Slower (macro expansion) |
| **Runtime Performance** | Identical | Identical |
| **Binary Size** | Slightly smaller | Slightly larger |
| **Error Messages** | Manual context | Automatic from types |
| **IDE Support** | Standard | Enhanced (struct fields) |
| **Learning Curve** | Moderate | Lower for Rust developers |
| **Maintenance** | Manual sync required | Single source of truth |
| **Dynamic Construction** | Native | Requires hybrid approach |
| **Plugin Support** | Excellent | Limited without builder |

## Migration Between Approaches

### Builder to Derive

When migrating from builder to derive, follow this mapping:

```rust
// BEFORE: Builder pattern
let cmd = Command::new("app")
    .arg(Arg::new("config")
        .short('c')
        .long("config")
        .help("Configuration file")
        .value_name("FILE"));

// AFTER: Derive pattern
#[derive(Parser)]
struct Cli {
    /// Configuration file
    #[arg(short = 'c', long, value_name = "FILE")]
    config: Option<String>,
}
```

### Derive to Builder (for dynamic extension)

```rust
use clap::{CommandFactory, Parser};

#[derive(Parser)]
struct BaseCli {
    #[arg(short, long)]
    verbose: bool,
}

fn extend_cli() -> clap::Command {
    // Start from derive-generated command
    let mut cmd = BaseCli::command();

    // Add dynamic elements
    cmd = cmd.arg(
        clap::Arg::new("dynamic-arg")
            .long("dynamic")
            .help("Added at runtime")
    );

    cmd
}
```

## Performance Considerations

Both approaches produce identical runtime performance. The key differences are:

1. **Compile Time**: Derive macros add compilation overhead (typically 0.5-2 seconds)
2. **Binary Size**: Derive includes additional trait implementations (~5-15KB difference)
3. **Startup Time**: Both are effectively instantaneous (parsing is not a bottleneck)

For most applications, these differences are negligible. Choose based on ergonomics and architecture, not performance.

## Summary

### Key Takeaways

1. **Derive is the default choice** for applications with compile-time-known CLI structures
2. **Builder enables dynamic CLIs** for plugin systems and configuration-driven tools
3. **Hybrid approaches** combine compile-time safety with runtime flexibility
4. **Both produce identical runtime behavior**; choose based on architectural needs
5. **Migration between approaches is straightforward** using Clap's unified internal representation
6. **Type safety** is derive's primary advantage; **flexibility** is builder's

> **Cross-Reference**: For deeper exploration of the builder pattern, see [Chapter 6: Builder Pattern Deep Dive](../part2-core-patterns/06-builder-pattern-deep-dive.md). For advanced derive techniques, see [Chapter 7: Derive Macro Mastery](../part2-core-patterns/07-derive-macro-mastery.md).

---

*Next: [Type System Integration](./03-type-system-integration.md)*
