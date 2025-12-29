# Ownership as Architectural Constraint

## Learning Objectives

By the end of this section, you will understand:
- How Rust's ownership rules shape system architecture
- The single-owner principle and its implications for resource management
- How ownership graphs prevent resource leaks and data races
- Architectural patterns that emerge from ownership constraints

## Introduction: Ownership as Design Force

In most languages, memory management is an implementation detail—something you handle after designing your system's architecture. Rust inverts this relationship: **ownership becomes a primary architectural constraint that shapes how you design systems from the ground up**.

This isn't a limitation; it's a feature. By encoding resource ownership into the type system, Rust forces you to make explicit architectural decisions that would otherwise remain implicit and error-prone. The result is systems where resource management, concurrency safety, and data flow are manifest in the code structure itself.

## The Single-Owner Principle

At the core of Rust's ownership system is a deceptively simple rule: **every value has exactly one owner at any given time**. This principle has profound implications for how we architect systems.

Consider the `Queue` type from the `queue` project:

```rust
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last.
    younger: Vec<char>  // younger elements, youngest last.
}
```

This structure establishes clear ownership boundaries. The `Queue` owns two `Vec<char>` instances. Those vectors, in turn, own their heap-allocated buffers. When the queue is dropped, Rust automatically and deterministically frees all associated resources in reverse order of acquisition—a perfect implementation of RAII (Resource Acquisition Is Initialization).

### Ownership at System Boundaries

The single-owner principle becomes particularly important at system boundaries. Consider this method signature from the queue implementation:

```rust
pub fn split(self) -> (Vec<char>, Vec<char>) {
    (self.older, self.younger)
}
```

Note that `split` takes `self` by value, not by reference. This is an **ownership transfer**—the caller gives up ownership of the queue and receives ownership of the two vectors. After calling `split()`, the original queue no longer exists in the caller's scope. The type system enforces this: attempting to use the queue after calling `split()` results in a compile-time error.

This pattern forces architectural clarity. The method signature communicates: "This operation consumes the queue and decomposes it into its constituents." There's no ambiguity about whether the queue remains valid or whether the vectors are copies or views. The ownership semantics are explicit and compiler-verified.

## Ownership Graphs and Resource Management

In complex systems, ownership relationships form directed acyclic graphs (DAGs). Each resource has exactly one owner, but that owner may itself be owned by another resource, forming ownership chains.

The `binary-tree` project demonstrates this clearly:

```rust
enum BinaryTree<T> {
    Empty,
    NonEmpty(Box<TreeNode<T>>),
}

struct TreeNode<T> {
    element: T,
    left: BinaryTree<T>,
    right: BinaryTree<T>,
}
```

Here, the ownership graph is a tree structure:
- A `BinaryTree<T>` owns a `Box<TreeNode<T>>`
- That `TreeNode<T>` owns an element of type `T` and two child `BinaryTree<T>` instances
- Those children, in turn, may own their own nodes

When the root tree goes out of scope, Rust automatically traverses this ownership graph in reverse order, dropping each node recursively. This is safe because ownership is exclusive—no node can have multiple parents, preventing cycles and ensuring deterministic cleanup.

### Preventing Resource Leaks Through Type Safety

The ownership graph prevents an entire class of resource management bugs:

1. **No dangling pointers**: You can't have a reference to a node whose parent was deallocated, because the parent owns the children. The type system ensures children can't outlive their parent.

2. **No use-after-free**: Once ownership is transferred (via move semantics), the original binding is invalidated. The compiler prevents use of moved values.

3. **No double-free**: Each allocation has exactly one owner responsible for cleanup. Dropping that owner frees the resource exactly once.

4. **No memory leaks**: Resources are tied to ownership. When the owner goes out of scope, the resource is automatically freed. There's no way to "forget" to clean up.

## Implications for Multi-Threaded Systems

The single-owner principle has critical implications for concurrent systems. Data races occur when multiple threads access the same memory concurrently with at least one write. Rust's ownership system prevents this at compile time through a simple rule:

**You can't share mutable state across threads without explicit synchronization.**

This emerges naturally from ownership rules. To send data to another thread, you must transfer ownership:

```rust
use std::thread;

let data = vec![1, 2, 3, 4, 5];

// Transfer ownership to the new thread
let handle = thread::spawn(move || {
    // This closure takes ownership of `data`
    println!("Sum: {}", data.iter().sum::<i32>());
});

// Error: `data` was moved into the closure
// println!("{:?}", data);

handle.join().unwrap();
```

Once `data` is moved into the spawned thread, the original thread can no longer access it. There's no possibility of a data race because only one thread owns the data at any time.

For shared ownership across threads, Rust requires explicit opt-in via `Arc<T>` (atomic reference counting). For shared mutable state, you must use synchronization primitives like `Mutex<T>`:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    let handle = thread::spawn(move || {
        let mut num = counter.lock().unwrap();
        *num += 1;
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

println!("Result: {}", *counter.lock().unwrap());
```

The type system enforces that:
- `Arc<T>` provides shared ownership (multiple owners via reference counting)
- `Mutex<T>` provides interior mutability with runtime-checked exclusive access
- You cannot access the inner `T` without acquiring the lock

This architecture makes concurrency explicit. You can't accidentally create a data race—the type system requires you to encode your synchronization strategy in the types themselves.

## Architectural Decision Framework

When designing with ownership in mind, ask:

### 1. Who should own this resource?

**Single owner**: Use direct ownership for clear lifetime hierarchies.
```rust
struct Config {
    database: Database,  // Config owns the database connection
    cache: Cache,        // Config owns the cache
}
```

**Shared ownership**: Use `Rc<T>` (single-threaded) or `Arc<T>` (multi-threaded) when multiple components need to keep a resource alive.
```rust
use std::rc::Rc;

struct Node {
    value: i32,
    parent: Option<Rc<Node>>,  // Multiple children can share a parent
}
```

### 2. Does ownership need to be transferred?

**Consuming operations**: Take ownership (`self`) when the operation invalidates the original.
```rust
pub fn into_vec(self) -> Vec<T> {
    // Consume and transform
}
```

**Non-consuming operations**: Borrow (`&self` or `&mut self`) when the original remains valid.
```rust
pub fn len(&self) -> usize {
    // Read-only access
}
```

### 3. Can this resource be shared across threads?

**Thread-local**: Use `Rc<T>` for shared ownership within a single thread.

**Thread-safe**: Use `Arc<T>` for shared ownership across threads.

**Mutable and shared**: Combine with `Mutex<T>`, `RwLock<T>`, or atomic types for safe concurrent mutation.

## Real-World Example: Queue Ownership

The queue implementation demonstrates ownership principles in practice:

```rust
impl Queue {
    pub fn push(&mut self, c: char) {
        self.younger.push(c);  // Queue owns the vectors, can mutate them
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }

            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);  // Transfer ownership between fields
            self.older.reverse();
        }

        self.older.pop()  // Transfer ownership from vector to caller
    }
}
```

The `pop` method exemplifies ownership architecture:
1. It takes `&mut self` (exclusive mutable borrow) to prevent concurrent modification
2. It uses `swap` to efficiently transfer ownership between the `older` and `younger` vectors
3. It returns `Option<char>`, transferring ownership of the character to the caller
4. The queue retains ownership of its internal vectors

This design is impossible to use incorrectly. The type system ensures you can't:
- Pop from a queue while iterating over it
- Access the queue from multiple threads without synchronization
- Leak memory by forgetting to free elements

## Conclusion

Ownership transforms from a memory management mechanism into an architectural tool. By encoding resource ownership in the type system, Rust forces explicit decisions about:

- **Responsibility**: Who owns each resource?
- **Lifetime**: When is each resource freed?
- **Transfer**: When and how is ownership transferred?
- **Sharing**: How is shared access coordinated?

These constraints might seem restrictive at first, but they eliminate entire classes of bugs and make complex systems more understandable. The ownership graph becomes a map of your system's resource management strategy, compiler-verified and impossible to violate.

In the next section, we'll explore how **move semantics** enable powerful architectural patterns like pipelines and state machines.

## Cross-References

- **Chapter 3: Traits and Generics** - How trait bounds interact with ownership
- **Chapter 6: Async Architecture** - Ownership in async contexts with `Send` and `Sync`
- **Section 2.3: Borrowing as Interface** - Temporary access without ownership transfer
- **Section 2.5: Queue-Based Systems** - Deep dive into the queue implementation
