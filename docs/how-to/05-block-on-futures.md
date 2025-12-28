# How to Block the Current Thread on a Future

## Problem

You need to run an async function from synchronous code and wait for its result. This is common when:
- Starting an async runtime from `main()`
- Running async code in tests
- Bridging sync and async boundaries

## Solution

Use a `block_on` function to execute a future on the current thread and wait for its completion.

## Prerequisites

- Basic understanding of Rust futures and the `Future` trait
- Familiarity with the `Poll` type and async/await syntax

## Step-by-Step Guide

### 1. Add Required Dependencies

```toml
[dependencies]
waker-fn = "1.1"
futures-lite = "1.11"
crossbeam = "0.8"
```

### 2. Implement the Basic block_on Executor

Here's a minimal executor that blocks on a future:

```rust
use waker_fn::waker_fn;
use futures_lite::pin;
use crossbeam::sync::Parker;
use std::future::Future;
use std::task::{Context, Poll};

pub fn block_on<F: Future>(future: F) -> F::Output {
    // Create a parker/unparker pair for thread synchronization
    let parker = Parker::new();
    let unparker = parker.unparker().clone();

    // Create a waker that unparks our thread when the future makes progress
    let waker = waker_fn(move || unparker.unpark());
    let mut context = Context::from_waker(&waker);

    // Pin the future to prevent it from moving in memory
    pin!(future);

    // Poll the future repeatedly until it completes
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => parker.park(), // Block until waker is called
        }
    }
}
```

### 3. Understanding How It Works

The executor works in four steps:

1. **Thread Parking**: Uses `Parker` to block the current thread efficiently
2. **Waker Creation**: Creates a waker that will unpark the thread when called
3. **Polling Loop**: Repeatedly polls the future with the context
4. **Suspension**: When pending, parks the thread until the waker unparks it

### 4. Use block_on to Run Async Code

```rust
fn main() {
    // Run a simple async computation
    let result = block_on(async {
        42
    });
    println!("Result: {}", result); // Output: Result: 42

    // Run async code that actually awaits
    let result = block_on(async {
        async_std::task::sleep(std::time::Duration::from_millis(100)).await;
        "Done!"
    });
    println!("{}", result); // Output: Done!
}
```

### 5. Testing Async Code

The `block_on` pattern is especially useful in tests:

```rust
#[test]
fn test_async_function() {
    use async_std::task::sleep;
    use std::time::Duration;

    let result = block_on(async {
        sleep(Duration::from_millis(100)).await;
        2 + 2
    });

    assert_eq!(result, 4);
}
```

### 6. Racing Futures

Combine `block_on` with future combinators:

```rust
use async_std::task::{spawn, sleep};
use futures_lite::FutureExt;
use std::time::Duration;

fn main() {
    let result = block_on({
        let one_sec = async {
            sleep(Duration::from_secs(1)).await;
            "Slow"
        };

        let half_sec = async {
            sleep(Duration::from_millis(500)).await;
            "Fast"
        };

        // Race returns the result of the first future to complete
        spawn(one_sec.race(half_sec))
    });

    println!("Winner: {}", result); // Output: Winner: Fast
}
```

## When to Use block_on vs Async

**Use block_on when:**
- You're in synchronous code (like `main` or tests)
- You need to bridge sync and async boundaries
- You're implementing a simple single-threaded executor

**Use async/await when:**
- You're already in an async context
- You want concurrent execution of multiple tasks
- You need better performance with many I/O operations

## Comparison to Python's asyncio.run()

Rust's `block_on` is similar to Python's `asyncio.run()`:

**Python:**
```python
import asyncio

async def async_task():
    await asyncio.sleep(1)
    return 42

result = asyncio.run(async_task())  # Blocks until complete
```

**Rust:**
```rust
async fn async_task() -> i32 {
    async_std::task::sleep(Duration::from_secs(1)).await;
    42
}

let result = block_on(async_task());  // Blocks until complete
```

**Key Differences:**
- Rust's `block_on` is simpler - just a poll loop
- Python's `asyncio.run()` creates a new event loop each time
- Rust gives you control over the executor implementation
- Both block the calling thread until the future/coroutine completes

## Common Pitfalls

### Don't Call block_on Inside Async Code

```rust
// BAD: Will deadlock or panic
async fn bad_example() {
    block_on(async {  // Don't block inside async!
        println!("This is wrong");
    });
}

// GOOD: Just await
async fn good_example() {
    async {
        println!("This is correct");
    }.await;
}
```

### Don't Use for CPU-Bound Work

```rust
// BAD: Blocks the executor thread
block_on(async {
    // CPU-intensive work
    for _ in 0..1_000_000 {
        // heavy computation
    }
});

// GOOD: Use spawn_blocking (see spawn-blocking guide)
block_on(async {
    spawn_blocking(|| {
        for _ in 0..1_000_000 {
            // heavy computation
        }
    }).await
});
```

## Production Use

For production code, use established runtimes instead of a custom `block_on`:

**With tokio:**
```rust
#[tokio::main]
async fn main() {
    // Your async code here
}

// Equivalent to:
fn main() {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async_main());
}
```

**With async-std:**
```rust
fn main() {
    async_std::task::block_on(async {
        // Your async code here
    });
}
```

## Summary

- `block_on` executes a future synchronously on the current thread
- It uses a parker/unparker pattern to efficiently wait for the future
- Essential for bridging sync and async code
- Similar to Python's `asyncio.run()` but more explicit
- Use runtime-provided versions (`async_std::task::block_on`, `tokio::runtime::Runtime::block_on`) in production

## Related

- [How to Spawn Blocking Tasks](06-spawn-blocking-tasks.md) - Running blocking code in async context
- [How to Build an Async Echo Server](07-build-async-echo-server.md) - Using block_on to start a server
