# Async as Architectural Primitive

Async programming in Rust represents a fundamental architectural choice, not merely a syntax feature. Unlike languages with built-in async runtimes, Rust treats futures as first-class architectural primitives that you compose into scalable systems. Understanding this distinction transforms how you design concurrent applications.

## Futures as First-Class Values

At its core, Rust's async model centers on the `Future` trait—a zero-cost abstraction representing a computation that may not have completed yet:

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

This simple interface has profound architectural implications. Futures are **lazy**—they do nothing until polled. They are **values**—you can store, compose, and transform them. They are **zero-cost**—the compiler transforms them into state machines with no runtime overhead.

Consider this async function from the `cheapo-request` project:

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

The compiler transforms this into a state machine that pauses at each `.await` point. The generated code has no allocations, no boxing, and no dynamic dispatch unless you explicitly choose those. This is what "zero-cost abstraction" means—you pay only for what you use.

## The Executor Pattern

Futures don't execute themselves. They require an **executor**—the runtime component that polls futures to completion. Rust's standard library deliberately doesn't provide one, making the executor an explicit architectural choice.

Here's a minimal executor from the `block-on` project:

```rust
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

This 20-line function demonstrates the executor pattern:

1. **Create a waker**: The callback mechanism for notifying when progress is possible
2. **Build a context**: Wraps the waker with polling metadata
3. **Poll the future**: Ask if it's complete
4. **Park on pending**: Block the thread until the waker fires
5. **Loop until ready**: Repeat until completion

This pattern scales from simple single-threaded executors to sophisticated work-stealing runtimes like Tokio. The architecture is the same—only the implementation complexity differs.

### Executor Guarantees

Well-designed executors provide these guarantees:

- **Progress**: Futures that are ready to proceed will eventually be polled
- **Wake**: Wakers will be called when futures can make progress
- **Fair scheduling**: No future starves indefinitely (in multi-task executors)
- **Cancellation**: Dropping a future stops its execution cleanly

These guarantees form the contract between futures and executors, enabling compositional reasoning about async systems.

## Task Spawning and Lifecycle

Spawning creates independent units of work that execute concurrently. The `many-requests` project shows this pattern:

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

This code spawns multiple HTTP requests concurrently, each as an independent task. The `spawn_local` call creates a future that runs on the same thread (no `Send` requirement), while `spawn` (not shown) allows multi-threaded execution.

### Task Cancellation

Cancellation in async Rust is **structural**—dropping a future immediately stops execution at the next `.await` point:

```rust
use async_std::task;
use std::time::Duration;

async fn example() {
    let handle = task::spawn(async {
        expensive_operation().await;
        cleanup().await;  // Never runs if handle dropped
    });

    task::sleep(Duration::from_millis(100)).await;
    drop(handle);  // Cancels the task immediately
}
```

This contrasts with explicit cancellation tokens common in other languages. Rust's ownership model makes cancellation a natural consequence of resource management. However, this means you must carefully structure async code to handle cancellation at any `.await` point—a critical architectural consideration.

## Async as Scaling Primitive

The choice between async and threads fundamentally affects system scalability. Consider this comparison:

**Thread-per-connection** (from `echo-server`):

```rust
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

This architecture spawns an OS thread per connection. Each thread:
- Consumes ~2MB of stack space (on Linux)
- Requires kernel scheduling
- Involves context switch overhead
- Limits to ~10,000 concurrent connections on typical systems

**Async-per-connection** architecture:

```rust
async fn echo_async(addr: &str) -> io::Result<()> {
    let listener = async_std::net::TcpListener::bind(addr).await?;
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        task::spawn(async move {
            let (reader, writer) = &mut (&stream, &stream);
            io::copy(reader, writer).await?;
            Ok::<_, io::Error>(())
        });
    }
    Ok(())
}
```

Each async task:
- Consumes ~70 bytes (just the state machine)
- Scheduled cooperatively by the executor
- No kernel involvement
- Scales to millions of concurrent connections

### When to Choose Async

Use async as your scaling primitive when:

1. **I/O dominates**: Most time spent waiting on network, disk, or databases
2. **High concurrency**: Thousands to millions of concurrent operations
3. **Latency matters**: Response time is critical
4. **Resource constrained**: Memory or CPU limits thread count

Avoid async when:

1. **CPU-bound**: Most time spent computing (use threads or rayon)
2. **Simple sequential flow**: Async adds unnecessary complexity
3. **Blocking APIs**: Must integrate with non-async libraries (use `spawn_blocking`)
4. **Real-time**: Need deterministic timing (async scheduling is cooperative)

## Async Decision Tree

```
Is the work primarily I/O-bound?
├─ Yes: Do you need >1000 concurrent operations?
│  ├─ Yes: Use async ✓
│  └─ No: Are you comfortable with async complexity?
│     ├─ Yes: Use async ✓
│     └─ No: Use threads (simpler)
└─ No (CPU-bound):
   ├─ Need parallelism? Use threads or rayon ✓
   └─ Sequential? Don't use async ✓
```

## The Async Contract

When you choose async as your architectural primitive, you accept certain contracts:

**Performance Contract**: Async functions should return quickly from each `.await` point. Blocking operations must use `spawn_blocking` to avoid starving the executor.

**Composability Contract**: Async functions compose with `.await`. Mixing blocking and async requires explicit bridging.

**Cancellation Contract**: Any `.await` point may be a cancellation point. Design for graceful shutdown.

**Send Contract**: Tasks spawned across threads require `Send` futures. This propagates through your entire async call stack—an architectural decision with deep implications.

Understanding async as an architectural primitive means recognizing that it's not just about making code concurrent—it's about choosing a specific model of concurrency with particular tradeoffs, guarantees, and constraints. The next sections explore how to design systems that leverage these properties effectively.
