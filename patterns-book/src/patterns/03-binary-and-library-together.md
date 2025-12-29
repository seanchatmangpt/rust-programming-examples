# 3. BINARY AND LIBRARY TOGETHER

*A project that provides both reusable logic and a ready-to-run tool, each serving different audiences.*

...within a **RUST PROJECT STRUCTURE**, when you have valuable logic that should be reusable, but also want to provide a convenient command-line interface...

◆ ◆ ◆

**You've built a useful library with core functionality, but users keep asking: "How do I just run it?" Or conversely, you've built a great command-line tool, and developers keep asking: "Can I use this logic in my own code?"**

There is a tension between two valid needs. Library users want clean APIs they can integrate into their programs. End users want executable tools they can run immediately. You could create two separate projects, but then the logic diverges—fixes to one don't automatically benefit the other.

Rust's project structure elegantly resolves this: a single crate can contain both `src/lib.rs` (the library) and `src/main.rs` (the binary). The binary becomes a thin wrapper around the library, demonstrating its use while providing immediate utility.

Consider how this would work for the `gcd` program. The core `gcd()` function could live in `src/lib.rs`, marked `pub` so other crates can use it. The `src/main.rs` file then becomes a command-line interface to this functionality—parsing arguments, calling the library function, and formatting output.

This pattern creates a natural separation of concerns. The library contains the *what*—the algorithms, data structures, and domain logic. The binary contains the *how*—the user interface, argument parsing, and formatting. Each can be tested independently. Each serves its audience.

The library doesn't need to know about command-line arguments or `println!`. The binary doesn't need to understand the algorithm. This is more than organization—it's a clear statement about where responsibilities lie.

**Therefore:**

**Place core logic in `src/lib.rs` with public APIs. Create `src/main.rs` as a thin binary that demonstrates and provides command-line access to the library. The binary depends on the library through its public API.**

```rust
// src/lib.rs
/// Compute the greatest common divisor of two numbers.
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

#[test]
fn test_gcd() {
    assert_eq!(gcd(14, 15), 1);
    assert_eq!(gcd(2 * 3 * 5 * 11 * 17,
                   3 * 7 * 11 * 13 * 19),
               3 * 11);
}
```

```rust
// src/main.rs
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
        // Call the library function
        d = gcd::gcd(d, *m);
    }

    println!("The greatest common divisor of {:?} is {}",
             numbers, d);
}
```

*Two files, two audiences: `lib.rs` serves programmers who need the algorithm; `main.rs` serves users who need the tool. Both benefit from shared implementation.*

◆ ◆ ◆

The binary serves as the first example in **EXAMPLES DIRECTORY FOR USAGE** (5). The library needs **TESTS DIRECTORY BESIDE SOURCE** (4) to verify the public API. As complexity grows, the library will naturally lead to **MODULE FILES BESIDE LIB** for organization.
