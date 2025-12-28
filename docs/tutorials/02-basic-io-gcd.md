# Tutorial 2: Basic I/O and the GCD Calculator

## Introduction

In this tutorial, we'll build a deep understanding of the GCD (Greatest Common Divisor) calculator. This is an excellent first Rust program because it introduces command-line arguments, number parsing, error handling, and basic algorithms.

## What You'll Learn

- Reading command-line arguments in Rust
- String parsing and number conversion
- Rust's type system and integer types
- The Euclidean algorithm implementation
- Error handling with `expect()` and `panic`
- Testing in Rust

## Prerequisites

- Completed Tutorial 1 (Getting Started)
- The rust-programming-examples repository cloned
- Basic understanding of the GCD concept

## The Program Overview

The GCD calculator takes multiple numbers as command-line arguments and computes their greatest common divisor.

Example usage:
```bash
cargo run 24 60
# Output: The greatest common divisor of [24, 60] is 12

cargo run 100 75 50 25
# Output: The greatest common divisor of [100, 75, 50, 25] is 25
```

## Step 1: Examine the Complete Code

Navigate to the gcd directory and open `src/main.rs`:

```bash
cd ~/rust-programming-examples/gcd
cat src/main.rs
```

Let's look at the complete program:

```rust
#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

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

#[test]
fn test_gcd() {
    assert_eq!(gcd(14, 15), 1);
    assert_eq!(gcd(2 * 3 * 5 * 11 * 17,
                   3 * 7 * 11 * 13 * 19),
               3 * 11);
}

use std::str::FromStr;
use std::env;

fn main() {
    let mut numbers = Vec::new();

    for arg in env::args().skip(1) {
        numbers.push(u64::from_str(&arg)
                     .expect("error parsing argument"));
    }

    if numbers.len() == 0 {
        eprintln!("Usage: gcd NUMBER ...");
        std::process::exit(1);
    }

    let mut d = numbers[0];
    for m in &numbers[1..] {
        d = gcd(d, *m);
    }

    println!("The greatest common divisor of {:?} is {}",
             numbers, d);
}
```

Don't worry if this seems overwhelming! We'll break it down piece by piece.

## Step 2: Understanding the Main Function

Let's start with the entry point:

```rust
fn main() {
    let mut numbers = Vec::new();
    // ... rest of the code
}
```

### Python Comparison: Lists vs Vectors

**Rust**:
```rust
let mut numbers = Vec::new();  // Create empty vector
numbers.push(42);               // Add element
```

**Python**:
```python
numbers = []          # Create empty list
numbers.append(42)    # Add element
```

**Key Differences**:
- In Rust, `Vec` is a growable array (like Python's list)
- `let mut` means the variable is mutable (can be changed)
- `let` without `mut` creates an immutable variable
- Rust requires explicit mutability

Try this experiment:

```rust
// This works:
let mut numbers = Vec::new();
numbers.push(1);

// This fails to compile:
let numbers = Vec::new();
numbers.push(1);  // ERROR: cannot mutate immutable variable
```

## Step 3: Parsing Command-Line Arguments

The program reads arguments using `std::env::args()`:

```rust
use std::env;

for arg in env::args().skip(1) {
    numbers.push(u64::from_str(&arg)
                 .expect("error parsing argument"));
}
```

### Breaking This Down

**`env::args()`** returns an iterator over command-line arguments.

```rust
// If you run: cargo run 24 60
// env::args() returns: ["target/debug/gcd", "24", "60"]
```

**`.skip(1)`** skips the first argument (the program name).

```rust
// After skip(1): ["24", "60"]
```

**The for loop** iterates over each remaining argument:

```rust
for arg in env::args().skip(1) {
    // arg is a String: "24", then "60"
}
```

### Python Comparison: Command-Line Arguments

**Rust**:
```rust
use std::env;

for arg in env::args().skip(1) {
    println!("{}", arg);
}
```

**Python (sys.argv)**:
```python
import sys

for arg in sys.argv[1:]:
    print(arg)
```

**Python (argparse)**:
```python
import argparse

parser = argparse.ArgumentParser()
parser.add_argument('numbers', nargs='+', type=int)
args = parser.parse_args()

for num in args.numbers:
    print(num)
```

Rust's approach is lower-level (like `sys.argv`), but there are crates like `clap` that provide argparse-like functionality.

## Step 4: Understanding Number Types and Parsing

```rust
use std::str::FromStr;

numbers.push(u64::from_str(&arg)
             .expect("error parsing argument"));
```

### Rust's Number Types

Unlike Python's unified `int`, Rust has specific integer types:

| Type | Size | Range | Signed? |
|------|------|-------|---------|
| `i8` | 8 bits | -128 to 127 | Yes |
| `u8` | 8 bits | 0 to 255 | No |
| `i32` | 32 bits | -2B to 2B | Yes |
| `u32` | 32 bits | 0 to 4B | No |
| `i64` | 64 bits | Large negative to positive | Yes |
| `u64` | 64 bits | 0 to very large | No |

`u64` means "unsigned 64-bit integer" - perfect for GCD (always positive, potentially large).

### Parsing Strings to Numbers

**`u64::from_str(&arg)`** converts a string to a `u64`.

```rust
// If arg is "24":
let result = u64::from_str("24");
// result is Ok(24)

// If arg is "abc":
let result = u64::from_str("abc");
// result is Err(ParseIntError)
```

This returns a `Result` type - either `Ok(number)` or `Err(error)`.

### Error Handling with expect()

**`.expect("error parsing argument")`** unwraps the Result:

```rust
let number = u64::from_str(&arg).expect("error parsing argument");
// If Ok(24): number = 24
// If Err(_): program panics with message "error parsing argument"
```

Try this:

```bash
cargo run 24 abc
```

You'll see:
```
thread 'main' panicked at 'error parsing argument: ParseIntError { kind: InvalidDigit }'
```

### Python Comparison: Type Conversion

**Rust**:
```rust
let num = u64::from_str("24").expect("error");
// Explicit type, explicit error handling
```

**Python**:
```python
num = int("24")
# Dynamic type, throws ValueError on error

# Python error handling:
try:
    num = int("abc")
except ValueError as e:
    print(f"error: {e}")
```

Rust forces you to handle errors at compile time. Python handles them at runtime.

## Step 5: Input Validation

```rust
if numbers.len() == 0 {
    eprintln!("Usage: gcd NUMBER ...");
    std::process::exit(1);
}
```

### Understanding eprintln!

**`eprintln!`** prints to standard error (stderr):

```rust
println!("This goes to stdout");
eprintln!("This goes to stderr");
```

**Python Comparison**:
```python
import sys

print("This goes to stdout")
print("This goes to stderr", file=sys.stderr)
```

### Exiting the Program

**`std::process::exit(1)`** terminates with exit code 1 (indicating error).

```rust
std::process::exit(0);  // Success
std::process::exit(1);  // Error
```

**Python Comparison**:
```python
import sys
sys.exit(1)
```

## Step 6: The GCD Algorithm (Euclidean Algorithm)

Now let's understand the core algorithm:

```rust
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
```

### Function Signature

```rust
fn gcd(mut n: u64, mut m: u64) -> u64
```

Breaking this down:
- `fn gcd` - function named "gcd"
- `mut n: u64` - mutable parameter n of type u64
- `mut m: u64` - mutable parameter m of type u64
- `-> u64` - returns a u64

**Python Comparison**:
```python
def gcd(n: int, m: int) -> int:
    # In Python, parameters are always mutable (technically, rebindable)
    pass
```

### The assert! Macro

```rust
assert!(n != 0 && m != 0);
```

This checks that both numbers are non-zero. If not, the program panics.

```rust
// This panics:
gcd(0, 5);  // panic: assertion failed

// This is fine:
gcd(5, 10);  // returns 5
```

**Python Comparison**:
```python
assert n != 0 and m != 0, "Numbers must be non-zero"
```

### The Algorithm Logic

The Euclidean algorithm works by repeatedly replacing the larger number with the remainder:

```rust
while m != 0 {
    if m < n {
        // Swap if m is smaller
        let t = m;
        m = n;
        n = t;
    }
    m = m % n;  // Replace m with remainder
}
n  // Return n (the GCD)
```

**Example: gcd(24, 60)**

| Iteration | n | m | m % n | Action |
|-----------|---|---|-------|--------|
| Start | 24 | 60 | - | - |
| 1 | 24 | 60 | 12 | m < n is false, m = 60 % 24 = 12 |
| 2 | 24 | 12 | - | m < n is true, swap: n=12, m=24 |
| 2 cont. | 12 | 24 | 0 | m = 24 % 12 = 0 |
| 3 | 12 | 0 | - | m == 0, exit loop |
| Return | **12** | - | - | GCD is 12 |

### Python Implementation

```python
def gcd(n: int, m: int) -> int:
    assert n != 0 and m != 0
    while m != 0:
        if m < n:
            n, m = m, n  # Python's tuple unpacking
        m = m % n
    return n
```

The logic is identical, but Python's tuple unpacking is more concise for swapping.

## Step 7: Computing GCD of Multiple Numbers

```rust
let mut d = numbers[0];
for m in &numbers[1..] {
    d = gcd(d, *m);
}
```

### Breaking This Down

**`let mut d = numbers[0]`** - Start with the first number.

**`&numbers[1..]`** - A slice of the vector from index 1 to the end:
- `numbers[1..]` would move/copy the data
- `&numbers[1..]` borrows it (creates a reference)

**`for m in &numbers[1..]`** - Iterate over references to remaining numbers.

**`*m`** - Dereference to get the actual value:
- `m` is a `&u64` (reference to u64)
- `*m` is a `u64` (the value itself)

### Python Comparison

**Rust**:
```rust
let mut d = numbers[0];
for m in &numbers[1..] {
    d = gcd(d, *m);
}
```

**Python**:
```python
d = numbers[0]
for m in numbers[1:]:
    d = gcd(d, m)
```

**Python with functools**:
```python
from functools import reduce

result = reduce(gcd, numbers)
```

## Step 8: Output Formatting

```rust
println!("The greatest common divisor of {:?} is {}",
         numbers, d);
```

### Understanding println! Formatting

**`{:?}`** - Debug format (prints the entire vector):
```rust
println!("{:?}", numbers);  // Prints: [24, 60]
```

**`{}`** - Display format (for types that implement Display):
```rust
println!("{}", d);  // Prints: 12
```

### Python Comparison

**Rust**:
```rust
println!("The GCD of {:?} is {}", numbers, d);
```

**Python (f-strings)**:
```python
print(f"The GCD of {numbers} is {d}")
```

**Python (format)**:
```python
print("The GCD of {} is {}".format(numbers, d))
```

## Step 9: Testing

The code includes a unit test:

```rust
#[test]
fn test_gcd() {
    assert_eq!(gcd(14, 15), 1);
    assert_eq!(gcd(2 * 3 * 5 * 11 * 17,
                   3 * 7 * 11 * 13 * 19),
               3 * 11);
}
```

### Understanding #[test]

**`#[test]`** is an attribute that marks a function as a test:

```rust
#[test]
fn test_gcd() {
    assert_eq!(gcd(14, 15), 1);  // Check gcd(14, 15) == 1
}
```

Run tests with:
```bash
cargo test
```

Output:
```
running 1 test
test test_gcd ... ok

test result: ok. 1 passed; 0 failed
```

### Python Comparison

**Rust**:
```rust
#[test]
fn test_gcd() {
    assert_eq!(gcd(14, 15), 1);
}
```

**Python (pytest)**:
```python
def test_gcd():
    assert gcd(14, 15) == 1
```

**Python (unittest)**:
```python
import unittest

class TestGCD(unittest.TestCase):
    def test_gcd(self):
        self.assertEqual(gcd(14, 15), 1)
```

Rust's testing is built into the language and compiler - no external framework needed!

## Step 10: Hands-On Exercises

### Exercise 1: Test Edge Cases

Add more tests to verify the gcd function:

```rust
#[test]
fn test_gcd_edge_cases() {
    assert_eq!(gcd(1, 1), 1);
    assert_eq!(gcd(100, 10), 10);
    assert_eq!(gcd(17, 19), 1);  // Prime numbers
}
```

Run:
```bash
cargo test
```

### Exercise 2: Handle Invalid Input Gracefully

Currently, the program panics on invalid input. Try:

```bash
cargo run 24 abc
```

Observe the panic message. In a later tutorial, we'll improve this with better error handling.

### Exercise 3: Try Different Numbers

Experiment with the program:

```bash
cargo run 48 18          # Should print 6
cargo run 100 50 25      # Should print 25
cargo run 13 17          # Should print 1 (both prime)
cargo run 1071 462       # Should print 21
```

### Exercise 4: Add a Helper Function

Add a function to validate that a number is positive:

```rust
fn is_positive(n: u64) -> bool {
    n > 0
}

#[test]
fn test_is_positive() {
    assert_eq!(is_positive(5), true);
    assert_eq!(is_positive(0), false);
}
```

## Common Mistakes and Solutions

### Mistake 1: Forgetting mut

```rust
let numbers = Vec::new();
numbers.push(1);  // ERROR: cannot mutate immutable variable
```

**Solution**: Use `let mut`:
```rust
let mut numbers = Vec::new();
numbers.push(1);  // OK
```

### Mistake 2: Wrong Type

```rust
let num: u32 = u64::from_str("24").expect("error");
// ERROR: expected u32, found u64
```

**Solution**: Match types:
```rust
let num: u64 = u64::from_str("24").expect("error");
// Or use u32::from_str for u32
```

### Mistake 3: Borrowing Confusion

```rust
for m in numbers[1..] {  // ERROR: can't move out of vector
    d = gcd(d, m);
}
```

**Solution**: Borrow with `&`:
```rust
for m in &numbers[1..] {
    d = gcd(d, *m);
}
```

## Key Concepts Summary

### Ownership and Borrowing
- `Vec::new()` creates owned data
- `&numbers[1..]` borrows data
- `*m` dereferences a borrow

### Mutability
- `let` creates immutable variables
- `let mut` creates mutable variables
- Parameters can be `mut` too

### Types
- Explicit types: `u64`, `i32`, etc.
- No automatic conversion
- Type safety at compile time

### Error Handling
- `Result<T, E>` for operations that can fail
- `.expect()` for quick error handling
- Better approaches exist (covered later)

## Comparison Table: Python vs Rust

| Concept | Python | Rust |
|---------|--------|------|
| Variables | `x = 5` | `let x = 5;` |
| Mutable vars | `x = 5; x = 6` | `let mut x = 5; x = 6;` |
| Lists/Vectors | `list = []` | `let mut vec = Vec::new();` |
| Add to list | `list.append(1)` | `vec.push(1);` |
| Command args | `sys.argv[1:]` | `env::args().skip(1)` |
| Parse int | `int("24")` | `u64::from_str("24")?` |
| Print | `print(x)` | `println!("{}", x);` |
| Print debug | `print(repr(x))` | `println!("{:?}", x);` |
| Assert | `assert x == y` | `assert_eq!(x, y);` |
| Exit | `sys.exit(1)` | `std::process::exit(1);` |

## Next Steps

In the next tutorial, we'll explore file operations by building a file copy utility. You'll learn:
- Reading and writing files
- The `std::fs` module
- More sophisticated error handling with `Result`
- Working with paths

## Further Reading

- [The Rust Book - Chapter 2](https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html) - Similar beginner project
- [Rust by Example - Functions](https://doc.rust-lang.org/rust-by-example/fn.html)
- [Rust by Example - Testing](https://doc.rust-lang.org/rust-by-example/testing.html)

Congratulations! You now understand command-line argument parsing, number types, and basic algorithms in Rust. The concepts you learned here are foundational for all Rust programs.
