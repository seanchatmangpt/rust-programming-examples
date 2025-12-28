# Programming Rust: Code Examples

## Project Overview

This is the **official code examples repository** for "Programming Rust" by Jim Blandy, Jason Orendorff, and Leonora Tindall. It contains 24 complete, self-contained Rust projects organized by book chapter, demonstrating Rust language features, patterns, and best practices.

Each subdirectory is an independent Rust project with its own `Cargo.toml` file, buildable and testable in isolation.

**Repository**: https://github.com/ProgrammingRust/code-examples
**License**: MIT License (see LICENSE-MIT)

---

## Stack & Versions

- **Language**: Rust
- **Edition**: 2018 (all projects)
- **MSRV** (Minimum Supported Rust Version): 1.56+ (typical for code examples from circa 2021-2022)
- **Target Platform**: Linux, macOS, Windows (cross-platform compatible)

### Key Dependencies by Project Type

| Feature Area | Projects | Key Crates | Versions |
|--------------|----------|-----------|----------|
| Web Frameworks | actix-gcd | actix-web | 4.1+ |
| HTTP Clients | http-get | reqwest | 0.11 (with blocking) |
| Async Runtime | cheapo-request, many-requests | async-std | 1.7+ |
| FFI | libgit2-rs, libgit2-rs-safe | Manual C bindings | — |
| Macros | json-macro | (procedural macros) | — |
| Testing | fern_sim, spawn-blocking | — | — |

---

## Repository Map

```
rust-programming-examples/
├── README.md                          # Chapter-by-chapter guide
├── LICENSE-MIT                        # MIT license
├── CLAUDE.md                          # This file - AI assistant guide
│
├── gcd/                               # Ch. 2: Simple CLI program
├── actix-gcd/                         # Ch. 2: Web service (actix-web)
│
├── fern_sim/                          # Ch. 8: Module structure
│   ├── src/lib.rs
│   ├── src/simulation.rs
│   ├── src/spores.rs
│   └── tests/
│
├── queue/                             # Ch. 9: Basic struct type
├── generic-queue/                     # Ch. 9: Generic types
│
├── binary-tree/                       # Ch. 10 & 15: Enums + Iterators
├── basic-router/                      # Ch. 14: Closures & callbacks
│
├── complex/                           # Ch. 12 & 17: Operator overloading + Display
├── interval/                          # Ch. 12: PartialOrd trait
│
├── grep/                              # Ch. 18: CLI tool for text search
├── copy/                              # Ch. 18: Directory tree copying
├── echo-server/                       # Ch. 18: Simple network service
├── http-get/                          # Ch. 18: HTTP client (reqwest)
│
├── cheapo-request/                    # Ch. 20: Async HTTP (async-std)
├── many-requests/                     # Ch. 20: Concurrent requests
├── many-requests-surf/                # Ch. 20: Async HTTP (surf crate)
├── spawn-blocking/                    # Ch. 20: Custom async primitives
├── block-on/                          # Ch. 20: Simple executor
│
├── json-macro/                        # Ch. 21: Procedural macros
│
├── ascii/                             # Ch. 22: Unsafe blocks & functions
├── ref-with-flag/                     # Ch. 22: Raw pointers
├── gap-buffer/                        # Ch. 22: Pointer arithmetic
│
├── libgit2-rs/                        # Ch. 22 FFI: Unsafe FFI bindings
└── libgit2-rs-safe/                   # Ch. 22 FFI: Safe wrapper around libgit2
```

### Project Categories by Purpose

**Basic Examples** (Single-file implementations):
- `gcd`, `queue`, `generic-queue`, `interval`, `complex`, `echo-server`, `block-on`

**Binary Programs** (Executable tools):
- `gcd`, `grep`, `copy`, `http-get`, `echo-server`, `many-requests-surf`

**Library Examples** (Reusable components):
- `queue`, `generic-queue`, `binary-tree`, `complex`, `interval`, `ascii`, `ref-with-flag`, `gap-buffer`

**Web/Network** (HTTP and web services):
- `actix-gcd`, `http-get`, `echo-server`, `cheapo-request`, `many-requests`

**Advanced Features**:
- **Async/Await**: `cheapo-request`, `many-requests`, `many-requests-surf`, `spawn-blocking`
- **Unsafe Code**: `ascii`, `ref-with-flag`, `gap-buffer`
- **FFI**: `libgit2-rs`, `libgit2-rs-safe`
- **Macros**: `json-macro`
- **Modules**: `fern_sim`

---

## Project Difficulty & Learning Paths

This section organizes all 24 projects by difficulty level, estimated completion time, and key learning concepts. Use this guide to plan your learning journey through the repository.

### Difficulty Legend

- 🟢 **Beginner**: Fundamental Rust concepts, minimal complexity, single-file implementations
- 🟡 **Intermediate**: Multi-file projects, advanced traits, moderate external dependencies
- 🔴 **Advanced**: Unsafe code, FFI, async runtime internals, macros, complex architecture

### Complete Project Catalog

| Project | Chapter | Difficulty | Est. Time | Key Concepts | Type |
|---------|---------|------------|-----------|--------------|------|
| **gcd** | Ch. 2 | 🟢 Beginner | 30 min | CLI args, functions, loops | Binary |
| **actix-gcd** | Ch. 2 | 🟡 Intermediate | 1.5 hrs | Web framework, HTTP, routing, forms | Binary |
| **fern_sim** | Ch. 8 | 🟡 Intermediate | 2 hrs | Module structure, visibility, organization | Library |
| **queue** | Ch. 9 | 🟢 Beginner | 45 min | Structs, methods, ownership | Library |
| **generic-queue** | Ch. 9 | 🟡 Intermediate | 1 hr | Generic types, type parameters, constraints | Library |
| **binary-tree** | Ch. 10 & 15 | 🟡 Intermediate | 2 hrs | Enums, recursive types, iterators | Library |
| **complex** | Ch. 12 & 17 | 🟢 Beginner | 1 hr | Operator overloading, Add/Sub/Mul traits | Library |
| **interval** | Ch. 12 | 🟡 Intermediate | 1 hr | PartialOrd, trait bounds, comparisons | Library |
| **basic-router** | Ch. 14 | 🟡 Intermediate | 1.5 hrs | Closures, callbacks, function types | Library |
| **grep** | Ch. 18 | 🟡 Intermediate | 2 hrs | I/O, error handling, CLI tools | Binary |
| **copy** | Ch. 18 | 🟡 Intermediate | 1.5 hrs | Filesystem, recursion, path handling | Binary |
| **echo-server** | Ch. 18 | 🟢 Beginner | 1 hr | TCP networking, basic I/O | Binary |
| **http-get** | Ch. 18 | 🟢 Beginner | 45 min | HTTP client, reqwest, blocking I/O | Binary |
| **cheapo-request** | Ch. 20 | 🟡 Intermediate | 2 hrs | async/await basics, async-std, futures | Binary |
| **many-requests** | Ch. 20 | 🟡 Intermediate | 2 hrs | Concurrent async, join operations | Binary |
| **many-requests-surf** | Ch. 20 | 🟡 Intermediate | 1.5 hrs | Surf crate, async HTTP patterns | Binary |
| **spawn-blocking** | Ch. 20 | 🔴 Advanced | 3 hrs | Custom async primitives, executor internals | Library |
| **block-on** | Ch. 20 | 🔴 Advanced | 2.5 hrs | Simple executor implementation, polling | Library |
| **json-macro** | Ch. 21 | 🔴 Advanced | 4 hrs | Procedural macros, TokenStream, syntax | Library |
| **ascii** | Ch. 22 | 🔴 Advanced | 2 hrs | Unsafe blocks, transmute, invariants | Library |
| **ref-with-flag** | Ch. 22 | 🔴 Advanced | 2.5 hrs | Raw pointers, bit manipulation, unsafe | Library |
| **gap-buffer** | Ch. 22 | 🔴 Advanced | 3 hrs | Pointer arithmetic, Vec internals, unsafe | Library |
| **libgit2-rs** | Ch. 22 | 🔴 Advanced | 4 hrs | FFI, C bindings, extern blocks, raw types | Library |
| **libgit2-rs-safe** | Ch. 22 | 🔴 Advanced | 3 hrs | Safe wrapper design, lifetime management | Library |

### Project Distribution by Difficulty

**🟢 Beginner (6 projects)**: `gcd`, `queue`, `complex`, `echo-server`, `http-get`, and parts of `interval`
- Total estimated time: ~6 hours
- Focus: Core Rust syntax, basic ownership, simple traits

**🟡 Intermediate (11 projects)**: `actix-gcd`, `fern_sim`, `generic-queue`, `binary-tree`, `interval`, `basic-router`, `grep`, `copy`, `cheapo-request`, `many-requests`, `many-requests-surf`
- Total estimated time: ~18-20 hours
- Focus: Advanced traits, generics, async basics, multi-module architecture

**🔴 Advanced (7 projects)**: `spawn-blocking`, `block-on`, `json-macro`, `ascii`, `ref-with-flag`, `gap-buffer`, `libgit2-rs`, `libgit2-rs-safe`
- Total estimated time: ~24 hours
- Focus: Unsafe code, FFI, macro systems, runtime internals

### Learning Time Estimates

**Quick Introduction** (1-3 hours per project):
- `gcd`, `queue`, `complex`, `echo-server`, `http-get`
- Ideal for first exposure to Rust concepts

**Moderate Depth** (1.5-2.5 hours per project):
- `actix-gcd`, `fern_sim`, `generic-queue`, `binary-tree`, `grep`, `copy`, `cheapo-request`, `many-requests`
- Require understanding of intermediate concepts

**Deep Dive** (3-4 hours per project):
- `spawn-blocking`, `json-macro`, `gap-buffer`, `libgit2-rs`, `libgit2-rs-safe`
- Demand careful study and experimentation

### Concept Coverage Map

| Concept | Beginner Projects | Intermediate Projects | Advanced Projects |
|---------|------------------|----------------------|-------------------|
| **Ownership & Borrowing** | gcd, queue | generic-queue, binary-tree | gap-buffer, ascii |
| **Structs & Enums** | queue, complex | binary-tree, interval | ref-with-flag |
| **Traits** | complex | interval, basic-router | ascii, gap-buffer |
| **Generics** | — | generic-queue, binary-tree | — |
| **Modules** | — | fern_sim, grep | — |
| **Error Handling** | — | grep, copy, http-get | libgit2-rs-safe |
| **Closures** | — | basic-router | — |
| **Iterators** | — | binary-tree | — |
| **I/O & Networking** | echo-server, http-get | grep, copy | — |
| **Web Development** | — | actix-gcd | — |
| **Async/Await** | — | cheapo-request, many-requests | spawn-blocking, block-on |
| **Unsafe Code** | — | — | ascii, ref-with-flag, gap-buffer |
| **FFI** | — | — | libgit2-rs, libgit2-rs-safe |
| **Macros** | — | — | json-macro |

---

### Recommended Learning Sequences

Choose a learning path that matches your current skill level and goals. Each path builds progressively on previous concepts.

#### Path 1: Complete Beginner → Rust Mastery

**Phase 1: Fundamentals** (6-8 hours)
```
1. gcd (30 min)
   └─> Learn: Functions, loops, CLI arguments, cargo basics

2. queue (45 min)
   └─> Learn: Structs, methods, ownership, borrowing

3. complex (1 hr)
   └─> Learn: Operator overloading, trait implementation

4. echo-server (1 hr)
   └─> Learn: TCP networking, basic I/O, error handling

5. http-get (45 min)
   └─> Learn: External crates, HTTP clients, blocking I/O
```

**Phase 2: Intermediate Concepts** (12-15 hours)
```
6. generic-queue (1 hr)
   └─> Learn: Generic types, type parameters, Vec evolution

7. binary-tree (2 hrs)
   └─> Learn: Recursive enums, pattern matching, iterators

8. interval (1 hr)
   └─> Learn: PartialOrd, comparisons, trait bounds

9. fern_sim (2 hrs)
   └─> Learn: Multi-module projects, pub/visibility, organization

10. basic-router (1.5 hrs)
    └─> Learn: Closures, function types, callbacks

11. grep (2 hrs)
    └─> Learn: CLI tools, file I/O, regex, error propagation

12. copy (1.5 hrs)
    └─> Learn: Filesystem operations, recursion, path handling

13. actix-gcd (1.5 hrs)
    └─> Learn: Web frameworks, routing, HTML forms, HTTP
```

**Phase 3: Async Programming** (6-8 hours)
```
14. cheapo-request (2 hrs)
    └─> Learn: async/await basics, futures, async-std runtime

15. many-requests (2 hrs)
    └─> Learn: Concurrent async, join operations, parallelism

16. many-requests-surf (1.5 hrs)
    └─> Learn: Alternative async HTTP clients, API differences
```

**Phase 4: Advanced Topics** (12-16 hours)
```
17. spawn-blocking (3 hrs)
    └─> Learn: Custom async primitives, blocking operations

18. block-on (2.5 hrs)
    └─> Learn: Executor implementation, polling, Waker

19. ascii (2 hrs)
    └─> Learn: Unsafe blocks, transmute, safety invariants

20. ref-with-flag (2.5 hrs)
    └─> Learn: Raw pointers, bit manipulation, alignment

21. gap-buffer (3 hrs)
    └─> Learn: Pointer arithmetic, manual memory management

22. libgit2-rs (4 hrs)
    └─> Learn: FFI basics, extern blocks, C interop

23. libgit2-rs-safe (3 hrs)
    └─> Learn: Safe wrapper patterns, lifetime management

24. json-macro (4 hrs)
    └─> Learn: Procedural macros, TokenStream, quote crate
```

**Total Time**: ~40-50 hours for complete mastery

---

#### Path 2: Experienced Developer Fast Track

Skip basics, focus on Rust-specific features and advanced patterns.

**Stage 1: Rust Ownership Model** (3-4 hours)
```
1. queue → generic-queue (Combined study: 2 hrs)
   └─> Compare: Reference semantics vs Rust ownership

2. binary-tree (2 hrs)
   └─> Focus: Recursive ownership, enum variants
```

**Stage 2: Trait System & Abstractions** (4-5 hours)
```
3. complex (1 hr)
   └─> Study: Operator overloading patterns

4. interval (1 hr)
   └─> Study: PartialOrd, comparison traits

5. basic-router (1.5 hrs)
   └─> Study: Closures vs function pointers

6. binary-tree iterators (Re-visit: 1 hr)
   └─> Study: Iterator trait, custom iteration
```

**Stage 3: Real-World Applications** (6-8 hours)
```
7. actix-gcd (1.5 hrs)
   └─> Build: Web service with actix-web

8. grep (2 hrs)
   └─> Build: CLI tool with proper error handling

9. fern_sim (2 hrs)
   └─> Study: Large-scale module organization
```

**Stage 4: Async Rust** (6-8 hours)
```
10. cheapo-request (2 hrs)
    └─> Learn: async/await syntax and semantics

11. many-requests (2 hrs)
    └─> Practice: Concurrent async operations

12. spawn-blocking (3 hrs)
    └─> Deep dive: Async runtime internals

13. block-on (Optional: 2.5 hrs)
    └─> Understand: Executor implementation
```

**Stage 5: Unsafe Rust & FFI** (10-12 hours)
```
14. ascii (2 hrs)
    └─> Introduction: Unsafe blocks, invariants

15. ref-with-flag (2.5 hrs)
    └─> Practice: Raw pointer manipulation

16. gap-buffer (3 hrs)
    └─> Master: Pointer arithmetic, manual memory

17. libgit2-rs → libgit2-rs-safe (Combined: 6-7 hrs)
    └─> Complete: FFI and safe wrapper patterns
```

**Stage 6: Metaprogramming** (4 hours)
```
18. json-macro (4 hrs)
    └─> Advanced: Procedural macro development
```

**Total Time**: ~33-41 hours for comprehensive coverage

---

#### Path 3: Specialization Tracks

Focus on specific Rust domains based on your goals.

**Track A: Async/Concurrency Expert** (14-16 hours)
```
Prerequisites:
  - Basic Rust: gcd, queue, complex (2.5 hrs)

Core Sequence:
1. echo-server (1 hr)
   └─> Foundation: Synchronous networking

2. http-get (45 min)
   └─> Foundation: Blocking HTTP

3. cheapo-request (2 hrs)
   └─> Async basics: async-std, futures

4. many-requests (2 hrs)
   └─> Concurrency: join, parallel requests

5. many-requests-surf (1.5 hrs)
   └─> Alternative: surf crate patterns

6. spawn-blocking (3 hrs)
   └─> Internals: Custom async primitives

7. block-on (2.5 hrs)
   └─> Deep dive: Executor from scratch

Advanced Topics:
  - Study tokio vs async-std differences
  - Benchmark async vs threaded approaches
  - Experiment with different executors
```

**Track B: Unsafe & Systems Programming Expert** (16-18 hours)
```
Prerequisites:
  - Ownership model: queue, generic-queue, binary-tree (4 hrs)

Core Sequence:
1. ascii (2 hrs)
   └─> Introduction: Transmute, safety invariants

2. ref-with-flag (2.5 hrs)
   └─> Pointers: Raw pointer manipulation

3. gap-buffer (3 hrs)
   └─> Arithmetic: Pointer math, manual memory

4. libgit2-rs (4 hrs)
   └─> FFI: C interop, extern blocks

5. libgit2-rs-safe (3 hrs)
   └─> Wrappers: Safe abstraction patterns

Advanced Topics:
  - Study std library unsafe code
  - Read The Rustonomicon
  - Practice unsafe optimization patterns
  - Understand memory layout and alignment
```

**Track C: Web Development Specialist** (10-12 hours)
```
Prerequisites:
  - Basics: gcd, queue, complex (2.5 hrs)

Core Sequence:
1. http-get (45 min)
   └─> Client side: reqwest basics

2. actix-gcd (1.5 hrs)
   └─> Server side: actix-web framework

3. cheapo-request (2 hrs)
   └─> Async client: async-std HTTP

4. many-requests (2 hrs)
   └─> Concurrent: Multiple requests

5. fern_sim (2 hrs)
   └─> Architecture: Module organization for web apps

Additional Practice:
  - Extend actix-gcd with database
  - Add authentication/authorization
  - Implement REST API
  - Study actix-web middleware
```

**Track D: CLI Tool Developer** (8-10 hours)
```
Prerequisites:
  - Basics: gcd, queue (1.25 hrs)

Core Sequence:
1. grep (2 hrs)
   └─> Text processing: regex, I/O

2. copy (1.5 hrs)
   └─> Filesystem: recursion, paths

3. http-get (45 min)
   └─> Network tools: HTTP client

4. fern_sim (2 hrs)
   └─> Structure: Large CLI apps

Additional Tools:
  - Study clap for argument parsing
  - Add progress bars with indicatif
  - Implement configuration files
  - Error handling with anyhow/thiserror
```

**Track E: Type System & Traits Expert** (8-10 hours)
```
Prerequisites:
  - Basics: gcd, queue (1.25 hrs)

Core Sequence:
1. complex (1 hr)
   └─> Operators: Add, Sub, Mul traits

2. interval (1 hr)
   └─> Comparisons: PartialOrd, PartialEq

3. generic-queue (1 hr)
   └─> Generics: Type parameters

4. binary-tree (2 hrs)
   └─> Iterators: Iterator trait

5. basic-router (1.5 hrs)
   └─> Functions: Fn, FnMut, FnOnce

Advanced Study:
  - Experiment with trait objects
  - Compare static vs dynamic dispatch
  - Study From/Into conversions
  - Explore advanced trait bounds
```

**Track F: Macro & Metaprogramming Specialist** (6-8 hours)
```
Prerequisites:
  - Strong Rust foundation: Paths 1-2 fundamentals

Core Sequence:
1. json-macro (4 hrs)
   └─> Procedural macros: Complete implementation

Deep Dive:
  - Study declarative macros (macro_rules!)
  - Read syn crate documentation
  - Experiment with attribute macros
  - Build derive macros
  - Study quote! macro
  - Understand hygiene and span

Additional Resources:
  - The Little Book of Rust Macros
  - syn/quote/proc-macro2 docs
  - macro_railroad for visualization
```

---

### Project Relationships & Evolution

Understanding how projects build upon each other reveals the pedagogical progression of Rust concepts. This section maps the evolutionary chains and conceptual dependencies between projects.

#### Concept Evolution Chains

**Chain 1: Data Structures → Generics → Traits**
```
queue (Concrete type)
  │
  ├─> Demonstrates: Struct methods, ownership, basic encapsulation
  │
  ▼
generic-queue (Generic version)
  │
  ├─> Adds: Type parameters, constraints, Vec<T> usage
  ├─> Shows: How to make code reusable across types
  │
  ▼
binary-tree (Generic + Recursive)
  │
  ├─> Adds: Recursive types, Box for indirection
  ├─> Shows: Pattern matching, Option usage
  │
  ▼
binary-tree iterators
  │
  └─> Adds: Iterator trait implementation
      └─> Shows: Custom iteration, state management
```

**Pedagogical Insight**: Start with concrete types to understand ownership, then abstract with generics, then add behavioral traits. This mirrors how you would design real systems.

**Chain 2: Unsafe → Safe Wrappers**
```
libgit2-rs (Raw FFI)
  │
  ├─> Demonstrates: extern blocks, raw pointers, C types
  ├─> Safety: None - all functions unsafe
  │
  ▼
libgit2-rs-safe (Safe wrapper)
  │
  ├─> Adds: Safe abstractions, lifetime management
  ├─> Shows: How to encapsulate unsafe code
  └─> Pattern: Zero-cost abstraction over C library
```

**Pedagogical Insight**: First understand the unsafe foundation, then learn to build safe abstractions. This teaches defensive programming and API design.

**Chain 3: Synchronous → Asynchronous I/O**
```
echo-server (Blocking TCP)
  │
  ├─> Demonstrates: std::net, synchronous I/O
  ├─> Limitation: One connection at a time
  │
  ▼
http-get (Blocking HTTP)
  │
  ├─> Adds: reqwest (blocking), HTTP protocol
  ├─> Limitation: Sequential requests
  │
  ▼
cheapo-request (Async HTTP)
  │
  ├─> Adds: async/await, futures, async-std
  ├─> Improvement: Non-blocking I/O
  │
  ▼
many-requests (Concurrent async)
  │
  ├─> Adds: join operations, parallel async
  ├─> Shows: True concurrency benefits
  │
  ▼
spawn-blocking (Async internals)
  │
  └─> Adds: Custom primitives, thread pool
      └─> Shows: How async runtime works internally
```

**Pedagogical Insight**: Understand blocking I/O limitations before appreciating async benefits. Then peek under the hood to understand the magic.

**Chain 4: Operator Overloading Progression**
```
complex (Basic operators)
  │
  ├─> Implements: Add, Sub, Mul, Neg
  ├─> Shows: Operator trait basics
  │
  ▼
interval (Comparison operators)
  │
  ├─> Implements: PartialOrd, PartialEq
  └─> Shows: Why Partial vs Total ordering matters
```

**Pedagogical Insight**: Start with arithmetic (familiar), then move to comparisons (nuanced with NaN, etc.).

#### Cross-Project Dependencies & Patterns

**Pattern Reuse Across Projects**:

1. **Error Handling Evolution**
   - `gcd`: Basic error messages, unwrap
   - `http-get`: Result types, `?` operator
   - `grep`: Custom error types, error propagation
   - `libgit2-rs-safe`: Error conversion, safe failure modes

2. **Module Organization**
   - `gcd`: Single file
   - `complex`: Single file with tests module
   - `fern_sim`: Multi-file, src/lib.rs structure
   - `libgit2-rs-safe`: Multiple modules, clear separation

3. **Testing Strategies**
   - `queue`: Unit tests in same file
   - `binary-tree`: Extensive test cases, edge conditions
   - `fern_sim`: Integration tests in tests/
   - `gap-buffer`: Testing unsafe invariants

#### Conceptual Prerequisites

Some projects assume knowledge from others:

| Project | Recommended Prerequisites | Why |
|---------|-------------------------|-----|
| `generic-queue` | `queue` | Understand concrete version first |
| `binary-tree` | `generic-queue` | Builds on generics knowledge |
| `interval` | `complex` | Extends trait implementation concepts |
| `actix-gcd` | `gcd` | Same algorithm, web interface |
| `many-requests` | `cheapo-request` | Builds on async fundamentals |
| `spawn-blocking` | `cheapo-request` | Requires async/await understanding |
| `block-on` | `spawn-blocking` | Executor requires async knowledge |
| `libgit2-rs-safe` | `libgit2-rs` | Wraps the unsafe version |
| `ref-with-flag` | `ascii` | Builds on unsafe concepts |
| `gap-buffer` | `ref-with-flag` | More complex pointer usage |

#### Thematic Groupings for Comparative Study

Study these together to compare approaches:

**Group 1: HTTP Clients (Compare async vs sync)**
- `http-get` (blocking reqwest)
- `cheapo-request` (async-std)
- `many-requests-surf` (surf crate)

**Comparison Points**: API differences, performance, error handling, ease of use

**Group 2: Executors (Compare implementations)**
- `block-on` (simple executor)
- `spawn-blocking` (custom primitives)
- async-std (external, study documentation)

**Comparison Points**: Complexity, features, scheduling strategies

**Group 3: Unsafe Code (Compare safety approaches)**
- `ascii` (simple transmute)
- `ref-with-flag` (bit manipulation)
- `gap-buffer` (pointer arithmetic)

**Comparison Points**: Safety invariants, documentation, testing strategies

**Group 4: CLI Tools (Compare architecture)**
- `gcd` (minimal)
- `grep` (moderate complexity)
- `copy` (filesystem heavy)

**Comparison Points**: Argument parsing, error handling, code organization

#### Evolution of Complexity

Observe how similar concepts grow in sophistication:

**Ownership Examples**:
1. `queue`: Basic struct ownership
2. `generic-queue`: Generic ownership with Vec<T>
3. `binary-tree`: Recursive ownership with Box
4. `gap-buffer`: Manual memory management

**Trait Sophistication**:
1. `complex`: Simple trait implementation (Add, Sub)
2. `interval`: Conditional traits (PartialOrd)
3. `binary-tree`: Iterator trait (stateful)
4. `basic-router`: Function traits (Fn, FnMut)

**Async Complexity**:
1. `cheapo-request`: Basic async/await
2. `many-requests`: Concurrent operations
3. `spawn-blocking`: Custom async types
4. `block-on`: Executor from scratch

#### Practical Learning Strategies

**Strategy 1: Diff-Based Learning**
```bash
# Compare concrete vs generic
diff queue/src/main.rs generic-queue/src/main.rs

# Compare unsafe vs safe wrapper
diff libgit2-rs/src/lib.rs libgit2-rs-safe/src/lib.rs
```

**Strategy 2: Feature Addition**
Start with simpler project, add features from complex one:
- Add iterator to `queue` (borrowing from `binary-tree`)
- Add async to `echo-server` (borrowing from `cheapo-request`)
- Add safe wrapper to `ascii` (borrowing from `libgit2-rs-safe`)

**Strategy 3: Reverse Engineering**
Start with complex, strip down to basics:
- Remove unsafe from `gap-buffer` → create safe version
- Remove async from `many-requests` → create threaded version
- Remove generics from `binary-tree` → create i32 tree

---

## Standard Commands

### Building and Testing

```bash
# Build a specific project
cd <project-name> && cargo build

# Build with release optimizations
cargo build --release

# Run a binary project
cargo run

# Run with arguments
cargo run -- <arguments>

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test <test_name>

# Check compilation without building
cargo check

# Format code (if rustfmt installed)
cargo fmt

# Lint with clippy
cargo clippy

# Generate documentation
cargo doc --open
```

### Repository-Wide Operations

```bash
# Build all projects
for dir in */; do cd "$dir" && cargo build && cd ..; done

# Run tests on all projects
for dir in */; do cd "$dir" && cargo test && cd ..; done

# Check all projects compile
for dir in */; do cd "$dir" && cargo check && cd ..; done
```

### Git Workflow (for feature branch development)

```bash
# Check current status
git status

# Create feature branch
git checkout -b feature/<description>

# View changes
git diff

# Stage changes
git add <file-path>

# Commit changes
git commit -m "descriptive message"

# Push to feature branch
git push -u origin claude/create-claude-documentation-rCOwU

# Create pull request
gh pr create --title "PR Title" --body "Description"
```

---

## Code Style & Conventions

### Rust Edition and Format

- **Edition**: Rust 2018 across all projects
- **Formatting**: Follow `rustfmt` defaults (implied standard)
- **Naming**:
  - Types: `PascalCase` (e.g., `Queue`, `BinaryTree`, `Complex`)
  - Functions: `snake_case` (e.g., `new_queue`, `build_tree`)
  - Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_SIZE`)
  - Modules: `snake_case` (e.g., `simulation`, `spores`)

### Module Structure

Most projects use one of two patterns:

**Single-file pattern** (for simple projects):
```rust
// src/lib.rs or src/main.rs
// All code in one file
```

**Multi-file pattern** (for complex projects like fern_sim):
```
src/
├── lib.rs          // Main module declarations
├── simulation.rs   // Feature module
├── spores.rs       // Feature module
└── tests/          // Integration tests
```

### Type System Patterns

This repository showcases many trait implementations:

- **Custom Types**: Prefer `struct` for data containers, `enum` for variant types
- **Trait Implementations**: Common patterns include:
  - `Display` and `Debug` for formatting
  - `Operator traits` (Add, Sub, Mul, etc.) for operator overloading
  - `Iterator` for iteration support
  - `Deref` for smart pointers
  - `PartialOrd` and `PartialEq` for comparisons

Example from `complex/`:
```rust
use std::ops::Add;

#[derive(Clone, Copy, Debug)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Add for Complex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { ... }
}
```

### Unsafe Code Patterns

Projects using unsafe (`ascii`, `ref-with-flag`, `gap-buffer`) follow these principles:

- **Minimal scope**: Unsafe code is isolated in dedicated functions/blocks
- **Documentation**: Unsafe invariants are clearly documented
- **Safety**: Caller/enclosing code must maintain invariants
- **Example** from `ascii/`:
```rust
pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Ascii {
    std::mem::transmute(bytes)
}
```

### Error Handling

Most examples use `Result` for fallible operations:

```rust
use std::io;

fn do_something() -> io::Result<String> {
    // Function body with ? operator
}
```

Simpler examples may use `unwrap()` or `expect()` for clarity in educational context.

### Testing Conventions

**Unit Tests** (within source files):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operation() {
        // Test implementation
    }
}
```

**Integration Tests** (in `tests/` directory):
```rust
// tests/integration_test.rs
#[test]
fn test_from_another_crate() {
    // Uses the library like an external consumer
}
```

Example from `fern_sim/`:
```
tests/
├── integration_test.rs
└── ...
```

---

## Development Workflows

### Setting Up a Project for Work

```bash
# 1. Ensure on feature branch
git checkout claude/create-claude-documentation-rCOwU

# 2. Navigate to project
cd <project-name>

# 3. Verify it builds
cargo build

# 4. Run existing tests
cargo test

# 5. Start development
# (use cargo watch or editor-integrated tools)
```

### Adding New Code

1. **For new functions**: Add to appropriate module file
2. **For new types**: Create in dedicated section or new file if large
3. **For new modules**: Create new `.rs` file and declare in `lib.rs` or `main.rs`
4. **For tests**: Add `#[test]` functions to relevant modules or create test file

### Modifying Existing Code

1. **Understand first**: Read surrounding code and any documentation
2. **Plan changes**: Sketch out impact on dependent code
3. **Implement incrementally**: Make small, testable changes
4. **Test thoroughly**: Run full test suite after changes
5. **Review**: Ask AI to review for style and correctness

### Documentation Standards

- **Inline comments**: Explain "why", not "what" (code shows what)
- **Function docs**: Use `///` for public functions (shown in cargo doc)
- **Module docs**: Use `//!` at top of files if complex
- **Examples**: Include in doc comments for public APIs

---

## Testing Strategy

### Testing Framework

- **Unit testing**: Built-in `#[test]` attribute and `assert!` macros
- **Integration testing**: Separate crate in `tests/` directory
- **No external test frameworks** required (standard library sufficient)

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_behavior() {
        assert_eq!(result, expected);
    }

    #[test]
    fn test_edge_case() {
        assert!(condition);
    }

    #[test]
    #[should_panic]
    fn test_panics_appropriately() {
        // Code that should panic
    }
}
```

### Testing Best Practices

1. **Test structure**: Follow `Arrange → Act → Assert` pattern
2. **Test naming**: Use descriptive names like `test_<function>_<scenario>`
3. **Edge cases**: Include tests for boundary conditions
4. **Error cases**: Use `#[should_panic]` or `Result` returns for error testing
5. **Coverage**: Aim for tests of public APIs; internal functions tested implicitly

### Running Tests

```bash
# Run all tests with output
cargo test -- --nocapture --test-threads=1

# Run specific test
cargo test test_name

# Run tests and show panics
cargo test -- --nocapture

# Run ignoring some tests
cargo test --lib     # Only unit tests
cargo test --test '*' # Only integration tests
```

---

## Git & Branch Conventions

### Branch Strategy

- **Development branch**: `claude/create-claude-documentation-rCOwU` (current feature branch)
- **Feature branches**: Used for development on specific features/improvements
- **Main branch**: Production-ready code (protected)

### Commit Message Format

Keep commit messages clear and concise:

```
<type>: <short description>

<optional detailed explanation>
```

**Types**:
- `feat`: New feature or capability
- `fix`: Bug fix
- `docs`: Documentation update
- `refactor`: Code reorganization without functional change
- `test`: Test additions or modifications
- `chore`: Dependency updates, build configuration

**Examples**:
```
feat: Add CLAUDE.md documentation for AI assistants

docs: Update README chapter references

fix: Correct unsafe code invariant in gap-buffer

chore(deps): bump actix-web from 4.0 to 4.1
```

### Push to Feature Branch

When pushing to the designated feature branch:

```bash
# Push with upstream tracking
git push -u origin claude/create-claude-documentation-rCOwU

# Subsequent pushes
git push origin claude/create-claude-documentation-rCOwU
```

**Important**: The branch name must start with `claude/` and end with the session ID (`-rCOwU` in this case).

### Creating Pull Requests

When code is ready:

```bash
gh pr create --title "Feature: <description>" \
  --body "Description of changes and testing"
```

Use the template:
```markdown
## Summary
- Brief description of changes
- Any important implementation notes

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Code follows style guide
- [ ] No clippy warnings

## Related Issues
- Fixes #<issue-number> (if applicable)
```

---

## AI Assistant Guidelines

### Philosophy

This guide is designed for **AI assistants working alongside developers**. AI tools should:

1. **Enhance human judgment**, not replace it
2. **Accelerate common tasks** (boilerplate, refactoring, documentation)
3. **Maintain code quality** through testing and review
4. **Communicate clearly** about assumptions and trade-offs

### How to Work With Claude Code

#### Initial Exploration (Plan Phase)

When starting work on a feature or bug:

```
1. Have Claude explore relevant files
   - Understand current implementation
   - Identify related code patterns
   - Check existing tests

2. Request a plan before coding
   - Use Plan Mode for complex changes
   - Review the proposed approach
   - Suggest adjustments if needed

3. Get approval before implementation
   - Ensure alignment on strategy
   - Clarify any ambiguities
   - Set expectations for testing
```

#### Implementation (Code Phase)

During implementation:

```
1. Work in focused iterations
   - Target 5-20 file changes per iteration
   - Implement one feature at a time
   - Test frequently

2. Use checkpoints for rollback
   - Commit working states regularly
   - Keep git history clean
   - Enable easy reversion if needed

3. Ask for code review
   - "Review this for style compliance"
   - "Check for unsafe code issues"
   - "Verify test coverage"
```

#### Verification (Test Phase)

Before finalizing:

```
1. Run full test suite
   - Verify existing tests still pass
   - Add tests for new functionality
   - Check coverage of edge cases

2. Lint and format
   - Run clippy for warnings
   - Use cargo fmt for formatting
   - Verify no compiler warnings

3. Manual review
   - Read all changes once more
   - Verify against style guide
   - Check for unintended side effects
```

### Effective Prompts for AI Assistants

**Good Prompt**:
```
In the binary-tree project, add a method `depth(&self) -> usize` to the BinaryTree type that returns the maximum depth of the tree (height + 1 for consistency with the existing codebase). Follow the iterator pattern already used in this crate. Include tests for edge cases (empty tree, single node, balanced vs unbalanced trees).
```

**Vague Prompt**:
```
Add a method to binary-tree
```

**Specific Context**:
```
In the actix-gcd project, the web framework is actix-web 4.1. When adding the new route handler for /lcm (least common multiple), follow the same pattern as the existing /gcd route in src/main.rs (lines 23-35). The response should be JSON with the same structure as the GCD endpoint.
```

### Do Not Touch Zones

⚠️ **Critical Files** - Modify only with explicit user approval:

- `LICENSE-MIT` - License terms (immutable)
- Root `README.md` - Official chapter mapping (coordinate changes)
- `.git/` - Git internals (never modify)
- `Cargo.lock` files - Only update via `cargo update`

### Code Style Compliance

When adding code, ensure:

1. ✅ **Naming conventions**: Use `snake_case` for functions, `PascalCase` for types
2. ✅ **Module organization**: Follow existing project structure
3. ✅ **Documentation**: Add `///` comments for public items
4. ✅ **Testing**: Include `#[test]` functions for new functionality
5. ✅ **No compiler warnings**: Code should compile cleanly
6. ✅ **Consistent formatting**: Run `cargo fmt` before committing

### When to Ask for Clarification

Ask the developer (not just assume) about:

- **Design decisions**: "Should we use an enum or trait objects for X?"
- **Performance trade-offs**: "This recursive approach is elegant but slower—is that acceptable?"
- **Architecture impact**: "This change affects the module boundary. Should we refactor?"
- **Testing scope**: "Should we add benchmark tests or just functional tests?"
- **Backward compatibility**: "Should we keep the old function signature as deprecated?"

### Common Task Patterns

**Pattern: Adding a New Function**
```
1. Research: Where should it live? What's the existing API?
2. Plan: What will the signature be? What are edge cases?
3. Code: Implement with clear logic
4. Test: Add [#test] functions for coverage
5. Review: Check style, docs, performance
6. Commit: Message explaining the addition
```

**Pattern: Fixing a Bug**
```
1. Understand: What's the root cause? How does the code currently fail?
2. Plan: What's the minimal fix? Are there side effects?
3. Test: Write test that reproduces the bug
4. Fix: Apply the minimal change
5. Verify: Confirm test now passes
6. Commit: Reference any issue number, explain the fix
```

**Pattern: Refactoring**
```
1. Understand: What's the goal? Better readability? Performance?
2. Plan: What's the scope? What could break?
3. Code: Make incremental changes
4. Test: Ensure existing tests still pass (functionality unchanged)
5. Review: Is the code clearer/better?
6. Commit: Explain why the refactoring improves the code
```

---

## Security & Compliance

### Code Safety

- **Unsafe code**: Clearly documented in `ascii`, `ref-with-flag`, `gap-buffer` projects
- **Invariants**: Documented when unsafe code relies on assumptions
- **Public APIs**: Should be safe to use (unsafe hidden in implementation)

### Dependencies

- **Minimize external crates**: Most examples use few dependencies
- **Review transitive dependencies**: Check `cargo tree` output
- **Security updates**: Keep dependencies current for security patches

### Testing Security

- **Input validation**: Test edge cases and invalid inputs
- **Error handling**: Don't panic on bad input (use Result types)
- **Unsafe boundaries**: Test assumptions made by unsafe code

---

## Troubleshooting Common Issues

### Build Failures

**Issue**: `error: failed to find required target libgit2`

**Solution**: For libgit2 projects, ensure libgit2 is installed:
```bash
# macOS
brew install libgit2

# Linux
apt-get install libgit2-dev

# Windows
# Follow instructions in libgit2-rs/build.rs comments
```

**Issue**: Dependency version conflicts

**Solution**: Check `Cargo.lock` and run `cargo update` to resolve:
```bash
cd <project>
cargo clean
cargo update
cargo build
```

### Test Failures

**Issue**: Tests pass locally but not in CI

**Solution**: Run tests with full output:
```bash
cargo test -- --nocapture --test-threads=1
```

### Performance Issues

**Issue**: Binary runs slowly

**Solution**: Build with optimizations:
```bash
cargo build --release
./target/release/<binary>
```

---

## Best Practices Summary for AI Assistants

### When Analyzing Code

1. ✅ Read the entire source file first
2. ✅ Understand the surrounding context
3. ✅ Check for existing patterns to follow
4. ✅ Look at related code in other projects
5. ✅ Review existing tests to understand expected behavior

### When Writing Code

1. ✅ Follow project style conventions
2. ✅ Add documentation comments for public items
3. ✅ Include tests for new functionality
4. ✅ Use meaningful variable and function names
5. ✅ Keep functions focused and testable

### When Committing

1. ✅ Write clear, descriptive commit messages
2. ✅ Group related changes together
3. ✅ Verify tests pass before committing
4. ✅ Use conventional commit format
5. ✅ Reference related issues if applicable

### When Stuck

1. ✅ Explain the current understanding of the problem
2. ✅ Describe what has been tried so far
3. ✅ Ask specific questions (not vague ones)
4. ✅ Provide code examples of the issue
5. ✅ Request a plan or architecture discussion

---

## Additional Resources

### External Repositories (Mentioned in README)

- [Mandelbrot Set Plotter](https://github.com/ProgrammingRust/mandelbrot) - Multi-threaded graphics
- [Fingertips Search Engine](https://github.com/ProgrammingRust/fingertips) - Concurrency patterns
- [Async Chat Application](https://github.com/ProgrammingRust/async-chat) - Complete async example

### Rust Documentation

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust Standard Library](https://doc.rust-lang.org/std/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Rustlings (Interactive Learning)](https://github.com/rust-lang/rustlings)

### Crate Documentation

- [Actix-web](https://actix.rs/) - Web framework used in actix-gcd
- [reqwest](https://docs.rs/reqwest/) - HTTP client
- [async-std](https://docs.rs/async-std/) - Async runtime
- [serde](https://serde.rs/) - Serialization framework

---

## Document History

- **Created**: December 2025
- **Purpose**: Comprehensive guide for AI assistants working on Rust examples
- **Scope**: All 24 projects in the repository
- **Status**: Active and maintained during development sessions

---

## Questions for AI Assistants

Before starting work, clarify:

1. **Scope**: Which project(s) are we modifying?
2. **Goal**: What feature/fix are we implementing?
3. **Testing**: What should the test coverage include?
4. **Documentation**: Should existing docs be updated?
5. **Compatibility**: Any backward compatibility concerns?

---

*This guide enables clear communication between developers and AI assistants to build high-quality, well-tested Rust code that demonstrates language best practices.*
