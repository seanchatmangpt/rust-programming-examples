# How-To Guides

**Task-Oriented Documentation**

How-to guides are practical instructions for accomplishing specific tasks. Unlike tutorials that teach concepts step-by-step, how-to guides assume you know what you want to do and show you the most direct path to do it.

## What Are How-To Guides?

How-to guides are:
- **Goal-oriented**: Focused on achieving a specific outcome
- **Practical**: Working code you can adapt to your needs
- **Assume knowledge**: You should understand basic Rust concepts
- **Problem-solving**: Solutions to common challenges

Think of how-to guides as recipes: you know you want to bake bread, the recipe shows you how.

## When to Use These Guides

Use how-to guides when:
- You know what you want to build
- You need a quick solution to a specific problem
- You're looking for best practices for a particular task
- You want to see how to use a specific feature or pattern

If you're still learning Rust basics, start with [Tutorials](../tutorials/index.md) instead.

## For Python Developers

Each guide includes Python comparisons showing how you might solve the same problem in Python vs. Rust, highlighting the trade-offs and advantages of each approach.

## Available Guides

### Data Structures

#### [How to Implement a Queue](data-structures/queue.md)
Build a FIFO queue using `Vec<T>`.

**When to use**: You need a simple queue without external dependencies.

**Python equivalent**: Using `collections.deque` or a list with `append()/pop(0)`.

**Key topics**:
- Struct definition and methods
- Working with `Vec<T>`
- Returning `Option<T>` for empty cases
- Method visibility (pub vs private)

---

#### [How to Make Data Structures Generic](data-structures/generic-queue.md)
Transform a concrete data structure into a generic one.

**When to use**: Your data structure should work with multiple types.

**Python equivalent**: Using typing generics like `class Queue[T]`.

**Key topics**:
- Generic type parameters
- Trait bounds
- The `Clone` trait
- Where clauses

---

#### [How to Implement a Binary Search Tree](data-structures/binary-tree.md)
Create a recursive binary tree structure.

**When to use**: You need a sorted tree structure or want to learn recursive types.

**Python equivalent**: Defining a tree node class with left/right children.

**Key topics**:
- Recursive enums
- `Box<T>` for heap allocation
- Pattern matching on recursive structures
- Tree operations (insert, search, traverse)

---

#### [How to Build a Gap Buffer](data-structures/gap-buffer.md)
Implement an efficient text editor buffer.

**When to use**: Building a text editor or need efficient insertion/deletion.

**Python equivalent**: No direct equivalent; would use list slicing (slower).

**Key topics**:
- Efficient insertion/deletion
- Cursor management
- Vec manipulation methods
- Maintaining data structure invariants

---

### Type System and Safety

#### [How to Create Type-Safe Wrappers (Newtype Pattern)](types/newtype-ascii.md)
Wrap existing types to add compile-time guarantees.

**When to use**: You want to prevent mixing up similar values (e.g., user IDs vs. product IDs).

**Python equivalent**: Creating wrapper classes, but only runtime checking.

**Key topics**:
- Newtype pattern
- Zero-cost abstractions
- Validation at construction
- Implementing From/Into traits

---

#### [How to Overload Operators](types/operator-overloading.md)
Implement arithmetic and comparison operators for custom types.

**When to use**: Creating mathematical types or domain-specific types with natural operators.

**Python equivalent**: Implementing `__add__`, `__mul__`, etc.

**Key topics**:
- std::ops traits (Add, Mul, etc.)
- Generic operator implementations
- Trait bounds for operators
- The num_traits crate

---

#### [How to Work with Generic Bounds](types/generic-bounds.md)
Constrain generic types with trait bounds.

**When to use**: Your generic code needs specific capabilities from type parameters.

**Python equivalent**: Protocol or ABC constraints, but compile-time.

**Key topics**:
- Trait bounds
- Where clauses
- Associated types
- Complex trait requirements

---

#### [How to Do Low-Level Bit Manipulation](types/bit-manipulation.md)
Perform pointer arithmetic and bit-level operations safely.

**When to use**: Performance-critical code or systems programming.

**Python equivalent**: Not typically done in Python; would use ctypes.

**Key topics**:
- Raw pointers
- Bit manipulation
- Alignment and memory layout
- Unsafe blocks
- PhantomData

---

### Web and Networking

#### [How to Make HTTP Requests](web/http-client.md)
Send HTTP requests using reqwest.

**When to use**: Your application needs to call HTTP APIs.

**Python equivalent**: Using `requests.get()` or `httpx.AsyncClient`.

**Key topics**:
- The reqwest crate
- Async/await
- Error handling with `?` operator
- JSON serialization/deserialization
- Request headers and parameters

---

#### [How to Build a TCP Server](web/tcp-server.md)
Create a TCP server that accepts connections.

**When to use**: Building network servers or learning networking fundamentals.

**Python equivalent**: Using `socket` or `socketserver` module.

**Key topics**:
- TcpListener and TcpStream
- Reading and writing bytes
- Handling multiple connections
- Error handling in network I/O
- Basic protocol implementation

---

#### [How to Make Concurrent HTTP Requests](web/concurrent-requests.md)
Execute multiple HTTP requests in parallel.

**When to use**: You need to fetch data from multiple sources concurrently.

**Python equivalent**: Using `asyncio.gather()` or `concurrent.futures`.

**Key topics**:
- Async/await
- `join!` and `try_join!` macros
- Error handling in concurrent code
- Performance considerations
- The futures crate

---

#### [How to Build a Web Service with Actix-web](web/actix-web-service.md)
Create a REST API using the Actix-web framework.

**When to use**: Building web APIs or microservices.

**Python equivalent**: Using Flask, FastAPI, or Django.

**Key topics**:
- Route handlers
- Request extraction
- JSON responses
- Form handling
- State management
- Middleware

---

#### [How to Implement Custom HTTP Routing](web/custom-router.md)
Build a URL router from scratch.

**When to use**: Understanding web frameworks or building custom routing logic.

**Python equivalent**: Understanding Flask/Django routing internals.

**Key topics**:
- URL pattern matching
- Handler dispatch
- Path parameters
- Route table design
- Pattern matching for routing

---

### Async and Concurrency

#### [How to Use Async/Await](async/async-await-basics.md)
Write asynchronous code using async/await.

**When to use**: I/O-bound operations (network, file I/O).

**Python equivalent**: Using Python's `async def` and `await`.

**Key topics**:
- Async functions
- The `.await` syntax
- Choosing an async runtime (tokio vs async-std)
- Async traits
- Common pitfalls

---

#### [How to Build a Simple Async Executor](async/block-on.md)
Understand async internals by building a basic executor.

**When to use**: You want to deeply understand how async Rust works.

**Python equivalent**: Understanding asyncio event loop internals.

**Key topics**:
- Futures trait
- Polling and pinning
- Wakers and task scheduling
- Executor architecture
- Context and polling state

---

#### [How to Mix Blocking and Async Code](async/spawn-blocking.md)
Integrate blocking operations into async applications.

**When to use**: You have blocking operations (CPU-intensive, blocking I/O) in async code.

**Python equivalent**: Using `asyncio.to_thread()` or `run_in_executor()`.

**Key topics**:
- `spawn_blocking` pattern
- Thread pools
- When to use blocking vs async
- Bridging sync and async
- Performance trade-offs

---

### Foreign Function Interface (FFI)

#### [How to Call C Functions from Rust](ffi/unsafe-ffi.md)
Use unsafe Rust to call C library functions.

**When to use**: Integrating with existing C libraries.

**Python equivalent**: Using `ctypes` or `cffi`.

**Key topics**:
- `extern "C"` blocks
- FFI types (c_int, c_char, etc.)
- Unsafe blocks
- Pointer handling
- Memory ownership across boundaries

---

#### [How to Create Safe Wrappers for C Libraries](ffi/safe-wrappers.md)
Wrap unsafe FFI code in safe Rust APIs.

**When to use**: Making C libraries ergonomic and safe for Rust users.

**Python equivalent**: Creating Pythonic wrappers around C libraries.

**Key topics**:
- RAII pattern
- Drop trait for cleanup
- Encapsulating unsafety
- Error handling across FFI
- Lifetime management

---

### Macros

#### [How to Write Declarative Macros](macros/declarative-macros.md)
Create code-generating macros with `macro_rules!`.

**When to use**: Reducing boilerplate or creating DSLs.

**Python equivalent**: Decorators, metaclasses, or AST manipulation (but more powerful).

**Key topics**:
- `macro_rules!` syntax
- Pattern matching in macros
- Repetition (`$()*`)
- Macro hygiene
- Debugging macros

---

#### [How to Build a JSON Literal Macro](macros/json-macro.md)
Create a macro for JSON literals similar to `serde_json::json!`.

**When to use**: Learning macro metaprogramming by example.

**Python equivalent**: Similar to building a DSL, no direct equivalent.

**Key topics**:
- Nested macro patterns
- Token tree parsing
- HashMap construction
- Array handling
- Type inference in macros

---

### Complete Applications

#### [How to Structure a Multi-Module Project](applications/project-structure.md)
Organize larger Rust applications across modules.

**When to use**: Your project grows beyond a single file.

**Python equivalent**: Organizing code into packages and modules.

**Key topics**:
- Module system
- File organization
- Public vs private APIs
- Re-exports
- Workspace organization

---

#### [How to Build a Simulation Application](applications/fern-sim.md)
Create a complete application with external dependencies.

**When to use**: Building real-world applications with multiple components.

**Python equivalent**: A complete Python application using multiple libraries.

**Key topics**:
- Project dependencies
- Image generation
- Command-line arguments
- Error propagation
- Integration of multiple crates

---

## Guide Selection Helper

**Not sure which guide to use?** Answer these questions:

1. **What are you building?**
   - Data structure → Data Structures section
   - Web service/API → Web and Networking section
   - Library integration → FFI section
   - Performance-critical code → Type System and Safety section

2. **What's your main challenge?**
   - "Type doesn't implement X trait" → Generic Bounds guide
   - "Borrow checker errors" → Type-Safe Wrappers guide
   - "Slow sequential operations" → Concurrent Requests guide
   - "Integrating C library" → FFI guides
   - "Repetitive code" → Macros guides

3. **What Python feature are you trying to replicate?**
   - `collections.deque` → Queue guides
   - `requests.get()` → HTTP Client guide
   - Flask/FastAPI → Actix-web guide
   - `async/await` → Async guides
   - `ctypes` → FFI guides
   - Decorators → Macros guides

## Tips for Using How-To Guides

1. **Adapt, don't copy**: Use the code as a starting point and modify for your needs
2. **Understand trade-offs**: Each approach has pros and cons discussed in the guide
3. **Check versions**: Ensure crate versions match your `Cargo.toml`
4. **Read errors**: Rust's compiler will guide you if something doesn't match your use case
5. **Combine techniques**: Real applications often use multiple patterns together

## Beyond How-To Guides

- **Need to understand why?** → [Explanations](../explanation/index.md)
- **Want to learn from scratch?** → [Tutorials](../tutorials/index.md)
- **Looking up specifics?** → [Reference](../reference/index.md)

---

*These guides show you the path. Your journey makes it your own.*
