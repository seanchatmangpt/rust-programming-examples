//! # Integration Tests for CLI
//!
//! These tests run the actual binary and verify its behavior.
//!
//! Run with: cargo test -p testing --test integration

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to get the command
fn cmd() -> Command {
    Command::cargo_bin("testing").unwrap()
}

// =============================================================================
// HELLO COMMAND TESTS
// =============================================================================

#[test]
fn test_hello_world() {
    cmd()
        .args(["hello", "World"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"));
}

#[test]
fn test_hello_with_name() {
    cmd()
        .args(["hello", "Alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, Alice!"));
}

#[test]
fn test_hello_verbose() {
    cmd()
        .args(["-v", "hello", "Bob"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[verbose]"))
        .stdout(predicate::str::contains("Hello, Bob!"));
}

// =============================================================================
// ADD COMMAND TESTS
// =============================================================================

#[test]
fn test_add_numbers() {
    cmd()
        .args(["add", "1", "2", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Sum: 6"));
}

#[test]
fn test_add_single_number() {
    cmd()
        .args(["add", "42"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Sum: 42"));
}

#[test]
fn test_add_negative_numbers() {
    cmd()
        .args(["add", "-5", "10", "-3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Sum: 2"));
}

#[test]
fn test_add_verbose() {
    cmd()
        .args(["-v", "add", "1", "2", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 + 2 + 3"))
        .stdout(predicate::str::contains("Sum: 6"));
}

// =============================================================================
// GREET COMMAND TESTS
// =============================================================================

#[test]
fn test_greet_default() {
    cmd()
        .args(["greet", "Charlie"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, Charlie!"));
}

#[test]
fn test_greet_custom_greeting() {
    cmd()
        .args(["greet", "Diana", "--greeting", "Good evening"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Good evening, Diana!"));
}

#[test]
fn test_greet_multiple_times() {
    cmd()
        .args(["greet", "Eve", "--count", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, Eve!"));
}

#[test]
fn test_greet_short_flags() {
    cmd()
        .args(["greet", "Frank", "-g", "Hi", "-c", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Frank!"));
}

// =============================================================================
// PROCESS COMMAND TESTS
// =============================================================================

#[test]
fn test_process_basic() {
    cmd()
        .args(["process", "input.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Processed input.txt"))
        .stdout(predicate::str::contains("balanced"));
}

#[test]
fn test_process_with_output() {
    cmd()
        .args(["process", "data.csv", "-o", "result.csv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("result.csv"));
}

#[test]
fn test_process_with_mode() {
    cmd()
        .args(["process", "data.csv", "-m", "fast"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fast"));
}

// =============================================================================
// ERROR HANDLING TESTS
// =============================================================================

#[test]
fn test_missing_command() {
    cmd()
        .assert()
        .failure();
}

#[test]
fn test_invalid_command() {
    cmd()
        .args(["invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_missing_required_arg() {
    cmd()
        .args(["hello"])
        .assert()
        .failure();
}

#[test]
fn test_add_no_numbers() {
    cmd()
        .args(["add"])
        .assert()
        .failure();
}

// =============================================================================
// HELP AND VERSION TESTS
// =============================================================================

#[test]
fn test_help() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("testable-cli"))
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("greet"));
}

#[test]
fn test_version() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("testable-cli"));
}

#[test]
fn test_subcommand_help() {
    cmd()
        .args(["hello", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Say hello"));
}

// =============================================================================
// KEY CONCEPTS:
// =============================================================================
//
// 1. ASSERT_CMD CRATE:
//    Provides Command::cargo_bin() to run your binary
//    .args() to pass arguments
//    .assert() for fluent assertions
//
// 2. PREDICATES CRATE:
//    predicate::str::contains() - check substring
//    predicate::str::starts_with() - check prefix
//    predicate::eq() - exact match
//
// 3. ASSERTION METHODS:
//    .success() - exit code 0
//    .failure() - non-zero exit code
//    .code(N) - specific exit code
//    .stdout(predicate) - check stdout
//    .stderr(predicate) - check stderr
//
// 4. TEST ORGANIZATION:
//    Group tests by command/feature
//    Test both success and error cases
//    Test help and version output
//
// BEST PRACTICES:
//
// - Use descriptive test names
// - Test edge cases and error conditions
// - Verify both stdout and stderr when appropriate
// - Test command aliases and short flags
// - Keep tests independent (no shared state)
