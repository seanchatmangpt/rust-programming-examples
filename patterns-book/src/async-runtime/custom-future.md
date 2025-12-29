# Custom Future

## Context

You're building async functionality in Rust. While most async code uses `async fn` or `async` blocks, sometimes you need fine-grained control over the polling mechanism, state transitions, or need to integrate with non-async code. You want to bridge blocking operations with the async ecosystem.

You have a clear state machine in mind—perhaps "waiting for data," then "processing," then "ready"—and you want to express that directly rather than through `async` syntax.

## Problem

**How do you create custom asynchronous primitives that integrate seamlessly with Rust's async ecosystem without using `async` syntax?**

The `Future` trait is the foundation of Rust's async system, but implementing it manually requires understanding:
- The polling mechanism and what `Poll::Ready` vs `Poll::Pending` mean
- How to store and use wakers to notify the executor
- Proper state management across poll invocations
- Thread safety when state is shared across threads

Without this knowledge, you might:
- Return `Poll::Pending` without storing the waker, causing the future to never wake
- Create race conditions in multi-threaded scenarios
- Leak resources by not properly managing state transitions
- Make executors poll wastefully by waking too frequently

## Forces

- **Executor Integration**: Your custom future must work with any standard async executor (tokio, async-std, futures)
- **Waker Protocol**: You must store and use wakers correctly or the future will never complete
- **State Transitions**: The future must transition cleanly from pending to ready, consuming the result exactly once
- **Thread Safety**: When bridging threads (common with blocking operations), you need proper synchronization
- **Zero-Cost Abstraction**: The implementation should be as efficient as hand-written state machines
- **Pinning Requirements**: Futures are often self-referential, requiring proper Pin handling
- **Type Safety**: The type system should prevent misuse of the future

## Solution

**Implement the `Future` trait manually by creating a state machine that stores both the result and the waker.**

The key insight from `spawn-blocking` is to use `Arc<Mutex<Shared<T>>>` to share state between the background thread producing the result and the async code polling for it:

```rust
use std::sync::{Arc, Mutex};
use std::task::Waker;

pub struct SpawnBlocking<T>(Arc<Mutex<Shared<T>>>);

struct Shared<T> {
    value: Option<T>,  // The result, once ready
    waker: Option<Waker>,  // How to wake the executor
}
```

**State Management**: The shared state has two fields:
- `value`: Starts as `None`, becomes `Some(result)` when ready
- `waker`: Stores the waker from the most recent poll

**The Future Implementation**:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

impl<T: Send> Future for SpawnBlocking<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut guard = self.0.lock().unwrap();

        // Check if value is ready
        if let Some(value) = guard.value.take() {
            return Poll::Ready(value);
        }

        // Not ready yet - store the waker for later
        guard.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}
```

**The Polling Protocol**:
1. **Lock the shared state** to check the value atomically
2. **If value is present**: Take it (consuming it) and return `Poll::Ready(value)`
3. **If value is absent**: Store the current waker and return `Poll::Pending`

**Critical Pattern**: The value is `take()`n (not just cloned), ensuring it's consumed exactly once. The waker is stored so the producer can wake the executor when the value becomes ready.

**Producer Side** (in `spawn_blocking` function):

```rust
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

            // Store value and get waker in one atomic operation
            let maybe_waker = {
                let mut guard = inner.lock().unwrap();
                guard.value = Some(value);
                guard.waker.take()
            };

            // Wake the executor if it's waiting
            if let Some(waker) = maybe_waker {
                waker.wake();
            }
        }
    });

    SpawnBlocking(inner)
}
```

**Producer Pattern**:
1. Run the blocking operation (outside the lock)
2. Store the result and retrieve the waker in a single critical section
3. Call `waker.wake()` to notify the executor (outside the lock to avoid deadlock)

**Complete Example** from the tests:

```rust
// CPU-intensive password verification
async fn verify_password(password: &str, hash: &str, key: &str)
                        -> Result<bool, argonautica::Error>
{
    let password = password.to_string();
    let hash = hash.to_string();
    let key = key.to_string();

    spawn_blocking(move || {
        argonautica::Verifier::default()
            .with_hash(hash)
            .with_password(password)
            .with_secret_key(key)
            .verify()
    }).await
}
```

This pattern allows blocking operations to integrate seamlessly with async code.

## Resulting Context

**Benefits**:

1. **Perfect Executor Integration**: Works with any Rust async executor without modification
2. **Efficient Waking**: The executor is woken exactly when needed, not before
3. **One-Shot Consumption**: The value is consumed exactly once through `take()`
4. **Thread-Safe**: `Arc<Mutex<>>` provides safe sharing between producer and consumer threads
5. **Type-Safe**: The type system prevents polling after the value is consumed
6. **Composable**: Custom futures compose with `async` functions using `.await`

**New Challenges**:

1. **Complexity**: Manual Future implementation is more complex than `async` blocks
2. **Lock Contention**: Every poll acquires the mutex, which could be a bottleneck under high contention
3. **Error Handling**: Must manually propagate errors through the type system
4. **No Async Syntax**: Cannot use `.await` within the Future implementation itself
5. **Testing**: Requires understanding of executors to test properly

**When This Pattern Applies**:
- Bridging blocking operations with async code
- Implementing custom async primitives (channels, timers, I/O)
- Creating futures with complex state machines
- Building runtime components that need fine control over waking

**When to Use `async` Instead**:
- Sequential async operations that await other futures
- Business logic that doesn't need custom waking behavior
- Code that doesn't maintain complex internal state

## Related Patterns

- **Simple Executor**: Consumes futures created by this pattern
- **Waker Mechanism**: The protocol this pattern uses to notify executors
- **Thread Pool for Blocking**: Often combined with this pattern for efficiency
- **Polling Loop**: The executor side of this protocol
- **State Machine Pattern**: The underlying structure of futures

## Known Uses

1. **tokio::task::spawn_blocking**: Production implementation of this pattern with a thread pool
2. **async-std::task::spawn_blocking**: Similar implementation in async-std
3. **futures::channel**: Custom futures for channels and synchronization primitives
4. **tokio::time::Sleep**: Timer future that wakes on timer expiry
5. **tokio::net::TcpStream**: I/O futures that wake on readiness events

**From the spawn-blocking tests**:

```rust
// Parallel blocking operations
let futures: Vec<_> = (0..100)
    .map(|i| (i, spawn_blocking(move || i)))
    .collect();

for (i, f) in futures {
    assert_eq!(f.await, i);
}
```

This pattern is fundamental to Rust's async ecosystem, enabling safe integration of blocking and async code.
