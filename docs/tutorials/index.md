# Tutorials

**Learning-Oriented Documentation**

Tutorials are lessons that guide you through learning Rust by building real projects. Unlike how-to guides that assume you know what you're doing, tutorials take you by the hand and teach you step by step.

## What Are Tutorials?

Tutorials are:
- **Learning-oriented**: Designed to help you learn, not just complete a task
- **Step-by-step**: Each step builds on the previous one
- **Beginner-friendly**: No assumptions about prior Rust knowledge
- **Safe to explore**: You can't break anything by following along

Think of tutorials as a cooking class where the instructor demonstrates each technique, rather than a recipe you follow on your own.

## For Python Developers

Each tutorial includes "Python Parallels" sections to help you map your existing knowledge to Rust concepts. We'll build similar functionality to what you might write in Python, showing you the Rust way.

## Suggested Learning Path

Follow this path for the best learning experience:

### Phase 1: Rust Fundamentals (Start Here!)

#### 1. [GCD - Your First Rust Program](gcd.md)
**Time**: 30 minutes | **Difficulty**: Beginner

Learn the basics of Rust syntax, functions, and control flow by implementing the greatest common divisor algorithm.

**You'll learn:**
- Writing functions in Rust
- Basic types (`u64`, etc.)
- Control flow (`if`, `while`)
- The `assert!` macro for testing
- Command-line argument parsing

**Python parallel**: Like writing your first Python function, but with type annotations everywhere.

#### 2. [Copy - Understanding Ownership](copy.md)
**Time**: 45 minutes | **Difficulty**: Beginner

Dive into Rust's most unique feature: ownership and borrowing. Learn when values are copied vs. moved.

**You'll learn:**
- The Copy trait
- Move semantics
- The difference between stack and heap
- When to use references (`&T`)
- Mutable references (`&mut T`)

**Python parallel**: Unlike Python where everything is a reference, Rust gives you explicit control over copying and moving.

#### 3. [Grep - Working with Strings](grep.md)
**Time**: 1 hour | **Difficulty**: Beginner

Build a simplified version of the `grep` command-line tool. Learn about strings, I/O, and error handling.

**You'll learn:**
- `String` vs `&str`
- Reading files
- Iterating over lines
- Error handling with `Result`
- Pattern matching

**Python parallel**: Similar to using `open()` and `for line in file:`, but with explicit error handling.

### Phase 2: Data Structures

#### 4. [Queue - Implementing a Simple Queue](queue.md)
**Time**: 45 minutes | **Difficulty**: Intermediate

Implement a basic queue data structure using `Vec<T>`.

**You'll learn:**
- Defining structs
- Implementing methods
- Working with `Vec<T>`
- Option types (`Some`/`None`)
- Public vs private APIs

**Python parallel**: Like implementing a class in Python, but with explicit memory layout and borrowing rules.

#### 5. [Generic Queue - Introduction to Generics](generic-queue.md)
**Time**: 1 hour | **Difficulty**: Intermediate

Make your queue work with any type using Rust's generics system.

**You'll learn:**
- Generic type parameters (`<T>`)
- Trait bounds
- When to use `Clone`
- The `where` clause
- Generic implementations

**Python parallel**: Similar to Python's typing generics (`list[T]`), but enforced at compile time.

#### 6. [Binary Tree - Recursive Data Structures](binary-tree.md)
**Time**: 1.5 hours | **Difficulty**: Intermediate

Build a binary search tree to learn about recursive data structures and smart pointers.

**You'll learn:**
- Recursive types
- `Box<T>` for heap allocation
- Pattern matching on enums
- Recursive functions
- Tree traversal

**Python parallel**: Like defining a tree node class, but you'll need `Box` for heap allocation.

#### 7. [Gap Buffer - Advanced Data Structures](gap-buffer.md)
**Time**: 1.5 hours | **Difficulty**: Advanced

Implement a gap buffer (used in text editors like Emacs) to master Vec manipulation.

**You'll learn:**
- Vec manipulation (`split_off`, `drain`)
- Index bounds checking
- Cursor positions
- Efficient insertion/deletion
- Maintaining invariants

**Python parallel**: Advanced list manipulation, but with explicit memory management.

### Phase 3: Type System Deep Dive

#### 8. [ASCII - Newtype Pattern](ascii.md)
**Time**: 45 minutes | **Difficulty**: Intermediate

Create a type-safe ASCII string type using the newtype pattern.

**You'll learn:**
- The newtype pattern
- Type safety at zero cost
- Implementing From/Into traits
- Validation in constructors
- Type-driven design

**Python parallel**: Like creating a class that wraps a string, but with compile-time guarantees.

#### 9. [Complex Numbers - Operator Overloading](complex.md)
**Time**: 1 hour | **Difficulty**: Intermediate

Implement complex numbers with operator overloading.

**You'll learn:**
- Operator overloading (Add, Mul, etc.)
- Generic implementations
- The `num_traits` crate
- Mathematical operators
- Type conversions

**Python parallel**: Similar to implementing `__add__`, `__mul__`, etc. in Python classes.

#### 10. [Interval - Generic Constraints](interval.md)
**Time**: 1 hour | **Difficulty**: Intermediate

Build an interval type with generic bounds.

**You'll learn:**
- Trait bounds
- The `PartialOrd` trait
- Generic constraints
- Associated types
- Where clauses

**Python parallel**: Like ABC (Abstract Base Classes) but checked at compile time.

#### 11. [Ref-with-Flag - Bit-Level Manipulation](ref-with-flag.md)
**Time**: 1.5 hours | **Difficulty**: Advanced

Store a reference and a boolean flag in a single word using bit manipulation.

**You'll learn:**
- Pointer representation
- Bit manipulation
- Alignment guarantees
- Unsafe Rust
- PhantomData

**Python parallel**: Low-level optimization not typically done in Python.

### Phase 4: Networking and Web

#### 12. [HTTP GET - Making HTTP Requests](http-get.md)
**Time**: 1 hour | **Difficulty**: Intermediate

Build a simple HTTP client using the `reqwest` crate.

**You'll learn:**
- Using external crates (dependencies)
- Async/await basics
- Error handling in async code
- The `tokio` runtime
- HTTP concepts

**Python parallel**: Similar to using `requests.get()`, but async by default.

#### 13. [Cheapo Request - Lower-Level HTTP](cheapo-request.md)
**Time**: 1.5 hours | **Difficulty**: Advanced

Implement HTTP requests at a lower level using TCP sockets.

**You'll learn:**
- TCP socket programming
- Manual HTTP request formatting
- Reading responses
- Buffer management
- Network I/O

**Python parallel**: Like using Python's `socket` module instead of `requests`.

#### 14. [Echo Server - TCP Server](echo-server.md)
**Time**: 1 hour | **Difficulty**: Intermediate

Build a TCP echo server to learn server-side networking.

**You'll learn:**
- Listening on a socket
- Accepting connections
- Reading and writing streams
- Basic concurrency
- Server architecture

**Python parallel**: Similar to Python's `socketserver` module.

#### 15. [Many Requests - Concurrent HTTP](many-requests.md)
**Time**: 1.5 hours | **Difficulty**: Advanced

Make multiple HTTP requests concurrently using async Rust.

**You'll learn:**
- Concurrent async operations
- `join!` and `try_join!`
- Error handling in concurrent code
- The `futures` crate
- Performance optimization

**Python parallel**: Like using `asyncio.gather()` in Python.

#### 16. [Many Requests Surf - Alternative HTTP Client](many-requests-surf.md)
**Time**: 1 hour | **Difficulty**: Intermediate

Same as above but using the `surf` HTTP client.

**You'll learn:**
- Alternative async HTTP clients
- API design differences
- Choosing the right crate
- Concurrent requests with `surf`

**Python parallel**: Like comparing `requests` vs `httpx` in Python.

#### 17. [Actix GCD - Web Framework](actix-gcd.md)
**Time**: 2 hours | **Difficulty**: Advanced

Build a web service using the Actix-web framework.

**You'll learn:**
- Web framework basics
- Routing and handlers
- Form handling
- HTML templates
- Web application architecture

**Python parallel**: Similar to Flask or FastAPI, but with compile-time route checking.

#### 18. [Basic Router - Custom Routing](basic-router.md)
**Time**: 1.5 hours | **Difficulty**: Advanced

Implement a simple HTTP router from scratch.

**You'll learn:**
- URL parsing and matching
- Handler dispatch
- Router design patterns
- Pattern matching for routing
- Building framework components

**Python parallel**: Understanding how Flask/Django routing works under the hood.

### Phase 5: Async and Concurrency

#### 19. [Block-On - Async Runtime Basics](block-on.md)
**Time**: 1.5 hours | **Difficulty**: Advanced

Understand async runtimes by implementing a simple executor.

**You'll learn:**
- How async/await works under the hood
- Futures and polling
- Building a basic executor
- The async runtime model
- Wakers and task scheduling

**Python parallel**: Understanding asyncio's event loop internals.

#### 20. [Spawn-Blocking - Mixing Sync and Async](spawn-blocking.md)
**Time**: 1 hour | **Difficulty**: Advanced

Learn to integrate blocking operations with async code.

**You'll learn:**
- `spawn_blocking` pattern
- Thread pools
- When to use blocking vs async
- Performance considerations
- Hybrid architectures

**Python parallel**: Similar to `asyncio.to_thread()` in Python 3.9+.

### Phase 6: Foreign Function Interface (FFI)

#### 21. [libgit2-rs - Unsafe FFI](libgit2-rs.md)
**Time**: 2 hours | **Difficulty**: Advanced

Call C library functions (libgit2) using unsafe Rust.

**You'll learn:**
- FFI basics
- Unsafe Rust
- Working with C types
- Memory management across FFI boundary
- Null pointer handling

**Python parallel**: Similar to using `ctypes`, but with manual memory management.

#### 22. [libgit2-rs-safe - Safe FFI Wrappers](libgit2-rs-safe.md)
**Time**: 2 hours | **Difficulty**: Advanced

Wrap unsafe FFI code in safe Rust abstractions.

**You'll learn:**
- Safe wrapper patterns
- RAII (Resource Acquisition Is Initialization)
- Drop trait for cleanup
- Hiding unsafety behind safe APIs
- Best practices for FFI

**Python parallel**: Like creating a Pythonic wrapper around a C library.

### Phase 7: Metaprogramming

#### 23. [JSON Macro - Declarative Macros](json-macro.md)
**Time**: 2 hours | **Difficulty**: Advanced

Build a JSON literal macro using Rust's macro system.

**You'll learn:**
- Declarative macros (`macro_rules!`)
- Pattern matching in macros
- Repetition patterns
- Macro hygiene
- Code generation

**Python parallel**: More powerful than decorators, similar to AST manipulation.

### Phase 8: Putting It All Together

#### 24. [Fern Sim - Complete Application](fern_sim.md)
**Time**: 2-3 hours | **Difficulty**: Advanced

Build a complete fern simulation application integrating multiple concepts.

**You'll learn:**
- Project structure
- Multiple modules
- External crate integration
- Image generation
- Real-world application design

**Python parallel**: Like building a complete Python application with multiple modules.

## How to Use These Tutorials

1. **Follow the order**: Each tutorial builds on previous concepts
2. **Type the code**: Don't copy-paste. Typing helps muscle memory
3. **Experiment**: After each tutorial, modify the code and see what breaks
4. **Read compiler errors**: Rust's compiler errors are educational
5. **Take breaks**: Some concepts (like ownership) take time to internalize

## Getting Help

- **Compiler errors**: Read them carefully - they often suggest fixes
- **Documentation**: Use `cargo doc --open` to see local docs
- **Community**:
  - [Rust Users Forum](https://users.rust-lang.org/)
  - [Rust Discord](https://discord.gg/rust-lang)
  - [r/rust](https://reddit.com/r/rust)

## After Tutorials: What's Next?

Once you've completed the tutorials:

1. **Practice**: Try reimplementing Python projects you've built
2. **Read**: The [Rust Book](https://doc.rust-lang.org/book/) for deeper coverage
3. **Build**: Start your own Rust project
4. **Explore**: Check out [How-To Guides](../how-to/index.md) for specific tasks
5. **Deepen**: Read [Explanations](../explanation/index.md) for conceptual understanding

---

*Remember: Learning Rust is a journey. The ownership system will click suddenly after some practice. Keep going!*
