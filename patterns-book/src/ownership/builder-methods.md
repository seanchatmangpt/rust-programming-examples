# Builder Methods

**Also Known As:** Consuming Self, Move Methods, Ownership Transfer Methods, Typestate Transitions

## Context

You're designing an API for a data structure or builder type. Some operations naturally transform or consume the structure: extracting inner values, converting between representations, or transitioning through builder states. You could use mutable references (`&mut self`) for everything, but this allows reuse after transformation, which may be semantically wrong or inefficient.

## Problem

How do you design methods that consume a data structure, preventing further use after transformation, while expressing intent clearly and leveraging Rust's ownership system for compile-time correctness?

**The key insight:** Not all methods should preserve the object. Some operations are inherently final—they _consume_ the structure to produce something else.

## Forces

- **Semantic clarity**: Some operations are naturally destructive (splitting, unwrapping, conversion)
- **Resource ownership**: Inner values may need to be moved out without cloning
- **API safety**: Preventing use-after-transform prevents bugs
- **Efficiency**: Moving instead of cloning avoids expensive copies
- **Builder patterns**: State transitions should prevent returning to previous states
- **Ergonomics**: Method chaining should feel natural, not cumbersome
- **Compiler assistance**: Type system should prevent misuse

## Solution

**Define methods that take `self` (not `&self` or `&mut self`), consuming ownership of the structure. The compiler then prevents further use, making invalid states unrepresentable.**

The simplest form—extracting inner values:

```rust
impl Queue {
    pub fn split(self) -> (Vec<char>, Vec<char>) {
        (self.older, self.younger)
    }
}
```

**What happens at the call site:**

```rust
let mut q = Queue::new();
q.push('P');
q.push('D');
assert_eq!(q.pop(), Some('P'));
q.push('X');

let (older, younger) = q.split();
// q is now moved; cannot be used here
assert_eq!(older, vec!['D']);
assert_eq!(younger, vec!['X']);

// Compiler error: q was moved
// q.push('Z');  // ERROR: borrow of moved value
```

After `split`, the `Queue` is gone. The ownership of both `Vec<char>` instances has been transferred to the caller. This isn't a limitation—it's the _point_. Splitting a queue consumes it.

**The generic version preserves the pattern:**

```rust
impl<T> Queue<T> {
    pub fn split(self) -> (Vec<T>, Vec<T>) {
        (self.older, self.younger)
    }
}
```

The method signature is identical, only the types are parameterized. The ownership semantics are unchanged.

**Contrast with borrowing methods:**

```rust
impl<T> Queue<T> {
    // Borrows immutably - queue remains usable
    pub fn is_empty(&self) -> bool {
        self.older.is_empty() && self.younger.is_empty()
    }

    // Borrows mutably - queue remains usable (but modified)
    pub fn push(&mut self, t: T) {
        self.younger.push(t);
    }

    // Consumes - queue is GONE after call
    pub fn split(self) -> (Vec<T>, Vec<T>) {
        (self.older, self.younger)
    }
}
```

**The decision matrix:**

| Method Type | Signature | After Call | Use Case |
|-------------|-----------|------------|----------|
| **Reader** | `&self` | Queue still usable, unmodified | `is_empty`, `len`, `peek` |
| **Mutator** | `&mut self` | Queue still usable, modified | `push`, `pop`, `clear` |
| **Consumer** | `self` | Queue moved, unusable | `split`, `into_iter`, `unwrap` |

**Builder pattern with typestate:**

Consuming methods enable state machine APIs:

```rust
struct RequestBuilder {
    url: Option<String>,
    method: Option<String>,
}

impl RequestBuilder {
    fn new() -> Self {
        RequestBuilder { url: None, method: None }
    }

    // Consumes self, returns modified self
    fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    fn method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    // Final consumer: transforms into Request
    fn build(self) -> Result<Request, BuildError> {
        Request::new(
            self.url.ok_or(BuildError::MissingUrl)?,
            self.method.ok_or(BuildError::MissingMethod)?,
        )
    }
}

// Fluent API:
let request = RequestBuilder::new()
    .url("https://example.com")
    .method("GET")
    .build()?;
```

Each method consumes `self` and returns a new `Self`, creating a chain. The final `build()` consumes the builder and produces a `Request`. You cannot use the builder afterward—it's been consumed.

**The `into_` convention:**

Rust convention: methods that consume `self` and transform the type use the `into_` prefix:

```rust
impl String {
    pub fn into_bytes(self) -> Vec<u8> { ... }
}

impl Vec<T> {
    pub fn into_iter(self) -> IntoIter<T> { ... }
}

impl Option<T> {
    pub fn into_iter(self) -> IntoIter<T> { ... }
}
```

This signals: "This method consumes the value and gives you something else."

**When NOT to consume self:**

Don't use `self` when:
- The operation is naturally non-destructive (reading, checking state)
- The value should remain usable after the call
- You're implementing a mutating operation that doesn't change the type
- Cloning would be more ergonomic than reconstruction

**Bad example:**

```rust
// WRONG: pop consumes the entire queue
pub fn pop(self) -> Option<T> { ... }

// You'd have to reconstruct the queue after every pop!
let q = Queue::new();
let (item, q) = q.pop();  // Awkward!
```

**Correct:**

```rust
// RIGHT: pop mutates, queue remains usable
pub fn pop(&mut self) -> Option<T> { ... }

let mut q = Queue::new();
q.push('a');
let item = q.pop();  // Clean, queue still exists
```

## Resulting Context

**Benefits:**
- **Compile-time prevention**: Cannot use a value after it's consumed; compiler enforces this
- **Zero-cost**: Moving is typically just pointer-sized; no copying, no runtime checks
- **Intent clarity**: Method signature documents that this operation is terminal
- **Fluent APIs**: Builder pattern chains are elegant and type-safe
- **Resource extraction**: Can move inner values out without unsafe code or cloning

**Liabilities:**
- **Reconstruction cost**: If you need the original, you must rebuild it (unless returned)
- **Learning curve**: Newcomers to Rust find `self` vs `&self` vs `&mut self` confusing
- **Chaining complexity**: Long builder chains can create large intermediate values
- **Error messages**: "use of moved value" errors can be cryptic for beginners

**What's now enabled:**
- Builder APIs with compile-time state validation
- Zero-copy transformations (e.g., `String::into_bytes`)
- Iterator adapters that consume and transform collections
- Typestate pattern implementations (state machine via types)

**What's prevented:**
- Using a value after logical consumption
- Accidentally leaving a value in a partial state
- Double-free bugs (ownership prevents this)

## Related Patterns

- [Generic Container](./generic-container.md) - `split` works identically on `Queue<T>`
- [Concrete Container](./concrete-container.md) - Pattern is the same for concrete types
- Typestate Pattern - Use consuming methods to enforce state machines
- Iterator Adapters - `map`, `filter` consume the iterator and return a new one

## Known Uses

**In this codebase:**
- `/home/user/rust-programming-examples/queue/src/lib.rs` - `split(self)`
- `/home/user/rust-programming-examples/generic-queue/src/lib.rs` - Generic `split(self)`

**In the standard library:**

**Conversion methods:**
```rust
String::into_bytes(self) -> Vec<u8>
Vec::into_boxed_slice(self) -> Box<[T]>
PathBuf::into_os_string(self) -> OsString
```

**Iterator consumption:**
```rust
Iterator::collect(self) -> B
Iterator::fold(self, init, f) -> B
```

**Builder patterns:**
```rust
std::process::Command::new()
    .arg("ls")
    .env("PATH", "/usr/bin")
    .spawn()?
```

**Option/Result extractors:**
```rust
Option::unwrap(self) -> T
Result::unwrap(self) -> T
Result::map(self, f) -> Result<U, E>
```

**In the ecosystem:**

**reqwest builder:**
```rust
reqwest::Client::builder()
    .timeout(Duration::from_secs(10))
    .build()?
```

**tokio runtime builder:**
```rust
tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)
    .build()?
```

**serde_json:**
```rust
serde_json::to_string(&value)?  // Consumes value if not Copy
```

**The philosophical point:**

In languages with garbage collection, you rarely think about whether a method "consumes" an object. Everything is a reference; the GC cleans up later. This flexibility comes at the cost of accidental aliasing bugs and unclear ownership.

Rust makes consumption _explicit_. When you call `split(self)`, you're making a clear statement: "I'm done with this queue; give me its parts." The compiler enforces this, preventing later code from assuming the queue still exists.

This isn't restrictive—it's liberating. You can't accidentally use a value after it's logically gone. The type system captures the lifecycle, making illegal states unrepresentable.

**Christopher Alexander on living structure:**

"Each pattern is a three-part rule, which expresses a relation between a certain context, a problem, and a solution."

The context: A method that logically ends an object's lifecycle.
The problem: How to prevent use-after-consumption bugs.
The solution: Take `self` by value; let the type system enforce single use.

This pattern appears in every Rust codebase because ownership is central to Rust. You'll write consuming methods for builders, unwrappers, converters, and resource extractors. Each time, the pattern is the same: `self` signals consumption; the compiler prevents misuse. Simple, powerful, safe.
