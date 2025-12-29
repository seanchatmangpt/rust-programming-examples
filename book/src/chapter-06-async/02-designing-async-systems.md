# Designing Async Systems

Designing async systems in Rust requires thinking beyond syntax—you must consider function signatures, trait bounds, backpressure, and the system-wide implications of async propagation. These design choices determine whether your system scales gracefully or collapses under load.

## Async Function Signatures and Contracts

An async function signature communicates more than parameter types—it establishes a contract about execution model, threading requirements, and error handling. Consider these alternatives:

```rust
// Basic async function - no threading constraints
async fn fetch_data(url: &str) -> Result<String, Error> {
    let response = http_get(url).await?;
    Ok(response.text().await?)
}

// Send-bound - can execute across threads
async fn fetch_data_parallel(url: String) -> Result<String, Error>
where
    Self: Send,  // Implicit in function, explicit when needed
{
    let response = http_get(&url).await?;
    Ok(response.text().await?)
}

// With explicit lifetime - borrows from caller
async fn fetch_data_borrowed<'a>(url: &'a str) -> Result<String, Error> {
    let response = http_get(url).await?;
    Ok(response.text().await?)
}
```

Each signature makes different guarantees:

**Signature 1**: Works anywhere, but can't be spawned on multi-threaded executors if it captures non-`Send` data.

**Signature 2**: Takes ownership of the URL (`String` instead of `&str`), ensuring the future is self-contained and `Send`. This is critical for `tokio::spawn` or `async_std::task::spawn`.

**Signature 3**: Borrows the URL, tying the future's lifetime to the caller. Cannot be spawned unless the caller outlives the task—typically requires `'static` for spawning.

### The Send Boundary

The `Send` trait creates a critical boundary in async systems. A future is `Send` if all data it holds across `.await` points is `Send`. This seemingly simple rule has profound implications:

```rust
use std::rc::Rc;

async fn not_send() {
    let rc = Rc::new(42);  // Rc is not Send
    some_async_fn().await;
    println!("{}", rc);    // rc held across await
}  // Future is NOT Send

async fn is_send() {
    {
        let rc = Rc::new(42);
        println!("{}", rc);
    }  // rc dropped before await
    some_async_fn().await;
}  // Future IS Send - rc not held across await
```

This affects your architecture: if your system uses multi-threaded executors (like Tokio's default), every async function in your call stack must be `Send`. One `Rc` or `RefCell` held across an `.await` makes the entire future non-`Send`.

**Design Principle**: Prefer owned, `Send` types in async function parameters when spawning is likely. Use borrowing for synchronous sections or single-threaded executors.

## Backpressure and Flow Control

A well-designed async system handles backpressure—the ability to slow down producers when consumers can't keep up. Without it, systems buffer unboundedly and crash.

### The Unbounded Problem

```rust
// DANGEROUS: No backpressure
async fn process_unbounded(urls: Vec<String>) {
    let mut handles = vec![];
    for url in urls {
        // Spawns ALL requests immediately
        handles.push(task::spawn(fetch_data(url)));
    }
    for handle in handles {
        handle.await;
    }
}
```

If `urls` contains 1 million entries, this spawns 1 million concurrent tasks. Even though each task is lightweight, the aggregate memory and connection count will overwhelm the system.

### Stream-Based Backpressure

```rust
use futures::stream::{self, StreamExt};

async fn process_with_backpressure(urls: Vec<String>) {
    stream::iter(urls)
        .map(|url| fetch_data(url))
        .buffer_unordered(100)  // At most 100 concurrent requests
        .for_each(|result| async move {
            match result {
                Ok(data) => process(data).await,
                Err(e) => eprintln!("Error: {}", e),
            }
        })
        .await;
}
```

The `buffer_unordered(100)` combinator limits concurrency to 100 requests. When 100 are in-flight, the stream stops pulling new URLs—creating backpressure upstream. This keeps memory constant regardless of input size.

### Channel-Based Backpressure

```rust
use async_std::channel;

async fn producer_consumer(items: Vec<String>) {
    let (sender, receiver) = channel::bounded(50);  // Bounded channel

    // Producer task
    task::spawn(async move {
        for item in items {
            sender.send(item).await.unwrap();  // Blocks when channel full
        }
    });

    // Consumer task
    while let Ok(item) = receiver.recv().await {
        expensive_processing(item).await;
    }
}
```

The `bounded(50)` channel creates backpressure: when 50 items are queued, the producer blocks on `.send()`, waiting for the consumer to catch up. This prevents unbounded memory growth.

**Design Principle**: Always limit concurrency explicitly. Use bounded channels, buffered streams, or semaphores to control resource usage.

## Building Responsive Systems

Responsiveness means the system remains interactive even under load. Async excels here, but only with proper design.

### Cooperative Scheduling

Async Rust uses **cooperative multitasking**—tasks must yield control voluntarily at `.await` points. A task that never awaits starves others:

```rust
// BAD: Never yields, blocks executor
async fn cpu_bound_bad() {
    loop {
        expensive_computation();  // No await - monopolizes thread
    }
}

// GOOD: Yields periodically
async fn cpu_bound_good() {
    loop {
        expensive_computation();
        task::yield_now().await;  // Yields to other tasks
    }
}
```

Production systems should yield at least every ~10ms to maintain responsiveness. For longer computations, use `spawn_blocking`:

```rust
async fn hybrid_approach() {
    let result = task::spawn_blocking(|| {
        // Long CPU work on separate thread pool
        expensive_computation()
    }).await;

    // Back to async context
    store_result(result).await;
}
```

### Timeout and Cancellation

Responsive systems need timeouts to prevent hanging on slow operations:

```rust
use async_std::future::timeout;
use std::time::Duration;

async fn fetch_with_timeout(url: &str) -> Result<String, Error> {
    match timeout(Duration::from_secs(5), fetch_data(url)).await {
        Ok(result) => result,
        Err(_) => Err(Error::Timeout),
    }
}
```

Timeouts combine with cancellation for robust error handling. When a timeout fires, the future is dropped, triggering automatic cancellation—no manual cleanup needed.

### Priority and Fairness

Most executors provide fair scheduling, but you can implement priorities manually:

```rust
async fn prioritize_requests(
    high_priority: Vec<Request>,
    low_priority: Vec<Request>
) {
    // Process all high-priority first
    for req in high_priority {
        handle_request(req).await;
    }

    // Then low-priority
    for req in low_priority {
        handle_request(req).await;
    }
}
```

For more sophisticated priority queues, combine channels with custom scheduling logic.

## The Async Propagation Problem

Async is "viral"—if function A calls async function B, A must be async. This propagates through your entire call stack:

```rust
fn sync_function() {
    // Can't call async_function() directly - no .await in sync context
}

async fn async_function() {
    // Can call both sync and async
    sync_helper();
    async_helper().await;
}
```

This creates an architectural challenge: once you start using async, most of your I/O-related code becomes async. You have three strategies:

### Strategy 1: Go Fully Async

Make your entire I/O layer async. This is the most performant but requires buy-in across the codebase:

```rust
// All async, top to bottom
async fn main_async() {
    let data = load_config().await;  // Async
    let result = process(data).await;  // Async
    save_result(result).await;  // Async
}
```

### Strategy 2: Async Islands

Keep async isolated in specific subsystems, bridging with `block_on`:

```rust
fn main_sync() {
    let data = load_config_sync();  // Sync
    let result = async_std::task::block_on(async {
        process_async(data).await  // Async island
    });
    save_result_sync(result);  // Sync
}
```

This works for gradual adoption but loses some efficiency at the boundaries.

### Strategy 3: Dual APIs

Provide both sync and async versions of key functions:

```rust
// Sync version
pub fn fetch_data_sync(url: &str) -> Result<String, Error> {
    block_on(fetch_data_async(url))
}

// Async version
pub async fn fetch_data_async(url: &str) -> Result<String, Error> {
    // Implementation
}
```

This maximizes flexibility but doubles your API surface.

**Design Principle**: Choose an async strategy early. Retrofitting is expensive.

## Error Handling in Async Systems

Async amplifies error handling complexity because errors can occur in multiple tasks simultaneously:

```rust
async fn parallel_operations() -> Result<(), Error> {
    let results = futures::join!(
        operation_a(),
        operation_b(),
        operation_c(),
    );

    // If any failed, which error do we return?
    match results {
        (Ok(a), Ok(b), Ok(c)) => Ok(()),
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => Err(e),
    }
}
```

The `join!` macro waits for all futures but doesn't stop on first error. For fail-fast behavior:

```rust
async fn fail_fast() -> Result<(), Error> {
    let a = operation_a().await?;  // Stops here if fails
    let b = operation_b().await?;  // Never runs if a failed
    let c = operation_c().await?;
    Ok(())
}
```

Or use `try_join!` for concurrent fail-fast:

```rust
use futures::try_join;

async fn concurrent_fail_fast() -> Result<(), Error> {
    try_join!(
        operation_a(),
        operation_b(),
        operation_c(),
    )?;
    Ok(())
}
```

This runs all three concurrently but returns immediately on first error.

**Design Principle**: Be explicit about error semantics—fail-fast vs. collect-all-errors vs. best-effort.

Designing async systems means architecting for concurrency, backpressure, responsiveness, and error handling from the start. The patterns in this chapter form the foundation for scalable, maintainable async Rust applications.
