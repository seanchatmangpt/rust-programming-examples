# 1. BINARY WITH MAIN FUNCTION

*A simple executable program, complete in a single file, waiting to be run from the command line.*

...within a **RUST PROJECT STRUCTURE**, when you need to create a standalone program that users can execute directly...

◆ ◆ ◆

**You need to build something that runs on its own—a tool, a command-line utility, or a standalone application that performs a specific task when invoked.**

The temptation is to create elaborate structures immediately, with multiple modules and complex organization. But many useful programs need only one thing: a clear entry point that does the work and exits cleanly.

In Rust, every executable program begins its life at `fn main()`. This is not merely convention—it is the contract between your code and the operating system. When a user runs your program, the OS looks for this function and executes it. Everything flows from this single point.

Consider the `gcd` program, which calculates the greatest common divisor of numbers. It needs no complex architecture. It reads command-line arguments, computes a result, and prints it. The entire program lives in one file: `src/main.rs`. This simplicity is not a limitation—it is clarity.

The pattern emerges from observation: the smallest useful programs need only `main()`, the algorithm, and perhaps a test. Adding more structure before you need it obscures the essential logic. The `main()` function becomes the program's story: "Parse arguments, compute answer, display result."

**Therefore:**

**Place your executable code in `src/main.rs` with a `fn main()` entry point. Keep the program's logic close to main until complexity demands separation.**

```rust
use std::str::FromStr;
use std::env;

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

#[test]
fn test_gcd() {
    assert_eq!(gcd(14, 15), 1);
    assert_eq!(gcd(2 * 3 * 5 * 11 * 17,
                   3 * 7 * 11 * 13 * 19),
               3 * 11);
}
```

*The program structure: Cargo.toml declares `name = "gcd"`, and `src/main.rs` contains everything. Cargo knows to build a binary executable.*

◆ ◆ ◆

This pattern naturally leads to **TESTS DIRECTORY BESIDE SOURCE** (4) as the program grows, and eventually to **BINARY AND LIBRARY TOGETHER** (3) when you discover the core algorithm deserves reuse.
