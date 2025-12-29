# Polling Loop

## Context

You're building or understanding an async executor in Rust. You have futures that need to be driven to completion, wakers that signal readiness, and a thread that needs to coordinate it all. You understand the individual pieces—`Future::poll()`, `Poll::Ready`, `Poll::Pending`, wakers—but you're unclear about how they fit together into a working executor.

You need to understand the core event loop that powers all async execution in Rust, from simple test executors to production runtimes like tokio. This is the "main loop" of async programming.

## Problem

**How do you create the central loop that repeatedly polls a future, responding to waker signals, until the future completes?**

The challenges:
- **When to poll**: Poll too often and you waste CPU; poll too rarely and you add latency
- **How to wait**: Busy-waiting wastes CPU; sleeping with arbitrary durations adds latency
- **Waker integration**: The loop must respond to waker signals, not arbitrary timers
- **Completion detection**: Must recognize when the future is done and extract the value
- **State management**: Must maintain the future's state across poll invocations
- **Pinning**: Must keep the future pinned while polling

Without this pattern, you might:
- Busy-wait in a loop (100% CPU usage)
- Sleep with arbitrary delays (high latency)
- Poll after completion (logic error)
- Fail to respond to waker signals (deadlock)

## Forces

- **Efficiency**: Only poll when the future can make progress
- **Responsiveness**: Poll promptly when wakers signal readiness
- **Termination**: Stop polling once the future completes
- **Correctness**: Follow the Future protocol exactly
- **Simplicity**: The loop should be understandable and maintainable
- **Waker Protocol**: Integrate with the waker mechanism properly
- **Pinning Safety**: Maintain pinning guarantees across iterations

## Solution

**Create a loop that polls the future, parks the thread on `Poll::Pending`, and unparks on waker signals, continuing until `Poll::Ready`.**

The complete pattern from `block-on`:

```rust
use waker_fn::waker_fn;
use futures_lite::pin;
use crossbeam::sync::Parker;
use std::future::Future;
use std::task::{Context, Poll};

pub fn block_on<F: Future>(future: F) -> F::Output {
    let parker = Parker::new();
    let unparker = parker.unparker().clone();
    let waker = waker_fn(move || unparker.unpark());
    let mut context = Context::from_waker(&waker);

    pin!(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => parker.park(),
        }
    }
}
```

**The Loop Structure**:

```rust
loop {
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => return value,
        Poll::Pending => parker.park(),
    }
}
```

**Breaking It Down**:

### 1. The Loop Condition

```rust
loop {  // Infinite loop
```

**Why infinite?** Because we don't know how many polls are needed. The future itself signals completion via `Poll::Ready`. The loop exits via `return`, not a condition.

### 2. The Poll Invocation

```rust
future.as_mut().poll(&mut context)
```

**Why `as_mut()`?** The future is `Pin<&mut F>`. We need `Pin<&mut F>` for polling, so we get a mutable reference to the pinned future.

**What happens inside poll?**
- Future examines its internal state
- If work is complete: returns `Poll::Ready(value)`
- If work is pending: stores the waker from `context` and returns `Poll::Pending`

### 3. The Ready Branch

```rust
Poll::Ready(value) => return value,
```

**Immediate return**: As soon as the future completes, extract the value and exit the function. No more polling needed.

### 4. The Pending Branch

```rust
Poll::Pending => parker.park(),
```

**Park the thread**: Put the current thread to sleep. It will wake when `unparker.unpark()` is called.

**Who calls unpark?** The waker, which is called by the future when it becomes ready:
```rust
// Inside the future (e.g., spawn_blocking)
if let Some(waker) = maybe_waker {
    waker.wake();  // This calls unparker.unpark()
}
```

### 5. Loop Continuation

After unparking, the loop continues:
```
Poll (attempt 1) → Pending → Park
  ↓
Waker called → Unpark
  ↓
Poll (attempt 2) → Ready → Return
```

**Complete Flow Diagram**:

```
Start
  │
  ├─→ Create Parker, Unparker, Waker, Context
  │
  ├─→ Pin the future
  │
  └─→ Loop:
       │
       ├─→ Poll future with context
       │    │
       │    ├─→ Poll::Ready(value)
       │    │    └─→ Return value (exit function)
       │    │
       │    └─→ Poll::Pending
       │         │
       │         ├─→ Future stored waker
       │         │
       │         └─→ Park thread (sleep)
       │              │
       │              └─→ [Wait for waker.wake()]
       │                   │
       │                   └─→ Unpark (wake thread)
       │                        │
       │                        └─→ Continue loop (poll again)
```

**Example Execution** (from tests):

```rust
use std::time::Duration;
use async_std::task::sleep;

block_on(async {
    println!("Starting");
    sleep(Duration::from_millis(100)).await;
    println!("Done");
    42
})
```

**Execution Timeline**:
```
t=0ms:   Poll 1: Sleep future returns Poll::Pending, stores waker
         Thread parks

t=100ms: Timer expires, calls waker.wake()
         Thread unparks

t=100ms: Poll 2: Sleep future returns Poll::Ready(())
         Continue to next statement

t=100ms: Poll 3: async block returns Poll::Ready(42)
         block_on returns 42
```

**Multiple Pending Cycles**:

```rust
block_on(async {
    let mut count = 0;
    loop {
        sleep(Duration::from_millis(10)).await;
        count += 1;
        if count >= 5 {
            break count;
        }
    }
})
```

**Execution**:
- Poll 1: Sleep returns Pending → Park
- Wake at 10ms, Poll 2: Sleep returns Ready, loop continues
- Poll 3: Sleep returns Pending → Park
- Wake at 20ms, Poll 4: Sleep returns Ready, loop continues
- ... (repeats 5 times)
- Final poll: async block returns Ready(5)

**Why This Works**:

1. **No Busy-Waiting**: The thread sleeps (parked) when waiting, not spinning
2. **Prompt Waking**: Waker immediately unparks the thread when work is available
3. **Correct Termination**: Loop exits exactly when the future completes
4. **Minimal Overhead**: Each poll is purposeful (triggered by readiness)

**Alternative Approaches** (and why they fail):

**❌ Busy-Wait**:
```rust
// WRONG: Wastes CPU
loop {
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => return value,
        Poll::Pending => {
            // No parking - loop immediately polls again
        }
    }
}
// 100% CPU usage, millions of wasted polls
```

**❌ Fixed Sleep**:
```rust
// WRONG: High latency
loop {
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => return value,
        Poll::Pending => {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}
// Adds 0-100ms latency to every operation
```

**✅ Park/Unpark** (correct):
```rust
// RIGHT: Efficient and responsive
loop {
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => return value,
        Poll::Pending => parker.park(),  // Wakes immediately on signal
    }
}
```

## Resulting Context

**Benefits**:

1. **Efficient**: Thread sleeps when no work available (0% CPU when idle)
2. **Responsive**: Wakes immediately when work becomes available (<1μs latency)
3. **Correct**: Follows the Future protocol exactly
4. **Simple**: ~10 lines of code for the core loop
5. **Universal**: Works with any future that follows the protocol
6. **Composable**: Handles nested futures, combinators, etc.

**Characteristics**:

1. **Synchronous Blocking**: `block_on` itself blocks the calling thread
   ```rust
   // This blocks the thread until the future completes
   let result = block_on(async_operation());
   ```

2. **Single Future**: Executes one future at a time
   ```rust
   // These run sequentially, not in parallel
   block_on(async {
       task_a().await;
       task_b().await;
   });
   ```

3. **Deterministic**: No scheduling decisions, no task prioritization

**Advanced Variations**:

**Multi-Threaded Executor** (conceptual):
```rust
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where F: Future + Send + 'static
{
    let task = Task::new(future);
    GLOBAL_QUEUE.push(task.clone());
    WORKER_THREADS.notify();  // Wake a worker
    JoinHandle { task }
}

// Each worker thread runs:
fn worker_loop() {
    loop {
        let task = GLOBAL_QUEUE.pop_or_park();

        loop {
            match task.poll() {
                Poll::Ready(value) => {
                    task.complete(value);
                    break;  // Get next task
                }
                Poll::Pending => break,  // Task is waiting
            }
        }
    }
}
```

**With Timeout**:
```rust
pub fn block_on_timeout<F: Future>(
    future: F,
    timeout: Duration
) -> Result<F::Output, Timeout> {
    let parker = Parker::new();
    let unparker = parker.unparker().clone();
    let waker = waker_fn(move || unparker.unpark());
    let mut context = Context::from_waker(&waker);

    pin!(future);
    let deadline = Instant::now() + timeout;

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return Ok(value),
            Poll::Pending => {
                let now = Instant::now();
                if now >= deadline {
                    return Err(Timeout);
                }
                parker.park_timeout(deadline - now);
            }
        }
    }
}
```

**Integration with I/O Reactor** (conceptual):
```rust
loop {
    // Poll all ready tasks
    for task in ready_tasks {
        match task.poll(&mut context) {
            Poll::Ready(value) => task.complete(value),
            Poll::Pending => { /* Task registered its interest */ }
        }
    }

    // Wait for I/O events or wakers
    let events = epoll.wait(timeout);

    // Wake tasks with ready I/O
    for event in events {
        if let Some(waker) = io_registry.get_waker(event) {
            waker.wake();
        }
    }
}
```

## Related Patterns

- **Simple Executor**: The complete pattern this loop is part of
- **Waker Mechanism**: What triggers loop iterations
- **Custom Future**: What the loop polls
- **Spawn Blocking**: Futures that wake from background threads
- **Thread Pool for Blocking**: Advanced executor architecture

## Known Uses

1. **futures::executor::block_on**: Standard implementation
   ```rust
   use futures::executor::block_on;

   block_on(async {
       // Work
   })
   ```

2. **tokio::runtime::Runtime::block_on**: Bridge to sync code
   ```rust
   let rt = tokio::runtime::Runtime::new()?;
   rt.block_on(async {
       // Async work
   })
   ```

3. **async-std::task::block_on**: Similar implementation
   ```rust
   async_std::task::block_on(async {
       // Async work
   })
   ```

4. **Test Harnesses**: Every async test fundamentally uses this loop
   ```rust
   #[tokio::test]
   async fn my_test() {
       // Tokio's test macro uses block_on internally
       assert_eq!(my_async_fn().await, expected);
   }
   ```

5. **Main Functions**: Async main is syntactic sugar for block_on
   ```rust
   #[tokio::main]
   async fn main() {
       // Expands to:
       // fn main() {
       //     tokio::runtime::Runtime::new()
       //         .unwrap()
       //         .block_on(async { /* body */ })
       // }
   }
   ```

**Production Executor Loop** (tokio, simplified):
```rust
// Worker thread event loop
loop {
    // Run all tasks in local queue
    while let Some(task) = local_queue.pop() {
        task.poll();
    }

    // Steal work from other threads
    if let Some(task) = global_queue.steal() {
        local_queue.push(task);
        continue;
    }

    // Park until new work arrives
    park_with_timeout(timeout);
}
```

**Performance Characteristics**:
- **Poll Overhead**: ~100ns per poll (no-op future)
- **Wake Latency**: ~1μs (OS thread unpark time)
- **Memory**: O(1) for the loop itself, O(n) for nested futures
- **CPU Usage**: 0% when parked, 100% when polling (brief)

The polling loop is the heart of every async executor in Rust. Understanding this pattern is essential for debugging, optimizing, and reasoning about async code behavior.
