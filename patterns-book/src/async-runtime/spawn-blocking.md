# Spawn Blocking

## Context

You're writing async Rust code that needs to perform operations that block the thread—CPU-intensive computations, synchronous I/O, or calls to blocking libraries. Running blocking code directly in an async function would stall the entire executor thread, preventing other async tasks from making progress.

You want to use a blocking library (like a CPU-intensive password hasher, database connection, or file parser) from within async code. The library doesn't have async versions of its functions, and rewriting it isn't an option.

## Problem

**How do you safely execute blocking operations within an async context without stalling the executor and starving other tasks?**

The fundamental conflict:
- **Async executors** expect tasks to return quickly from `.await` points
- **Blocking operations** hold the thread until completion
- **Running blocking code** in async tasks can deadlock the entire runtime
- **Converting to async** may be impossible (third-party libraries, OS calls, CPU-bound work)

Naive approaches fail:
```rust
// WRONG: Blocks the executor thread
async fn bad_password_check(password: &str) -> bool {
    expensive_hash_function(password)  // Blocks entire async runtime!
}
```

You need a way to:
- Move blocking work off the async executor threads
- Return control to the executor immediately
- Get notified when the blocking operation completes
- Integrate the result seamlessly with async code

## Forces

- **Executor Freedom**: Async tasks must not block executor threads
- **Seamless Integration**: Blocking operations should work with `.await` syntax
- **Type Safety**: The solution must preserve Rust's Send + 'static guarantees
- **Thread Overhead**: Creating threads is expensive—can't spawn on every call
- **Cancellation**: What happens if the future is dropped before completion?
- **Result Retrieval**: The blocking operation's result must be delivered back to async context
- **Error Propagation**: Errors from blocking code should propagate naturally

## Solution

**Spawn the blocking operation on a dedicated thread, returning a Future that completes when the thread finishes.**

The pattern from `spawn-blocking` demonstrates the complete solution:

```rust
use std::sync::{Arc, Mutex};
use std::task::Waker;

pub struct SpawnBlocking<T>(Arc<Mutex<Shared<T>>>);

struct Shared<T> {
    value: Option<T>,
    waker: Option<Waker>,
}

pub fn spawn_blocking<T, F>(closure: F) -> SpawnBlocking<T>
where F: FnOnce() -> T,
      F: Send + 'static,
      T: Send + 'static,
{
    let inner = Arc::new(Mutex::new(Shared {
        value: None,
        waker: None,
    }));

    std::thread::spawn({
        let inner = inner.clone();
        move || {
            let value = closure();

            let maybe_waker = {
                let mut guard = inner.lock().unwrap();
                guard.value = Some(value);
                guard.waker.take()
            };

            if let Some(waker) = maybe_waker {
                waker.wake();
            }
        }
    });

    SpawnBlocking(inner)
}
```

**The Architecture**:

1. **Shared State**: `Arc<Mutex<Shared<T>>>` coordinates between the spawned thread and the async task
2. **Thread Spawn**: The blocking operation runs in a new thread, not on the executor
3. **Future Return**: Returns immediately with a `SpawnBlocking<T>` future
4. **Value Delivery**: The thread stores the result and wakes the executor when done
5. **Async Integration**: The future can be `.await`ed like any other async operation

**The Future Implementation**:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

impl<T: Send> Future for SpawnBlocking<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut guard = self.0.lock().unwrap();

        if let Some(value) = guard.value.take() {
            return Poll::Ready(value);
        }

        guard.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}
```

**Usage Example** (from the tests):

```rust
async fn verify_password(password: &str, hash: &str, key: &str)
                        -> Result<bool, argonautica::Error>
{
    // Clone strings to move into the closure
    let password = password.to_string();
    let hash = hash.to_string();
    let key = key.to_string();

    spawn_blocking(move || {
        // This blocking operation runs on a dedicated thread
        argonautica::Verifier::default()
            .with_hash(hash)
            .with_password(password)
            .with_secret_key(key)
            .verify()
    }).await  // Seamlessly awaitable
}
```

**The Magic**: From the caller's perspective, this looks like any async operation. The blocking work happens on a separate thread, leaving the executor free to run other tasks.

**Parallel Blocking Operations**:

```rust
// Spawn 100 blocking operations in parallel
let futures: Vec<_> = (0..100)
    .map(|i| (i, spawn_blocking(move || {
        // Each runs on its own thread
        expensive_computation(i)
    })))
    .collect();

// Await them all
for (i, f) in futures {
    assert_eq!(f.await, i);
}
```

**Type Safety**:
- `F: Send + 'static`: The closure must be sendable to another thread
- `T: Send + 'static`: The result must be sendable back
- The compiler prevents data races and use-after-free errors

**Critical Sections**:
```rust
// Atomic operation: store value and get waker
let maybe_waker = {
    let mut guard = inner.lock().unwrap();
    guard.value = Some(value);
    guard.waker.take()
};
// Lock released before calling wake()
if let Some(waker) = maybe_waker {
    waker.wake();  // Safe to call outside the lock
}
```

## Resulting Context

**Benefits**:

1. **Non-Blocking**: Executor threads remain free to run other tasks
2. **Seamless Integration**: Works with `.await` and async/await syntax
3. **Type-Safe**: Compiler enforces thread safety and lifetime correctness
4. **Simple API**: One-line conversion from blocking to async
5. **Composable**: Multiple blocking operations can run in parallel
6. **Error Propagation**: Errors return through the Result type naturally

**Trade-offs and Limitations**:

1. **Thread Overhead**: Each call spawns a new thread (expensive)
   - **Solution**: Use a thread pool (see Thread Pool for Blocking pattern)

2. **No Cancellation**: Dropping the future doesn't stop the blocking operation
   ```rust
   {
       let _f = spawn_blocking(|| {
           expensive_computation()  // Continues even if _f is dropped
       });
   }  // Future dropped, but thread keeps running
   ```

3. **Resource Limits**: Spawning thousands of threads can exhaust system resources
   ```rust
   // Dangerous: Could spawn 10,000 threads
   for i in 0..10_000 {
       spawn_blocking(move || expensive_work(i));
   }
   ```

4. **Memory Usage**: Each thread has its own stack (typically 2MB on Linux)

5. **No Backpressure**: Easy to spawn more work than the system can handle

**Production Hardening** (not in the example):
```rust
// Real implementations use thread pools
static THREAD_POOL: Lazy<ThreadPool> = Lazy::new(|| {
    ThreadPool::new(num_cpus::get())
});

pub fn spawn_blocking<T, F>(closure: F) -> SpawnBlocking<T>
where F: FnOnce() -> T + Send + 'static,
      T: Send + 'static,
{
    // Submit to pool instead of spawning thread
    THREAD_POOL.execute(closure)
}
```

**When to Use This Pattern**:
- ✅ CPU-intensive computations (password hashing, encryption, parsing)
- ✅ Synchronous I/O (old file APIs, databases without async drivers)
- ✅ Blocking library calls (OS APIs, C FFI, legacy code)
- ✅ Operations that can't be made async (physics simulations, rendering)

**When NOT to Use This Pattern**:
- ❌ Async-compatible operations (use native async instead)
- ❌ High-frequency operations (thread overhead too high)
- ❌ Operations needing cancellation (threads can't be stopped)
- ❌ Tight loops calling blocking code (pool gets exhausted)

## Related Patterns

- **Custom Future**: The underlying mechanism this pattern uses
- **Thread Pool for Blocking**: Production-grade version with pooling
- **Waker Mechanism**: How the thread notifies the executor
- **Simple Executor**: What consumes the spawned futures
- **Async Wrapper Pattern**: General technique for wrapping sync code

## Known Uses

1. **tokio::task::spawn_blocking**: Production implementation with thread pool
   ```rust
   use tokio::task;

   let result = task::spawn_blocking(|| {
       // Blocking work
   }).await?;
   ```

2. **async-std::task::spawn_blocking**: Similar implementation
   ```rust
   use async_std::task;

   let result = task::spawn_blocking(|| {
       // Blocking work
   }).await;
   ```

3. **actix-web::web::block**: Web framework integration
   ```rust
   use actix_web::web;

   let result = web::block(|| {
       // Database query
   }).await?;
   ```

4. **rayon integration**: CPU-intensive parallel work
   ```rust
   spawn_blocking(|| {
       data.par_iter().map(|x| expensive(x)).collect()
   }).await
   ```

**Real-World Example** (from the tests):

```rust
// Argonautica password hashing is intentionally CPU-intensive
static PASSWORD: &str = "P@ssw0rd";
static HASH: &str = "$argon2id$v=19$m=4096,t=192,p=4$...";
static SECRET_KEY: &str = "secret key...";

async_std::task::block_on(async {
    assert!(verify_password(PASSWORD, HASH, SECRET_KEY).await.unwrap());
});
```

**Performance Characteristics**:
- **Latency**: Thread spawn overhead (~10-100μs) + blocking operation time
- **Throughput**: Limited by thread pool size (if using pool) or system thread limit
- **Memory**: 2MB per thread (typical stack size) + shared state overhead
- **Scalability**: Limited to hundreds of concurrent operations (thousands with pooling)

This pattern is essential for integrating blocking code in async Rust, enabling gradual migration from sync to async and integration with existing ecosystems.
