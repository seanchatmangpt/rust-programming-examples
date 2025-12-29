# Spawn Local

## Context

You need to execute async tasks concurrently, but your tasks contain types that are not `Send`—such as `Rc`, `RefCell`, or raw pointers. The standard `spawn` function requires tasks to be `Send + 'static` so they can safely move between threads, but your task cannot meet this requirement.

Alternatively, you want deterministic single-threaded execution for testing or to avoid the overhead of thread synchronization.

You understand **CONCURRENT FUTURES** and want to spawn tasks, but the `Send` bound prevents you from doing so.

## Problem

**How do you spawn concurrent async tasks that contain non-`Send` types, or ensure tasks run on a single thread?**

The standard async runtime `spawn` function has a signature like:

```rust
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send,
```

This `Send` bound allows the runtime to move the task between threads for load balancing. But many useful types are not `Send`:

- `Rc<T>`: Reference-counted pointer (not thread-safe)
- `RefCell<T>`: Runtime borrow checking (not thread-safe)
- `*const T`, `*mut T`: Raw pointers (no thread safety guarantees)
- Types that contain the above

Attempting to spawn a task with these types fails to compile:

```rust
let data = Rc::new(42);
task::spawn(async move {
    println!("{}", data);  // ERROR: Rc is not Send
});
```

You need a way to spawn tasks that guarantees they run on the current thread only.

## Forces

- **Thread Safety**: `Send` types are safe to move between threads. Non-`Send` types like `Rc` would cause data races if accessed from multiple threads.
- **Concurrency vs. Parallelism**: Single-threaded concurrency can still efficiently handle I/O-bound work through task interleaving, without thread parallelism.
- **API Compatibility**: Some libraries and FFI code provide non-`Send` types that you cannot change.
- **Performance**: Thread synchronization (atomics, locks) has overhead. Single-threaded execution avoids this cost.
- **Testing**: Deterministic single-threaded execution makes tests more reproducible.
- **Ergonomics**: Removing the `Send` bound allows using reference counting and mutable shared state without `Arc` and `Mutex`.

## Solution

**Use `spawn_local` to spawn tasks that run only on the current thread, removing the `Send` requirement.**

The `spawn_local` function spawns a task that is guaranteed to execute on the same thread where it was spawned. This allows the task to safely contain non-`Send` types. The task runs concurrently with other tasks on that thread through cooperative multitasking.

### Structure

```rust
async fn concurrent_local_tasks() {
    let mut handles = vec![];

    for item in items {
        // spawn_local doesn't require Send
        let handle = task::spawn_local(async move {
            // Can use non-Send types here
            let rc_data = Rc::new(item);
            process(rc_data).await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await;
    }
}
```

### Real Example from many-requests

```rust
use async_std::io::prelude::*;
use async_std::net;

async fn cheapo_request(host: &str, port: u16, path: &str)
                            -> std::io::Result<String>
{
    let mut socket = net::TcpStream::connect((host, port)).await?;

    let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
    socket.write_all(request.as_bytes()).await?;
    socket.shutdown(net::Shutdown::Write)?;

    let mut response = String::new();
    socket.read_to_string(&mut response).await?;

    Ok(response)
}

async fn many_requests(requests: Vec<(String, u16, String)>)
                           -> Vec<std::io::Result<String>>
{
    use async_std::task;

    let mut handles = vec![];

    // spawn_local: tasks run on current thread only
    for (host, port, path) in requests {
        handles.push(task::spawn_local(async move {
            cheapo_request(&host, port, &path).await
        }));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}
```

### Running spawn_local Tasks

`spawn_local` tasks must run within a local task scope:

```rust
fn main() {
    let requests = vec![
        ("example.com".to_string(),      80, "/".to_string()),
        ("www.red-bean.com".to_string(), 80, "/".to_string()),
        ("en.wikipedia.org".to_string(), 80, "/".to_string()),
    ];

    // block_on provides the local task scope
    let results = async_std::task::block_on(many_requests(requests));

    for result in results {
        match result {
            Ok(response) => println!("{}", response),
            Err(err) => eprintln!("error: {}", err),
        }
    }
}
```

### Key Mechanisms

1. **No `Send` Bound**:
   ```rust
   pub fn spawn_local<F>(future: F) -> JoinHandle<F::Output>
   where
       F: Future + 'static,
       // Note: No Send bound!
   ```

2. **Thread Affinity**:
   - Task is pinned to the current thread
   - Runtime scheduler won't move it to another thread
   - Safe to use `Rc`, `RefCell`, and other non-`Send` types

3. **Cooperative Multitasking**:
   - Tasks yield at `await` points
   - Runtime interleaves execution on the single thread
   - Still achieves concurrency for I/O-bound work

4. **Local Task Scope**:
   - Must be called from within an async context
   - Typically used inside an async function
   - `block_on` provides the initial local scope

### spawn vs. spawn_local Comparison

| Feature | `spawn` | `spawn_local` |
|---------|---------|---------------|
| **Requires `Send`** | Yes | No |
| **Thread safety** | Multi-threaded | Single-threaded |
| **Can use `Rc`** | No | Yes |
| **Can use `RefCell`** | No | Yes |
| **True parallelism** | Yes (multiple cores) | No (one core) |
| **Concurrency** | Yes | Yes |
| **Sync overhead** | Higher (atomics) | Lower (no sync) |
| **Use case** | CPU-bound + I/O | I/O-bound only |

### Example with Non-Send Types

```rust
use std::rc::Rc;
use async_std::task;

async fn use_non_send_types() {
    let mut handles = vec![];

    for i in 0..10 {
        handles.push(task::spawn_local(async move {
            // Rc is not Send, but spawn_local allows it
            let data = Rc::new(vec![i; 100]);
            process_data(data).await
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
```

## Resulting Context

Your concurrent tasks now:

- **Can use non-`Send` types**: `Rc`, `RefCell`, raw pointers work fine
- **Run concurrently**: Multiple tasks interleave execution for efficient I/O handling
- **Execute deterministically**: Single-threaded execution is more predictable for testing
- **Avoid sync overhead**: No atomic operations or mutex locking needed
- **Remain on one thread**: Guaranteed thread affinity for FFI or thread-local state

However, you've introduced new considerations:

- **No parallelism**: Tasks won't utilize multiple CPU cores, limiting throughput for CPU-bound work
- **Thread blocking**: If any task blocks the thread (non-async I/O, long computation), all tasks stall
- **Scope requirements**: Must be called from within a local task context (typically an async function)
- **Runtime dependency**: Requires runtime support for `spawn_local` (async-std, tokio with `LocalSet`)

When you have `Send` types and want true parallelism across CPU cores, use the regular `spawn` instead.

When you need to make concurrent HTTP requests with `Send` types, **ASYNC HTTP CLIENT LIBRARY** examples use regular `spawn`.

## Related Patterns

- **CONCURRENT FUTURES**: General pattern for spawning tasks; `spawn_local` is a specific variant
- **ASYNC/AWAIT BASICS**: Foundation for writing async functions
- **ASYNC HTTP REQUEST**: Common use case for `spawn_local` when working with local state

## Known Uses

- **Single-threaded runtimes**: async-std with single-threaded executor, tokio's `LocalSet`
- **FFI integration**: When calling C libraries that require thread affinity
- **WebAssembly**: WASM is single-threaded; all async tasks use `spawn_local` semantics
- **Testing**: Tests often use `spawn_local` for deterministic, reproducible execution
- **UI frameworks**: Event loop must run on main thread (Druid, Iced use similar patterns)
- **Reference-counted state**: When you want to share data across tasks without `Arc`

Real-world libraries:

- **async-std**: Provides `task::spawn_local` for single-threaded task spawning
- **tokio**: Provides `LocalSet` for spawning `!Send` futures
- **actix**: Actor framework heavily uses thread-local execution
- **wasm-bindgen-futures**: Uses spawn_local semantics for browser async code

Common scenarios:

```rust
// Sharing Rc across tasks
let data = Rc::new(expensive_data);
for i in 0..10 {
    let data = data.clone();  // Cheap Rc clone
    task::spawn_local(async move {
        process(&data, i).await
    });
}

// RefCell for shared mutable state
let counter = Rc::new(RefCell::new(0));
for _ in 0..10 {
    let counter = counter.clone();
    task::spawn_local(async move {
        *counter.borrow_mut() += 1;
    });
}
```

The choice between `spawn` and `spawn_local` depends on:
- Whether your types are `Send` (use `spawn` if possible)
- Whether you need true parallelism (use `spawn` for CPU-bound work)
- Whether you're integrating with thread-local APIs or FFI (use `spawn_local`)
- Whether you want deterministic single-threaded execution (use `spawn_local` for testing)

For pure I/O-bound work with `Send` types, prefer regular `spawn` to allow the runtime more scheduling freedom.
