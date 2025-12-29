# Web Service Architecture

Building web services in Rust requires careful architectural decisions that balance performance, safety, and developer ergonomics. This section examines a complete web service implementation, revealing how Rust's type system and ownership model create robust HTTP applications.

## The HTTP Request/Response Cycle

At its core, every web service implements the HTTP request/response cycle. In Rust, this cycle becomes a study in type-driven architecture, where the compiler enforces correct handling of resources across asynchronous boundaries.

Consider the fundamental structure of the `actix-gcd` web service:

```rust
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

This deceptively simple code embodies several critical architectural decisions:

**Factory Pattern for Thread Safety**: `HttpServer::new` accepts a closure that produces `App` instances. Why? Because actix-web runs multiple worker threads, each needing its own application instance. The closure ensures each worker gets an independent copy, preventing data races at compile time.

**Lazy Evaluation**: Routes are defined declaratively but not executed until requests arrive. The `.route()` method builds a routing table that maps HTTP methods and paths to handler functions. This separation of configuration from execution enables powerful optimization opportunities.

**Async-First Design**: The `#[actix_web::main]` macro transforms the synchronous-looking `main` function into an async runtime. Under the hood, it spawns a Tokio reactor that manages thousands of concurrent connections without blocking.

## Routing Architecture Patterns

Routing in web frameworks faces a fundamental tension: flexibility versus type safety. Dynamically typed languages excel at flexible routing but sacrifice compile-time guarantees. Rust's approach leverages the type system to provide both.

The routing architecture in actix-gcd demonstrates three key patterns:

```rust
App::new()
    .route("/", web::get().to(get_index))
    .route("/gcd", web::post().to(post_gcd))
```

**Type-Safe Method Matching**: `web::get()` and `web::post()` are not strings but typed route builders. This prevents runtime errors from typos like `"GET"` vs `"get"`. The type system ensures only valid HTTP methods exist.

**Path-to-Handler Binding**: The `.to()` method performs sophisticated type inference. It accepts any async function whose signature matches the handler pattern, automatically extracting request data based on parameter types. This is **extraction by type**, a powerful Rust idiom.

**Handler Signatures as Contracts**: Compare two handler signatures:

```rust
async fn get_index() -> HttpResponse {
    // No parameters: expects no request data
}

async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    // Parameters specify extraction: form data as GcdParameters
}
```

The compiler enforces these contracts. If a handler expects form data but receives JSON, the type mismatch is caught at compile time, not during a production request.

## Handler Architecture Deep Dive

Handlers are where business logic meets HTTP semantics. The `post_gcd` handler reveals sophisticated architectural patterns:

```rust
#[derive(Deserialize)]
struct GcdParameters {
    n: u64,
    m: u64,
}

async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Computing the GCD with zero is boring.");
    }

    let response = format!(
        "The greatest common divisor of the numbers {} and {} is <b>{}</b>\n",
        form.n, form.m, gcd(form.n, form.m)
    );

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}
```

**Automatic Deserialization**: The `web::Form<GcdParameters>` parameter triggers automatic form parsing. Actix-web examines the `Content-Type` header, parses the form data, and deserializes it into `GcdParameters`. If parsing fails, actix returns a 400 Bad Request automatically—no manual error handling needed.

**Builder Pattern for Responses**: `HttpResponse::BadRequest()` returns a builder, not a final response. The builder pattern allows incremental construction:

```rust
HttpResponse::BadRequest()
    .content_type("text/html")  // Builder method
    .body("...")                // Consumes builder, produces HttpResponse
```

This pattern prevents incomplete responses. The type system ensures you can't send a response without a body, and you can't add headers after setting the body.

**Early Returns for Error Paths**: The validation check uses early return:

```rust
if form.n == 0 || form.m == 0 {
    return HttpResponse::BadRequest()...;
}
```

This "fail fast" pattern keeps the happy path unindented, improving readability. In larger handlers, this prevents rightward drift from nested error handling.

## Integration with Web Frameworks

Actix-web's architecture demonstrates how Rust frameworks leverage advanced type system features. Three integration patterns stand out:

**Extractor Pattern**: The framework defines the `FromRequest` trait, which types implement to extract data from requests:

```rust
// Simplified illustration (not actual code)
trait FromRequest {
    async fn from_request(req: &HttpRequest) -> Result<Self, Error>;
}
```

Types like `web::Form<T>`, `web::Json<T>`, and `web::Path<T>` implement this trait. Handlers simply declare parameters of these types, and the framework calls the appropriate extractor automatically. This is **dependency injection via types**.

**Zero-Cost Abstraction**: Despite the sophisticated routing and extraction, actix-web compiles to extremely efficient code. The routing table becomes a compile-time data structure. Handler dispatching uses static dispatch (generics), not dynamic dispatch (trait objects), eliminating virtual function call overhead.

**Middleware Composition**: Though not shown in this minimal example, actix-web supports middleware through the `wrap()` method:

```rust
App::new()
    .wrap(middleware::Logger::default())
    .wrap(middleware::Compress::default())
    .route("/", web::get().to(handler))
```

Middleware forms a chain, with each layer wrapping the next. The type system ensures middleware ordering constraints are satisfied at compile time.

## Error Handling in HTTP Context

HTTP error handling requires translating Rust's `Result` types into appropriate HTTP status codes. The actix-gcd example uses a simplified approach with `expect()`, but production code should propagate errors properly:

```rust
// Production-quality handler
async fn post_gcd(form: web::Form<GcdParameters>) -> Result<HttpResponse, actix_web::Error> {
    let n = form.n.try_into()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid parameter"))?;
    let m = form.m.try_into()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid parameter"))?;

    let result = compute_gcd(n, m)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(result))
}
```

This pattern demonstrates **error context enrichment**: domain errors (like integer conversion failures) are converted into HTTP errors with appropriate status codes. The `?` operator handles propagation, keeping the code clean.

**Implementing Custom Error Types**: For complex services, define domain-specific error types that implement `actix_web::ResponseError`:

```rust
#[derive(Debug)]
enum AppError {
    InvalidInput(String),
    ComputationError(String),
}

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::InvalidInput(msg) =>
                HttpResponse::BadRequest().body(msg),
            AppError::ComputationError(msg) =>
                HttpResponse::InternalServerError().body(msg),
        }
    }
}
```

Now handlers can return `Result<HttpResponse, AppError>`, and the framework automatically converts errors into appropriate HTTP responses.

## Testing Web Service Architecture

Actix-web provides excellent testing support, allowing you to test handlers without starting a full server:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_gcd_handler() {
        let app = test::init_service(
            App::new().route("/gcd", web::post().to(post_gcd))
        ).await;

        let req = test::TestRequest::post()
            .uri("/gcd")
            .set_form(&GcdParameters { n: 42, m: 56 })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
```

**Testing Strategy**:
- **Unit tests** for pure business logic (like the `gcd` function)
- **Integration tests** for handlers, using `test::init_service`
- **End-to-end tests** with `test::start()` for full server simulation

The testing architecture mirrors the application architecture: extractors work identically in tests and production, ensuring test fidelity.

## Architectural Lessons from actix-gcd

This case study reveals several architectural principles for Rust web services:

1. **Type-Driven Development**: Let types specify your API contracts. The compiler becomes your first line of defense against bugs.

2. **Separation of Concerns**: Business logic (the `gcd` function) remains pure and testable. HTTP concerns live in handlers. This enables easy testing and reuse.

3. **Async Throughout**: Don't mix blocking and async code. The entire request path should be async to maximize throughput.

4. **Builder Patterns for Configuration**: Use builders for complex types (like `HttpResponse`). They prevent invalid states and improve discoverability.

5. **Extractors as Dependencies**: Handler parameters declare their dependencies through types. This makes dependencies explicit and mockable.

## Cross-References to Foundational Patterns

This web service architecture builds on patterns from earlier chapters:

- **Chapter 2 (Ownership)**: Handler parameters demonstrate move vs. borrow semantics. `web::Form` moves data, while `web::Data` borrows shared state.
- **Chapter 5 (Error Handling)**: The `Result` type propagates errors through the HTTP layer, with custom error types implementing `ResponseError`.
- **Chapter 6 (Traits)**: The `FromRequest` trait enables the extractor pattern, while `ResponseError` allows custom error types.
- **Chapter 8 (Async)**: The entire HTTP cycle runs on async/await, demonstrating non-blocking I/O at scale.

The actix-gcd case study shows how these foundational patterns combine to create a production-ready web service. Each architectural decision leverages Rust's strengths: zero-cost abstractions, type safety, and fearless concurrency. The result is a web framework that's both ergonomic and extremely fast, routinely appearing at the top of web framework benchmarks.

In the next section, we'll examine how complex domain logic gets organized in the fern simulation system, revealing patterns for managing hierarchical module structures and domain-specific state.
