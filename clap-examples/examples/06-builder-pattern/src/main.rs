//! # Example 06: Builder Pattern
//!
//! Demonstrates dynamic CLI construction using the builder API,
//! reusable argument components, and runtime CLI configuration.
//!
//! ## Run Examples:
//! ```bash
//! # View help
//! cargo run -p builder-pattern -- --help
//!
//! # Basic usage
//! cargo run -p builder-pattern -- file.txt --output result.json
//!
//! # Using subcommands
//! cargo run -p builder-pattern -- process file.txt --format json
//! cargo run -p builder-pattern -- convert input.csv --to json --output output.json
//! cargo run -p builder-pattern -- analyze data.log --depth 3
//!
//! # Using global options
//! cargo run -p builder-pattern -- --verbose process file.txt
//! cargo run -p builder-pattern -- --config app.toml analyze data.log
//! ```
//!
//! ## Why Builder Pattern?
//!
//! Use the builder pattern when:
//! - CLI structure is determined at runtime
//! - Arguments are loaded from configuration files
//! - Building plugin systems with dynamic commands
//! - Creating reusable argument components
//! - Testing CLI behavior programmatically

use clap::{Arg, ArgAction, Command, value_parser};

// =============================================================================
// REUSABLE ARGUMENT BUILDERS
// =============================================================================

/// Creates a standard verbose flag that can be reused across commands.
fn verbose_arg() -> Arg {
    Arg::new("verbose")
        .short('v')
        .long("verbose")
        .help("Enable verbose output")
        .action(ArgAction::Count)
        .global(true)
}

/// Creates a standard quiet flag.
fn quiet_arg() -> Arg {
    Arg::new("quiet")
        .short('q')
        .long("quiet")
        .help("Suppress non-essential output")
        .action(ArgAction::SetTrue)
        .global(true)
        .conflicts_with("verbose")
}

/// Creates a standard config file argument.
fn config_arg() -> Arg {
    Arg::new("config")
        .short('c')
        .long("config")
        .help("Configuration file path")
        .value_name("FILE")
        .global(true)
}

/// Creates a standard output file argument.
fn output_arg() -> Arg {
    Arg::new("output")
        .short('o')
        .long("output")
        .help("Output file path")
        .value_name("FILE")
}

/// Creates an input file positional argument.
fn input_arg() -> Arg {
    Arg::new("input")
        .help("Input file path")
        .required(true)
        .index(1)
}

/// Creates a format selection argument.
fn format_arg() -> Arg {
    Arg::new("format")
        .short('f')
        .long("format")
        .help("Output format")
        .value_name("FORMAT")
        .value_parser(["json", "yaml", "xml", "csv"])
        .default_value("json")
}

// =============================================================================
// REUSABLE COMMAND BUILDERS
// =============================================================================

/// Creates a common set of global arguments for any command.
fn add_global_args(cmd: Command) -> Command {
    cmd.arg(verbose_arg())
       .arg(quiet_arg())
       .arg(config_arg())
}

/// Creates a subcommand for processing files.
fn process_command() -> Command {
    Command::new("process")
        .about("Process input files")
        .arg(input_arg())
        .arg(format_arg())
        .arg(output_arg())
        .arg(
            Arg::new("validate")
                .long("validate")
                .help("Validate input before processing")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("parallel")
                .short('p')
                .long("parallel")
                .help("Number of parallel workers")
                .value_name("N")
                .value_parser(value_parser!(usize))
                .default_value("1")
        )
}

/// Creates a subcommand for converting between formats.
fn convert_command() -> Command {
    Command::new("convert")
        .about("Convert between file formats")
        .arg(input_arg())
        .arg(
            Arg::new("from")
                .long("from")
                .help("Source format (auto-detected if not specified)")
                .value_name("FORMAT")
                .value_parser(["json", "yaml", "xml", "csv"])
        )
        .arg(
            Arg::new("to")
                .long("to")
                .help("Target format")
                .value_name("FORMAT")
                .value_parser(["json", "yaml", "xml", "csv"])
                .required(true)
        )
        .arg(output_arg())
        .arg(
            Arg::new("pretty")
                .long("pretty")
                .help("Pretty-print output")
                .action(ArgAction::SetTrue)
        )
}

/// Creates a subcommand for analyzing data.
fn analyze_command() -> Command {
    Command::new("analyze")
        .about("Analyze data files")
        .arg(input_arg())
        .arg(
            Arg::new("depth")
                .short('d')
                .long("depth")
                .help("Analysis depth (1-10)")
                .value_name("LEVEL")
                .value_parser(value_parser!(u8).range(1..=10))
                .default_value("5")
        )
        .arg(
            Arg::new("metrics")
                .short('m')
                .long("metrics")
                .help("Metrics to compute")
                .value_name("METRIC")
                .value_parser(["all", "count", "sum", "avg", "min", "max"])
                .action(ArgAction::Append)
        )
        .arg(output_arg())
}

// =============================================================================
// DYNAMIC COMMAND BUILDER
// =============================================================================

/// Build CLI dynamically based on configuration.
///
/// This demonstrates how you might load command definitions from a config file
/// or plugin system.
fn build_dynamic_command(name: &str, description: &str, args: Vec<(&str, &str, bool)>) -> Command {
    let mut cmd = Command::new(name.to_string())
        .about(description.to_string());

    for (arg_name, arg_help, required) in args {
        let arg_name_owned = arg_name.to_string();
        let mut arg = Arg::new(arg_name_owned.clone())
            .long(arg_name_owned)
            .help(arg_help.to_string());

        if required {
            arg = arg.required(true);
        }

        cmd = cmd.arg(arg);
    }

    cmd
}

// =============================================================================
// MAIN CLI BUILDER
// =============================================================================

fn build_cli() -> Command {
    let mut cmd = Command::new("builder-demo")
        .version("1.0.0")
        .author("Clap Architecture Book")
        .about("Demonstrates the builder pattern for CLI construction")
        .arg_required_else_help(true);

    // Add global arguments
    cmd = add_global_args(cmd);

    // Add subcommands
    cmd = cmd
        .subcommand(process_command())
        .subcommand(convert_command())
        .subcommand(analyze_command());

    // Add a dynamically built command (simulating plugin/config loading)
    let dynamic_cmd = build_dynamic_command(
        "custom",
        "A dynamically configured command",
        vec![
            ("name", "Resource name", true),
            ("type", "Resource type", false),
            ("tags", "Resource tags", false),
        ]
    );
    cmd = cmd.subcommand(dynamic_cmd);

    // Add top-level file argument for simple usage
    cmd = cmd.arg(
        Arg::new("file")
            .help("Input file for simple mode (without subcommand)")
            .index(1)
    );
    cmd = cmd.arg(output_arg());

    cmd
}

// =============================================================================
// COMMAND HANDLING
// =============================================================================

fn main() {
    let matches = build_cli().get_matches();

    // Handle global flags
    let verbose: u8 = *matches.get_one::<u8>("verbose").unwrap_or(&0);
    let quiet = matches.get_flag("quiet");
    let config = matches.get_one::<String>("config");

    if verbose > 0 && !quiet {
        println!("Verbose level: {}", verbose);
    }
    if let Some(cfg) = config {
        println!("Using config: {}", cfg);
    }

    // Route to subcommand or handle top-level
    match matches.subcommand() {
        Some(("process", sub_m)) => handle_process(sub_m, verbose, quiet),
        Some(("convert", sub_m)) => handle_convert(sub_m, verbose, quiet),
        Some(("analyze", sub_m)) => handle_analyze(sub_m, verbose, quiet),
        Some(("custom", sub_m)) => handle_custom(sub_m, verbose, quiet),
        _ => {
            // Handle top-level file argument
            if let Some(file) = matches.get_one::<String>("file") {
                println!("Processing file: {}", file);
                if let Some(output) = matches.get_one::<String>("output") {
                    println!("Output to: {}", output);
                }
            } else {
                println!("No command or file specified. Use --help for usage.");
            }
        }
    }
}

fn handle_process(matches: &clap::ArgMatches, verbose: u8, quiet: bool) {
    let input = matches.get_one::<String>("input").unwrap();
    let format = matches.get_one::<String>("format").unwrap();
    let validate = matches.get_flag("validate");
    let parallel: usize = *matches.get_one::<usize>("parallel").unwrap();

    if !quiet {
        println!("=== Process Command ===");
        println!("Input: {}", input);
        println!("Format: {}", format);
        println!("Validate: {}", validate);
        println!("Parallel workers: {}", parallel);

        if let Some(output) = matches.get_one::<String>("output") {
            println!("Output: {}", output);
        }

        if verbose > 0 {
            println!("\n[verbose] Starting processing pipeline...");
        }
    }
}

fn handle_convert(matches: &clap::ArgMatches, verbose: u8, quiet: bool) {
    let input = matches.get_one::<String>("input").unwrap();
    let to_format = matches.get_one::<String>("to").unwrap();
    let from_format = matches.get_one::<String>("from");
    let pretty = matches.get_flag("pretty");

    if !quiet {
        println!("=== Convert Command ===");
        println!("Input: {}", input);
        if let Some(from) = from_format {
            println!("From: {}", from);
        } else {
            println!("From: (auto-detect)");
        }
        println!("To: {}", to_format);
        println!("Pretty: {}", pretty);

        if let Some(output) = matches.get_one::<String>("output") {
            println!("Output: {}", output);
        }

        if verbose > 0 {
            println!("\n[verbose] Detecting source format...");
        }
    }
}

fn handle_analyze(matches: &clap::ArgMatches, verbose: u8, quiet: bool) {
    let input = matches.get_one::<String>("input").unwrap();
    let depth: u8 = *matches.get_one::<u8>("depth").unwrap();
    let metrics: Vec<&String> = matches.get_many::<String>("metrics")
        .map(|v| v.collect())
        .unwrap_or_default();

    if !quiet {
        println!("=== Analyze Command ===");
        println!("Input: {}", input);
        println!("Depth: {}", depth);

        if metrics.is_empty() {
            println!("Metrics: (default)");
        } else {
            println!("Metrics: {:?}", metrics);
        }

        if let Some(output) = matches.get_one::<String>("output") {
            println!("Output: {}", output);
        }

        if verbose > 0 {
            println!("\n[verbose] Running {} analysis passes...", depth);
        }
    }
}

fn handle_custom(matches: &clap::ArgMatches, verbose: u8, quiet: bool) {
    let name = matches.get_one::<String>("name").unwrap();

    if !quiet {
        println!("=== Custom Command ===");
        println!("Name: {}", name);

        if let Some(rtype) = matches.get_one::<String>("type") {
            println!("Type: {}", rtype);
        }
        if let Some(tags) = matches.get_one::<String>("tags") {
            println!("Tags: {}", tags);
        }

        if verbose > 0 {
            println!("\n[verbose] Executing custom command...");
        }
    }
}

// =============================================================================
// KEY CONCEPTS:
// =============================================================================
//
// 1. BUILDER PATTERN BENEFITS:
//    - Dynamic CLI construction at runtime
//    - Reusable argument components
//    - Easier testing and composition
//    - Plugin system support
//    - Configuration-driven CLIs
//
// 2. REUSABLE ARGUMENTS:
//    Create functions that return Arg to share across commands.
//    fn verbose_arg() -> Arg { Arg::new("verbose")... }
//
// 3. COMMAND COMPOSITION:
//    Build complex CLIs by combining smaller pieces.
//    add_global_args(cmd).subcommand(process_command())
//
// 4. GLOBAL ARGUMENTS:
//    .global(true) makes an argument available to all subcommands.
//    Access with matches.get_one::<T>("name")
//
// 5. DYNAMIC COMMANDS:
//    Build commands from configuration or plugins.
//    Useful for extensible applications.
//
// 6. ARGMATCHES API:
//    - get_one::<T>() - get single value
//    - get_many::<T>() - get multiple values
//    - get_flag() - get boolean flag
//    - subcommand() - get matched subcommand
//
// 7. VALUE PARSERS:
//    value_parser!(type) - automatic parsing
//    value_parser!(u8).range(1..=10) - with validation
//    ["a", "b", "c"] - possible values
//
// BUILDER vs DERIVE:
//
// Use BUILDER when:
// - CLI structure changes at runtime
// - Loading commands from config files
// - Building plugin systems
// - Need maximum flexibility
//
// Use DERIVE when:
// - CLI structure is known at compile time
// - Want type-safe argument access
// - Prefer less boilerplate
// - Standard CLI patterns suffice
//
// You can also COMBINE both patterns!
