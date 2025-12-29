# Pattern Catalog

## Introduction

This catalog organizes all patterns in this book by multiple dimensions: scale, category, common sequences, and anti-patterns. Use this catalog to:

- **Find patterns by size**: Small, focused patterns vs. large architectural patterns
- **Browse by domain**: Ownership, types, concurrency, safety, etc.
- **Learn sequences**: Which patterns work well together
- **Avoid pitfalls**: Common mistakes and how to prevent them

## How to Use This Catalog

### By Scale
Start here if you're learning incrementally—begin with small, foundational patterns and progress to larger architectural patterns.

### By Category
Start here if you have a specific problem domain—find all patterns related to ownership, async programming, unsafe code, etc.

### By Sequence
Start here if you're building a complete feature—see which patterns commonly combine to solve larger problems.

### Anti-Patterns
Start here to learn what to avoid—understand common mistakes before making them.

---

## Patterns by Scale

Patterns organized from smallest (single-line idioms) to largest (architectural patterns).

### Nano Scale (Single Expression / Statement)

**Scope**: Individual expressions, single operations, basic syntax

| Pattern | Category | Source | Description |
|---------|----------|--------|-------------|
| **Fragment Specifiers** | Macros | json-macro | Choose appropriate macro parameter types (expr, ty, ident, tt) |
| **From/Into Conversion** | Types | complex, json-macro | Implement conversion traits for type flexibility |
| **Newtype Pattern** | Types | Expected | Wrap primitive types for type safety |
| **? Operator** | Error Handling | grep, copy | Propagate errors concisely |
| **Iterator Adapters** | Iterators | Expected | Chain iterator methods (map, filter, collect) |

### Micro Scale (Single Function / Method)

**Scope**: Individual functions, methods, small algorithms

| Pattern | Category | Source | Description |
|---------|----------|--------|-------------|
| **Ownership Transfer** | Ownership | queue, generic-queue | Move values to transfer ownership |
| **Borrowing** | Ownership | queue, binary-tree | Reference values without taking ownership |
| **Option Handling** | Error Handling | binary-tree | Use Option for nullable values |
| **Result Handling** | Error Handling | grep, http-get | Use Result for fallible operations |
| **Lifetime Annotations** | Lifetimes | Expected | Specify reference validity scopes |
| **Recursive Functions** | Algorithms | binary-tree | Functions that call themselves |
| **Closure Capture** | Closures | basic-router | Capture environment in closures |

### Meso Scale (Type / Module)

**Scope**: Complete types, traits, modules, small systems

| Pattern | Category | Source | Description |
|---------|----------|--------|-------------|
| **Struct with Methods** | Types | queue, complex | Group data and behavior |
| **Enum with Variants** | Types | binary-tree | Represent choice among alternatives |
| **Generic Types** | Generics | generic-queue | Parameterize types for reusability |
| **Trait Implementation** | Traits | complex, interval | Implement behavior for types |
| **Operator Overloading** | Traits | complex | Implement operator traits (Add, Sub, etc.) |
| **Iterator Implementation** | Iterators | binary-tree | Create custom iteration behavior |
| **Builder Pattern** | API Design | Expected | Construct complex objects step-by-step |
| **RAII (Resource Management)** | Ownership | Expected | Tie resource lifetime to value scope |
| **Macro Rules** | Macros | json-macro | Define declarative macros |
| **Recursive Macro** | Macros | json-macro | Handle nested structures in macros |

### Macro Scale (Multi-Module System)

**Scope**: Multiple modules, crates, larger subsystems

| Pattern | Category | Source | Description |
|---------|----------|--------|-------------|
| **Module Organization** | Architecture | fern_sim | Structure code across multiple files |
| **Public API Design** | API Design | Expected | Define clear public interfaces |
| **Error Type Hierarchy** | Error Handling | Expected | Custom error types with conversions |
| **Trait Object Pattern** | Traits | Expected | Dynamic dispatch with trait objects |
| **Type State Pattern** | Types | Expected | Encode state in type system |
| **DSL Construction** | Macros | json-macro | Build domain-specific languages |
| **Async Runtime Integration** | Async | cheapo-request | Work with async executors |
| **FFI Boundary Design** | Unsafe | libgit2-rs | Interface with C libraries |
| **Safe Wrapper Pattern** | Unsafe | libgit2-rs-safe | Encapsulate unsafe code safely |

### Mega Scale (Architectural)

**Scope**: Complete applications, architectural decisions

| Pattern | Category | Source | Description |
|---------|----------|--------|-------------|
| **Actor Model** | Concurrency | actix-gcd | Message-passing concurrency |
| **Request/Response Pattern** | Web | actix-gcd | HTTP request handling |
| **Concurrent Async Operations** | Async | many-requests | Parallel async tasks |
| **Custom Executor** | Async | spawn-blocking, block-on | Build async runtime components |
| **Procedural Macro System** | Macros | json-macro | Complex compile-time code generation |
| **CLI Application Structure** | Architecture | grep, copy | Command-line tool organization |
| **Web Service Architecture** | Architecture | actix-gcd | Web server structure |

---

## Patterns by Category

Patterns organized by problem domain and technical area.

### Ownership & Borrowing

**Core Principle**: Managing memory safely without garbage collection

| Pattern | Scale | Description | When to Use |
|---------|-------|-------------|-------------|
| **Ownership Transfer** | Micro | Move values between scopes | Transferring responsibility |
| **Immutable Borrowing** | Micro | Read-only references (&T) | Multiple readers needed |
| **Mutable Borrowing** | Micro | Exclusive write access (&mut T) | Single writer needed |
| **Lifetime Annotations** | Micro | Specify reference relationships | Complex reference patterns |
| **RAII** | Meso | Tie cleanup to scope exit | Resource management |
| **Reference Counting (Rc)** | Meso | Shared ownership | Multiple owners needed |
| **Interior Mutability (RefCell)** | Meso | Mutable access through immutable reference | Runtime borrow checking |

**Pattern Sequence**: Start with Ownership Transfer → Try Borrowing → Add Lifetimes if needed → Use RAII for cleanup → Consider Rc/RefCell for shared mutable state

**Related Anti-Patterns**: Fighting the Borrow Checker, Excessive Cloning, Leaking References

### Types & Traits

**Core Principle**: Strong static typing with flexible abstractions

| Pattern | Scale | Description | When to Use |
|---------|-------|-------------|-------------|
| **Struct Definition** | Meso | Group related data | Compound data types |
| **Enum Definition** | Meso | Represent alternatives | Choice among variants |
| **Generic Types** | Meso | Type parameterization | Code reuse across types |
| **Trait Implementation** | Meso | Define behavior | Shared functionality |
| **Operator Overloading** | Meso | Custom operator behavior | Intuitive APIs |
| **From/Into Conversion** | Nano | Type conversions | Flexible function signatures |
| **Newtype Pattern** | Nano | Wrap primitives | Type safety |
| **Type State** | Macro | Encode state in types | Compile-time state checking |
| **Trait Objects** | Macro | Dynamic dispatch | Runtime polymorphism |

**Pattern Sequence**: Define Structs/Enums → Add Generic Parameters → Implement Traits → Use From/Into for conversions → Consider Trait Objects for flexibility

**Related Anti-Patterns**: God Object, Primitive Obsession, Trait Soup

### Error Handling

**Core Principle**: Explicit, type-safe error management

| Pattern | Scale | Description | When to Use |
|---------|-------|-------------|-------------|
| **Option for Nullable** | Micro | Represent optional values | May or may not have value |
| **Result for Fallible** | Micro | Represent success/failure | Operations that can fail |
| **? Operator** | Nano | Propagate errors | Error bubbling |
| **Custom Error Types** | Meso | Domain-specific errors | Rich error information |
| **Error Conversion (From)** | Meso | Convert between error types | Error type compatibility |
| **Error Type Hierarchy** | Macro | Organize error categories | Complex error domains |
| **Error Context** | Meso | Add context to errors | Debugging information |

**Pattern Sequence**: Use Result → Apply ? Operator → Create Custom Error Types → Implement From conversions → Add Error Context

**Related Anti-Patterns**: Unwrap Abuse, Swallowing Errors, Generic Error Messages

### Iterators & Collections

**Core Principle**: Efficient, composable data processing

| Pattern | Scale | Description | When to Use |
|---------|-------|-------------|-------------|
| **Iterator Adapters** | Nano | Chain transformations | Data pipelines |
| **Custom Iterator** | Meso | Implement Iterator trait | Domain-specific iteration |
| **Lazy Evaluation** | Meso | Defer computation | Large datasets |
| **Collecting Results** | Micro | Gather iterator results | Materialize data |
| **Vec for Sequences** | Micro | Dynamic arrays | Growable lists |
| **HashMap for Lookups** | Micro | Key-value storage | Fast lookups |
| **BTreeMap for Ordered** | Micro | Sorted maps | Ordered iteration |

**Pattern Sequence**: Start with Vec/HashMap → Use Iterator Adapters → Implement Custom Iterator if needed → Optimize with Lazy Evaluation

**Related Anti-Patterns**: Eager Evaluation Waste, Index Overuse, Collect Too Early

### Async & Concurrency

**Core Principle**: Efficient concurrent operations without data races

| Pattern | Scale | Description | When to Use |
|---------|-------|-------------|-------------|
| **Async Functions** | Micro | Non-blocking operations | I/O-bound tasks |
| **Future Trait** | Meso | Represent pending work | Async abstractions |
| **Async Runtime Integration** | Macro | Work with executors | Async applications |
| **Concurrent Operations** | Macro | Parallel async tasks | Multiple concurrent I/O |
| **Spawn Blocking** | Macro | Run blocking code in async | CPU-bound in async context |
| **Custom Executor** | Mega | Build runtime components | Specialized scheduling |
| **Actor Model** | Mega | Message-passing concurrency | Isolated concurrent entities |
| **Send/Sync Bounds** | Meso | Thread-safe types | Concurrent data sharing |

**Pattern Sequence**: Define Async Functions → Use Async Runtime → Spawn Concurrent Tasks → Handle Blocking with spawn_blocking → Consider Actor Model for complex state

**Related Anti-Patterns**: Blocking in Async, Over-Threading, Missing Send/Sync, Deadlock Patterns

### Unsafe & FFI

**Core Principle**: Controlled violation of safety guarantees

| Pattern | Scale | Description | When to Use |
|---------|-------|-------------|-------------|
| **Unsafe Block** | Micro | Isolated unsafe operations | Necessary unsafe code |
| **SAFETY Documentation** | Micro | Document invariants | Every unsafe block |
| **Raw Pointers** | Micro | Untracked pointers | Low-level memory access |
| **FFI Boundary** | Macro | C interop | Integrate C libraries |
| **Safe Wrapper** | Macro | Encapsulate unsafe code | Public API safety |
| **Transmute** | Micro | Type reinterpretation | Low-level conversions |
| **Pointer Arithmetic** | Micro | Manual pointer math | Performance-critical code |
| **Minimal Unsafe Scope** | Design | Limit unsafe code | Safety by design |

**Pattern Sequence**: Define FFI Boundary → Encapsulate with Safe Wrapper → Document with SAFETY comments → Test thoroughly → Consider unsafe only as last resort

**Related Anti-Patterns**: Unsafe Everywhere, Missing Invariant Documentation, Unsafe Public API

### Macros & Metaprogramming

**Core Principle**: Compile-time code generation

| Pattern | Scale | Description | When to Use |
|---------|-------|-------------|-------------|
| **Macro Rules** | Meso | Declarative macros | Pattern-based generation |
| **Fragment Specifiers** | Nano | Type macro parameters | Type-safe macros |
| **Recursive Macro** | Meso | Handle nested structures | Arbitrary depth |
| **Macro Hygiene** | Design | Prevent name conflicts | All macros |
| **DSL Construction** | Macro | Domain-specific syntax | Embedded languages |
| **Procedural Macros** | Mega | Custom derive, attributes | Complex generation |
| **$crate Prefix** | Nano | Absolute paths in macros | Cross-crate macros |

**Pattern Sequence**: Start with Macro Rules → Choose Fragment Specifiers → Add Recursion for nesting → Ensure Hygiene → Build DSL if needed

**Related Anti-Patterns**: Over-Engineering with Macros, tt Overuse, Missing $crate, Poor Error Messages

### Web & Networking

**Core Principle**: HTTP services and network communication

| Pattern | Scale | Description | When to Use |
|---------|-------|-------------|-------------|
| **Request/Response** | Macro | HTTP request handling | Web services |
| **Route Handling** | Meso | URL routing | Web applications |
| **Middleware Pattern** | Macro | Request/response pipeline | Cross-cutting concerns |
| **Actor-based Server** | Mega | Message-passing web server | Actix-web style |
| **HTTP Client** | Meso | Make HTTP requests | API consumption |
| **Async HTTP** | Macro | Non-blocking HTTP | Concurrent requests |

**Pattern Sequence**: Define Routes → Handle Requests → Add Middleware → Use Async for concurrency → Consider Actor Model for state

**Related Anti-Patterns**: Blocking in Web Handlers, Synchronous HTTP in Async Context, Missing Error Handling

### Architecture & Design

**Core Principle**: Code organization and structure

| Pattern | Scale | Description | When to Use |
|---------|-------|-------------|-------------|
| **Module Organization** | Macro | Multi-file structure | Large projects |
| **Public API Design** | Macro | Clear interfaces | Library development |
| **Builder Pattern** | Meso | Flexible construction | Complex objects |
| **Type State** | Macro | State in types | Compile-time states |
| **Newtype** | Nano | Semantic types | Type safety |
| **CLI Structure** | Mega | Command-line apps | CLI tools |
| **Web Service Architecture** | Mega | Server organization | Web applications |

**Pattern Sequence**: Organize Modules → Design Public API → Use Builder for complex objects → Apply Newtype for safety → Consider Type State for state machines

**Related Anti-Patterns**: God Module, Leaky Abstractions, Premature Abstraction

---

## Pattern Sequences (Common Combinations)

Patterns that frequently appear together to solve larger problems.

### Sequence 1: Building a Library Type

**Goal**: Create a reusable, well-designed library type

```
1. Struct Definition (define the type)
   ↓
2. Generic Parameters (make it reusable)
   ↓
3. Trait Implementations (add standard behavior)
   ↓
4. From/Into Conversions (flexible construction)
   ↓
5. Custom Iterator (if needed)
   ↓
6. Public API Design (clean interface)
```

**Example**: generic-queue, binary-tree

**Outcome**: Type-safe, ergonomic library component

### Sequence 2: Web Service Development

**Goal**: Build a web application with Actix-web

```
1. Request/Response Pattern (basic handlers)
   ↓
2. Route Handling (URL mapping)
   ↓
3. Async Functions (non-blocking handlers)
   ↓
4. Error Handling (Result types)
   ↓
5. Actor Model (state management)
   ↓
6. Middleware Pattern (cross-cutting concerns)
```

**Example**: actix-gcd

**Outcome**: Scalable, concurrent web service

### Sequence 3: Async I/O Application

**Goal**: Perform concurrent I/O operations efficiently

```
1. Async Functions (define async operations)
   ↓
2. Async Runtime Integration (tokio/async-std)
   ↓
3. Concurrent Operations (spawn multiple tasks)
   ↓
4. Error Handling (async-aware Result)
   ↓
5. Spawn Blocking (handle blocking code)
   ↓
6. Custom Executor (if needed)
```

**Example**: cheapo-request → many-requests → spawn-blocking

**Outcome**: Efficient concurrent I/O application

### Sequence 4: CLI Tool Development

**Goal**: Create a command-line application

```
1. CLI Structure (argument parsing)
   ↓
2. Error Handling (custom error types)
   ↓
3. File I/O (read/write operations)
   ↓
4. Iterator Adapters (process data)
   ↓
5. Result Handling (error propagation)
   ↓
6. Public API Design (if reusable)
```

**Example**: grep, copy

**Outcome**: Robust, user-friendly CLI tool

### Sequence 5: FFI Integration

**Goal**: Safely integrate C library

```
1. FFI Boundary (extern blocks)
   ↓
2. Unsafe Blocks (call C functions)
   ↓
3. SAFETY Documentation (document invariants)
   ↓
4. Safe Wrapper (encapsulate unsafe)
   ↓
5. Error Handling (convert C errors)
   ↓
6. RAII (manage C resources)
```

**Example**: libgit2-rs → libgit2-rs-safe

**Outcome**: Safe Rust API over C library

### Sequence 6: DSL Implementation

**Goal**: Build a domain-specific language

```
1. Macro Rules (define basic syntax)
   ↓
2. Fragment Specifiers (type-safe parameters)
   ↓
3. Recursive Macro (handle nesting)
   ↓
4. Macro Hygiene (prevent conflicts)
   ↓
5. DSL Construction (complete language)
   ↓
6. Testing & Documentation (ensure correctness)
```

**Example**: json-macro (complete DSL)

**Outcome**: Embedded domain-specific language

### Sequence 7: Type-Safe Builder

**Goal**: Construct complex objects safely

```
1. Struct Definition (base type)
   ↓
2. Builder Pattern (incremental construction)
   ↓
3. Type State (enforce construction order)
   ↓
4. Generic Parameters (flexibility)
   ↓
5. From/Into (conversions)
   ↓
6. Error Handling (validation)
```

**Example**: Common in API design

**Outcome**: Ergonomic, type-safe construction

### Sequence 8: Iterator Pipeline

**Goal**: Process data efficiently

```
1. Iterator Source (Vec, HashMap, custom)
   ↓
2. Iterator Adapters (map, filter, etc.)
   ↓
3. Lazy Evaluation (defer computation)
   ↓
4. Error Handling (Result in pipeline)
   ↓
5. Collecting Results (materialize)
   ↓
6. Custom Iterator (if needed)
```

**Example**: Data processing in grep, binary-tree

**Outcome**: Efficient, composable data pipeline

---

## Anti-Patterns

Common mistakes to avoid, with explanations and better alternatives.

### Ownership Anti-Patterns

#### Fighting the Borrow Checker

**Symptom**: Constantly struggling with lifetime errors, excessive cloning

**Problem**:
```rust
// ❌ Trying to return reference to local
fn bad() -> &str {
    let s = String::from("hello");
    &s  // ERROR: s is dropped
}
```

**Solution**: Return owned value or accept borrow
```rust
// ✅ Return owned value
fn good() -> String {
    String::from("hello")
}

// ✅ Or accept parameter
fn good2(s: &str) -> &str {
    s
}
```

**Better Pattern**: OWNERSHIP TRANSFER, BORROWING

#### Excessive Cloning

**Symptom**: .clone() everywhere, performance issues

**Problem**:
```rust
// ❌ Unnecessary clones
fn process(data: Vec<i32>) {
    let copy1 = data.clone();
    let copy2 = data.clone();
    // ... each clone allocates
}
```

**Solution**: Use borrowing
```rust
// ✅ Borrow instead
fn process(data: &Vec<i32>) {
    // Work with data without cloning
}
```

**Better Pattern**: BORROWING, REFERENCE COUNTING (if truly needed)

#### Leaking References

**Symptom**: Trying to store references that outlive their data

**Problem**:
```rust
// ❌ Reference outlives data
struct BadContainer<'a> {
    data: &'a str,
}

fn bad() -> BadContainer<'static> {
    let s = String::from("temp");
    BadContainer { data: &s }  // ERROR
}
```

**Solution**: Store owned data
```rust
// ✅ Own the data
struct GoodContainer {
    data: String,
}
```

**Better Pattern**: OWNERSHIP TRANSFER, RAII

### Type Anti-Patterns

#### Primitive Obsession

**Symptom**: Using i32, String everywhere instead of domain types

**Problem**:
```rust
// ❌ What do these represent?
fn transfer(from: i32, to: i32, amount: i32) { }
```

**Solution**: Use newtypes
```rust
// ✅ Clear semantics
struct AccountId(i32);
struct Amount(i32);

fn transfer(from: AccountId, to: AccountId, amount: Amount) { }
```

**Better Pattern**: NEWTYPE PATTERN

#### God Object

**Symptom**: One struct with too many responsibilities

**Problem**:
```rust
// ❌ Does everything
struct Application {
    config: Config,
    database: Database,
    server: Server,
    logger: Logger,
    cache: Cache,
    // ... 20 more fields
}
```

**Solution**: Separate concerns
```rust
// ✅ Focused types
struct Application {
    config: Config,
    services: Services,
}

struct Services {
    database: Database,
    server: Server,
}
```

**Better Pattern**: MODULE ORGANIZATION, PUBLIC API DESIGN

#### Trait Soup

**Symptom**: Too many small traits, hard to understand what implements what

**Problem**:
```rust
// ❌ Fragmented behavior
trait Readable { fn read(&self) -> String; }
trait Writable { fn write(&self, s: String); }
trait Seekable { fn seek(&self, pos: u64); }
trait Flushable { fn flush(&self); }
// ... user implements all 4
```

**Solution**: Combine related traits
```rust
// ✅ Cohesive trait
trait File: Read + Write + Seek {
    // Combined functionality
}
```

**Better Pattern**: TRAIT IMPLEMENTATION (cohesive traits)

### Error Handling Anti-Patterns

#### Unwrap Abuse

**Symptom**: .unwrap() everywhere, panics in production

**Problem**:
```rust
// ❌ Panics on error
fn bad(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()
}
```

**Solution**: Propagate errors
```rust
// ✅ Return Result
fn good(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}
```

**Better Pattern**: RESULT HANDLING, ? OPERATOR

#### Swallowing Errors

**Symptom**: Catching errors but not handling them

**Problem**:
```rust
// ❌ Ignores errors
fn bad(path: &str) {
    let _ = std::fs::read_to_string(path);
    // Error silently ignored
}
```

**Solution**: Handle or propagate
```rust
// ✅ Log error
fn good(path: &str) -> Result<(), std::io::Error> {
    match std::fs::read_to_string(path) {
        Ok(content) => { /* use */ },
        Err(e) => {
            eprintln!("Error reading {}: {}", path, e);
            return Err(e);
        }
    }
    Ok(())
}
```

**Better Pattern**: ERROR HANDLING, CUSTOM ERROR TYPES

#### Generic Error Messages

**Symptom**: Errors like "something went wrong" with no context

**Problem**:
```rust
// ❌ Useless error
Err("Error")
```

**Solution**: Provide context
```rust
// ✅ Informative error
Err(format!("Failed to open file '{}': {}", path, e))
```

**Better Pattern**: ERROR CONTEXT, CUSTOM ERROR TYPES

### Async Anti-Patterns

#### Blocking in Async

**Symptom**: Using blocking operations in async context

**Problem**:
```rust
// ❌ Blocks executor thread
async fn bad() {
    std::thread::sleep(Duration::from_secs(1));
}
```

**Solution**: Use async sleep
```rust
// ✅ Yields to executor
async fn good() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

**Better Pattern**: ASYNC FUNCTIONS, SPAWN BLOCKING

#### Over-Threading

**Symptom**: Creating threads instead of using async

**Problem**:
```rust
// ❌ Thread per request (doesn't scale)
for request in requests {
    std::thread::spawn(move || {
        handle(request);
    });
}
```

**Solution**: Use async tasks
```rust
// ✅ Lightweight async tasks
for request in requests {
    tokio::spawn(async move {
        handle(request).await;
    });
}
```

**Better Pattern**: CONCURRENT ASYNC OPERATIONS

#### Missing Send/Sync

**Symptom**: Trying to send non-Send types across threads

**Problem**:
```rust
// ❌ Rc is not Send
async fn bad(rc: Rc<i32>) {
    tokio::spawn(async move {
        println!("{}", rc);  // ERROR: Rc not Send
    });
}
```

**Solution**: Use Send types
```rust
// ✅ Arc is Send
async fn good(arc: Arc<i32>) {
    tokio::spawn(async move {
        println!("{}", arc);
    });
}
```

**Better Pattern**: SEND/SYNC BOUNDS

### Macro Anti-Patterns

#### Over-Engineering with Macros

**Symptom**: Using macros for simple tasks

**Problem**:
```rust
// ❌ Macro for simple addition
macro_rules! add {
    ($a:expr, $b:expr) => { $a + $b };
}
```

**Solution**: Use a function
```rust
// ✅ Function is clearer
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Better Pattern**: Functions for simple logic, macros for syntax

#### tt Overuse

**Symptom**: Using tt for everything, losing type safety

**Problem**:
```rust
// ❌ No type checking
macro_rules! bad {
    ($value:tt) => {
        let x: i32 = $value;  // What if not i32?
    };
}
```

**Solution**: Use specific specifier
```rust
// ✅ Type-safe
macro_rules! good {
    ($value:expr) => {
        let x: i32 = $value;
    };
}
```

**Better Pattern**: FRAGMENT SPECIFIERS

#### Missing $crate

**Symptom**: Exported macros break in other crates

**Problem**:
```rust
// ❌ Assumes Json in scope
#[macro_export]
macro_rules! bad {
    () => { Json::Null };
}
```

**Solution**: Use absolute path
```rust
// ✅ Works across crates
#[macro_export]
macro_rules! good {
    () => { $crate::Json::Null };
}
```

**Better Pattern**: MACRO HYGIENE, $CRATE PREFIX

### Unsafe Anti-Patterns

#### Unsafe Everywhere

**Symptom**: Large unsafe blocks, unclear invariants

**Problem**:
```rust
// ❌ Too much unsafe
unsafe {
    // 100 lines of code
    // What's unsafe? What are invariants?
}
```

**Solution**: Minimize unsafe scope
```rust
// ✅ Isolated unsafe
fn safe_function() {
    // Safe code
    unsafe {
        // One specific unsafe operation
        // SAFETY: explain why this is safe
    }
    // More safe code
}
```

**Better Pattern**: MINIMAL UNSAFE SCOPE, SAFETY DOCUMENTATION

#### Missing Invariant Documentation

**Symptom**: Unsafe code without explaining why it's safe

**Problem**:
```rust
// ❌ No explanation
unsafe {
    *ptr = 42;
}
```

**Solution**: Document invariants
```rust
// ✅ Explained
unsafe {
    // SAFETY: ptr is non-null and properly aligned,
    // and we have exclusive access to it within this scope.
    *ptr = 42;
}
```

**Better Pattern**: SAFETY DOCUMENTATION

#### Unsafe Public API

**Symptom**: Exposing unsafe functions when safe wrapper possible

**Problem**:
```rust
// ❌ Unsafe public API
pub unsafe fn do_thing(ptr: *mut i32) {
    *ptr = 42;
}
```

**Solution**: Provide safe wrapper
```rust
// ✅ Safe public API
pub fn do_thing(value: &mut i32) {
    unsafe {
        // SAFETY: reference is valid
        *value = 42;
    }
}
```

**Better Pattern**: SAFE WRAPPER PATTERN

---

## Pattern Relationships

### Dependencies

Patterns that require or build upon other patterns:

- **Generic Types** requires **Struct/Enum Definition**
- **Trait Implementation** requires **Type Definition**
- **Custom Iterator** requires **Iterator Adapters** knowledge
- **Recursive Macro** requires **Macro Rules**
- **DSL Construction** requires **Macro Rules**, **Fragment Specifiers**, **Recursive Macro**, **Macro Hygiene**
- **Safe Wrapper** requires **Unsafe Blocks**, **FFI Boundary**
- **Concurrent Async Operations** requires **Async Functions**, **Async Runtime**

### Alternatives

Patterns that solve similar problems differently:

- **Ownership Transfer** vs **Borrowing** vs **Reference Counting**
- **Option** vs **Result** (nullable vs fallible)
- **Trait Objects** vs **Generic Types** (dynamic vs static dispatch)
- **Async** vs **Threading** (I/O-bound vs CPU-bound)
- **Macro Rules** vs **Procedural Macros** (declarative vs procedural)

### Complements

Patterns that work well together:

- **Struct Definition** + **Trait Implementation**
- **Generic Types** + **From/Into Conversion**
- **Error Handling** + **? Operator**
- **Async Functions** + **Concurrent Operations**
- **Module Organization** + **Public API Design**
- **FFI Boundary** + **Safe Wrapper**

---

## Learning Paths

### Path 1: Beginner → Intermediate

**Focus**: Ownership, types, basic patterns

1. Ownership Transfer
2. Borrowing
3. Struct Definition
4. Enum Definition
5. Option Handling
6. Result Handling
7. ? Operator
8. Trait Implementation
9. Iterator Adapters
10. Module Organization

**Projects**: gcd → queue → complex → echo-server

### Path 2: Intermediate → Advanced

**Focus**: Generics, async, macros

1. Generic Types
2. Lifetime Annotations
3. Async Functions
4. Async Runtime Integration
5. Macro Rules
6. Fragment Specifiers
7. Custom Iterator
8. Error Type Hierarchy

**Projects**: generic-queue → binary-tree → cheapo-request → json-macro

### Path 3: Advanced → Expert

**Focus**: Unsafe, FFI, complex systems

1. Unsafe Blocks
2. FFI Boundary
3. Safe Wrapper
4. Custom Executor
5. Procedural Macros
6. Concurrent Async Operations
7. Actor Model

**Projects**: libgit2-rs → libgit2-rs-safe → spawn-blocking → block-on

---

## Quick Reference

### By Problem Type

| I Need To... | Use Pattern | Category |
|--------------|-------------|----------|
| Avoid copying data | Borrowing | Ownership |
| Handle optional values | Option | Error Handling |
| Handle errors | Result, ? Operator | Error Handling |
| Reuse code across types | Generic Types | Types |
| Define custom operators | Operator Overloading | Traits |
| Process sequences | Iterator Adapters | Iterators |
| Do concurrent I/O | Async Functions | Async |
| Run multiple tasks | Concurrent Operations | Async |
| Generate code | Macro Rules | Macros |
| Handle nested syntax | Recursive Macro | Macros |
| Call C code | FFI Boundary | Unsafe |
| Make C code safe | Safe Wrapper | Unsafe |

### By Difficulty

**Easy**: Borrowing, Option, Result, ? Operator, Struct Definition, Iterator Adapters

**Medium**: Generic Types, Trait Implementation, Async Functions, Macro Rules, Module Organization

**Hard**: Lifetimes, Custom Iterator, Recursive Macro, FFI Boundary, Concurrent Async

**Expert**: Unsafe Blocks, Safe Wrapper, Custom Executor, Procedural Macros, DSL Construction

---

## Conclusion

This catalog provides multiple ways to navigate the pattern space:

- **Scale**: From small to large
- **Category**: By technical domain
- **Sequence**: Common combinations
- **Anti-Patterns**: What to avoid

Use this catalog as a reference when:
- Learning Rust incrementally
- Solving specific problems
- Building complete systems
- Avoiding common mistakes
- Understanding pattern relationships

For detailed information on each pattern, see the individual pattern chapters in this book.
