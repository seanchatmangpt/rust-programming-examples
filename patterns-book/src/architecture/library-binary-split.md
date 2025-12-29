# Library-Binary Split

## Pattern Summary

Separate your program's core logic into a library crate (`lib.rs`) and keep the command-line interface as a thin binary crate (`main.rs`), enabling reusability, better testing, and clearer architectural boundaries.

---

## Context

You are building a Rust program that has grown beyond a simple command-line tool. The core logic is becoming substantial—perhaps it processes data, implements algorithms, manages state, or provides services. You want to make this logic reusable in different contexts (web service, library, different CLI interfaces) while maintaining a command-line interface.

Alternatively, you're starting a new project and anticipate that the core logic will be valuable independent of the CLI.

---

## Problem

**How do you structure a program so that the core logic is reusable, testable, and independent of the command-line interface, while still providing a convenient CLI for users?**

Keeping everything in `main.rs` creates problems:
- **Not reusable**: Core logic is tied to CLI
- **Hard to test**: Logic mixed with I/O can't be easily unit-tested
- **Single interface**: Can only use via command line
- **No library use**: Other programs can't depend on your logic
- **Unclear boundaries**: No separation between domain logic and UI

Creating a separate library crate in a different repository creates problems:
- **Duplication**: Must maintain two repos for one project
- **Coordination overhead**: Versioning, releases, dependencies
- **Discovery issues**: Users may not find the library

---

## Forces

- **Reusability**: Core logic should be usable in different contexts
- **Testability**: Logic should be testable without running the full CLI
- **Simplicity**: Don't over-complicate small programs
- **Documentation**: Library and CLI have different documentation needs
- **API stability**: Library API should be stable; CLI can evolve differently
- **Convenience**: Single repository, single project
- **Performance**: Binary should have minimal overhead

Too early splitting creates unnecessary complexity. Too late splitting makes refactoring painful.

---

## Solution

**Split your project into a library crate (`src/lib.rs`) containing reusable logic and a binary crate (`src/main.rs`) that provides a thin CLI layer over the library.**

### Structure

```
project/
├── Cargo.toml
├── src/
│   ├── lib.rs        ← Library crate: reusable logic
│   ├── main.rs       ← Binary crate: CLI interface
│   └── modules/      ← Shared by library
└── tests/
    └── integration_tests.rs  ← Tests against library API
```

In `Cargo.toml`:
```toml
[package]
name = "my_tool"

[lib]
name = "my_tool"      # Library name
path = "src/lib.rs"

[[bin]]
name = "my_tool"      # Binary name (CLI)
path = "src/main.rs"
```

### Implementation

**Example: Evolving `gcd` from simple CLI to library + binary**

**BEFORE: Simple CLI (single main.rs)**

```rust
// src/main.rs (before split)
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

fn main() {
    let mut numbers = Vec::new();
    for arg in std::env::args().skip(1) {
        numbers.push(arg.parse::<u64>().expect("error parsing argument"));
    }
    if numbers.is_empty() {
        eprintln!("Usage: gcd NUMBER ...");
        std::process::exit(1);
    }
    let mut d = numbers[0];
    for m in &numbers[1..] {
        d = gcd(d, *m);
    }
    println!("The greatest common divisor of {:?} is {}", numbers, d);
}
```

**AFTER: Library + Binary Split**

**Library crate** (`src/lib.rs`) - Reusable logic:

```rust
//! GCD computation library.
//!
//! This library provides functions for computing the greatest common divisor
//! (GCD) of integers using Euclid's algorithm.

#![warn(rust_2018_idioms)]

/// Compute the greatest common divisor of two numbers.
///
/// # Panics
///
/// Panics if either argument is zero.
///
/// # Examples
///
/// ```
/// use gcd::gcd;
///
/// assert_eq!(gcd(14, 15), 1);
/// assert_eq!(gcd(12, 18), 6);
/// ```
pub fn gcd(mut n: u64, mut m: u64) -> u64 {
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

/// Compute the GCD of multiple numbers.
///
/// # Panics
///
/// Panics if the input is empty or contains zero.
///
/// # Examples
///
/// ```
/// use gcd::gcd_multiple;
///
/// assert_eq!(gcd_multiple(&[12, 18, 24]), 6);
/// ```
pub fn gcd_multiple(numbers: &[u64]) -> u64 {
    assert!(!numbers.is_empty(), "Cannot compute GCD of empty list");
    numbers.iter().copied().reduce(|acc, n| gcd(acc, n)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(14, 15), 1);
        assert_eq!(gcd(2 * 3 * 5 * 11 * 17,
                       3 * 7 * 11 * 13 * 19),
                   3 * 11);
    }

    #[test]
    fn test_gcd_multiple() {
        assert_eq!(gcd_multiple(&[12, 18, 24]), 6);
        assert_eq!(gcd_multiple(&[14, 15]), 1);
    }

    #[test]
    #[should_panic(expected = "Cannot compute GCD of empty list")]
    fn test_gcd_multiple_empty() {
        gcd_multiple(&[]);
    }
}
```

**Binary crate** (`src/main.rs`) - Thin CLI layer:

```rust
//! Command-line interface for the GCD library.

use std::env;
use std::str::FromStr;

// Import from our library crate
use gcd::gcd_multiple;

fn main() {
    // Parse command-line arguments
    let mut numbers = Vec::new();

    for arg in env::args().skip(1) {
        numbers.push(u64::from_str(&arg)
                     .expect("error parsing argument"));
    }

    // Validate input
    if numbers.is_empty() {
        eprintln!("Usage: gcd NUMBER ...");
        eprintln!("Calculate the greatest common divisor of all numbers");
        std::process::exit(1);
    }

    // Call library function
    let d = gcd_multiple(&numbers);

    // Output result
    println!("The greatest common divisor of {:?} is {}", numbers, d);
}
```

**Now the library can be used in other contexts**:

```rust
// Another program using gcd as a library
use gcd::{gcd, gcd_multiple};

fn main() {
    let result = gcd(48, 18);
    println!("GCD: {}", result);

    let batch_result = gcd_multiple(&[100, 50, 75]);
    println!("Batch GCD: {}", batch_result);
}
```

---

### Real-World Example: fern_sim

**Library structure** (`src/lib.rs`):

```rust
//! Simulate the growth of ferns, from the level of
//! individual cells on up.

#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

// Public modules - part of library API
pub mod plant_structures;
pub mod simulation;
pub mod spores;

// Re-exports for convenience
pub use plant_structures::Fern;
pub use simulation::Terrarium;

pub mod net;
pub use net::connect;
```

**Usage as a library**:

```rust
// Another program using fern_sim as a library
use fern_sim::{Terrarium, Fern, FernType};
use std::time::Duration;

fn main() {
    let mut terrarium = Terrarium::new();
    terrarium.apply_sunlight(Duration::from_secs(3600));

    // Access as a library - no CLI involved
    println!("Simulation complete");
}
```

**Could also have a binary** (`src/main.rs`):

```rust
//! Command-line interface for fern simulation.

use fern_sim::{Terrarium};
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let duration = if args.len() > 1 {
        args[1].parse::<u64>().expect("Invalid duration")
    } else {
        3600  // Default: 1 hour
    };

    let mut terrarium = Terrarium::load("default.tm");
    terrarium.apply_sunlight(Duration::from_secs(duration));

    println!("Simulation ran for {} seconds", duration);
}
```

---

### Guidelines

1. **What goes in lib.rs**:
   - ✅ Core algorithms and data structures
   - ✅ Domain logic
   - ✅ Reusable utilities
   - ✅ Public API types
   - ✅ Comprehensive tests

2. **What goes in main.rs**:
   - ✅ Command-line argument parsing
   - ✅ User interaction (prompts, menus)
   - ✅ Output formatting specific to CLI
   - ✅ Process exit handling
   - ✅ Minimal logic - mostly orchestration

3. **main.rs should be thin**:
   ```rust
   // ✓ GOOD: Thin main
   fn main() {
       let config = parse_args();
       match my_lib::run(config) {
           Ok(result) => println!("{}", result),
           Err(e) => {
               eprintln!("Error: {}", e);
               std::process::exit(1);
           }
       }
   }

   // ✗ BAD: Fat main with business logic
   fn main() {
       // ... 200 lines of logic that should be in lib.rs ...
   }
   ```

4. **Use `use crate_name::...` in main.rs**:
   ```rust
   // In main.rs
   use gcd::{gcd, gcd_multiple};  // Import from your own library

   // NOT:
   // mod gcd;  // This would try to import a module file
   ```

5. **Documentation strategy**:
   - Library docs: API documentation, usage examples
   - Binary docs: User guide, CLI help text

---

## Resulting Context

### Benefits

- **Reusability**: Logic available as library to other programs
  ```rust
  // Other programs can use your library
  use my_tool::{process, Config};
  ```

- **Better testing**: Test library API directly
  ```rust
  // tests/integration.rs
  use my_tool::process;

  #[test]
  fn test_processing() {
      let result = process("input");
      assert_eq!(result, "expected");
  }
  ```

- **Multiple interfaces**: Same logic, different UIs
  - CLI binary
  - Web server (using actix-web)
  - GUI (using egui)
  - Library for other programs

- **Clear boundaries**: Separation of concerns enforced
  - Library: domain logic
  - Binary: I/O and user interaction

- **Documentation clarity**: Library API docs separate from CLI docs

- **Incremental builds**: Changes to main.rs don't recompile lib.rs

### Drawbacks

- **Initial overhead**: More files to set up initially
  - *Mitigation*: Use `cargo new --lib` template
- **Import syntax**: Need `use crate_name::...` in main.rs
  - *Mitigation*: Clear convention, easy to learn
- **Two crates in one**: Cargo.toml more complex
  - *Mitigation*: Standard pattern, well-documented

### Invariants Maintained

- Library crate has no I/O in core logic (pure functions where possible)
- Binary crate is thin orchestration layer
- Library is fully testable without binary
- Both crates can evolve independently (within reason)

---

## Related Patterns

- **[Simple CLI Program](simple-cli-program.md)**: Start with this, evolve to library-binary split
- **[Separation of Concerns](separation-of-concerns.md)**: lib.rs vs main.rs is ultimate separation
- **[Hierarchical Modules](hierarchical-modules.md)**: Library often has hierarchical modules

---

## Known Uses

### From the Rust Ecosystem

**ripgrep** (`rg`):
```
ripgrep/
├── crates/
│   ├── core/       ← Library: search algorithm
│   ├── printer/    ← Library: output formatting
│   └── ...
└── src/
    └── main.rs     ← Binary: CLI interface
```

**serde**:
```
serde/
├── src/
│   └── lib.rs      ← Library: serialization framework
└── serde_derive/
    └── lib.rs      ← Separate crate: derive macros
```

**tokio** (no binary, pure library):
```
tokio/
└── src/
    └── lib.rs      ← Pure library, no CLI
```

**cargo** itself:
```
cargo/
├── src/
│   ├── cargo/      ← Library: build system logic
│   └── bin/
│       └── cargo.rs  ← Binary: CLI
```

---

## Migration Path

### Step 1: Simple CLI (starting point)

```rust
// src/main.rs only
fn algorithm(input: &str) -> String { /* ... */ }

fn main() {
    let input = std::env::args().nth(1).unwrap();
    println!("{}", algorithm(&input));
}
```

### Step 2: Extract library (when complexity grows)

Create `src/lib.rs`:
```rust
pub fn algorithm(input: &str) -> String {
    // Moved from main.rs
}
```

Update `src/main.rs`:
```rust
use my_crate::algorithm;  // Import from lib

fn main() {
    let input = std::env::args().nth(1).unwrap();
    println!("{}", algorithm(&input));
}
```

### Step 3: Expand library (as project grows)

```rust
// src/lib.rs
pub mod core;
pub mod utils;

pub use core::algorithm;
pub use utils::helper;
```

---

## Decision Criteria

**Use library-binary split when**:
- ✅ Your program exceeds ~200-300 lines
- ✅ Core logic is reusable in other contexts
- ✅ You want to provide both library and CLI
- ✅ Testing would benefit from library interface
- ✅ You anticipate multiple frontends (CLI, web, GUI)

**Stay with simple main.rs when**:
- ✅ Program is < 200 lines and unlikely to grow
- ✅ Logic is CLI-specific (argument parsing helpers)
- ✅ One-off tool with no reuse value
- ✅ You're prototyping and structure is unclear

---

## Checklist

When splitting library and binary:

- [ ] `lib.rs` contains all core logic
- [ ] `main.rs` is thin (< 100 lines typically)
- [ ] Library has comprehensive unit tests
- [ ] Binary has integration tests (if complex)
- [ ] Library is documented with `///` doc comments
- [ ] Binary has `--help` text for users
- [ ] Library API is public (not just `pub(crate)`)
- [ ] `Cargo.toml` declares both `[lib]` and `[[bin]]`
- [ ] Library can be used without binary

---

## Examples: Different Split Strategies

### Strategy 1: Minimal Binary

```rust
// src/lib.rs - All logic
pub fn run(config: Config) -> Result<()> {
    // Everything happens here
}

// src/main.rs - Just parsing
fn main() {
    let config = Config::from_args();
    if let Err(e) = my_tool::run(config) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

### Strategy 2: Binary with UI Logic

```rust
// src/lib.rs - Core logic only
pub fn process_data(input: &[u8]) -> Vec<Result> { /* ... */ }

// src/main.rs - CLI-specific formatting
use my_tool::process_data;

fn main() {
    let input = read_input();
    let results = process_data(&input);

    // CLI-specific output formatting
    for (i, result) in results.iter().enumerate() {
        println!("{}: {:?}", i, result);
    }
}
```

### Strategy 3: Multiple Binaries

```toml
# Cargo.toml
[[bin]]
name = "my_tool"
path = "src/bin/cli.rs"

[[bin]]
name = "my_tool_server"
path = "src/bin/server.rs"
```

```rust
// src/bin/cli.rs
use my_tool::core;

fn main() {
    // CLI interface
}

// src/bin/server.rs
use my_tool::core;

fn main() {
    // Web server interface
}
```

---

## Anti-Patterns

### Fat Binary

```rust
// ❌ BAD: All logic in main.rs
fn main() {
    // ... 500 lines of business logic ...
}
```

**Fix**: Extract to lib.rs.

### Anemic Library

```rust
// ❌ BAD: Library just re-exports main's functions
pub fn run(args: Vec<String>) {
    // Still doing CLI work in library
    let config = parse_args(args);  // CLI concern!
    // ...
}
```

**Fix**: Library should have pure API, not CLI-aware.

### Duplicate Logic

```rust
// ❌ BAD: Logic duplicated between lib and bin
// lib.rs
pub fn process_a(...) { /* ... */ }

// main.rs
fn process_b(...) {
    // Similar logic duplicated here
}
```

**Fix**: Move all logic to library.

---

## Further Reading

- *Programming Rust*, Chapter 8: Crates and Modules
- [Rust Book: Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)
- [Cargo Book: Package Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html)
- [Command Line Applications in Rust](https://rust-cli.github.io/book/tutorial/setup.html)
