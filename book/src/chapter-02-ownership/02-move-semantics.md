# Designing with Move Semantics

## Learning Objectives

By the end of this section, you will understand:
- How move semantics enable zero-cost resource transfer
- Architectural patterns that leverage moves: pipelines, builders, and state machines
- When to design APIs that consume values versus borrow them
- The performance characteristics of move operations

## Introduction: Moves as Zero-Cost Transfers

In languages with garbage collection, passing data between functions typically involves copying pointers and incrementing reference counts. In C++, you must carefully choose between copy constructors, move constructors, and references. Rust simplifies this with **move semantics by default**.

When you pass a non-Copy type to a function or assign it to a new variable, Rust **moves** the value—transferring ownership without copying the underlying data. This is a zero-cost abstraction: moving a `Vec<T>` with a million elements is as cheap as moving an `i32`.

```rust
let v1 = vec![1, 2, 3, 4, 5];  // v1 owns the vector
let v2 = v1;                    // Ownership moves to v2 (v1 is now invalid)
// println!("{:?}", v1);        // Compile error: v1 was moved
println!("{:?}", v2);           // OK: v2 owns the vector
```

This simple rule enables powerful architectural patterns where resource ownership flows through your system without runtime overhead.

## Move Semantics and Value Categories

Rust distinguishes between types that are **Copy** and those that are **Move**.

### Copy Types

Small types that can be copied cheaply implement the `Copy` trait:
- All primitive integers (`i32`, `u64`, etc.)
- Floating-point numbers (`f32`, `f64`)
- Booleans (`bool`)
- Characters (`char`)
- Tuples and arrays of Copy types

```rust
let x = 42;
let y = x;  // x is copied (both x and y are valid)
println!("{} {}", x, y);  // OK: Copy types can be used after "moving"
```

### Move Types

Types that manage resources (heap allocations, file handles, sockets) use move semantics:
- `String`, `Vec<T>`, `Box<T>`, `HashMap<K, V>`
- User-defined types (unless explicitly `Copy`)
- Types containing non-Copy fields

```rust
let s1 = String::from("hello");
let s2 = s1;  // Ownership moves (s1 is now invalid)
// println!("{}", s1);  // Error: s1 was moved
```

The distinction is architectural: **Copy types represent values; Move types represent resources**.

## Pipeline Architectures: Flow-Based Systems

Move semantics naturally express **pipeline architectures** where data flows through a series of transformations, with each stage consuming the input and producing a new output.

### Example: Data Transformation Pipeline

```rust
fn parse(raw: String) -> Result<ParsedData, Error> {
    // Consume the raw string, produce parsed data
    // The String's buffer is reused internally if possible
}

fn validate(data: ParsedData) -> Result<ValidatedData, Error> {
    // Consume parsed data, produce validated data
}

fn process(data: ValidatedData) -> ProcessedData {
    // Consume validated data, produce processed result
}

fn serialize(data: ProcessedData) -> String {
    // Consume processed data, produce serialized output
}

// Usage: data flows through the pipeline
let result = serialize(
    process(
        validate(
            parse(input_string)?
        )?
    )
);
```

Each function **consumes** its input, making it impossible to accidentally reuse intermediate stages. The type system enforces the pipeline structure—you can't use `ParsedData` after passing it to `validate()`.

This pattern is particularly powerful for builder APIs and request/response processing:

```rust
// HTTP request builder (pseudo-code)
let response = HttpRequest::new("https://api.example.com")
    .with_header("Authorization", token)  // Each method consumes and returns Self
    .with_body(payload)
    .send()?;  // Final method consumes the builder
```

## Consumer-Producer Patterns

The queue implementation demonstrates a classic consumer-producer pattern where ownership flows through the queue:

```rust
pub fn push(&mut self, c: char) {
    self.younger.push(c);  // Ownership of `c` transfers to the queue
}

pub fn pop(&mut self) -> Option<char> {
    // ...
    self.older.pop()  // Ownership transfers from queue to caller
}
```

When you call `push()`, the character is **moved into** the queue. When you call `pop()`, the character is **moved out of** the queue. The queue acts as a temporary owner, managing the lifetime of elements without copying them.

### Generic Ownership Transfer

The generic queue makes this even more powerful:

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}

impl<T> Queue<T> {
    pub fn push(&mut self, t: T) {
        self.younger.push(t);  // Works for any T (Copy or Move)
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.older.is_empty() {
            use std::mem::swap;

            if self.younger.is_empty() {
                return None;
            }

            swap(&mut self.older, &mut self.younger);  // Zero-cost swap
            self.older.reverse();
        }

        self.older.pop()
    }
}
```

This implementation works identically whether `T` is a `Copy` type like `i32` or a `Move` type like `String`. The abstraction is zero-cost: for `Copy` types, `push` and `pop` copy; for `Move` types, they transfer ownership without copying the underlying data.

### Performance Characteristics

The `swap` operation is particularly enlightening:

```rust
use std::mem::swap;
swap(&mut self.older, &mut self.younger);
```

`std::mem::swap` exchanges the contents of two values **without allocating**. For vectors, this swaps three pointer-sized values (pointer to buffer, length, capacity)—regardless of how many elements the vectors contain. This is a constant-time operation that enables efficient queue rebalancing.

## State Machines via Move Semantics

Move semantics enable **compile-time state machines** where each state transition consumes the current state and produces a new state. This makes invalid state transitions impossible.

### Example: Connection State Machine

```rust
struct Disconnected;
struct Connecting;
struct Connected { session_id: String }
struct Disconnecting;

struct Connection<State> {
    address: String,
    state: State,
}

impl Connection<Disconnected> {
    fn new(address: String) -> Self {
        Connection {
            address,
            state: Disconnected,
        }
    }

    fn connect(self) -> Connection<Connecting> {
        println!("Connecting to {}...", self.address);
        Connection {
            address: self.address,  // Move address to new state
            state: Connecting,
        }
    }
}

impl Connection<Connecting> {
    fn finalize(self, session_id: String) -> Connection<Connected> {
        Connection {
            address: self.address,
            state: Connected { session_id },
        }
    }

    fn fail(self) -> Connection<Disconnected> {
        Connection {
            address: self.address,
            state: Disconnected,
        }
    }
}

impl Connection<Connected> {
    fn send_data(&self, data: &[u8]) {
        println!("Sending {} bytes to {}", data.len(), self.address);
    }

    fn disconnect(self) -> Connection<Disconnecting> {
        Connection {
            address: self.address,
            state: Disconnecting,
        }
    }
}

impl Connection<Disconnecting> {
    fn finalize(self) -> Connection<Disconnected> {
        Connection {
            address: self.address,
            state: Disconnected,
        }
    }
}
```

Usage:

```rust
let conn = Connection::new("192.168.1.1".to_string());
let conn = conn.connect();  // Disconnected -> Connecting
let conn = conn.finalize("session123".to_string());  // Connecting -> Connected

conn.send_data(b"Hello");  // Only available in Connected state

let conn = conn.disconnect();  // Connected -> Disconnecting
let conn = conn.finalize();     // Disconnecting -> Disconnected

// conn.send_data(b"World");  // Compile error: no send_data for Disconnected
```

Each state transition **consumes** the connection in one state and **produces** it in another. The type system enforces:
- You can't send data unless the connection is in `Connected` state
- You can't connect an already-connected connection
- You can't skip states (e.g., go directly from `Disconnected` to `Connected`)

This pattern is called the **typestate pattern**, and it's only possible because of move semantics.

## Builder Pattern: Consuming Builders

The builder pattern in Rust commonly uses move semantics to construct complex objects step-by-step:

```rust
struct RequestBuilder {
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl RequestBuilder {
    fn new(url: String) -> Self {
        RequestBuilder {
            url,
            headers: Vec::new(),
            body: None,
        }
    }

    fn header(mut self, key: String, value: String) -> Self {
        self.headers.push((key, value));
        self  // Return the modified builder
    }

    fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    fn build(self) -> Request {
        Request {
            url: self.url,
            headers: self.headers,
            body: self.body,
        }
    }
}

struct Request {
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}
```

Usage:

```rust
let request = RequestBuilder::new("https://api.example.com".to_string())
    .header("Content-Type".to_string(), "application/json".to_string())
    .header("Authorization".to_string(), "Bearer token".to_string())
    .body("{\"key\": \"value\"}".to_string())
    .build();
```

Each method takes `self` by value (consumes the builder) and returns `Self` (produces a new builder). This enables:
- **Fluent chaining**: Each method returns the builder for the next call
- **No cloning**: The builder is moved, not copied, at each step
- **Final consumption**: `build()` consumes the builder, preventing reuse

## Decision Framework: When to Move vs. Borrow

Designing APIs requires choosing whether to consume values (move) or borrow them:

### Take ownership (`self` or `value: T`) when:

1. **The function consumes the input**
   ```rust
   fn into_string(self) -> String  // Convert to String, original is gone
   ```

2. **The function stores the value**
   ```rust
   fn push(&mut self, item: T)  // Queue stores the item
   ```

3. **The function transforms the value**
   ```rust
   fn map<F, U>(self, f: F) -> Iterator<U>  // Transform iterator
   ```

4. **The function represents a state transition**
   ```rust
   fn connect(self) -> Connection<Connected>  // Typestate pattern
   ```

### Borrow (`&self` or `&T`) when:

1. **The function only reads the value**
   ```rust
   fn len(&self) -> usize  // Read-only access
   ```

2. **The value should remain usable after the call**
   ```rust
   fn print_debug(data: &Data)  // Don't consume the data
   ```

3. **The function might be called multiple times**
   ```rust
   fn get(&self, index: usize) -> Option<&T>  // Retrieve without consuming
   ```

### Borrow mutably (`&mut self` or `&mut T`) when:

1. **The function modifies the value in place**
   ```rust
   fn sort(&mut self)  // Modify, don't consume
   ```

2. **The function needs exclusive access**
   ```rust
   fn pop(&mut self) -> Option<T>  // Modify internal state
   ```

## Performance Implications

Move semantics are designed for zero-cost abstraction:

### Stack-to-Stack Moves

Moving a value from one stack location to another is as cheap as copying a few words:

```rust
struct Point { x: f64, y: f64 }

let p1 = Point { x: 3.0, y: 4.0 };
let p2 = p1;  // Copy two f64s (16 bytes) - same cost as pointer copy
```

### Heap Ownership Transfers

Moving a heap-allocated value transfers ownership of the pointer without touching the heap data:

```rust
let v1 = vec![1, 2, 3, ..., 1_000_000];  // 1 million elements
let v2 = v1;  // Copy 3 words (pointer, length, capacity) - constant time!
```

The million integers remain in place on the heap. Only the stack-allocated vector header is copied.

### Comparison with Alternatives

| Operation | Cost | When to Use |
|-----------|------|-------------|
| **Move** | 1-3 word copy | Default for non-Copy types |
| **Clone** | Deep copy (O(n)) | When you need two independent copies |
| **Borrow** | Reference copy | When you need temporary read access |
| **Mutable borrow** | Reference copy | When you need temporary write access |

## Real-World Pattern: Iterator Chains

Rust's iterator chains leverage move semantics for zero-cost abstractions:

```rust
let processed: Vec<_> = data
    .into_iter()           // Consume the collection
    .filter(|x| x.is_valid())
    .map(|x| x.process())
    .collect();            // Consume the iterator
```

Each method in the chain consumes the previous iterator and produces a new one. The entire chain compiles down to a single loop with no heap allocations for the intermediate iterators—all thanks to move semantics and monomorphization.

## Conclusion

Move semantics transform resource management from a runtime concern into a compile-time architectural tool. By defaulting to moves instead of copies, Rust enables:

- **Pipeline architectures** where data flows through transformations
- **Typestate patterns** that enforce state machine invariants at compile time
- **Builder patterns** with fluent, zero-cost APIs
- **Iterator chains** that compile to tight, efficient loops

The key insight: **moves represent resource transfer, not data copying**. This makes resource flow explicit in the type system, preventing entire classes of bugs while maintaining zero runtime overhead.

In the next section, we'll explore **borrowing as an architectural interface**—how temporary access patterns shape API design.

## Cross-References

- **Section 2.1: Ownership as Constraint** - The foundation of move semantics
- **Section 2.3: Borrowing as Interface** - When to borrow instead of move
- **Chapter 3: Traits and Generics** - Iterator patterns and trait-based moves
- **Chapter 6: Async Architecture** - Move semantics in async contexts
