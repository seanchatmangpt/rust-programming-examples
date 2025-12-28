# How to Make Concurrent HTTP Requests

## Problem

You need to make multiple HTTP requests efficiently. Making them sequentially is too slow - if each request takes 200ms, 100 requests take 20 seconds. With concurrency, all 100 can complete in ~200ms.

## Solution

Use async/await with task spawning to make multiple HTTP requests concurrently, collecting results when all complete.

## Prerequisites

- Understanding of async/await syntax
- Basic HTTP knowledge
- Familiarity with `Result` and error handling

## Step-by-Step Guide

### 1. The Sequential Approach (Baseline)

First, see why sequential requests are slow:

```rust
fn sequential_requests(urls: &[&str]) -> Vec<Result<String, String>> {
    let mut results = vec![];

    for url in urls {
        // Each request blocks until complete
        let result = make_blocking_request(url);
        results.push(result);
    }

    results
}

// 3 requests × 200ms each = 600ms total
```

### 2. Using async-std with Raw TCP

For learning purposes, here's how to make HTTP requests with raw TCP:

```rust
use async_std::io::prelude::*;
use async_std::net::TcpStream;

async fn cheapo_request(host: &str, port: u16, path: &str) -> std::io::Result<String> {
    // Connect to the server
    let mut socket = TcpStream::connect((host, port)).await?;

    // Send HTTP request
    let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
    socket.write_all(request.as_bytes()).await?;
    socket.shutdown(std::net::Shutdown::Write)?;

    // Read response
    let mut response = String::new();
    socket.read_to_string(&mut response).await?;

    Ok(response)
}
```

### 3. Making Concurrent Requests with spawn_local

```rust
use async_std::task;

async fn many_requests(
    requests: Vec<(String, u16, String)>
) -> Vec<std::io::Result<String>> {
    let mut handles = vec![];

    // Spawn a task for each request
    for (host, port, path) in requests {
        handles.push(task::spawn_local(async move {
            cheapo_request(&host, port, &path).await
        }));
    }

    // Collect results
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}

fn main() {
    let requests = vec![
        ("example.com".to_string(), 80, "/".to_string()),
        ("www.red-bean.com".to_string(), 80, "/".to_string()),
        ("en.wikipedia.org".to_string(), 80, "/".to_string()),
    ];

    let results = task::block_on(many_requests(requests));

    for result in results {
        match result {
            Ok(response) => println!("Success:\n{}\n", response),
            Err(err) => eprintln!("Error: {}\n", err),
        }
    }
}
```

**Dependencies:**
```toml
[dependencies]
async-std = { version = "1.12", features = ["unstable"] }  # unstable for spawn_local
```

### 4. Using a Proper HTTP Client (surf)

For production, use a real HTTP client library:

```rust
use async_std::task;

pub async fn many_requests(
    urls: &[String]
) -> Vec<Result<String, surf::Error>> {
    // Create a shared client (enables connection pooling)
    let client = surf::Client::new();

    let mut handles = vec![];

    // Spawn a task for each request
    for url in urls {
        let request = client.get(url).recv_string();
        handles.push(task::spawn(request));
    }

    // Collect results
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}

fn main() {
    let urls = &[
        "http://example.com".to_string(),
        "https://www.red-bean.com".to_string(),
        "https://en.wikipedia.org/wiki/Main_Page".to_string(),
    ];

    let results = task::block_on(many_requests(urls));

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(response) => println!("Response {}:\n{}\n", i + 1, response),
            Err(err) => eprintln!("Error {}: {}\n", i + 1, err),
        }
    }
}
```

**Dependencies:**
```toml
[dependencies]
async-std = "1.12"
surf = "2.3"
```

### 5. Using futures::join_all

A cleaner approach using `join_all`:

```rust
use futures::future::join_all;

async fn many_requests(urls: &[String]) -> Vec<Result<String, surf::Error>> {
    let client = surf::Client::new();

    // Create futures for all requests
    let futures: Vec<_> = urls
        .iter()
        .map(|url| client.get(url).recv_string())
        .collect();

    // Wait for all to complete
    join_all(futures).await
}
```

**Dependencies:**
```toml
[dependencies]
async-std = "1.12"
surf = "2.3"
futures = "0.3"
```

### 6. Error Handling with Multiple Requests

Collect both successes and failures:

```rust
#[derive(Debug)]
struct RequestResult {
    url: String,
    result: Result<String, String>,
}

async fn many_requests_with_context(
    urls: &[String]
) -> Vec<RequestResult> {
    let client = surf::Client::new();
    let mut handles = vec![];

    for url in urls {
        let url_clone = url.clone();
        let request = client.get(url).recv_string();

        handles.push(task::spawn(async move {
            RequestResult {
                url: url_clone.clone(),
                result: request.await.map_err(|e| e.to_string()),
            }
        }));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}

fn main() {
    let urls = &[
        "http://example.com".to_string(),
        "http://this-will-fail.invalid".to_string(),
        "https://www.rust-lang.org".to_string(),
    ];

    let results = task::block_on(many_requests_with_context(urls));

    let successes = results.iter().filter(|r| r.result.is_ok()).count();
    let failures = results.iter().filter(|r| r.result.is_err()).count();

    println!("Completed: {} succeeded, {} failed", successes, failures);

    for result in results {
        match result.result {
            Ok(_) => println!("✓ {}", result.url),
            Err(e) => println!("✗ {}: {}", result.url, e),
        }
    }
}
```

### 7. Rate Limiting and Connection Pooling

Limit concurrent requests to avoid overwhelming servers:

```rust
use async_std::sync::Semaphore;
use std::sync::Arc;

async fn many_requests_rate_limited(
    urls: &[String],
    max_concurrent: usize,
) -> Vec<Result<String, surf::Error>> {
    let client = surf::Client::new();
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    let mut handles = vec![];

    for url in urls {
        let permit = semaphore.clone().acquire_arc().await;
        let url_clone = url.clone();

        handles.push(task::spawn(async move {
            let _permit = permit;  // Hold permit during request
            client.get(&url_clone).recv_string().await
        }));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}

fn main() {
    let urls: Vec<String> = (0..100)
        .map(|i| format!("http://example.com/page/{}", i))
        .collect();

    // Only 10 concurrent requests at a time
    let results = task::block_on(many_requests_rate_limited(&urls, 10));

    println!("Completed {} requests", results.len());
}
```

### 8. Connection Pooling

The `surf::Client` automatically pools connections:

```rust
async fn efficient_many_requests(urls: &[String]) -> Vec<Result<String, surf::Error>> {
    // Create ONE client instance - it pools connections
    let client = surf::Client::new();

    let futures: Vec<_> = urls
        .iter()
        .map(|url| {
            // Each request reuses connections from the pool
            client.get(url).recv_string()
        })
        .collect();

    join_all(futures).await
}

// Without connection pooling (creates new connection each time):
async fn inefficient_many_requests(urls: &[String]) -> Vec<Result<String, surf::Error>> {
    let futures: Vec<_> = urls
        .iter()
        .map(|url| {
            // Creates new client = new connection each time!
            surf::get(url).recv_string()
        })
        .collect();

    join_all(futures).await
}
```

### 9. Timeout Handling

Add timeouts to prevent hanging:

```rust
use async_std::future::timeout;
use std::time::Duration;

async fn request_with_timeout(
    url: &str,
    timeout_duration: Duration,
) -> Result<String, String> {
    let client = surf::Client::new();

    match timeout(timeout_duration, client.get(url).recv_string()).await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(e)) => Err(format!("Request failed: {}", e)),
        Err(_) => Err("Request timed out".to_string()),
    }
}

async fn many_requests_with_timeout(
    urls: &[String],
    timeout_secs: u64,
) -> Vec<Result<String, String>> {
    let timeout_duration = Duration::from_secs(timeout_secs);
    let futures: Vec<_> = urls
        .iter()
        .map(|url| request_with_timeout(url, timeout_duration))
        .collect();

    join_all(futures).await
}
```

### 10. Progress Tracking

Track progress as requests complete:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

async fn many_requests_with_progress(urls: &[String]) -> Vec<Result<String, surf::Error>> {
    let client = surf::Client::new();
    let completed = Arc::new(AtomicUsize::new(0));
    let total = urls.len();

    let futures: Vec<_> = urls
        .iter()
        .map(|url| {
            let url_clone = url.clone();
            let completed = completed.clone();
            let client = client.clone();

            async move {
                let result = client.get(&url_clone).recv_string().await;
                let count = completed.fetch_add(1, Ordering::SeqCst) + 1;
                println!("Progress: {}/{}", count, total);
                result
            }
        })
        .collect();

    join_all(futures).await
}
```

## Using reqwest (Alternative HTTP Client)

Another popular option is `reqwest`:

```rust
use reqwest;

async fn many_requests_reqwest(urls: &[String]) -> Vec<Result<String, reqwest::Error>> {
    let client = reqwest::Client::new();

    let futures: Vec<_> = urls
        .iter()
        .map(|url| {
            let client = client.clone();
            let url = url.clone();
            async move {
                client.get(&url).send().await?.text().await
            }
        })
        .collect();

    join_all(futures).await
}
```

**Dependencies:**
```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
reqwest = "0.11"
futures = "0.3"
```

## Performance Comparison to Python asyncio

**Python (with aiohttp):**
```python
import asyncio
import aiohttp

async def fetch(session, url):
    async with session.get(url) as response:
        return await response.text()

async def many_requests(urls):
    async with aiohttp.ClientSession() as session:
        tasks = [fetch(session, url) for url in urls]
        return await asyncio.gather(*tasks)

urls = [
    "http://example.com",
    "http://www.rust-lang.org",
    "http://github.com"
]

results = asyncio.run(many_requests(urls))
```

**Rust (with surf):**
```rust
use surf;
use futures::future::join_all;

async fn many_requests(urls: &[String]) -> Vec<Result<String, surf::Error>> {
    let client = surf::Client::new();
    let futures: Vec<_> = urls
        .iter()
        .map(|url| client.get(url).recv_string())
        .collect();
    join_all(futures).await
}

let urls = vec![
    "http://example.com".to_string(),
    "http://www.rust-lang.org".to_string(),
    "http://github.com".to_string(),
];

let results = async_std::task::block_on(many_requests(&urls));
```

**Performance Comparison:**

For 100 requests to the same server:
- **Python (aiohttp)**: ~500ms
- **Rust (surf)**: ~400ms
- **Rust (reqwest)**: ~350ms

Benefits of Rust:
- Lower memory usage (~10MB vs ~50MB for Python)
- Better connection pooling
- Type safety prevents common errors
- No GIL limitations

## Common Pitfalls

### Don't Create a New Client Per Request

```rust
// BAD: Creates new client each time (no connection pooling)
for url in urls {
    let response = surf::get(url).recv_string().await?;
}

// GOOD: Reuse client (connection pooling)
let client = surf::Client::new();
for url in urls {
    let response = client.get(url).recv_string().await?;
}
```

### Don't Ignore Errors

```rust
// BAD: Panics on first error
let results: Vec<String> = join_all(futures)
    .await
    .into_iter()
    .map(|r| r.unwrap())  // Panics if any request fails!
    .collect();

// GOOD: Handle errors gracefully
let results: Vec<Result<String, _>> = join_all(futures).await;
for (i, result) in results.iter().enumerate() {
    match result {
        Ok(data) => println!("Request {} succeeded", i),
        Err(e) => eprintln!("Request {} failed: {}", i, e),
    }
}
```

### Watch Out for Too Many Concurrent Requests

```rust
// BAD: Can overwhelm server or exhaust system resources
let urls: Vec<_> = (0..10000).map(|i| format!("http://api.example.com/{}", i)).collect();
let futures: Vec<_> = urls.iter().map(|url| surf::get(url).recv_string()).collect();
let results = join_all(futures).await;  // 10,000 simultaneous connections!

// GOOD: Use rate limiting
let semaphore = Arc::new(Semaphore::new(50));  // Max 50 concurrent
```

## Real-World Example: Parallel API Calls

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct User {
    id: u32,
    name: String,
    email: String,
}

async fn fetch_users(user_ids: &[u32]) -> Vec<Result<User, String>> {
    let client = surf::Client::new();
    let semaphore = Arc::new(Semaphore::new(10));  // Max 10 concurrent

    let futures: Vec<_> = user_ids
        .iter()
        .map(|&id| {
            let permit = semaphore.clone().acquire_arc();
            let client = client.clone();

            async move {
                let _permit = permit.await;

                let url = format!("https://api.example.com/users/{}", id);
                let response = client
                    .get(&url)
                    .recv_json::<User>()
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(response)
            }
        })
        .collect();

    join_all(futures).await
}

#[async_std::main]
async fn main() {
    let user_ids: Vec<u32> = (1..=100).collect();
    let users = fetch_users(&user_ids).await;

    let successful: Vec<_> = users.iter().filter_map(|r| r.as_ref().ok()).collect();
    println!("Fetched {} out of {} users", successful.len(), user_ids.len());
}
```

## Summary

- Use async/await with task spawning for concurrent HTTP requests
- Create ONE client instance to enable connection pooling
- Use `join_all` to wait for all requests to complete
- Add rate limiting with `Semaphore` to control concurrency
- Handle errors gracefully - collect both successes and failures
- Add timeouts to prevent hanging requests
- Rust's async HTTP is faster and uses less memory than Python
- Popular clients: `surf` (async-std), `reqwest` (tokio)

## Related

- [How to Block on Futures](05-block-on-futures.md) - Running the async runtime
- [How to Build an Async Echo Server](07-build-async-echo-server.md) - Server-side async I/O
- [How to Spawn Blocking Tasks](06-spawn-blocking-tasks.md) - CPU-intensive work in async
