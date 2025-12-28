# Tutorial 1: Getting Started with Rust

## Introduction

Welcome to Rust! If you're coming from Python, you'll find Rust both familiar and different. This tutorial will help you set up your development environment and run your first Rust program.

## What You'll Learn

- Installing Rust and essential tools
- Understanding the Cargo build system
- Exploring Rust project structure
- Running your first Rust program
- Key differences from Python development

## Prerequisites

- Basic command-line knowledge
- Familiarity with programming concepts (from Python or another language)
- A computer with a modern operating system (Linux, macOS, or Windows)

## Step 1: Install Rust

### Installing Rustup

Rust's installation is managed by `rustup`, which handles the compiler and toolchain versions.

On Linux or macOS, open your terminal and run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Windows, download and run the installer from [https://rustup.rs](https://rustup.rs).

After installation, restart your terminal and verify the installation:

```bash
rustc --version
cargo --version
```

You should see output like:
```
rustc 1.XX.0 (hash date)
cargo 1.XX.0 (hash date)
```

### Python Parallel: pip and virtualenv

If you're familiar with Python:
- `rustup` is like `pyenv` - it manages versions
- You just installed the Rust compiler (`rustc`) and package manager (`cargo`)
- Unlike Python, Rust compiles to native binaries (no interpreter needed)

## Step 2: Understanding Cargo

Cargo is Rust's build system and package manager. It's like `pip` + `setuptools` + `virtualenv` combined into one tool.

### What Cargo Does

1. **Creates projects**: `cargo new` (like `mkdir` + `setup.py` creation)
2. **Builds code**: `cargo build` (like `python setup.py build`)
3. **Runs programs**: `cargo run` (like `python script.py`)
4. **Runs tests**: `cargo test` (like `pytest`)
5. **Manages dependencies**: via `Cargo.toml` (like `requirements.txt` or `pyproject.toml`)

### Key Cargo Commands

```bash
cargo new my-project     # Create a new project
cargo build              # Compile the project
cargo run                # Compile and run
cargo test               # Run tests
cargo check              # Check code without building
cargo build --release    # Build optimized version
```

## Step 3: Clone the Examples Repository

Now let's get the example projects:

```bash
cd ~
git clone https://github.com/yourusername/rust-programming-examples.git
cd rust-programming-examples
```

Take a look at the directory structure:

```bash
ls -la
```

You'll see several directories, each containing a small Rust project:
- `gcd/` - Greatest Common Divisor calculator
- `copy/` - File copy utility
- `grep/` - Pattern search tool
- And others...

## Step 4: Explore a Rust Project Structure

Let's explore the `gcd` project:

```bash
cd gcd
ls -la
```

You'll see:
```
.
├── Cargo.toml       # Project metadata and dependencies
└── src/
    └── main.rs      # Main source code
```

### Understanding Cargo.toml

Open `Cargo.toml`:

```toml
[package]
name = "gcd"
version = "0.1.0"
authors = ["You <you@example.com>"]
edition = "2018"

[dependencies]
```

**Python Parallel**:
- This is like `setup.py` or `pyproject.toml`
- `[package]` section: metadata (name, version, authors)
- `[dependencies]` section: like `install_requires` in setup.py
- The `edition` field specifies the Rust language edition (2018, 2021, etc.)

### Understanding src/main.rs

The `src/main.rs` file contains your program's entry point:

```rust
fn main() {
    println!("Hello, world!");
}
```

**Python Parallel**:
```python
# Python
def main():
    print("Hello, world!")

if __name__ == "__main__":
    main()
```

In Rust, `fn main()` is automatically the entry point - no `if __name__ == "__main__"` needed!

## Step 5: Run Your First Rust Program

Still in the `gcd` directory, run:

```bash
cargo run
```

You'll see output like:
```
   Compiling gcd v0.1.0 (/path/to/gcd)
    Finished dev [unoptimized + debuginfo] target(s) in 1.23s
     Running `target/debug/gcd`
Usage: gcd NUMBER ...
```

### What Just Happened?

1. **Compilation**: Cargo compiled your Rust code to a native binary
2. **Build artifacts**: Created in `target/debug/gcd`
3. **Execution**: Ran the compiled binary

Unlike Python, Rust is compiled before execution. The first compilation might be slow, but subsequent runs are much faster.

### Try It With Arguments

The GCD program expects numbers as arguments:

```bash
cargo run 24 60
```

Output:
```
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/gcd 24 60`
The greatest common divisor of [24, 60] is 12
```

Try more numbers:

```bash
cargo run 100 50 25
```

Output:
```
The greatest common divisor of [100, 50, 25] is 25
```

## Step 6: Understanding Compilation vs Interpretation

### Python (Interpreted)

```python
# file: hello.py
print("Hello")

# Run it:
$ python hello.py
# Python reads the file and executes it line by line
```

### Rust (Compiled)

```rust
// file: main.rs
fn main() {
    println!("Hello");
}

// Compile it:
$ cargo build
// Creates a binary: target/debug/hello

// Run the binary:
$ ./target/debug/hello
// Or use cargo run to do both:
$ cargo run
```

**Key Differences**:
- Rust catches errors at compile time
- Rust programs run faster (native code)
- Rust requires an explicit compilation step
- Python is more flexible for quick scripts

## Step 7: Build Modes

Rust has two main build modes:

### Debug Mode (Default)

```bash
cargo build
# Or
cargo run
```

- Faster compilation
- Larger binaries
- Includes debug symbols
- No optimizations
- Use for development

### Release Mode

```bash
cargo build --release
cargo run --release
```

- Slower compilation
- Smaller binaries
- Full optimizations
- Use for production

**Python Parallel**: Like the difference between running Python normally vs. with `-O` flag, but much more significant in Rust.

## Step 8: Working with Multiple Projects

The repository contains multiple independent projects. Each has its own `Cargo.toml`.

To work on a different project:

```bash
cd ~/rust-programming-examples/copy
cargo run
```

Each project is self-contained - no virtual environments needed!

## Step 9: Checking Your Code

Before building, you can quickly check for errors:

```bash
cargo check
```

This is faster than `cargo build` because it doesn't produce binaries.

**Python Parallel**: Like running `mypy` or `pylint`, but it's part of the core toolchain.

## Step 10: Running Tests

Rust has built-in testing support. In the `gcd` project:

```bash
cargo test
```

Output:
```
   Compiling gcd v0.1.0
    Finished test [unoptimized + debuginfo] target(s)
     Running unittests (target/debug/deps/gcd-...)

running 1 test
test test_gcd ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured
```

Tests are written in the same file as the code (by convention), marked with `#[test]`.

## Common Issues and Solutions

### Issue: "command not found: cargo"

**Solution**: Restart your terminal or run:
```bash
source $HOME/.cargo/env
```

### Issue: Permission denied during installation

**Solution**: Don't use `sudo` with rustup. Install in your home directory.

### Issue: Slow first compilation

**Solution**: This is normal. Rust compiles all dependencies and your code. Subsequent compilations use caching and are much faster.

## Python Developer Quick Reference

| Python | Rust Equivalent | Purpose |
|--------|----------------|---------|
| `pip install` | `cargo add` (or edit Cargo.toml) | Add dependency |
| `python script.py` | `cargo run` | Run program |
| `pytest` | `cargo test` | Run tests |
| `pylint` / `mypy` | `cargo check` / `cargo clippy` | Check code |
| `requirements.txt` | `Cargo.toml` | Dependencies |
| `.py` files | `.rs` files | Source code |
| `if __name__ == "__main__"` | `fn main()` | Entry point |

## Summary

Congratulations! You've:
- Installed Rust and Cargo
- Cloned the examples repository
- Understood Rust project structure
- Run your first Rust program (gcd)
- Learned key differences from Python

## Next Steps

In the next tutorial, we'll dive deep into the GCD example to understand:
- Command-line argument parsing
- Rust's type system
- Error handling
- Basic algorithms in Rust

## Further Reading

- [The Rust Book](https://doc.rust-lang.org/book/) - Official comprehensive guide
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Learn by examples
- [Cargo Book](https://doc.rust-lang.org/cargo/) - Deep dive into Cargo

## Exercises

1. Create a new Rust project with `cargo new hello-rust` and make it print your name
2. Add a test to your new project that always passes
3. Build the `grep` and `copy` projects in release mode
4. Find the compiled binaries in the `target/` directory and run them directly

Remember: Rust's learning curve is steeper than Python's, but the benefits (performance, safety, concurrency) are worth it. Take your time and experiment!
