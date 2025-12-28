# How to Build a Simple HTTP Client in Rust

## Overview

This guide shows you how to build a minimal HTTP client from scratch using TCP sockets. You'll learn how HTTP works at the protocol level by manually constructing requests and parsing responses. This is useful for understanding web protocols and deciding when to use low-level vs high-level libraries.

## Prerequisites

- Basic Rust knowledge
- Understanding of TCP/IP concepts
- Familiarity with HTTP protocol basics
- Async programming concepts (we'll use async-std)

## Step 1: Add the async-std dependency

Add `async-std` to your `Cargo.toml`:

```toml
[dependencies]
async-std = "1.7"
```

**Why async-std?** It provides async versions of standard library types like `TcpStream`, making network I/O non-blocking.

## Step 2: Import required modules

```rust
use async_std::io::prelude::*;
use async_std::net;
```

These imports give us:
- `Read` and `Write` traits for I/O operations
- `TcpStream` for TCP connections

## Step 3: Create the TCP connection

```rust
async fn cheapo_request(host: &str, port: u16, path: &str)
                            -> std::io::Result<String>
{
    let mut socket = net::TcpStream::connect((host, port)).await?;

    // ... rest of the function
}
```

**How it works:**
- `TcpStream::connect()` establishes a TCP connection to the server
- The `(host, port)` tuple specifies the destination
- `.await?` waits for the connection and propagates errors
- Port 80 is standard for HTTP, 443 for HTTPS

## Step 4: Construct the HTTP request

HTTP requests are plain text following a specific format:

```rust
let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
socket.write_all(request.as_bytes()).await?;
socket.shutdown(net::Shutdown::Write)?;
```

**Breaking down the HTTP request:**
```
GET / HTTP/1.1\r\n
Host: example.com\r\n
\r\n
```

- Line 1: `GET` method, `/` path, `HTTP/1.1` version
- Line 2: `Host:` header (required in HTTP/1.1)
- Line 3: Empty line signals end of headers
- `\r\n` is the HTTP line terminator (carriage return + line feed)

**Why shutdown?** Calling `shutdown(Write)` tells the server we're done sending data, which signals it to start processing the request.

## Step 5: Read the response

```rust
let mut response = String::new();
socket.read_to_string(&mut response).await?;

Ok(response)
```

This reads the entire response into a String. The server will close the connection when done, signaling EOF.

## Step 6: Run the async function

```rust
fn main() -> std::io::Result<()> {
    use async_std::task;

    let response = task::block_on(cheapo_request("example.com", 80, "/"))?;
    println!("{}", response);
    Ok(())
}
```

`task::block_on()` runs the async function and blocks until it completes. This bridges sync and async code.

## Complete example

Here's the full working code:

```rust
use async_std::io::prelude::*;
use async_std::net;

async fn cheapo_request(host: &str, port: u16, path: &str)
                            -> std::io::Result<String>
{
    let mut socket = net::TcpStream::connect((host, port)).await?;

    let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
    socket.write_all(request.as_bytes()).await?;
    socket.shutdown(net::Shutdown::Write)?;

    let mut response = String::new();
    socket.read_to_string(&mut response).await?;

    Ok(response)
}

fn main() -> std::io::Result<()> {
    use async_std::task;

    let response = task::block_on(cheapo_request("example.com", 80, "/"))?;
    println!("{}", response);
    Ok(())
}
```

## Understanding the HTTP response

When you run this, you'll see output like:

```
HTTP/1.1 200 OK
Content-Type: text/html; charset=UTF-8
Content-Length: 1256
Date: Tue, 28 Dec 2025 10:30:00 GMT

<!doctype html>
<html>
...
</html>
```

**Response structure:**
1. Status line: `HTTP/1.1 200 OK`
2. Headers: Key-value pairs (Content-Type, Content-Length, etc.)
3. Empty line: Separates headers from body
4. Body: The actual content (HTML, JSON, etc.)

## When to use low-level vs high-level libraries

### Use low-level TCP (like this example) when:

- Learning how HTTP works
- Implementing custom protocols
- Need maximum control over the connection
- Debugging network issues
- Building protocol testing tools

### Use high-level libraries (like reqwest) when:

- Building production applications
- Need HTTPS/TLS support
- Want automatic redirect handling
- Need cookie management
- Require connection pooling
- Want timeout and retry logic

## Limitations of this simple client

This basic implementation doesn't handle:

1. **HTTPS/TLS encryption** - Only works with plain HTTP
2. **Redirects** - Doesn't follow 301/302 responses
3. **Chunked encoding** - Can't parse chunked transfer encoding
4. **Keep-alive** - Opens a new connection for each request
5. **Timeouts** - Will hang if server doesn't respond
6. **Large responses** - Loads everything into memory
7. **Authentication** - No support for auth headers
8. **Cookies** - Doesn't store or send cookies

## Adding more features

### Adding a custom header

```rust
let request = format!(
    "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: MyClient/1.0\r\n\r\n",
    path, host
);
```

### POST request with body

```rust
let body = "key1=value1&key2=value2";
let request = format!(
    "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n{}",
    path, host, body.len(), body
);
```

### Parsing the response status

```rust
let first_line = response.lines().next().unwrap();
let status_code = first_line.split_whitespace().nth(1).unwrap();
println!("Status: {}", status_code);
```

### Separating headers and body

```rust
let parts: Vec<&str> = response.split("\r\n\r\n").collect();
let headers = parts[0];
let body = parts[1];
```

## Comparison to higher-level libraries

### This low-level approach:

```rust
let mut socket = net::TcpStream::connect(("example.com", 80)).await?;
let request = format!("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n");
socket.write_all(request.as_bytes()).await?;
socket.shutdown(net::Shutdown::Write)?;
let mut response = String::new();
socket.read_to_string(&mut response).await?;
```

### Using reqwest:

```rust
let response = reqwest::get("http://example.com").await?.text().await?;
```

The high-level library handles all the protocol details for you.

## Troubleshooting

### "Connection refused" error

The server isn't listening on that port. Check:
- Is the hostname correct?
- Is port 80 open?
- Is there a firewall blocking the connection?

### Response truncated or empty

The server might require additional headers or use a different protocol version.

### "Invalid UTF-8" error

The response might contain binary data. Use `read_to_end()` with a `Vec<u8>` instead:

```rust
let mut response = Vec::new();
socket.read_to_end(&mut response).await?;
```

## Best practices

1. **Always shutdown the write side** - Signals to server you're done sending
2. **Use async for network I/O** - Prevents blocking the thread
3. **Handle errors properly** - Network operations can fail in many ways
4. **Set timeouts** - Don't wait forever for unresponsive servers
5. **Consider using a library** - For production, use reqwest or hyper

## Alternative: Using standard library (synchronous)

For a synchronous version using `std::net`:

```rust
use std::io::{Read, Write};
use std::net::TcpStream;

fn cheapo_request_sync(host: &str, port: u16, path: &str)
                       -> std::io::Result<String>
{
    let mut socket = TcpStream::connect((host, port))?;

    let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
    socket.write_all(request.as_bytes())?;
    socket.shutdown(std::net::Shutdown::Write)?;

    let mut response = String::new();
    socket.read_to_string(&mut response)?;

    Ok(response)
}
```

## See also

- [How to make HTTP requests](01-make-http-requests.md) - using the reqwest library
- [HTTP protocol specification](https://www.rfc-editor.org/rfc/rfc2616)
- [async-std documentation](https://docs.rs/async-std/)
