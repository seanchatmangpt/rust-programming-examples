# Async Refactoring & Concurrency Patterns

Modern Rust applications increasingly rely on asynchronous programming. This chapter explores the transition from synchronous to asynchronous code, examines common concurrency patterns, and provides practical guidance for building robust async systems.

## Async/Await Fundamentals

### Understanding Futures

A `Future` in Rust represents a computation that may not have completed yet. Futures are lazy—they do nothing until polled. This fundamental difference drives Rust's async model:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct YieldOnce {
    yielded: bool,
}

impl Future for YieldOnce {
    type Output = &'static str;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            Poll::Ready("completed")
        } else {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
```

The `async/await` syntax provides ergonomic access without manual `Future` implementation:

```rust
async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    response.text().await
}
```

### Executors and Runtimes

Futures require an executor to drive them to completion. Rust's standard library provides the `Future` trait but no runtime—this is intentional, allowing ecosystem diversity:

| Runtime | Use Case | Key Features |
|---------|----------|--------------|
| **Tokio** | General-purpose, production servers | Work-stealing, io-uring |
| **async-std** | Standard library-like API | Familiar naming |
| **smol** | Minimal, embeddable | Small binary size |
| **Embassy** | Embedded systems | No-std, interrupt-driven |

## Refactoring to Async

### From Blocking to Async I/O

The transition from blocking to async code follows predictable patterns:

```rust
// Before (Blocking)
fn read_lines(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

// After (Async)
async fn read_lines(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut result = Vec::new();
    while let Some(line) = lines.next_line().await? {
        result.push(line);
    }
    Ok(result)
}
```

The async version enables concurrent execution when combined with spawning:

```rust
async fn process_files_concurrent(paths: &[&str]) -> std::io::Result<Vec<String>> {
    let futures: Vec<_> = paths
        .iter()
        .map(|path| read_lines(path))
        .collect();

    let results = futures::future::try_join_all(futures).await?;
    Ok(results.into_iter().flatten().collect())
}
```

### Runtime Abstraction Patterns

To write runtime-agnostic code, abstract over runtime-specific types using traits:

```rust
pub trait Spawner: Send + Sync + 'static {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static;
}

#[cfg(feature = "tokio-runtime")]
pub struct TokioSpawner;

#[cfg(feature = "tokio-runtime")]
impl Spawner for TokioSpawner {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(future);
    }
}
```

## Concurrency Patterns

### Channels for Communication

Channels provide safe communication between async tasks without shared mutable state:

```rust
use tokio::sync::mpsc;

#[derive(Debug)]
enum Command {
    Increment,
    Decrement,
    Get(tokio::sync::oneshot::Sender<i64>),
}

async fn counter_actor(mut rx: mpsc::Receiver<Command>) {
    let mut count: i64 = 0;

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Increment => count += 1,
            Command::Decrement => count -= 1,
            Command::Get(reply) => {
                let _ = reply.send(count);
            }
        }
    }
}
```

### Shared State with Arc<Mutex<T>>

For shared mutable state, use `Arc<Mutex<T>>`:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct SharedState {
    data: Arc<Mutex<Vec<String>>>,
}

impl SharedState {
    async fn add(&self, item: String) {
        let mut guard = self.data.lock().await;
        guard.push(item);
    }
}
```

**Important**: Use `tokio::sync::Mutex` for async code, not `std::sync::Mutex`. The standard library mutex blocks the entire thread.

### Task Spawning and Structured Concurrency

```rust
use tokio::task::JoinSet;

async fn structured_concurrency() -> Vec<Result<String, String>> {
    let mut set = JoinSet::new();

    let urls = vec!["url1", "url2", "url3"];

    for url in urls {
        set.spawn(async move {
            fetch_and_process(url).await
        });
    }

    let mut results = Vec::new();

    while let Some(result) = set.join_next().await {
        match result {
            Ok(Ok(data)) => results.push(Ok(data)),
            Ok(Err(e)) => results.push(Err(e)),
            Err(join_error) => results.push(Err(format!("Task panicked: {}", join_error))),
        }
    }

    results
}
```

## Testing Async Code

### Using tokio-test

```rust
#[tokio::test]
async fn test_async_parser() {
    let parser = AsyncParser::new();
    let args = futures::stream::iter(vec![
        "--verbose".to_string(),
        "--output".to_string(),
        "file.txt".to_string(),
    ]);

    let result = parser.parse_stream(args).await;
    assert!(result.is_ok());
}
```

## Case Study: Async Refactoring with Streaming Parsing

Consider a CLI framework that needs to parse arguments from streaming sources:

```rust
use futures::stream::Stream;
use tokio::sync::mpsc;

pub struct AsyncParser {
    rules: Vec<Rule>,
}

impl AsyncParser {
    pub async fn parse_stream<S>(&self, args: S) -> Result<ParseResult, ParseError>
    where
        S: Stream<Item = String> + Unpin,
    {
        let mut state = ParseState::new();
        let mut args = args;

        while let Some(arg) = args.next().await {
            self.process_arg(&mut state, &arg).await?;
        }

        state.finalize()
    }
}
```

## Best Practices

### Async-Aware Design

1. **Design for cancellation**: Ensure resources are cleaned up when tasks are cancelled
2. **Prefer bounded channels**: Unbounded channels risk memory exhaustion
3. **Use structured concurrency**: `JoinSet` over bare `spawn` when possible
4. **Document Send/Sync requirements**: Be explicit about thread safety

### Avoiding Blocking

```rust
// WRONG: Blocks the executor thread
async fn bad_sleep() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}

// CORRECT: Yields to executor
async fn good_sleep() {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}

// CORRECT: Move blocking to dedicated thread
async fn blocking_computation() -> i32 {
    tokio::task::spawn_blocking(|| {
        expensive_computation()
    }).await.unwrap()
}
```

## Summary

Async programming in Rust provides powerful tools for building concurrent, efficient applications. Key takeaways:

- **Futures are lazy**: They require an executor to make progress
- **Runtime abstraction enables flexibility**: Design for portability across runtimes
- **Channels over shared state**: Prefer message passing for cleaner concurrency
- **Cancellation safety matters**: Use RAII patterns for resource cleanup
- **Test thoroughly**: Async code has unique failure modes requiring dedicated testing

By applying these patterns, you can build robust async systems that leverage Rust's safety guarantees while achieving excellent performance.
