# Evolution of Architecture Patterns (2022-2026)

## Learning Objectives

By the end of this chapter, you will:
- Understand which patterns have become idiomatic
- Recognize patterns that have fallen out of favor
- Learn from production Rust systems since 2022
- Apply lessons from large-scale deployments

## Introduction

The Rust community learns fast. Patterns that were experimental in 2021 are now production-standard in 2026. This chapter traces the evolution of architectural patterns across the ecosystem, examining what changed and why, with lessons drawn from the 24 projects in this repository and the broader production landscape.

## Pattern Lifecycle: From Experiment to Standard

### The Pattern Maturity Model

```
Experimental → Emerging → Established → Standard → Legacy
    ↓             ↓           ↓            ↓          ↓
  Few users   Early      Production   Ecosystem   Superseded
              adopters    proven       default
```

**Timeline observations (2022-2026)**:
- Async/await: Established → Standard
- GATs: Experimental → Established
- Proc macros: Emerging → Established
- Manual futures: Standard → Legacy (for most use cases)

## Patterns That Became Idiomatic

### 1. The Newtype Pattern (Standard → Ubiquitous)

**2021 status**: Recommended in Rust book, inconsistently applied.

**2026 status**: Industry standard for type safety.

**Evidence from our projects**:

```rust
// gcd project (2021): Used raw types
fn gcd(a: u64, b: u64) -> u64 { /* ... */ }

// Modern version (2026): Type-safe wrappers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonZeroU64(u64);

impl NonZeroU64 {
    pub fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(NonZeroU64(value))
        }
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

// Prevents zero denominator at compile time
fn gcd(a: NonZeroU64, b: NonZeroU64) -> NonZeroU64 {
    // Implementation guarantees non-zero output
}
```

**Why it won**: Type system catches bugs at compile time, zero runtime cost.

### 2. Builder Pattern with Typestate

**2021 status**: Niche, seen in some libraries.

**2026 status**: Standard for complex construction.

**Evolution from queue → typestate queue**:

```rust
// 2021: Simple builder
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self { /* ... */ }
    pub fn with_capacity(cap: usize) -> Self { /* ... */ }
}

// 2026: Typestate builder prevents misuse
pub struct QueueBuilder<State> {
    capacity: Option<usize>,
    state: PhantomData<State>,
}

pub struct Unconfigured;
pub struct Configured;

impl QueueBuilder<Unconfigured> {
    pub fn new() -> Self {
        QueueBuilder {
            capacity: None,
            state: PhantomData,
        }
    }

    pub fn with_capacity(self, cap: usize) -> QueueBuilder<Configured> {
        QueueBuilder {
            capacity: Some(cap),
            state: PhantomData,
        }
    }
}

impl QueueBuilder<Configured> {
    pub fn build<T>(self) -> Queue<T> {
        Queue {
            older: Vec::with_capacity(self.capacity.unwrap()),
            younger: Vec::with_capacity(self.capacity.unwrap()),
        }
    }
}

// Compile error if you forget configuration:
// QueueBuilder::new().build()  // ERROR: can't build Unconfigured
```

**Why it won**: Prevents runtime errors, self-documenting API.

### 3. Error Handling with thiserror + anyhow

**2021 status**: Emerging pattern, competing approaches.

**2026 status**: De facto standard.

**Application to our projects** (e.g., `grep`, `http-get`):

```rust
// 2021 pattern: Box<dyn Error>
fn search(pattern: &str, path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(path)?;
    // ...
}

// 2026 pattern: thiserror for libraries
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GrepError {
    #[error("Failed to open file: {path}")]
    FileOpen { path: PathBuf, source: io::Error },

    #[error("Invalid regex pattern: {pattern}")]
    InvalidPattern { pattern: String, source: regex::Error },

    #[error("UTF-8 decoding failed")]
    Utf8Error(#[from] std::str::Utf8Error),
}

// Libraries use custom error types
pub fn search(pattern: &str, path: &Path) -> Result<Vec<String>, GrepError> {
    // Precise error context
}

// 2026 pattern: anyhow for applications
use anyhow::{Context, Result};

// Applications use anyhow for convenience
fn main() -> Result<()> {
    let results = search(pattern, path)
        .context("Search operation failed")?;
    Ok(())
}
```

**Why it won**: Clear separation—libraries provide specific errors, applications get convenience.

### 4. Async Trait Patterns

**2021 status**: Required `async-trait` macro, awkward.

**2026 status**: Native async in traits (stable 1.75).

**Impact on `spawn-blocking` style code**:

```rust
// 2021: async-trait macro required
#[async_trait]
pub trait AsyncExecutor {
    async fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send;
}

// 2026: Native syntax
pub trait AsyncExecutor {
    async fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send;
}

// No macro magic, better error messages, faster compilation
```

**Why it won**: Language support always beats macros.

## Patterns That Stabilized

### 5. Repository Structure (Cargo Workspaces)

**2021**: Each project independent.

**2026**: Workspace organization standard.

**Our repository modernized**:

```
rust-programming-examples/
├── Cargo.toml              # Workspace root
├── Cargo.lock              # Shared lock file
├── members/
│   ├── gcd/
│   ├── actix-gcd/
│   ├── binary-tree/
│   └── ... (all 24 projects)
├── shared/                 # Shared utilities
│   └── test-utils/
└── book/                   # This mdbook
```

**Benefits realized**:
- Single `cargo test --workspace` runs all tests
- Dependency deduplication across projects
- Consistent versioning

### 6. Integration Testing Patterns

**2021**: Ad-hoc test organization.

**2026**: Standardized structure.

**From `actix-gcd` modernization**:

```
actix-gcd/
├── src/
│   └── main.rs
├── tests/
│   ├── common/
│   │   └── mod.rs        # Shared test infrastructure
│   ├── http_tests.rs     # Integration tests
│   ├── api_tests.rs
│   └── load_tests.rs
└── benches/
    └── server_bench.rs
```

**Common pattern** (`tests/common/mod.rs`):

```rust
// Shared test server fixture
use actix_web::{test, App};

pub async fn create_test_app() -> impl actix_web::dev::Service {
    test::init_service(
        App::new()
            .service(/* routes */)
    ).await
}

pub async fn test_gcd_request(n: u64, m: u64) -> u64 {
    let app = create_test_app().await;
    let req = test::TestRequest::post()
        .uri(&format!("/gcd?n={}&m={}", n, m))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Parse response...
}
```

**Why it stabilized**: Reduces test boilerplate, improves maintainability.

## Patterns That Fell Out of Favor

### 7. Manual Future Implementation

**2021 status**: Common for async primitives.

**2026 status**: Rarely necessary.

**Example from `block-on` project**:

The entire `block-on` project demonstrated manual future polling:

```rust
// 2021: Educational, sometimes necessary
impl Future for MyFuture {
    type Output = i32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
        // Manual state machine...
    }
}

// 2026: Use runtime primitives
tokio::spawn(async {
    // Runtime handles future lifecycle
})
```

**Why it declined**: Runtimes matured, manual implementation error-prone. Still valuable for education (hence `block-on` remains relevant teaching tool).

### 8. Overuse of Macros

**2021-2023**: Macro fever—many libraries overused procedural macros.

**2026**: Retrenchment—const fn and GATs often better.

**Example evolution**:

```rust
// 2021-2023: Macro for compile-time map
macro_rules! const_map {
    ($($key:expr => $value:expr),* $(,)?) => {
        // Complex macro generating HashMap at compile time
    };
}

// 2026: Const fn with array
const fn create_map() -> &'static [(u32, &'static str)] {
    &[
        (1, "one"),
        (2, "two"),
        (3, "three"),
    ]
}

// Simpler, debuggable, better compile times
```

**Lesson**: `json-macro` justified its complexity (clear DSL benefit), but many macro uses don't.

### 9. Arc<Mutex<T>> Overuse in Async

**2021-2023**: Default pattern for shared state.

**2026**: Recognized as anti-pattern, better alternatives emerged.

```rust
// 2021: Overused pattern
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));

tokio::spawn({
    let counter = counter.clone();
    async move {
        let mut count = counter.lock().unwrap();  // Blocks executor!
        *count += 1;
    }
});

// 2026: Async-aware primitives
use tokio::sync::Mutex;  // Async mutex

let counter = Arc::new(Mutex::new(0));

tokio::spawn({
    let counter = counter.clone();
    async move {
        let mut count = counter.lock().await;  // Yields to executor
        *count += 1;
    }
});

// Or better: channels for communication
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(100);

tokio::spawn(async move {
    while let Some(msg) = rx.recv().await {
        // Process message
    }
});

tx.send(Message::Increment).await.unwrap();
```

**Why it changed**: Blocking in async is invisible performance killer.

## Lessons from Large Production Systems (2022-2026)

### Case Study: Discord (Rust since 2020)

**Key patterns adopted**:
1. **Service isolation**: Each microservice owns its data, no shared state
2. **Async all the way**: No blocking operations in async context
3. **Zero-copy deserialization**: Custom serde deserializers for hot paths
4. **Explicit error handling**: No `.unwrap()` in production code

**Pattern from our `actix-gcd` applied at scale**:

```rust
// Principle: Request handlers return Result, not panic
async fn handle_gcd(
    query: web::Query<GcdParams>,
) -> Result<HttpResponse, ApiError> {
    let result = gcd(query.n, query.m)?;
    Ok(HttpResponse::Ok().json(GcdResponse { result }))
}

// At Discord scale: comprehensive error types, metrics, tracing
```

### Case Study: Cloudflare Workers (Rust at edge)

**Key patterns**:
1. **WebAssembly targets**: Code compiles to WASM for edge deployment
2. **Zero dependencies**: Minimize binary size
3. **Deterministic performance**: No unpredictable garbage collection

**Relevance to our projects**: The `complex` math project compiles to WASM beautifully:

```bash
# 2026: WASM is first-class target
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release

# Result: ~30KB WASM binary
```

### Case Study: AWS (Firecracker, Bottlerocket)

**Key patterns**:
1. **Minimal unsafe**: Safe wrappers over FFI
2. **Extensive testing**: Property-based, fuzzing standard
3. **Performance budgets**: Nanosecond-level latency requirements

**Pattern from our `gap-buffer` unsafe code**:

```rust
// Production pattern: Unsafe isolated, extensively tested
pub struct GapBuffer<T> {
    storage: Vec<T>,
    gap_start: usize,
    gap_end: usize,
}

impl<T> GapBuffer<T> {
    // Safe API
    pub fn insert(&mut self, value: T) {
        // SAFETY: Invariants documented and tested
        unsafe {
            self.insert_unchecked(value);
        }
    }

    // Unsafe internals isolated
    unsafe fn insert_unchecked(&mut self, value: T) {
        debug_assert!(self.gap_start < self.gap_end);
        // Implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn gap_buffer_never_corrupts(
            inserts in prop::collection::vec(0..255u8, 0..1000)
        ) {
            let mut buffer = GapBuffer::new();
            for value in inserts {
                buffer.insert(value);
                assert!(buffer.verify_invariants());
            }
        }
    }
}
```

## Pattern Decision Matrix (2026)

| Use Case | 2021 Pattern | 2026 Pattern | Reason for Change |
|----------|--------------|--------------|-------------------|
| Async traits | `async-trait` macro | Native async | Language support |
| Error handling (lib) | `Box<dyn Error>` | `thiserror` | Precise types |
| Error handling (bin) | Manual `Result` | `anyhow` | Ergonomics |
| Shared async state | `Arc<Mutex<T>>` | Channels / `Arc<Tokio::Mutex>` | Avoid blocking |
| Type safety | Raw types | Newtype pattern | Compile-time verification |
| Complex construction | Direct construction | Typestate builder | API safety |
| Manual futures | Common | Rare | Runtime handles it |
| JSON generation | Runtime parsing | `serde` / macros | Performance + ergonomics |

## What's Emerging (2024-2026)

### 1. Effect Systems (Experimental)

**Not in stable Rust, but influencing design**:

```rust
// Hypothetical future: tracked effects
async fn fetch_user(id: UserId) -> Result<User>
    effects(IO, Network, Database)  // Explicit effect tracking
{
    let data = database::query(id).await;  // Database effect
    Ok(parse_user(data)?)
}

// Compiler verifies: Pure functions have no effects
fn parse_user(data: &[u8]) -> Result<User>
    // No effects - can be safely parallelized, cached, etc.
{
    // ...
}
```

**Current workaround**: Separate pure and effectful code by convention.

### 2. Embedded Async (embassy)

**Growing rapidly for embedded systems**:

```rust
// 2026: Async on microcontrollers
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    spawner.spawn(blink_task(p.LED)).unwrap();
    spawner.spawn(uart_task(p.UART)).unwrap();
}

#[embassy_executor::task]
async fn blink_task(led: AnyPin) {
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
```

**Relevance**: Async patterns from `cheapo-request` now work on 32KB RAM devices.

## Idiomatic Rust Checklist (2026 Edition)

For each project in this repository, modernization would involve:

- [ ] **Async**: Native async in traits, no `async-trait` macro
- [ ] **Errors**: `thiserror` for libraries, `anyhow` for binaries
- [ ] **Testing**: Property-based tests for core logic
- [ ] **Documentation**: Executable doc tests with examples
- [ ] **Workspace**: Multi-crate workspace with shared dependencies
- [ ] **Security**: `cargo audit` in CI, dependencies vetted
- [ ] **Type safety**: Newtype wrappers for domain types
- [ ] **Builder patterns**: Typestate builders for complex construction
- [ ] **No blocking in async**: Use async mutexes and channels
- [ ] **Edition 2021**: Latest edition for newest features

## Summary

The Rust ecosystem has matured dramatically since 2021:

**Patterns that won**:
- Newtype pattern for type safety
- Typestate builders for safe APIs
- Native async in traits
- `thiserror`/`anyhow` for error handling
- Cargo workspaces for organization

**Patterns that declined**:
- Manual future implementation
- Macro overuse
- Blocking primitives in async code

**Lessons from production**:
- Test exhaustively (property-based, fuzzing)
- Isolate unsafe code
- Make correctness obvious (types > comments)
- Measure performance, don't assume

The 24 projects in this repository teach timeless principles. Modern patterns layer improvements on top, but ownership, borrowing, and zero-cost abstractions remain foundational.

## Further Reading

- Chapter 11.3: Resilient systems patterns from production
- Chapter 11.1: Modern ecosystem tooling
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
