//! # Example 04: Groups and Conflicts
//!
//! Demonstrates argument groups, mutual exclusivity, required groups,
//! and complex argument relationships.
//!
//! ## Run Examples:
//! ```bash
//! # Basic usage with file input
//! cargo run -p groups-and-conflicts -- --input file.txt
//!
//! # Using stdin instead
//! cargo run -p groups-and-conflicts -- --stdin
//!
//! # ERROR: Cannot use both --input and --stdin (mutually exclusive)
//! cargo run -p groups-and-conflicts -- --input file.txt --stdin
//!
//! # Using URL source
//! cargo run -p groups-and-conflicts -- --url https://example.com
//!
//! # Output options (at least one required)
//! cargo run -p groups-and-conflicts -- --input file.txt --json
//! cargo run -p groups-and-conflicts -- --input file.txt --yaml --pretty
//!
//! # Mutually exclusive output formats
//! cargo run -p groups-and-conflicts -- --input file.txt --json --yaml  # ERROR
//!
//! # Authentication (must provide either both user/pass OR token)
//! cargo run -p groups-and-conflicts -- --input file.txt --json \
//!     --user admin --password secret
//!
//! # Or use token authentication
//! cargo run -p groups-and-conflicts -- --input file.txt --json \
//!     --token abc123
//!
//! # ERROR: Cannot mix auth methods
//! cargo run -p groups-and-conflicts -- --input file.txt --json \
//!     --user admin --token abc123
//!
//! # Conditional requirements: --dry-run requires --force
//! cargo run -p groups-and-conflicts -- --input file.txt --json --force --dry-run
//!
//! # View detailed help
//! cargo run -p groups-and-conflicts -- --help
//! ```

use clap::{Parser, ArgGroup, Args};

/// Demonstrates argument groups, conflicts, and complex relationships.
///
/// This example shows how to enforce rules like:
/// - Mutually exclusive options (use --input OR --stdin, not both)
/// - Required groups (at least one output format must be specified)
/// - Conditional requirements (--dry-run requires --force)
/// - Related options (--user and --password go together)
#[derive(Parser, Debug)]
#[command(name = "processor")]
#[command(version, about, long_about = None)]
// Define argument groups
#[command(group(
    ArgGroup::new("input_source")
        .required(true)
        .args(["input", "stdin", "url"]),
))]
#[command(group(
    ArgGroup::new("output_format")
        .required(true)
        .args(["json", "yaml", "xml", "csv"]),
))]
#[command(group(
    ArgGroup::new("auth_basic")
        .args(["user", "password"])
        .requires_all(["user", "password"])
        .conflicts_with("auth_token"),
))]
#[command(group(
    ArgGroup::new("auth_token")
        .args(["token"])
        .conflicts_with("auth_basic"),
))]
struct Cli {
    // =========================================================================
    // INPUT SOURCE GROUP (mutually exclusive, one required)
    // =========================================================================

    /// Read from input file
    #[arg(short, long)]
    input: Option<String>,

    /// Read from standard input
    #[arg(long)]
    stdin: bool,

    /// Read from URL
    #[arg(long)]
    url: Option<String>,

    // =========================================================================
    // OUTPUT FORMAT GROUP (mutually exclusive, one required)
    // =========================================================================

    /// Output as JSON
    #[arg(long)]
    json: bool,

    /// Output as YAML
    #[arg(long)]
    yaml: bool,

    /// Output as XML
    #[arg(long)]
    xml: bool,

    /// Output as CSV
    #[arg(long)]
    csv: bool,

    // =========================================================================
    // OUTPUT MODIFIERS (can be combined)
    // =========================================================================

    /// Pretty print output
    #[arg(long)]
    pretty: bool,

    /// Include metadata in output
    #[arg(long)]
    metadata: bool,

    /// Write output to file (default: stdout)
    #[arg(short, long)]
    output: Option<String>,

    // =========================================================================
    // AUTHENTICATION (user+password OR token, not both)
    // =========================================================================

    /// Username for basic auth
    #[arg(short, long)]
    user: Option<String>,

    /// Password for basic auth
    #[arg(short, long)]
    password: Option<String>,

    /// Authentication token
    #[arg(long)]
    token: Option<String>,

    // =========================================================================
    // CONDITIONAL REQUIREMENTS
    // =========================================================================

    /// Force operation (required for --dry-run)
    #[arg(long)]
    force: bool,

    /// Dry run mode (requires --force)
    #[arg(long, requires = "force")]
    dry_run: bool,

    // =========================================================================
    // CONFLICTS WITHOUT GROUPS
    // =========================================================================

    /// Verbose output
    #[arg(short, long, conflicts_with = "quiet")]
    verbose: bool,

    /// Quiet mode (no output except errors)
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,

    // =========================================================================
    // NESTED ARGUMENT GROUPS (via Args derive)
    // =========================================================================

    #[command(flatten)]
    advanced: AdvancedOptions,
}

/// Advanced options that are grouped together.
///
/// Using #[command(flatten)] includes these in the parent struct
/// while keeping them organized in the code.
#[derive(Args, Debug)]
#[command(next_help_heading = "Advanced Options")]
struct AdvancedOptions {
    /// Maximum retries
    #[arg(long, default_value_t = 3)]
    retries: u32,

    /// Timeout in seconds
    #[arg(long, default_value_t = 30)]
    timeout: u64,

    /// Enable compression
    #[arg(long)]
    compress: bool,

    /// Compression level (1-9, requires --compress)
    #[arg(long, requires = "compress", value_parser = clap::value_parser!(u8).range(1..=9))]
    compression_level: Option<u8>,
}

fn main() {
    let cli = Cli::parse();

    println!("=== Configuration ===\n");

    // Display input source
    println!("Input Source:");
    if let Some(file) = &cli.input {
        println!("  File: {}", file);
    } else if cli.stdin {
        println!("  Standard input");
    } else if let Some(url) = &cli.url {
        println!("  URL: {}", url);
    }

    // Display output format
    println!("\nOutput Format:");
    let format = if cli.json { "JSON" }
        else if cli.yaml { "YAML" }
        else if cli.xml { "XML" }
        else { "CSV" };
    println!("  Format: {}", format);
    println!("  Pretty: {}", cli.pretty);
    println!("  Metadata: {}", cli.metadata);
    if let Some(output) = &cli.output {
        println!("  Output file: {}", output);
    }

    // Display authentication
    println!("\nAuthentication:");
    if let Some(user) = &cli.user {
        println!("  Basic auth: {}:***", user);
    } else if let Some(_) = &cli.token {
        println!("  Token auth: ***");
    } else {
        println!("  None (anonymous)");
    }

    // Display flags
    println!("\nFlags:");
    println!("  Force: {}", cli.force);
    println!("  Dry run: {}", cli.dry_run);
    println!("  Verbose: {}", cli.verbose);
    println!("  Quiet: {}", cli.quiet);

    // Display advanced options
    println!("\nAdvanced Options:");
    println!("  Retries: {}", cli.advanced.retries);
    println!("  Timeout: {}s", cli.advanced.timeout);
    println!("  Compression: {}", cli.advanced.compress);
    if let Some(level) = cli.advanced.compression_level {
        println!("  Compression level: {}", level);
    }
}

// =============================================================================
// KEY CONCEPTS:
// =============================================================================
//
// 1. ARGUMENT GROUPS:
//    ArgGroup::new("name") creates a named group of arguments.
//    Groups can be required, mutually exclusive, or have relationships.
//
// 2. MUTUALLY EXCLUSIVE OPTIONS:
//    By default, groups are mutually exclusive (only one can be used).
//    #[arg(conflicts_with = "other")] for simple two-way conflicts.
//
// 3. REQUIRED GROUPS:
//    .required(true) means at least one arg in the group must be provided.
//    Useful for "choose one of these options" patterns.
//
// 4. REQUIRES:
//    #[arg(requires = "other")] - this arg requires another arg
//    .requires_all([...]) - group requires multiple args together
//
// 5. CONFLICTS:
//    #[arg(conflicts_with = "other")] - cannot use both
//    .conflicts_with("group") - group conflicts with another group
//
// 6. FLATTENED ARGS:
//    #[command(flatten)] includes Args struct fields in parent
//    Useful for organizing related options and code reuse
//
// 7. HELP HEADINGS:
//    #[command(next_help_heading = "Section")] groups args in help
//    Makes --help output more readable for complex CLIs
//
// 8. VALIDATION:
//    Clap validates all relationships at parse time
//    Provides clear error messages when rules are violated
//
// COMMON PATTERNS:
//
// a) "One of many" (--format json|yaml|xml):
//    ArgGroup::new("format").required(true).args([...])
//
// b) "All or nothing" (--user AND --password):
//    ArgGroup with requires_all
//
// c) "If A then B" (--dry-run requires --force):
//    #[arg(requires = "force")]
//
// d) "A or B, not both" (--verbose vs --quiet):
//    #[arg(conflicts_with = "other")]
