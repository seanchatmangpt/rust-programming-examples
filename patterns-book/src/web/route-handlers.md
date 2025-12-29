# Route Handlers

## Context

You have defined routes in your web service (paths like `/`, `/gcd`, `/users/{id}`) and need to implement the logic that processes requests to those endpoints. Each route needs different behavior: returning HTML, processing form data, querying databases, or returning JSON. Handlers must be type-safe, composable, and efficient.

## Problem

**How do you define handler functions that extract request data, perform business logic, and return responses in a type-safe, composable manner?**

Handler functions face several design challenges:
- Extracting different types of data (path params, query strings, forms, JSON, headers)
- Maintaining type safety without verbose boilerplate
- Handling both synchronous and asynchronous operations
- Composing extractors (e.g., session + database + request body)
- Returning different response types (HTML, JSON, redirects, errors)

Traditional approaches have limitations:
- **Untyped handlers** (Node.js, PHP): `req.params['id']` can fail at runtime
- **Manual extraction**: Verbose parsing code in every handler
- **Reflection-based frameworks**: Runtime overhead, unclear types

## Forces

- **Type Safety vs Ergonomics**: Explicit types are verbose; inference reduces boilerplate
- **Sync vs Async**: Blocking handlers are simpler; async handlers scale better
- **Extractor Composition**: Need multiple data sources in one handler
- **Error Handling**: Some errors should propagate (500), others return client errors (400/404)
- **Response Types**: Handlers may return different response types

## Solution

**Define handlers as async functions with typed extractor parameters and return types implementing `Responder`.**

The pattern has several key components:

### 1. Simple Handler: No Extractors

```rust
async fn get_index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(
            r#"
                <title>GCD Calculator</title>
                <form action="/gcd" method="post">
                <input type="text" name="n"/>
                <input type="text" name="m"/>
                <button type="submit">Compute GCD</button>
                </form>
            "#,
        )
}
```

**Handler signature**:
- `async fn` enables asynchronous operations (`.await`)
- No parameters = no request data extraction
- Returns `HttpResponse` (implements `Responder`)

**Response construction**:
- `HttpResponse::Ok()` creates 200 status builder
- `.content_type("text/html")` sets header
- `.body(...)` sets response body

**Registration**:
```rust
.route("/", web::get().to(get_index))
```

### 2. Handler with Form Extractor

```rust
use serde::Deserialize;

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

    let response =
        format!("The greatest common divisor of the numbers {} and {} \
                 is <b>{}</b>\n",
                form.n, form.m, gcd(form.n, form.m));

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}
```

**Extractor signature**:
- `form: web::Form<GcdParameters>` extracts and parses form data
- If parsing fails, handler **never runs** (automatic 400 response)
- `form.n` and `form.m` are guaranteed valid `u64` values

**Business logic**:
- Validation (zero check) happens in handler
- Computation delegates to pure function `gcd()`
- Early return for error case

**Registration**:
```rust
.route("/gcd", web::post().to(post_gcd))
```

### 3. Handler Type System

**The handler signature contract**:
```rust
async fn handler(
    extractor1: Extractor1,
    extractor2: Extractor2,
    // ... up to 12 extractors
) -> impl Responder {
    // Handler body
}
```

**Extractors** (first N parameters):
- `web::Form<T>`: URL-encoded form data
- `web::Json<T>`: JSON request body
- `web::Path<T>`: URL path parameters
- `web::Query<T>`: URL query string
- `web::Data<T>`: Application state
- `HttpRequest`: Raw request object
- `web::Bytes`: Raw request body
- Custom extractors implementing `FromRequest`

**Responders** (return type):
- `HttpResponse`: Full control over response
- `String`: 200 OK with text/plain
- `&str`: 200 OK with text/plain
- `Result<T, E>`: T on Ok, error response on Err
- `impl Responder`: Any type implementing trait
- Custom types implementing `Responder`

## Resulting Context

### Benefits

1. **Type-Safe Extraction**: Extractors are checked at compile time
2. **Automatic Deserialization**: No manual parsing or error handling
3. **Composable**: Mix multiple extractors in one handler
4. **Async-First**: Supports `.await` for database calls, HTTP requests, etc.
5. **Flexible Returns**: Different handlers can return different response types
6. **Early Validation**: Invalid requests never reach handler logic

### Liabilities

1. **Async Complexity**: Must understand `async`/`await` and futures
2. **Error Propagation**: `?` operator requires careful Result type management
3. **Extractor Limits**: Maximum 12 extractors per handler (rarely a problem)
4. **Generic Error Messages**: Default errors may need customization
5. **Compile-Time Cost**: Heavy use of generics increases compile time

### Consequences

**Handler Execution Order**:
1. Route matching (path + method)
2. Extractor execution (in parameter order)
3. If any extractor fails → automatic error response (handler never runs)
4. If all extractors succeed → handler runs
5. Handler returns `Responder` → converted to HTTP response

**Error Handling Strategies**:

**Strategy 1: Return Error Responses Directly**
```rust
async fn handler(form: web::Form<Params>) -> HttpResponse {
    if form.value < 10 {
        return HttpResponse::BadRequest().body("Value too small");
    }
    HttpResponse::Ok().body("Success")
}
```

**Strategy 2: Use Result Return Type**
```rust
async fn handler(form: web::Form<Params>) -> Result<HttpResponse, MyError> {
    validate(&form)?;  // Propagate validation errors
    Ok(HttpResponse::Ok().body("Success"))
}
```

**Strategy 3: Implement Responder for Custom Types**
```rust
struct JsonResponse<T> {
    data: T,
}

impl<T: Serialize> Responder for JsonResponse<T> {
    type Body = BoxBody;
    fn respond_to(self, _req: &HttpRequest) -> HttpResponse {
        HttpResponse::Ok().json(self.data)
    }
}

async fn handler() -> JsonResponse<User> {
    JsonResponse { data: get_user() }
}
```

## Related Patterns

- **HTTP Service Skeleton**: Route registration connects paths to handlers
- **Form Handling**: `web::Form` is one type of extractor
- **Application State**: `web::Data` extractor accesses shared state
- **Request-Response Cycle**: Understanding when handlers execute
- **Middleware**: Wraps handlers with cross-cutting concerns

## Known Uses

From **actix-gcd** (`/home/user/rust-programming-examples/actix-gcd/src/main.rs`):

### Example 1: Stateless GET Handler

```rust
async fn get_index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(
            r#"
                <title>GCD Calculator</title>
                <form action="/gcd" method="post">
                <input type="text" name="n"/>
                <input type="text" name="m"/>
                <button type="submit">Compute GCD</button>
                </form>
            "#,
        )
}
```

**Characteristics**:
- No parameters = no extraction
- Synchronous HTML generation (no `.await`)
- Returns fixed content (could be generated dynamically)
- 200 OK with `text/html` content type

### Example 2: POST Handler with Form Extraction

```rust
async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    // Validation
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Computing the GCD with zero is boring.");
    }

    // Business logic
    let response =
        format!("The greatest common divisor of the numbers {} and {} \
                 is <b>{}</b>\n",
                form.n, form.m, gcd(form.n, form.m));

    // Success response
    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}
```

**Characteristics**:
- One extractor: `web::Form<GcdParameters>`
- Early return for error case (400 Bad Request)
- Delegates computation to pure function `gcd()`
- Dynamic HTML generation with `format!`

### Example 3: Pure Business Logic (Called by Handler)

```rust
fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);
    while m != 0 {
        if m < n {
            let t = m;
            m = n;
            n = t;
        }
        m = m % n;
    }
    n
}
```

**Separation of Concerns**:
- `gcd()` is a **pure function**: no I/O, no state, deterministic
- Handler validates input before calling `gcd()`
- `gcd()` can be unit tested independently of HTTP layer
- `assert!` is safe because handler guarantees non-zero inputs

## Implementation Notes

### Common Extractor Combinations

**Path + Query Parameters**:
```rust
#[derive(Deserialize)]
struct UserPath {
    id: u64,
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn get_user_posts(
    path: web::Path<UserPath>,
    query: web::Query<Pagination>,
) -> HttpResponse {
    let user_id = path.id;
    let page = query.page.unwrap_or(1);
    // Fetch posts for user...
    HttpResponse::Ok().body("Posts")
}

// Route: .route("/users/{id}/posts", web::get().to(get_user_posts))
// Example: GET /users/42/posts?page=2&limit=10
```

**Form + Application State**:
```rust
struct AppState {
    db_pool: DbPool,
}

async fn create_user(
    form: web::Form<CreateUserForm>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let conn = state.db_pool.get().await.unwrap();
    // Use conn to insert user...
    HttpResponse::Created().body("User created")
}
```

**JSON Request + JSON Response**:
```rust
#[derive(Deserialize)]
struct ApiRequest {
    query: String,
}

#[derive(Serialize)]
struct ApiResponse {
    results: Vec<String>,
}

async fn api_search(
    json: web::Json<ApiRequest>,
) -> web::Json<ApiResponse> {
    let results = perform_search(&json.query);
    web::Json(ApiResponse { results })
}
```

### Response Builder Patterns

**Status Codes**:
```rust
HttpResponse::Ok()           // 200
HttpResponse::Created()      // 201
HttpResponse::NoContent()    // 204
HttpResponse::BadRequest()   // 400
HttpResponse::Unauthorized() // 401
HttpResponse::NotFound()     // 404
HttpResponse::InternalServerError() // 500
```

**Headers**:
```rust
HttpResponse::Ok()
    .content_type("application/json")
    .insert_header(("X-Custom-Header", "value"))
    .body(json_string)
```

**Redirects**:
```rust
HttpResponse::Found()
    .insert_header((header::LOCATION, "/new-path"))
    .finish()
```

**Streaming Responses**:
```rust
use actix_web::body::BodyStream;
use futures::stream;

async fn stream_handler() -> HttpResponse {
    let stream = stream::iter(vec![
        Ok::<_, Error>(web::Bytes::from("chunk1")),
        Ok(web::Bytes::from("chunk2")),
    ]);
    HttpResponse::Ok()
        .content_type("text/plain")
        .streaming(stream)
}
```

### Performance Characteristics

- **Handler Dispatch**: ~100ns per request (hash table lookup)
- **Extractor Overhead**: ~1-10μs depending on type (Form, JSON, etc.)
- **Response Building**: ~100-500ns for simple responses
- **Total Latency**: ~10-50μs for simple handlers (excluding business logic)

### Testing Strategies

**Unit Testing Handlers**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_get_index() {
        let resp = get_index().await;
        assert_eq!(resp.status(), StatusCode::OK);
        // Can't easily test body without creating a full app
    }

    #[actix_web::test]
    async fn test_post_gcd_valid() {
        let form = web::Form(GcdParameters { n: 42, m: 56 });
        let resp = post_gcd(form).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_post_gcd_zero() {
        let form = web::Form(GcdParameters { n: 0, m: 56 });
        let resp = post_gcd(form).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
```

**Integration Testing with TestRequest**:
```rust
#[actix_web::test]
async fn test_integration() {
    let app = test::init_service(
        App::new()
            .route("/", web::get().to(get_index))
            .route("/gcd", web::post().to(post_gcd))
    ).await;

    // Test GET /
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Test POST /gcd with valid data
    let req = test::TestRequest::post()
        .uri("/gcd")
        .set_form(&GcdParameters { n: 42, m: 56 })
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
```

## Advanced Patterns

### Custom Extractors

```rust
use actix_web::{FromRequest, HttpRequest};
use futures::future::{ready, Ready};

struct Authentication {
    user_id: u64,
}

impl FromRequest for Authentication {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        // Extract and validate auth token
        if let Some(token) = req.headers().get("Authorization") {
            // Validate token, extract user_id
            ready(Ok(Authentication { user_id: 123 }))
        } else {
            ready(Err(actix_web::error::ErrorUnauthorized("Missing auth")))
        }
    }
}

async fn protected_handler(auth: Authentication) -> HttpResponse {
    HttpResponse::Ok().body(format!("User ID: {}", auth.user_id))
}
```

### Handler Factories

For complex handlers, use factory pattern:

```rust
fn make_user_handler(config: &Config) -> impl Fn(web::Path<UserId>) -> HttpResponse {
    let max_id = config.max_user_id;
    move |path: web::Path<UserId>| {
        if path.id > max_id {
            HttpResponse::NotFound().finish()
        } else {
            HttpResponse::Ok().body(format!("User {}", path.id))
        }
    }
}
```

## Security Considerations

1. **Input Validation**: Always validate business rules even after extractor succeeds
2. **Output Encoding**: Escape HTML/SQL/etc. before embedding user input
3. **Error Information Leakage**: Don't return stack traces or internal details in production
4. **Rate Limiting**: Apply per-endpoint rate limits for expensive operations
5. **Authentication**: Use custom extractors to enforce authentication

### Example: Safe HTML Output

```rust
use html_escape::encode_text;

async fn post_comment(form: web::Form<Comment>) -> HttpResponse {
    // UNSAFE: Direct embedding
    // let html = format!("<p>{}</p>", form.text);

    // SAFE: Escape user input
    let escaped = encode_text(&form.text);
    let html = format!("<p>{}</p>", escaped);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}
```

## References

- **actix-web Handlers**: https://actix.rs/docs/handlers/
- **actix-web Extractors**: https://actix.rs/docs/extractors/
- **Rust Async Book**: https://rust-lang.github.io/async-book/
- **Responder Trait**: https://docs.rs/actix-web/latest/actix_web/trait.Responder.html
