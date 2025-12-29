# Async HTTP Request

## Context

You are building an application that needs to make HTTP requests—fetching data from web APIs, downloading resources, or communicating with remote services. Your application handles multiple concurrent operations, and you cannot afford to block threads waiting for network I/O.

You have an async runtime (async-std or tokio) available and understand **ASYNC/AWAIT BASICS**.

## Problem

**How do you make HTTP requests without blocking the current thread, allowing your application to handle many concurrent requests efficiently?**

HTTP requests involve multiple I/O operations: DNS lookup, TCP connection, writing the request, and reading the response. In a synchronous implementation, each of these steps blocks the thread. If you're handling many requests (like a web scraper or API gateway), blocking I/O limits scalability since each blocked thread consumes memory.

You need a solution that:
- Makes HTTP requests without blocking threads
- Handles network latency efficiently
- Allows concurrent requests to different hosts
- Provides clean error handling
- Integrates with your async runtime

## Forces

- **Network Latency**: HTTP requests can take hundreds of milliseconds. Blocking a thread for this duration wastes resources.
- **Connection Management**: TCP connections need to be established, managed, and closed properly, even in async contexts.
- **Protocol Complexity**: HTTP has headers, chunked encoding, redirects, and various edge cases.
- **Error Handling**: Network operations can fail in many ways (DNS failure, connection timeout, invalid response).
- **Abstraction Level**: Low-level TCP gives control but requires manual HTTP protocol handling. High-level clients are easier but less flexible.
- **Resource Usage**: Each blocked thread consumes stack space (typically 2MB). Async I/O allows thousands of concurrent requests with minimal memory.

## Solution

**Write an async function that performs HTTP requests using async I/O primitives, awaiting each network operation to yield control while waiting.**

Use async TCP sockets to connect, send the HTTP request, and read the response. Mark the function `async` and `await` each I/O operation. This allows the runtime to multiplex many concurrent requests on a small thread pool.

### Structure

```rust
async fn http_request(host: &str, port: u16, path: &str)
    -> std::io::Result<String>
{
    // 1. Establish async connection
    let mut connection = async_connect(host, port).await?;

    // 2. Send HTTP request
    let request = format_http_request(path, host);
    async_write(connection, request).await?;

    // 3. Read HTTP response
    let response = async_read_to_end(connection).await?;

    Ok(response)
}
```

### Real Example from cheapo-request

```rust
use async_std::io::prelude::*;
use async_std::net;

async fn cheapo_request(host: &str, port: u16, path: &str)
                            -> std::io::Result<String>
{
    // Connect to the server asynchronously
    // await yields control until connection is established
    let mut socket = net::TcpStream::connect((host, port)).await?;

    // Format HTTP/1.1 request
    let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);

    // Write request to socket asynchronously
    // await yields control until all bytes are written
    socket.write_all(request.as_bytes()).await?;

    // Signal we're done writing (half-close the connection)
    socket.shutdown(net::Shutdown::Write)?;

    // Read entire response asynchronously
    // await yields control while waiting for data
    let mut response = String::new();
    socket.read_to_string(&mut response).await?;

    Ok(response)
}
```

### Calling from Synchronous Code

```rust
fn main() -> std::io::Result<()> {
    use async_std::task;

    // block_on executes the async function to completion
    let response = task::block_on(cheapo_request("example.com", 80, "/"))?;
    println!("{}", response);
    Ok(())
}
```

### Key Mechanisms

1. **Async TCP Connection**:
   - `TcpStream::connect((host, port)).await?` doesn't block
   - DNS lookup and TCP handshake happen asynchronously
   - While waiting, the thread can execute other tasks

2. **Async Write**:
   - `write_all(...).await?` sends data without blocking
   - Handles partial writes automatically
   - Yields if the socket buffer is full

3. **Async Read**:
   - `read_to_string(...).await?` reads without blocking
   - Yields while waiting for data from the network
   - Continues until connection is closed or error occurs

4. **Error Propagation**:
   - The `?` operator works across await boundaries
   - Network errors propagate up as `io::Error`
   - Clean error handling without callback nesting

### Low-Level vs. High-Level

This example uses low-level TCP sockets for educational purposes. In production, you would typically use **ASYNC HTTP CLIENT LIBRARY** like surf or reqwest, which handle HTTP protocol details, connection pooling, TLS, redirects, etc.

```rust
// Low-level (manual HTTP protocol)
let mut socket = net::TcpStream::connect((host, port)).await?;
socket.write_all(b"GET / HTTP/1.1\r\n...").await?;

// High-level (protocol handled by library)
let response = surf::get("https://example.com").await?;
```

## Resulting Context

Your HTTP request code now:

- **Doesn't block threads**: While waiting for network I/O, the thread can execute other tasks
- **Scales efficiently**: Can handle thousands of concurrent requests with minimal memory
- **Reads sequentially**: Code follows a clear connect → write → read flow
- **Handles errors cleanly**: Uses standard `Result` error handling with `?`
- **Composes with other async operations**: Can be combined with **CONCURRENT FUTURES** to make multiple requests in parallel

However, you've introduced new considerations:

- **Protocol handling**: This implementation is minimal and doesn't handle HTTP features like chunked encoding, compression, redirects, or HTTPS
- **Connection reuse**: Each request creates a new connection; production code should use connection pooling
- **Timeouts**: No timeout mechanism—a slow server could delay indefinitely
- **Resource cleanup**: Connections should be closed properly even on errors

For production HTTP clients, use **ASYNC HTTP CLIENT LIBRARY** which handles these concerns.

When you need to make multiple HTTP requests concurrently (not sequentially), use **CONCURRENT FUTURES** to start them all at once and collect results.

## Related Patterns

- **ASYNC/AWAIT BASICS**: Foundation for writing async functions
- **CONCURRENT FUTURES**: Making multiple HTTP requests in parallel
- **ASYNC HTTP CLIENT LIBRARY**: Using higher-level HTTP clients like surf or reqwest
- **ASYNC MAIN**: Bridging synchronous code to async runtime

## Known Uses

- **Web scrapers**: Tools that fetch many pages concurrently use async HTTP to avoid blocking on each request
- **API clients**: Applications that call multiple microservices use async HTTP to make concurrent requests
- **Webhook handlers**: Services that send HTTP callbacks use async to avoid blocking worker threads
- **Monitoring tools**: Health checkers that ping many endpoints concurrently
- **Proxy servers**: HTTP proxies use async I/O to forward requests without dedicating threads per connection

Production implementations:
- **reqwest**: Popular async HTTP client built on hyper and tokio
- **surf**: Async HTTP client with a friendly API, built on async-std
- **hyper**: Low-level HTTP library that powers many higher-level clients
- **actix-web**: Web framework that uses async HTTP for both client and server

The pattern demonstrated here (manual HTTP protocol over async TCP) is primarily educational. It shows how async I/O works at a fundamental level. Production code should use battle-tested HTTP client libraries that handle protocol edge cases, security, and performance optimizations.
