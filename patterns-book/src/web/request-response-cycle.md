# Request-Response Cycle

## Context

You have a working web service with routes, handlers, extractors, and possibly shared state. To debug issues, optimize performance, or add middleware, you need to understand the complete lifecycle of an HTTP request: from the moment bytes arrive on the socket to when the response is sent back to the client.

## Problem

**How does actix-web transform raw HTTP bytes into typed Rust values, route them to handlers, execute business logic, and serialize responses back to HTTP?**

Understanding the request-response cycle is crucial for:
- Debugging: "Why isn't my handler being called?"
- Performance: "Where are the bottlenecks?"
- Middleware: "When does middleware execute?"
- Error handling: "When do extractors fail vs handlers fail?"
- Testing: "How do I test individual stages?"

Without a mental model of the cycle, developers:
- Write inefficient code (blocking in async handlers)
- Misunderstand error propagation (extractor vs handler errors)
- Place middleware in wrong order
- Struggle with async lifetimes and borrows

## Forces

- **Async Execution**: Requests don't block threads; understanding Future execution is key
- **Type Safety**: Runtime HTTP data becomes compile-time Rust types
- **Error Boundaries**: Different stages have different error handling strategies
- **Efficiency**: Zero-copy parsing, minimal allocations
- **Composability**: Middleware, extractors, and handlers form a pipeline

## Solution

**Understand the request-response cycle as a series of stages, each with specific responsibilities and error handling characteristics.**

The complete cycle has 8 distinct stages:

### Stage 1: TCP Connection and HTTP Parsing

```
Client → TCP Socket → actix-web HTTP Parser → Request Object
```

**What happens:**
1. Client establishes TCP connection to server
2. Client sends raw HTTP bytes (e.g., `POST /gcd HTTP/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\n\r\nn=42&m=56`)
3. Actix-web parses bytes into `HttpRequest` struct:
   - Method (`POST`)
   - Path (`/gcd`)
   - Headers (`Content-Type: application/x-www-form-urlencoded`)
   - Body (as stream of bytes)

**Error handling:**
- Malformed HTTP → 400 Bad Request (automatic)
- Connection timeout → Connection closed
- Invalid UTF-8 in headers → 400 Bad Request

**Performance:**
- Zero-copy parsing where possible
- Headers parsed on-demand (lazy)
- Body stored as `Bytes` (reference-counted buffer)

### Stage 2: Route Matching

```
Request Object → Router → Handler Selection
```

**What happens:**
1. Router extracts method (GET, POST, etc.) and path (`/gcd`)
2. Router searches registered routes in order:
   ```rust
   .route("/", web::get().to(get_index))       // Not matched (path mismatch)
   .route("/gcd", web::post().to(post_gcd))    // ✓ Matched (POST + /gcd)
   ```
3. If matched, proceed to handler; if no match, return 404

**Route matching rules:**
- First match wins (order matters for overlapping routes)
- Method guard must match (`web::get()`, `web::post()`)
- Path pattern must match (static paths or patterns like `/users/{id}`)

**Example from actix-gcd:**
```rust
// Request: POST /gcd
// Matches: .route("/gcd", web::post().to(post_gcd))
// Handler: post_gcd

// Request: GET /
// Matches: .route("/", web::get().to(get_index))
// Handler: get_index

// Request: GET /gcd
// No match (wrong method) → 405 Method Not Allowed

// Request: POST /unknown
// No match (wrong path) → 404 Not Found
```

### Stage 3: Extractor Execution

```
Matched Handler → Execute Extractors → Typed Parameters
```

**What happens:**
1. Handler signature determines extractors:
   ```rust
   async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse
   //                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Extractor
   ```
2. Extractors run in parameter order (left to right)
3. Each extractor deserializes part of the request:
   - `web::Form<T>`: Parses URL-encoded body
   - `web::Json<T>`: Parses JSON body
   - `web::Path<T>`: Parses URL path parameters
   - `web::Query<T>`: Parses query string
   - `web::Data<T>`: Retrieves application state
4. If any extractor fails, handler **never runs** (automatic error response)

**actix-gcd example:**
```rust
async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse
```

**Extraction steps:**
1. Read request body as bytes
2. Parse as `application/x-www-form-urlencoded` (e.g., `n=42&m=56`)
3. Deserialize into `GcdParameters { n: 42, m: 56 }` using Serde
4. If successful, wrap in `web::Form` and call handler
5. If failed (missing field, wrong type), return 400 Bad Request

**Error examples:**

| Request Body | Extraction Result |
|--------------|-------------------|
| `n=42&m=56` | ✓ Success: `GcdParameters { n: 42, m: 56 }` |
| `n=42` | ✗ Failure: Missing field 'm' → 400 |
| `n=abc&m=56` | ✗ Failure: Invalid u64 'abc' → 400 |
| (empty) | ✗ Failure: Missing fields → 400 |

### Stage 4: Handler Execution

```
Typed Parameters → Handler Function → Responder
```

**What happens:**
1. Handler receives typed, validated parameters
2. Handler executes business logic (may `.await` async operations)
3. Handler returns a type implementing `Responder`

**actix-gcd example:**
```rust
async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    // Validation (business logic, not parsing)
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Computing the GCD with zero is boring.");
    }

    // Computation (pure function)
    let result = gcd(form.n, form.m);

    // Response construction
    let response = format!(
        "The greatest common divisor of the numbers {} and {} is <b>{}</b>\n",
        form.n, form.m, result
    );

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}
```

**Handler responsibilities:**
- **Validation**: Business rules (e.g., non-zero check)
- **Computation**: Delegate to pure functions
- **I/O**: Database queries, API calls (with `.await`)
- **Response construction**: Build `HttpResponse` with status, headers, body

**Error handling in handlers:**
- **Early return**: `return HttpResponse::BadRequest().body("error")`
- **Result propagation**: `async fn handler() -> Result<HttpResponse, MyError>`
- **Panic (avoid)**: Panics are caught but log errors and return 500

### Stage 5: Responder Conversion

```
Responder → HttpResponse
```

**What happens:**
1. Handler returns type implementing `Responder` trait
2. Responder's `.respond_to()` method converts to `HttpResponse`

**Common Responder types:**
- `HttpResponse`: Already a response (no conversion)
- `String` or `&str`: 200 OK with `text/plain`
- `web::Json<T>`: 200 OK with JSON body
- `Result<T, E>`: If Ok, convert T; if Err, convert E to error response

**actix-gcd example:**
```rust
HttpResponse::Ok()           // Status: 200
    .content_type("text/html")  // Header: Content-Type
    .body(response)              // Body: computed HTML string
```

**Responder flow:**
```
HttpResponse → (already a response, no conversion)
↓
Response with:
  - Status: 200 OK
  - Headers: Content-Type: text/html
  - Body: "The greatest common divisor..."
```

### Stage 6: Middleware (Optional)

```
HttpResponse → Middleware Chain → Modified Response
```

**What happens:**
1. Response passes through middleware in **reverse** registration order
2. Each middleware can:
   - Modify response (add headers, change status)
   - Log request/response details
   - Time request duration
3. Middleware can also run **before** handlers (request middleware)

**Example middleware:**
```rust
use actix_web::middleware::Logger;

App::new()
    .wrap(Logger::default())  // Logs all requests
    .route("/", web::get().to(get_index))
```

**Middleware execution order:**

```
Request → Middleware A → Middleware B → Handler → Middleware B → Middleware A → Response
          (in)           (in)                      (out)           (out)
```

**actix-gcd has no middleware** (handlers return responses directly).

### Stage 7: HTTP Serialization

```
HttpResponse → HTTP Bytes
```

**What happens:**
1. Status code serialized (e.g., `200 OK`)
2. Headers serialized (e.g., `Content-Type: text/html`)
3. Body written to socket (streaming if large)

**Example:**
```rust
HttpResponse::Ok()
    .content_type("text/html")
    .body("The greatest common divisor of the numbers 42 and 56 is <b>14</b>\n")
```

**Becomes:**
```http
HTTP/1.1 200 OK
Content-Type: text/html
Content-Length: 66

The greatest common divisor of the numbers 42 and 56 is <b>14</b>
```

### Stage 8: TCP Response and Connection Handling

```
HTTP Bytes → TCP Socket → Client
```

**What happens:**
1. Response bytes sent over TCP connection
2. Connection handling depends on HTTP version and headers:
   - **HTTP/1.0**: Close after response (default)
   - **HTTP/1.1**: Keep-alive (reuse connection for next request)
   - **HTTP/2**: Multiplex multiple requests on same connection

**actix-gcd uses HTTP/1.1:**
- Connection stays open for next request
- Client can send multiple requests without reconnecting
- Server closes connection after timeout or client closes

## Resulting Context

### Benefits of Understanding the Cycle

1. **Debugging**: Pinpoint exactly where errors occur (parsing, routing, extraction, handler)
2. **Performance**: Identify bottlenecks (slow extractors, blocking handlers)
3. **Middleware**: Place middleware at correct stage (before/after handlers)
4. **Testing**: Test individual stages in isolation
5. **Error Handling**: Understand error propagation (extractor errors vs handler errors)

### Key Insights

**Separation of Concerns:**
- **Parsing**: actix-web handles HTTP protocol
- **Routing**: Router matches paths and methods
- **Extraction**: Deserializers convert bytes to types
- **Business Logic**: Handlers contain application code
- **Serialization**: Responders convert types to HTTP

**Error Boundaries:**
- **HTTP Parsing Errors**: 400 Bad Request (automatic)
- **Routing Errors**: 404 Not Found or 405 Method Not Allowed (automatic)
- **Extraction Errors**: 400 Bad Request (automatic)
- **Handler Errors**: Custom responses (manual)

**Performance Characteristics:**
- **Stage 1 (Parsing)**: ~10-50μs
- **Stage 2 (Routing)**: ~100-500ns (hash table lookup)
- **Stage 3 (Extraction)**: ~1-10μs (depends on body size)
- **Stage 4 (Handler)**: Variable (your code!)
- **Stage 5-7 (Response)**: ~1-5μs
- **Total overhead**: ~15-70μs (excluding handler logic)

## Related Patterns

- **HTTP Service Skeleton**: Defines routes and starts the cycle
- **Route Handlers**: Stage 4 (handler execution)
- **Form Handling**: Stage 3 (extractor execution)
- **Application State**: Extracted in Stage 3, used in Stage 4
- **HTML Templating**: Stage 5 (response construction)

## Known Uses

### Complete Cycle: actix-gcd POST Request

**Request:**
```http
POST /gcd HTTP/1.1
Host: localhost:3000
Content-Type: application/x-www-form-urlencoded
Content-Length: 10

n=42&m=56
```

**Stage-by-Stage Execution:**

**Stage 1: HTTP Parsing**
```
Raw bytes → HttpRequest {
  method: POST,
  path: "/gcd",
  headers: { "Content-Type": "application/x-www-form-urlencoded" },
  body: Bytes([110, 61, 52, 50, 38, 109, 61, 53, 54])  // "n=42&m=56"
}
```

**Stage 2: Route Matching**
```
Router checks:
  .route("/", web::get().to(get_index))
    → Path "/" != "/gcd" → Skip

  .route("/gcd", web::post().to(post_gcd))
    → Path "/gcd" == "/gcd" ✓
    → Method POST == POST ✓
    → Match! Handler: post_gcd
```

**Stage 3: Extractor Execution**
```
post_gcd(form: web::Form<GcdParameters>)
         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
         Extract and deserialize form data

web::Form extractor:
1. Read body: "n=42&m=56"
2. Parse URL encoding: { "n": "42", "m": "56" }
3. Deserialize to GcdParameters:
   GcdParameters { n: 42, m: 56 }  // Success!
4. Wrap in web::Form and proceed to handler
```

**Stage 4: Handler Execution**
```rust
async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    // form.n = 42, form.m = 56

    // Validation
    if form.n == 0 || form.m == 0 {
        // Not taken (42 != 0, 56 != 0)
    }

    // Computation
    let result = gcd(42, 56);  // Returns 14

    // Response construction
    let response = format!(
        "The greatest common divisor of the numbers {} and {} is <b>{}</b>\n",
        42, 56, 14
    );
    // response = "The greatest common divisor of the numbers 42 and 56 is <b>14</b>\n"

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}
```

**Stage 5: Responder Conversion**
```
HttpResponse (already a response) → No conversion needed
```

**Stage 6: Middleware**
```
(No middleware configured in actix-gcd)
```

**Stage 7: HTTP Serialization**
```
HttpResponse → Raw bytes:

HTTP/1.1 200 OK\r\n
Content-Type: text/html\r\n
Content-Length: 74\r\n
\r\n
The greatest common divisor of the numbers 42 and 56 is <b>14</b>\n
```

**Stage 8: TCP Response**
```
Bytes sent to client → Connection kept alive (HTTP/1.1)
```

### Error Case: Invalid Form Data

**Request:**
```http
POST /gcd HTTP/1.1
Content-Type: application/x-www-form-urlencoded

n=abc&m=56
```

**Stage 3 Fails:**
```
web::Form extractor:
1. Read body: "n=abc&m=56"
2. Parse URL encoding: { "n": "abc", "m": "56" }
3. Deserialize to GcdParameters:
   n: "abc" → u64::from_str("abc") → Error! Invalid digit
4. Extraction failed → Return 400 Bad Request
   Handler NEVER RUNS
```

**Response:**
```http
HTTP/1.1 400 Bad Request
Content-Type: text/plain

Form deserialize error: invalid digit found in string
```

### GET Request Cycle

**Request:**
```http
GET / HTTP/1.1
Host: localhost:3000
```

**Stage 2: Route Matching**
```
.route("/", web::get().to(get_index))
  → Path "/" == "/" ✓
  → Method GET == GET ✓
  → Match! Handler: get_index
```

**Stage 3: Extractor Execution**
```
get_index()  // No parameters = no extractors
```

**Stage 4: Handler Execution**
```rust
async fn get_index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(r#"
            <title>GCD Calculator</title>
            <form action="/gcd" method="post">
            <input type="text" name="n"/>
            <input type="text" name="m"/>
            <button type="submit">Compute GCD</button>
            </form>
        "#)
}
```

**Response:**
```http
HTTP/1.1 200 OK
Content-Type: text/html

<title>GCD Calculator</title>
<form action="/gcd" method="post">
<input type="text" name="n"/>
<input type="text" name="m"/>
<button type="submit">Compute GCD</button>
</form>
```

## Implementation Notes

### Async Execution Model

Handlers are **async functions**, meaning:
- They return `impl Future`
- They don't block threads
- They can `.await` other futures

**Execution flow:**
```rust
async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    // This is NOT a regular function!
    // It returns a Future that will be polled by Tokio runtime

    // Synchronous code (no .await)
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest().body("error");
    }

    // If we had async operations:
    // let user = db.get_user(form.user_id).await;
    //                                     ^^^^^^ Yields control, doesn't block

    HttpResponse::Ok().body("result")
}
```

**Why async matters:**
- Synchronous handlers would block worker threads
- Async handlers can yield while waiting for I/O
- One worker thread can handle thousands of concurrent requests

### Extractor Ordering

Extractors execute **left to right**:

```rust
async fn handler(
    path: web::Path<UserId>,      // Executes FIRST
    query: web::Query<Filters>,   // Executes SECOND
    json: web::Json<CreateUser>,  // Executes THIRD
    state: web::Data<AppState>,   // Executes FOURTH
) -> HttpResponse {
    // If any extractor fails, subsequent extractors don't run
}
```

**Optimization tip**: Put cheap extractors first:
- `web::Data`: Just a pointer lookup (very fast)
- `web::Path`: Parse URL segment (fast)
- `web::Query`: Parse query string (fast)
- `web::Form`, `web::Json`: Parse request body (slower)

### Error Propagation

**Extractor errors:**
- Automatic 400 Bad Request
- Handler never runs
- No custom error handling (unless you implement custom extractor)

**Handler errors:**
- Manual error responses (`return HttpResponse::BadRequest()`)
- Or `Result<HttpResponse, E>` where `E: ResponseError`

**Example with Result:**
```rust
#[derive(Debug)]
struct MyError(String);

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}

impl actix_web::ResponseError for MyError {}

async fn handler(form: web::Form<Params>) -> Result<HttpResponse, MyError> {
    if form.value < 0 {
        return Err(MyError("Value must be positive".to_string()));
    }
    Ok(HttpResponse::Ok().body("Success"))
}
```

### Performance Profiling

**Measure each stage:**
```rust
use std::time::Instant;

async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    let start = Instant::now();

    // Business logic
    let result = gcd(form.n, form.m);

    let elapsed = start.elapsed();
    println!("Handler took {:?}", elapsed);

    HttpResponse::Ok().body(format!("Result: {}", result))
}
```

**Add middleware for request timing:**
```rust
use actix_web::middleware::Logger;

App::new()
    .wrap(Logger::new("%r %s %Dms"))  // Logs: "POST /gcd 200 5ms"
    .route("/gcd", web::post().to(post_gcd))
```

### Testing Individual Stages

**Test extractors:**
```rust
#[actix_web::test]
async fn test_extraction() {
    let (req, mut payload) = test::TestRequest::post()
        .uri("/gcd")
        .set_form(&GcdParameters { n: 42, m: 56 })
        .to_http_parts();

    let form = web::Form::<GcdParameters>::from_request(&req, &mut payload)
        .await
        .unwrap();

    assert_eq!(form.n, 42);
    assert_eq!(form.m, 56);
}
```

**Test handlers:**
```rust
#[actix_web::test]
async fn test_handler() {
    let form = web::Form(GcdParameters { n: 42, m: 56 });
    let resp = post_gcd(form).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
```

**Test full cycle:**
```rust
#[actix_web::test]
async fn test_full_cycle() {
    let app = test::init_service(
        App::new().route("/gcd", web::post().to(post_gcd))
    ).await;

    let req = test::TestRequest::post()
        .uri("/gcd")
        .set_form(&GcdParameters { n: 42, m: 56 })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    assert!(body.contains(b"14"));
}
```

## Debugging Strategies

### Logging Each Stage

```rust
use log::info;

async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    info!("Stage 3: Extraction successful: n={}, m={}", form.n, form.m);

    if form.n == 0 || form.m == 0 {
        info!("Stage 4: Validation failed (zero input)");
        return HttpResponse::BadRequest().body("error");
    }

    info!("Stage 4: Computing GCD...");
    let result = gcd(form.n, form.m);
    info!("Stage 4: GCD computed: {}", result);

    info!("Stage 5: Building response");
    HttpResponse::Ok().body(format!("GCD: {}", result))
}
```

### Common Issues and Diagnosis

**Issue: Handler never runs**
- Check: Route matching (method + path)
- Check: Extractor errors (validate input matches expected types)

**Issue: 405 Method Not Allowed**
- Route exists but method guard doesn't match
- Example: POST to a GET-only route

**Issue: 404 Not Found**
- No route matches the path
- Check: Exact path match (trailing slashes matter)

**Issue: 400 Bad Request**
- Extractor failed (form parsing, JSON parsing, etc.)
- Check: Request Content-Type header
- Check: Request body format

**Issue: 500 Internal Server Error**
- Handler panicked
- Check: Server logs for panic message

## Visual Diagram

```
Client Request
    │
    ├─ [Stage 1: HTTP Parsing]
    │   └─→ Parse raw bytes to HttpRequest
    │       ├─ Success → Continue
    │       └─ Failure → 400 Bad Request
    │
    ├─ [Stage 2: Route Matching]
    │   └─→ Match method + path to handler
    │       ├─ Success → Continue to handler
    │       ├─ No match → 404 Not Found
    │       └─ Wrong method → 405 Method Not Allowed
    │
    ├─ [Stage 3: Extractor Execution]
    │   └─→ Deserialize request data to types
    │       ├─ Success → Continue to handler
    │       └─ Failure → 400 Bad Request (handler NEVER runs)
    │
    ├─ [Stage 4: Handler Execution]
    │   └─→ Execute business logic
    │       ├─ Success → Return Responder
    │       ├─ Error → Custom error response
    │       └─ Panic → 500 Internal Server Error
    │
    ├─ [Stage 5: Responder Conversion]
    │   └─→ Convert Responder to HttpResponse
    │
    ├─ [Stage 6: Middleware (optional)]
    │   └─→ Modify response
    │
    ├─ [Stage 7: HTTP Serialization]
    │   └─→ Convert HttpResponse to raw bytes
    │
    └─ [Stage 8: TCP Response]
        └─→ Send bytes to client
```

## Security Considerations

1. **Stage 1**: Validate HTTP protocol (actix-web handles this)
2. **Stage 2**: Don't leak information in 404 responses (avoid route enumeration)
3. **Stage 3**: Extractors validate input types (but not business rules)
4. **Stage 4**: Always validate business logic (even after extraction succeeds)
5. **Stage 5-7**: Escape output to prevent XSS (especially HTML responses)
6. **Stage 8**: Use HTTPS to encrypt responses (configure with rustls)

## References

- **actix-web Architecture**: https://actix.rs/docs/
- **HTTP/1.1 Specification**: RFC 7230-7235
- **Tokio Runtime**: https://tokio.rs/
- **Rust Async Book**: https://rust-lang.github.io/async-book/
