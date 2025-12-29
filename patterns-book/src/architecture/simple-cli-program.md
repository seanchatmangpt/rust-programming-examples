# Simple CLI Program

## Pattern Summary

Structure a command-line program as a single `main.rs` file containing a core algorithm function, argument parsing, error handling, and a main entry point—keeping the program focused, testable, and easy to understand.

---

## Context

You are building a simple command-line tool that performs a focused task: calculating the GCD of numbers, searching text files, converting formats, or processing data. The program is small enough that it doesn't need elaborate architecture, but you still want it to be well-structured, testable, and maintainable.

Your tool reads input from command-line arguments, performs a computation or transformation, and produces output to stdout or stderr.

---

## Problem

**How do you structure a simple command-line program to be clear, testable, and maintainable without over-engineering?**

Common pitfalls:
- **Monolithic main**: All logic in `main()` is hard to test
- **Untestable code**: Direct coupling to `std::env::args()` and `println!` prevents unit testing
- **Poor error handling**: Using `unwrap()` everywhere leads to unhelpful error messages
- **No separation**: Algorithm logic mixed with I/O concerns
- **Over-engineering**: Creating multiple modules for a 50-line program

---

## Forces

- **Simplicity**: Don't over-complicate a simple tool
- **Testability**: Core logic should be unit-testable
- **Usability**: Clear usage messages and error handling
- **Maintainability**: Code should be easy to read and modify
- **Correctness**: Edge cases should be handled
- **Performance**: For simple tools, clarity trumps micro-optimizations

Too little structure leads to untestable, fragile code. Too much structure creates unnecessary complexity for a simple tool.

---

## Solution

**Structure your CLI program with three clear layers in a single `main.rs` file: a pure, testable algorithm function; argument parsing and validation with clear error messages; and a thin main() that orchestrates I/O.**

### Structure

```rust
// 1. Core algorithm (pure, testable)
fn algorithm(input: InputType) -> OutputType {
    // Pure computation - no I/O
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_algorithm() {
        // Unit tests for core logic
    }
}

// 2. Argument parsing (I/O, error handling)
use std::env;
use std::str::FromStr;

// 3. Main entry point (orchestration)
fn main() {
    // Parse args
    // Call algorithm
    // Print results
    // Handle errors with clear messages
}
```

### Implementation

From `gcd`, a command-line tool to calculate the greatest common divisor:

```rust
#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

// LAYER 1: Core Algorithm
// Pure function - no I/O, easily testable
fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);
    while m != 0 {
        if m < n {
            let t = m;
            m = n;
            n = t;
        }
        m = m % n;
    }
    n
}

// Tests for core algorithm
#[test]
fn test_gcd() {
    assert_eq!(gcd(14, 15), 1);

    assert_eq!(gcd(2 * 3 * 5 * 11 * 17,
                   3 * 7 * 11 * 13 * 19),
               3 * 11);
}

// LAYER 2: Argument Parsing
use std::str::FromStr;
use std::env;

// LAYER 3: Main Entry Point
fn main() {
    // Step 1: Collect arguments into vector
    let mut numbers = Vec::new();

    for arg in env::args().skip(1) {  // skip(1) skips program name
        numbers.push(u64::from_str(&arg)
                     .expect("error parsing argument"));
    }

    // Step 2: Validate input
    if numbers.len() == 0 {
        eprintln!("Usage: gcd NUMBER ...");
        std::process::exit(1);
    }

    // Step 3: Compute result using algorithm
    let mut d = numbers[0];
    for m in &numbers[1..] {
        d = gcd(d, *m);
    }

    // Step 4: Output result
    println!("The greatest common divisor of {:?} is {}",
             numbers, d);
}
```

### Key Design Decisions

**1. Pure algorithm function**
```rust
fn gcd(mut n: u64, mut m: u64) -> u64 {
    // ✓ No I/O - takes inputs, returns output
    // ✓ Testable - can call from unit tests
    // ✓ Reusable - could be extracted to library
    assert!(n != 0 && m != 0);  // Precondition check
    // ... algorithm ...
}
```

**2. Clear error handling**
```rust
// Parse with helpful error message
numbers.push(u64::from_str(&arg)
             .expect("error parsing argument"));

// Validate input with usage message
if numbers.len() == 0 {
    eprintln!("Usage: gcd NUMBER ...");  // stderr for error messages
    std::process::exit(1);               // Non-zero exit code
}
```

**3. Separation of I/O and logic**
```rust
// I/O: Parsing arguments
for arg in env::args().skip(1) { ... }

// Logic: Computing result
for m in &numbers[1..] {
    d = gcd(d, *m);
}

// I/O: Printing result
println!("The greatest common divisor of {:?} is {}", numbers, d);
```

### Usage

```bash
$ cargo run
Usage: gcd NUMBER ...

$ cargo run 14 15
The greatest common divisor of [14, 15] is 1

$ cargo run 12 18 24
The greatest common divisor of [12, 18, 24] is 6

$ cargo test
running 1 test
test test_gcd ... ok
```

### Guidelines

1. **Keep algorithm pure**
   - Input through parameters, not `stdin` or `env::args()`
   - Output through return value, not `println!`
   - No side effects in core logic

2. **Handle errors gracefully**
   ```rust
   // ✓ GOOD: Helpful error message
   .expect("error parsing argument")

   // ✗ BAD: Silent failure or panic with no context
   .unwrap()
   ```

3. **Use stderr for errors, stdout for output**
   ```rust
   eprintln!("Usage: ...");  // Errors to stderr
   println!("Result: ...");  // Results to stdout
   ```

4. **Provide clear usage messages**
   ```rust
   if args_invalid {
       eprintln!("Usage: {} NUMBER ...", program_name);
       eprintln!("Calculate the greatest common divisor of all numbers");
       std::process::exit(1);
   }
   ```

5. **Test the algorithm, not I/O**
   ```rust
   #[test]
   fn test_gcd() {
       assert_eq!(gcd(14, 15), 1);  // Test pure function
       // Don't test main() - it's just I/O plumbing
   }
   ```

6. **Use assertions for preconditions**
   ```rust
   fn gcd(mut n: u64, mut m: u64) -> u64 {
       assert!(n != 0 && m != 0);  // Catch programming errors early
       // ...
   }
   ```

---

## Resulting Context

### Benefits

- **Testability**: Core algorithm is pure function, easily tested
- **Clarity**: Three-layer structure is immediately understandable
- **Maintainability**: Logic separated from I/O makes changes easier
- **Reusability**: Algorithm can be extracted to library later
- **Simplicity**: Single file, no over-engineering
- **Good error messages**: Users know what went wrong

### Drawbacks

- **Limited structure**: For larger CLIs (>200 lines), needs refactoring
  - *Mitigation*: See [Library-Binary Split](library-binary-split.md) pattern
- **Testing I/O is harder**: main() isn't unit-tested
  - *Mitigation*: Keep main() thin; test algorithm thoroughly
- **No argument parser**: Manual parsing can get complex
  - *Mitigation*: For complex CLIs, use `clap` or `structopt`

### Invariants Maintained

- Core algorithm is pure (no I/O)
- Tests exist for core logic
- Error messages are helpful
- Exit codes are meaningful (0 = success, 1 = error)
- Usage information is provided

---

## Related Patterns

- **[Library-Binary Split](library-binary-split.md)**: When the CLI grows, extract logic to `lib.rs`
- **[Separation of Concerns](separation-of-concerns.md)**: Algorithm vs. I/O is a separation of concerns
- **Result Type Pattern**: Use `Result<T, E>` instead of `expect()` for recoverable errors

---

## Known Uses

### From `gcd`

Perfect example of this pattern:
- Pure `gcd()` function
- Argument parsing in `main()`
- Clear error handling
- Comprehensive tests

### Ripgrep (simplified structure)

```rust
fn search(pattern: &str, content: &str) -> Vec<Match> {
    // Pure search algorithm
}

fn main() {
    let args = parse_args();  // I/O layer
    let content = read_file(&args.file)?;  // I/O layer
    let matches = search(&args.pattern, &content);  // Pure logic
    print_matches(&matches);  // I/O layer
}
```

### Standard Unix utilities (Rust implementations)

- **fd** (find alternative): Core search in library, CLI in main
- **bat** (cat alternative): Syntax highlighting logic separate from CLI
- **exa** (ls alternative): File listing logic separate from rendering

---

## Examples and Variations

### Variation 1: With Result Type

More robust error handling:

```rust
use std::error::Error;
use std::result::Result;

fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);
    // ... algorithm ...
}

fn parse_args() -> Result<Vec<u64>, Box<dyn Error>> {
    let mut numbers = Vec::new();
    for arg in std::env::args().skip(1) {
        let num = arg.parse::<u64>()?;  // ? for early return on error
        numbers.push(num);
    }
    if numbers.is_empty() {
        return Err("no numbers provided".into());
    }
    Ok(numbers)
}

fn main() {
    match parse_args() {
        Ok(numbers) => {
            let mut d = numbers[0];
            for &m in &numbers[1..] {
                d = gcd(d, m);
            }
            println!("The greatest common divisor of {:?} is {}", numbers, d);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Usage: gcd NUMBER ...");
            std::process::exit(1);
        }
    }
}
```

### Variation 2: With clap for Argument Parsing

For more complex CLIs:

```rust
use clap::Parser;

#[derive(Parser)]
#[clap(about = "Calculate greatest common divisor")]
struct Args {
    #[clap(required = true)]
    numbers: Vec<u64>,
}

fn gcd(mut n: u64, mut m: u64) -> u64 {
    // ... same pure algorithm ...
}

fn main() {
    let args = Args::parse();  // clap handles parsing and validation

    let mut d = args.numbers[0];
    for &m in &args.numbers[1..] {
        d = gcd(d, m);
    }

    println!("The greatest common divisor of {:?} is {}", args.numbers, d);
}
```

### Variation 3: Reading from stdin

For pipeline-friendly tools:

```rust
use std::io::{self, BufRead};

fn process_line(line: &str) -> String {
    // Pure transformation
    line.to_uppercase()
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let result = process_line(&line);
        println!("{}", result);
    }
    Ok(())
}
```

---

## Anti-Patterns to Avoid

### Monolithic main()

```rust
// ❌ BAD: All logic in main, untestable
fn main() {
    let mut numbers = Vec::new();
    for arg in std::env::args().skip(1) {
        numbers.push(arg.parse::<u64>().unwrap());
    }

    // GCD algorithm inlined in main
    let mut d = numbers[0];
    for m in &numbers[1..] {
        let mut n = d;
        let mut m = *m;
        while m != 0 {
            if m < n {
                let t = m;
                m = n;
                n = t;
            }
            m = m % n;
        }
        d = n;
    }

    println!("Result: {}", d);
}
```

**Problem**: Can't test algorithm without running full program.

**Fix**: Extract algorithm to separate function.

### Silent Failures

```rust
// ❌ BAD: Panic with no context
fn main() {
    let arg = std::env::args().nth(1).unwrap();  // Panic if no arg!
    let num = arg.parse::<u64>().unwrap();       // Panic if not a number!
    println!("{}", num);
}
```

**Fix**: Provide helpful error messages.

### No Tests

```rust
// ❌ BAD: No tests at all
fn gcd(n: u64, m: u64) -> u64 {
    // ... algorithm ...
}

fn main() { /* ... */ }

// Where are the tests?
```

**Fix**: Always test core algorithm.

---

## Checklist

For a well-structured simple CLI:

- [ ] Core algorithm is a pure function (testable)
- [ ] Arguments are parsed with clear error messages
- [ ] Usage information is provided for invalid input
- [ ] Errors go to stderr, results to stdout
- [ ] Exit codes are meaningful (0 for success, 1 for errors)
- [ ] Unit tests exist for algorithm
- [ ] Edge cases are handled (empty input, invalid numbers)
- [ ] Code is in a single `main.rs` (for simple tools)

---

## When to Evolve Beyond This Pattern

**Move to [Library-Binary Split](library-binary-split.md) when**:
- Your main.rs exceeds ~200 lines
- You want to reuse the algorithm in other programs
- You need to support multiple output formats
- The logic deserves separate documentation

**Use a proper argument parser when**:
- You have subcommands (`git commit`, `git push`)
- You have optional flags and arguments
- You need help messages generated automatically
- Argument parsing logic exceeds ~20 lines

**Create multiple modules when**:
- You have multiple related algorithms
- You need different output formatters
- Configuration becomes complex

---

## Further Reading

- *Programming Rust*, Chapter 2: A Simple CLI Program
- [Command Line Applications in Rust](https://rust-cli.github.io/book/)
- [clap documentation](https://docs.rs/clap/) - Argument parsing library
- [Rust API Guidelines: Flexibility (C-VALIDATE)](https://rust-lang.github.io/api-guidelines/flexibility.html#validate-arguments-c-validate)
