# How to Build an Async Echo Server

## Problem

You need to build a TCP server that can handle many concurrent connections efficiently. A traditional thread-per-connection approach doesn't scale well beyond a few thousand connections.

## Solution

Build an async TCP server using async-std or tokio that can handle thousands of concurrent connections with minimal resource usage.

## Prerequisites

- Understanding of async/await syntax
- Basic knowledge of TCP networking
- Familiarity with `Result` and error handling

## Step-by-Step Guide

### 1. The Synchronous Approach (Baseline)

First, understand the traditional approach to see what we're improving:

```rust
use std::net::TcpListener;
use std::io;
use std::thread::spawn;

fn sync_echo_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("listening on {}", addr);

    loop {
        // Wait for a client to connect
        let (mut stream, addr) = listener.accept()?;
        println!("connection received from {}", addr);

        // Spawn a thread for each connection
        let mut write_stream = stream.try_clone()?;
        spawn(move || {
            // Echo everything back
            io::copy(&mut stream, &mut write_stream)
                .expect("error in client thread");
            println!("connection closed");
        });
    }
}
```

**Problems with this approach:**
- Each connection needs a thread (~2MB of memory)
- Thread creation overhead (~100μs per connection)
- Context switching overhead with many threads
- Doesn't scale beyond ~10,000 connections

### 2. Add async-std Dependency

```toml
[dependencies]
async-std = { version = "1.12", features = ["attributes"] }
```

### 3. Build a Basic Async Echo Server

```rust
use async_std::net::{TcpListener, TcpStream};
use async_std::io;
use async_std::prelude::*;
use async_std::task;

async fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let (reader, writer) = &mut (&stream, &stream);
    io::copy(reader, writer).await?;
    Ok(())
}

async fn run_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {}", addr);

    let mut incoming = listener.incoming();
    while let Some(stream_result) = incoming.next().await {
        let stream = stream_result?;

        // Spawn a task for each connection (lightweight!)
        task::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }

    Ok(())
}

#[async_std::main]
async fn main() {
    run_server("127.0.0.1:8080")
        .await
        .expect("Server error");
}
```

### 4. Understanding the Async Version

**Key differences from sync version:**

1. **Async functions**: Use `async fn` instead of regular `fn`
2. **Async I/O**: Use `async_std::net::TcpListener` instead of `std::net::TcpListener`
3. **Await points**: Call `.await` on async operations
4. **Task spawning**: Use `task::spawn` instead of `thread::spawn` (much lighter!)
5. **Async main**: Use `#[async_std::main]` attribute

### 5. Add Error Handling and Logging

```rust
use async_std::net::{TcpListener, TcpStream, SocketAddr};
use async_std::io;
use async_std::prelude::*;
use async_std::task;

async fn handle_client(mut stream: TcpStream, addr: SocketAddr) -> io::Result<()> {
    println!("Client {} connected", addr);

    let (reader, writer) = &mut (&stream, &stream);

    match io::copy(reader, writer).await {
        Ok(bytes_copied) => {
            println!("Client {} disconnected, {} bytes echoed", addr, bytes_copied);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error with client {}: {}", addr, e);
            Err(e)
        }
    }
}

async fn run_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    println!("Echo server listening on {}", local_addr);

    let mut incoming = listener.incoming();
    while let Some(stream_result) = incoming.next().await {
        match stream_result {
            Ok(stream) => {
                let addr = stream.peer_addr()?;

                task::spawn(async move {
                    let _ = handle_client(stream, addr).await;
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

#[async_std::main]
async fn main() {
    if let Err(e) = run_server("127.0.0.1:8080").await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
```

### 6. Accept Connections Concurrently

For better performance, accept multiple connections in parallel:

```rust
use async_std::net::TcpListener;
use async_std::task;

async fn run_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {}", listener.local_addr()?);

    loop {
        // Accept connections in a loop
        let (stream, addr) = listener.accept().await?;

        // Spawn each handler as a separate task
        task::spawn(async move {
            match handle_client(stream, addr).await {
                Ok(_) => println!("Client {} completed successfully", addr),
                Err(e) => eprintln!("Client {} error: {}", addr, e),
            }
        });
    }
}
```

### 7. Reading and Writing Async Streams

For more control, manually read and write to streams:

```rust
use async_std::io::{BufReader, BufWriter};
use async_std::prelude::*;

async fn handle_client_manual(stream: TcpStream, addr: SocketAddr) -> io::Result<()> {
    let reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    let mut lines = reader.lines();

    while let Some(line_result) = lines.next().await {
        let line = line_result?;

        // Echo the line back
        writer.write_all(line.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        println!("Client {}: {}", addr, line);

        // Exit on "quit"
        if line.trim() == "quit" {
            break;
        }
    }

    Ok(())
}
```

### 8. Add a Shutdown Signal

Handle graceful shutdown with Ctrl+C:

```rust
use async_std::channel::{unbounded, Receiver, Sender};
use async_std::future;
use async_std::task;

async fn run_server_with_shutdown(
    addr: &str,
    shutdown: Receiver<()>,
) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {}", listener.local_addr()?);

    loop {
        let accept = listener.accept();

        // Race between accepting a connection and shutdown signal
        match future::race(accept, shutdown.recv()).await {
            future::Either::Left((Ok((stream, addr)), _)) => {
                task::spawn(async move {
                    let _ = handle_client(stream, addr).await;
                });
            }
            future::Either::Left((Err(e), _)) => {
                eprintln!("Accept error: {}", e);
            }
            future::Either::Right(_) => {
                println!("Shutdown signal received, stopping server");
                break;
            }
        }
    }

    Ok(())
}

#[async_std::main]
async fn main() {
    let (sender, receiver) = unbounded();

    // Spawn a task to listen for Ctrl+C
    task::spawn(async move {
        let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, sender);
    });

    run_server_with_shutdown("127.0.0.1:8080", receiver)
        .await
        .expect("Server error");
}
```

## Using Tokio Instead

Here's the same server with tokio:

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::io;

async fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let (mut reader, mut writer) = stream.split();
    io::copy(&mut reader, &mut writer).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on {}", listener.local_addr()?);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("connection from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                eprintln!("Error: {}", e);
            }
        });
    }
}
```

**Dependencies for tokio:**
```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
```

## Error Handling in Async Code

### Pattern 1: Result Propagation

```rust
async fn handle_client(stream: TcpStream, addr: SocketAddr) -> io::Result<()> {
    // Use ? to propagate errors
    let (reader, writer) = &mut (&stream, &stream);
    io::copy(reader, writer).await?;
    Ok(())
}
```

### Pattern 2: Match on Results

```rust
async fn handle_client(stream: TcpStream, addr: SocketAddr) {
    let (reader, writer) = &mut (&stream, &stream);

    match io::copy(reader, writer).await {
        Ok(n) => println!("Copied {} bytes for {}", n, addr),
        Err(e) => eprintln!("Error for {}: {}", addr, e),
    }
}
```

### Pattern 3: Timeout Handling

```rust
use async_std::future::timeout;
use std::time::Duration;

async fn handle_client_with_timeout(
    stream: TcpStream,
    addr: SocketAddr,
) -> io::Result<()> {
    let timeout_duration = Duration::from_secs(30);

    match timeout(timeout_duration, do_echo(&stream)).await {
        Ok(result) => result,
        Err(_) => {
            eprintln!("Client {} timed out", addr);
            Err(io::Error::new(io::ErrorKind::TimedOut, "Connection timeout"))
        }
    }
}

async fn do_echo(stream: &TcpStream) -> io::Result<()> {
    let (reader, writer) = &mut (stream, stream);
    io::copy(reader, writer).await?;
    Ok(())
}
```

## Testing Your Echo Server

### Test with netcat:

```bash
# Terminal 1: Start the server
cargo run

# Terminal 2: Connect with netcat
nc localhost 8080
# Type messages and see them echoed back
```

### Test with telnet:

```bash
telnet localhost 8080
```

### Automated test:

```rust
#[async_std::test]
async fn test_echo_server() {
    use async_std::net::TcpStream;
    use async_std::io::prelude::*;

    // Spawn the server in a background task
    task::spawn(async {
        run_server("127.0.0.1:8081").await
    });

    // Give server time to start
    task::sleep(Duration::from_millis(100)).await;

    // Connect and test
    let mut stream = TcpStream::connect("127.0.0.1:8081").await.unwrap();

    stream.write_all(b"Hello, World!\n").await.unwrap();

    let mut buffer = vec![0u8; 14];
    stream.read_exact(&mut buffer).await.unwrap();

    assert_eq!(&buffer, b"Hello, World!\n");
}
```

## Performance Comparison

**Synchronous (thread-per-connection):**
- 1,000 connections: ~2GB memory
- 10,000 connections: ~20GB memory (impractical)
- Context switching overhead

**Asynchronous (task-per-connection):**
- 1,000 connections: ~100MB memory
- 10,000 connections: ~1GB memory
- 100,000+ connections: Possible on modern hardware
- Minimal context switching

## Common Pitfalls

### Don't Block in Async Code

```rust
// BAD: Blocks the executor
async fn bad_handler(stream: TcpStream) {
    std::thread::sleep(Duration::from_secs(1));  // Blocks!
}

// GOOD: Use async sleep
async fn good_handler(stream: TcpStream) {
    task::sleep(Duration::from_secs(1)).await;  // Yields to executor
}
```

### Don't Forget Error Handling

```rust
// BAD: Panics on error
loop {
    let (stream, addr) = listener.accept().await.unwrap();  // Can panic!
}

// GOOD: Handle errors gracefully
loop {
    match listener.accept().await {
        Ok((stream, addr)) => { /* handle */ }
        Err(e) => eprintln!("Accept failed: {}", e),
    }
}
```

### Watch Memory with Many Connections

```rust
// Consider limiting concurrent connections
use async_std::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(1000));  // Max 1000 concurrent

loop {
    let permit = semaphore.clone().acquire_arc().await;
    let (stream, addr) = listener.accept().await?;

    task::spawn(async move {
        let _permit = permit;  // Hold permit during connection
        handle_client(stream, addr).await
    });
}
```

## Summary

- Async servers handle many more connections than thread-based servers
- Use `async_std::net::TcpListener` or `tokio::net::TcpListener`
- Spawn lightweight tasks with `task::spawn` or `tokio::spawn`
- Use async I/O operations with `.await`
- Handle errors gracefully with `Result` types
- Add timeouts to prevent resource exhaustion
- Test with `nc`, `telnet`, or automated tests

## Related

- [How to Block on Futures](05-block-on-futures.md) - Starting the async runtime
- [How to Make Concurrent HTTP Requests](08-concurrent-http-requests.md) - Client-side async I/O
- [How to Spawn Blocking Tasks](06-spawn-blocking-tasks.md) - Running CPU-intensive work
