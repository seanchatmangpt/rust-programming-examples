//! # Example 12: Real-World Project - cloudctl
//!
//! A production-quality CLI application demonstrating best practices
//! for building professional command-line tools.
//!
//! ## Run Examples:
//! ```bash
//! # View help
//! cargo run -p cloudctl -- --help
//!
//! # Compute commands
//! cargo run -p cloudctl -- compute instances list
//! cargo run -p cloudctl -- compute instances create my-vm --type standard
//! cargo run -p cloudctl -- compute instances delete my-vm --force
//!
//! # Storage commands
//! cargo run -p cloudctl -- storage buckets list
//! cargo run -p cloudctl -- storage buckets create my-bucket --region us-east-1
//! cargo run -p cloudctl -- storage objects list my-bucket
//!
//! # Network commands
//! cargo run -p cloudctl -- network vpcs list
//! cargo run -p cloudctl -- network security-groups list
//!
//! # IAM commands
//! cargo run -p cloudctl -- iam users list
//! cargo run -p cloudctl -- iam roles list
//!
//! # Config management
//! cargo run -p cloudctl -- config get project
//! cargo run -p cloudctl -- config set project my-project
//!
//! # Output formats
//! cargo run -p cloudctl -- --output json compute instances list
//! cargo run -p cloudctl -- --output yaml storage buckets list
//! cargo run -p cloudctl -- --output table iam users list
//!
//! # Using profiles
//! cargo run -p cloudctl -- --profile production compute instances list
//!
//! # Verbose/Debug mode
//! cargo run -p cloudctl -- -v compute instances list
//! cargo run -p cloudctl -- --debug storage buckets list
//! ```

mod cli;
mod commands;
mod config;
mod error;
mod output;

use cli::Cli;
use clap::Parser;
use error::CloudError;

fn main() {
    // Initialize application
    if let Err(e) = run() {
        eprintln!("Error: {}", e);

        // Show suggestion if available
        if let Some(suggestion) = e.suggestion() {
            eprintln!("\nSuggestion: {}", suggestion);
        }

        // Show help hint for certain errors
        if e.show_help_hint() {
            eprintln!("\nFor more information, try '--help'");
        }

        std::process::exit(e.exit_code());
    }
}

fn run() -> Result<(), CloudError> {
    let cli = Cli::parse();

    // Setup logging based on verbosity
    if cli.debug {
        eprintln!("[debug] Debug mode enabled");
        eprintln!("[debug] Profile: {}", cli.profile.as_deref().unwrap_or("default"));
    }

    // Load configuration
    let config = config::load_config(cli.config.as_deref())?;

    if cli.verbose {
        eprintln!("[info] Using project: {}", config.project.as_deref().unwrap_or("(not set)"));
    }

    // Create output formatter
    let formatter = output::Formatter::new(cli.output.clone());

    // Create execution context
    let ctx = commands::Context {
        config,
        formatter,
        verbose: cli.verbose,
        debug: cli.debug,
        dry_run: cli.dry_run,
        profile: cli.profile.clone(),
    };

    // Execute command
    commands::execute(&cli.command, &ctx)
}
