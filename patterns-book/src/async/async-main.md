# Async Main

## Context

You are writing an application that needs to perform asynchronous operations. You understand **ASYNC/AWAIT BASICS** and can write async functions, but you're faced with a fundamental problem: Rust's `main` function is synchronous and cannot be marked `async`.

```rust
// This doesn't work!
async fn main() {  // ERROR: main cannot be async
    make_request().await;
}
```

You need a way to bridge from the synchronous entry point (`main`) to your asynchronous code.

## Problem

**How do you structure your program's entry point to execute async code when `main` must be synchronous?**

The `main` function is the entry point of every Rust program, and by default it must be synchronous. But async functions return `Future`s that don't execute until awaited or polled. You cannot simply call an async function from `main` without some way to execute it.

```rust
fn main() {
    let future = async_operation();  // Returns a Future
    // But the future doesn't run! How do we execute it?
}
```

You need a mechanism to:
- Bridge from synchronous `main` to async code
- Initialize and run the async runtime
- Block `main` until async operations complete
- Propagate errors from async code to the process exit status
- Handle cleanup properly when async work finishes

## Forces

- **Synchronous Entry**: Operating systems call `main` as a regular function; it cannot be async.
- **Runtime Initialization**: Async code requires a runtime (executor, reactor) to be initialized before execution.
- **Blocking vs. Non-blocking**: `main` must block until the program's work is complete, but async code is built around non-blocking operations.
- **Error Propagation**: Errors from async operations should propagate to `main` and affect the process exit code.
- **Resource Cleanup**: The runtime should shut down cleanly when async work completes.
- **Simplicity**: The bridging code should be minimal and not obscure the program's logic.

## Solution

**Use the runtime's `block_on` function to execute an async function to completion, blocking the main thread until it finishes.**

The `block_on` function (or macro like `#[tokio::main]`) initializes the async runtime and runs an async function, blocking the calling thread until the future completes. This bridges the synchronous `main` to async code.

### Structure

```rust
fn main() -> Result<(), Error> {
    // Initialize runtime and run async code to completion
    runtime::block_on(async_main())?;
    Ok(())
}

async fn async_main() -> Result<(), Error> {
    // All your async code goes here
    async_operation().await?;
    Ok(())
}
```

### Real Example from cheapo-request

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

fn main() -> std::io::Result<()> {
    use async_std::task;

    // block_on bridges sync main to async code
    let response = task::block_on(cheapo_request("example.com", 80, "/"))?;
    println!("{}", response);
    Ok(())
}
```

### Real Example from many-requests

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

fn main() {
    let requests = vec![
        ("example.com".to_string(),      80, "/".to_string()),
        ("www.red-bean.com".to_string(), 80, "/".to_string()),
        ("en.wikipedia.org".to_string(), 80, "/".to_string()),
    ];

    // block_on runs the async function and waits for completion
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

1. **Runtime Initialization**:
   ```rust
   task::block_on(future)
   ```
   - Initializes the async runtime (executor, reactor, thread pool)
   - Creates the initial task from the provided future
   - Starts the event loop

2. **Blocking Execution**:
   - `block_on` blocks the calling thread (main thread)
   - The thread waits until the future completes
   - Other async tasks can run concurrently during this time

3. **Result Propagation**:
   ```rust
   fn main() -> std::io::Result<()> {
       let result = task::block_on(async_operation())?;
       Ok(())
   }
   ```
   - Return type of `main` can be `Result`
   - Errors from async code propagate via `?`
   - Non-zero exit code on error

4. **Runtime Shutdown**:
   - When the future completes, `block_on` returns
   - Runtime shuts down and cleans up resources
   - `main` exits normally

### Alternative: Macro Attribute

Some runtimes provide a macro to transform `main` into async:

**tokio**:
```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Can directly await here
    async_operation().await?;
    Ok(())
}

// Expands to:
fn main() -> Result<(), Error> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            async_operation().await?;
            Ok(())
        })
}
```

**async-std**:
```rust
#[async_std::main]
async fn main() -> std::io::Result<()> {
    cheapo_request("example.com", 80, "/").await?;
    Ok(())
}

// Expands to:
fn main() -> std::io::Result<()> {
    async_std::task::block_on(async {
        cheapo_request("example.com", 80, "/").await?;
        Ok(())
    })
}
```

### Explicit vs. Macro

**Explicit `block_on`** (used in examples):
```rust
fn main() {
    let result = async_std::task::block_on(async_main());
}
```

**Pros**:
- Explicit about what's happening
- No magic macros
- Easy to debug
- Clear separation of sync and async

**Cons**:
- Slightly more verbose
- Extra function or closure needed

**Macro attribute** (`#[tokio::main]`, `#[async_std::main]`):
```rust
#[tokio::main]
async fn main() {
    async_operation().await;
}
```

**Pros**:
- Concise, less boilerplate
- `main` looks async directly
- Standard pattern in tokio projects

**Cons**:
- Macro magic obscures what's happening
- Harder to debug macro expansion
- Less explicit about runtime initialization

Choose based on your preference and project conventions. The examples in this repository use explicit `block_on` for clarity.

### Error Handling Patterns

**Pattern 1: Propagate all errors**
```rust
fn main() -> std::io::Result<()> {
    task::block_on(async {
        operation1().await?;
        operation2().await?;
        Ok(())
    })
}
```

**Pattern 2: Handle some errors, propagate others**
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    task::block_on(async {
        match risky_operation().await {
            Ok(val) => println!("Success: {}", val),
            Err(e) => eprintln!("Warning: {}", e),
        }
        critical_operation().await?;  // Propagate this one
        Ok(())
    })
}
```

**Pattern 3: Exit with specific code**
```rust
fn main() {
    if let Err(e) = task::block_on(async_main()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

## Resulting Context

Your program entry point now:

- **Bridges sync to async**: `main` is synchronous, but can run async code
- **Initializes runtime**: Async executor and reactor are set up automatically
- **Blocks appropriately**: Main thread waits for async work to complete before exiting
- **Propagates errors**: Async errors flow to `main` and affect exit status
- **Cleans up resources**: Runtime shuts down properly when async work finishes

However, you've introduced new considerations:

- **Single entry point**: `block_on` typically runs one top-level future; for multiple independent operations, structure them within that future
- **Runtime overhead**: Initializing the runtime has a small startup cost
- **Blocking main thread**: While async code runs, main thread is blocked (but this is necessary for program lifecycle)
- **Cannot call block_on from async**: `block_on` is only for bridging from sync to async, not within async code (would deadlock)

When you need to run multiple independent async operations, use **CONCURRENT FUTURES** within the async context:

```rust
fn main() {
    task::block_on(async {
        // Use CONCURRENT FUTURES here
        let handles = vec![
            task::spawn(operation1()),
            task::spawn(operation2()),
        ];
        for h in handles {
            h.await;
        }
    });
}
```

## Related Patterns

- **ASYNC/AWAIT BASICS**: Foundation for writing async functions that `block_on` executes
- **CONCURRENT FUTURES**: Running multiple async operations within the async context
- **SPAWN LOCAL**: Spawning local tasks within the async context created by `block_on`

## Known Uses

- **CLI tools**: Command-line applications use `block_on` to run async operations to completion
- **Batch processors**: Scripts that perform async I/O use `block_on` for the main workflow
- **Servers**: Web servers often use the macro form (`#[tokio::main]`) for async main
- **Testing**: Async tests use similar patterns (e.g., `#[async_std::test]`, `#[tokio::test]`)

Real-world examples:

**async-std examples**:
```rust
fn main() {
    async_std::task::block_on(async {
        // Application logic
    })
}
```

**tokio examples**:
```rust
#[tokio::main]
async fn main() {
    // Application logic
}
```

**actix-web server**:
```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new())
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
```

**async tests**:
```rust
#[async_std::test]
async fn test_async_operation() {
    let result = async_operation().await;
    assert_eq!(result, expected);
}
```

Common patterns across projects:

1. **Simple CLI** (cheapo-request pattern):
   ```rust
   fn main() -> Result<(), Error> {
       task::block_on(async {
           let result = single_operation().await?;
           println!("{}", result);
           Ok(())
       })
   }
   ```

2. **Multiple operations** (many-requests pattern):
   ```rust
   fn main() {
       let results = task::block_on(many_operations(inputs));
       process_results(results);
   }
   ```

3. **Server/daemon**:
   ```rust
   #[tokio::main]
   async fn main() -> Result<(), Error> {
       let server = Server::new();
       server.run().await?;  // Runs until shutdown signal
       Ok(())
   }
   ```

The ASYNC MAIN pattern is universal in Rust async programming—every async application needs some way to bridge from the synchronous entry point to async code. Whether you use explicit `block_on` or a macro attribute, the underlying mechanism is the same: initialize the runtime, run the top-level future, and block until completion.
