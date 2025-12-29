# Borrowing as Architectural Interface

## Learning Objectives

By the end of this section, you will understand:
- How borrowing rules define API contracts
- Immutable borrows as read-only guarantees
- Mutable borrows as exclusive access guarantees
- Interior mutability patterns and when to use them
- Thread-safety through `Arc<Mutex<T>>` and `Arc<RwLock<T>>`

## Introduction: Borrowing as Contract

If ownership represents **permanent responsibility** for a resource, borrowing represents **temporary access**. In Rust, borrowing isn't just a convenience—it's a fundamental architectural interface that encodes access patterns directly in the type system.

Every function signature that accepts a reference is making an explicit contract:
- `&T` means "I will read this data, but not modify it, and I won't keep it beyond this function's scope"
- `&mut T` means "I need exclusive access to modify this data, and I won't keep it beyond this function's scope"

These contracts are compiler-verified, making entire classes of bugs impossible: no data races, no iterator invalidation, no use-after-free.

## Immutable Borrows: Read-Only Contracts

An immutable borrow (`&T`) provides read-only access to data. Multiple immutable borrows can exist simultaneously, enabling concurrent reads without locks.

### Example: Queue Inspection

From the `queue` project:

```rust
impl Queue {
    pub fn is_empty(&self) -> bool {
        self.older.is_empty() && self.younger.is_empty()
    }
}
```

The `is_empty` method takes `&self`—an immutable borrow of the queue. This signature guarantees:
1. The method won't modify the queue
2. Multiple threads could call `is_empty` simultaneously (if the queue is `Sync`)
3. The queue remains usable after the method returns

This is a **read-only architectural interface**. Callers can inspect the queue's state without fear of side effects:

```rust
let mut q = Queue::new();
q.push('a');

// Multiple immutable borrows allowed
let empty1 = q.is_empty();
let empty2 = q.is_empty();
let empty3 = q.is_empty();

// All can be used together
if !empty1 && !empty2 && !empty3 {
    println!("Queue is not empty");
}
```

### Immutable Borrows Enable Aliasing

Unlike mutable borrows, immutable borrows allow **aliasing**—multiple references to the same data:

```rust
let data = vec![1, 2, 3, 4, 5];

let slice1 = &data[0..3];  // Immutable borrow
let slice2 = &data[2..5];  // Another immutable borrow (overlaps!)

// Both can be used together
println!("{:?} and {:?}", slice1, slice2);  // OK: both are read-only
```

This is safe because neither reference can modify the underlying data. The compiler guarantees no modifications can occur while these borrows exist.

### Architectural Pattern: Shared State with Immutable Views

Immutable borrows enable architectural patterns where multiple components share read-only access to centralized state:

```rust
struct Configuration {
    database_url: String,
    api_key: String,
    timeout: Duration,
}

struct DatabaseService<'a> {
    config: &'a Configuration,
}

struct ApiClient<'a> {
    config: &'a Configuration,
}

impl<'a> DatabaseService<'a> {
    fn new(config: &'a Configuration) -> Self {
        DatabaseService { config }
    }

    fn connect(&self) -> Connection {
        connect(&self.config.database_url)
    }
}

impl<'a> ApiClient<'a> {
    fn new(config: &'a Configuration) -> Self {
        ApiClient { config }
    }

    fn fetch(&self, endpoint: &str) -> Response {
        // Uses self.config.api_key and self.config.timeout
    }
}

// Usage: multiple services share the same config
let config = Configuration { /* ... */ };
let db = DatabaseService::new(&config);
let api = ApiClient::new(&config);

// Both services can read config simultaneously
db.connect();
api.fetch("/users");
```

The lifetime `'a` ensures that both services can't outlive the configuration they reference—a compile-time guarantee that prevents dangling pointers.

## Mutable Borrows: Exclusive Access Guarantees

A mutable borrow (`&mut T`) provides exclusive write access. **Only one mutable borrow can exist at a time**, and no immutable borrows can coexist with it.

This rule prevents data races at compile time and ensures that mutations are isolated and controlled.

### Example: Queue Modification

```rust
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

Both `push` and `pop` take `&mut self`, signaling:
1. These methods modify the queue
2. You can't call them simultaneously
3. You can't read the queue while calling these methods

This architectural interface prevents common bugs:

```rust
let mut q = Queue::new();
q.push('a');
q.push('b');

// Error: can't borrow immutably while borrowed mutably
// let is_empty = q.is_empty();  // Immutable borrow
// q.push('c');                   // Mutable borrow (conflict!)

// Correct: finish mutable access before immutable access
q.push('c');                   // Mutable borrow ends here
let is_empty = q.is_empty();  // Now immutable borrow is OK
```

### The Split Borrow Pattern

Rust's borrow checker is smart enough to allow **split borrows**—borrowing different parts of a struct simultaneously:

```rust
impl Queue {
    fn process(&mut self) {
        self.rebalance(&mut self.older, &mut self.younger);
    }

    fn rebalance(&mut self, older: &mut Vec<char>, younger: &mut Vec<char>) {
        // Can mutably borrow both fields simultaneously
        use std::mem::swap;
        swap(older, younger);
        older.reverse();
    }
}
```

This works because `self.older` and `self.younger` are distinct fields. The borrow checker understands they don't alias, so mutable borrows of both are safe.

### Architectural Pattern: Exclusive Mutation Windows

Mutable borrows define **mutation windows**—regions where data can be modified, surrounded by regions where it's immutable:

```rust
let mut data = vec![1, 2, 3, 4, 5];

// Immutable window: multiple readers allowed
println!("Original: {:?}", data);
let first = &data[0];
let last = &data[4];

// Mutation window: exclusive access required
{
    let mutable_ref = &mut data;
    mutable_ref.push(6);
    mutable_ref.push(7);
}  // Mutable borrow ends here

// Back to immutable window
println!("Modified: {:?}", data);
```

This pattern makes mutation explicit and localized, preventing action-at-a-distance bugs.

## Interior Mutability: Controlled Exceptions

Sometimes you need to mutate data through an immutable reference. Rust provides **interior mutability** patterns that use runtime checks to enforce borrowing rules.

### Cell<T>: Single-Threaded Interior Mutability

`Cell<T>` allows mutation through a shared reference, but only for `Copy` types:

```rust
use std::cell::Cell;

struct Counter {
    count: Cell<i32>,  // Interior mutability
}

impl Counter {
    fn increment(&self) {  // Takes &self, not &mut self!
        let current = self.count.get();
        self.count.set(current + 1);
    }

    fn get(&self) -> i32 {
        self.count.get()
    }
}

let counter = Counter { count: Cell::new(0) };
counter.increment();  // Mutates through &self
counter.increment();
assert_eq!(counter.get(), 2);
```

`Cell<T>` bypasses the borrow checker by moving values in and out (via `get` and `set`) instead of borrowing them. This is safe for `Copy` types because there's no aliasing of non-Copy data.

### RefCell<T>: Runtime-Checked Borrowing

`RefCell<T>` provides interior mutability for non-Copy types by enforcing borrowing rules at **runtime**:

```rust
use std::cell::RefCell;

struct Cache {
    data: RefCell<HashMap<String, String>>,
}

impl Cache {
    fn get(&self, key: &str) -> Option<String> {
        let data = self.data.borrow();  // Runtime immutable borrow
        data.get(key).cloned()
    }

    fn insert(&self, key: String, value: String) {
        let mut data = self.data.borrow_mut();  // Runtime mutable borrow
        data.insert(key, value);
    }
}

let cache = Cache { data: RefCell::new(HashMap::new()) };
cache.insert("key".to_string(), "value".to_string());
assert_eq!(cache.get("key"), Some("value".to_string()));
```

`RefCell<T>` tracks borrows at runtime using a counter. If you violate the borrowing rules (e.g., try to get a mutable borrow while an immutable borrow exists), the program **panics** at runtime.

**When to use `RefCell<T>`**:
- When you need mutation through a shared reference
- When you can't restructure your code to use compile-time mutable borrows
- When you're certain the borrow rules won't be violated (or panicking is acceptable)

**Warning**: `RefCell<T>` trades compile-time safety for flexibility. Use it sparingly and document the invariants carefully.

## Thread-Safe Sharing: Arc<Mutex<T>> and Arc<RwLock<T>>

For multi-threaded scenarios, Rust provides thread-safe interior mutability through atomic reference counting and locks.

### Arc<Mutex<T>>: Shared Mutable State Across Threads

`Arc<T>` (Atomic Reference Counted) provides shared ownership across threads. Combined with `Mutex<T>`, it enables safe concurrent mutation:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);  // Clone the Arc, not the data
    let handle = thread::spawn(move || {
        let mut num = counter.lock().unwrap();  // Acquire lock
        *num += 1;  // Exclusive access guaranteed
    });  // Lock automatically released when `num` goes out of scope
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

assert_eq!(*counter.lock().unwrap(), 10);
```

The architecture here is explicit:
- `Arc<T>` provides shared ownership (multiple threads can own the counter)
- `Mutex<T>` provides interior mutability (mutation through shared references)
- `lock()` enforces exclusive access at runtime (blocks if another thread holds the lock)

### Arc<RwLock<T>>: Optimized for Read-Heavy Workloads

`RwLock<T>` (Read-Write Lock) allows multiple concurrent readers or one exclusive writer:

```rust
use std::sync::{Arc, RwLock};
use std::thread;

let config = Arc::new(RwLock::new(HashMap::new()));

// Multiple reader threads
let mut reader_handles = vec![];
for i in 0..5 {
    let config = Arc::clone(&config);
    let handle = thread::spawn(move || {
        let data = config.read().unwrap();  // Shared read access
        println!("Reader {}: {:?}", i, data.get("key"));
    });
    reader_handles.push(handle);
}

// One writer thread
let config_writer = Arc::clone(&config);
let writer_handle = thread::spawn(move || {
    let mut data = config_writer.write().unwrap();  // Exclusive write access
    data.insert("key".to_string(), "value".to_string());
});

for handle in reader_handles {
    handle.join().unwrap();
}
writer_handle.join().unwrap();
```

**When to use `RwLock<T>` instead of `Mutex<T>`**:
- When reads vastly outnumber writes
- When read operations are expensive and you want true concurrency for readers
- When the overhead of tracking reader counts is acceptable

## Decision Framework: Choosing the Right Borrowing Pattern

| Scenario | Pattern | Example |
|----------|---------|---------|
| **Read-only access** | `&T` | `fn len(&self) -> usize` |
| **Exclusive modification** | `&mut T` | `fn push(&mut self, item: T)` |
| **Shared state, single thread** | `Rc<RefCell<T>>` | GUI with shared mutable state |
| **Shared state, multi-threaded** | `Arc<Mutex<T>>` | Web server with request counter |
| **Read-heavy multi-threaded** | `Arc<RwLock<T>>` | Configuration cache |
| **Caching small values** | `Cell<T>` (Copy types only) | Timestamp or counter |

## Architectural Implications

Borrowing rules shape system architecture in profound ways:

### 1. No Iterator Invalidation

```rust
let mut v = vec![1, 2, 3, 4, 5];

// This won't compile: can't modify while iterating
// for item in &v {
//     v.push(*item * 2);  // Error: can't borrow mutably while borrowed immutably
// }

// Correct: collect into new vector
let doubled: Vec<_> = v.iter().map(|x| x * 2).collect();
v.extend(doubled);
```

The borrow checker prevents iterator invalidation bugs that plague C++ and other languages.

### 2. Explicit Synchronization

In multi-threaded code, synchronization is explicit in the types:

```rust
// Thread-safe: types enforce synchronization
Arc<Mutex<T>>  // Requires lock acquisition
Arc<RwLock<T>> // Requires read or write lock

// Not thread-safe: won't compile in multi-threaded context
Rc<RefCell<T>>  // Single-threaded only (not Sync)
```

You can't accidentally forget to synchronize—the type system requires it.

### 3. Clear Ownership Hierarchies

Borrowing relationships establish clear hierarchies:

```rust
struct Database {
    connections: Vec<Connection>,
}

impl Database {
    fn get_connection(&self) -> &Connection {
        &self.connections[0]  // Lifetime tied to &self
    }
}

// The returned &Connection can't outlive the Database
```

This prevents dangling pointers and use-after-free bugs at compile time.

## Real-World Example: Queue with Interior Mutability

While the standard `Queue` uses `&mut self` for mutations, you could design a thread-safe queue using interior mutability:

```rust
use std::sync::{Arc, Mutex};

struct ThreadSafeQueue<T> {
    inner: Arc<Mutex<Queue<T>>>,
}

impl<T> ThreadSafeQueue<T> {
    fn new() -> Self {
        ThreadSafeQueue {
            inner: Arc::new(Mutex::new(Queue::new())),
        }
    }

    fn push(&self, item: T) {  // Takes &self, not &mut self
        let mut queue = self.inner.lock().unwrap();
        queue.push(item);
    }  // Lock released automatically

    fn pop(&self) -> Option<T> {
        let mut queue = self.inner.lock().unwrap();
        queue.pop()
    }

    fn clone(&self) -> Self {
        ThreadSafeQueue {
            inner: Arc::clone(&self.inner),
        }
    }
}
```

This design trades compile-time guarantees for runtime flexibility. Multiple threads can share a `ThreadSafeQueue`, but lock contention becomes a performance consideration.

## Conclusion

Borrowing isn't just syntax—it's an architectural interface that encodes access patterns in the type system:

- **Immutable borrows** (`&T`) guarantee read-only access and enable concurrent reads
- **Mutable borrows** (`&mut T`) guarantee exclusive access and prevent data races
- **Interior mutability** (`RefCell<T>`, `Mutex<T>`) trades compile-time checks for runtime flexibility
- **Thread-safe sharing** (`Arc<Mutex<T>>`, `Arc<RwLock<T>>`) makes synchronization explicit

By choosing the right borrowing pattern, you encode your concurrency and access control strategy directly in the types, making entire classes of bugs impossible.

In the next section, we'll explore **lifetimes in large systems**—how lifetime annotations scale to complex architectures.

## Cross-References

- **Section 2.1: Ownership as Constraint** - The foundation of borrowing
- **Section 2.2: Move Semantics** - When to move instead of borrow
- **Chapter 6: Async Architecture** - Borrowing in async contexts
- **Chapter 3: Traits and Generics** - Trait bounds involving lifetimes and borrows
