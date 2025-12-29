# Async HTTP Client Library

## Context

You need to make HTTP requests in an async application. You've seen **ASYNC HTTP REQUEST** which demonstrates manual HTTP protocol handling over TCP sockets, but you recognize that production code needs to handle HTTPS, redirects, compression, connection pooling, timeouts, and other HTTP complexities.

You want a battle-tested library that provides a clean async API without requiring you to implement HTTP protocol details.

## Problem

**How do you make production-ready HTTP requests asynchronously without implementing the HTTP protocol yourself?**

Manual HTTP implementation over raw TCP sockets (as shown in **ASYNC HTTP REQUEST**) works for simple cases but has significant limitations:

- No HTTPS support (TLS encryption)
- No redirect following (3xx responses)
- No compression (gzip, deflate)
- No cookie handling
- No connection pooling (new connection per request)
- No request/response streaming
- No timeout mechanism
- Manual header parsing and construction
- No support for HTTP/2 or HTTP/3

Implementing these features correctly is complex and error-prone. You need a library that:
- Handles the full HTTP protocol correctly
- Provides an async API that integrates with your runtime
- Manages connections efficiently
- Offers ergonomic request building and response parsing
- Handles errors comprehensively

## Forces

- **Abstraction Level**: Higher-level APIs are easier to use but may hide important details. Lower-level APIs give control but require more code.
- **Runtime Compatibility**: Different HTTP clients are built for different async runtimes (tokio vs. async-std).
- **Feature Set**: Simple requests are easy with any client, but advanced features (HTTP/2, WebSockets, custom middleware) vary by library.
- **Performance**: Connection pooling, HTTP/2 multiplexing, and efficient parsing affect throughput and latency.
- **Type Safety**: Strong typing prevents errors but can make the API more verbose.
- **Error Handling**: HTTP has many failure modes; the library should provide detailed error types.
- **Dependency Weight**: HTTP libraries pull in many dependencies (TLS, compression, parsing). Bundle size matters for some applications.

## Solution

**Use a high-level async HTTP client library like `surf` or `reqwest` that handles protocol details and provides an ergonomic async API.**

These libraries abstract away protocol complexity, provide connection pooling, handle TLS, and offer clean APIs for building requests and parsing responses. They integrate seamlessly with async/await.

### Structure

```rust
async fn make_requests(urls: &[String]) -> Vec<Result<String, Error>> {
    let client = HttpClient::new();

    let mut handles = vec![];
    for url in urls {
        let request = client.get(url).recv_string();
        handles.push(spawn_task(request));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}
```

### Real Example from many-requests-surf

```rust
pub async fn many_requests(urls: &[String])
                           -> Vec<Result<String, surf::Exception>>
{
    // Create a reusable client (enables connection pooling)
    let client = surf::Client::new();

    let mut handles = vec![];

    for url in urls {
        // Build request and start receiving response body as string
        let request = client.get(&url).recv_string();

        // Spawn task on thread pool (not spawn_local)
        // surf types are Send, allowing multi-threaded execution
        handles.push(async_std::task::spawn(request));
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
    let requests = &[
        "http://example.com".to_string(),
        "https://www.red-bean.com".to_string(),
        "https://en.wikipedia.org/wiki/Main_Page".to_string()
    ];

    let results = async_std::task::block_on(many_requests(requests));

    for result in results {
        match result {
            Ok(response) => println!("*** {}\n", response),
            Err(err) => eprintln!("error: {}\n", err),
        }
    }
}
```

### Key Mechanisms

1. **Client Creation**:
   ```rust
   let client = surf::Client::new();
   ```
   - Reusable client for connection pooling
   - Shares connection pool across requests to same host
   - Can be cloned cheaply (Arc internally)

2. **Request Building**:
   ```rust
   let request = client.get(&url).recv_string();
   ```
   - Fluent API for building requests
   - `get()`, `post()`, `put()`, `delete()` methods
   - `recv_string()` reads response body as String
   - Can chain additional methods: `.header()`, `.body()`, `.timeout()`

3. **Multi-threaded Spawning**:
   ```rust
   async_std::task::spawn(request)
   ```
   - Uses `spawn` (not `spawn_local`) for thread pool execution
   - surf's types implement `Send`, allowing true parallelism
   - Multiple requests can execute on different CPU cores

4. **Error Handling**:
   ```rust
   Result<String, surf::Exception>
   ```
   - Comprehensive error type covering network, protocol, and parsing errors
   - Includes status codes for HTTP errors (4xx, 5xx)

### Comparison: Manual vs. Library

**Manual HTTP (from cheapo-request)**:
```rust
let mut socket = net::TcpStream::connect((host, port)).await?;
let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
socket.write_all(request.as_bytes()).await?;
socket.shutdown(net::Shutdown::Write)?;
let mut response = String::new();
socket.read_to_string(&mut response).await?;
```

**Library (surf)**:
```rust
let response = client.get(url).recv_string().await?;
```

The library version:
- Automatically handles HTTPS (TLS)
- Follows redirects
- Decompresses response bodies
- Parses headers and status codes
- Reuses connections via pooling
- Provides proper timeout handling

### Advanced Features

```rust
// Custom headers
let response = client
    .get("https://api.example.com")
    .header("Authorization", "Bearer token")
    .header("Accept", "application/json")
    .recv_string()
    .await?;

// POST request with JSON body
let response = client
    .post("https://api.example.com/items")
    .body_json(&json_data)?
    .recv_json()  // Parse response as JSON
    .await?;

// Timeouts
use std::time::Duration;
let response = client
    .get("https://slow-server.com")
    .timeout(Duration::from_secs(10))
    .recv_string()
    .await?;

// Response details
let mut response = client.get("https://example.com").await?;
let status = response.status();  // 200, 404, etc.
let headers = response.headers();
let body = response.body_string().await?;
```

### Popular Async HTTP Client Libraries

| Library | Runtime | Features | Use Case |
|---------|---------|----------|----------|
| **surf** | async-std | Simple API, connection pooling, middleware | General async-std projects |
| **reqwest** | tokio | Feature-rich, widely used, excellent docs | General tokio projects |
| **hyper** | tokio | Low-level, HTTP/1/2/3, powers reqwest | Building custom clients/servers |
| **awc** (actix-web) | actix | Integrated with actix ecosystem | actix-web applications |

### surf vs. reqwest Example

**surf (async-std)**:
```rust
let body = surf::get("https://example.com")
    .recv_string()
    .await?;
```

**reqwest (tokio)**:
```rust
let body = reqwest::get("https://example.com")
    .await?
    .text()
    .await?;
```

Very similar APIs, main difference is runtime compatibility.

## Resulting Context

Your HTTP client code now:

- **Handles HTTPS**: TLS encryption and certificate validation work automatically
- **Follows redirects**: 3xx responses are followed transparently
- **Pools connections**: Reuses TCP connections to the same host
- **Parses responses**: Headers, status codes, and body are easily accessible
- **Composes with async**: Integrates naturally with **CONCURRENT FUTURES** and other async patterns
- **Provides rich errors**: Detailed error types for network, protocol, and application errors

However, you've introduced new considerations:

- **Dependency weight**: HTTP client libraries have many dependencies (TLS, parser, compression)
- **Runtime binding**: Most clients are tied to a specific async runtime (tokio or async-std)
- **Learning curve**: Each library has its own API and configuration options
- **Version compatibility**: HTTP libraries evolve; major versions can have breaking changes
- **Debugging complexity**: When things go wrong, you're debugging library internals, not your code

For simple educational examples, **ASYNC HTTP REQUEST** with manual TCP sockets demonstrates async I/O fundamentals.

For production code, always use a library like surf or reqwest.

## Related Patterns

- **ASYNC HTTP REQUEST**: Low-level manual HTTP over TCP sockets (educational)
- **CONCURRENT FUTURES**: Run multiple HTTP requests in parallel
- **ASYNC/AWAIT BASICS**: Foundation for using async HTTP clients
- **SPAWN LOCAL**: Contrast with `spawn` used here for thread pool execution

## Known Uses

- **Web scrapers**: Tools like `spider` use async HTTP clients to fetch pages concurrently
- **API clients**: SDKs for REST APIs (GitHub, AWS, etc.) built on reqwest/surf
- **Monitoring**: Health checkers and uptime monitors use async clients to ping many endpoints
- **Proxies**: HTTP proxies use client libraries to forward requests asynchronously
- **Testing**: Integration tests use HTTP clients to verify API behavior
- **Webhooks**: Applications that send HTTP callbacks use async clients

Real-world applications:

- **cargo**: Uses `reqwest` for downloading crates and registry operations
- **rustup**: Uses async HTTP clients for downloading Rust toolchains
- **Cloud SDKs**: AWS SDK, Azure SDK use reqwest for API calls
- **Web frameworks**: actix-web, tide, warp all provide HTTP client functionality
- **GraphQL clients**: async-graphql-client, graphql-client built on async HTTP

Library ecosystem:

- **reqwest**: Most popular async HTTP client in Rust ecosystem
  - 10M+ downloads, extensive middleware, well-documented
  - Features: HTTP/1, HTTP/2, cookies, redirects, compression, streaming

- **surf**: Simpler async-std alternative to reqwest
  - Clean API, middleware support, built on async-h1/async-h2
  - Smaller community but well-maintained

- **hyper**: Low-level foundation for many higher-level clients
  - HTTP/1, HTTP/2, HTTP/3 support
  - Used internally by reqwest and many web frameworks

Production recommendations:
- **For tokio projects**: Use `reqwest`
- **For async-std projects**: Use `surf`
- **For custom protocols**: Consider `hyper` as a foundation
- **For actix projects**: Use `awc` (actix-web client)

The async HTTP client library you choose should match your async runtime and feature requirements. All major libraries provide similar core functionality (GET/POST, headers, JSON), so the decision often comes down to runtime compatibility and specific advanced features you need.
