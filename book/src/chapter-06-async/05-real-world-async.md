# Real-World Async Architectures

Theory meets practice when building production async systems. This section examines real code from the repository's async projects, explores ecosystem choices, and distills patterns that separate robust systems from brittle ones. These aren't academic examples—they're the building blocks of systems handling millions of requests.

## Case Study: HTTP Request Concurrency

The `many-requests` project demonstrates fundamental patterns for concurrent I/O operations:

```rust
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

### Architectural Insights

**Spawning strategy**: Each request spawns an independent task with `spawn_local`, creating concurrency without threads. The `spawn_local` variant doesn't require `Send`, allowing simpler code when tasks run on a single-threaded executor.

**Error handling**: Individual request failures don't crash the entire batch. Each `Result` is collected independently, enabling partial success patterns.

**Resource management**: The socket cleanup happens automatically through RAII. When the future drops (error or completion), the socket's `Drop` implementation closes the connection.

### Production Improvements

This example shows the pattern but needs hardening for production:

```rust
use futures::stream::{self, StreamExt};
use std::time::Duration;

async fn many_requests_production(
    requests: Vec<(String, u16, String)>,
    concurrency: usize,
    timeout: Duration,
) -> Vec<std::io::Result<String>> {
    stream::iter(requests)
        .map(|(host, port, path)| async move {
            // Add timeout to prevent hanging
            async_std::future::timeout(timeout, cheapo_request(&host, port, &path)).await
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "request timeout"))?
        })
        .buffer_unordered(concurrency)  // Limit concurrent requests
        .collect()
        .await
}
```

Changes for production:
1. **Bounded concurrency**: `buffer_unordered(concurrency)` prevents resource exhaustion
2. **Timeouts**: Each request has a deadline
3. **Streaming results**: Memory-efficient for large request sets
4. **Error context**: Timeout errors are distinguishable

## Case Study: Simple Executor Implementation

The `block-on` project implements a minimal but complete async executor, revealing how async actually works:

```rust
use waker_fn::waker_fn;
use futures_lite::pin;
use crossbeam::sync::Parker;
use std::future::Future;
use std::task::{Context, Poll};

pub fn block_on<F: Future>(future: F) -> F::Output {
    let parker = Parker::new();
    let unparker = parker.unparker().clone();
    let waker = waker_fn(move || unparker.unpark());
    let mut context = Context::from_waker(&waker);

    pin!(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => parker.park(),
        }
    }
}
```

### How It Works

**Parker/Unparker pair**: Crossbeam's `Parker` provides efficient thread parking. The thread blocks (`park()`) until another thread calls `unpark()`.

**Waker mechanism**: The `waker_fn` creates a waker that calls `unparker.unpark()` when invoked. When the future makes progress (I/O completes, timer fires), it calls this waker, unparking the thread.

**Polling loop**: The executor repeatedly polls the future. If pending, it parks the thread. When woken, it polls again. This continues until `Poll::Ready`.

**Pinning**: The `pin!` macro ensures the future's memory address won't change, satisfying the `Pin<&mut Self>` requirement of `poll`.

### Production Executors

Real executors like Tokio add:
- **Multi-threaded work-stealing**: Distribute tasks across CPU cores
- **I/O driver**: Integrate with epoll/kqueue/IOCP for efficient I/O
- **Timer driver**: Schedule futures for delayed execution
- **Fairness**: Prevent task starvation
- **Metrics**: Track task count, poll durations, etc.

But the core pattern—poll, park, wake, repeat—remains the same.

## Blocking Operations in Async Context

The `spawn-blocking` project bridges blocking operations with async code:

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

### The Pattern

1. **Spawn OS thread**: Blocking work runs in a real thread, not on the async executor
2. **Shared state**: `Arc<Mutex<Shared>>` coordinates between thread and async task
3. **Waker storage**: Async task stores its waker when it first polls
4. **Wake on completion**: Thread wakes the task after storing the result
5. **Result retrieval**: Next poll finds the value and returns `Poll::Ready`

### When to Use

Use `spawn_blocking` when:
- **File I/O**: Non-async file operations (though `tokio::fs` provides async alternatives)
- **CPU-bound work**: Intensive computation that would block the executor
- **C FFI**: Calling blocking C libraries
- **Legacy code**: Integrating synchronous libraries

Avoid when:
- **True async APIs exist**: Prefer `tokio::fs` over `std::fs`
- **Work is trivial**: Overhead exceeds benefit for microsecond operations
- **Frequent calls**: Thread spawn overhead adds up

## Ecosystem Choices: Tokio vs async-std vs smol

Choosing an async runtime is a foundational architectural decision. Each has distinct philosophies:

### Tokio: The Ecosystem Standard

```rust
#[tokio::main]
async fn main() {
    let response = reqwest::get("https://example.com").await.unwrap();
    println!("Status: {}", response.status());
}
```

**Strengths**:
- **Ecosystem**: Most async crates target Tokio
- **Performance**: Highly optimized work-stealing scheduler
- **Features**: Comprehensive (timers, I/O, sync primitives, macros)
- **Production**: Battle-tested at massive scale (Discord, AWS, etc.)

**Drawbacks**:
- **Complexity**: Larger learning curve, more concepts
- **Size**: Larger binary size and dependency tree
- **Flexibility**: Opinionated design choices

**When to choose**: Default for most production applications, especially if using ecosystem crates (hyper, tonic, axum).

### async-std: The Familiar API

```rust
#[async_std::main]
async fn main() {
    let response = surf::get("https://example.com").await.unwrap();
    println!("Status: {}", response.status());
}
```

**Strengths**:
- **Familiarity**: Mirrors std library API (e.g., `async_std::fs` vs `std::fs`)
- **Simplicity**: Fewer concepts, gentler learning curve
- **Stability**: Conservative API design

**Drawbacks**:
- **Ecosystem**: Fewer crates integrate directly
- **Performance**: Slightly behind Tokio in benchmarks
- **Maintenance**: Smaller team, slower development

**When to choose**: Learning async, smaller projects, or when std-like API is valued.

### smol: The Minimalist

```rust
fn main() {
    smol::block_on(async {
        let response = surf::get("https://example.com").await.unwrap();
        println!("Status: {}", response.status());
    });
}
```

**Strengths**:
- **Tiny**: Minimal dependencies, small binary
- **Flexible**: Composable components you can swap
- **Performance**: Competitive despite small size

**Drawbacks**:
- **Ecosystem**: Even fewer integrations than async-std
- **DIY**: More assembly required (choose your own I/O driver, timer, etc.)
- **Documentation**: Sparser than alternatives

**When to choose**: Embedded systems, libraries wanting runtime-agnostic async, or size-constrained environments.

### Runtime-Agnostic Code

Write runtime-agnostic libraries when possible:

```rust
use futures::io::{AsyncRead, AsyncWrite};

async fn copy_data<R, W>(reader: &mut R, writer: &mut W) -> std::io::Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    futures::io::copy(reader, writer).await
}
```

This works with any runtime. Depend on `futures` crate abstractions, not runtime-specific APIs.

## Production Patterns

### Pattern 1: Graceful Shutdown

```rust
use tokio::signal;
use tokio::sync::broadcast;

async fn server_with_shutdown() {
    let (shutdown_tx, _) = broadcast::channel(1);

    // Spawn server tasks
    let shutdown_rx = shutdown_tx.subscribe();
    tokio::spawn(async move {
        let mut shutdown = shutdown_rx;
        loop {
            tokio::select! {
                _ = shutdown.recv() => break,
                result = handle_request() => {
                    if let Err(e) = result {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }
    });

    // Wait for SIGINT
    signal::ctrl_c().await.unwrap();
    println!("Shutting down...");
    shutdown_tx.send(()).unwrap();  // Notify all tasks

    // Give tasks time to clean up
    tokio::time::sleep(Duration::from_secs(2)).await;
}
```

### Pattern 2: Connection Pooling

```rust
use deadpool_postgres::{Config, Pool, Runtime};

async fn with_pool() {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    let pool = cfg.create_pool(Runtime::Tokio1).unwrap();

    // Get connection from pool
    let client = pool.get().await.unwrap();
    let rows = client.query("SELECT * FROM users", &[]).await.unwrap();

    // Connection automatically returned to pool when dropped
}
```

### Pattern 3: Rate Limiting

```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

async fn rate_limited_requests() {
    let limiter = RateLimiter::direct(
        Quota::per_second(NonZeroU32::new(10).unwrap())
    );

    for url in urls {
        limiter.until_ready().await;  // Wait if over limit
        let response = fetch(url).await;
        process(response);
    }
}
```

## Gotchas and Pitfalls

**Gotcha 1: Blocking in async**
```rust
// WRONG: Blocks executor thread
async fn bad() {
    std::thread::sleep(Duration::from_secs(1));  // BAD
}

// RIGHT: Async sleep
async fn good() {
    tokio::time::sleep(Duration::from_secs(1)).await;  // GOOD
}
```

**Gotcha 2: Holding locks across await**
```rust
// WRONG: Lock held during await
async fn bad(data: Arc<Mutex<State>>) {
    let guard = data.lock().unwrap();
    expensive_async_operation().await;  // Lock held entire time!
    guard.update();
}

// RIGHT: Release lock before await
async fn good(data: Arc<Mutex<State>>) {
    let value = {
        let guard = data.lock().unwrap();
        guard.get_value()
    };  // Lock released

    let result = expensive_async_operation(value).await;

    {
        let mut guard = data.lock().unwrap();
        guard.set_result(result);
    }  // Lock released
}
```

**Gotcha 3: Not checking cancellation**
```rust
// WRONG: Ignores cancellation
async fn bad() {
    for i in 0..1000000 {
        expensive_sync_work(i);  // No await - can't cancel
    }
}

// RIGHT: Yield periodically
async fn good() {
    for i in 0..1000000 {
        expensive_sync_work(i);
        if i % 1000 == 0 {
            tokio::task::yield_now().await;  // Allow cancellation
        }
    }
}
```

Real-world async architectures succeed when they respect the async execution model: cooperative scheduling, cancellation at await points, and avoiding blocking operations. The patterns in this chapter—from HTTP concurrency to executor internals to runtime selection—form a toolkit for building robust, scalable async systems. Master these, and you'll leverage async Rust's full power in production.
