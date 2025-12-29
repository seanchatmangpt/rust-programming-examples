# TCP Server Loop

## Context

You are building a network service that needs to accept connections from multiple clients. This could be a chat server, game server, proxy, protocol implementation, or any daemon that listens on a TCP port. Each client connection should be handled independently, potentially in parallel, while the server continues accepting new connections.

The server runs indefinitely, waiting for connections, handling them, and returning to wait for more. Connections may arrive at any time, and clients may disconnect without notice.

## Problem

**How do you create a server that continuously accepts TCP connections and handles multiple clients concurrently without blocking new connections, while managing resources properly and handling errors gracefully?**

Handling one client at a time blocks acceptance of new connections. Managing concurrent connections manually with threads is error-prone. Rust's ownership system makes sharing state between threads challenging. Network I/O can fail in many ways that must be handled.

## Forces

- **Concurrency**: Must handle multiple clients simultaneously
- **Responsiveness**: Accepting new connections shouldn't block on existing clients
- **Resource management**: Each connection consumes resources (memory, file descriptors)
- **Error isolation**: One client's error shouldn't crash the server
- **Protocol handling**: Must read and write data according to protocol
- **Connection lifecycle**: Clients connect and disconnect unpredictably
- **Thread safety**: Shared state must be synchronized properly
- **Graceful shutdown**: Server should clean up resources on exit

## Solution

**Use `TcpListener::bind()` to create a listening socket, loop infinitely calling `accept()` to get new connections, and spawn a thread for each client to handle their communication independently.** Clone the stream for bidirectional communication and propagate errors appropriately.

### Structure

```rust
use std::net::TcpListener;
use std::io;
use std::thread;

fn server_loop(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("Listening on {}", addr);

    loop {
        let (stream, client_addr) = listener.accept()?;
        println!("Connection from {}", client_addr);

        thread::spawn(move || {
            handle_client(stream);
        });
    }
}

fn handle_client(stream: TcpStream) {
    // Handle communication with client
}
```

### Real Implementation (from echo-server)

```rust
use std::net::TcpListener;
use std::io;
use std::thread::spawn;

/// Accept connections forever, spawning a thread for each one.
fn echo_main(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("listening on {}", addr);

    loop {
        // Wait for a client to connect.
        let (mut stream, addr) = listener.accept()?;
        println!("connection received from {}", addr);

        // Spawn a thread to handle this client.
        let mut write_stream = stream.try_clone()?;
        spawn(move || {
            // Echo everything we receive from `stream` back to it.
            io::copy(&mut stream, &mut write_stream)
                .expect("error in client thread: ");
            println!("connection closed");
        });
    }
}

fn main() {
    echo_main("127.0.0.1:17007").expect("error: ");
}
```

### Key Elements

1. **TcpListener::bind()**: Binds to an address and port, creating a listening socket
2. **Infinite loop**: Server runs forever, continuously accepting connections
3. **listener.accept()**: Blocks until a client connects, returns `(TcpStream, SocketAddr)`
4. **thread::spawn()**: Creates a new thread for each client
5. **try_clone()**: Duplicates the stream for bidirectional communication
6. **move closure**: Transfers ownership of stream to thread
7. **io::copy()**: Efficiently copies data between read and write ends

### Protocol Pattern

```rust
// Read: stream implements Read trait
stream.read(&mut buffer)?;

// Write: stream implements Write trait
stream.write_all(b"response")?;

// Bidirectional: clone stream for separate read/write
let mut read_stream = stream.try_clone()?;
let mut write_stream = stream;
```

## Resulting Context

### Benefits

- **Concurrency**: Each client handled in parallel without blocking others
- **Simplicity**: Thread-per-connection is straightforward to implement
- **Isolation**: Client errors don't affect server or other clients
- **Bidirectional**: Separate read/write streams enable full-duplex communication
- **Standard API**: Uses familiar `Read` and `Write` traits

### Liabilities

- **Thread overhead**: One OS thread per connection limits scalability
- **Resource consumption**: Thousands of clients = thousands of threads
- **No shared state**: Threads don't easily share data (by design)
- **Context switching**: Many threads increase scheduling overhead
- **Not async**: Blocking I/O doesn't integrate with async ecosystem

### Performance Characteristics

- **Scalability**: Good for tens/hundreds of clients, poor for thousands
- **Latency**: Low latency per client due to dedicated thread
- **Memory**: ~1-2MB stack per thread on Linux
- **CPU**: Context switching overhead grows with client count

## Variations

### Line-Based Protocol Handler

```rust
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn handle_line_protocol(mut stream: TcpStream) -> io::Result<()> {
    let reader = BufReader::new(stream.try_clone()?);

    for line_result in reader.lines() {
        let line = line_result?;
        println!("Received: {}", line);

        // Process and respond
        let response = process_command(&line);
        writeln!(stream, "{}", response)?;
        stream.flush()?;
    }

    Ok(())
}

fn server_main(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;

    for stream_result in listener.incoming() {
        let stream = stream_result?;

        thread::spawn(move || {
            if let Err(e) = handle_line_protocol(stream) {
                eprintln!("Error handling client: {}", e);
            }
        });
    }

    Ok(())
}
```

### With Connection Counter

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn server_with_counter(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    let connection_count = Arc::new(AtomicUsize::new(0));

    loop {
        let (stream, addr) = listener.accept()?;
        let count = connection_count.clone();
        let id = count.fetch_add(1, Ordering::SeqCst);

        println!("Connection {} from {}", id, addr);

        thread::spawn(move || {
            handle_client(stream, id);
            count.fetch_sub(1, Ordering::SeqCst);
        });
    }
}
```

### With Shared State (Broadcast Server)

```rust
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

type Clients = Arc<Mutex<Vec<TcpStream>>>;

fn broadcast_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));

    for stream_result in listener.incoming() {
        let stream = stream_result?;
        let write_stream = stream.try_clone()?;

        // Add to client list
        clients.lock().unwrap().push(write_stream);

        let clients_clone = clients.clone();
        thread::spawn(move || {
            handle_broadcast_client(stream, clients_clone);
        });
    }

    Ok(())
}

fn handle_broadcast_client(stream: TcpStream, clients: Clients) {
    let reader = BufReader::new(stream);

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            // Broadcast to all clients
            let mut clients = clients.lock().unwrap();
            clients.retain_mut(|client| {
                writeln!(client, "{}", line).is_ok()
            });
        }
    }
}
```

### With Timeout

```rust
use std::time::Duration;

fn server_with_timeout(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;

    for stream_result in listener.incoming() {
        let mut stream = stream_result?;

        // Set read/write timeouts
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;

        thread::spawn(move || {
            match handle_client_with_timeout(stream) {
                Ok(()) => println!("Client disconnected"),
                Err(e) => eprintln!("Client error: {}", e),
            }
        });
    }

    Ok(())
}
```

### Using `incoming()` Iterator

```rust
use std::net::TcpListener;

fn server_with_iterator(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;

    // incoming() returns iterator over Result<TcpStream>
    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }

    Ok(())
}
```

### Thread Pool for Resource Control

```rust
use std::sync::mpsc;
use threadpool::ThreadPool;

fn server_with_pool(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    let pool = ThreadPool::new(10); // Max 10 concurrent clients

    for stream_result in listener.incoming() {
        let stream = stream_result?;

        pool.execute(move || {
            handle_client(stream);
        });
    }

    Ok(())
}
```

## Related Patterns

- **Error Propagation**: Handle network errors gracefully
- **Line-Oriented Processing**: Parse protocol commands line by line
- **Blocking HTTP Client**: Complementary pattern for client side
- **Thread-Per-Connection**: This is the classic implementation

## Known Uses

### Standard Protocols

- **Echo server**: Reflects data back to client
- **Time server**: Sends current time on connection
- **Daytime protocol**: Simple text-based time service
- **Telnet server**: Line-based command protocol
- **FTP server**: File transfer protocol implementation

### Real Projects

```rust
// Simple HTTP server (toy implementation)
fn http_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;

    for stream in listener.incoming() {
        let mut stream = stream?;

        thread::spawn(move || {
            // Read HTTP request (simplified)
            let mut buffer = [0; 512];
            stream.read(&mut buffer).unwrap();

            // Send HTTP response
            let response = "HTTP/1.1 200 OK\r\n\
                           Content-Length: 13\r\n\
                           \r\n\
                           Hello, World!";

            stream.write_all(response.as_bytes()).unwrap();
        });
    }

    Ok(())
}

// Chat server
fn chat_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    let clients = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        let stream = stream?;
        let clients = clients.clone();

        thread::spawn(move || {
            let peer = stream.peer_addr().unwrap();
            println!("{} connected", peer);

            clients.lock().unwrap().push(stream.try_clone().unwrap());

            let reader = BufReader::new(stream.try_clone().unwrap());

            for line in reader.lines().filter_map(|l| l.ok()) {
                let msg = format!("{}: {}", peer, line);

                // Broadcast to all clients
                for client in clients.lock().unwrap().iter_mut() {
                    let _ = writeln!(client, "{}", msg);
                }
            }

            // Remove client on disconnect
            clients.lock().unwrap().retain(|c| {
                c.peer_addr().unwrap() != peer
            });

            println!("{} disconnected", peer);
        });
    }

    Ok(())
}

// Key-value store server
fn kvstore_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    let store = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let stream = stream?;
        let store = store.clone();

        thread::spawn(move || {
            handle_kvstore_client(stream, store);
        });
    }

    Ok(())
}

fn handle_kvstore_client(
    stream: TcpStream,
    store: Arc<Mutex<HashMap<String, String>>>
) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream;
    let mut line = String::new();

    while reader.read_line(&mut line).unwrap() > 0 {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        let response = match parts.as_slice() {
            ["GET", key] => {
                let store = store.lock().unwrap();
                store.get(*key)
                    .map(|v| v.clone())
                    .unwrap_or_else(|| "NOT_FOUND".to_string())
            }
            ["SET", key, value] => {
                let mut store = store.lock().unwrap();
                store.insert(key.to_string(), value.to_string());
                "OK".to_string()
            }
            _ => "ERROR".to_string(),
        };

        writeln!(writer, "{}", response).unwrap();
        line.clear();
    }
}
```

## Implementation Notes

### Error Handling in Threads

```rust
fn safe_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    // Catch panics in client handler
                    let result = std::panic::catch_unwind(|| {
                        handle_client(stream)
                    });

                    if let Err(e) = result {
                        eprintln!("Client handler panicked: {:?}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
                // Continue accepting other connections
            }
        }
    }

    Ok(())
}
```

### Graceful Shutdown

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn server_with_shutdown(addr: &str, shutdown: Arc<AtomicBool>) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    listener.set_nonblocking(true)?;

    while !shutdown.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, addr)) => {
                let shutdown = shutdown.clone();
                thread::spawn(move || {
                    handle_client_with_shutdown(stream, shutdown);
                });
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }

    println!("Server shutting down");
    Ok(())
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;
    use std::io::{Read, Write};

    #[test]
    fn test_echo_server() {
        // Start server in background thread
        thread::spawn(|| {
            echo_main("127.0.0.1:0").unwrap();
        });

        thread::sleep(Duration::from_millis(100));

        // Connect client
        let mut stream = TcpStream::connect("127.0.0.1:17007").unwrap();

        // Send data
        stream.write_all(b"Hello").unwrap();

        // Receive echo
        let mut buffer = [0; 5];
        stream.read_exact(&mut buffer).unwrap();

        assert_eq!(&buffer, b"Hello");
    }
}
```

### When to Use Async Instead

Consider async I/O (tokio, async-std) when:
- Handling thousands of concurrent connections
- CPU overhead per connection is minimal
- Need integration with async ecosystem
- Memory per connection is a concern

```rust
// Async version for comparison
use tokio::net::TcpListener;
use tokio::io;

#[tokio::main]
async fn async_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Connection from {}", addr);

        tokio::spawn(async move {
            handle_client_async(stream).await;
        });
    }
}
```

## References

- Rust std::net documentation
- "Programming Rust" Chapter 18: Input and Output
- TCP/IP Illustrated by Stevens
- Unix Network Programming by Stevens
- tokio documentation (for async alternative)
