# Concurrent System Design

Concurrent system design in Rust centers on choosing between message passing and shared state—two fundamentally different approaches to coordinating multiple tasks. Each has distinct implications for correctness, performance, and maintainability. Understanding when to apply each pattern and how to implement them effectively determines the success of your concurrent architecture.

## Message Passing: The Actor Model

Message passing eliminates shared state by having tasks communicate through channels. Each task owns its data, preventing data races by construction.

### Channel-Based Concurrency

```rust
use async_std::channel::{self, Sender, Receiver};
use async_std::task;

enum Command {
    Fetch(String, Sender<Result<String, Error>>),
    Shutdown,
}

async fn worker_task(receiver: Receiver<Command>) {
    while let Ok(cmd) = receiver.recv().await {
        match cmd {
            Command::Fetch(url, reply) => {
                let result = fetch_data(&url).await;
                reply.send(result).await.ok();  // Ignore send errors
            }
            Command::Shutdown => break,
        }
    }
}

async fn coordinator() {
    let (sender, receiver) = channel::bounded(100);

    // Spawn worker
    task::spawn(worker_task(receiver));

    // Send work
    let (reply_tx, reply_rx) = channel::bounded(1);
    sender.send(Command::Fetch("https://example.com".to_string(), reply_tx)).await.ok();

    // Wait for response
    if let Ok(result) = reply_rx.recv().await {
        println!("Got result: {:?}", result);
    }

    // Shutdown
    sender.send(Command::Shutdown).await.ok();
}
```

This pattern creates clear ownership boundaries:

- **Worker owns** the receive end of the command channel
- **Coordinator owns** the send end
- **No shared state** between tasks
- **Type safety** enforces protocol via the `Command` enum

### Benefits of Message Passing

1. **No data races**: Impossible by construction—no shared mutable state
2. **Clear interfaces**: Channels define communication protocols
3. **Isolation**: Tasks fail independently
4. **Testing**: Mock channels for unit testing individual tasks
5. **Distribution**: Scales to distributed systems (with appropriate channels)

### Drawbacks

1. **Ownership constraints**: Must transfer or clone data across channels
2. **Latency**: Message passing adds overhead compared to shared memory
3. **Complexity**: Requires managing channel lifecycle and error handling
4. **Backpressure**: Bounded channels can block; unbounded can OOM

**When to use**: Prefer message passing for complex coordination, distributed systems, or when isolation is critical.

## Shared State: Arc and Mutex

Shared state allows multiple tasks to access the same data concurrently through synchronization primitives. Rust's type system ensures safe access through `Arc` (atomic reference counting) and `Mutex` (mutual exclusion).

### The Arc<Mutex<T>> Pattern

From the `spawn-blocking` project:

```rust
use std::sync::{Arc, Mutex};
use std::task::Waker;

pub struct SpawnBlocking<T>(Arc<Mutex<Shared<T>>>);

struct Shared<T> {
    value: Option<T>,
    waker: Option<Waker>,
}

pub fn spawn_blocking<T, F>(closure: F) -> SpawnBlocking<T>
where F: FnOnce() -> T,
      F: Send + 'static,
      T: Send + 'static,
{
    let inner = Arc::new(Mutex::new(Shared {
        value: None,
        waker: None,
    }));

    std::thread::spawn({
        let inner = inner.clone();  // Clone the Arc, not the data
        move || {
            let value = closure();

            let maybe_waker = {
                let mut guard = inner.lock().unwrap();
                guard.value = Some(value);
                guard.waker.take()
            };

            if let Some(waker) = maybe_waker {
                waker.wake();
            }
        }
    });

    SpawnBlocking(inner)
}
```

This code demonstrates several crucial patterns:

**Arc for shared ownership**: The `Arc::clone()` creates a new reference, not a data copy. Both the spawned thread and the returned `SpawnBlocking` can access `inner`.

**Mutex for synchronized access**: The `Mutex` ensures only one thread accesses `Shared` at a time. The `lock()` call blocks until access is available.

**Minimal critical sections**: Lock is held only while modifying `value` and taking the `waker`, then released before calling `waker.wake()`. Keeping critical sections small reduces contention.

**Waker protocol**: The worker stores the waker under lock, takes it when work completes, then releases the lock before waking. This prevents deadlocks.

### Implementing the Future

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

impl<T: Send> Future for SpawnBlocking<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut guard = self.0.lock().unwrap();
        if let Some(value) = guard.value.take() {
            return Poll::Ready(value);
        }

        guard.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}
```

The polling logic:
1. **Lock the shared state**
2. **Check for completion**: If value is present, return it
3. **Store waker**: If not ready, store the waker for later notification
4. **Return pending**: Release lock and suspend

This pattern bridges blocking operations (running in threads) with async operations (the returned future).

### Benefits of Shared State

1. **Performance**: No message copying, direct memory access
2. **Simplicity**: For simple coordination, less boilerplate than channels
3. **Flexibility**: Multiple readers/writers with appropriate synchronization
4. **Atomic operations**: Lock-free updates for simple cases (with `AtomicUsize`, etc.)

### Drawbacks

1. **Lock contention**: High contention degrades performance
2. **Deadlock risk**: Multiple locks can deadlock if not carefully ordered
3. **Error handling**: `Mutex::lock()` can panic if poisoned
4. **Testing complexity**: Shared state is harder to test in isolation

**When to use**: Prefer shared state for performance-critical sections with simple coordination or when data is naturally shared (caches, connection pools).

## Lock-Free Data Structures

For the highest performance, consider lock-free structures using atomic operations:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct Counter {
    count: AtomicUsize,
}

impl Counter {
    fn new() -> Self {
        Counter { count: AtomicUsize::new(0) }
    }

    fn increment(&self) -> usize {
        self.count.fetch_add(1, Ordering::Relaxed)
    }

    fn get(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}

// Use across async tasks
async fn count_requests(counter: Arc<Counter>) {
    let handles: Vec<_> = (0..1000).map(|_| {
        let counter = counter.clone();
        task::spawn(async move {
            counter.increment();
        })
    }).collect();

    for handle in handles {
        handle.await;
    }

    println!("Total requests: {}", counter.get());
}
```

Atomic operations provide lock-free concurrency for simple cases like counters, flags, or indices. They're extremely fast but limited to specific data types and operations.

### When Lock-Free Applies

Use atomics when:
- **Simple operations**: Counters, flags, single-value updates
- **No complex invariants**: Can't maintain multi-field consistency atomically
- **High contention**: Many readers/writers where locks would bottleneck

Avoid when:
- **Complex data**: Structs with multiple fields requiring coordinated updates
- **Algorithms are complex**: Lock-free algorithms are notoriously hard to get right
- **Readability matters**: Locks are easier to reason about

## Choosing Between Message Passing and Shared State

Use this decision framework:

```
Do tasks need coordinated access to complex shared data?
├─ No: Use message passing ✓
│  └─ Are tasks logically independent?
│     ├─ Yes: Message passing ideal ✓
│     └─ No: Consider shared state
│
└─ Yes: How many readers/writers?
   ├─ One writer, many readers: RwLock<T> ✓
   ├─ Many writers, simple data: Arc<Mutex<T>> ✓
   ├─ Many writers, high contention: Lock-free or sharding
   └─ Complex coordination: Message passing with state machine
```

### Hybrid Approaches

Real systems often combine both patterns:

```rust
struct System {
    // Shared read-only configuration
    config: Arc<Config>,

    // Message passing for work distribution
    work_queue: Sender<WorkItem>,

    // Shared mutable state with synchronization
    metrics: Arc<Mutex<Metrics>>,

    // Lock-free for high-frequency updates
    request_count: Arc<AtomicUsize>,
}
```

This hybrid approach uses each pattern where it fits best:
- **Arc for shared config**: No synchronization needed for read-only data
- **Channels for work**: Clean task isolation
- **Mutex for metrics**: Infrequent updates, complex data
- **Atomics for counters**: Frequent updates, simple data

## Real-World Pattern: Worker Pool

Combining message passing and shared state creates effective worker pools:

```rust
use async_std::channel;
use std::sync::Arc;

struct WorkerPool<T, R> {
    workers: usize,
    sender: Sender<WorkItem<T, R>>,
}

enum WorkItem<T, R> {
    Task(T, Sender<R>),
    Shutdown,
}

impl<T, R> WorkerPool<T, R>
where
    T: Send + 'static,
    R: Send + 'static,
{
    fn new<F>(workers: usize, handler: F) -> Self
    where
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        let (sender, receiver) = channel::bounded(workers * 2);
        let handler = Arc::new(handler);

        for _ in 0..workers {
            let receiver = receiver.clone();
            let handler = handler.clone();

            task::spawn(async move {
                while let Ok(item) = receiver.recv().await {
                    match item {
                        WorkItem::Task(task, reply) => {
                            let result = handler(task);
                            reply.send(result).await.ok();
                        }
                        WorkItem::Shutdown => break,
                    }
                }
            });
        }

        WorkerPool { workers, sender }
    }

    async fn execute(&self, task: T) -> Result<R, Error> {
        let (reply_tx, reply_rx) = channel::bounded(1);
        self.sender.send(WorkItem::Task(task, reply_tx)).await?;
        Ok(reply_rx.recv().await?)
    }

    async fn shutdown(self) {
        for _ in 0..self.workers {
            self.sender.send(WorkItem::Shutdown).await.ok();
        }
    }
}
```

This pattern:
- **Uses channels** for work distribution
- **Shares handler** via `Arc` (read-only, so no Mutex needed)
- **Isolates workers** for failure independence
- **Bounds concurrency** via channel capacity

## Deadlock Prevention

When using shared state, prevent deadlocks with strict lock ordering:

```rust
struct Account {
    balance: Mutex<u64>,
}

// WRONG: Can deadlock if two threads transfer in opposite directions
fn transfer_bad(from: &Account, to: &Account, amount: u64) {
    let mut from_balance = from.balance.lock().unwrap();
    let mut to_balance = to.balance.lock().unwrap();  // Potential deadlock
    *from_balance -= amount;
    *to_balance += amount;
}

// RIGHT: Always lock in consistent order (e.g., by memory address)
fn transfer_safe(from: &Account, to: &Account, amount: u64) {
    use std::cmp::Ordering;

    let (first, second) = match (from as *const Account).cmp(&(to as *const Account)) {
        Ordering::Less => (from, to),
        Ordering::Greater => (to, from),
        Ordering::Equal => return,  // Same account
    };

    let mut first_balance = first.balance.lock().unwrap();
    let mut second_balance = second.balance.lock().unwrap();

    // Actual transfer logic...
}
```

Consistent lock ordering eliminates circular wait conditions, preventing deadlocks.

## Performance Tuning

Profile and optimize concurrent systems with these strategies:

1. **Minimize critical sections**: Hold locks for minimal time
2. **Use RwLock for read-heavy**: Multiple readers, infrequent writers
3. **Shard data**: Reduce contention by splitting data across multiple locks
4. **Batch operations**: Amortize lock acquisition cost
5. **Prefer message passing for complexity**, shared state for speed

Concurrent system design is about choosing the right coordination primitive for each component. Message passing provides safety and isolation; shared state provides performance. Combine them judiciously to build robust, scalable systems that leverage Rust's safety guarantees while meeting performance requirements.
