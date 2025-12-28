# How to Implement URL Routing in Rust

## Overview

This guide shows you how to build a URL router from scratch in Rust. You'll learn how to map URL paths to handler functions, use generics for flexibility, and implement pattern matching for request dispatch. This is the foundation of how web frameworks like Flask, FastAPI, and Actix-web work.

## Prerequisites

- Intermediate Rust knowledge
- Understanding of closures and trait objects
- Familiarity with `HashMap`
- Basic web development concepts

## Step 1: Define your data structures

First, create types to represent HTTP requests and responses:

```rust
use std::collections::HashMap;

struct Request {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Vec<u8>
}

struct Response {
    code: u32,
    headers: HashMap<String, String>,
    body: Vec<u8>
}
```

**Why these structures?**
- `Request` contains everything the server receives
- `Response` contains everything to send back
- `Vec<u8>` for body allows binary data (images, files, etc.)
- `HashMap` for headers provides flexible key-value storage

## Step 2: Define the handler type

Handlers are functions that process requests and return responses:

```rust
type BoxedCallback = Box<dyn Fn(&Request) -> Response>;
```

**Breaking this down:**
- `Fn(&Request) -> Response` - A function that takes a request and returns a response
- `dyn` - Makes this a trait object (dynamic dispatch)
- `Box<>` - Heap-allocates the function (allows different sized closures)
- This allows storing different handler functions in the same collection

## Step 3: Create the router structure

```rust
struct BasicRouter {
    routes: HashMap<String, BoxedCallback>
}
```

The router stores a mapping from URL paths to handler functions.

## Step 4: Implement router creation

```rust
impl BasicRouter {
    // Create an empty router.
    fn new() -> BasicRouter {
        BasicRouter { routes: HashMap::new() }
    }
}
```

## Step 5: Implement route registration

This is the key method that adds routes:

```rust
impl BasicRouter {
    // Add a route to the router.
    fn add_route<C>(&mut self, url: &str, callback: C)
        where C: Fn(&Request) -> Response + 'static
    {
        self.routes.insert(url.to_string(), Box::new(callback));
    }
}
```

**Why generics here?**
- `<C>` makes this work with any function type
- `C: Fn(&Request) -> Response` - C must be a function with this signature
- `'static` - The function must not contain references with limited lifetimes
- `Box::new(callback)` - Converts the concrete type to a trait object

## Step 6: Implement request handling

```rust
impl BasicRouter {
    fn handle_request(&self, request: &Request) -> Response {
        match self.routes.get(&request.url) {
            None => not_found_response(),
            Some(callback) => callback(request)
        }
    }
}
```

**How it works:**
1. Look up the URL in the routes HashMap
2. If not found, return a 404 response
3. If found, call the handler function with the request

## Step 7: Create helper response functions

```rust
fn not_found_response() -> Response {
    Response {
        code: 404,
        headers: HashMap::new(),
        body: b"<h1>Page not found</h1>".to_vec()
    }
}

fn get_form_response() -> Response {
    Response {
        code: 200,
        headers: HashMap::new(),
        body: b"<form>".to_vec()
    }
}

fn get_gcd_response(_req: &Request) -> Response {
    Response {
        code: 500,
        headers: HashMap::new(),
        body: b"<h1>Internal server error</h1>".to_vec()
    }
}
```

These are simple handlers that return different responses.

## Complete example with usage

```rust
use std::collections::HashMap;

// ... [all the struct and impl definitions from above] ...

fn main() {
    let mut router = BasicRouter::new();

    // Add routes with closures
    router.add_route("/", |_| get_form_response());
    router.add_route("/gcd", |req| get_gcd_response(req));

    // Create a request
    let request = Request {
        method: "GET".to_string(),
        url: "/".to_string(),
        headers: HashMap::new(),
        body: vec![]
    };

    // Handle the request
    let response = router.handle_request(&request);
    println!("Response code: {}", response.code);
}
```

## Testing your router

```rust
fn req(url: &str) -> Request {
    Request {
        method: "GET".to_string(),
        url: url.to_string(),
        headers: HashMap::new(),
        body: vec![]
    }
}

#[test]
fn test_router() {
    let mut router = BasicRouter::new();
    router.add_route("/", |_| get_form_response());
    router.add_route("/gcd", |req| get_gcd_response(req));

    assert_eq!(router.handle_request(&req("/piano")).code, 404);
    assert_eq!(router.handle_request(&req("/")).code, 200);
    assert_eq!(router.handle_request(&req("/gcd")).code, 500);
}
```

Run tests with:
```bash
cargo test
```

## Advanced: Adding method routing

To handle different HTTP methods (GET, POST, etc.):

```rust
type RouteKey = (String, String);  // (method, url)

struct MethodRouter {
    routes: HashMap<RouteKey, BoxedCallback>
}

impl MethodRouter {
    fn add_route<C>(&mut self, method: &str, url: &str, callback: C)
        where C: Fn(&Request) -> Response + 'static
    {
        let key = (method.to_string(), url.to_string());
        self.routes.insert(key, Box::new(callback));
    }

    fn handle_request(&self, request: &Request) -> Response {
        let key = (request.method.clone(), request.url.clone());
        match self.routes.get(&key) {
            None => not_found_response(),
            Some(callback) => callback(request)
        }
    }
}
```

Usage:
```rust
let mut router = MethodRouter::new();
router.add_route("GET", "/users", |_| list_users());
router.add_route("POST", "/users", |req| create_user(req));
```

## Advanced: Pattern matching with path parameters

For routes like `/users/:id`:

```rust
struct PatternRouter {
    routes: Vec<(String, BoxedCallback)>
}

impl PatternRouter {
    fn add_route<C>(&mut self, pattern: &str, callback: C)
        where C: Fn(&Request) -> Response + 'static
    {
        self.routes.push((pattern.to_string(), Box::new(callback)));
    }

    fn handle_request(&self, request: &Request) -> Response {
        for (pattern, callback) in &self.routes {
            if self.matches(pattern, &request.url) {
                return callback(request);
            }
        }
        not_found_response()
    }

    fn matches(&self, pattern: &str, url: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let url_parts: Vec<&str> = url.split('/').collect();

        if pattern_parts.len() != url_parts.len() {
            return false;
        }

        for (p, u) in pattern_parts.iter().zip(url_parts.iter()) {
            if p.starts_with(':') {
                // This is a parameter, matches anything
                continue;
            }
            if p != u {
                return false;
            }
        }
        true
    }
}
```

## Comparison to Flask routing (Python)

### Flask version:

```python
from flask import Flask, request

app = Flask(__name__)

@app.route('/')
def index():
    return '<form></form>'

@app.route('/gcd', methods=['POST'])
def gcd():
    return '<h1>Result</h1>'

if __name__ == '__main__':
    app.run()
```

### Rust version:

```rust
let mut router = BasicRouter::new();

router.add_route("/", |_| Response {
    code: 200,
    headers: HashMap::new(),
    body: b"<form></form>".to_vec()
});

router.add_route("/gcd", |_| Response {
    code: 200,
    headers: HashMap::new(),
    body: b"<h1>Result</h1>".to_vec()
});
```

**Key differences:**
- Flask uses decorators (`@app.route`), Rust uses explicit method calls
- Flask handles serialization automatically, Rust requires manual conversion
- Flask routes are registered at module load time, Rust at runtime
- Rust requires explicit types, Python uses duck typing

## Comparison to FastAPI routing (Python)

### FastAPI version:

```python
from fastapi import FastAPI

app = FastAPI()

@app.get("/")
async def root():
    return {"message": "Hello"}

@app.get("/users/{user_id}")
async def get_user(user_id: int):
    return {"user_id": user_id}
```

### Rust equivalent concept:

```rust
router.add_route("GET", "/", |_| {
    json_response(r#"{"message": "Hello"}"#)
});

router.add_route("GET", "/users/:user_id", |req| {
    let user_id = extract_param(req, "user_id");
    json_response(&format!(r#"{{"user_id": {}}}"#, user_id))
});
```

## Generic router design benefits

1. **Type safety** - Compile-time guarantees
2. **Zero-cost abstractions** - No runtime overhead
3. **Flexibility** - Works with any function type
4. **Extensibility** - Easy to add middleware or features
5. **No macros needed** - Unlike web frameworks, this uses plain Rust

## Common patterns

### Middleware-style routing

```rust
fn with_logging<F>(handler: F) -> impl Fn(&Request) -> Response
    where F: Fn(&Request) -> Response
{
    move |req| {
        println!("Request: {} {}", req.method, req.url);
        let response = handler(req);
        println!("Response: {}", response.code);
        response
    }
}

// Usage
router.add_route("/", with_logging(|_| get_form_response()));
```

### Shared state

```rust
use std::sync::Arc;

struct AppState {
    db_connection: String,
}

let state = Arc::new(AppState {
    db_connection: "localhost:5432".to_string(),
});

let state_clone = state.clone();
router.add_route("/", move |_req| {
    println!("DB: {}", state_clone.db_connection);
    get_form_response()
});
```

## Best practices

1. **Use exact matches first** - Before pattern matching
2. **Order routes by specificity** - Most specific first
3. **Return proper status codes** - 200, 404, 500, etc.
4. **Set Content-Type headers** - Tell clients what you're sending
5. **Handle all HTTP methods** - GET, POST, PUT, DELETE, etc.
6. **Validate input** - Check request data before processing
7. **Use type safety** - Let the compiler catch errors

## Limitations of this simple router

- No regex pattern matching
- No query parameter parsing
- No path parameter extraction
- No automatic content negotiation
- No built-in middleware support
- Exact URL matching only (no wildcards)

For production applications, use established frameworks like Actix-web, Rocket, or Axum that handle these cases.

## See also

- [How to build a web server with Actix](04-build-web-server-actix.md)
- [Actix-web routing documentation](https://actix.rs/docs/url-dispatch)
- [Rust closures and trait objects](https://doc.rust-lang.org/book/ch13-01-closures.html)
