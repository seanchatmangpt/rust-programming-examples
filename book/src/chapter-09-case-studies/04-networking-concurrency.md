# Networking and Concurrency Architecture

Network services present unique architectural challenges: they must handle multiple simultaneous connections, manage I/O efficiently, and maintain correctness in the face of concurrency. Rust's ownership system and async ecosystem provide powerful tools for building robust network services. This section examines two architectures—thread-per-connection and async concurrency—revealing their trade-offs and implementation patterns.

## TCP Server Architecture: The Echo Server

The echo server demonstrates the classic thread-per-connection model:

```rust
use std::net::TcpListener;
use std::io;
use std::thread::spawn;

fn echo_main(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("listening on {}", addr);

    loop {
        let (mut stream, addr) = listener.accept()?;
        println!("connection received from {}", addr);

        let mut write_stream = stream.try_clone()?;
        spawn(move || {
            io::copy(&mut stream, &mut write_stream)
                .expect("error in client thread: ");
            println!("connection closed");
        });
    }
}
```

This architecture embodies several fundamental patterns:

**The Accept Loop**: The infinite `loop` with `listener.accept()` is the **reactor pattern**—waiting for events (new connections) and dispatching them to handlers (threads). This loop is **blocking**—it pauses until a connection arrives.

**Thread-Per-Connection**: Each connection spawns a dedicated thread. This provides **isolation**—one slow client can't block others. However, threads have overhead: ~2MB stack per thread on Linux, plus context switching costs. This architecture scales to hundreds of connections, not thousands.

**Stream Cloning for Bidirectional I/O**: The `stream.try_clone()` call creates a second handle to the same TCP socket. Why? Because `io::copy(&mut stream, &mut write_stream)` needs both read and write access simultaneously. The same socket serves both roles.

**Move Closure for Thread Safety**: The `move || { ... }` closure takes ownership of `stream` and `write_stream`. The `spawn` function requires `'static` lifetime—the thread might outlive its parent. Moving ownership ensures the streams remain valid for the thread's lifetime.

## Concurrent Request Handling Patterns

The echo server's architecture reveals a critical pattern:

```rust
spawn(move || {
    io::copy(&mut stream, &mut write_stream)
        .expect("error in client thread: ");
    println!("connection closed");
});
```

**Error Handling in Threads**: The `expect()` call will panic if I/O fails. In a thread, panics don't crash the process—only the panicking thread dies. Other client threads continue running. This is **fault isolation**.

However, silent thread panics are problematic in production. Better approach:

```rust
let handle = spawn(move || -> io::Result<()> {
    io::copy(&mut stream, &mut write_stream)?;
    println!("connection closed");
    Ok(())
});

// In production, you'd join and log errors:
match handle.join() {
    Ok(Ok(())) => {},
    Ok(Err(e)) => eprintln!("Client error: {}", e),
    Err(_) => eprintln!("Client thread panicked"),
}
```

**Resource Management**: When the thread finishes, `stream` and `write_stream` are dropped, automatically closing the socket. This is **RAII for network resources**—no explicit cleanup needed, even in error paths.

## Async Concurrency: Many Concurrent Requests

The many-requests example demonstrates async architecture:

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
```

**Async Functions**: The `async fn` keyword creates a function that returns a `Future`. The function doesn't execute immediately—it produces a computation that can be executed later. This is **lazy evaluation**.

**Await Points**: Each `.await` is a **suspension point**. If the operation would block (waiting for network data), the future yields control to the executor. Other futures can run while this one waits. This is **cooperative multitasking**.

**Error Propagation in Async**: The `?` operator works seamlessly in async functions, propagating errors through the Future. This is async-aware error handling.

## Concurrent Execution with async-std

The `many_requests` function demonstrates concurrent async execution:

```rust
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

**Architecture Patterns**:

**Spawn-Join Pattern**: This is the async equivalent of thread spawning. `spawn_local` schedules a task on the executor. The returned handle allows awaiting completion. This is **structured concurrency**—you explicitly wait for spawned tasks.

**Why spawn_local?**: The `_local` variant means the task runs on the current thread. Non-local spawn (`task::spawn`) allows work stealing across threads but requires `Send` futures. Local spawning avoids that requirement.

**Sequential Collection**: The second loop awaits handles sequentially. This is a weakness—we could use `join_all` to wait concurrently:

```rust
use futures::future::join_all;

async fn many_requests_improved(requests: Vec<(String, u16, String)>)
    -> Vec<std::io::Result<String>>
{
    let futures = requests.into_iter().map(|(host, port, path)| {
        cheapo_request(&host, port, &path)
    });

    join_all(futures).await
}
```

This version is more elegant and efficient—all requests start immediately, and `join_all` awaits them concurrently.

## Resource Pooling Strategies

Network services often need resource pools to limit concurrency. The echo server has no limit—it spawns unbounded threads. Production systems need constraints:

**Thread Pool Pattern**:
```rust
use threadpool::ThreadPool;

fn echo_with_pool(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    let pool = ThreadPool::new(100);  // Max 100 concurrent clients

    loop {
        let (stream, _) = listener.accept()?;
        let stream_clone = stream.try_clone()?;

        pool.execute(move || {
            io::copy(&mut stream, &mut stream_clone).ok();
        });
    }
}
```

The thread pool **bounds resource usage**. If 100 threads are busy, new connections queue until a thread becomes available. This prevents resource exhaustion.

**Async Semaphore Pattern**:
```rust
use async_std::sync::Semaphore;

async fn limited_requests(
    requests: Vec<Request>,
    max_concurrent: usize
) -> Vec<Response> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    let futures = requests.into_iter().map(|req| {
        let sem = semaphore.clone();
        async move {
            let _permit = sem.acquire().await;
            process_request(req).await
        }
    });

    join_all(futures).await
}
```

The semaphore limits concurrent execution. `acquire()` blocks until a permit is available. This is **backpressure**—slowing input to match processing capacity.

## Case Study: Echo Server Architecture

Let's trace a complete connection through the echo server:

**1. Server Initialization**:
```rust
let listener = TcpListener::bind("127.0.0.1:17007")?;
// OS allocates socket, binds to port 17007
// listener is now ready to accept connections
```

**2. Waiting for Connections**:
```rust
loop {
    let (mut stream, addr) = listener.accept()?;
    // Blocks until client connects
    // Returns new socket for this client
```

The accept call is **blocking I/O**—the thread pauses until a client connects. In async systems, accept would be awaitable, allowing other work during the wait.

**3. Connection Handling**:
```rust
let mut write_stream = stream.try_clone()?;
// Creates second handle to same socket
// Both handles share underlying file descriptor
```

**4. Thread Spawning**:
```rust
spawn(move || {
    // stream and write_stream moved into thread
    // Parent thread can't access them anymore
    io::copy(&mut stream, &mut write_stream)
        .expect("error in client thread: ");
});
// Thread runs independently, parent continues to accept()
```

**5. Echo Logic**:
```rust
io::copy(&mut stream, &mut write_stream)
// Reads from stream, writes to write_stream
// Since both point to same socket, data echoes back
// Continues until client closes connection (read returns 0)
```

**6. Cleanup**:
```rust
println!("connection closed");
// Thread exits
// stream and write_stream dropped
// Socket automatically closed
// OS frees resources
```

This architecture is **simple and correct** but has limitations: thread overhead limits scalability, no graceful shutdown mechanism, and errors crash the thread silently.

## Case Study: Concurrent HTTP Pattern

The many-requests architecture demonstrates async HTTP:

**1. Request Preparation**:
```rust
let requests = vec![
    ("example.com".to_string(), 80, "/".to_string()),
    ("www.red-bean.com".to_string(), 80, "/".to_string()),
    ("en.wikipedia.org".to_string(), 80, "/".to_string()),
];
```

**2. Task Spawning**:
```rust
for (host, port, path) in requests {
    handles.push(task::spawn_local(async move {
        cheapo_request(&host, port, &path).await
    }));
}
// All three requests start immediately
// They don't wait for each other
```

**3. Concurrent Execution**:
```
Timeline:
0ms:  All three TCP connect operations start
10ms: example.com connected, sending request
15ms: red-bean.com connected, sending request
20ms: wikipedia.org connected, sending request
25ms: example.com receives response
30ms: red-bean.com receives response
40ms: wikipedia.org receives response
```

In the synchronous version, these would be sequential:
```
Timeline:
0ms:  example.com connect
10ms: example.com request/response
25ms: red-bean.com connect
35ms: red-bean.com request/response
60ms: wikipedia.org connect
80ms: wikipedia.org request/response
```

The async version is **3x faster** for this workload. The speedup comes from overlapping I/O—while waiting for one server's response, we're sending requests to others.

**4. Result Collection**:
```rust
for handle in handles {
    results.push(handle.await);
}
// Waits for all tasks to complete
// Results collected in order
```

## Backpressure and Flow Control

Network services need backpressure to prevent overload. Consider an echo server that reads faster than it writes:

```rust
// PROBLEMATIC: Unbounded buffering
let mut buffer = Vec::new();
loop {
    stream.read_to_end(&mut buffer)?;  // Could OOM!
    stream.write_all(&buffer)?;
}

// BETTER: Bounded buffering
let mut buffer = [0u8; 4096];
loop {
    let n = stream.read(&mut buffer)?;
    if n == 0 { break; }
    stream.write_all(&buffer[..n])?;
}
```

The bounded buffer provides **natural backpressure**. If the client sends data faster than the server can echo it back, the TCP window fills, and the client slows down. This is **TCP's built-in flow control**.

In async systems, backpressure is more explicit:

```rust
use futures::stream::StreamExt;

async fn bounded_processing(input: impl Stream<Item = Request>) {
    input
        .buffer_unordered(100)  // Max 100 in-flight requests
        .for_each(|result| async move {
            process(result).await
        })
        .await
}
```

The `buffer_unordered(100)` limits concurrent processing. If 100 requests are being processed, the stream pauses accepting new ones. This **prevents resource exhaustion**.

## Architectural Trade-offs

Comparing thread-based and async architectures:

**Thread-Per-Connection**:
- ✅ Simple to understand and implement
- ✅ Isolation: one client's problem doesn't affect others
- ✅ Blocking I/O is fine—thread can block
- ❌ Limited scalability (hundreds of connections)
- ❌ Higher memory usage (stack per thread)
- ❌ Context switching overhead

**Async/Await**:
- ✅ High scalability (thousands/millions of connections)
- ✅ Low memory overhead (futures are small)
- ✅ Efficient I/O multiplexing
- ❌ More complex: futures, executors, pinning
- ❌ Must avoid blocking operations
- ❌ Error handling can be tricky across await points

**When to Use Each**:
- Threads: Low connection count, heavy CPU work per connection, need simplicity
- Async: High connection count, I/O-bound workloads, need maximum throughput

## Testing Concurrent Systems

Concurrency introduces non-determinism. Tests must be robust:

**Testing Echo Server**:
```rust
#[test]
fn test_echo_server() {
    use std::net::TcpStream;
    use std::io::{Write, Read};

    // Start server in background thread
    std::thread::spawn(|| {
        echo_main("127.0.0.1:17008").unwrap();
    });

    std::thread::sleep(Duration::from_millis(100));  // Wait for server

    let mut stream = TcpStream::connect("127.0.0.1:17008").unwrap();
    stream.write_all(b"hello").unwrap();

    let mut buf = [0u8; 5];
    stream.read_exact(&mut buf).unwrap();

    assert_eq!(&buf, b"hello");
}
```

**Testing Async Code**:
```rust
#[async_std::test]
async fn test_concurrent_requests() {
    let requests = vec![
        ("example.com".to_string(), 80, "/".to_string()),
    ];

    let results = many_requests(requests).await;
    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok());
}
```

The `#[async_std::test]` macro provides an async test runtime, similar to how `#[actix_web::test]` works for web tests.

## Cross-References to Earlier Patterns

Network architectures build on foundational concepts:

- **Chapter 2 (Ownership)**: Move semantics enable safe thread spawning. The `move` closure transfers ownership.
- **Chapter 5 (Error Handling)**: `?` operator propagates I/O errors through sync and async code uniformly.
- **Chapter 7 (Concurrency)**: Thread spawning, async/await, and task management all appear here.
- **Chapter 8 (Async)**: The many-requests example demonstrates async I/O, futures, and executors.

These case studies show how Rust's ownership model enables **fearless concurrency** in network services. The compiler prevents data races, the type system enforces proper resource management, and the async ecosystem provides performance without sacrificing safety.

Next, we'll synthesize all these patterns, examining how they integrate in systems with multiple architectural layers.
