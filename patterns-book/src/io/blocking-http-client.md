# Blocking HTTP Client

## Context

You are building a command-line tool, script, or simple application that needs to make HTTP requests to fetch data, call APIs, or download files. The tool runs synchronously—it waits for each request to complete before continuing. You don't need concurrent requests or complex async runtime management.

Your use case might be a web scraper, API client, file downloader, webhook tester, or any utility where HTTP communication is required but doesn't need to be asynchronous.

## Problem

**How do you make synchronous HTTP requests in Rust without dealing with async/await complexity, while maintaining proper error handling, response validation, and ergonomic API usage?**

Writing HTTP requests from scratch using `TcpStream` is tedious and error-prone. Async HTTP clients require understanding futures, runtimes, and async/await. You need a simple, blocking API that handles HTTP protocol details, connection management, redirects, and common content types automatically.

## Forces

- **Simplicity**: Want straightforward, synchronous code without async complexity
- **Protocol handling**: HTTP is complex (headers, status codes, redirects, chunked encoding)
- **Error handling**: Network errors, HTTP errors, and parsing errors must be handled
- **Connection reuse**: Should reuse connections for efficiency (connection pooling)
- **HTTPS support**: Need TLS/SSL for secure connections
- **Response handling**: Must read response bodies efficiently
- **Content types**: Should handle JSON, text, binary data appropriately
- **Ergonomics**: API should be simple for common use cases

## Solution

**Use the `reqwest` crate with the `blocking` feature to make synchronous HTTP requests.** Create a client or use the convenience functions, call HTTP methods (GET, POST, etc.), check the response status, and read the response body with appropriate methods.

### Structure

```rust
use reqwest::blocking;
use std::error::Error;

fn fetch_url(url: &str) -> Result<String, Box<dyn Error>> {
    let response = blocking::get(url)?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let body = response.text()?;
    Ok(body)
}
```

### Real Implementation (from http-get)

```rust
use std::error::Error;
use std::io;

fn http_get_main(url: &str) -> Result<(), Box<dyn Error>> {
    // Send the HTTP request and get a response.
    let mut response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        Err(format!("{}", response.status()))?;
    }

    // Read the response body and write it to stdout.
    let stdout = io::stdout();
    io::copy(&mut response, &mut stdout.lock())?;

    Ok(())
}

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

### Key Elements

1. **reqwest::blocking module**: Synchronous HTTP client API
2. **Convenience function**: `blocking::get(url)` for simple GET requests
3. **Response object**: Contains status, headers, and body
4. **Status checking**: `response.status().is_success()` validates HTTP response
5. **Body streaming**: `io::copy(&mut response, &mut writer)` streams without loading into memory
6. **Error propagation**: Network, HTTP, and I/O errors all return as `Result`

### Cargo Dependency

```toml
[dependencies]
reqwest = { version = "0.11", features = ["blocking"] }
```

The `blocking` feature enables synchronous API. Without it, reqwest is async-only.

## Resulting Context

### Benefits

- **Simple API**: No async/await complexity for synchronous use cases
- **Full HTTP support**: Handles all HTTP methods, redirects, compression, cookies
- **Connection pooling**: Automatically reuses connections for performance
- **HTTPS support**: Built-in TLS support via rustls or native-tls
- **Error handling**: Unified error type for all failure modes
- **Streaming**: Can stream large responses without loading into memory
- **Widely used**: Battle-tested library with extensive ecosystem support

### Liabilities

- **Blocking threads**: Blocks the calling thread during request
- **No concurrency**: Can't handle multiple requests simultaneously (without threads)
- **Resource usage**: Each concurrent request needs a thread
- **Binary size**: Adds significant dependencies (TLS, HTTP parser)
- **Not for async**: If you need async, must use non-blocking reqwest API

### Performance Characteristics

- **Connection pooling**: Reuses TCP connections, reducing latency
- **Compression**: Automatic gzip/deflate decompression
- **Memory**: Streaming avoids loading entire response into memory
- **Latency**: Network round-trip time is the dominant factor

## Variations

### Using Client for Configuration

```rust
use reqwest::blocking::Client;
use std::time::Duration;

fn create_configured_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("MyApp/1.0")
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
}

fn fetch_with_client(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = create_configured_client()?;
    let response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }

    let text = response.text()?;
    Ok(text)
}
```

### POST Request with JSON

```rust
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct UserResponse {
    id: u32,
    name: String,
}

fn create_user(name: &str, email: &str) -> Result<UserResponse, Box<dyn std::error::Error>> {
    let client = Client::new();

    let user = CreateUser {
        name: name.to_string(),
        email: email.to_string(),
    };

    let response = client
        .post("https://api.example.com/users")
        .json(&user)
        .send()?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }

    let user_response: UserResponse = response.json()?;
    Ok(user_response)
}
```

### Download File to Disk

```rust
use std::fs::File;
use std::io::copy;
use reqwest::blocking;

fn download_file(url: &str, path: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let mut response = blocking::get(url)?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }

    let mut file = File::create(path)?;
    let bytes_copied = copy(&mut response, &mut file)?;

    Ok(bytes_copied)
}

// Usage:
// download_file("https://example.com/file.zip", "download.zip")?;
```

### Custom Headers and Authentication

```rust
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

fn fetch_with_auth(url: &str, token: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json")
    );

    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    let response = client.get(url).send()?;
    let text = response.text()?;
    Ok(text)
}
```

### Form Data Submission

```rust
use reqwest::blocking::Client;
use std::collections::HashMap;

fn submit_form(url: &str, data: &HashMap<&str, &str>) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let response = client.post(url).form(data).send()?;
    let text = response.text()?;
    Ok(text)
}

// Usage:
// let mut form = HashMap::new();
// form.insert("username", "alice");
// form.insert("password", "secret");
// submit_form("https://example.com/login", &form)?;
```

### Handling Different Status Codes

```rust
use reqwest::blocking;
use reqwest::StatusCode;

fn fetch_with_status_handling(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = blocking::get(url)?;

    match response.status() {
        StatusCode::OK => {
            let text = response.text()?;
            Ok(text)
        }
        StatusCode::NOT_FOUND => {
            Err("Resource not found".into())
        }
        StatusCode::UNAUTHORIZED => {
            Err("Authentication required".into())
        }
        StatusCode::TOO_MANY_REQUESTS => {
            Err("Rate limit exceeded".into())
        }
        status => {
            Err(format!("Unexpected status: {}", status).into())
        }
    }
}
```

## Related Patterns

- **Error Propagation**: Essential for handling network and HTTP errors
- **Argument Parsing**: Often combined to get URLs from command line
- **Line-Oriented Processing**: Can process response body line by line
- **TCP Server Loop**: Complementary pattern for building servers

## Known Uses

### Standard Use Cases

- **CLI tools**: Fetch data from APIs, download files
- **Web scrapers**: Extract data from web pages
- **Health checkers**: Monitor service availability
- **Webhook testers**: Send test requests to webhooks
- **API clients**: Interface with REST APIs

### Real Projects

```rust
// Weather fetcher
use serde::Deserialize;

#[derive(Deserialize)]
struct Weather {
    temperature: f32,
    description: String,
}

fn get_weather(city: &str, api_key: &str) -> Result<Weather, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.weather.com/v1/current?city={}&key={}",
        city, api_key
    );

    let response = reqwest::blocking::get(&url)?;
    let weather: Weather = response.json()?;
    Ok(weather)
}

// RSS feed reader
fn fetch_rss_feed(url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let text = response.text()?;

    // Parse XML feed (simplified)
    let items = parse_rss(&text)?;
    Ok(items)
}

// Link checker
fn check_link(url: &str) -> bool {
    match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .and_then(|client| client.head(url).send())
    {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

// Batch image downloader
fn download_images(urls: &[&str], dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    for (i, url) in urls.iter().enumerate() {
        let filename = format!("{}/image_{}.jpg", dir, i);
        download_file(url, &filename)?;
        println!("Downloaded {}", filename);
    }
    Ok(())
}
```

### Integration with Other Patterns

```rust
// Combined with argument parsing and error handling
use std::error::Error;
use std::env;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <url>", args[0]);
        std::process::exit(1);
    }

    let url = &args[1];
    let response = reqwest::blocking::get(url)?;

    println!("Status: {}", response.status());
    println!("Headers: {:#?}", response.headers());

    let body = response.text()?;
    println!("\nBody:\n{}", body);

    Ok(())
}
```

## Implementation Notes

### Error Handling

```rust
use reqwest;

fn robust_fetch(url: &str) -> Result<String, String> {
    let response = reqwest::blocking::get(url)
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response.text()
        .map_err(|e| format!("Failed to read response: {}", e))
}
```

### Retry Logic

```rust
use std::time::Duration;
use std::thread;

fn fetch_with_retry(url: &str, max_retries: u32) -> Result<String, Box<dyn std::error::Error>> {
    let mut attempts = 0;

    loop {
        match reqwest::blocking::get(url) {
            Ok(response) if response.status().is_success() => {
                return response.text().map_err(Into::into);
            }
            Ok(response) => {
                if attempts >= max_retries {
                    return Err(format!("HTTP {}", response.status()).into());
                }
            }
            Err(e) => {
                if attempts >= max_retries {
                    return Err(e.into());
                }
            }
        }

        attempts += 1;
        thread::sleep(Duration::from_secs(2_u64.pow(attempts)));
    }
}
```

### Progress Tracking

```rust
use std::io::{self, Read, Write};
use std::fs::File;

fn download_with_progress(url: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut response = reqwest::blocking::get(url)?;
    let total_size = response.content_length().unwrap_or(0);

    let mut file = File::create(path)?;
    let mut downloaded = 0u64;
    let mut buffer = [0; 8192];

    loop {
        let n = response.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        file.write_all(&buffer[..n])?;
        downloaded += n as u64;

        if total_size > 0 {
            let percent = (downloaded as f64 / total_size as f64) * 100.0;
            print!("\rProgress: {:.1}%", percent);
            io::stdout().flush()?;
        }
    }

    println!();
    Ok(())
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[test]
    fn test_successful_fetch() {
        let mock = mockito::mock("GET", "/test")
            .with_status(200)
            .with_body("test response")
            .create();

        let url = format!("{}/test", mockito::server_url());
        let result = fetch_url(&url).unwrap();

        assert_eq!(result, "test response");
        mock.assert();
    }

    #[test]
    fn test_http_error() {
        let mock = mockito::mock("GET", "/error")
            .with_status(404)
            .create();

        let url = format!("{}/error", mockito::server_url());
        let result = fetch_url(&url);

        assert!(result.is_err());
        mock.assert();
    }
}
```

### When to Use Async Instead

Consider async reqwest when:
- Making many concurrent requests
- Building a web server or service
- Integrating with async frameworks (tokio, actix)
- Need to do other work while waiting for responses

```rust
// Async version for comparison
async fn fetch_async(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    Ok(text)
}
```

## References

- reqwest documentation: https://docs.rs/reqwest
- HTTP status codes: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status
- "Programming Rust" Chapter 18: Input and Output
- Rust Cookbook - Web Programming: https://rust-lang-nursery.github.io/rust-cookbook/web.html
