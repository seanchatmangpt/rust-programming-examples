//! # Example 01: Hello World
//!
//! The simplest possible Clap application using derive macros.
//!
//! ## Run Examples:
//! ```bash
//! # Basic usage
//! cargo run -p hello-world -- World
//!
//! # With optional greeting
//! cargo run -p hello-world -- World --greeting "Good morning"
//!
//! # View help
//! cargo run -p hello-world -- --help
//!
//! # View version
//! cargo run -p hello-world -- --version
//! ```

use clap::Parser;

/// A simple greeting program demonstrating Clap basics.
///
/// This is the program description that appears in --help output.
/// Multiple lines are combined into a single paragraph.
#[derive(Parser, Debug)]
#[command(name = "hello")]
#[command(author = "Your Name <your@email.com>")]
#[command(version = "1.0")]
#[command(about = "A simple greeting program", long_about = None)]
struct Args {
    /// The name to greet (required positional argument)
    name: String,

    /// Optional custom greeting (defaults to "Hello")
    #[arg(short, long, default_value = "Hello")]
    greeting: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    // Parse command-line arguments into our Args struct
    let args = Args::parse();

    // Use the parsed arguments
    for _ in 0..args.count {
        println!("{}, {}!", args.greeting, args.name);
    }
}

// =============================================================================
// KEY CONCEPTS:
// =============================================================================
//
// 1. DERIVE MACRO:
//    #[derive(Parser)] generates the argument parsing code automatically.
//    The struct fields become CLI arguments.
//
// 2. POSITIONAL vs OPTIONAL ARGUMENTS:
//    - `name: String` - positional argument (required, no flag needed)
//    - `#[arg(short, long)]` - optional argument with -g/--greeting flags
//
// 3. DEFAULT VALUES:
//    - `default_value = "Hello"` - for String types
//    - `default_value_t = 1` - for types implementing Display
//
// 4. DOCUMENTATION:
//    Doc comments (///) become help text for arguments.
//    Program-level docs use #[command(...)] attributes.
//
// 5. AUTO-GENERATED FEATURES:
//    - --help/-h: Shows usage information
//    - --version/-V: Shows version from Cargo.toml or #[command(version)]
//    - Error messages for invalid input
//    - Short flags from first letter (-g, -c)
