# Async and Concurrency in Rust

## Two Powerful But Different Concepts

Python developers often use "async" and "concurrency" interchangeably, but in Rust they represent distinct approaches to handling multiple tasks. Understanding the difference is crucial:

- **Async/await**: Cooperative multitasking for I/O-bound work (like Python's asyncio)
- **Threads**: True parallelism for CPU-bound work (like Python's threading/multiprocessing)

Rust excels at both, but with compile-time safety guarantees that Python can't provide.

## Async/Await: Rust vs Python

Both Rust and Python have async/await, but they work very differently under the hood.

### Python's Approach

```python
import asyncio

async def fetch_data(url):
    # Cooperative multitasking
    response = await make_request(url)
    return response.json()

async def main():
    result = await fetch_data("https://api.example.com")
    print(result)

# asyncio provides the runtime
asyncio.run(main())
```

Python's async features:
- **Built-in runtime** (asyncio event loop)
- **Single-threaded** cooperative multitasking
- **GIL still applies** (no true parallelism)
- **Runtime overhead** from the event loop

### Rust's Approach

Rust's async is fundamentally different. From `/home/user/rust-programming-examples/block-on/src/lib.rs`:

```rust
use std::future::Future;
use std::task::{Context, Poll};
use waker_fn::waker_fn;
use futures_lite::pin;
use crossbeam::sync::Parker;

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

This is a minimal async runtime. It shows what Python's asyncio hides:

1. **Polling**: Futures are polled to check if they're ready
2. **Wakers**: Mechanisms to notify when a future can make progress
3. **Parking**: Suspending the current thread while waiting

Rust's async features:
- **No built-in runtime** (you choose: tokio, async-std, or roll your own)
- **Zero-cost abstraction** (compiles to state machines)
- **No GIL** (true parallelism possible)
- **Explicit runtime** (you control the executor)

## Futures: The Building Blocks

In Python, async functions return coroutines. In Rust, they return `Future`:

```rust
trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}
```

A `Future` represents a computation that might not be complete yet. Unlike Python coroutines, Rust futures:
- **Do nothing until polled** (lazy evaluation)
- **Are polled to completion** (no implicit suspension)
- **Compile to state machines** (zero allocation overhead)

### Writing Async Functions

```rust
async fn fetch_data(url: &str) -> Result<String, Error> {
    let response = http::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}
```

The compiler transforms this into a `Future` implementation:

```rust
// Conceptually similar to:
impl Future for FetchDataFuture {
    type Output = Result<String, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // State machine handling each await point
    }
}
```

Python does something similar but with more runtime overhead.

## Executors and Runtimes

Python has asyncio built-in. Rust requires you to choose an async runtime:

### Tokio (most popular)

```rust
#[tokio::main]
async fn main() {
    let result = fetch_data("https://example.com").await;
}
```

### async-std (similar to Python's stdlib)

```rust
#[async_std::main]
async fn main() {
    let result = fetch_data("https://example.com").await;
}
```

### Custom (like our block_on example)

The `block_on` function is a minimal executor. It:
1. Creates a waker (callback for when the future makes progress)
2. Polls the future
3. If `Pending`, parks the thread until the waker is called
4. If `Ready`, returns the value

This is what Python's `asyncio.run()` does under the hood, but Rust makes it explicit.

## Send and Sync: Thread Safety Traits

Here's where Rust shines compared to Python. These two auto-traits provide compile-time guarantees:

```rust
pub trait Send {}  // Type can be transferred between threads
pub trait Sync {}  // Type can be referenced from multiple threads
```

From `/home/user/rust-programming-examples/spawn-blocking/src/lib.rs`:

```rust
pub fn spawn_blocking<T, F>(closure: F) -> SpawnBlocking<T>
where F: FnOnce() -> T,
      F: Send + 'static,  // Closure must be sendable between threads
      T: Send + 'static,  // Return value must be sendable too
{
    let inner = Arc::new(Mutex::new(Shared {
        value: None,
        waker: None,
    }));

    std::thread::spawn({
        let inner = inner.clone();
        move || {
            let value = closure();
            let mut guard = inner.lock().unwrap();
            guard.value = Some(value);
        }
    });

    SpawnBlocking(inner)
}
```

The `Send + 'static` bounds ensure:
- The closure can be safely moved to another thread (`Send`)
- It doesn't reference any short-lived data (`'static`)

The compiler verifies this. In Python:

```python
import threading

def spawn_blocking(closure):
    # No compile-time checks!
    # Might race, might deadlock, might access freed memory
    thread = threading.Thread(target=closure)
    thread.start()
    return thread
```

Python can't guarantee thread safety at compile time. Rust does.

### What Send and Sync Mean

- **Send**: Safe to transfer ownership to another thread
  - Most types are `Send` (numbers, strings, owned data)
  - `Rc<T>` is NOT `Send` (not thread-safe)
  - `Arc<T>` IS `Send` (atomic reference counting)

- **Sync**: Safe to share references between threads
  - `T` is `Sync` if `&T` is `Send`
  - `Cell<T>` is NOT `Sync` (interior mutability without synchronization)
  - `Mutex<T>` IS `Sync` (provides synchronization)

These traits prevent data races at compile time:

```rust
use std::rc::Rc;

fn try_to_share_rc() {
    let rc = Rc::new(42);
    std::thread::spawn(move || {
        println!("{}", rc);  // COMPILE ERROR: Rc is not Send
    });
}
```

The compiler rejects this. In Python, this would compile and might crash at runtime.

## Concurrency: Async vs Threads

When to use async vs threads?

### Use Async For I/O-Bound Work

```rust
async fn handle_connections() {
    // Thousands of concurrent connections
    let mut tasks = vec![];

    for i in 0..10000 {
        tasks.push(async move {
            let response = fetch_data(format!("api/{}", i)).await?;
            process(response).await
        });
    }

    futures::future::join_all(tasks).await;
}
```

Benefits:
- **Lightweight** (futures are just state machines)
- **Many concurrent tasks** (thousands or millions)
- **Efficient I/O** (no blocking threads)
- **Single-threaded** (or multi-threaded with work-stealing)

Python equivalent:

```python
async def handle_connections():
    tasks = []
    for i in range(10000):
        tasks.append(fetch_data(f"api/{i}"))
    await asyncio.gather(*tasks)
```

Similar, but Python's tasks have more overhead.

### Use Threads For CPU-Bound Work

```rust
use std::thread;

fn process_in_parallel(data: Vec<Data>) -> Vec<Result> {
    let chunk_size = data.len() / num_cpus::get();
    let chunks: Vec<_> = data.chunks(chunk_size).collect();

    let handles: Vec<_> = chunks.into_iter().map(|chunk| {
        thread::spawn(move || {
            chunk.iter().map(|item| expensive_computation(item)).collect()
        })
    }).collect();

    handles.into_iter()
        .flat_map(|h| h.join().unwrap())
        .collect()
}
```

Benefits:
- **True parallelism** (uses multiple CPU cores)
- **No async overhead** (direct thread usage)
- **CPU-bound work** (computation, not I/O)

Python equivalent:

```python
from multiprocessing import Pool

def process_in_parallel(data):
    # Must use multiprocessing, not threading (GIL!)
    with Pool() as pool:
        return pool.map(expensive_computation, data)
```

Python's GIL prevents thread parallelism for CPU work. Rust has no GIL.

### Combining Async and Threads

The `spawn_blocking` example shows bridging async and sync:

```rust
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

This future:
1. Spawns a thread for blocking work
2. Returns `Poll::Pending` until the thread completes
3. Wakes the async task when done
4. Returns `Poll::Ready(value)` with the result

Python's `asyncio` has similar utilities:

```python
async def run_blocking_work():
    loop = asyncio.get_event_loop()
    result = await loop.run_in_executor(None, blocking_function)
    return result
```

But Rust's version is type-safe and zero-cost.

## When to Use Async

Async is ideal for:

1. **I/O-bound applications**
   - Web servers handling many connections
   - Database clients with connection pools
   - Network services
   - File I/O (with async file APIs)

2. **High concurrency**
   - Thousands of simultaneous operations
   - Low memory overhead per task
   - Cooperative scheduling

3. **Event-driven systems**
   - Message passing
   - Stream processing
   - Reactive systems

Avoid async for:

1. **CPU-bound work** - Use threads instead
2. **Simple sequential code** - Async adds complexity
3. **Blocking operations** - Use `spawn_blocking` or threads

## Real Example: Block On

The `block_on` implementation shows async internals:

```rust
pub fn block_on<F: Future>(future: F) -> F::Output {
    let parker = Parker::new();
    let unparker = parker.unparker().clone();

    // Create a waker that unparks the thread
    let waker = waker_fn(move || unparker.unpark());
    let mut context = Context::from_waker(&waker);

    pin!(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => parker.park(),  // Block until waker is called
        }
    }
}
```

This is a minimal executor that:
- Polls the future
- Parks the current thread if pending
- Gets woken up when the future makes progress
- Loops until the future completes

Compare to Python's approach (conceptual):

```python
def run(coro):
    # Python's event loop does this internally
    task = Task(coro)
    while not task.done():
        task.step()  # Execute until next await
        if not task.done():
            event_loop.park_until_ready(task)
    return task.result()
```

Rust makes the mechanism explicit and gives you control.

## Async Traits and Bounds

When working with async, you often need trait bounds:

```rust
async fn process<T>(items: Vec<T>) -> Vec<Result<Data, Error>>
where
    T: Send + Sync + 'static,  // Can be used across threads
{
    let futures = items.into_iter().map(|item| async move {
        fetch_data_for(item).await
    });

    futures::future::join_all(futures).await
}
```

The bounds ensure thread safety:
- `Send`: Values can move between threads
- `Sync`: References can be shared between threads
- `'static`: No borrowed data (lives for the entire program)

Python has no equivalent compile-time checks.

## Performance Characteristics

### Async Overhead

Rust async:
- **Compile-time transformation** to state machines
- **No allocations** for the state (usually)
- **Zero runtime cost** when not awaiting
- **Poll-based** (no implicit suspension)

Python async:
- **Runtime overhead** for the event loop
- **Allocations** for coroutine frames
- **GC pressure** from async objects
- **Implicit suspension** at await points

### Threading Overhead

Rust threads:
- **OS threads** (heavyweight but parallel)
- **No GIL** (true parallelism)
- **Zero-cost** after creation
- **Send/Sync** checking prevents races

Python threads:
- **OS threads** (heavyweight)
- **GIL prevents parallelism** for CPU work
- **Must use multiprocessing** for true parallelism
- **No compile-time safety** checks

## Key Differences from Python

| Aspect | Python | Rust |
|--------|--------|------|
| Async runtime | Built-in (asyncio) | Choose your runtime |
| Future overhead | Higher (GC, allocations) | Zero-cost (state machines) |
| Thread parallelism | Blocked by GIL | True parallelism |
| Safety guarantees | Runtime checks | Compile-time (Send/Sync) |
| Async syntax | async/await | async/await |
| Explicit executor | No (`asyncio.run`) | Yes (`block_on`, `spawn`) |
| Data race prevention | None | Enforced by type system |

## Key Takeaways

1. **Async is lazy** - Futures do nothing until polled, unlike Python coroutines
2. **Choose your runtime** - Tokio, async-std, or custom, not built-in like asyncio
3. **Send and Sync** - Compile-time thread safety, preventing data races
4. **Zero-cost abstraction** - Async compiles to state machines with no overhead
5. **No GIL** - True parallelism for CPU-bound work with threads
6. **Explicit control** - You see and control the async machinery
7. **Type-safe concurrency** - The compiler prevents race conditions

Rust's async model requires more upfront understanding than Python's, but rewards you with better performance and compile-time safety guarantees. The lack of a GIL means threads actually provide parallelism, and the Send/Sync traits prevent entire classes of concurrency bugs that plague Python programs.
