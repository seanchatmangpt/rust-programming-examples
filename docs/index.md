# Rust Programming Examples

Welcome to the Rust Programming Examples documentation! This repository is designed to help Python developers learn Rust through practical, hands-on examples.

## For Python Developers

If you're coming from Python, you'll find Rust both familiar and different. Like Python, Rust emphasizes readability and expressiveness. Unlike Python, Rust is statically typed and compiled, offering performance comparable to C/C++ with memory safety guarantees.

Key differences you'll encounter:
- **Ownership & Borrowing**: Rust's unique memory management system (no garbage collector!)
- **Explicit Types**: Though Rust has type inference, types are checked at compile time
- **Error Handling**: Result types instead of exceptions
- **Immutability by Default**: Variables are immutable unless declared with `mut`
- **Pattern Matching**: More powerful than Python's match statement

## Documentation Structure (Diataxis Framework)

This documentation follows the [Diataxis framework](https://diataxis.fr/), organizing content by your needs:

### [Tutorials](tutorials/index.md) - Learning-Oriented
**When you want to learn by doing**

Step-by-step lessons that take you from beginner to confident. Start here if you're new to Rust or want structured learning.

- Start with fundamentals (GCD, copying values)
- Progress through data structures
- Build real-world applications (web servers, HTTP clients)

### [How-To Guides](how-to/index.md) - Task-Oriented
**When you need to accomplish a specific task**

Practical guides for solving specific problems. Use these when you know what you want to do.

- Implement common data structures
- Build web servers and HTTP clients
- Work with async/await
- Interface with C libraries via FFI
- Create custom macros

### [Reference](reference/index.md) - Information-Oriented
**When you need to look something up**

Technical descriptions of each project's API, modules, and functions. Your quick lookup resource.

- API documentation for all 24 projects
- Function signatures and types
- Module organization

### [Explanation](explanation/index.md) - Understanding-Oriented
**When you want to understand concepts deeply**

Deep dives into Rust concepts and design decisions. Read these to understand *why* things work the way they do.

- Ownership and borrowing explained
- Type system deep dive
- Async runtime internals
- FFI safety guarantees
- Macro system concepts

## Prerequisites

Before diving in, ensure you have:

1. **Rust Toolchain** (rustc, cargo)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Basic Programming Knowledge**
   - Variables, functions, control flow
   - If you know Python, you're ready!

3. **A Text Editor or IDE**
   - VS Code with rust-analyzer extension (recommended)
   - IntelliJ IDEA with Rust plugin
   - Any editor with LSP support

## Repository Contents

This repository contains 24 example projects organized by category:

- **Fundamentals**: gcd, copy, grep
- **Data Structures**: queue, generic-queue, binary-tree, gap-buffer
- **Types**: ascii, complex, interval, ref-with-flag
- **Web/Networking**: http-get, cheapo-request, echo-server, many-requests, many-requests-surf, actix-gcd, basic-router
- **Async/Concurrency**: block-on, spawn-blocking
- **FFI**: libgit2-rs, libgit2-rs-safe
- **Macros**: json-macro
- **Simulation**: fern_sim

## Quick Start

1. **New to Rust?** Start with [Tutorials](tutorials/index.md)
2. **Have a specific task?** Check [How-To Guides](how-to/index.md)
3. **Need API details?** See [Reference](reference/index.md)
4. **Want deeper understanding?** Read [Explanations](explanation/index.md)

## Running Examples

Each project can be run independently:

```bash
cd <project-name>
cargo run
```

Run tests:
```bash
cargo test
```

Check your code:
```bash
cargo check
cargo clippy
```

## Python to Rust Quick Reference

| Python Concept | Rust Equivalent | Example Project |
|---------------|-----------------|-----------------|
| List | `Vec<T>` | queue, generic-queue |
| Dictionary | `HashMap<K, V>` | (various) |
| None | `Option<T>` | (various) |
| try/except | `Result<T, E>` | http-get, cheapo-request |
| @decorator | Attributes/Macros | json-macro |
| async/await | async/await | many-requests-surf |
| ctypes/cffi | FFI | libgit2-rs, libgit2-rs-safe |

## Contributing

Found an issue or want to add an example? Contributions are welcome! Please check the main repository README for contribution guidelines.

## Additional Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust for Python Programmers](https://github.com/rochacbruno/py2rs)
- [Rustlings - Interactive Exercises](https://github.com/rust-lang/rustlings)

---

*Happy learning! Remember: the Rust compiler is your friend, not your enemy. Those error messages are there to help you write better code.*
