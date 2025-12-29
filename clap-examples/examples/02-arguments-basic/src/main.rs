//! # Example 02: Basic Arguments
//!
//! Demonstrates all common argument types and modifiers.
//!
//! ## Run Examples:
//! ```bash
//! # Minimal required arguments
//! cargo run -p arguments-basic -- input.txt --output output.txt
//!
//! # With all options
//! cargo run -p arguments-basic -- input.txt \
//!     --output output.txt \
//!     --count 5 \
//!     --ratio 0.75 \
//!     --verbose \
//!     --tags rust,cli,clap \
//!     --config ~/.config/app.toml
//!
//! # Using short flags
//! cargo run -p arguments-basic -- input.txt -o output.txt -c 10 -v
//!
//! # Multiple verbose flags (increases verbosity level)
//! cargo run -p arguments-basic -- input.txt -o output.txt -vvv
//!
//! # Multiple input files
//! cargo run -p arguments-basic -- file1.txt file2.txt file3.txt -o output.txt
//! ```

use clap::{Parser, ValueHint};
use std::path::PathBuf;

/// A comprehensive example of Clap argument types.
///
/// This program demonstrates integers, strings, booleans, floats,
/// paths, optional values, vectors, and various argument modifiers.
#[derive(Parser, Debug)]
#[command(name = "args-demo")]
#[command(version, about, long_about = None)]
struct Args {
    // =========================================================================
    // POSITIONAL ARGUMENTS
    // =========================================================================

    /// Input file(s) to process (at least one required)
    ///
    /// Multiple files can be specified as positional arguments.
    #[arg(value_hint = ValueHint::FilePath)]
    input: Vec<PathBuf>,

    // =========================================================================
    // REQUIRED OPTIONS
    // =========================================================================

    /// Output file path (required)
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    output: PathBuf,

    // =========================================================================
    // INTEGER ARGUMENTS
    // =========================================================================

    /// Number of iterations (default: 1)
    #[arg(short, long, default_value_t = 1)]
    count: u32,

    /// Port number (range validated: 1-65535)
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    port: Option<u16>,

    // =========================================================================
    // FLOATING POINT ARGUMENTS
    // =========================================================================

    /// Processing ratio (0.0 to 1.0)
    #[arg(short, long, default_value_t = 1.0)]
    ratio: f64,

    /// Threshold value (optional)
    #[arg(long)]
    threshold: Option<f64>,

    // =========================================================================
    // BOOLEAN FLAGS
    // =========================================================================

    /// Enable verbose output
    ///
    /// Can be specified multiple times to increase verbosity:
    /// -v (info), -vv (debug), -vvv (trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Run in quiet mode (no output)
    #[arg(short, long)]
    quiet: bool,

    /// Enable debug mode
    #[arg(long)]
    debug: bool,

    // =========================================================================
    // STRING ARGUMENTS
    // =========================================================================

    /// Output format
    #[arg(short, long, default_value = "json")]
    format: String,

    /// Custom message (optional)
    #[arg(short, long)]
    message: Option<String>,

    // =========================================================================
    // PATH ARGUMENTS
    // =========================================================================

    /// Configuration file path
    #[arg(long, value_hint = ValueHint::FilePath)]
    config: Option<PathBuf>,

    /// Working directory
    #[arg(long, value_hint = ValueHint::DirPath)]
    workdir: Option<PathBuf>,

    // =========================================================================
    // VECTOR/LIST ARGUMENTS
    // =========================================================================

    /// Tags (comma-separated or multiple --tag flags)
    ///
    /// Examples: --tags a,b,c or --tags a --tags b --tags c
    #[arg(long, value_delimiter = ',')]
    tags: Vec<String>,

    /// Include patterns (can be specified multiple times)
    #[arg(short = 'I', long = "include", action = clap::ArgAction::Append)]
    includes: Vec<String>,

    /// Exclude patterns (can be specified multiple times)
    #[arg(short = 'E', long = "exclude", action = clap::ArgAction::Append)]
    excludes: Vec<String>,

    // =========================================================================
    // ARGUMENTS WITH VALUE NAMES
    // =========================================================================

    /// Set a key=value pair
    #[arg(long, value_name = "KEY=VALUE")]
    set: Vec<String>,

    /// Number of parallel jobs
    #[arg(short = 'j', long, value_name = "N")]
    jobs: Option<usize>,
}

fn main() {
    let args = Args::parse();

    // Display parsed arguments
    println!("=== Parsed Arguments ===\n");

    // Positional arguments
    println!("Input files: {:?}", args.input);
    println!("Output file: {:?}", args.output);

    // Integers
    println!("\n--- Integers ---");
    println!("Count: {}", args.count);
    if let Some(port) = args.port {
        println!("Port: {}", port);
    }

    // Floats
    println!("\n--- Floats ---");
    println!("Ratio: {}", args.ratio);
    if let Some(threshold) = args.threshold {
        println!("Threshold: {}", threshold);
    }

    // Booleans
    println!("\n--- Booleans ---");
    println!("Verbose level: {} ({})", args.verbose, verbosity_name(args.verbose));
    println!("Quiet mode: {}", args.quiet);
    println!("Debug mode: {}", args.debug);

    // Strings
    println!("\n--- Strings ---");
    println!("Format: {}", args.format);
    if let Some(msg) = &args.message {
        println!("Message: {}", msg);
    }

    // Paths
    println!("\n--- Paths ---");
    if let Some(config) = &args.config {
        println!("Config: {:?}", config);
    }
    if let Some(workdir) = &args.workdir {
        println!("Working directory: {:?}", workdir);
    }

    // Vectors
    println!("\n--- Vectors ---");
    if !args.tags.is_empty() {
        println!("Tags: {:?}", args.tags);
    }
    if !args.includes.is_empty() {
        println!("Includes: {:?}", args.includes);
    }
    if !args.excludes.is_empty() {
        println!("Excludes: {:?}", args.excludes);
    }

    // Key-value pairs
    if !args.set.is_empty() {
        println!("\n--- Key-Value Pairs ---");
        for kv in &args.set {
            println!("  {}", kv);
        }
    }

    // Jobs
    if let Some(jobs) = args.jobs {
        println!("\nParallel jobs: {}", jobs);
    }
}

fn verbosity_name(level: u8) -> &'static str {
    match level {
        0 => "normal",
        1 => "info",
        2 => "debug",
        _ => "trace",
    }
}

// =============================================================================
// KEY CONCEPTS:
// =============================================================================
//
// 1. ARGUMENT TYPES:
//    - Positional: No flag needed, position matters
//    - Optional: Use Option<T> for truly optional values
//    - Required: Non-Option types without defaults are required
//    - Flags: Boolean switches (--verbose, --debug)
//
// 2. VALUE PARSING:
//    - Strings: Parsed directly
//    - Integers: u8, u16, u32, u64, i8, i16, i32, i64
//    - Floats: f32, f64
//    - Paths: PathBuf with ValueHint for shell completion
//    - Vectors: Multiple values with action = Append or value_delimiter
//
// 3. ARGUMENT MODIFIERS:
//    - default_value: String default
//    - default_value_t: Default for types implementing Display
//    - value_parser: Custom parsing/validation
//    - value_hint: Shell completion hints
//    - value_name: Display name in help
//    - value_delimiter: Split single value into multiple
//
// 4. ACTIONS:
//    - Set (default): Replace value
//    - Append: Collect multiple values
//    - Count: Count occurrences (useful for verbosity)
//    - SetTrue/SetFalse: Boolean flags
//
// 5. VALIDATION:
//    - Range validation: value_parser!(u16).range(1..)
//    - Pattern validation: Use custom ValueParser (see example 05)
//    - Required vs optional enforced by type system
