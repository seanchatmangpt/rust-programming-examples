# Networking API Reference

This reference provides complete API documentation for networking examples, from simple HTTP clients to web servers.

## http-get - Blocking HTTP Client

Simple command-line HTTP GET client using the `reqwest` blocking API.

### Command-Line Interface

```
http-get URL
```

**Arguments:**
- `URL`: HTTP or HTTPS URL to fetch

**Output:**
- Writes response body to stdout
- Errors written to stderr

**Exit Codes:**
- `0`: Success
- Non-zero: HTTP error or network failure

**Example:**
```bash
$ http-get https://example.com > page.html
$ http-get https://api.github.com/users/rust-lang
```

### Function API

#### `http_get_main`

Performs an HTTP GET request and writes the response to stdout.

**Signature:**
```rust
fn http_get_main(url: &str) -> Result<(), Box<dyn Error>>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | `&str` | URL to fetch |

**Returns:**
- `Ok(())`: Request succeeded and response written
- `Err(Box<dyn Error>)`: Network error or non-success status code

**Dependencies:**
- `reqwest::blocking::get` - Blocking HTTP client
- `std::io::copy` - Streaming response to stdout

**Example:**
```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    http_get_main("https://example.com")?;
    Ok(())
}
```

**Error Cases:**
- Network connection failure
- DNS resolution failure
- HTTP status code other than 2xx
- I/O error writing to stdout

---

## cheapo-request - Async HTTP Client

Minimal async HTTP client implementation using raw TCP sockets and `async-std`.

### Function API

#### `cheapo_request`

Performs a simple HTTP/1.1 GET request over a TCP connection.

**Signature:**
```rust
async fn cheapo_request(host: &str, port: u16, path: &str)
    -> std::io::Result<String>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `host` | `&str` | Hostname or IP address |
| `port` | `u16` | TCP port number |
| `path` | `&str` | HTTP path (e.g., `"/"`) |

**Returns:**
- `Ok(String)`: Full HTTP response (headers + body)
- `Err(std::io::Error)`: Network or I/O error

**Behavior:**
1. Establishes TCP connection to `host:port`
2. Sends minimal HTTP/1.1 GET request
3. Shuts down write side of connection
4. Reads entire response into `String`
5. Closes connection

**HTTP Request Format:**
```
GET {path} HTTP/1.1\r\n
Host: {host}\r\n
\r\n
```

**Example:**
```rust
use async_std::task;

let response = task::block_on(async {
    cheapo_request("example.com", 80, "/").await
})?;
println!("{}", response);
```

**Limitations:**
- No HTTPS support
- No request headers (except Host)
- No connection reuse
- Reads entire response into memory
- No timeout handling

**Use Case:**
- Educational example
- Demonstrates async I/O with TCP
- Shows raw HTTP protocol

---

## basic-router - Simple HTTP Router

A basic HTTP request router demonstrating closures and trait objects.

### Types

#### `Request`

HTTP request representation.

**Definition:**
```rust
struct Request {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Vec<u8>
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `method` | `String` | HTTP method (GET, POST, etc.) |
| `url` | `String` | Request URL/path |
| `headers` | `HashMap<String, String>` | HTTP headers |
| `body` | `Vec<u8>` | Request body bytes |

---

#### `Response`

HTTP response representation.

**Definition:**
```rust
struct Response {
    code: u32,
    headers: HashMap<String, String>,
    body: Vec<u8>
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `code` | `u32` | HTTP status code |
| `headers` | `HashMap<String, String>` | Response headers |
| `body` | `Vec<u8>` | Response body bytes |

---

#### `BoxedCallback`

Type alias for route handler functions.

**Definition:**
```rust
type BoxedCallback = Box<dyn Fn(&Request) -> Response>
```

**Description:**
- Boxed trait object for dynamic dispatch
- Takes `&Request`, returns `Response`
- Stored in router's route table

---

### BasicRouter

Main router structure.

**Definition:**
```rust
struct BasicRouter {
    routes: HashMap<String, BoxedCallback>
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `routes` | `HashMap<String, BoxedCallback>` | URL to handler mapping |

---

### Methods

#### `new`

Creates a new empty router.

**Signature:**
```rust
fn new() -> BasicRouter
```

**Returns:**
- `BasicRouter`: Empty router with no routes

**Example:**
```rust
let mut router = BasicRouter::new();
```

---

#### `add_route`

Registers a route handler for a specific URL.

**Signature:**
```rust
fn add_route<C>(&mut self, url: &str, callback: C)
where
    C: Fn(&Request) -> Response + 'static
```

**Type Parameters:**
| Parameter | Bounds | Description |
|-----------|--------|-------------|
| `C` | `Fn(&Request) -> Response` | Callback function type |
| `C` | `'static` | Must not capture borrowed references |

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | `&str` | URL path to match |
| `callback` | `C` | Handler function |

**Example:**
```rust
router.add_route("/", |_| Response {
    code: 200,
    headers: HashMap::new(),
    body: b"<h1>Home</h1>".to_vec()
});

router.add_route("/api/users", |req| {
    // Handle request
    Response { code: 200, headers: HashMap::new(), body: vec![] }
});
```

**Closure Capture:**
```rust
// Can capture owned data
let message = String::from("Hello");
router.add_route("/hello", move |_| Response {
    code: 200,
    headers: HashMap::new(),
    body: message.as_bytes().to_vec()
});

// Cannot capture references (wouldn't compile)
// let data = String::from("test");
// router.add_route("/bad", |_| use_data(&data));  // Error: 'static
```

---

#### `handle_request`

Dispatches a request to the appropriate handler.

**Signature:**
```rust
fn handle_request(&self, request: &Request) -> Response
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `request` | `&Request` | Incoming HTTP request |

**Returns:**
- `Response`: Handler response or 404 if no route matches

**Behavior:**
- Looks up handler by exact URL match
- Returns handler response if found
- Returns 404 response if not found

**Example:**
```rust
let request = Request {
    method: "GET".to_string(),
    url: "/api/users".to_string(),
    headers: HashMap::new(),
    body: vec![]
};

let response = router.handle_request(&request);
assert_eq!(response.code, 200);
```

---

### Helper Functions

#### `not_found_response`

Creates a 404 Not Found response.

**Signature:**
```rust
fn not_found_response() -> Response
```

**Returns:**
- `Response`: 404 response with HTML body

**Response:**
```rust
Response {
    code: 404,
    headers: HashMap::new(),
    body: b"<h1>Page not found</h1>".to_vec()
}
```

---

## actix-gcd - Actix Web Server

Web server using the Actix Web framework to provide a GCD calculator service.

### Server Configuration

**Bind Address:** `127.0.0.1:3000`

**Routes:**
| Route | Method | Handler | Description |
|-------|--------|---------|-------------|
| `/` | GET | `get_index` | HTML form |
| `/gcd` | POST | `post_gcd` | GCD calculation |

---

### Route Handlers

#### `get_index`

Serves the HTML form for GCD input.

**Signature:**
```rust
async fn get_index() -> HttpResponse
```

**Returns:**
- `HttpResponse`: HTML form with status 200

**Response Content:**
```html
<title>GCD Calculator</title>
<form action="/gcd" method="post">
<input type="text" name="n"/>
<input type="text" name="m"/>
<button type="submit">Compute GCD</button>
</form>
```

**Content-Type:** `text/html`

---

#### `post_gcd`

Processes GCD calculation from form submission.

**Signature:**
```rust
async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `form` | `web::Form<GcdParameters>` | Deserialized form data |

**Returns:**
- `HttpResponse`: Calculation result (200) or error (400)

**Validation:**
- Returns 400 Bad Request if either `n` or `m` is zero

**Success Response:**
```
The greatest common divisor of the numbers {n} and {m} is <b>{result}</b>
```

**Error Response:**
```
Computing the GCD with zero is boring.
```

**Example Request:**
```http
POST /gcd HTTP/1.1
Content-Type: application/x-www-form-urlencoded

n=48&m=18
```

**Example Response:**
```http
HTTP/1.1 200 OK
Content-Type: text/html

The greatest common divisor of the numbers 48 and 18 is <b>6</b>
```

---

### Form Types

#### `GcdParameters`

Form data structure for GCD inputs.

**Definition:**
```rust
#[derive(Deserialize)]
struct GcdParameters {
    n: u64,
    m: u64,
}
```

**Derives:**
- `Deserialize` - Automatic form parsing by Serde

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `n` | `u64` | First number |
| `m` | `u64` | Second number |

**Form Encoding:**
- Parsed from `application/x-www-form-urlencoded`
- Field names must match struct field names

---

### Core Functions

#### `gcd`

Computes the greatest common divisor.

**Signature:**
```rust
fn gcd(mut n: u64, mut m: u64) -> u64
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `n` | `u64` | First positive integer |
| `m` | `u64` | Second positive integer |

**Returns:**
- `u64`: Greatest common divisor

**Panics:**
- If `n == 0` or `m == 0`

**Algorithm:**
- Euclidean algorithm
- Time complexity: O(log(min(n, m)))

---

### Server Lifecycle

#### `main`

Starts the Actix web server.

**Signature:**
```rust
#[actix_web::main]
async fn main()
```

**Attributes:**
- `#[actix_web::main]` - Actix runtime initialization macro

**Configuration:**
- Creates `HttpServer` with route configuration
- Binds to `127.0.0.1:3000`
- Runs server until interrupted

**Example Usage:**
```bash
$ cargo run
Serving on http://localhost:3000...
```

---

## Networking Patterns

### Blocking vs Async

| Project | Model | Runtime | Use Case |
|---------|-------|---------|----------|
| http-get | Blocking | None | Simple scripts |
| cheapo-request | Async | async-std | Learning async |
| actix-gcd | Async | Actix | Production web servers |

### Error Handling Strategies

**http-get:**
```rust
Result<(), Box<dyn Error>>  // Flexible error type
```

**cheapo-request:**
```rust
std::io::Result<String>  // Specific error type
```

**actix-gcd:**
```rust
HttpResponse  // HTTP-level error responses
```

### Route Handler Patterns

**Static closure (basic-router):**
```rust
router.add_route("/", |_| get_form_response());
```

**Move closure (basic-router):**
```rust
let data = String::from("data");
router.add_route("/data", move |_| {
    Response { body: data.clone().into_bytes(), ..Default::default() }
});
```

**Async function (actix-gcd):**
```rust
async fn get_index() -> HttpResponse { ... }
```

### Request/Response Flow

**Basic Router:**
```
Request -> Router::handle_request -> HashMap lookup -> Callback -> Response
```

**Actix Web:**
```
HTTP Request -> Actix Router -> Handler Function -> HttpResponse -> HTTP Response
```

## Dependencies Summary

| Project | Key Dependencies | Purpose |
|---------|-----------------|---------|
| http-get | `reqwest` (blocking) | HTTP client |
| cheapo-request | `async-std` | Async runtime and TCP |
| basic-router | (none) | Pure Rust demonstration |
| actix-gcd | `actix-web`, `serde` | Web framework, serialization |

## Security Considerations

**Input Validation:**
- actix-gcd: Validates non-zero inputs
- basic-router: No validation (example only)

**Type Safety:**
- Serde deserialization prevents type errors
- Compile-time route handler type checking

**Memory Safety:**
- No unsafe code in networking examples
- Actix handles connection lifecycle safely
