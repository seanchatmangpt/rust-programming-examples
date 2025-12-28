# Reference Documentation

**Information-Oriented Documentation**

Reference documentation provides technical descriptions of the code. Unlike tutorials or how-to guides, reference material is dry, precise, and comprehensive. Use this when you need to look up specific details about types, functions, or APIs.

## What Is Reference Documentation?

Reference docs are:
- **Information-oriented**: Facts, not instructions
- **Comprehensive**: Covers all public APIs systematically
- **Accurate**: Precise type signatures, constraints, and behavior
- **Searchable**: Organized for quick lookup

Think of reference docs as a dictionary: you know the word, you just need to know its exact meaning.

## Generating API Documentation

Each project has full API documentation that you can generate locally:

```bash
# Generate and open documentation for a specific project
cd <project-name>
cargo doc --open

# Generate docs with private items
cargo doc --document-private-items --open

# Generate docs for dependencies too
cargo doc --open --no-deps
```

## For Python Developers

Rust's documentation system is similar to Python's docstrings and Sphinx, but built into the compiler:

| Python | Rust |
|--------|------|
| `help(function)` | `cargo doc --open` |
| Docstrings (`"""..."""`) | Doc comments (`///` or `//!`) |
| Sphinx | `rustdoc` (built-in) |
| Type hints | Actual types in signatures |
| `__doc__` | Markdown in doc comments |

## Project API Reference

All 24 projects are documented below, organized by category. Each entry includes:
- **Purpose**: What the project demonstrates
- **Key types and functions**: Main API surface
- **Dependencies**: External crates used
- **Related projects**: Similar examples

---

## Fundamentals

### [gcd](../api/gcd/index.html)

**Purpose**: Greatest common divisor calculation demonstrating basic Rust syntax.

**Key functions**:
- `fn gcd(mut n: u64, mut m: u64) -> u64` - Computes GCD using Euclidean algorithm
- `fn main()` - Parses command-line arguments and prints result

**Dependencies**: None (uses only std)

**Python equivalent**: Simple function with integer arithmetic

**Documentation**: Run `cargo doc --open` in the `gcd` directory

---

### [copy](../api/copy/index.html)

**Purpose**: Demonstrates Copy trait, move semantics, and the difference between stack and heap types.

**Key types**:
- Examples with `i32` (Copy type)
- Examples with `String` (non-Copy type)
- Examples with `Vec<T>` (non-Copy type)

**Key concepts demonstrated**:
- Copy vs Move semantics
- Borrowing with `&T` and `&mut T`
- Clone trait

**Dependencies**: None (uses only std)

**Python equivalent**: All Python objects are references; no direct equivalent

**Documentation**: Run `cargo doc --open` in the `copy` directory

---

### [grep](../api/grep/index.html)

**Purpose**: Simple text search tool demonstrating string handling and I/O.

**Key functions**:
- `fn search(pattern: &str, content: &str) -> Vec<&str>` - Finds matching lines
- `fn main() -> Result<(), Box<dyn Error>>` - File I/O and error handling

**Key types used**:
- `&str` - String slices
- `String` - Owned strings
- `Result<T, E>` - Error handling
- `Vec<T>` - Dynamic arrays

**Dependencies**: None (uses only std)

**Python equivalent**: Using `open()` and string methods like `if pattern in line`

**Documentation**: Run `cargo doc --open` in the `grep` directory

---

## Data Structures

### [queue](../api/queue/index.html)

**Purpose**: FIFO queue implementation using `Vec<i32>`.

**Key types**:
```rust
pub struct Queue {
    // Private fields
}
```

**Key methods**:
- `pub fn new() -> Queue` - Creates empty queue
- `pub fn push(&mut self, value: i32)` - Adds to back
- `pub fn pop(&mut self) -> Option<i32>` - Removes from front
- `pub fn is_empty(&self) -> bool` - Checks if empty

**Dependencies**: None (uses only std)

**Python equivalent**: `collections.deque` or list with `append()/pop(0)`

**Documentation**: Run `cargo doc --open` in the `queue` directory

---

### [generic-queue](../api/generic_queue/index.html)

**Purpose**: Generic FIFO queue working with any type.

**Key types**:
```rust
pub struct Queue<T> {
    // Private fields
}
```

**Key methods**:
- `pub fn new() -> Queue<T>` - Creates empty generic queue
- `pub fn push(&mut self, value: T)` - Adds to back
- `pub fn pop(&mut self) -> Option<T>` - Removes from front
- `pub fn is_empty(&self) -> bool` - Checks if empty

**Type parameters**:
- `T` - Type of elements (no trait bounds)

**Dependencies**: None (uses only std)

**Python equivalent**: `class Queue[T]` with typing generics

**Documentation**: Run `cargo doc --open` in the `generic-queue` directory

---

### [binary-tree](../api/binary_tree/index.html)

**Purpose**: Binary search tree with recursive structure.

**Key types**:
```rust
pub enum Tree<T> {
    Empty,
    Node(Box<TreeNode<T>>),
}

pub struct TreeNode<T> {
    pub value: T,
    pub left: Tree<T>,
    pub right: Tree<T>,
}
```

**Key methods**:
- `pub fn new() -> Tree<T>` - Creates empty tree
- `pub fn insert(&mut self, value: T) where T: Ord` - Inserts value
- `pub fn contains(&self, value: &T) -> bool where T: Ord` - Searches
- Various traversal methods

**Dependencies**: None (uses only std)

**Python equivalent**: Tree node class with left/right children

**Documentation**: Run `cargo doc --open` in the `binary-tree` directory

---

### [gap-buffer](../api/gap_buffer/index.html)

**Purpose**: Text editor buffer with efficient insertion/deletion.

**Key types**:
```rust
pub struct GapBuffer<T> {
    // Private fields
}
```

**Key methods**:
- `pub fn new() -> GapBuffer<T>` - Creates empty buffer
- `pub fn insert(&mut self, item: T)` - Inserts at cursor
- `pub fn remove(&mut self) -> Option<T>` - Removes at cursor
- `pub fn position(&self) -> usize` - Gets cursor position
- `pub fn set_position(&mut self, pos: usize)` - Moves cursor

**Dependencies**: None (uses only std)

**Python equivalent**: No direct equivalent; list operations but less efficient

**Documentation**: Run `cargo doc --open` in the `gap-buffer` directory

---

## Type System Examples

### [ascii](../api/ascii/index.html)

**Purpose**: Type-safe ASCII string using newtype pattern.

**Key types**:
```rust
pub struct Ascii(Vec<u8>);
```

**Key methods**:
- `pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError>` - Validates
- Implements `From<Ascii>` for `String` and other conversions

**Dependencies**: None (uses only std)

**Python equivalent**: Wrapper class with validation, but runtime only

**Documentation**: Run `cargo doc --open` in the `ascii` directory

---

### [complex](../api/complex/index.html)

**Purpose**: Complex number type with operator overloading.

**Key types**:
```rust
pub struct Complex<T> {
    pub re: T,
    pub im: T,
}
```

**Key trait implementations**:
- `Add`, `Sub`, `Mul`, `Div` - Arithmetic operators
- `Neg` - Negation operator
- Generic over numeric types

**Dependencies**: num-traits

**Python equivalent**: Class with `__add__`, `__mul__`, etc.

**Documentation**: Run `cargo doc --open` in the `complex` directory

---

### [interval](../api/interval/index.html)

**Purpose**: Interval type with generic bounds.

**Key types**:
```rust
pub struct Interval<T> {
    pub lower: T,
    pub upper: T,
}
```

**Key methods**:
- `pub fn new(lower: T, upper: T) -> Interval<T> where T: PartialOrd`
- `pub fn contains(&self, value: &T) -> bool where T: PartialOrd`
- `pub fn overlaps(&self, other: &Interval<T>) -> bool where T: PartialOrd`

**Type constraints**: `T: PartialOrd` for ordering comparisons

**Dependencies**: None (uses only std)

**Python equivalent**: Generic class with protocol constraints

**Documentation**: Run `cargo doc --open` in the `interval` directory

---

### [ref-with-flag](../api/ref_with_flag/index.html)

**Purpose**: Stores reference and boolean in single word using bit manipulation.

**Key types**:
```rust
pub struct RefWithFlag<'a, T> {
    // Private: stores pointer with flag in low bit
}
```

**Key methods**:
- `pub fn new(ptr: &'a T, flag: bool) -> RefWithFlag<'a, T>` - Creates
- `pub fn get_ref(&self) -> &'a T` - Extracts reference
- `pub fn get_flag(&self) -> bool` - Extracts flag

**Safety**: Uses unsafe internally, safe API externally

**Dependencies**: None (uses only std)

**Python equivalent**: Not applicable (low-level optimization)

**Documentation**: Run `cargo doc --open` in the `ref-with-flag` directory

---

## Web and Networking

### [http-get](../api/http_get/index.html)

**Purpose**: HTTP client using reqwest library.

**Key functions**:
- `async fn fetch_url(url: &str) -> Result<String, reqwest::Error>` - Fetches URL

**Key types used**:
- `reqwest::Client` - HTTP client
- `reqwest::Response` - HTTP response
- `tokio` runtime for async

**Dependencies**: reqwest, tokio

**Python equivalent**: `requests.get()` or `httpx.get()`

**Documentation**: Run `cargo doc --open` in the `http-get` directory

---

### [cheapo-request](../api/cheapo_request/index.html)

**Purpose**: Low-level HTTP using TCP sockets.

**Key functions**:
- `fn http_get(url: &str) -> Result<String, Box<dyn Error>>` - Manual HTTP request

**Key types used**:
- `TcpStream` - TCP connection
- Manual HTTP request/response parsing

**Dependencies**: url (for parsing)

**Python equivalent**: Using `socket` module directly

**Documentation**: Run `cargo doc --open` in the `cheapo-request` directory

---

### [echo-server](../api/echo_server/index.html)

**Purpose**: TCP server that echoes received data.

**Key functions**:
- `fn handle_client(stream: TcpStream)` - Handles connection
- `fn main()` - Accepts connections

**Key types used**:
- `TcpListener` - Server socket
- `TcpStream` - Client connection

**Dependencies**: None (uses only std)

**Python equivalent**: `socketserver.TCPServer`

**Documentation**: Run `cargo doc --open` in the `echo-server` directory

---

### [many-requests](../api/many_requests/index.html)

**Purpose**: Concurrent HTTP requests using futures.

**Key functions**:
- `async fn fetch_all(urls: Vec<String>) -> Vec<Result<String, reqwest::Error>>`

**Key macros used**:
- `join!` - Wait for all futures
- `try_join!` - Wait and propagate errors

**Dependencies**: reqwest, tokio, futures

**Python equivalent**: `asyncio.gather()`

**Documentation**: Run `cargo doc --open` in the `many-requests` directory

---

### [many-requests-surf](../api/many_requests_surf/index.html)

**Purpose**: Concurrent HTTP requests using surf client.

**Key functions**:
- Similar to many-requests but with surf API

**Dependencies**: surf, async-std

**Python equivalent**: `httpx.AsyncClient` with `asyncio.gather()`

**Documentation**: Run `cargo doc --open` in the `many-requests-surf` directory

---

### [actix-gcd](../api/actix_gcd/index.html)

**Purpose**: Web service using Actix-web framework.

**Key functions**:
- Handler functions for routes
- Form processing
- HTML responses

**Key types used**:
- `actix_web::App` - Application builder
- `actix_web::HttpServer` - Server
- Various extractors

**Dependencies**: actix-web, serde

**Python equivalent**: Flask or FastAPI application

**Documentation**: Run `cargo doc --open` in the `actix-gcd` directory

---

### [basic-router](../api/basic_router/index.html)

**Purpose**: Custom HTTP router implementation.

**Key types**:
```rust
pub struct Router {
    // Route table
}
```

**Key methods**:
- `pub fn add_route(pattern: &str, handler: Handler)` - Registers route
- `pub fn dispatch(path: &str) -> Option<Response>` - Finds and calls handler

**Dependencies**: Minimal or none

**Python equivalent**: Understanding Werkzeug routing (Flask internals)

**Documentation**: Run `cargo doc --open` in the `basic-router` directory

---

## Async and Concurrency

### [block-on](../api/block_on/index.html)

**Purpose**: Simple async executor implementation.

**Key functions**:
- `fn block_on<F: Future>(future: F) -> F::Output` - Executes future

**Key types**:
- Custom `Future` implementations
- `Waker` and `Context` types

**Dependencies**: futures (for traits)

**Python equivalent**: Understanding `asyncio.run()` internals

**Documentation**: Run `cargo doc --open` in the `block-on` directory

---

### [spawn-blocking](../api/spawn_blocking/index.html)

**Purpose**: Mixing blocking and async code.

**Key functions**:
- `async fn run_blocking_task()` - Demonstrates spawn_blocking

**Key patterns**:
- `tokio::task::spawn_blocking` - Runs blocking code in thread pool
- Integration with async code

**Dependencies**: tokio

**Python equivalent**: `asyncio.to_thread()` or `run_in_executor()`

**Documentation**: Run `cargo doc --open` in the `spawn-blocking` directory

---

## Foreign Function Interface (FFI)

### [libgit2-rs](../api/libgit2_rs/index.html)

**Purpose**: Unsafe bindings to libgit2 C library.

**Key modules**:
- Raw FFI declarations
- Unsafe function wrappers

**Key types**:
- C types via `std::os::raw`
- Raw pointers

**Safety**: Extensively uses `unsafe`

**Dependencies**: libgit2-sys (or manual bindings)

**Python equivalent**: `ctypes` bindings

**Documentation**: Run `cargo doc --open` in the `libgit2-rs` directory

---

### [libgit2-rs-safe](../api/libgit2_rs_safe/index.html)

**Purpose**: Safe Rust wrapper around libgit2 FFI.

**Key types**:
```rust
pub struct Repository {
    // RAII wrapper around C pointer
}
```

**Key methods**:
- Safe API hiding unsafe FFI calls
- Automatic cleanup via Drop

**Safety**: Safe API, unsafe implementation

**Dependencies**: Builds on libgit2-rs

**Python equivalent**: Pythonic wrapper like GitPython

**Documentation**: Run `cargo doc --open` in the `libgit2-rs-safe` directory

---

## Macros

### [json-macro](../api/json_macro/index.html)

**Purpose**: JSON literal macro similar to `serde_json::json!`.

**Key macros**:
```rust
macro_rules! json {
    // Patterns for objects, arrays, literals
}
```

**Usage example**:
```rust
let value = json!({
    "name": "Alice",
    "age": 30
});
```

**Dependencies**: serde_json (for types)

**Python equivalent**: Dict/list literals (no macro needed)

**Documentation**: Run `cargo doc --open` in the `json-macro` directory

---

## Complete Applications

### [fern_sim](../api/fern_sim/index.html)

**Purpose**: Complete fern simulation application.

**Key modules**:
- Simulation logic
- Image generation
- CLI argument parsing

**Key functions**:
- Main simulation loop
- Rendering to image

**Dependencies**: image, clap (or similar)

**Python equivalent**: Complete Python application with argparse and PIL

**Documentation**: Run `cargo doc --open` in the `fern-sim` directory

---

## Quick Lookup Tables

### By Rust Concept

| Concept | Projects |
|---------|----------|
| Ownership & Borrowing | copy, queue, binary-tree |
| Generics | generic-queue, complex, interval, binary-tree |
| Traits | complex, interval, gap-buffer |
| Error Handling | grep, http-get, cheapo-request |
| Async/Await | http-get, many-requests, many-requests-surf, actix-gcd |
| FFI & Unsafe | libgit2-rs, libgit2-rs-safe, ref-with-flag |
| Macros | json-macro |
| Pattern Matching | binary-tree, basic-router |
| Lifetimes | ref-with-flag, grep |

### By Python Equivalent

| Python Feature | Rust Project |
|----------------|--------------|
| `collections.deque` | queue, generic-queue |
| Class with `__add__`, `__mul__` | complex |
| `requests.get()` | http-get |
| `socket` module | cheapo-request, echo-server |
| `asyncio.gather()` | many-requests, many-requests-surf |
| Flask/FastAPI | actix-gcd |
| `ctypes` | libgit2-rs, libgit2-rs-safe |
| Decorators/metaprogramming | json-macro |

### By Dependency/Crate

| Crate | Projects Using It |
|-------|-------------------|
| tokio | http-get, many-requests, spawn-blocking |
| reqwest | http-get, many-requests |
| actix-web | actix-gcd |
| serde/serde_json | json-macro, actix-gcd |
| futures | many-requests, block-on |
| async-std | many-requests-surf |
| surf | many-requests-surf |

## Documentation Standards

All projects follow these documentation standards:

1. **Module-level docs** (`//!`): Overview of the module/crate
2. **Item-level docs** (`///`): Description of functions, types, methods
3. **Examples**: Code examples in doc comments
4. **Panics section**: When functions can panic
5. **Safety section**: For unsafe code, what invariants must be upheld
6. **Errors section**: What errors can be returned

## Generating Documentation Tips

```bash
# Open docs in browser
cargo doc --open

# Include private items
cargo doc --document-private-items

# Generate for specific package in workspace
cargo doc -p <package-name>

# Don't include dependencies
cargo doc --no-deps

# Check doc tests
cargo test --doc
```

## Beyond Reference Docs

- **Want to learn?** → [Tutorials](../tutorials/index.md)
- **Need to solve a problem?** → [How-To Guides](../how-to/index.md)
- **Want to understand deeply?** → [Explanations](../explanation/index.md)

---

*Reference documentation: Precise, accurate, comprehensive. Your technical lookup resource.*
