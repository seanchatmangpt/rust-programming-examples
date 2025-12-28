# Async API Reference

This reference provides complete API documentation for asynchronous programming examples, from custom executors to concurrent servers.

## block_on - Custom Async Executor

A minimal implementation of an async executor that blocks the current thread until a future completes.

### Function API

#### `block_on`

Executes a future to completion on the current thread.

**Signature:**
```rust
pub fn block_on<F: Future>(future: F) -> F::Output
```

**Type Parameters:**
| Parameter | Bounds | Description |
|-----------|--------|-------------|
| `F` | `Future` | Type of future to execute |

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `future` | `F` | Future to execute to completion |

**Returns:**
- `F::Output`: The value produced by the future

**Implementation Details:**
- Uses `crossbeam::sync::Parker` for thread parking
- Creates a waker that unparks the thread
- Polls the future in a loop until ready

**Example:**
```rust
use block_on::block_on;

let result = block_on(async {
    42
});
assert_eq!(result, 42);
```

**With Async Operations:**
```rust
use async_std::task::sleep;
use std::time::Duration;

let result = block_on(async {
    sleep(Duration::from_millis(100)).await;
    "done"
});
assert_eq!(result, "done");
```

**Dependencies:**
| Crate | Version | Purpose |
|-------|---------|---------|
| `waker-fn` | 1.1 | Create waker from closure |
| `futures-lite` | 1.11 | `pin!` macro |
| `crossbeam` | 0.8 | Thread parking |

---

### Algorithm

**Execution Flow:**
1. Create a `Parker` for the current thread
2. Create a waker that unparks the thread when called
3. Pin the future to the stack
4. Loop:
   - Poll the future with the waker context
   - If `Poll::Ready(value)`, return the value
   - If `Poll::Pending`, park the thread (wait for waker)

**Waker Behavior:**
- Waker is cloned and passed to async operations
- When async operation completes, it calls `waker.wake()`
- This unparks the thread, allowing the loop to continue

**Complexity:**
- Time: Depends on future execution time
- Space: O(1) for executor state + O(n) for future state

---

### Usage Patterns

**Simple Async Value:**
```rust
let value = block_on(std::future::ready(42));
```

**Async I/O:**
```rust
use async_std::fs;

let content = block_on(async {
    fs::read_to_string("file.txt").await
})?;
```

**Racing Futures:**
```rust
use async_std::task::{spawn, sleep};
use futures_lite::FutureExt;
use std::time::Duration;

let result = block_on({
    let fast = async {
        sleep(Duration::from_millis(100)).await;
        "fast"
    };
    let slow = async {
        sleep(Duration::from_secs(1)).await;
        "slow"
    };
    spawn(fast.race(slow))
});
assert_eq!(result, "fast");
```

---

## spawn_blocking - Async Bridge to Blocking Code

Runs blocking code on a background thread and returns a future that yields the result.

### Types

#### `SpawnBlocking<T>`

Future that represents a blocking operation running on a background thread.

**Definition:**
```rust
pub struct SpawnBlocking<T>(Arc<Mutex<Shared<T>>>)
```

**Type Parameters:**
| Parameter | Bounds | Description |
|-----------|--------|-------------|
| `T` | `Send + 'static` | Type of value produced by blocking code |

**Shared State:**
```rust
struct Shared<T> {
    value: Option<T>,      // Result of blocking operation
    waker: Option<Waker>,  // Waker to notify when complete
}
```

---

### Function API

#### `spawn_blocking`

Spawns a blocking operation on a background thread.

**Signature:**
```rust
pub fn spawn_blocking<T, F>(closure: F) -> SpawnBlocking<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
```

**Type Parameters:**
| Parameter | Bounds | Description |
|-----------|--------|-------------|
| `F` | `FnOnce() -> T` | Closure to run |
| `F` | `Send + 'static` | Can be sent to thread |
| `T` | `Send + 'static` | Result can be sent back |

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `closure` | `F` | Blocking operation to execute |

**Returns:**
- `SpawnBlocking<T>`: Future that yields `T` when complete

**Behavior:**
1. Creates shared state (`Arc<Mutex<Shared<T>>>`)
2. Spawns a new thread
3. Thread executes the closure
4. Thread stores result and wakes the future (if polled)
5. Future yields result when polled

**Example:**
```rust
use spawn_blocking::spawn_blocking;

let result = block_on(async {
    spawn_blocking(|| {
        // Blocking operation
        std::thread::sleep(std::time::Duration::from_millis(100));
        42
    }).await
});
assert_eq!(result, 42);
```

---

### Future Implementation

#### `Future` for `SpawnBlocking<T>`

**Signature:**
```rust
impl<T: Send> Future for SpawnBlocking<T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T>
}
```

**Poll Behavior:**
| State | Action | Return |
|-------|--------|--------|
| Value ready | Take value from shared state | `Poll::Ready(value)` |
| Value not ready | Store waker in shared state | `Poll::Pending` |

**Thread Safety:**
- Uses `Mutex` to synchronize access to shared state
- `Arc` allows both thread and future to access state
- Waker is cloned when stored

---

### Usage Patterns

**CPU-Intensive Work:**
```rust
let result = spawn_blocking(|| {
    // Expensive computation
    (0..1_000_000).sum::<u64>()
}).await;
```

**Blocking I/O:**
```rust
let content = spawn_blocking(|| {
    std::fs::read_to_string("large_file.txt")
}).await?;
```

**Legacy APIs:**
```rust
let connection = spawn_blocking(|| {
    // Blocking database connection
    create_blocking_db_connection()
}).await;
```

---

## echo-server - Multi-threaded Echo Server

TCP server that echoes back all received data, spawning a thread per connection.

### Function API

#### `echo_main`

Runs a TCP echo server that spawns a thread for each client.

**Signature:**
```rust
fn echo_main(addr: &str) -> io::Result<()>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `addr` | `&str` | Socket address to bind (e.g., `"127.0.0.1:17007"`) |

**Returns:**
- `io::Result<()>`: Never returns on success (infinite loop)
- `Err(io::Error)`: Binding or accept error

**Behavior:**
1. Binds `TcpListener` to the specified address
2. Prints "listening on {addr}"
3. Infinite loop:
   - Accepts incoming connection
   - Prints "connection received from {addr}"
   - Spawns thread to handle connection
   - Thread echoes data until client disconnects

**Thread-Per-Connection Model:**
```rust
spawn(move || {
    io::copy(&mut stream, &mut write_stream)
        .expect("error in client thread: ");
    println!("connection closed");
});
```

**Example:**
```rust
use std::io;

fn main() -> io::Result<()> {
    echo_main("127.0.0.1:17007")
}
```

---

### Connection Handling

**Per-Connection Thread:**
- Clones the `TcpStream` for reading and writing
- Uses `io::copy` to echo data
- Thread terminates when connection closes

**Error Handling:**
- Panics on `io::copy` error (example code)
- Production code should handle errors gracefully

**Resource Usage:**
- Creates one OS thread per connection
- Each thread has ~2MB stack overhead
- Suitable for moderate connection counts (<1000)

---

### Client Interaction

**Example Client:**
```bash
$ nc localhost 17007
hello
hello
world
world
^C
```

**Telnet:**
```bash
$ telnet localhost 17007
Trying 127.0.0.1...
Connected to localhost.
test
test
```

---

## many-requests - Concurrent HTTP Requests

Demonstrates concurrent HTTP requests using async tasks.

### Function API

#### `cheapo_request`

Performs a simple HTTP GET request over TCP.

**Signature:**
```rust
async fn cheapo_request(host: &str, port: u16, path: &str)
    -> std::io::Result<String>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `host` | `&str` | Hostname |
| `port` | `u16` | TCP port |
| `path` | `&str` | HTTP path |

**Returns:**
- `Ok(String)`: Full HTTP response
- `Err(io::Error)`: Network error

**See Also:** [cheapo-request in Networking API](04-networking-api.md#cheapo_request)

---

#### `many_requests`

Executes multiple HTTP requests concurrently.

**Signature:**
```rust
async fn many_requests(requests: Vec<(String, u16, String)>)
    -> Vec<std::io::Result<String>>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `requests` | `Vec<(String, u16, String)>` | List of (host, port, path) tuples |

**Returns:**
- `Vec<std::io::Result<String>>`: Results in the same order as input

**Concurrency Pattern:**
1. Create a handle for each request using `task::spawn_local`
2. Collect all handles into a vector
3. Await each handle sequentially
4. Return results in original order

**Example:**
```rust
let requests = vec![
    ("example.com".to_string(), 80, "/".to_string()),
    ("www.red-bean.com".to_string(), 80, "/".to_string()),
    ("en.wikipedia.org".to_string(), 80, "/".to_string()),
];

let results = many_requests(requests).await;
for result in results {
    match result {
        Ok(response) => println!("{}", response),
        Err(err) => eprintln!("error: {}", err),
    }
}
```

---

### Implementation Details

**Task Spawning:**
```rust
for (host, port, path) in requests {
    handles.push(task::spawn_local(async move {
        cheapo_request(&host, port, &path).await
    }));
}
```

**Why `spawn_local`:**
- Spawns tasks on the current thread
- No `Send` requirement for captured values
- More efficient than `spawn` for single-threaded executor

**Collecting Results:**
```rust
let mut results = vec![];
for handle in handles {
    results.push(handle.await);
}
```

**Ordering:**
- Results are in the same order as requests
- Awaiting is sequential, but execution is concurrent
- First request to finish waits for earlier handles to be awaited

---

### Concurrency Patterns

#### Sequential Execution

```rust
// One at a time (slow)
for url in urls {
    let response = fetch(url).await;
}
```

#### Concurrent with spawn

```rust
// All at once (fast)
let handles: Vec<_> = urls.into_iter()
    .map(|url| task::spawn_local(fetch(url)))
    .collect();

for handle in handles {
    let response = handle.await;
}
```

#### Join/Select

```rust
// Wait for all
futures::future::join_all(futures).await;

// Wait for first
futures::future::select_all(futures).await;
```

---

## Async Patterns Comparison

### Executor Implementations

| Feature | block_on | async-std | tokio |
|---------|----------|-----------|-------|
| Thread pool | No | Yes | Yes |
| Work stealing | No | Yes | Yes |
| Multi-threaded | No | Yes | Yes |
| Timer support | No | Yes | Yes |
| I/O reactor | No | Yes | Yes |

### Blocking Bridge Patterns

| Pattern | Use Case | Overhead |
|---------|----------|----------|
| `spawn_blocking` | CPU-bound work | Thread spawn |
| Thread pool | Reusable threads | Pool maintenance |
| Rayon | Data parallelism | Work stealing |

### Server Architectures

| Model | Example | Scalability | Overhead |
|-------|---------|-------------|----------|
| Thread-per-connection | echo-server | Low (<1000 conn) | ~2MB per conn |
| Async event loop | Actix, Tokio | High (10k+ conn) | ~4KB per task |
| Hybrid | Thread pool + async | Medium | Configurable |

---

## Performance Considerations

### block_on

**Best For:**
- Test code
- Simple scripts
- Integrating async into sync code

**Avoid:**
- Production servers
- Nested event loops
- Long-running services

---

### spawn_blocking

**Best For:**
- CPU-intensive computation
- Blocking I/O (files, databases)
- Legacy blocking APIs

**Overhead:**
- Thread spawn: ~1ms
- Thread pool (production): ~100µs
- Context switch: ~1-10µs

---

### echo-server

**Scalability:**
- Good: <100 connections
- Acceptable: 100-1000 connections
- Poor: >1000 connections

**Memory:**
- Per connection: ~2MB (thread stack)
- 1000 connections: ~2GB

---

### many-requests

**Concurrency:**
- Limited by file descriptors
- Typical limit: 1024 (ulimit)
- Can be increased: `ulimit -n 65536`

**Performance:**
- Network-bound: high concurrency helps
- CPU-bound: limit concurrency to CPU count

---

## Error Handling

### Future Errors

```rust
// Option propagation
let value = future.await?;

// Result propagation
let response = request(url).await?;

// Pattern matching
match future.await {
    Ok(value) => { /* ... */ },
    Err(e) => { /* ... */ },
}
```

### Panic Handling

```rust
// spawn_blocking panic = thread panic
let handle = spawn_blocking(|| panic!("oops"));
// Panics when awaited: handle.await

// Catching panics
use std::panic::catch_unwind;
let result = spawn_blocking(|| {
    catch_unwind(|| {
        // Might panic
    })
}).await;
```

### Cancellation

**Future Drop:**
```rust
let future = long_operation();
drop(future);  // Cancels the operation
```

**Timeout:**
```rust
use async_std::future::timeout;
use std::time::Duration;

let result = timeout(
    Duration::from_secs(5),
    long_operation()
).await;
```

---

## Dependencies Summary

| Project | Runtime | Key Crates | Purpose |
|---------|---------|------------|---------|
| block_on | None (custom) | crossbeam, waker-fn | Learning |
| spawn_blocking | None (std::thread) | (none) | Demonstration |
| echo-server | None (sync) | (none) | Multi-threading |
| many-requests | async-std | async-std | Concurrency |

## Further Reading

- **Custom executors:** See `block_on` implementation
- **Async I/O:** See `cheapo_request` for raw TCP
- **Concurrency:** See `many_requests` for task spawning
- **Threading:** See `echo_server` for thread-per-connection model
