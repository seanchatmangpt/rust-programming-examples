# 39. ASYNC FUNCTION WITH AWAIT

*A messenger who, while waiting for a reply, sets down their burden and helps with other tasks, returning instantly when the response arrives to continue their original errand*

...within a **FUNCTION RETURNING RESULT (38)** or **PUBLIC FUNCTION (35)**, when the operation requires waiting for I/O, network, or other external events without blocking the entire program...

◆ ◆ ◆

**How do you perform I/O operations that might wait for arbitrary time periods without blocking other work, without manually managing thread pools and callbacks?**

Traditional blocking I/O stops the thread until the operation completes. If you're waiting for a network response that takes 500ms, that thread does nothing for half a second. Spawning a thread per operation works for small scale but becomes expensive with thousands of concurrent operations. Callbacks avoid blocking but fragment your logic across multiple functions, making it hard to follow the flow or handle errors.

Rust's async/await syntax provides the illusion of blocking code while actually yielding control when waiting. An `async fn` returns a Future that can be suspended and resumed. The `.await` keyword marks suspension points—places where the function might pause and let other work proceed.

The async runtime (tokio, async-std) manages these suspended functions efficiently, often handling thousands with just a few threads. The syntax remains sequential and clear—you write `let data = fetch().await?;` as if it were blocking, but the runtime schedules other work while fetch() waits.

This pattern requires discipline: you must `.await` every async operation and avoid blocking operations within async functions. A single blocking call will freeze all tasks on that thread.

**Therefore:**

**Mark functions that perform I/O or waiting with `async`, and place `.await` after every asynchronous operation to allow cooperative multitasking.**

```rust
// From cheapo-request/src/main.rs
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

    // Block on the async function from synchronous main
    let response = task::block_on(cheapo_request("example.com", 80, "/"))?;
    println!("{}", response);
    Ok(())
}
```

*The function appears to wait, but actually releases control at each await point, like a dancer who spins away and returns in perfect time*

◆ ◆ ◆

This works with **FUNCTION RETURNING RESULT (38)** through the `?` operator on awaited operations, connects to **MATCH ON RESULT (28)** for handling async errors, and enables **CONCURRENT JOIN (future pattern)** when combining multiple async operations.
