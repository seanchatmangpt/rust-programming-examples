# Application State

## Context

Your web service needs to share data across multiple request handlers: database connection pools, configuration settings, template engines, caches, or API clients. Each handler needs efficient, thread-safe access to this shared state without data races or excessive locking. The state is initialized once at startup and used for the lifetime of the application.

## Problem

**How do you safely share mutable state across concurrent HTTP request handlers while maintaining Rust's ownership and thread-safety guarantees?**

Shared state in web services faces several challenges:
- Multiple handlers run concurrently and need access to the same resources
- State must be initialized before the server starts accepting requests
- Shared mutable state risks data races in concurrent environments
- Cloning heavy resources (like database pools) for each request is expensive
- State lifetime must match or exceed the server's lifetime

Traditional approaches have weaknesses:
- **Global variables**: Unsafe in Rust, require `unsafe` or `static mut`
- **Thread-local storage**: Each thread gets its own copy (wastes memory for pools)
- **Passing state through middleware**: Verbose, no type safety
- **Locks everywhere**: Arc<Mutex<T>> adds contention and deadlock risk

## Forces

- **Sharing vs Safety**: Multiple threads need access; Rust prevents data races
- **Initialization vs Usage**: State created once at startup; used for server lifetime
- **Cloning Cost**: Some types (pools) are cheap to clone; others (large configs) are expensive
- **Mutability**: Most state is read-only; some needs interior mutability
- **Type Safety**: State access should be checked at compile time

## Solution

**Wrap shared state in `web::Data<T>` and register it with the application; extract it in handlers as a parameter.**

The pattern has four key components:

### 1. Define State Structure

```rust
use sqlx::PgPool;
use std::sync::Arc;

struct AppState {
    db_pool: PgPool,
    api_key: String,
    request_count: Arc<AtomicU64>,
}
```

**Design principles:**
- **Cheap-to-clone types**: Database pools (`PgPool`), Arc-wrapped data
- **Immutable data**: Configuration strings, constants
- **Interior mutability**: `Arc<Mutex<T>>`, `Arc<RwLock<T>>`, or atomics for mutable state

**Why this works:**
- `PgPool` is internally Arc-based (cloning is cheap, just increments ref count)
- `String` is cloned per worker thread (acceptable for small configs)
- `Arc<AtomicU64>` allows lock-free increments across threads

### 2. Initialize State in Main

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize shared state once
    let db_pool = PgPool::connect("postgres://...").await.unwrap();
    let api_key = std::env::var("API_KEY").unwrap();
    let request_count = Arc::new(AtomicU64::new(0));

    let state = AppState {
        db_pool,
        api_key,
        request_count,
    };

    // Wrap in web::Data (internally an Arc)
    let shared_state = web::Data::new(state);

    HttpServer::new(move || {
        App::new()
            .app_data(shared_state.clone())  // Register state
            .route("/users", web::get().to(get_users))
            .route("/stats", web::get().to(get_stats))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
```

**Critical details:**
- State is created **outside** the `HttpServer::new()` closure
- `web::Data::new(state)` wraps state in `Arc` (atomic reference counting)
- `move` closure captures `shared_state` by value
- `.app_data(shared_state.clone())` clones the `Arc`, not the underlying data
- Each worker thread gets its own `Arc` pointing to the **same shared state**

### 3. Extract State in Handlers

```rust
async fn get_users(state: web::Data<AppState>) -> HttpResponse {
    // Access database pool
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&state.db_pool)
        .await
        .unwrap();

    // Increment request counter
    state.request_count.fetch_add(1, Ordering::SeqCst);

    HttpResponse::Ok().json(users)
}

async fn get_stats(state: web::Data<AppState>) -> HttpResponse {
    let count = state.request_count.load(Ordering::SeqCst);
    HttpResponse::Ok().body(format!("Total requests: {}", count))
}
```

**Type safety:**
- `state: web::Data<AppState>` is an extractor (like `web::Form` or `web::Path`)
- If state isn't registered, **compilation fails** (extractor can't find type)
- `state.db_pool` is accessible directly (Deref trait)
- State is immutable (can't reassign fields), but fields can have interior mutability

### 4. Actix-GCD Example (No State)

The actix-gcd example is **stateless**:

```rust
#[actix_web::main]
async fn main() {
    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(get_index))
            .route("/gcd", web::post().to(post_gcd))
    });
    // No .app_data() calls = no shared state
}

async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    // Pure computation, no database or shared resources
    let result = gcd(form.n, form.m);
    HttpResponse::Ok().body(format!("GCD: {}", result))
}
```

**Why no state needed:**
- `gcd()` is a pure function (deterministic, no side effects)
- No database, no caching, no external APIs
- Each request is fully independent

**When actix-gcd WOULD need state:**
- **Logging requests**: Shared request counter or logger
- **Rate limiting**: Track request counts per IP
- **Caching results**: Store computed GCDs to avoid recomputation
- **Metrics**: Track response times, error rates

## Resulting Context

### Benefits

1. **Type-Safe Access**: Compiler enforces state registration and extraction
2. **Thread-Safe by Default**: Arc-based sharing prevents data races
3. **Zero-Cost Abstraction**: `web::Data` compiles to efficient pointer access
4. **Scoped Lifetime**: State lives as long as the application
5. **Efficient Cloning**: Arc clones are cheap (single pointer copy)
6. **No Global State**: No unsafe code or static mut required

### Liabilities

1. **Memory Overhead**: Each worker thread holds an Arc to state
2. **Initialization Complexity**: State must be fully initialized before server starts
3. **Interior Mutability Required**: Mutable state needs `Mutex`, `RwLock`, or atomics
4. **Error Handling**: Initialization failures must be handled before server starts
5. **Testing Complexity**: Tests need to construct full state objects

### Consequences

**State Cloning Semantics**:
- `web::Data<T>` wraps `Arc<T>` internally
- Cloning `web::Data<AppState>` increments Arc reference count (cheap)
- Cloning the **fields** inside `AppState` happens once per worker thread
- Workers share the same underlying data (via Arc)

**Mutability Patterns**:

**Immutable State (Most Common)**:
```rust
struct AppState {
    config: Config,  // Read-only
    db_pool: PgPool, // Pool is read-only, connections are from pool
}
```

**Mutable State with Mutex**:
```rust
struct AppState {
    cache: Arc<Mutex<HashMap<String, String>>>,
}

async fn handler(state: web::Data<AppState>) -> HttpResponse {
    let mut cache = state.cache.lock().unwrap();
    cache.insert("key".to_string(), "value".to_string());
    HttpResponse::Ok().finish()
}
```

**Mutable State with RwLock** (many readers, few writers):
```rust
struct AppState {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

async fn handler(state: web::Data<AppState>) -> HttpResponse {
    let cache = state.cache.read().unwrap();
    let value = cache.get("key").cloned();
    HttpResponse::Ok().body(value.unwrap_or_default())
}
```

**Mutable State with Atomics** (lock-free):
```rust
struct AppState {
    request_count: Arc<AtomicU64>,
}

async fn handler(state: web::Data<AppState>) -> HttpResponse {
    state.request_count.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().finish()
}
```

## Related Patterns

- **HTTP Service Skeleton**: State is registered in the app factory
- **Route Handlers**: Handlers extract state as parameters
- **Middleware**: Can access and modify state
- **Database Connection Pooling**: Pools are the most common shared state

## Known Uses

### Example 1: Database Pool (Typical Use Case)

```rust
use sqlx::PgPool;

struct AppState {
    db_pool: PgPool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect("postgres://localhost/mydb")
        .await
        .expect("Failed to connect to database");

    let state = web::Data::new(AppState { db_pool: pool });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/users", web::get().to(get_users))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}

async fn get_users(state: web::Data<AppState>) -> HttpResponse {
    let users = sqlx::query!("SELECT id, name FROM users")
        .fetch_all(&state.db_pool)
        .await
        .unwrap();

    HttpResponse::Ok().json(users)
}
```

### Example 2: Configuration and Secrets

```rust
struct AppState {
    jwt_secret: String,
    max_upload_size: usize,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        jwt_secret: std::env::var("JWT_SECRET").unwrap(),
        max_upload_size: 10 * 1024 * 1024, // 10 MB
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/login", web::post().to(login))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}

async fn login(
    state: web::Data<AppState>,
    form: web::Json<LoginForm>,
) -> HttpResponse {
    // Use JWT secret from state
    let token = generate_jwt(&form.username, &state.jwt_secret);
    HttpResponse::Ok().json(json!({ "token": token }))
}
```

### Example 3: Template Engine (Tera)

```rust
use tera::Tera;

struct AppState {
    templates: Tera,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tera = Tera::new("templates/**/*.html")
        .expect("Failed to load templates");

    let state = web::Data::new(AppState { templates: tera });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}

async fn index(state: web::Data<AppState>) -> HttpResponse {
    let mut ctx = tera::Context::new();
    ctx.insert("title", "Home");
    let html = state.templates.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(html)
}
```

### Example 4: Request Metrics

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct AppState {
    total_requests: Arc<AtomicU64>,
    successful_requests: Arc<AtomicU64>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        total_requests: Arc::new(AtomicU64::new(0)),
        successful_requests: Arc::new(AtomicU64::new(0)),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/api/data", web::get().to(get_data))
            .route("/metrics", web::get().to(metrics))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}

async fn get_data(state: web::Data<AppState>) -> HttpResponse {
    state.total_requests.fetch_add(1, Ordering::Relaxed);

    // Process request...
    let result = "data";

    state.successful_requests.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().body(result)
}

async fn metrics(state: web::Data<AppState>) -> HttpResponse {
    let total = state.total_requests.load(Ordering::Relaxed);
    let successful = state.successful_requests.load(Ordering::Relaxed);

    HttpResponse::Ok().json(json!({
        "total_requests": total,
        "successful_requests": successful,
        "error_rate": (total - successful) as f64 / total as f64,
    }))
}
```

### Example 5: Extending actix-gcd with State

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct AppState {
    computation_count: Arc<AtomicU64>,
}

#[actix_web::main]
async fn main() {
    let state = web::Data::new(AppState {
        computation_count: Arc::new(AtomicU64::new(0)),
    });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(get_index))
            .route("/gcd", web::post().to(post_gcd))
            .route("/stats", web::get().to(get_stats))
    });

    println!("Serving on http://localhost:3000...");
    server
        .bind("127.0.0.1:3000")
        .expect("error binding server to address")
        .run()
        .await
        .expect("error running server");
}

async fn post_gcd(
    form: web::Form<GcdParameters>,
    state: web::Data<AppState>,
) -> HttpResponse {
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Computing the GCD with zero is boring.");
    }

    // Increment counter
    state.computation_count.fetch_add(1, Ordering::Relaxed);

    let response = format!(
        "The greatest common divisor of the numbers {} and {} is <b>{}</b>\n",
        form.n,
        form.m,
        gcd(form.n, form.m)
    );

    HttpResponse::Ok().content_type("text/html").body(response)
}

async fn get_stats(state: web::Data<AppState>) -> HttpResponse {
    let count = state.computation_count.load(Ordering::Relaxed);
    HttpResponse::Ok().body(format!("Total GCD computations: {}", count))
}
```

## Implementation Notes

### Multiple State Types

You can register multiple state types:

```rust
struct DatabaseState {
    pool: PgPool,
}

struct CacheState {
    cache: Arc<Mutex<HashMap<String, String>>>,
}

HttpServer::new(move || {
    App::new()
        .app_data(web::Data::new(DatabaseState { pool: pool.clone() }))
        .app_data(web::Data::new(CacheState { cache: cache.clone() }))
        .route("/", web::get().to(handler))
})

async fn handler(
    db: web::Data<DatabaseState>,
    cache: web::Data<CacheState>,
) -> HttpResponse {
    // Access both states
    HttpResponse::Ok().finish()
}
```

### State Initialization Patterns

**Lazy Initialization** (if state setup can fail):
```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(
        AppState::new().await.expect("Failed to initialize state")
    );

    HttpServer::new(move || {
        App::new().app_data(state.clone())
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}

impl AppState {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let pool = PgPool::connect("postgres://...").await?;
        Ok(AppState { db_pool: pool })
    }
}
```

### Performance Characteristics

- **Arc Clone**: ~5-10ns (atomic increment + pointer copy)
- **Field Access**: 0ns (direct pointer dereference via Deref)
- **Mutex Lock**: ~50-100ns (uncontended), much slower if contended
- **RwLock Read**: ~30-50ns (uncontended)
- **Atomic Operation**: ~10-20ns (lock-free)

**Recommendation**: Prefer atomics > RwLock > Mutex for hot paths.

### Testing Strategies

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_with_state() {
        // Create test state
        let state = web::Data::new(AppState {
            computation_count: Arc::new(AtomicU64::new(0)),
        });

        // Create test app
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/gcd", web::post().to(post_gcd))
                .route("/stats", web::get().to(get_stats))
        )
        .await;

        // Test request
        let req = test::TestRequest::post()
            .uri("/gcd")
            .set_form(&GcdParameters { n: 42, m: 56 })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Verify state was updated
        assert_eq!(state.computation_count.load(Ordering::Relaxed), 1);
    }
}
```

## Common Patterns

### Database Pool Pattern

```rust
struct AppState {
    db: PgPool,
}

// In handlers:
async fn handler(state: web::Data<AppState>) -> HttpResponse {
    let row = sqlx::query!("SELECT id FROM users WHERE id = $1", user_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    HttpResponse::Ok().json(row)
}
```

**Why pools work well:**
- Pools are internally Arc-based (cheap to clone)
- Connections are acquired from pool per request
- No lock contention (pool handles it internally)

### Configuration Pattern

```rust
#[derive(Clone)]
struct Config {
    api_url: String,
    timeout_secs: u64,
}

struct AppState {
    config: Config,
}

// In handlers:
async fn handler(state: web::Data<AppState>) -> HttpResponse {
    let timeout = Duration::from_secs(state.config.timeout_secs);
    // Use config...
    HttpResponse::Ok().finish()
}
```

### Cache Pattern

```rust
struct AppState {
    cache: Arc<RwLock<HashMap<String, CachedData>>>,
}

async fn handler(state: web::Data<AppState>) -> HttpResponse {
    // Try to read from cache
    {
        let cache = state.cache.read().unwrap();
        if let Some(data) = cache.get("key") {
            return HttpResponse::Ok().json(data);
        }
    }

    // Cache miss - compute and cache
    let data = compute_data();
    {
        let mut cache = state.cache.write().unwrap();
        cache.insert("key".to_string(), data.clone());
    }

    HttpResponse::Ok().json(data)
}
```

## Security Considerations

1. **Secret Management**: Never log or expose state containing secrets
2. **Connection Limits**: Database pools should have max connections configured
3. **Memory Leaks**: State lives for server lifetime; don't accumulate unbounded data
4. **Atomic Overflows**: Use saturating operations or periodic resets
5. **Lock Safety**: Always use timeouts or try_lock to prevent deadlocks

## References

- **actix-web State Management**: https://actix.rs/docs/application/#state
- **Arc Documentation**: https://doc.rust-lang.org/std/sync/struct.Arc.html
- **Mutex vs RwLock**: https://doc.rust-lang.org/std/sync/
- **Atomic Types**: https://doc.rust-lang.org/std/sync/atomic/
