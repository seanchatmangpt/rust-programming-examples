# Concurrent Futures

## Context

You have multiple independent asynchronous operations to perform—such as making several HTTP requests, reading multiple files, or querying different databases. These operations don't depend on each other, and you want them to execute concurrently to minimize total completion time.

You understand **ASYNC/AWAIT BASICS** and can write async functions, but currently you're executing them sequentially, which wastes time.

## Problem

**How do you execute multiple independent futures concurrently, starting them all simultaneously and collecting their results when all complete?**

If you await futures sequentially, each one blocks until completion before the next starts:

```rust
// Sequential - SLOW (takes sum of all durations)
let result1 = request1().await?;  // waits for completion
let result2 = request2().await?;  // then starts this one
let result3 = request3().await?;  // then starts this one
```

This defeats the purpose of async I/O. Three requests that each take 100ms would take 300ms total, even though they could run concurrently in ~100ms.

You need a way to:
- Start multiple futures executing simultaneously
- Continue other work while they run
- Collect all results when they complete
- Handle individual failures without losing successful results
- Maintain type safety and ownership rules

## Forces

- **Concurrency vs. Parallelism**: Futures enable concurrency (interleaved execution), which on a single thread can handle I/O-bound work efficiently. True parallelism (multiple CPU cores) requires spawning tasks.
- **Task Spawning**: Some runtimes support spawning tasks on other threads (requiring `Send`), others support lightweight spawning on the current thread (no `Send` requirement).
- **Error Handling**: If one operation fails, do you want to cancel the others, or continue and collect both successes and failures?
- **Ordering**: Do you need results in the same order as the requests, or is any order acceptable?
- **Resource Limits**: Spawning thousands of concurrent tasks can overwhelm the target (like a server) or consume too much memory.
- **Synchronization**: Managing shared state across concurrent tasks requires careful coordination.

## Solution

**Spawn each future as a separate task, store the handles, then await each handle to collect results.**

Use the runtime's task spawning mechanism (`spawn` or `spawn_local`) to start futures executing concurrently. Collect the join handles, then await each one to get results. This pattern starts all futures before waiting for any of them to complete.

### Structure

```rust
async fn concurrent_operations(inputs: Vec<Input>)
    -> Vec<Result<Output, Error>>
{
    let mut handles = vec![];

    // Start all futures concurrently
    for input in inputs {
        let handle = spawn_task(async move {
            async_operation(input).await
        });
        handles.push(handle);
    }

    // Collect results as they complete
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
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

    // Spawn all requests concurrently
    for (host, port, path) in requests {
        handles.push(task::spawn_local(async move {
            cheapo_request(&host, port, &path).await
        }));
    }

    // Collect all results
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}
```

### Usage Example

```rust
fn main() {
    let requests = vec![
        ("example.com".to_string(),      80, "/".to_string()),
        ("www.red-bean.com".to_string(), 80, "/".to_string()),
        ("en.wikipedia.org".to_string(), 80, "/".to_string()),
    ];

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

1. **Task Spawning**:
   - `spawn_local` creates a new task on the current thread
   - The task runs concurrently with other tasks on the same thread
   - Returns a `JoinHandle` that can be awaited for the result

2. **Async Move Closure**:
   - `async move { ... }` captures ownership of variables
   - Necessary because the spawned task may outlive the current scope
   - The closure returns a Future that spawn_local will execute

3. **Two-Phase Execution**:
   - **Phase 1**: Spawn all tasks (starts them executing)
   - **Phase 2**: Await all handles (collects results)
   - This ensures tasks run concurrently, not sequentially

4. **Result Collection**:
   - Each `handle.await` yields the result of that task
   - Results are collected in the same order as tasks were spawned
   - Individual task failures don't prevent collecting other results

### Timing Comparison

```rust
// Sequential: 300ms (100ms + 100ms + 100ms)
let r1 = request1().await?;
let r2 = request2().await?;
let r3 = request3().await?;

// Concurrent: ~100ms (all three run simultaneously)
let handles = vec![
    spawn_local(request1()),
    spawn_local(request2()),
    spawn_local(request3()),
];
let results: Vec<_> = handles.into_iter()
    .map(|h| h.await)
    .collect();
```

### spawn vs. spawn_local

- **`spawn_local`**: Runs on the current thread only, doesn't require `Send` bound
- **`spawn`**: Can run on any thread in the pool, requires `Send` + `'static`

Use `spawn_local` when:
- Working with `Rc`, `RefCell`, or other non-`Send` types
- Want deterministic single-threaded execution
- Performance: Avoid thread synchronization overhead

Use `spawn` (see **ASYNC HTTP CLIENT LIBRARY**) when:
- Want true parallel execution on multiple CPU cores
- Tasks are `Send` and `'static`
- Need to maximize throughput for CPU-bound work

## Resulting Context

Your concurrent operations now:

- **Execute simultaneously**: All futures start before any completes, minimizing total time
- **Scale efficiently**: Can handle hundreds or thousands of concurrent I/O operations on a single thread
- **Preserve order**: Results are collected in the same order as requests (if desired)
- **Handle errors gracefully**: Each result is independent; failures don't affect other tasks
- **Compose naturally**: Can nest concurrent operations within larger async functions

However, you've introduced new considerations:

- **Unbounded concurrency**: Spawning too many tasks at once can overwhelm resources (use semaphores or batching for large workloads)
- **Error aggregation**: Individual errors are preserved, but you may want to combine them or short-circuit on first error
- **Memory usage**: All spawned tasks and their state exist in memory simultaneously
- **Ordering guarantees**: Tasks may complete in different order than spawned; if order matters, you must maintain it explicitly

When you need thread parallelism (not just concurrency), use `spawn` instead of `spawn_local` (requires `Send` bound).

When making HTTP requests specifically, consider using **ASYNC HTTP CLIENT LIBRARY** which may have built-in concurrency support.

## Related Patterns

- **ASYNC/AWAIT BASICS**: Foundation for writing async functions
- **SPAWN LOCAL**: Details on spawning tasks on the current thread
- **ASYNC HTTP REQUEST**: Common use case for concurrent execution
- **ASYNC HTTP CLIENT LIBRARY**: Higher-level clients with built-in concurrency

## Known Uses

- **Web scrapers**: Fetch hundreds of pages concurrently without blocking threads
- **API aggregators**: Call multiple backend services in parallel, combine results
- **Batch processors**: Process many independent items concurrently (image resizing, file conversion)
- **Health checkers**: Ping many endpoints simultaneously to check service health
- **Database queries**: Execute independent queries concurrently (with connection pooling)
- **File operations**: Read or write many files concurrently on async runtimes

Real-world implementations:

- **futures::future::join_all**: Waits for all futures to complete, returns `Vec` of results
- **futures::future::try_join_all**: Like `join_all` but short-circuits on first error
- **tokio::join!**: Macro for awaiting a fixed number of futures concurrently
- **async-std::task::spawn**: Spawns tasks across thread pool
- **rayon** (not async): Data parallelism for CPU-bound work

The pattern shown here (manual spawn + collect) gives full control over execution and error handling. Libraries provide more convenient combinators like `join_all`, but understanding the manual pattern helps when you need custom behavior like rate limiting, retries, or partial result processing.
