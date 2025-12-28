# How to Spawn Blocking Tasks in Async Context

## Problem

You need to run CPU-intensive or blocking operations from async code without freezing your async runtime. Common scenarios:
- Password hashing (CPU-intensive)
- File I/O on filesystems without async support
- Calling synchronous libraries
- Database operations with sync-only drivers

## Solution

Use `spawn_blocking` to run blocking code on a separate thread pool while maintaining an async interface.

## Prerequisites

- Understanding of async/await and futures
- Basic knowledge of thread synchronization (`Arc`, `Mutex`)
- Familiarity with the `Waker` concept

## Step-by-Step Guide

### 1. Understanding the Problem

Blocking code in an async runtime blocks the entire executor thread:

```rust
// BAD: Blocks the async runtime!
async fn bad_hash_password(password: String) -> String {
    // This CPU-intensive work freezes all other tasks
    expensive_hash(&password)  // Takes 100ms+
}
```

### 2. Implement spawn_blocking

Here's a minimal implementation that shows the pattern:

```rust
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// Shared state between the blocking thread and async task
struct Shared<T> {
    value: Option<T>,
    waker: Option<Waker>,
}

pub struct SpawnBlocking<T>(Arc<Mutex<Shared<T>>>);

pub fn spawn_blocking<T, F>(closure: F) -> SpawnBlocking<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let inner = Arc::new(Mutex::new(Shared {
        value: None,
        waker: None,
    }));

    // Spawn a new OS thread to run the blocking work
    std::thread::spawn({
        let inner = inner.clone();
        move || {
            // Run the blocking closure
            let value = closure();

            // Store the result and wake the async task
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

// Implement Future so we can await the result
impl<T: Send> Future for SpawnBlocking<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut guard = self.0.lock().unwrap();

        // If the result is ready, return it
        if let Some(value) = guard.value.take() {
            return Poll::Ready(value);
        }

        // Otherwise, register the waker and return pending
        guard.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}
```

### 3. How It Works

The pattern has four components:

1. **Shared State**: `Arc<Mutex<Shared>>` to coordinate between threads
2. **Thread Spawn**: Creates an OS thread to run the blocking work
3. **Result Storage**: Blocking thread stores result in shared state
4. **Waker Notification**: Blocking thread wakes the async task when done

### 4. Use spawn_blocking for CPU-Intensive Work

```rust
use argonautica::{Hasher, Verifier};

async fn hash_password(password: String) -> Result<String, String> {
    spawn_blocking(move || {
        Hasher::default()
            .with_password(password)
            .with_secret_key("secret")
            .hash()
            .map_err(|e| e.to_string())
    }).await
}

async fn verify_password(hash: String, password: String) -> Result<bool, String> {
    spawn_blocking(move || {
        Verifier::default()
            .with_hash(hash)
            .with_password(password)
            .with_secret_key("secret")
            .verify()
            .map_err(|e| e.to_string())
    }).await
}
```

### 5. Use in Practice

```rust
#[async_std::main]
async fn main() {
    let password = "my_secure_password".to_string();

    // Hash the password (CPU-intensive, runs on thread pool)
    let hash = hash_password(password.clone()).await
        .expect("Failed to hash password");

    println!("Password hashed!");

    // Verify the password (CPU-intensive, runs on thread pool)
    let is_valid = verify_password(hash, password).await
        .expect("Failed to verify password");

    println!("Password valid: {}", is_valid);
}
```

### 6. Running Multiple Blocking Tasks Concurrently

```rust
use async_std::task;

async fn process_users() {
    let passwords = vec![
        "password1".to_string(),
        "password2".to_string(),
        "password3".to_string(),
    ];

    // Spawn all hash operations concurrently
    let futures: Vec<_> = passwords
        .into_iter()
        .map(|pwd| task::spawn(hash_password(pwd)))
        .collect();

    // Wait for all to complete
    for future in futures {
        match future.await {
            Ok(hash) => println!("Hash: {}", hash),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
```

## Using Runtime-Provided spawn_blocking

In production, use the runtime's built-in `spawn_blocking`:

### With async-std:

```rust
use async_std::task;

async fn example() {
    let result = task::spawn_blocking(|| {
        // Blocking or CPU-intensive work
        std::thread::sleep(std::time::Duration::from_secs(1));
        42
    }).await;

    println!("Result: {}", result);
}
```

### With tokio:

```rust
use tokio::task;

async fn example() {
    let result = task::spawn_blocking(|| {
        // Blocking or CPU-intensive work
        std::thread::sleep(std::time::Duration::from_secs(1));
        42
    }).await.unwrap();  // Note: tokio returns JoinHandle

    println!("Result: {}", result);
}
```

## Thread Pools for CPU-Bound Work

Runtime implementations use thread pools to avoid creating too many threads:

```rust
// async-std uses a dedicated blocking thread pool
// The pool size adjusts based on workload

async fn many_blocking_tasks() {
    let mut handles = vec![];

    // Spawns 100 tasks, but uses a limited thread pool
    for i in 0..100 {
        handles.push(task::spawn_blocking(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            i * 2
        }));
    }

    for handle in handles {
        let result = handle.await;
        println!("{}", result);
    }
}
```

## Bridging Sync and Async Code

### Pattern 1: Async Wrapper for Sync Library

```rust
// Sync library
fn sync_database_query(query: &str) -> Vec<String> {
    // Blocking database call
    std::thread::sleep(std::time::Duration::from_millis(100));
    vec!["result1".to_string(), "result2".to_string()]
}

// Async wrapper
async fn async_database_query(query: String) -> Vec<String> {
    task::spawn_blocking(move || {
        sync_database_query(&query)
    }).await
}
```

### Pattern 2: Blocking File I/O

```rust
use std::fs;

async fn read_config_file(path: String) -> std::io::Result<String> {
    task::spawn_blocking(move || {
        fs::read_to_string(path)
    }).await
}

async fn write_log_file(path: String, content: String) -> std::io::Result<()> {
    task::spawn_blocking(move || {
        fs::write(path, content)
    }).await
}
```

### Pattern 3: Mixed Async and Blocking Operations

```rust
async fn process_request(user_id: i32) -> Result<String, String> {
    // Async operation: fetch from API
    let user_data = fetch_user_from_api(user_id).await?;

    // Blocking operation: hash password
    let hashed = task::spawn_blocking(move || {
        expensive_hash(&user_data.password)
    }).await;

    // Async operation: save to database
    save_to_database(user_id, &hashed).await?;

    Ok(hashed)
}
```

## Comparison to Python's run_in_executor

Python's asyncio provides a similar pattern:

**Python:**
```python
import asyncio
from concurrent.futures import ThreadPoolExecutor

async def main():
    loop = asyncio.get_event_loop()

    # Run blocking code in thread pool
    result = await loop.run_in_executor(
        ThreadPoolExecutor(),
        blocking_function,
        arg1,
        arg2
    )

    return result

def blocking_function(arg1, arg2):
    # Blocking work here
    return arg1 + arg2
```

**Rust:**
```rust
use async_std::task;

async fn main() {
    let arg1 = 1;
    let arg2 = 2;

    // Run blocking code in thread pool
    let result = task::spawn_blocking(move || {
        blocking_function(arg1, arg2)
    }).await;

    result
}

fn blocking_function(arg1: i32, arg2: i32) -> i32 {
    // Blocking work here
    arg1 + arg2
}
```

**Key Differences:**
- Rust's `spawn_blocking` is simpler - just a closure
- Python requires explicit executor management
- Rust's version is type-safe and returns the actual type
- Both use thread pools to manage concurrency

## Common Pitfalls

### Don't Use for Async I/O

```rust
// BAD: Use async I/O instead
task::spawn_blocking(|| {
    std::fs::read_to_string("file.txt")
});

// GOOD: Use async file I/O
async_std::fs::read_to_string("file.txt").await;
```

### Watch Out for Thread Exhaustion

```rust
// BAD: Can exhaust thread pool
for i in 0..10000 {
    task::spawn_blocking(move || {
        std::thread::sleep(Duration::from_secs(60));
    });
}

// GOOD: Use a semaphore to limit concurrency
let semaphore = Arc::new(Semaphore::new(10));
for i in 0..10000 {
    let permit = semaphore.clone().acquire_owned().await;
    task::spawn_blocking(move || {
        let _permit = permit;  // Hold permit during work
        std::thread::sleep(Duration::from_secs(60));
    });
}
```

### Move Ownership Correctly

```rust
// BAD: Tries to borrow across thread boundary
let data = vec![1, 2, 3];
task::spawn_blocking(|| {
    println!("{:?}", data);  // Error: data not moved
});

// GOOD: Move ownership
let data = vec![1, 2, 3];
task::spawn_blocking(move || {  // Note: move keyword
    println!("{:?}", data);
});
```

## Performance Considerations

- **Thread Overhead**: Spawning a thread has overhead (~100μs). Use for work >1ms
- **Context Switching**: Too many threads causes context switching overhead
- **Thread Pool Size**: Most runtimes use CPU count * 2 for blocking pool
- **Memory**: Each thread uses ~2MB of stack space

## Summary

- `spawn_blocking` runs blocking code on separate threads
- Prevents blocking the async runtime executor
- Essential for CPU-intensive work and sync library integration
- Use runtime-provided implementations (`async_std::task::spawn_blocking`, `tokio::task::spawn_blocking`)
- Similar to Python's `run_in_executor` but simpler and type-safe
- Be mindful of thread pool exhaustion with many concurrent blocking tasks

## Related

- [How to Block on Futures](05-block-on-futures.md) - Running async code from sync context
- [How to Make Concurrent HTTP Requests](08-concurrent-http-requests.md) - Async I/O patterns
