# HTTP Service Skeleton

## Context

You are building a web service in Rust and need a foundation that handles HTTP requests, routes them to appropriate handlers, and manages the server lifecycle. You want compile-time safety, async I/O, and efficient request handling without the overhead of garbage collection or runtime reflection.

## Problem

**How do you structure a web service that is type-safe, efficient, and maintainable while handling concurrent HTTP requests?**

Creating a web service requires coordinating multiple concerns:
- Binding to network sockets and accepting connections
- Routing requests to appropriate handlers based on path and method
- Managing concurrent requests efficiently
- Ensuring type safety throughout the request/response cycle
- Handling server lifecycle (startup, graceful shutdown)

## Forces

- **Type Safety vs Flexibility**: Dynamic routing is flexible but error-prone; static typing catches errors at compile time but may seem rigid
- **Performance vs Ease of Use**: Low-level async primitives are fast but complex; high-level frameworks are ergonomic but may add overhead
- **Concurrency Model**: Thread-per-request is simple but doesn't scale; async I/O is efficient but requires careful state management
- **Startup Configuration**: Services need configuration before handling requests, but configuration logic shouldn't block request handling
- **Error Handling**: Server errors (binding failures) must be fatal, but request errors should be isolated and recoverable

## Solution

**Use actix-web's `HttpServer` and `App` to create a type-safe, async web service skeleton with explicit routing.**

The pattern has three key components:

### 1. Async Main Function with Runtime

```rust
#[actix_web::main]
async fn main() {
    // Service setup code runs once at startup
    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(get_index))
            .route("/gcd", web::post().to(post_gcd))
    });

    println!("Serving on http://localhost:3000...");
    server
        .bind("127.0.0.1:3000").expect("error binding server to address")
        .run()
        .await
        .expect("error running server");
}
```

**Why this works:**
- `#[actix_web::main]` macro sets up the Tokio async runtime
- `HttpServer::new()` takes a factory closure that creates `App` instances for each worker thread
- `.bind()` attempts socket binding and fails fast if the port is unavailable
- `.run().await` starts the server and blocks until shutdown

### 2. Application Factory Pattern

```rust
HttpServer::new(|| {
    App::new()
        .route("/", web::get().to(get_index))
        .route("/gcd", web::post().to(post_gcd))
})
```

The closure is called **once per worker thread**, allowing:
- Each worker to have its own `App` instance
- Shared state to be cloned efficiently (see **Application State** pattern)
- Route configuration to remain immutable during request handling

### 3. Route Registration with Method Guards

```rust
.route("/", web::get().to(get_index))
.route("/gcd", web::post().to(post_gcd))
```

Routes are **statically typed** at compile time:
- Path pattern (string literal for static paths, or `"/users/{id}"` for dynamic)
- HTTP method guard (`web::get()`, `web::post()`, etc.)
- Handler function (must return `impl Responder`)

The type system ensures:
- Only valid responders are registered
- Handler signatures match extractor types
- Route conflicts are detected (though not at compile time, unfortunately)

## Resulting Context

### Benefits

1. **Compile-Time Route Validation**: Handler signatures are checked against extractors
2. **Efficient Concurrency**: Async I/O with work-stealing scheduler handles thousands of concurrent connections
3. **Resource Safety**: Rust's ownership prevents data races and memory leaks
4. **Explicit Error Handling**: Startup failures (binding) vs runtime failures (bad requests) are distinguished
5. **Zero-Cost Abstractions**: The framework compiles to efficient machine code with no runtime overhead

### Liabilities

1. **Binary Size**: Async runtime and web framework add ~1-2MB to binary size
2. **Compile Time**: Generic-heavy code can slow down compilation
3. **Learning Curve**: Understanding async/await, extractors, and lifetimes requires Rust experience
4. **Limited Dynamic Routing**: Routes must be known at compile time (though you can use catch-all patterns)

### Consequences

- **Server Lifecycle**: Errors during `.bind()` or `.run()` are unrecoverable; service must restart
- **Worker Threads**: By default, actix spawns one worker per CPU core; adjust with `.workers(n)`
- **Graceful Shutdown**: Actix handles SIGTERM/SIGINT automatically, finishing in-flight requests
- **State Management**: Shared state must be wrapped in `web::Data<T>` (see **Application State** pattern)

## Related Patterns

- **Form Handling**: Extract and validate request data with type-safe deserializers
- **Route Handlers**: Define handler functions with extractors for request data
- **Application State**: Share database pools, configuration, and services across handlers
- **Middleware**: Add cross-cutting concerns (logging, authentication) as layers
- **Error Handlers**: Customize response generation for errors

## Known Uses

From **actix-gcd** (`/home/user/rust-programming-examples/actix-gcd/src/main.rs`):

```rust
use actix_web::{web, App, HttpResponse, HttpServer};

#[actix_web::main]
async fn main() {
    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(get_index))
            .route("/gcd", web::post().to(post_gcd))
    });

    println!("Serving on http://localhost:3000...");
    server
        .bind("127.0.0.1:3000").expect("error binding server to address")
        .run()
        .await
        .expect("error running server");
}
```

This minimal skeleton demonstrates:
- Single entry point with async main
- Two routes with method guards (GET and POST)
- Error handling for binding failures
- No shared state (handlers are stateless)

**Real-World Extensions**:
- Add `.workers(4)` to control concurrency
- Use `.bind_rustls()` for HTTPS
- Add `.wrap(Logger::default())` for request logging
- Chain multiple `App::new()` calls for different hosts or prefixes

## Implementation Notes

### Dependency Configuration

From `Cargo.toml`:
```toml
[dependencies]
actix-web = "4.1"
serde = { version = "1.0", features = ["derive"] }
```

Actix-web 4.x is built on Tokio 1.x and uses:
- `tokio` for async runtime
- `actix-rt` for actor-based work distribution
- `serde` for JSON/form deserialization

### Common Variants

**Multiple Apps per Server**:
```rust
HttpServer::new(|| {
    App::new()
        .service(
            web::scope("/api")
                .route("/users", web::get().to(get_users))
        )
        .service(
            web::scope("/admin")
                .route("/metrics", web::get().to(get_metrics))
        )
})
```

**Dynamic Configuration**:
```rust
let config = load_config();
HttpServer::new(move || {
    let cfg = config.clone();
    App::new()
        .app_data(web::Data::new(cfg))
        .route("/", web::get().to(get_index))
})
```

### Performance Characteristics

- **Startup**: ~1-10ms to bind socket and spawn workers
- **Request Latency**: ~10-50μs framework overhead per request
- **Throughput**: 100k+ requests/second on modern hardware (CPU-bound)
- **Memory**: ~10MB base + ~1KB per concurrent connection

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_index_get() {
        let app = test::init_service(
            App::new().route("/", web::get().to(get_index))
        ).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
```

## References

- **actix-web Documentation**: https://actix.rs/docs/
- **Tokio Runtime**: https://tokio.rs/
- **HTTP/1.1 Specification**: RFC 7230-7235
- **Rust Async Book**: https://rust-lang.github.io/async-book/
