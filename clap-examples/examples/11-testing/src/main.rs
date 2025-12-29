//! # Example 11: Testing CLI Applications
//!
//! Demonstrates unit testing, integration testing, and best practices
//! for testing command-line applications with Clap.
//!
//! ## Run Examples:
//! ```bash
//! # Run the CLI
//! cargo run -p testing -- hello World
//! cargo run -p testing -- add 1 2 3 4 5
//! cargo run -p testing -- greet Alice --greeting "Good morning"
//!
//! # Run tests
//! cargo test -p testing
//!
//! # Run tests with output
//! cargo test -p testing -- --nocapture
//!
//! # Run specific test
//! cargo test -p testing test_hello
//! ```

use clap::{Parser, Subcommand};

// =============================================================================
// CLI DEFINITION
// =============================================================================

/// A testable CLI application demonstrating testing patterns.
#[derive(Parser, Debug)]
#[command(name = "testable-cli")]
#[command(version, about)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Say hello to someone
    Hello {
        /// Name to greet
        name: String,
    },

    /// Add numbers together
    Add {
        /// Numbers to add
        #[arg(required = true)]
        numbers: Vec<i64>,
    },

    /// Greet with a custom message
    Greet {
        /// Name to greet
        name: String,

        /// Custom greeting
        #[arg(short, long, default_value = "Hello")]
        greeting: String,

        /// Number of times to greet
        #[arg(short, long, default_value_t = 1)]
        count: u32,
    },

    /// Process a file
    Process {
        /// Input file
        input: String,

        /// Output file
        #[arg(short, long)]
        output: Option<String>,

        /// Processing mode
        #[arg(short, long, value_parser = ["fast", "accurate", "balanced"])]
        mode: Option<String>,
    },
}

// =============================================================================
// BUSINESS LOGIC (Separated for testing)
// =============================================================================

/// Result of executing a command.
#[derive(Debug, PartialEq)]
pub struct CommandResult {
    pub output: String,
    pub exit_code: i32,
}

impl CommandResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            exit_code: 0,
        }
    }

    pub fn error(output: impl Into<String>, code: i32) -> Self {
        Self {
            output: output.into(),
            exit_code: code,
        }
    }
}

/// Execute the hello command.
pub fn execute_hello(name: &str, verbose: bool) -> CommandResult {
    if name.is_empty() {
        return CommandResult::error("Name cannot be empty", 1);
    }

    let output = if verbose {
        format!("[verbose] Greeting: {}\nHello, {}!", name, name)
    } else {
        format!("Hello, {}!", name)
    };

    CommandResult::success(output)
}

/// Execute the add command.
pub fn execute_add(numbers: &[i64], verbose: bool) -> CommandResult {
    if numbers.is_empty() {
        return CommandResult::error("No numbers provided", 1);
    }

    let sum: i64 = numbers.iter().sum();

    let output = if verbose {
        let nums_str = numbers
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(" + ");
        format!("[verbose] Calculating: {}\nSum: {}", nums_str, sum)
    } else {
        format!("Sum: {}", sum)
    };

    CommandResult::success(output)
}

/// Execute the greet command.
pub fn execute_greet(name: &str, greeting: &str, count: u32, verbose: bool) -> CommandResult {
    if name.is_empty() {
        return CommandResult::error("Name cannot be empty", 1);
    }

    if count == 0 {
        return CommandResult::error("Count must be at least 1", 1);
    }

    let mut lines = Vec::new();

    if verbose {
        lines.push(format!("[verbose] Greeting {} {} times", name, count));
    }

    for i in 0..count {
        if verbose && count > 1 {
            lines.push(format!("[{}] {}, {}!", i + 1, greeting, name));
        } else {
            lines.push(format!("{}, {}!", greeting, name));
        }
    }

    CommandResult::success(lines.join("\n"))
}

/// Execute the process command.
pub fn execute_process(
    input: &str,
    output: Option<&str>,
    mode: Option<&str>,
    verbose: bool,
) -> CommandResult {
    if input.is_empty() {
        return CommandResult::error("Input file required", 1);
    }

    let mode = mode.unwrap_or("balanced");
    let default_output = format!("{}.out", input);
    let output_file = output.unwrap_or(&default_output);

    let mut lines = Vec::new();

    if verbose {
        lines.push(format!("[verbose] Processing: {}", input));
        lines.push(format!("[verbose] Mode: {}", mode));
    }

    lines.push(format!("Processed {} -> {} (mode: {})", input, output_file, mode));

    CommandResult::success(lines.join("\n"))
}

/// Execute CLI based on parsed arguments.
pub fn execute(cli: &Cli) -> CommandResult {
    match &cli.command {
        Commands::Hello { name } => execute_hello(name, cli.verbose),
        Commands::Add { numbers } => execute_add(numbers, cli.verbose),
        Commands::Greet { name, greeting, count } => {
            execute_greet(name, greeting, *count, cli.verbose)
        }
        Commands::Process { input, output, mode } => {
            execute_process(input, output.as_deref(), mode.as_deref(), cli.verbose)
        }
    }
}

// =============================================================================
// MAIN
// =============================================================================

fn main() {
    let cli = Cli::parse();
    let result = execute(&cli);

    println!("{}", result.output);
    std::process::exit(result.exit_code);
}

// =============================================================================
// UNIT TESTS
// =============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Hello Command Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_hello_basic() {
        let result = execute_hello("World", false);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output, "Hello, World!");
    }

    #[test]
    fn test_hello_verbose() {
        let result = execute_hello("Alice", true);
        assert_eq!(result.exit_code, 0);
        assert!(result.output.contains("[verbose]"));
        assert!(result.output.contains("Hello, Alice!"));
    }

    #[test]
    fn test_hello_empty_name() {
        let result = execute_hello("", false);
        assert_eq!(result.exit_code, 1);
        assert!(result.output.contains("cannot be empty"));
    }

    // -------------------------------------------------------------------------
    // Add Command Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_add_single_number() {
        let result = execute_add(&[42], false);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output, "Sum: 42");
    }

    #[test]
    fn test_add_multiple_numbers() {
        let result = execute_add(&[1, 2, 3, 4, 5], false);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output, "Sum: 15");
    }

    #[test]
    fn test_add_negative_numbers() {
        let result = execute_add(&[-10, 5, -3], false);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output, "Sum: -8");
    }

    #[test]
    fn test_add_empty() {
        let result = execute_add(&[], false);
        assert_eq!(result.exit_code, 1);
        assert!(result.output.contains("No numbers"));
    }

    #[test]
    fn test_add_verbose() {
        let result = execute_add(&[1, 2, 3], true);
        assert!(result.output.contains("1 + 2 + 3"));
        assert!(result.output.contains("Sum: 6"));
    }

    // -------------------------------------------------------------------------
    // Greet Command Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_greet_basic() {
        let result = execute_greet("Bob", "Hello", 1, false);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output, "Hello, Bob!");
    }

    #[test]
    fn test_greet_custom_greeting() {
        let result = execute_greet("Alice", "Good morning", 1, false);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output, "Good morning, Alice!");
    }

    #[test]
    fn test_greet_multiple_times() {
        let result = execute_greet("Charlie", "Hi", 3, false);
        assert_eq!(result.exit_code, 0);
        let lines: Vec<_> = result.output.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines.iter().all(|l| l.contains("Hi, Charlie!")));
    }

    #[test]
    fn test_greet_zero_count() {
        let result = execute_greet("David", "Hello", 0, false);
        assert_eq!(result.exit_code, 1);
    }

    // -------------------------------------------------------------------------
    // Process Command Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_process_basic() {
        let result = execute_process("input.txt", None, None, false);
        assert_eq!(result.exit_code, 0);
        assert!(result.output.contains("input.txt"));
        assert!(result.output.contains("balanced"));
    }

    #[test]
    fn test_process_with_output() {
        let result = execute_process("data.csv", Some("result.csv"), None, false);
        assert!(result.output.contains("result.csv"));
    }

    #[test]
    fn test_process_with_mode() {
        let result = execute_process("data.csv", None, Some("fast"), false);
        assert!(result.output.contains("fast"));
    }

    // -------------------------------------------------------------------------
    // CLI Parsing Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_hello() {
        let cli = Cli::try_parse_from(["test", "hello", "World"]).unwrap();
        match cli.command {
            Commands::Hello { name } => assert_eq!(name, "World"),
            _ => panic!("Expected Hello command"),
        }
    }

    #[test]
    fn test_parse_add() {
        let cli = Cli::try_parse_from(["test", "add", "1", "2", "3"]).unwrap();
        match cli.command {
            Commands::Add { numbers } => assert_eq!(numbers, vec![1, 2, 3]),
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_parse_greet_with_options() {
        let cli = Cli::try_parse_from([
            "test", "greet", "Alice",
            "--greeting", "Good morning",
            "--count", "5",
        ]).unwrap();

        match cli.command {
            Commands::Greet { name, greeting, count } => {
                assert_eq!(name, "Alice");
                assert_eq!(greeting, "Good morning");
                assert_eq!(count, 5);
            }
            _ => panic!("Expected Greet command"),
        }
    }

    #[test]
    fn test_parse_verbose_flag() {
        let cli = Cli::try_parse_from(["test", "-v", "hello", "World"]).unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn test_parse_invalid_command() {
        let result = Cli::try_parse_from(["test", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_required_arg() {
        let result = Cli::try_parse_from(["test", "hello"]);
        assert!(result.is_err());
    }
}

// =============================================================================
// INTEGRATION TESTS (in tests/ directory typically)
// =============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test full command execution flow.
    #[test]
    fn test_full_hello_flow() {
        let cli = Cli::try_parse_from(["test", "hello", "Integration"]).unwrap();
        let result = execute(&cli);

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output, "Hello, Integration!");
    }

    #[test]
    fn test_full_add_flow() {
        let cli = Cli::try_parse_from(["test", "add", "10", "20", "30"]).unwrap();
        let result = execute(&cli);

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output, "Sum: 60");
    }

    #[test]
    fn test_full_greet_flow_verbose() {
        let cli = Cli::try_parse_from([
            "test", "-v", "greet", "Test",
            "--greeting", "Welcome",
            "--count", "2",
        ]).unwrap();

        let result = execute(&cli);

        assert_eq!(result.exit_code, 0);
        assert!(result.output.contains("[verbose]"));
        assert!(result.output.contains("Welcome, Test!"));
    }

    /// Test error handling.
    #[test]
    fn test_error_handling() {
        // Empty name should fail
        let result = execute_hello("", false);
        assert_ne!(result.exit_code, 0);
    }
}

// =============================================================================
// KEY CONCEPTS:
// =============================================================================
//
// 1. SEPARATION OF CONCERNS:
//    - CLI parsing in struct (Cli, Commands)
//    - Business logic in functions (execute_*)
//    - Easy to test each layer independently
//
// 2. TESTABLE DESIGN:
//    - Return results instead of printing directly
//    - Accept dependencies as parameters
//    - Use Result/Option for error handling
//
// 3. CLI PARSING TESTS:
//    - Use try_parse_from() for testing
//    - Test valid argument combinations
//    - Test error cases (missing args, invalid values)
//    - Test flag behavior
//
// 4. UNIT TESTS:
//    - Test individual functions
//    - Test edge cases (empty input, zero values)
//    - Test error conditions
//    - Test verbose mode
//
// 5. INTEGRATION TESTS:
//    - Test full command flow
//    - Test argument parsing -> execution -> result
//    - Use assert_cmd for binary testing (see tests/)
//
// TESTING PATTERNS:
//
// a) Parse testing:
//    Cli::try_parse_from(["program", "arg1", "arg2"])
//
// b) Result assertion:
//    assert_eq!(result.exit_code, 0);
//    assert!(result.output.contains("expected"));
//
// c) Error testing:
//    assert!(Cli::try_parse_from([...]).is_err());
//
// d) Binary testing (with assert_cmd):
//    Command::cargo_bin("testing")?
//        .arg("hello").arg("World")
//        .assert()
//        .success()
//        .stdout(predicate::str::contains("Hello, World!"));
//
// BEST PRACTICES:
//
// - Separate parsing from execution
// - Return structured results, not just strings
// - Test both success and error cases
// - Use descriptive test names
// - Group related tests in modules
// - Test edge cases explicitly
