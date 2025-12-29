# Real-World Pattern: Queue-Based Systems

## Learning Objectives

By the end of this section, you will understand:
- Ownership transfer through queue operations in depth
- The architectural implications of the two-stack queue pattern
- How generics interact with ownership in the generic queue
- Buffering strategies and their ownership characteristics
- Synchronous vs. async queue patterns and their tradeoffs

## Introduction: Queues as Ownership Transfer Mechanisms

Queues are fundamental building blocks in system architecture—message queues, task queues, event queues, and buffer queues appear everywhere from operating systems to web services. In Rust, queues serve a dual purpose: they're **data structures** and **ownership transfer mechanisms**.

The `queue` and `generic-queue` projects demonstrate how ownership principles shape queue implementation and usage patterns. By examining these implementations in detail, we'll uncover architectural patterns applicable to any system involving buffering, task scheduling, or asynchronous communication.

## The Two-Stack Queue Pattern

Both the `queue` and `generic-queue` projects implement a **two-stack queue** (also called a two-vector queue). This pattern uses two stacks to achieve amortized O(1) enqueue and dequeue operations.

### Implementation: Basic Queue

From `/home/user/rust-programming-examples/queue/src/lib.rs`:

```rust
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last.
    younger: Vec<char>  // younger elements, youngest last.
}

impl Queue {
    pub fn push(&mut self, c: char) {
        self.younger.push(c);
    }

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
}
```

### Ownership Architecture

This simple structure encodes several architectural decisions:

1. **Exclusive ownership**: The `Queue` owns both vectors. When the queue is dropped, both vectors (and their heap buffers) are automatically freed.

2. **Element ownership**: Characters pushed into the queue are **moved** (though `char` is `Copy`, so it's copied). The queue takes ownership, and when popped, ownership transfers to the caller.

3. **Mutation via exclusive borrow**: Both `push` and `pop` take `&mut self`, enforcing single-threaded access. You can't push and pop simultaneously from different threads without explicit synchronization.

### The `swap` Operation: Zero-Cost Ownership Transfer

The key insight is the `swap` operation:

```rust
use std::mem::swap;
swap(&mut self.older, &mut self.younger);
```

`std::mem::swap` exchanges the contents of two variables **without allocation**. For `Vec<T>`, this swaps three pointer-sized values (pointer to buffer, length, capacity). Regardless of how many elements the vectors contain, swapping is O(1).

This demonstrates **ownership transfer at the type level**. The ownership of the `younger` vector's buffer moves to `older`, and vice versa. No elements are copied; only the metadata is exchanged.

## Generic Queue: Ownership Across Types

The `generic-queue` extends the pattern to work with any type `T`:

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}

impl<T> Queue<T> {
    pub fn push(&mut self, t: T) {
        self.younger.push(t);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.older.is_empty() {
            use std::mem::swap;

            if self.younger.is_empty() {
                return None;
            }

            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        self.older.pop()
    }

    pub fn split(self) -> (Vec<T>, Vec<T>) {
        (self.older, self.younger)
    }
}
```

### Ownership Implications of Generics

The generic implementation works identically for `Copy` and non-`Copy` types:

```rust
let mut int_queue = Queue::new();
int_queue.push(42);    // i32 is Copy: value is copied
let value = int_queue.pop();  // Value is copied out

let mut string_queue = Queue::new();
string_queue.push(String::from("hello"));  // String is moved in
let s = string_queue.pop();  // String is moved out (no heap copy!)
```

For `String`, the ownership transfer is **zero-cost**:
- `push` moves the `String` into the vector (transfers ownership of heap buffer)
- `pop` moves the `String` out of the vector (transfers ownership back to caller)
- The underlying heap data is never copied—only the `String` struct (pointer, length, capacity) is moved

### The `split` Method: Consuming Transformation

```rust
pub fn split(self) -> (Vec<T>, Vec<T>) {
    (self.older, self.younger)
}
```

Notice that `split` takes `self` **by value**, not `&self` or `&mut self`. This is a **consuming operation**—it destroys the queue and returns its constituent parts.

From the test suite:

```rust
#[test]
fn test_split() {
    let mut q = Queue::new();

    q.push('P');
    q.push('D');
    assert_eq!(q.pop(), Some('P'));
    q.push('X');

    let (older, younger) = q.split();
    // q is now uninitialized - cannot be used
    assert_eq!(older, vec!['D']);
    assert_eq!(younger, vec!['X']);
}
```

After calling `split()`, `q` is no longer valid. The compiler enforces this:

```rust
let (older, younger) = q.split();
q.push('Z');  // Error: use of moved value: `q`
```

This pattern is common in Rust: **consuming methods** that transform a value into something else, making the original unavailable.

## Buffering Strategies and Ownership

Queue-based systems often involve **buffering**—temporarily storing data before processing. Ownership determines how buffering works.

### Strategy 1: Owned Buffering (Queue of Owned Values)

```rust
let mut task_queue: Queue<Task> = Queue::new();

task_queue.push(Task::new("process_image"));  // Task is moved into queue
task_queue.push(Task::new("send_email"));

// Worker processes tasks
while let Some(task) = task_queue.pop() {
    task.execute();  // Task is moved to worker, executed, then dropped
}
```

Each task is **owned** by the queue until popped. The queue manages the tasks' lifetimes.

### Strategy 2: Reference Buffering (Queue of Borrowed Values)

You can't directly create a `Queue<&T>` without lifetime annotations, but you can store references with a specific lifetime:

```rust
struct TaskQueue<'a> {
    queue: Queue<&'a Task>,
}

impl<'a> TaskQueue<'a> {
    fn push(&mut self, task: &'a Task) {
        self.queue.push(task);
    }

    fn pop(&mut self) -> Option<&'a Task> {
        self.queue.pop()
    }
}
```

This allows queueing **references** to tasks instead of owning them. The lifetime `'a` ensures tasks outlive the queue.

### Strategy 3: Shared Ownership Buffering (Arc)

For multi-threaded scenarios where multiple consumers need access to the same data:

```rust
use std::sync::Arc;

let mut shared_queue: Queue<Arc<Task>> = Queue::new();

let task = Arc::new(Task::new("compute"));
shared_queue.push(Arc::clone(&task));  // Increment reference count

// Multiple workers can hold references to the same task
let task_ref1 = shared_queue.pop().unwrap();  // Arc is moved out
let task_ref2 = Arc::clone(&task);             // Another reference

// Task is freed when all Arc references are dropped
```

This uses **shared ownership** via atomic reference counting, allowing multiple components to keep the task alive.

## Synchronous vs. Async Queue Patterns

The queue examples are **synchronous**—operations block the caller. In async systems, queues behave differently.

### Synchronous Queue (This Project)

```rust
let mut q = Queue::new();
q.push(value);         // Immediate operation
let v = q.pop();       // Immediate operation (returns None if empty)
```

**Characteristics**:
- No waiting: operations complete immediately
- Caller handles empty queue (via `Option`)
- Single-threaded: requires `&mut self`

### Bounded Async Queue (Conceptual)

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(100);  // Bounded: max 100 items

// Producer (async)
tx.send(value).await?;  // Waits if queue is full

// Consumer (async)
let value = rx.recv().await;  // Waits if queue is empty
```

**Characteristics**:
- Backpressure: `send` blocks when full
- Async waiting: `recv` suspends task when empty
- Multi-producer/multi-consumer: concurrent access

### Unbounded Async Queue (Conceptual)

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::unbounded_channel();

tx.send(value)?;  // Never blocks (can grow without bound)
let value = rx.recv().await;  // Waits if empty
```

**Characteristics**:
- No backpressure: can exhaust memory
- Useful for low-volume or event-driven scenarios

## Real-World Example: Message Processing Pipeline

Let's design a message processing system using queue ownership patterns:

```rust
struct Message {
    id: u64,
    payload: String,
}

struct MessageProcessor {
    input_queue: Queue<Message>,
    output_queue: Queue<ProcessedMessage>,
}

impl MessageProcessor {
    fn new() -> Self {
        MessageProcessor {
            input_queue: Queue::new(),
            output_queue: Queue::new(),
        }
    }

    fn enqueue_message(&mut self, msg: Message) {
        self.input_queue.push(msg);  // Ownership transferred to queue
    }

    fn process_batch(&mut self, batch_size: usize) {
        for _ in 0..batch_size {
            if let Some(msg) = self.input_queue.pop() {
                // Ownership transferred from input queue
                let processed = ProcessedMessage {
                    id: msg.id,
                    result: process(&msg.payload),
                };
                // Ownership transferred to output queue
                self.output_queue.push(processed);
            } else {
                break;  // No more messages
            }
        }
    }

    fn get_results(&mut self) -> Vec<ProcessedMessage> {
        let mut results = Vec::new();
        while let Some(result) = self.output_queue.pop() {
            results.push(result);  // Ownership transferred to results
        }
        results
    }
}

struct ProcessedMessage {
    id: u64,
    result: String,
}

fn process(payload: &str) -> String {
    payload.to_uppercase()
}
```

**Ownership Flow**:
1. `enqueue_message` moves `Message` into `input_queue`
2. `process_batch` moves `Message` out of `input_queue`, processes it, and moves `ProcessedMessage` into `output_queue`
3. `get_results` moves `ProcessedMessage` out of `output_queue` into a `Vec`

At each stage, ownership is transferred without copying the underlying data (the `String` payloads). The type system ensures:
- No message is processed twice (moved out of queue)
- No message is lost (ownership is always transferred)
- No message is accessed after being freed (borrow checker prevents)

## Performance Characteristics

The two-stack queue has specific performance characteristics tied to ownership:

### Amortized O(1) Operations

- **Push**: O(1) always (append to `younger`)
- **Pop**: O(1) amortized
  - O(1) if `older` is non-empty
  - O(n) if `older` is empty and `younger` must be reversed
  - Amortized O(1) because each element is reversed at most once

### Memory Allocation

```rust
let mut q = Queue::new();

// Initial capacity: 0
for i in 0..100 {
    q.push(i);  // Vec reallocates when capacity is exceeded
}

// After 100 pushes, younger has capacity ~128 (power of 2 growth)
// Total allocations: log₂(100) ≈ 7 reallocations
```

Vectors grow by **doubling capacity**, so pushes are amortized O(1) despite occasional reallocations.

### Comparison with Other Queue Implementations

| Implementation | Push | Pop | Space | Notes |
|----------------|------|-----|-------|-------|
| **Two-stack (this)** | O(1) | O(1) amortized | O(n) | Simple, good for general use |
| **Linked list** | O(1) | O(1) worst-case | O(n) + overhead | More allocations, cache-unfriendly |
| **Ring buffer** | O(1) | O(1) worst-case | O(capacity) | Fixed size, no reallocation |
| **VecDeque** (std) | O(1) | O(1) amortized | O(n) | Ring buffer + reallocation |

The two-stack pattern is simpler than `VecDeque` but has similar performance characteristics.

## Thread-Safe Queue: Adding Synchronization

The basic `Queue` is not thread-safe (no `Sync`). To share across threads, wrap in `Arc<Mutex<T>>`:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let queue = Arc::new(Mutex::new(Queue::new()));

// Producer thread
let queue_clone = Arc::clone(&queue);
let producer = thread::spawn(move || {
    let mut q = queue_clone.lock().unwrap();
    q.push('A');
    q.push('B');
});

// Consumer thread
let queue_clone = Arc::clone(&queue);
let consumer = thread::spawn(move || {
    thread::sleep(Duration::from_millis(100));
    let mut q = queue_clone.lock().unwrap();
    while let Some(c) = q.pop() {
        println!("{}", c);
    }
});

producer.join().unwrap();
consumer.join().unwrap();
```

**Architectural implications**:
- `Arc<T>` provides shared ownership across threads
- `Mutex<T>` provides interior mutability with exclusive access
- Lock contention is a concern: only one thread can access the queue at a time

For high-throughput scenarios, consider lock-free queues (e.g., `crossbeam::queue::SegQueue`).

## Design Patterns Enabled by Queue Ownership

### 1. Pipeline Pattern

```rust
fn pipeline(input: Queue<RawData>) -> Queue<ProcessedData> {
    let mut output = Queue::new();
    while let Some(data) = input.pop() {
        output.push(process(data));
    }
    output
}
```

The input queue is **consumed**, and a new output queue is produced.

### 2. Batch Processing

```rust
fn process_in_batches(mut queue: Queue<Task>, batch_size: usize) {
    while !queue.is_empty() {
        let batch: Vec<_> = (0..batch_size)
            .filter_map(|_| queue.pop())
            .collect();

        // Process batch
        for task in batch {
            task.execute();
        }
    }
}
```

### 3. Splitting Work

```rust
fn split_work(queue: Queue<Task>) -> (Queue<HighPriority>, Queue<LowPriority>) {
    let mut high = Queue::new();
    let mut low = Queue::new();

    while let Some(task) = queue.pop() {
        if task.priority > 5 {
            high.push(task);
        } else {
            low.push(task);
        }
    }

    (high, low)
}
```

## Key Takeaways

1. **Queues transfer ownership**: Elements are moved in and out, not copied (for non-Copy types).

2. **Two-stack pattern is efficient**: O(1) amortized operations with simple implementation.

3. **Generics preserve ownership semantics**: Works identically for `Copy` and `Move` types.

4. **Consuming methods enforce single use**: `split()` consumes the queue, preventing further use.

5. **Thread safety requires explicit synchronization**: Wrap in `Arc<Mutex<T>>` for multi-threaded access.

6. **Async queues differ fundamentally**: Support backpressure and waiting, unlike synchronous queues.

## Architectural Decision Framework

When designing queue-based systems, consider:

### Ownership Model

- **Owned elements** (`Queue<T>`): When the queue manages element lifetimes
- **Borrowed elements** (`Queue<&'a T>`): When elements outlive the queue
- **Shared elements** (`Queue<Arc<T>>`): When multiple consumers need the same element

### Synchronization

- **Single-threaded** (`Queue<T>`): No synchronization overhead
- **Multi-threaded** (`Arc<Mutex<Queue<T>>>`): Explicit locking
- **Lock-free** (external crate): For high-contention scenarios

### Blocking Behavior

- **Synchronous** (this project): Immediate return, caller handles empty queue
- **Async with backpressure**: Producers wait when full, consumers wait when empty
- **Unbounded async**: Producers never wait, risk of unbounded growth

### Capacity

- **Unbounded** (this project): Grows as needed, risk of OOM
- **Bounded**: Fixed maximum size, requires handling of "queue full" condition

## Conclusion

The `queue` and `generic-queue` projects demonstrate how ownership principles shape data structure design:

- **Clear ownership transfer** through `push` and `pop` operations
- **Zero-cost abstractions** via generics and move semantics
- **Compile-time safety** through exclusive mutable borrows
- **Efficient resource management** via RAII and automatic deallocation

These patterns extend beyond queues to any system involving resource pooling, task scheduling, or asynchronous communication. By designing with ownership in mind, you create systems that are **safe, efficient, and architecturally clear**.

## Cross-References

- **Section 2.1: Ownership as Constraint** - Queue as ownership hierarchy
- **Section 2.2: Move Semantics** - Pipeline and consumer patterns
- **Section 2.3: Borrowing as Interface** - `&mut self` in queue methods
- **Chapter 6: Async Architecture** - Async queue patterns (mpsc, broadcast)
- **Chapter 9: Case Studies** - Real-world queue-based architectures

---

**End of Chapter 2: Ownership-Based Architecture**

You now have a comprehensive understanding of how ownership, borrowing, move semantics, and lifetimes shape Rust system architecture. These concepts are the foundation for building safe, efficient, and maintainable systems.

In **Chapter 3: Traits and Generics**, we'll explore how trait-based abstraction builds on ownership principles to create flexible, reusable architectures.
