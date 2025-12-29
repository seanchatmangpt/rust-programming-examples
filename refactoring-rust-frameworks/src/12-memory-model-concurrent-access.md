# Memory Model & Concurrent Access Patterns

## Introduction: Understanding the Foundation

AI agents working with Rust concurrent code must understand the memory model at a deep level. Unlike higher-level concurrency abstractions, the memory model defines the fundamental rules governing how multiple threads interact with shared memory. This chapter provides the technical foundation necessary for verifying concurrent code correctness, detecting subtle bugs, and building lock-free data structures.

The Rust memory model builds upon C++20's memory model while leveraging Rust's ownership system to provide additional safety guarantees. Understanding this model is essential for:

- Verifying that concurrent algorithms are correct
- Choosing appropriate atomic operations and orderings
- Detecting data races and race conditions
- Building performant concurrent data structures
- Analyzing existing concurrent code for correctness

## 1. The Rust Memory Model

### Load and Store Semantics

At the hardware level, memory operations are not instantaneous or atomic by default. Modern processors reorder instructions, cache values, and perform speculative execution. The memory model defines what behaviors programs can observe.

**Basic Memory Operations:**

```rust
// Non-atomic load and store (UNSAFE in concurrent context)
let x: i32 = 42;
let y = x;  // Load
x = 10;     // Store
```

These operations appear sequential in single-threaded code, but in multi-threaded contexts without synchronization, they exhibit undefined behavior:

```rust
static mut X: i32 = 0;

// Thread 1
unsafe { X = 42; }  // UNDEFINED BEHAVIOR: Data race

// Thread 2
unsafe { let y = X; }  // UNDEFINED BEHAVIOR: Data race
```

The Rust compiler (correctly) rejects most such code at compile time through the type system.

### Happens-Before Relationships

The **happens-before** relation defines ordering guarantees between operations in different threads. If operation A happens-before operation B, then:

1. A's effects are visible to B
2. A is sequenced before B in program order
3. No reordering can place B before A

**Establishing Happens-Before:**

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let flag = Arc::new(AtomicBool::new(false));
let data = Arc::new(AtomicBool::new(false));

// Thread 1
data.store(true, Ordering::Relaxed);  // A
flag.store(true, Ordering::Release);   // B (establishes happens-before)

// Thread 2
if flag.load(Ordering::Acquire) {      // C (synchronizes with B)
    assert!(data.load(Ordering::Relaxed));  // D (sees A's effects)
}
```

The `Release` store at B happens-before the `Acquire` load at C, which means all operations before B (including A) are visible after C.

### Sequential Consistency

**Sequential consistency** (SeqCst) is the strongest memory ordering, providing a total global order of all SeqCst operations across all threads. This matches intuitive expectations but has performance costs.

```rust
use std::sync::atomic::{AtomicI32, Ordering};

static X: AtomicI32 = AtomicI32::new(0);
static Y: AtomicI32 = AtomicI32::new(0);

// With SeqCst, this cannot print "00"
// Thread 1
X.store(1, Ordering::SeqCst);
println!("{}", Y.load(Ordering::SeqCst));

// Thread 2
Y.store(1, Ordering::SeqCst);
println!("{}", X.load(Ordering::SeqCst));
```

With `SeqCst`, at least one thread will observe the other's write, preventing "00" output. With weaker orderings, "00" is possible due to reordering.

### Weak Memory Ordering

**Weak memory models** allow more reordering, enabling better performance on modern CPUs. Different platforms have different levels of "weakness":

**x86/x64 (Strong Ordering):**
- Loads are not reordered with loads
- Stores are not reordered with stores
- Stores are not reordered with prior loads
- Loads may be reordered with prior stores (store buffer)

**ARM/POWER (Weak Ordering):**
- Almost all reorderings are possible without barriers
- Requires explicit memory barriers for synchronization
- Much larger performance gap between orderings

**Example showing platform differences:**

```rust
// Dekker's algorithm (requires sequential consistency)
use std::sync::atomic::{AtomicBool, Ordering};

static FLAG1: AtomicBool = AtomicBool::new(false);
static FLAG2: AtomicBool = AtomicBool::new(false);

// Thread 1
FLAG1.store(true, Ordering::Relaxed);
if !FLAG2.load(Ordering::Relaxed) {
    // Critical section - BROKEN with Relaxed!
}

// Thread 2
FLAG2.store(true, Ordering::Relaxed);
if !FLAG1.load(Ordering::Relaxed) {
    // Critical section - both threads can enter!
}
```

On ARM, both threads can enter the critical section with `Relaxed` ordering due to reordering. This requires `SeqCst` or explicit acquire/release pairs.

### Platform Memory Models

Understanding platform differences is crucial for performance:

```rust
// Efficient on x86 (TSO model), expensive on ARM
fn increment_counter(counter: &AtomicI32) {
    counter.fetch_add(1, Ordering::SeqCst);
    // On ARM: full memory barrier (dmb ish)
    // On x86: just lock prefix (much cheaper)
}

// Better: Use Release/Acquire when sufficient
fn increment_with_visibility(counter: &AtomicI32) {
    counter.fetch_add(1, Ordering::Release);
    // On ARM: dmb ish (store barrier only)
    // On x86: lock prefix (same as SeqCst, but documents intent)
}
```

## 2. Data Races vs Race Conditions

### Formal Definition of Data Races

A **data race** occurs when:
1. Two or more threads access the same memory location
2. At least one access is a write
3. The accesses are not synchronized (no happens-before relationship)
4. At least one access is non-atomic

```rust
// Data race example (doesn't compile)
let mut x = 0;
std::thread::scope(|s| {
    s.spawn(|| x = 1);  // Write
    s.spawn(|| x = 2);  // Write - DATA RACE
});
```

The Rust compiler prevents this:
```
error[E0499]: cannot borrow `x` as mutable more than once at a time
```

### Why Rust Prevents Data Races

Rust's type system enforces exclusivity for mutable access:

**The Rule:** `&mut T` grants exclusive access; multiple `&T` allow shared access.

```rust
// Safe: Exclusive mutable access
let mut x = 0;
let r = &mut x;
*r = 42;  // Only one writer

// Safe: Shared immutable access
let x = 0;
let r1 = &x;
let r2 = &x;  // Multiple readers OK

// Prevents data races at compile time!
```

For concurrent code, Rust requires either:
- Atomics (`AtomicT`) for lock-free synchronization
- Mutexes (`Mutex<T>`) for exclusive access
- Channels for message passing

### Race Conditions (Non-Deterministic Behavior)

A **race condition** is different from a data race:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

let balance = Arc::new(AtomicI32::new(100));

// Thread 1
let b = balance.clone();
std::thread::spawn(move || {
    let current = b.load(Ordering::SeqCst);
    if current >= 50 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        b.fetch_sub(50, Ordering::SeqCst);  // RACE CONDITION!
    }
});

// Thread 2
let b = balance.clone();
std::thread::spawn(move || {
    let current = b.load(Ordering::SeqCst);
    if current >= 50 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        b.fetch_sub(50, Ordering::SeqCst);  // RACE CONDITION!
    }
});

// Balance can go negative! Not a data race, but still a bug.
```

**No data race:** All accesses are atomic with proper synchronization.
**Race condition exists:** The check-then-act pattern creates a window where both threads see sufficient balance and both withdraw, overdrawing the account.

### Preventing Both Races and Race Conditions

**For data races:** Use Rust's type system (it's automatic)

**For race conditions:** Use atomic compare-and-swap:

```rust
use std::sync::atomic::{AtomicI32, Ordering};

fn withdraw(balance: &AtomicI32, amount: i32) -> Result<(), &'static str> {
    loop {
        let current = balance.load(Ordering::Acquire);
        if current < amount {
            return Err("Insufficient funds");
        }

        let new_balance = current - amount;

        // Atomic check-and-set
        match balance.compare_exchange_weak(
            current,
            new_balance,
            Ordering::Release,
            Ordering::Acquire,
        ) {
            Ok(_) => return Ok(()),
            Err(_) => continue,  // Retry if concurrent modification
        }
    }
}
```

### Rust's Guarantees Don't Include Race Condition Prevention

**Rust guarantees:** No data races (memory safety)
**Rust does NOT guarantee:** Correct concurrent algorithms (logical correctness)

AI agents must verify:
- Atomicity of compound operations
- Lock ordering to prevent deadlocks
- Absence of TOCTOU (time-of-check-time-of-use) bugs
- Correct use of memory orderings

## 3. Atomic Types & Operations

### Atomic Types Available

```rust
use std::sync::atomic::*;

// Primitive atomics
AtomicBool      // 1 byte
AtomicI8, AtomicU8
AtomicI16, AtomicU16
AtomicI32, AtomicU32
AtomicI64, AtomicU64
AtomicIsize, AtomicUsize
AtomicPtr<T>    // Pointer-sized

// Example usage
let flag = AtomicBool::new(false);
let counter = AtomicI32::new(0);
let ptr: AtomicPtr<i32> = AtomicPtr::new(std::ptr::null_mut());
```

### Load and Store Operations

```rust
use std::sync::atomic::{AtomicI32, Ordering};

let value = AtomicI32::new(42);

// Load
let x = value.load(Ordering::Acquire);
// Store
value.store(100, Ordering::Release);

// Swap (atomic exchange)
let old = value.swap(200, Ordering::AcqRel);
```

**Key insight:** Every atomic operation requires an `Ordering` parameter, making synchronization explicit.

### Compare-and-Swap (CAS)

The fundamental primitive for lock-free algorithms:

```rust
use std::sync::atomic::{AtomicI32, Ordering};

let value = AtomicI32::new(10);

// Strong CAS: Never spuriously fails
match value.compare_exchange(
    10,      // Expected current value
    20,      // New value to set
    Ordering::Release,  // Success ordering
    Ordering::Acquire,  // Failure ordering
) {
    Ok(prev) => println!("Swapped, previous: {}", prev),
    Err(current) => println!("Failed, current: {}", current),
}

// Weak CAS: May spuriously fail (use in loops)
loop {
    let current = value.load(Ordering::Relaxed);
    let new = current + 1;

    match value.compare_exchange_weak(
        current,
        new,
        Ordering::Release,
        Ordering::Acquire,
    ) {
        Ok(_) => break,
        Err(_) => continue,  // Retry on spurious failure
    }
}
```

**When to use weak vs strong:**
- `compare_exchange_weak`: In loops (LL/SC architectures benefit)
- `compare_exchange`: Single attempts or non-looping contexts

### Fetch Operations

Atomic read-modify-write operations:

```rust
use std::sync::atomic::{AtomicI32, Ordering};

let counter = AtomicI32::new(0);

// Fetch and add
let prev = counter.fetch_add(5, Ordering::Relaxed);  // Returns old value

// Fetch and subtract
counter.fetch_sub(2, Ordering::Relaxed);

// Fetch and bitwise operations
counter.fetch_and(0xFF, Ordering::Relaxed);
counter.fetch_or(0x01, Ordering::Relaxed);
counter.fetch_xor(0x02, Ordering::Relaxed);

// Fetch max/min
counter.fetch_max(10, Ordering::Relaxed);
counter.fetch_min(5, Ordering::Relaxed);
```

## 4. Memory Ordering Explained

### Relaxed Ordering (No Synchronization)

**Use case:** Counters where only the final value matters, no coordination needed.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

let counter = Arc::new(AtomicU64::new(0));

// Multiple threads increment
for _ in 0..10 {
    let c = counter.clone();
    std::thread::spawn(move || {
        for _ in 0..1000 {
            c.fetch_add(1, Ordering::Relaxed);  // No synchronization overhead
        }
    });
}

// Relaxed is sufficient: only final count matters
```

**Guarantees:**
- Atomic modifications (no data race)
- No ordering guarantees between threads
- Can be reordered freely by compiler/CPU

**Does NOT guarantee:**
- Visibility of other operations
- Happens-before relationships

### Release Semantics (Establish Happens-Before)

**Use case:** Publishing data to other threads.

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let data = Arc::new(AtomicBool::new(false));
let ready = Arc::new(AtomicBool::new(false));

// Publisher thread
let d = data.clone();
let r = ready.clone();
std::thread::spawn(move || {
    d.store(true, Ordering::Relaxed);  // Prepare data
    r.store(true, Ordering::Release);   // Publish (release barrier)
});

// Consumer thread
let d = data.clone();
let r = ready.clone();
std::thread::spawn(move || {
    while !r.load(Ordering::Acquire) {}  // Wait (acquire barrier)
    assert!(d.load(Ordering::Relaxed));   // See published data
});
```

**Release guarantees:**
- All writes before Release are visible after corresponding Acquire
- Creates happens-before relationship

### Acquire Semantics (Synchronize with Prior Releases)

**Use case:** Consuming data published by another thread.

```rust
// Acquire load pairs with Release store
// All writes before Release store are visible after Acquire load

let flag = AtomicBool::new(false);

// Thread 1 (publisher)
expensive_initialization();
flag.store(true, Ordering::Release);

// Thread 2 (consumer)
if flag.load(Ordering::Acquire) {
    use_initialized_data();  // Safe: initialization happened-before
}
```

### AcqRel and SeqCst for Full Synchronization

**AcqRel (Acquire-Release):**
Combines both semantics for read-modify-write operations:

```rust
use std::sync::atomic::{AtomicI32, Ordering};

let value = AtomicI32::new(0);

// Fetch-add with AcqRel:
// - Acquire: See all writes before prior Release
// - Release: Make this write visible to future Acquire
value.fetch_add(1, Ordering::AcqRel);
```

**SeqCst (Sequentially Consistent):**
Strongest ordering, provides total order:

```rust
// Global sequential order across all threads
value.store(1, Ordering::SeqCst);
value.store(2, Ordering::SeqCst);

// All threads observe these in same order: 1 then 2
```

### Performance Implications of Each Ordering

**Relative costs (approximate):**

| Ordering | x86 Cost | ARM Cost | Use Case |
|----------|----------|----------|----------|
| Relaxed  | ~1x      | ~1x      | Counters, no coordination |
| Acquire  | ~1x      | ~3x      | Load with synchronization |
| Release  | ~1x      | ~3x      | Store with synchronization |
| AcqRel   | ~1x      | ~5x      | RMW with synchronization |
| SeqCst   | ~1-2x    | ~10x     | Total ordering required |

**Rule of thumb:** Use weakest ordering that provides required guarantees.

## 5. Lock-Free Patterns

### Simple Atomic Operations

```rust
// Thread-safe counter (most common pattern)
use std::sync::atomic::{AtomicUsize, Ordering};

struct Metrics {
    requests: AtomicUsize,
    errors: AtomicUsize,
}

impl Metrics {
    fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    fn get_stats(&self) -> (usize, usize) {
        (
            self.requests.load(Ordering::Relaxed),
            self.errors.load(Ordering::Relaxed),
        )
    }
}
```

### Compare-and-Swap Loops

The foundation of lock-free data structures:

```rust
use std::sync::atomic::{AtomicPtr, Ordering};

struct Node<T> {
    value: T,
    next: AtomicPtr<Node<T>>,
}

// Lock-free stack push
unsafe fn push<T>(head: &AtomicPtr<Node<T>>, node: *mut Node<T>) {
    loop {
        let current_head = head.load(Ordering::Relaxed);
        (*node).next.store(current_head, Ordering::Relaxed);

        // Try to swing head pointer
        match head.compare_exchange_weak(
            current_head,
            node,
            Ordering::Release,  // Success: publish new head
            Ordering::Relaxed,  // Failure: retry
        ) {
            Ok(_) => return,
            Err(_) => continue,
        }
    }
}
```

### Atomic References with Shared Ownership

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};

struct AtomicArc<T> {
    ptr: AtomicPtr<T>,
}

impl<T> AtomicArc<T> {
    fn new(value: Arc<T>) -> Self {
        let ptr = Arc::into_raw(value) as *mut T;
        Self {
            ptr: AtomicPtr::new(ptr),
        }
    }

    fn load(&self) -> Arc<T> {
        let ptr = self.ptr.load(Ordering::Acquire);
        unsafe {
            Arc::increment_strong_count(ptr);
            Arc::from_raw(ptr)
        }
    }

    fn store(&self, new: Arc<T>) {
        let new_ptr = Arc::into_raw(new) as *mut T;
        let old_ptr = self.ptr.swap(new_ptr, Ordering::Release);
        unsafe {
            Arc::from_raw(old_ptr);  // Decrement old ref count
        }
    }
}
```

### Building Lock-Free Data Structures

**Key challenges:**
1. **ABA problem:** Value changes from A to B and back to A, CAS succeeds incorrectly
2. **Memory reclamation:** When is it safe to free memory?
3. **Progress guarantees:** Lock-free (some thread makes progress) vs wait-free (all threads make progress)

```rust
// ABA-safe using epoch-based reclamation (conceptual)
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

struct Epoch {
    counter: AtomicUsize,
}

impl Epoch {
    fn pin(&self) -> EpochGuard {
        let epoch = self.counter.load(Ordering::Acquire);
        EpochGuard { epoch }
    }

    fn try_advance(&self) {
        // Only advance if no threads in old epochs
        self.counter.fetch_add(1, Ordering::Release);
    }
}

struct EpochGuard {
    epoch: usize,
}

// Memory is only reclaimed when epoch advances
```

### When Lock-Free is Actually Better

**Lock-free is NOT always faster:**

✅ **Use lock-free when:**
- Very low contention (< 10% conflicts)
- Readers vastly outnumber writers
- Latency critical (avoid blocking)
- Simple operations (increment, swap)

❌ **Avoid lock-free when:**
- High contention (locks amortize overhead)
- Complex operations (error-prone)
- Need mutual exclusion anyway
- Simplicity matters more than performance

**Example: When locks win:**

```rust
use std::sync::Mutex;

// Complex operation benefits from locking
struct Database {
    data: Mutex<HashMap<String, Value>>,
}

impl Database {
    fn transaction(&self) -> Result<(), Error> {
        let mut data = self.data.lock().unwrap();
        // Multiple operations under single lock acquisition
        let value = data.get("key1")?.clone();
        let new_value = expensive_computation(value)?;
        data.insert("key2".into(), new_value);
        Ok(())
    }
}

// Lock-free version would require many CAS loops and be error-prone
```

## 6. Mutual Exclusion Strategies

### Mutex<T> and Synchronization

```rust
use std::sync::{Arc, Mutex};

let data = Arc::new(Mutex::new(vec![1, 2, 3]));

// Clone Arc for each thread
let data_clone = data.clone();
std::thread::spawn(move || {
    let mut vec = data_clone.lock().unwrap();
    vec.push(4);
    // Lock automatically released when `vec` goes out of scope (RAII)
});

// Safe: Mutex ensures exclusive access
let vec = data.lock().unwrap();
println!("{:?}", *vec);
```

**Mutex guarantees:**
- Mutual exclusion (only one thread holds lock)
- Happens-before from unlock to next lock
- Interior mutability (access through `&Mutex<T>`)

### RwLock for Multiple Readers

When reads vastly outnumber writes:

```rust
use std::sync::RwLock;

let cache = RwLock::new(HashMap::new());

// Many readers can proceed concurrently
let data = cache.read().unwrap();
println!("{:?}", data.get("key"));

// Writer gets exclusive access
let mut data = cache.write().unwrap();
data.insert("key".into(), "value".into());
```

**RwLock costs:**
- Reader acquisition: Check writer count (cheap)
- Writer acquisition: Wait for all readers to finish (expensive)
- Read-biased by default (writers can starve)

**Performance considerations:**

```rust
// Good: Long read critical section
{
    let data = cache.read().unwrap();
    expensive_search(&data);  // Amortize lock overhead
}

// Bad: Short read critical section
for key in keys {
    let data = cache.read().unwrap();  // Reacquire each iteration!
    println!("{:?}", data.get(key));
}

// Better:
{
    let data = cache.read().unwrap();
    for key in keys {
        println!("{:?}", data.get(key));
    }
}
```

### Parking Lot for Advanced Strategies

The `parking_lot` crate provides more efficient primitives:

```rust
use parking_lot::{Mutex, RwLock};

// Smaller size (no poisoning)
let mutex = Mutex::new(42);
// No .unwrap() needed - never returns Err
let mut guard = mutex.lock();
*guard = 100;

// Fair RwLock (prevents writer starvation)
let rw = RwLock::new(Vec::new());
let reader = rw.read();
let writer = rw.write();  // Will acquire despite concurrent readers
```

### Deadlock Prevention

**Strategies to prevent deadlock:**

1. **Lock ordering:** Always acquire locks in same order

```rust
use std::sync::Mutex;

struct Account {
    id: u64,
    balance: Mutex<i32>,
}

fn transfer(from: &Account, to: &Account, amount: i32) {
    // Always lock lower ID first
    let (first, second) = if from.id < to.id {
        (&from.balance, &to.balance)
    } else {
        (&to.balance, &from.balance)
    };

    let mut f = first.lock().unwrap();
    let mut s = second.lock().unwrap();
    *f -= amount;
    *s += amount;
}
```

2. **Try-lock with backoff:**

```rust
use std::sync::Mutex;
use std::time::Duration;

fn try_both(lock1: &Mutex<i32>, lock2: &Mutex<i32>) {
    loop {
        let guard1 = lock1.lock().unwrap();

        // Try second lock with timeout
        if let Ok(guard2) = lock2.try_lock() {
            // Got both locks!
            break;
        }

        // Release first lock and retry
        drop(guard1);
        std::thread::sleep(Duration::from_millis(10));
    }
}
```

3. **Lock hierarchy:** Document lock levels, never acquire lower-level lock while holding higher-level

### Lock Ordering and Reentrance

**Rust mutexes are NOT reentrant:**

```rust
use std::sync::Mutex;

let mutex = Mutex::new(42);

let _guard1 = mutex.lock().unwrap();
let _guard2 = mutex.lock().unwrap();  // DEADLOCK! Same thread blocks itself
```

**Alternative: Use refcounting:**

```rust
use std::sync::{Arc, Mutex};

struct Reentrant<T> {
    data: Arc<Mutex<T>>,
    owner: AtomicUsize,
    count: AtomicUsize,
}

// Manual reentrant mutex (complex, usually avoid)
```

**Better: Restructure to avoid reentrance:**

```rust
// Instead of recursive locking:
fn process(&self) {
    let mut data = self.data.lock().unwrap();
    self.helper(&mut data);  // Pass guard
}

fn helper(&self, data: &mut GuardedData) {
    // Work with data without re-locking
}
```

## 7. Synchronization Primitives

### Barrier for Thread Synchronization

Coordinate multiple threads to reach a synchronization point:

```rust
use std::sync::{Arc, Barrier};
use std::thread;

let barrier = Arc::new(Barrier::new(5));

for i in 0..5 {
    let b = barrier.clone();
    thread::spawn(move || {
        println!("Thread {} preparing...", i);
        // Simulate work
        thread::sleep(Duration::from_millis(i * 100));

        println!("Thread {} waiting at barrier", i);
        b.wait();  // Block until all 5 threads arrive

        println!("Thread {} proceeding", i);
    });
}
```

### Condvar for Wait-Notify Patterns

Condition variables enable efficient waiting for conditions:

```rust
use std::sync::{Arc, Mutex, Condvar};

let pair = Arc::new((Mutex::new(false), Condvar::new()));

// Waiting thread
let pair_clone = pair.clone();
thread::spawn(move || {
    let (lock, cvar) = &*pair_clone;
    let mut ready = lock.lock().unwrap();

    // Wait until condition becomes true
    while !*ready {
        ready = cvar.wait(ready).unwrap();
    }

    println!("Condition satisfied!");
});

// Notifying thread
thread::sleep(Duration::from_secs(1));
let (lock, cvar) = &*pair;
let mut ready = lock.lock().unwrap();
*ready = true;
cvar.notify_one();  // Wake one waiter
// cvar.notify_all();  // Wake all waiters
```

**Key pattern: Always check condition in loop:**

```rust
// WRONG: Single check (spurious wakeups possible)
let mut ready = lock.lock().unwrap();
if !*ready {
    ready = cvar.wait(ready).unwrap();
}

// CORRECT: Loop (handles spurious wakeups)
let mut ready = lock.lock().unwrap();
while !*ready {
    ready = cvar.wait(ready).unwrap();
}
```

### Channel-Based Communication

Prefer channels over shared memory:

```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::channel();

// Sender thread
thread::spawn(move || {
    tx.send(42).unwrap();
});

// Receiver thread
let value = rx.recv().unwrap();
println!("Received: {}", value);
```

**Channel types:**
- `mpsc::channel()`: Unbounded, multiple producers, single consumer
- `mpsc::sync_channel(n)`: Bounded, blocks when full
- `crossbeam::channel`: More flexible, multiple consumers

### Once for One-Time Initialization

Thread-safe lazy initialization:

```rust
use std::sync::Once;

static INIT: Once = Once::new();
static mut CONFIG: Option<Config> = None;

fn get_config() -> &'static Config {
    INIT.call_once(|| {
        unsafe {
            CONFIG = Some(load_config());
        }
    });

    unsafe { CONFIG.as_ref().unwrap() }
}
```

**Modern alternative with `OnceCell`:**

```rust
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| load_config())
}
```

### Parking Lot Alternatives

```rust
use parking_lot::{Mutex, RwLock, Condvar, Once};

// Faster Mutex (no poisoning)
let mutex = Mutex::new(vec![1, 2, 3]);

// Fair RwLock
let rw = RwLock::new(HashMap::new());

// More efficient Condvar
let condvar = Condvar::new();
```

## 8. Arc & Shared Ownership

### Arc<T> for Reference Counting

```rust
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);

// Clone Arc (increments reference count)
let data_clone = Arc::clone(&data);

thread::spawn(move || {
    println!("{:?}", data_clone);
    // Reference count decrements when dropped
});

println!("{:?}", data);  // Still valid
```

**Arc internals:**
```rust
// Conceptual structure
struct Arc<T> {
    ptr: *const ArcInner<T>,
}

struct ArcInner<T> {
    strong: AtomicUsize,  // Strong reference count
    weak: AtomicUsize,    // Weak reference count
    data: T,
}
```

### Arc<Mutex<T>> Pattern

The most common concurrent sharing pattern:

```rust
use std::sync::{Arc, Mutex};

let shared_state = Arc::new(Mutex::new(HashMap::new()));

for i in 0..10 {
    let state = shared_state.clone();
    thread::spawn(move || {
        let mut map = state.lock().unwrap();
        map.insert(i, i * 2);
    });
}
```

### Arc Overhead and Performance

**Costs of Arc:**
1. **Atomic operations:** Increment/decrement on clone/drop
2. **Cache line contention:** Reference counts share cache line
3. **Indirection:** Extra pointer dereference
4. **Heap allocation:** Data stored on heap

**Benchmarks (approximate):**
- `Arc::clone()`: ~5-10ns (atomic increment)
- `Arc::drop()`: ~5-10ns (atomic decrement + conditional free)
- Compared to `&T`: ~0ns (zero cost)

**Optimization:**

```rust
// Expensive: Clone Arc in hot loop
for _ in 0..1000 {
    let data = arc.clone();
    process(&data);
}

// Better: Borrow Arc's contents
for _ in 0..1000 {
    process(&*arc);  // Deref to &T
}
```

### Compare to Static Lifetime

When data truly lives forever, prefer `'static`:

```rust
// Runtime overhead: Arc clone/drop
fn with_arc(data: Arc<Config>) {
    spawn(move || use_config(&data));
}

// Zero overhead: Direct reference
fn with_static(data: &'static Config) {
    spawn(move || use_config(data));
}

// Creating static data
static CONFIG: Config = Config { /* ... */ };
fn get_config() -> &'static Config {
    &CONFIG
}
```

### Weak<T> for Preventing Cycles

```rust
use std::sync::{Arc, Weak};

struct Node {
    value: i32,
    parent: Option<Weak<Node>>,  // Doesn't prevent deallocation
    children: Vec<Arc<Node>>,     // Strong references
}

// Create tree
let root = Arc::new(Node {
    value: 1,
    parent: None,
    children: vec![],
});

let child = Arc::new(Node {
    value: 2,
    parent: Some(Arc::downgrade(&root)),  // Weak reference to parent
    children: vec![],
});
```

**Weak usage:**
```rust
fn use_weak(weak: &Weak<Config>) {
    if let Some(config) = weak.upgrade() {
        // Got Arc<Config>, data still alive
        use_config(&config);
    } else {
        // Data was deallocated
        println!("Config dropped");
    }
}
```

## 9. Send & Sync Traits

### Send: Safe to Move Between Threads

```rust
pub unsafe auto trait Send {}
```

**Types that are `Send`:**
- Most types: `i32`, `String`, `Vec<T>` (if `T: Send`)
- `Arc<T>` (if `T: Send + Sync`)
- `Mutex<T>`, `RwLock<T>`

**Types that are NOT `Send`:**
- `Rc<T>`: Not thread-safe reference counting
- `*const T`, `*mut T`: Raw pointers (unsafe, no guarantees)
- `&mut T` without `T: Send`

### Sync: Safe to Share Between Threads

```rust
pub unsafe auto trait Sync {}

// T is Sync if &T is Send
```

**Types that are `Sync`:**
- Immutable types: `i32`, `&str`, `String` (if not mutated)
- `Arc<T>` (if `T: Sync`)
- `Mutex<T>`, `RwLock<T>` (provide interior mutability safely)

**Types that are NOT `Sync`:**
- `Cell<T>`, `RefCell<T>`: Interior mutability without synchronization
- `Rc<T>`: Not thread-safe

### Auto Trait Implementations

The compiler automatically implements `Send` and `Sync` when safe:

```rust
struct MyStruct {
    data: Vec<i32>,
    flag: AtomicBool,
}

// Automatically Send + Sync because:
// - Vec<i32> is Send + Sync
// - AtomicBool is Send + Sync
```

**Compiler derives:**
```rust
// Automatically generated
unsafe impl Send for MyStruct {}
unsafe impl Sync for MyStruct {}
```

### Negative Trait Bounds (!Send)

Explicitly opt out:

```rust
use std::marker::PhantomData;

struct NotSendable {
    data: i32,
    _marker: PhantomData<*const ()>,  // *const () is !Send
}

// Compiler will NOT generate Send impl
```

**Use cases:**
- FFI types tied to specific threads
- Types with platform-specific constraints
- RAII types managing thread-local resources

### Compiler Verification of Sendness

The compiler checks Send/Sync at API boundaries:

```rust
fn spawn_thread<F>(f: F)
where
    F: FnOnce() + Send + 'static,  // Explicit Send bound
{
    std::thread::spawn(f);
}

let rc = Rc::new(42);
spawn_thread(move || {
    println!("{}", rc);  // ERROR: Rc is not Send
});
```

**Verification algorithm:**
1. Function signature requires `Send`
2. Closure captures `Rc<i32>`
3. `Rc<i32>` does not implement `Send`
4. Compilation error with helpful message

## 10. Thread-Local Storage

### thread_local! Macro

```rust
use std::cell::RefCell;

thread_local! {
    static COUNTER: RefCell<u32> = RefCell::new(0);
}

fn increment() {
    COUNTER.with(|c| {
        *c.borrow_mut() += 1;
    });
}

// Each thread has its own COUNTER
thread::spawn(|| {
    increment();
    COUNTER.with(|c| {
        println!("Thread 1: {}", c.borrow());  // Prints 1
    });
});

thread::spawn(|| {
    increment();
    increment();
    COUNTER.with(|c| {
        println!("Thread 2: {}", c.borrow());  // Prints 2
    });
});
```

### Performance Characteristics

**TLS costs:**
- **Access time:** Platform-dependent (native TLS is fast)
- **Initialization:** Lazy, per-thread
- **Memory:** Per-thread copy of data

**Benchmarks:**
- Native TLS access: ~1-2ns (inlined to FS/GS segment access on x86)
- Global atomic: ~5-10ns (cross-core synchronization)

### Access Patterns

```rust
thread_local! {
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(1024));
}

fn process(data: &[u8]) -> Vec<u8> {
    BUFFER.with(|buf| {
        let mut buf = buf.borrow_mut();
        buf.clear();

        // Reuse thread-local buffer (avoid allocation)
        buf.extend_from_slice(data);
        process_in_place(&mut buf);

        buf.clone()  // Return copy
    })
}
```

### Initialization

```rust
use std::cell::RefCell;

thread_local! {
    static EXPENSIVE: RefCell<Database> = RefCell::new({
        // This runs once per thread, lazily
        println!("Initializing database for thread {:?}", std::thread::current().id());
        Database::connect()
    });
}
```

### When to Use TLS

**Good use cases:**
- Thread-local caches/buffers
- Random number generators
- Profiling/metrics per thread
- Avoiding contention on shared state

**Bad use cases:**
- Large data structures (multiplied by thread count)
- Data that must be shared
- Short-lived threads (initialization overhead)

## 11. Happens-Before Relationships (Formal Treatment)

### Establishing Happens-Before

The happens-before relation (→) is defined recursively:

1. **Sequenced-before:** Within a single thread, `A` → `B` if `A` is sequenced before `B`
2. **Synchronizes-with:** Release store synchronizes-with Acquire load of same location
3. **Transitivity:** If `A` → `B` and `B` → `C`, then `A` → `C`

```rust
// Formal example
let x = AtomicI32::new(0);
let y = AtomicI32::new(0);

// Thread 1
x.store(1, Ordering::Relaxed);        // A
y.store(1, Ordering::Release);         // B

// Thread 2
if y.load(Ordering::Acquire) == 1 {    // C
    assert_eq!(x.load(Ordering::Relaxed), 1);  // D
}

// Happens-before chain:
// A →(sequenced) B →(synchronizes-with) C →(sequenced) D
// Therefore A → D, so D sees A's write
```

### Visibility Across Threads

Without happens-before, operations can be reordered:

```rust
// EXAMPLE: No synchronization
let x = AtomicI32::new(0);
let y = AtomicI32::new(0);

// Thread 1
x.store(1, Ordering::Relaxed);  // A
y.store(1, Ordering::Relaxed);  // B

// Thread 2
let ry = y.load(Ordering::Relaxed);  // C
let rx = x.load(Ordering::Relaxed);  // D

// Possible outcomes:
// rx=0, ry=0  ✓
// rx=1, ry=0  ✓
// rx=0, ry=1  ✓ (even though A before B!)
// rx=1, ry=1  ✓
```

With Relaxed ordering, all four outcomes are legal. The CPU can reorder stores/loads.

### Compiler Reordering Within Thread

The compiler can reorder operations that don't affect single-threaded behavior:

```rust
// Source code
let x = 1;
let y = 2;
let z = x + y;

// Compiler might generate
let y = 2;
let x = 1;
let z = x + y;  // Same result, reordered
```

**Atomic operations prevent reordering:**

```rust
let x = AtomicI32::new(0);

x.store(1, Ordering::Release);
let y = expensive_computation();

// Compiler cannot move expensive_computation before store
// Release acts as a compiler barrier
```

### Platform Reordering (Weak Memory)

On ARM/POWER, hardware can reorder:

```rust
// Thread 1
x.store(1, Ordering::Relaxed);
y.store(1, Ordering::Relaxed);

// Thread 2 may observe y=1, x=0 due to store buffer
```

**Solution: Use appropriate ordering:**

```rust
// Thread 1
x.store(1, Ordering::Relaxed);
y.store(1, Ordering::Release);  // Barrier ensures x visible before y

// Thread 2
if y.load(Ordering::Acquire) == 1 {
    assert_eq!(x.load(Ordering::Relaxed), 1);  // Guaranteed
}
```

### Formal Verification of Orderings

AI agents can use model checking to verify orderings:

```rust
// Tool: loom (state space explorer)
#[cfg(loom)]
use loom::sync::atomic::{AtomicBool, Ordering};

#[test]
#[cfg(loom)]
fn check_ordering() {
    loom::model(|| {
        let flag = Arc::new(AtomicBool::new(false));
        let data = Arc::new(AtomicI32::new(0));

        let f = flag.clone();
        let d = data.clone();
        loom::thread::spawn(move || {
            d.store(42, Ordering::Relaxed);
            f.store(true, Ordering::Release);
        });

        let f = flag.clone();
        let d = data.clone();
        loom::thread::spawn(move || {
            if f.load(Ordering::Acquire) {
                let value = d.load(Ordering::Relaxed);
                assert_eq!(value, 42);  // Loom verifies all interleavings
            }
        });
    });
}
```

## 12. Cache Coherency

### False Sharing (Performance Pitfall)

When two threads access different variables on the same cache line:

```rust
// BAD: False sharing
struct Counters {
    thread1_count: AtomicUsize,  // Offset 0
    thread2_count: AtomicUsize,  // Offset 8 (same cache line!)
}

// Cache line size typically 64 bytes
// Both atomics in same line → contention
```

**Impact:**
- Thread 1 modifies `thread1_count`
- Cache line invalidated on Thread 2's core
- Thread 2 must reload entire cache line
- Ping-ponging between cores (expensive)

**Solution: Padding**

```rust
use std::sync::atomic::AtomicUsize;

#[repr(align(64))]  // Force 64-byte alignment
struct PaddedCounter {
    count: AtomicUsize,
}

struct Counters {
    thread1: PaddedCounter,  // Own cache line
    thread2: PaddedCounter,  // Different cache line
}
```

### Cache Line Size Implications

```rust
const CACHE_LINE_SIZE: usize = 64;

#[repr(C)]
struct CacheAligned<T> {
    value: T,
    _padding: [u8; CACHE_LINE_SIZE - std::mem::size_of::<T>()],
}

// Usage
struct Metrics {
    reads: CacheAligned<AtomicUsize>,
    writes: CacheAligned<AtomicUsize>,
}
```

### Padding for Performance

**When to pad:**
- High-contention atomic variables
- Per-thread counters
- Frequently modified data in different threads

**When NOT to pad:**
- Low-contention scenarios
- Memory-constrained environments
- Rarely accessed data

### NUMA Awareness

On NUMA systems, memory locality matters:

```rust
// Allocate on local NUMA node
#[cfg(target_os = "linux")]
fn allocate_local<T>(value: T) -> Box<T> {
    // Platform-specific: numa_alloc_local
    Box::new(value)
}

// Cross-NUMA access is 2-3x slower than local
```

### Contention Patterns

**Identifying contention:**

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

fn benchmark_contention() {
    let counter = Arc::new(AtomicUsize::new(0));
    let threads: Vec<_> = (0..8).map(|_| {
        let c = counter.clone();
        thread::spawn(move || {
            let start = Instant::now();
            for _ in 0..1_000_000 {
                c.fetch_add(1, Ordering::Relaxed);
            }
            start.elapsed()
        })
    }).collect();

    for (i, handle) in threads.into_iter().enumerate() {
        let elapsed = handle.join().unwrap();
        println!("Thread {}: {:?}", i, elapsed);
    }
    // High variance in times indicates contention
}
```

## 13. Async & Concurrency

### Futures and Concurrency

Futures enable concurrency without threads:

```rust
use futures::future::join_all;

async fn concurrent_requests() {
    let futures = (0..100).map(|i| async move {
        fetch_url(&format!("https://example.com/api/{}", i)).await
    });

    let results = join_all(futures).await;  // 100 concurrent requests
}
```

### Executor and Fair Scheduling

```rust
// Tokio's work-stealing scheduler
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    // Tasks distributed across 4 worker threads
    for i in 0..1000 {
        tokio::spawn(async move {
            process_task(i).await;
        });
    }
}
```

### Cancellation Safety

A future is cancellation-safe if dropping it mid-execution is safe:

```rust
// CANCELLATION-SAFE
async fn safe_operation() {
    let data = fetch_data().await;
    // If cancelled here, data is dropped cleanly
    process(data).await;
}

// NOT CANCELLATION-SAFE
async fn unsafe_operation() {
    let mut file = File::create("output.txt").await?;
    file.write_all(b"partial").await?;
    // If cancelled here, file may be corrupt
    file.write_all(b" data").await?;
}

// FIX: Use select_biased with guard
async fn safe_with_guard() {
    let _guard = FileGuard::new("output.txt");
    // Guard ensures cleanup on drop
}
```

### Drop During Async Execution

```rust
async fn demo_drop() {
    let resource = ExpensiveResource::new();

    async_operation().await;

    // Resource dropped here (Drop called)
}

impl Drop for ExpensiveResource {
    fn drop(&mut self) {
        // IMPORTANT: Drop cannot be async!
        // Must use blocking cleanup or spawn background task
        cleanup_sync();
    }
}
```

### Move Semantics in Async Blocks

```rust
let data = vec![1, 2, 3];

let future = async move {
    println!("{:?}", data);  // data moved into future
};

// println!("{:?}", data);  // ERROR: value moved

future.await;
```

## 14. Common Concurrency Bugs

### Using Mutex with Wrong Ordering

```rust
// WRONG: Relaxed with Mutex can cause issues
let flag = AtomicBool::new(false);
let data = Mutex::new(vec![]);

// Thread 1
data.lock().unwrap().push(42);
flag.store(true, Ordering::Relaxed);  // WRONG

// Thread 2
if flag.load(Ordering::Relaxed) {     // WRONG
    let vec = data.lock().unwrap();
    // May not see push(42)!
}
```

**Mutex provides synchronization, but atomic flag does not!**

**Fix: Use Acquire/Release or just rely on Mutex:**

```rust
// Better: Let Mutex handle synchronization
let ready = Mutex::new(false);
let data = Mutex::new(vec![]);

// Thread 1
data.lock().unwrap().push(42);
*ready.lock().unwrap() = true;  // Mutex provides synchronization

// Thread 2
if *ready.lock().unwrap() {
    let vec = data.lock().unwrap();
    // Guaranteed to see push(42)
}
```

### ABA Problem in CAS Loops

```rust
// VULNERABLE to ABA
let head = AtomicPtr::new(ptr_a);

// Thread 1
let current = head.load(Ordering::Acquire);  // Sees A
// ... context switch ...
// Thread 2 changes A→B→A
// ... context switch ...
head.compare_exchange(current, new_ptr, ...);  // Succeeds but wrong!
```

**Solution: Tagged pointers or epoch-based reclamation:**

```rust
// Tagged pointer (uses upper bits)
struct TaggedPtr<T> {
    ptr: usize,  // Lower bits: pointer, upper bits: tag
}

impl<T> TaggedPtr<T> {
    fn new(ptr: *mut T, tag: usize) -> Self {
        let addr = ptr as usize;
        Self {
            ptr: addr | (tag << 48),  // Tag in upper 16 bits
        }
    }

    fn compare_exchange(&self, current: TaggedPtr<T>, new: TaggedPtr<T>) {
        // Tag changes prevent ABA
    }
}
```

### Deadlocks and How to Avoid

See section 6 (Deadlock Prevention) for detailed strategies.

### Use-After-Free in Concurrent Code

Rust's type system prevents most use-after-free, but unsafe code can still have issues:

```rust
// UNSAFE: Incorrect lifetime management
unsafe fn bad_concurrent_access() {
    let data = Box::new(42);
    let ptr = &*data as *const i32;

    thread::spawn(move || {
        // data moved and dropped in this thread
    });

    println!("{}", *ptr);  // Use-after-free!
}
```

**Fix: Use Arc for shared ownership:**

```rust
fn safe_concurrent_access() {
    let data = Arc::new(42);
    let data_clone = data.clone();

    thread::spawn(move || {
        println!("{}", data_clone);  // Safe: Arc keeps data alive
    });

    println!("{}", data);  // Safe
}
```

## 15. Testing Concurrent Code

### Deterministic Testing

```rust
use std::sync::{Arc, Mutex};

#[test]
fn test_concurrent_counter() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let c = counter.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                *c.lock().unwrap() += 1;
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*counter.lock().unwrap(), 10_000);
}
```

### Loom for State Space Exploration

Loom exhaustively explores all possible thread interleavings:

```rust
#[cfg(loom)]
use loom::sync::atomic::{AtomicUsize, Ordering};
#[cfg(loom)]
use loom::sync::Arc;
#[cfg(loom)]
use loom::thread;

#[test]
#[cfg(loom)]
fn test_all_interleavings() {
    loom::model(|| {
        let v1 = Arc::new(AtomicUsize::new(0));
        let v2 = Arc::new(AtomicUsize::new(0));

        let t1 = {
            let v1 = v1.clone();
            let v2 = v2.clone();
            thread::spawn(move || {
                v1.store(1, Ordering::Release);
                v2.load(Ordering::Acquire)
            })
        };

        let t2 = {
            let v1 = v1.clone();
            let v2 = v2.clone();
            thread::spawn(move || {
                v2.store(1, Ordering::Release);
                v1.load(Ordering::Acquire)
            })
        };

        let a = t1.join().unwrap();
        let b = t2.join().unwrap();

        // Loom verifies: !(a == 0 && b == 0) for all interleavings
        assert!(a == 1 || b == 1);
    });
}
```

### ThreadSanitizer for Race Detection

```bash
# Run with ThreadSanitizer
RUSTFLAGS="-Z sanitizer=thread" cargo +nightly test

# Output shows data races:
# WARNING: ThreadSanitizer: data race
#   Write of size 4 at 0x7b0400000000 by thread T1:
#     #0 increment src/main.rs:42
#   Previous read of size 4 at 0x7b0400000000 by thread T2:
#     #0 read_value src/main.rs:38
```

### Property-Based Testing Concurrently

```rust
use proptest::prelude::*;
use std::sync::{Arc, Mutex};

proptest! {
    #[test]
    fn concurrent_insert_preserve_count(values in prop::collection::vec(0i32..100, 0..1000)) {
        let set = Arc::new(Mutex::new(HashSet::new()));
        let mut handles = vec![];

        for value in values.clone() {
            let s = set.clone();
            handles.push(thread::spawn(move || {
                s.lock().unwrap().insert(value);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let unique_count = values.iter().collect::<HashSet<_>>().len();
        assert_eq!(set.lock().unwrap().len(), unique_count);
    }
}
```

### Stress Testing Patterns

```rust
fn stress_test_lock_free_stack() {
    let stack = Arc::new(LockFreeStack::new());
    let iterations = 1_000_000;
    let threads = 8;

    let handles: Vec<_> = (0..threads).map(|i| {
        let s = stack.clone();
        thread::spawn(move || {
            for j in 0..iterations {
                s.push(i * iterations + j);
            }
            for _ in 0..iterations {
                s.pop();
            }
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final state
    assert!(stack.is_empty());
}
```

## 16. AI Agent Concurrency Analysis

### Verifying Send/Sync Bounds

AI agents should check:

1. **Does type need Send?**
   ```rust
   // Check: Will values cross thread boundaries?
   fn analyze_send<T>() {
       if contains_thread_spawn::<T>() || contains_channel_send::<T>() {
           assert_impl!(T: Send);
       }
   }
   ```

2. **Does type need Sync?**
   ```rust
   // Check: Will references be shared?
   fn analyze_sync<T>() {
       if contains_arc_wrapping::<T>() {
           assert_impl!(T: Sync);
       }
   }
   ```

### Checking Synchronization

Look for these patterns:

```rust
// ✓ SYNCHRONIZED
let data = Mutex::new(vec![]);
data.lock().unwrap().push(42);

// ✗ NOT SYNCHRONIZED
static mut DATA: Vec<i32> = Vec::new();
unsafe { DATA.push(42); }  // Data race!

// ✓ SYNCHRONIZED (atomic)
let counter = AtomicUsize::new(0);
counter.fetch_add(1, Ordering::Relaxed);
```

### Detecting Potential Race Conditions

Pattern matching for common bugs:

```rust
// PATTERN: Check-then-act (race condition)
if map.contains_key(&key) {
    let value = map.get(&key).unwrap();  // Key might be removed!
}

// FIX: Atomic operation
if let Some(value) = map.get(&key) {
    // Use value
}
```

### Understanding Happens-Before

Verify synchronization chains:

```rust
// Analyze this code:
data.store(42, Ordering::Relaxed);  // A
flag.store(true, Ordering::Release); // B (happens-after A)

if flag.load(Ordering::Acquire) {    // C (synchronizes-with B)
    let x = data.load(Ordering::Relaxed);  // D (happens-after C)
}

// Conclusion: A → B →(sync) C → D
// Therefore: D is guaranteed to see A's write
```

### Analyzing Lock Contention

AI agents can identify hotspots:

```rust
// HIGH CONTENTION: Global lock
static GLOBAL: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());

fn high_contention() {
    // All threads contend for same lock
    GLOBAL.lock().unwrap().insert(key, value);
}

// LOWER CONTENTION: Sharded locks
const SHARDS: usize = 16;
static SHARDED: [Mutex<HashMap<String, Value>>; SHARDS] = /* ... */;

fn lower_contention(key: &str) {
    let shard = hash(key) % SHARDS;
    SHARDED[shard].lock().unwrap().insert(key, value);
}
```

## Conclusion

Understanding Rust's memory model and concurrent access patterns requires deep knowledge of:

1. **Memory ordering semantics** and their performance implications
2. **Happens-before relationships** that establish visibility guarantees
3. **Atomic operations** and when to use each ordering
4. **Synchronization primitives** and their correct usage
5. **Send/Sync traits** and the safety guarantees they provide
6. **Common concurrency bugs** and how to prevent them
7. **Testing strategies** for verifying concurrent code

AI agents working with concurrent Rust code must:
- Verify Send/Sync bounds are correct
- Check that synchronization establishes proper happens-before relationships
- Identify race conditions (not just data races)
- Understand memory ordering choices and their correctness
- Use tools like Loom and ThreadSanitizer for verification

The combination of Rust's type system (preventing data races) and understanding of memory ordering (ensuring correct synchronization) enables building correct, high-performance concurrent systems. However, the responsibility for logical correctness—preventing race conditions, deadlocks, and ensuring proper algorithm design—still rests with the developer (or AI agent).

**Key Takeaways for AI Agents:**
1. Data race freedom is guaranteed by Rust's type system
2. Race condition freedom requires algorithmic correctness
3. Memory ordering is about visibility and performance
4. Always use weakest ordering that provides required guarantees
5. Test concurrent code with Loom and ThreadSanitizer
6. Document synchronization invariants clearly
7. Prefer message passing over shared memory when possible
8. Use atomic operations correctly or not at all
