# 2. LIBRARY CRATE WITH PUBLIC API

*A collection of reusable types and functions, carefully exposing what others need while hiding the implementation details.*

...within a **RUST PROJECT STRUCTURE**, when you want to create code that other programs can use without duplicating the implementation...

◆ ◆ ◆

**You've written code that solves a problem well, and you realize this solution would be valuable in multiple programs. But how do you make it reusable without exposing implementation details that might change?**

A binary executable exists to be run. A library exists to be used. This fundamental difference shapes everything about its structure. A binary has one entry point (`main`) and one purpose. A library has many entry points (its public functions and types) and serves many purposes.

The question becomes: what should be visible to users of your library, and what should remain hidden? Rust's module system gives you precise control through the `pub` keyword. Every item is private by default. You must consciously choose what to expose.

Look at the `Queue` library. It could expose the internal representation—the `older` and `younger` vectors that make the queue efficient. But this would be a mistake. Users don't need to know about the two-vector trick. They need `push`, `pop`, `is_empty`, and `new`. These four operations form the contract, the public API. The implementation remains hidden, free to change.

When you place code in `src/lib.rs`, Cargo understands: this is a library. Other crates can depend on it. The file becomes the root of your public interface. Every `pub fn`, `pub struct`, `pub enum` becomes a promise to users—a stable interface they can rely upon.

**Therefore:**

**Create a library crate with `src/lib.rs` as its root. Mark types and functions `pub` only when they form the essential public API. Keep implementation details private.**

```rust
/// A first-in, first-out queue of characters.
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last.
    younger: Vec<char>  // younger elements, youngest last.
}

impl Queue {
    /// Create a new, empty queue.
    pub fn new() -> Queue {
        Queue { older: Vec::new(), younger: Vec::new() }
    }

    /// Push a character onto the back of a queue.
    pub fn push(&mut self, c: char) {
        self.younger.push(c);
    }

    /// Pop a character off the front of a queue. Return `Some(c)` if there
    /// was a character to pop, or `None` if the queue was empty.
    pub fn pop(&mut self) -> Option<char> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }

            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        self.older.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.older.is_empty() && self.younger.is_empty()
    }
}
```

*The `Queue` struct is public, but its fields are private. Users create queues through `new()` and interact through methods. They cannot see or depend on the two-vector implementation.*

◆ ◆ ◆

Once you have a library, you need **TESTS DIRECTORY BESIDE SOURCE** (4) to verify it works as external code would use it, and **EXAMPLES DIRECTORY FOR USAGE** (5) to show how the API should be called. If the library needs a demonstration tool, add **BINARY AND LIBRARY TOGETHER** (3).
