# How to Build a Web Server with Actix-web

## Overview

This guide shows you how to build a complete web server using Actix-web, one of the fastest and most popular Rust web frameworks. You'll learn how to set up routes, handle forms, return HTML responses, and build a working GCD (Greatest Common Divisor) calculator web application.

## Prerequisites

- Intermediate Rust knowledge
- Understanding of async/await
- Basic web development concepts (HTTP, forms, etc.)
- Familiarity with HTML

## Step 1: Add dependencies

Add the following to your `Cargo.toml`:

```toml
[dependencies]
actix-web = "4.1"
serde = { version = "1.0", features = ["derive"] }
```

**What these provide:**
- `actix-web` - The web framework with routing, HTTP server, and utilities
- `serde` - Serialization/deserialization for form data and JSON

## Step 2: Import required modules

```rust
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Deserialize;
```

## Step 3: Create the main function

Actix-web requires an async runtime:

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

**Breaking this down:**
- `#[actix_web::main]` - Macro that sets up the async runtime
- `HttpServer::new()` - Creates the HTTP server
- `App::new()` - Creates an application instance
- `.route()` - Registers URL paths with handlers
- `web::get()` and `web::post()` - Specify HTTP methods
- `.bind()` - Binds to IP address and port
- `.run().await` - Starts the server

## Step 4: Create a GET route handler

This serves an HTML form:

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

**Key points:**
- `async fn` - Handler functions are async
- `HttpResponse::Ok()` - Returns 200 status code
- `.content_type()` - Sets the Content-Type header
- `.body()` - Sets the response body
- `r#"..."#` - Raw string literal (no need to escape quotes)

## Step 5: Define form data structure

Use Serde to automatically deserialize form data:

```rust
#[derive(Deserialize)]
struct GcdParameters {
    n: u64,
    m: u64,
}
```

**How it works:**
- `#[derive(Deserialize)]` - Automatically implements deserialization
- Field names must match form input names
- Actix-web will parse the form data into this struct

## Step 6: Create a POST route handler

This processes the form submission:

```rust
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

**Key features:**
- `web::Form<GcdParameters>` - Automatically deserializes form data
- Input validation - Checks for zero values
- `HttpResponse::BadRequest()` - Returns 400 status for invalid input
- `format!()` - Builds the HTML response dynamically

## Step 7: Implement the GCD algorithm

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

This is Euclid's algorithm for computing the greatest common divisor.

## Complete working example

```rust
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Deserialize;

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

## Running the server

```bash
cd actix-gcd
cargo run
```

Then visit `http://localhost:3000` in your browser.

## Testing the application

1. Open `http://localhost:3000`
2. Enter two numbers (e.g., 12 and 18)
3. Click "Compute GCD"
4. See the result: "The greatest common divisor of 12 and 18 is **6**"

## Advanced: Returning JSON

Add this handler for a JSON API:

```rust
use actix_web::web::Json;
use serde::Serialize;

#[derive(Serialize)]
struct GcdResult {
    n: u64,
    m: u64,
    gcd: u64,
}

async fn post_gcd_json(form: web::Form<GcdParameters>) -> Json<GcdResult> {
    Json(GcdResult {
        n: form.n,
        m: form.m,
        gcd: gcd(form.n, form.m),
    })
}

// Add to routes:
// .route("/api/gcd", web::post().to(post_gcd_json))
```

Test with curl:
```bash
curl -X POST http://localhost:3000/api/gcd -d "n=12&m=18"
```

Response:
```json
{"n":12,"m":18,"gcd":6}
```

## Advanced: Multiple routes and route parameters

```rust
async fn get_user(path: web::Path<u32>) -> HttpResponse {
    let user_id = path.into_inner();
    HttpResponse::Ok().body(format!("User ID: {}", user_id))
}

// Add to routes:
// .route("/users/{id}", web::get().to(get_user))
```

Visit `http://localhost:3000/users/42` to see "User ID: 42".

## Advanced: Shared application state

```rust
use std::sync::Mutex;

struct AppState {
    counter: Mutex<i32>,
}

async fn increment(data: web::Data<AppState>) -> HttpResponse {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    HttpResponse::Ok().body(format!("Counter: {}", counter))
}

// In main:
let app_state = web::Data::new(AppState {
    counter: Mutex::new(0),
});

let server = HttpServer::new(move || {
    App::new()
        .app_data(app_state.clone())
        .route("/increment", web::get().to(increment))
});
```

## Advanced: Middleware and logging

```rust
use actix_web::middleware::Logger;
use env_logger::Env;

#[actix_web::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(get_index))
    });

    // ... rest of server setup
}
```

Add to `Cargo.toml`:
```toml
env_logger = "0.10"
```

## Comparison to Python Flask

### Flask version:

```python
from flask import Flask, request, render_template_string

app = Flask(__name__)

@app.route('/')
def index():
    return '''
        <form action="/gcd" method="post">
            <input name="n"/>
            <input name="m"/>
            <button>Compute GCD</button>
        </form>
    '''

@app.route('/gcd', methods=['POST'])
def compute_gcd():
    n = int(request.form['n'])
    m = int(request.form['m'])

    if n == 0 or m == 0:
        return "Computing the GCD with zero is boring.", 400

    result = gcd(n, m)
    return f"The GCD of {n} and {m} is <b>{result}</b>"

def gcd(n, m):
    while m != 0:
        if m < n:
            n, m = m, n
        m = m % n
    return n

if __name__ == '__main__':
    app.run(port=3000)
```

### Actix-web version:

The complete example above.

**Key differences:**
- Flask uses decorators, Actix uses explicit route registration
- Flask has automatic type conversion, Rust uses Serde
- Rust requires async/await, Flask is synchronous by default
- Rust is strongly typed, Python uses duck typing
- Actix is significantly faster in production

## Comparison to Django

Django is more batteries-included with ORM, admin panel, etc. For a similar experience in Rust:

- Use `Diesel` or `SeaORM` for database ORM
- Use `Tera` or `Askama` for templating
- Use `actix-session` for session management
- Use `actix-identity` for authentication

## Performance characteristics

Actix-web is one of the fastest web frameworks across all languages:

- **Async I/O** - Non-blocking request handling
- **Zero-copy** - Efficient data handling
- **Type safety** - Compile-time guarantees
- **Low overhead** - Minimal runtime cost

Typical performance: 100,000+ requests/second on consumer hardware.

## Common patterns

### Error handling with Result

```rust
use actix_web::error;

async fn handler() -> Result<HttpResponse, error::Error> {
    let data = fetch_data()?;
    Ok(HttpResponse::Ok().json(data))
}
```

### Extracting query parameters

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct QueryParams {
    search: String,
    page: Option<u32>,
}

async fn search(query: web::Query<QueryParams>) -> HttpResponse {
    HttpResponse::Ok().body(format!("Search: {}, Page: {}",
        query.search, query.page.unwrap_or(1)))
}

// .route("/search", web::get().to(search))
// Visit: /search?search=rust&page=2
```

### File uploads

```rust
use actix_multipart::Multipart;
use futures::StreamExt;

async fn upload(mut payload: Multipart) -> HttpResponse {
    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // Process file chunk
        }
    }
    HttpResponse::Ok().body("Uploaded")
}
```

## Best practices

1. **Use extractors** - `web::Form`, `web::Json`, `web::Path`, etc.
2. **Handle errors properly** - Return `Result` types
3. **Set appropriate status codes** - 200, 400, 404, 500, etc.
4. **Use middleware** - For logging, CORS, authentication
5. **Validate input** - Always check user data
6. **Use app state carefully** - Minimize shared mutable state
7. **Structure your code** - Separate routes, handlers, and business logic
8. **Enable compression** - Use `actix-web::middleware::Compress`

## Troubleshooting

### "Address already in use"

Another process is using port 3000. Change the port:

```rust
.bind("127.0.0.1:8080")
```

### Form data not parsing

Ensure:
- Form field names match struct field names exactly
- Content-Type is `application/x-www-form-urlencoded`
- The handler uses `web::Form<T>`

### Async function errors

Make sure:
- Main function has `#[actix_web::main]` attribute
- All handlers are `async fn`
- You `.await` async operations

## Project structure for larger apps

```
src/
  main.rs          # Server setup
  handlers/        # Route handlers
    mod.rs
    users.rs
    gcd.rs
  models/          # Data structures
    mod.rs
    user.rs
  state.rs         # Application state
  config.rs        # Configuration
```

## See also

- [Actix-web documentation](https://actix.rs/)
- [How to implement URL routing](03-implement-url-router.md) - understand routing internals
- [Serde documentation](https://serde.rs/)
- [Tokio async runtime](https://tokio.rs/)
