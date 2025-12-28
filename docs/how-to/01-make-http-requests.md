# How to Make HTTP GET Requests in Rust

## Overview

This guide shows you how to make HTTP GET requests in Rust using the `reqwest` crate, handle responses, and deal with errors. If you're coming from Python's `requests` library, you'll find similar concepts with Rust's focus on explicit error handling.

## Prerequisites

- Basic Rust knowledge
- Familiarity with HTTP concepts
- Cargo installed

## Step 1: Add the reqwest dependency

Add `reqwest` to your `Cargo.toml`:

```toml
[dependencies]
reqwest = { version = "0.11", features = ["blocking"] }
```

**Why the "blocking" feature?** Reqwest is async by default. The `blocking` feature provides a synchronous API that's easier to use in simple CLI tools or when you don't need async.

## Step 2: Import the necessary modules

```rust
use std::error::Error;
use std::io;
```

The `Error` trait allows flexible error handling, and `io` provides utilities for reading/writing data.

## Step 3: Make the HTTP GET request

Here's a complete function that fetches a URL:

```rust
fn http_get_main(url: &str) -> Result<(), Box<dyn Error>> {
    // Send the HTTP request and get a response.
    let mut response = reqwest::blocking::get(url)?;

    // Check if the request was successful
    if !response.status().is_success() {
        Err(format!("{}", response.status()))?;
    }

    // Read the response body and write it to stdout.
    let stdout = io::stdout();
    io::copy(&mut response, &mut stdout.lock())?;

    Ok(())
}
```

**Key points:**
- `reqwest::blocking::get(url)?` sends the request and returns a `Response` object
- The `?` operator propagates errors up the call stack
- `response.status().is_success()` checks for 2xx status codes
- `io::copy()` efficiently streams the response to stdout

## Step 4: Handle errors properly

Rust requires explicit error handling. Here's how to use your function:

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: http-get URL");
        return;
    }

    if let Err(err) = http_get_main(&args[1]) {
        eprintln!("error: {}", err);
    }
}
```

This pattern:
1. Validates command-line arguments
2. Calls the function and captures any errors
3. Prints errors to stderr using `eprintln!`

## Common error scenarios

### Network errors

```rust
// Network timeout or connection refused
let response = reqwest::blocking::get("http://localhost:9999")?;
// Returns: Err(...)
```

### HTTP errors

```rust
// 404 Not Found
let mut response = reqwest::blocking::get("https://example.com/nonexistent")?;
if !response.status().is_success() {
    Err(format!("{}", response.status()))?;  // Propagates "404 Not Found"
}
```

### Invalid URLs

```rust
// Malformed URL
let response = reqwest::blocking::get("not a url")?;
// Returns: Err(...)
```

## Comparison to Python's requests library

### Python version:

```python
import requests
import sys

try:
    response = requests.get(sys.argv[1])
    response.raise_for_status()
    print(response.text)
except Exception as e:
    print(f"error: {e}", file=sys.stderr)
```

### Rust version:

```rust
use std::error::Error;
use std::io;

fn http_get_main(url: &str) -> Result<(), Box<dyn Error>> {
    let mut response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        Err(format!("{}", response.status()))?;
    }
    let stdout = io::stdout();
    io::copy(&mut response, &mut stdout.lock())?;
    Ok(())
}
```

**Key differences:**
- Rust uses `Result` types instead of exceptions
- Rust streams the response instead of loading it all into memory
- Rust requires explicit error propagation with `?`
- Python is more concise but less explicit about errors

## Advanced usage

### Reading response as text

```rust
let response_text = reqwest::blocking::get(url)?.text()?;
println!("{}", response_text);
```

### Reading response as JSON

Add `serde` and `serde_json` to your dependencies:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct ApiResponse {
    field1: String,
    field2: i32,
}

let response: ApiResponse = reqwest::blocking::get(url)?.json()?;
```

### Setting headers

```rust
let client = reqwest::blocking::Client::new();
let response = client
    .get(url)
    .header("User-Agent", "my-app/1.0")
    .header("Authorization", "Bearer token123")
    .send()?;
```

### Setting timeouts

```rust
use std::time::Duration;

let client = reqwest::blocking::Client::builder()
    .timeout(Duration::from_secs(10))
    .build()?;
let response = client.get(url).send()?;
```

## Running the example

From the `http-get` directory:

```bash
cargo run https://example.com
```

This will fetch and print the HTML from example.com.

## Troubleshooting

### SSL/TLS errors

If you get certificate errors, ensure your system's CA certificates are up to date. On Linux:

```bash
sudo apt-get update
sudo apt-get install ca-certificates
```

### Compilation errors with features

Make sure you have the `blocking` feature enabled if using synchronous code:

```toml
reqwest = { version = "0.11", features = ["blocking"] }
```

## Best practices

1. **Use the `?` operator**: Don't unwrap() in production code
2. **Stream large responses**: Use `io::copy()` instead of loading everything into memory
3. **Check status codes**: Always verify `response.status().is_success()`
4. **Set timeouts**: Prevent hanging on slow connections
5. **Reuse the client**: Create one `Client` instance and reuse it for multiple requests

## See also

- [reqwest documentation](https://docs.rs/reqwest/)
- [How to build a simple HTTP client](02-build-simple-http-client.md) - for low-level understanding
- [Rust error handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
