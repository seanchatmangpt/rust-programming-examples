# Simple Executor

## Context

You understand futures and async/await syntax in Rust, but you're unclear about what actually drives execution. You know that `async fn` creates a future, but who polls it? You've used runtime libraries like tokio and async-std, but you want to understand the core mechanism without the complexity of work-stealing schedulers and multi-threaded executors.

You need to run a future to completion, perhaps for testing, for a simple CLI tool, or to integrate async code into a synchronous context. You don't need sophisticated scheduling—just the minimum machinery to make async work.

## Problem

**How do you execute a future to completion with the minimal necessary machinery, understanding the fundamental polling protocol?**

The challenges:
- **Futures are lazy**: They don't execute until polled
- **Polling requires a waker**: Futures need a way to signal readiness
- **Parking requires a mechanism**: The executor must sleep when futures are pending
- **Pinning is mandatory**: Futures must be pinned before polling
- **Context construction**: You need to create a valid `Context` with a waker
- **Loop termination**: You must poll until `Poll::Ready`, not before or after

Without understanding these pieces, you might:
- Try to poll futures without pinning (compiler error)
- Create a context without a valid waker (runtime error or deadlock)
- Busy-wait in a loop (wasting CPU)
- Exit the loop too early (incomplete execution)

## Forces

- **Simplicity**: Minimize complexity to understand the core mechanism
- **Correctness**: Must follow the Future polling protocol exactly
- **Efficiency**: Should not busy-wait or waste CPU cycles
- **Waker Protocol**: Must create and manage wakers properly
- **Pinning Safety**: Must handle pinning correctly to prevent undefined behavior
- **Single-Threaded**: Keep it simple—no thread pools or work stealing
- **Testing**: Should be sufficient for tests and simple use cases

## Solution

**Create a minimal executor that polls a future in a loop, parking the thread when the future is pending and unparking when woken.**

The complete implementation from `block-on`:

```rust
use waker_fn::waker_fn;
use futures_lite::pin;
use crossbeam::sync::Parker;
use std::future::Future;
use std::task::{Context, Poll};

pub fn block_on<F: Future>(future: F) -> F::Output {
    // 1. Create parking machinery
    let parker = Parker::new();
    let unparker = parker.unparker().clone();

    // 2. Create waker that unparks the thread
    let waker = waker_fn(move || unparker.unpark());

    // 3. Create polling context with the waker
    let mut context = Context::from_waker(&waker);

    // 4. Pin the future (required for polling)
    pin!(future);

    // 5. Poll loop
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => parker.park(),
        }
    }
}
```

**The Five Essential Components**:

### 1. Parker/Unparker (Thread Parking)

```rust
let parker = Parker::new();
let unparker = parker.unparker().clone();
```

**Purpose**: Efficiently sleep the thread when waiting for futures.
- `parker.park()`: Puts the current thread to sleep
- `unparker.unpark()`: Wakes the parked thread
- More efficient than spin-waiting

### 2. Waker Creation

```rust
let waker = waker_fn(move || unparker.unpark());
```

**Purpose**: Create a waker that futures can call to wake the executor.
- The waker is a closure: when called, it unparks the thread
- `waker_fn` from the `waker-fn` crate simplifies waker creation
- The unparker is moved into the closure

### 3. Context Construction

```rust
let mut context = Context::from_waker(&waker);
```

**Purpose**: Package the waker for the future's poll method.
- `Context` is what gets passed to `poll()`
- It provides access to the waker via `cx.waker()`
- Futures clone this waker to call later

### 4. Future Pinning

```rust
pin!(future);
```

**Purpose**: Pin the future to prevent it from moving in memory.
- Futures may be self-referential (fields referencing other fields)
- Moving a self-referential struct invalidates pointers
- `Pin` guarantees the future won't move
- `pin!` macro pins to the stack

### 5. The Poll Loop

```rust
loop {
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => return value,
        Poll::Pending => parker.park(),
    }
}
```

**Purpose**: Drive the future to completion.
- **Poll the future**: `future.as_mut().poll(&mut context)`
- **If ready**: Return the value (done)
- **If pending**: Park the thread (sleep until woken)
- **Wake triggers**: Loop continues, polls again

**The Protocol**:
1. Poll the future
2. If `Poll::Ready(value)`: Execution complete, return value
3. If `Poll::Pending`:
   - Future has stored the waker
   - Park the thread
   - When future becomes ready, it calls `waker.wake()`
   - Thread unparks, loop continues
   - Poll again

**Example Usage** (from the tests):

```rust
// Simple future
assert_eq!(block_on(std::future::ready(42)), 42);

// Complex async computation
use async_std::task::{spawn, sleep};
use futures_lite::FutureExt;
use std::time::Duration;

assert_eq!(
    block_on({
        let one_sec = async {
            sleep(Duration::from_secs(1)).await;
            43
        };
        let half_sec = async {
            sleep(Duration::from_millis(500)).await;
            44
        };
        spawn(one_sec.race(half_sec))
    }),
    44
);
```

**Why This Works**:
1. The `sleep` futures store the waker
2. Timers wake the futures when deadlines expire
3. The waker calls `unparker.unpark()`
4. The poll loop continues and polls the race
5. The faster future (500ms) completes first
6. `Poll::Ready(44)` is returned

**Pinning Details**:

```rust
// The pin! macro expands roughly to:
let mut future = future;
let mut future = unsafe { Pin::new_unchecked(&mut future) };
```

This pins the future to the stack for the duration of `block_on`. The future cannot move because:
- It's behind a `Pin` reference
- The `Pin` API prevents getting a `&mut` to the inner value
- Only `Pin<&mut Self>` can be obtained, which maintains the pinning guarantee

## Resulting Context

**Benefits**:

1. **Simplicity**: ~20 lines of code to understand the entire execution model
2. **Correct**: Follows the Future protocol exactly
3. **Efficient**: No busy-waiting, thread sleeps when no work available
4. **Composable**: Works with any standard future
5. **Transparent**: Shows exactly what tokio/async-std do under the hood
6. **Testable**: Perfect for test code and simple CLI tools

**Limitations**:

1. **Single-Threaded**: Can only execute one future at a time
   ```rust
   // This works but runs sequentially
   block_on(async {
       task_a().await;
       task_b().await;
   });
   ```

2. **No Parallelism**: Can't take advantage of multiple cores
   ```rust
   // This spawns async tasks, but block_on runs on one thread
   block_on(async {
       let a = spawn(task_a());
       let b = spawn(task_b());
       // a and b might run in parallel in their spawned runtime,
       // but block_on itself is single-threaded
   });
   ```

3. **No Task Scheduling**: No prioritization or fairness guarantees

4. **No I/O Reactor**: Can't drive I/O operations itself (relies on spawned tasks using async-std/tokio)

5. **Stack Pinning**: Future is pinned to `block_on`'s stack frame, limiting flexibility

**Comparison with Production Executors**:

| Feature | block_on | tokio | async-std |
|---------|----------|-------|-----------|
| Threads | 1 | Thread pool | Thread pool |
| Work stealing | No | Yes | Yes |
| I/O reactor | No | Yes (epoll/kqueue) | Yes |
| Timer wheel | No | Yes | Yes |
| Task spawning | No | Yes | Yes |
| Complexity | ~20 LOC | ~10,000 LOC | ~5,000 LOC |

**When to Use This Pattern**:
- ✅ Testing async code
- ✅ Simple CLI tools
- ✅ Integrating async code in sync context
- ✅ Learning/understanding the async model
- ✅ Prototyping
- ✅ Single-future execution

**When to Use Production Executors**:
- ❌ Server applications (need parallelism)
- ❌ I/O-heavy workloads (need reactor)
- ❌ Multiple concurrent tasks (need scheduling)
- ❌ Performance-critical code (need work stealing)

**Extensions**:

To make this production-ready, you'd add:
1. **Thread pool**: Multiple threads polling different futures
2. **Work stealing**: Balance load across threads
3. **I/O reactor**: Integrate with OS event notification (epoll, kqueue, IOCP)
4. **Timer wheel**: Efficient timer management
5. **Task queue**: Spawn new futures dynamically
6. **Local task queue**: Thread-local tasks for cache efficiency

## Related Patterns

- **Waker Mechanism**: The protocol this executor uses
- **Polling Loop**: The core of this pattern
- **Custom Future**: What this executor consumes
- **Spawn Blocking**: Integrates with this executor
- **Thread Pool for Blocking**: Advanced scheduling

## Known Uses

1. **futures::executor::block_on**: Standard library implementation
   ```rust
   use futures::executor::block_on;

   let result = block_on(async {
       // async work
   });
   ```

2. **Test harnesses**: Many async tests use simple executors
   ```rust
   #[test]
   fn test_async_code() {
       block_on(async {
           assert_eq!(my_async_fn().await, expected);
       });
   }
   ```

3. **tokio::runtime::Runtime::block_on**: Bridge to sync code
   ```rust
   let rt = tokio::runtime::Runtime::new()?;
   let result = rt.block_on(async {
       // Use tokio features
   });
   ```

4. **async-std::task::block_on**: Similar bridge
   ```rust
   async_std::task::block_on(async {
       // Use async-std features
   });
   ```

**Example from real code**:

```rust
fn main() {
    let result = block_on(async {
        let data = fetch_data().await?;
        process(data).await?;
        Ok(())
    });

    match result {
        Ok(()) => println!("Success"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

**Performance Characteristics**:
- **Overhead**: Minimal (~100ns per poll on modern hardware)
- **Memory**: Single stack frame + future size
- **Latency**: Depends on OS thread parking latency (~1μs typical)
- **Throughput**: Single task at a time (no parallelism)

This pattern is the foundation for understanding all async executors in Rust. Every production executor is fundamentally this pattern plus optimizations and features.
