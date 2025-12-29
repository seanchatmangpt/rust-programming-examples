# 5. EXAMPLES DIRECTORY FOR USAGE

*Working programs that demonstrate how to use your library, serving as both documentation and verification that the API is practical.*

...within a **RUST PROJECT STRUCTURE**, when your library's public API needs clear, runnable demonstrations that show real-world usage...

◆ ◆ ◆

**Documentation can describe what functions do, but users still ask: "How do I actually use this?" Tests verify behavior, but they're written to catch bugs, not to teach. How do you show users the right way to use your library?**

The gap between API documentation and real understanding is wide. A function signature tells you what parameters it takes. Documentation comments explain what it does. But neither shows you the *pattern* of usage—how functions combine, what order to call them in, how to handle errors gracefully.

You could write more documentation, with longer examples in comments. But long examples in doc comments are hard to maintain—they're not compiled or tested independently. They bitrot silently. Users don't know if they still work.

Or you could tell users to read your integration tests. But tests optimize for coverage, not clarity. They test edge cases and error conditions. They use assertion syntax instead of showing real program flow. Tests answer "Does it work?" not "How should I use it?"

Examples occupy a third space. They are real programs, compiled and tested. But they optimize for demonstration, not verification. Each example shows one clear pattern of usage—"Here's how to parse configuration," "Here's how to make an HTTP request," "Here's how to process a file."

Rust's tooling embraces this pattern. Place examples in `examples/`, and Cargo treats them specially. Run them with `cargo run --example name`. They compile against your library's public API, just like integration tests. But they're meant to be read and modified by users learning your library.

**Therefore:**

**Create an `examples/` directory beside `src/`. Write small, focused programs demonstrating common usage patterns. Each example should be a complete, working program that shows one clear way to use your library.**

```rust
// examples/basic_queue.rs
use queue::Queue;

fn main() {
    // Create a new queue
    let mut q = Queue::new();

    // Add some elements
    println!("Adding characters to queue...");
    q.push('h');
    q.push('e');
    q.push('l');
    q.push('l');
    q.push('o');

    // Process them in order
    println!("Processing queue:");
    while !q.is_empty() {
        if let Some(c) = q.pop() {
            println!("  {}", c);
        }
    }
}
```

```rust
// examples/queue_as_buffer.rs
use queue::Queue;
use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    let mut buffer = Queue::new();

    println!("Enter characters (Ctrl+D to finish):");
    let stdin = io::stdin();

    // Buffer input characters
    for line in stdin.lock().lines() {
        for c in line?.chars() {
            buffer.push(c);
        }
    }

    // Process buffered input
    println!("\nProcessing {} characters",
             if buffer.is_empty() { 0 } else {
                 // ... implementation ...
                 0
             });

    Ok(())
}
```

*Project structure showing examples alongside tests:*
```
queue/
├── Cargo.toml
├── src/
│   └── lib.rs
├── tests/
│   └── integration_test.rs
└── examples/
    ├── basic_queue.rs
    └── queue_as_buffer.rs
```

*Users run: `cargo run --example basic_queue` to see the library in action.*

◆ ◆ ◆

Examples naturally lead to **README WITH USAGE SECTION** that links to them. Complex examples may need **EXAMPLE WITH ERROR HANDLING** to show robust patterns. As examples share code, you may extract **SHARED EXAMPLE UTILITIES** into a separate module.
