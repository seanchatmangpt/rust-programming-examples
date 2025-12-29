# Async/Await Basics

## Context

You are writing I/O-bound code that needs to wait for operations like network requests, file reads, or database queries. You want to write asynchronous code that doesn't block the thread while waiting, but you don't want to deal with the complexity of explicit callback chains or manual state machines.

Your codebase uses an async runtime like async-std or tokio, and you need to write functions that can perform asynchronous operations efficiently.

## Problem

**How do you write asynchronous code that is both efficient (non-blocking) and easy to read (looks like synchronous code)?**

Traditional blocking I/O ties up an entire thread while waiting for operations to complete. This limits scalability since each blocked thread consumes memory and OS resources. Callback-based approaches avoid blocking but lead to "callback hell" where logic becomes fragmented across multiple closures, making code hard to follow and maintain.

You need a way to express asynchronous logic that:
- Yields control back to the runtime while waiting for I/O
- Maintains readable, sequential-looking code flow
- Handles errors naturally with `?` operator
- Composes easily with other async operations

## Forces

- **Readability vs. Efficiency**: Sequential code is easy to read, but blocking operations waste resources. Callbacks are efficient but hard to follow.
- **Error Handling**: Asynchronous operations can fail, and you want to use Rust's standard `Result` error handling, not scattered error callbacks.
- **Control Flow**: Complex logic with branching and loops is straightforward in synchronous code but becomes convoluted with callbacks.
- **Zero-Cost Abstraction**: The async abstraction should have minimal runtime overhead—ideally compiling to an efficient state machine.
- **Composability**: Async operations should compose naturally, allowing you to build complex operations from simple ones.

## Solution

**Use the `async`/`await` keywords to write asynchronous functions that look like synchronous code but compile to efficient, non-blocking state machines.**

Mark functions with `async` to indicate they return a `Future`. Use `await` to suspend execution at points where you're waiting for asynchronous operations. The compiler transforms this into a state machine that yields control to the runtime when waiting.

### Structure

```rust
async fn async_function(params: Types) -> Result<ReturnType, ErrorType> {
    // Await asynchronous operations
    let result = some_async_operation().await?;

    // Sequential code flow
    let processed = process(result);

    // Chain multiple async operations
    another_async_operation(processed).await?;

    Ok(final_result)
}
```

### Real Example from cheapo-request

```rust
use async_std::io::prelude::*;
use async_std::net;

async fn cheapo_request(host: &str, port: u16, path: &str)
                            -> std::io::Result<String>
{
    // Connect asynchronously - await suspends until connected
    let mut socket = net::TcpStream::connect((host, port)).await?;

    // Write request - await suspends until write completes
    let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
    socket.write_all(request.as_bytes()).await?;
    socket.shutdown(net::Shutdown::Write)?;

    // Read response - await suspends until all data is read
    let mut response = String::new();
    socket.read_to_string(&mut response).await?;

    Ok(response)
}
```

### Key Mechanisms

1. **The `async` Keyword**:
   - Transforms a function to return a `Future<Output = T>` instead of `T`
   - The function doesn't execute immediately—it returns a future that must be awaited or polled

2. **The `await` Keyword**:
   - Suspends execution at this point, yielding control to the runtime
   - When the awaited future completes, execution resumes from this point
   - Works seamlessly with the `?` operator for error propagation

3. **Compilation**:
   - The compiler generates a state machine with states for each await point
   - Local variables are captured in the future's state
   - No heap allocations needed for the state machine itself

4. **Sequential Appearance**:
   - Code reads top-to-bottom like synchronous code
   - Control flow (if, match, loops) works naturally
   - Error handling with `?` propagates through async boundaries

### Integration with Sync Code

Since `async fn` returns a `Future`, you need a runtime to execute it:

```rust
fn main() -> std::io::Result<()> {
    use async_std::task;

    // block_on runs the async function to completion
    let response = task::block_on(cheapo_request("example.com", 80, "/"))?;
    println!("{}", response);
    Ok(())
}
```

## Resulting Context

Your asynchronous code now:

- **Reads sequentially**: Follows a clear top-to-bottom flow that's easy to understand
- **Doesn't block threads**: While awaiting I/O, the thread can execute other tasks
- **Handles errors naturally**: The `?` operator works across await points
- **Composes easily**: Async functions can call other async functions with `await`
- **Zero-cost**: Compiles to an efficient state machine with no unnecessary allocations

However, you've introduced new considerations:

- **Execution model**: Async functions don't run until awaited or spawned
- **Runtime dependency**: Need an async runtime (async-std, tokio) to execute futures
- **Function coloring**: Async functions can only await other async functions, creating a division between sync and async code
- **Trait limitations**: Some traits (like `Read`, `Write`) have separate async versions

When you need to run multiple independent async operations concurrently (not just sequentially), you'll need **CONCURRENT FUTURES** to start them all simultaneously.

When you need to call async code from a synchronous context, you'll need **ASYNC MAIN** to bridge the gap.

## Related Patterns

- **ASYNC MAIN**: Bridges synchronous main() to asynchronous runtime
- **CONCURRENT FUTURES**: Runs multiple async operations in parallel
- **ASYNC HTTP REQUEST**: Specific application of async/await for HTTP
- **SPAWN LOCAL**: Spawns async tasks that don't need to be Send

## Known Uses

- **async-std**: The examples in this repository use async-std's async implementations of standard library types
- **tokio**: Popular async runtime with async versions of I/O primitives
- **HTTP clients**: reqwest, surf, hyper all use async/await for non-blocking requests
- **Database drivers**: sqlx, tokio-postgres use async/await for concurrent query execution
- **Web frameworks**: actix-web, warp, rocket (async version) all built on async/await
- **File I/O**: async-std and tokio provide async file operations for high-concurrency file servers

The async/await syntax was stabilized in Rust 1.39 (November 2019) and has become the standard way to write asynchronous code in Rust, replacing older approaches like explicit Future combinators and callback-based designs.
