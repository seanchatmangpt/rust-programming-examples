# Testing Strategy and Patterns

> Visual reference for CLI testing approaches, test pyramids, and verification strategies

This document provides detailed diagrams of testing strategies for Clap-based CLI applications.

---

## CLI Testing Pyramid

```
+===========================================================================+
|                    CLI TESTING PYRAMID                                     |
+===========================================================================+

                              /\
                             /  \
                            /    \
                           / FUZZ \           <- Rare but important
                          / TESTS  \             Find edge cases
                         /----------\
                        /            \
                       /   SNAPSHOT   \       <- Moderate
                      /     TESTS      \        Catch regressions
                     /------------------\
                    /                    \
                   /    INTEGRATION       \   <- Common
                  /       TESTS            \    Full binary execution
                 /--------------------------\
                /                            \
               /       UNIT TESTS             \   <- Most common
              /        (try_parse_from)        \    Fast, isolated
             /----------------------------------\


    TEST TYPE DISTRIBUTION (recommended):
    =====================================

    +------------------+--------+------------------+-------------------+
    | Level            | Count  | Execution Time   | Coverage Focus    |
    +------------------+--------+------------------+-------------------+
    | Unit tests       | Many   | Milliseconds     | Parsing logic     |
    +------------------+--------+------------------+-------------------+
    | Integration tests| Medium | Seconds          | Binary behavior   |
    +------------------+--------+------------------+-------------------+
    | Snapshot tests   | Some   | Seconds          | Output stability  |
    +------------------+--------+------------------+-------------------+
    | Fuzz tests       | Few    | Minutes/Hours    | Edge cases        |
    +------------------+--------+------------------+-------------------+


    WHAT TO TEST AT EACH LEVEL:
    ===========================

    Unit Tests:
    -----------
    [x] Argument parsing combinations
    [x] Custom value parser functions
    [x] Default value behavior
    [x] Subcommand routing logic
    [x] Error conditions (missing args)

    Integration Tests:
    ------------------
    [x] Full binary execution
    [x] Exit codes
    [x] Stdout/stderr output
    [x] File system interactions
    [x] Environment variable handling

    Snapshot Tests:
    ---------------
    [x] --help output
    [x] --version output
    [x] Error message formatting
    [x] JSON/YAML output formats

    Fuzz Tests:
    -----------
    [x] Arbitrary argument strings
    [x] Unicode handling
    [x] Very long inputs
    [x] Special characters
```

**Diagram Description**: This pyramid illustrates the recommended distribution of test types for CLI applications, with unit tests forming the base (many, fast) and fuzz tests at the apex (few, thorough).

---

## Unit Test Patterns

```
+===========================================================================+
|                    UNIT TESTING PATTERNS                                   |
+===========================================================================+

    PATTERN 1: try_parse_from Testing
    ==================================

    Test Structure:
    +-----------------------------------------+
    | #[test]                                 |
    | fn test_valid_arguments() {             |
    |     let cli = Cli::try_parse_from([    |
    |         "myapp",                        |
    |         "--port", "8080",               |
    |         "--verbose",                    |
    |     ]).expect("should parse");          |
    |                                         |
    |     assert_eq!(cli.port, 8080);         |
    |     assert!(cli.verbose);               |
    | }                                       |
    +-----------------------------------------+

    Input Array:
    +--------+--------+--------+--------+
    | "myapp"| "--port"| "8080" |"--verbose"|
    +--------+--------+--------+--------+
        |         |        |         |
        v         v        v         v
    +------------------------------------------+
    | Clap parsing engine                      |
    +------------------------------------------+
                      |
                      v
    +------------------------------------------+
    | Result<Cli, Error>                       |
    +------------------------------------------+
                      |
              +-------+-------+
              |               |
              v               v
          Ok(Cli)        Err(Error)
              |               |
              v               v
    +------------+    +----------------+
    | Assert     |    | Assert error   |
    | field      |    | kind matches   |
    | values     |    | expectation    |
    +------------+    +----------------+


    PATTERN 2: Error Condition Testing
    ===================================

    #[test]
    fn test_missing_required_arg() {
        let result = Cli::try_parse_from(["myapp"]);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }


    PATTERN 3: Subcommand Testing
    ==============================

    #[test]
    fn test_subcommand_routing() {
        let cli = Cli::try_parse_from([
            "myapp", "config", "set", "key", "value"
        ]).unwrap();

        match cli.command {
            Commands::Config(ConfigArgs { command }) => {
                match command {
                    ConfigCommands::Set { key, value } => {
                        assert_eq!(key, "key");
                        assert_eq!(value, "value");
                    }
                    _ => panic!("Wrong subcommand"),
                }
            }
            _ => panic!("Wrong command"),
        }
    }


    PATTERN 4: Value Parser Testing
    ================================

    // Test the parser function directly
    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s"), Ok(Duration::from_secs(30)));
        assert_eq!(parse_duration("5m"), Ok(Duration::from_secs(300)));
        assert!(parse_duration("invalid").is_err());
    }

    // Test via CLI integration
    #[test]
    fn test_duration_in_cli() {
        let cli = Cli::try_parse_from([
            "myapp", "--timeout", "30s"
        ]).unwrap();
        assert_eq!(cli.timeout, Duration::from_secs(30));
    }
```

**Diagram Description**: This diagram shows four common unit testing patterns for Clap: basic argument parsing with try_parse_from, error condition testing, subcommand routing verification, and custom value parser testing.

---

## Integration Test Flow

```
+===========================================================================+
|                    INTEGRATION TEST ARCHITECTURE                           |
+===========================================================================+

    TEST EXECUTION FLOW:
    ====================

    Test Code                         Binary Under Test
    =========                         =================

    +------------------+
    | assert_cmd       |
    | Command::cargo_  |
    |   bin("myapp")   |
    +------------------+
            |
            | spawn process
            v
    +------------------+              +------------------+
    | .args([...])     | -----------> |    myapp         |
    | .env("KEY","VAL")|    stdin     |                  |
    | .write_stdin(...)|  --------->  |   main() {       |
    +------------------+              |     Cli::parse() |
            |                         |     ...          |
            | wait for exit           |   }              |
            |                         +------------------+
            v                                |
    +------------------+              stdout |  stderr
    | .assert()        | <--------------------+
    | .success()       |   exit code  |
    | .stdout(...)     | <------------+
    | .stderr(...)     |
    | .code(N)         |
    +------------------+


    ASSERTION TYPES:
    ================

    +------------------+----------------------------------+
    | Method           | What it checks                   |
    +------------------+----------------------------------+
    | .success()       | Exit code == 0                   |
    | .failure()       | Exit code != 0                   |
    | .code(N)         | Exit code == N                   |
    | .stdout(pred)    | Stdout matches predicate         |
    | .stderr(pred)    | Stderr matches predicate         |
    +------------------+----------------------------------+


    COMMON PREDICATES:
    ==================

    predicates::str::contains("text")
           |
           v
    +------------------+
    | Output contains  |
    | "text" substring |
    +------------------+

    predicates::str::is_match(r"\d+")
           |
           v
    +------------------+
    | Output matches   |
    | regex pattern    |
    +------------------+

    predicates::str::is_empty()
           |
           v
    +------------------+
    | Output is empty  |
    +------------------+


    TEST WITH TEMPORARY FILES:
    ==========================

    +------------------+
    | TempDir::new()   |
    +------------------+
            |
            v
    +------------------+
    | Create test      |
    | files in tempdir |
    +------------------+
            |
            v
    +------------------+
    | Run binary with  |
    | tempdir paths    |
    +------------------+
            |
            v
    +------------------+
    | Verify output    |
    | files created    |
    +------------------+
            |
            v
    +------------------+
    | TempDir dropped  |
    | (auto cleanup)   |
    +------------------+
```

**Diagram Description**: This diagram illustrates the architecture of integration tests using assert_cmd, showing the flow from test code to binary execution and back to assertions, including file system testing patterns.

---

## Snapshot Testing Strategy

```
+===========================================================================+
|                    SNAPSHOT TESTING WORKFLOW                               |
+===========================================================================+

    INITIAL SNAPSHOT CREATION:
    ==========================

    First test run (no snapshot exists):

    #[test]
    fn test_help_output() {
        let output = get_help_output();
        assert_snapshot!("help", output);
    }
           |
           v
    +--------------------------+
    | insta detects no         |
    | existing snapshot        |
    +--------------------------+
           |
           v
    +--------------------------+
    | Creates new file:        |
    | snapshots/help.snap.new  |
    +--------------------------+
           |
           v
    +--------------------------+
    | cargo insta review       |
    | (interactive approval)   |
    +--------------------------+
           |
           +----------+----------+
           |                     |
           v                     v
    +------------+        +-------------+
    | Accept     |        | Reject      |
    | (rename to |        | (delete new |
    | .snap)     |        | file)       |
    +------------+        +-------------+


    SUBSEQUENT TEST RUNS:
    =====================

    Snapshot exists, output unchanged:
    +----------------------------------+
    | Test passes silently             |
    +----------------------------------+

    Snapshot exists, output changed:
    +----------------------------------+
    | Test fails                       |
    | Creates .snap.new with new output|
    | cargo insta review to compare    |
    +----------------------------------+


    SNAPSHOT FILE STRUCTURE:
    ========================

    snapshots/
    +-- help.snap              <- Approved snapshot
    |   +-- source: test_help_output
    |   +-- expression: output
    |   +-- ---
    |   +-- Usage: myapp [OPTIONS]
    |   +-- ...
    |
    +-- help.snap.new          <- Pending (if changed)
        +-- (new content)


    REDACTION PATTERNS:
    ===================

    Dynamic content that changes between runs:

    Before redaction:           After redaction:
    +----------------------+    +----------------------+
    | Created: 2025-12-29  |    | Created: [TIMESTAMP] |
    | Path: /tmp/abc123    | -> | Path: [PATH]         |
    | Duration: 42.5ms     |    | Duration: [DURATION] |
    +----------------------+    +----------------------+

    fn redact(output: &str) -> String {
        let output = regex_replace(
            r"\d{4}-\d{2}-\d{2}",
            output,
            "[TIMESTAMP]"
        );
        // ... more redactions
        output
    }
```

**Diagram Description**: This diagram explains the snapshot testing workflow using insta, showing how snapshots are created, reviewed, approved, and how to handle dynamic content through redaction.

---

## Fuzz Testing Coverage

```
+===========================================================================+
|                    FUZZ TESTING TARGETS                                    |
+===========================================================================+

    WHAT TO FUZZ:
    =============

    +-----------------------------------+
    |     CLI Parsing Surface           |
    +-----------------------------------+
    |                                   |
    |  [1] Argument strings             |
    |      - Random bytes               |
    |      - Unicode sequences          |
    |      - Very long strings          |
    |      - Special characters         |
    |                                   |
    |  [2] Argument combinations        |
    |      - Valid + invalid mix        |
    |      - Conflicting flags          |
    |      - Duplicate arguments        |
    |                                   |
    |  [3] Value parser inputs          |
    |      - Boundary values            |
    |      - Malformed formats          |
    |      - Injection attempts         |
    |                                   |
    +-----------------------------------+


    FUZZ TARGET STRUCTURE:
    ======================

    fuzz_target!(|data: &[u8]| {
        //        ^^^^^^^^^^^^^
        //        Random bytes from fuzzer

        // 1. Convert to potential arguments
        if let Ok(s) = std::str::from_utf8(data) {
            let args: Vec<&str> = s.split_whitespace().collect();

            // 2. Prepend program name
            let mut full_args = vec!["myapp"];
            full_args.extend(args);

            // 3. Try to parse (should NEVER panic)
            let _ = Cli::try_parse_from(full_args);
        }
    });


    EXPECTED OUTCOMES:
    ==================

    +------------------+----------------------------------+
    | Fuzzer Input     | Expected Result                  |
    +------------------+----------------------------------+
    | Valid args       | Ok(Cli)                          |
    +------------------+----------------------------------+
    | Invalid args     | Err(Error) with proper kind      |
    +------------------+----------------------------------+
    | Malformed bytes  | Graceful UTF-8 handling          |
    +------------------+----------------------------------+
    | Any input        | NEVER PANICS                     |
    +------------------+----------------------------------+


    RUNNING FUZZER:
    ===============

    cargo +nightly fuzz run parse_args
           |
           v
    +-------------------------+
    | Fuzzer generates inputs |
    | at ~1000/sec            |
    +-------------------------+
           |
           v
    +-------------------------+
    | Each input tested       |
    | against fuzz target     |
    +-------------------------+
           |
    +------+------+
    |             |
    v             v
    No crash    CRASH!
    (continue)    |
                  v
           +------------+
           | Save crash |
           | input to   |
           | artifacts/ |
           +------------+
                  |
                  v
           +------------+
           | Debug and  |
           | fix issue  |
           +------------+
```

**Diagram Description**: This diagram explains fuzz testing for CLI applications, showing what to fuzz, how to structure fuzz targets, expected outcomes, and the fuzzing workflow.

---

## Test Organization

```
+===========================================================================+
|                    TEST FILE ORGANIZATION                                  |
+===========================================================================+

    PROJECT STRUCTURE:
    ==================

    my-cli/
    +-- src/
    |   +-- main.rs
    |   +-- cli.rs           <- CLI definition
    |   +-- commands/        <- Command handlers
    |   |   +-- mod.rs
    |   |   +-- init.rs
    |   |   +-- run.rs
    |   +-- lib.rs           <- Library (for testing)
    |
    +-- tests/               <- Integration tests
    |   +-- integration.rs   <- Main integration tests
    |   +-- helpers/         <- Test utilities
    |   |   +-- mod.rs
    |   +-- fixtures/        <- Test data
    |       +-- valid.toml
    |       +-- invalid.toml
    |
    +-- benches/             <- Performance tests
    |   +-- startup.rs
    |
    +-- fuzz/                <- Fuzz tests
        +-- Cargo.toml
        +-- fuzz_targets/
            +-- parse_args.rs


    TEST MODULE ORGANIZATION:
    =========================

    // src/cli.rs
    #[derive(Parser)]
    pub struct Cli { ... }

    #[cfg(test)]
    mod tests {
        use super::*;

        mod parsing {
            use super::*;
            #[test] fn test_minimal() { ... }
            #[test] fn test_all_args() { ... }
        }

        mod validation {
            use super::*;
            #[test] fn test_port_range() { ... }
            #[test] fn test_file_exists() { ... }
        }

        mod errors {
            use super::*;
            #[test] fn test_missing_required() { ... }
            #[test] fn test_invalid_value() { ... }
        }
    }
```

**Diagram Description**: This diagram shows the recommended file organization for CLI tests, including unit tests (in source files), integration tests (in tests/), benchmarks (in benches/), and fuzz tests (in fuzz/).

---

## Cross-Reference

- For architecture overview, see [architecture-overview.md](./architecture-overview.md)
- For error handling testing, see [error-recovery.md](./error-recovery.md)
- For parsing details, see [parsing-pipeline.md](./parsing-pipeline.md)

---

*This document is part of the Clap Architecture Book visual reference materials.*
