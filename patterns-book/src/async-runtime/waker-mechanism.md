# Waker Mechanism

## Context

You're working with Rust's async runtime. You understand that futures don't execute until polled, but you're unclear about what happens when a future returns `Poll::Pending`. How does the executor know when to poll again? Random polling would waste CPU; never polling would hang forever.

You're implementing a custom future or executor and need to understand the contract between them. The executor polls, the future says "not ready yet," but somehow the executor knows exactly when to try again.

## Problem

**How do futures signal to executors that they're ready to make progress, enabling efficient cooperative scheduling without busy-waiting?**

The challenge is coordination:
- **Futures** don't know when their underlying operations (I/O, timers, threads) complete
- **Executors** don't know the internal state of futures
- **Polling too often** wastes CPU cycles
- **Polling too rarely** causes high latency
- **Never polling again** creates deadlocks

You need a mechanism where:
- Futures can register interest in being polled again
- External events (I/O completion, timer expiry) can trigger polling
- No polling happens when no progress is possible
- No deadlocks occur when a future becomes ready

## Forces

- **Efficiency**: Minimize unnecessary polling to save CPU
- **Responsiveness**: Wake futures promptly when they can make progress
- **Decoupling**: Futures and executors shouldn't know about each other's internals
- **Thread Safety**: Wakers must work across thread boundaries
- **Clone Semantics**: Multiple parts of the system need to hold wakers
- **Wake Semantics**: Calling `wake()` should be cheap and thread-safe
- **No Lost Wakeups**: Every readiness event must trigger a poll
- **No Spurious Wakeups**: Waking before readiness wastes CPU (though safe)

## Solution

**Use the `Waker` type as a callback handle that futures store and call when they become ready to make progress.**

The waker mechanism from `block-on` shows the complete protocol:

**Executor Side** (creating the waker):

```rust
use waker_fn::waker_fn;
use crossbeam::sync::Parker;
use std::task::Context;

let parker = Parker::new();
let unparker = parker.unparker().clone();

// Create a waker that unparks the executor thread
let waker = waker_fn(move || unparker.unpark());
let mut context = Context::from_waker(&waker);
```

**Key Insight**: The waker is a closure that, when called, unparks the executor thread. This bridges the gap between "future is ready" and "executor should poll."

**The Waker Contract**:

1. **Executor creates waker** wrapping its scheduling mechanism (here, an unparker)
2. **Executor passes waker** to future via `Context` parameter in `poll()`
3. **Future stores waker** if returning `Poll::Pending`
4. **External event occurs** (I/O ready, timer expires, thread completes)
5. **Future calls `waker.wake()`** to notify executor
6. **Executor polls future** in response to wake call

**Future Side** (from `spawn-blocking`):

```rust
use std::task::{Context, Poll};

impl<T: Send> Future for SpawnBlocking<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut guard = self.0.lock().unwrap();

        if let Some(value) = guard.value.take() {
            return Poll::Ready(value);
        }

        // Store waker for later use
        guard.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}
```

**Critical Pattern**: The waker is cloned from `cx.waker()` and stored. The future doesn't know what the waker does—it just knows that calling `wake()` will make the executor poll again.

**Producer Side** (waking the executor):

```rust
std::thread::spawn({
    let inner = inner.clone();
    move || {
        let value = closure();

        let maybe_waker = {
            let mut guard = inner.lock().unwrap();
            guard.value = Some(value);
            guard.waker.take()  // Get the stored waker
        };

        // Call wake() to notify the executor
        if let Some(waker) = maybe_waker {
            waker.wake();
        }
    }
});
```

**Wake Semantics**:
- `waker.wake()` consumes the waker (moves it)
- `waker.wake_by_ref()` can be called multiple times
- `waker.clone()` creates another handle to the same underlying waker

**Complete Executor Loop** (from `block-on`):

```rust
pub fn block_on<F: Future>(future: F) -> F::Output {
    let parker = Parker::new();
    let unparker = parker.unparker().clone();
    let waker = waker_fn(move || unparker.unpark());
    let mut context = Context::from_waker(&waker);

    pin!(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => parker.park(),  // Wait for wake
        }
    }
}
```

**The Park/Unpark Pattern**:
1. **Poll the future** with the waker-equipped context
2. **If ready**: Return the value
3. **If pending**: Park the thread (sleep until unparked)
4. **When waker.wake() is called**: Thread is unparked
5. **Loop continues**: Poll again

**Efficiency Guarantee**: The executor thread only wakes (and polls) when explicitly notified via the waker. No busy-waiting, no wasted polls.

## Resulting Context

**Benefits**:

1. **Efficient Scheduling**: Executors only poll when progress is possible
2. **Decoupled Design**: Futures and executors communicate through a clean interface
3. **Thread-Safe**: Wakers implement `Send + Sync`, working across threads
4. **Composable**: Nested futures propagate wakers automatically
5. **Flexible**: Same waker mechanism works for I/O, timers, channels, threads
6. **No Polling Storm**: Each readiness event triggers exactly one poll (typically)

**Trade-offs**:

1. **Clone Overhead**: Wakers must be cloned when stored, though they're typically cheap (Arc-based)
2. **Lost Waker Risk**: Forgetting to store the waker causes deadlocks (future never wakes)
3. **Spurious Wakes**: Safe to wake extra times, but wastes a poll
4. **Wake After Ready**: Waking a completed future is harmless but wasteful

**Common Mistakes**:

1. **Not storing the waker**:
```rust
// WRONG: Returns Pending without storing waker
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
    if self.is_ready() {
        Poll::Ready(self.value)
    } else {
        Poll::Pending  // BUG: Never wakes!
    }
}
```

2. **Storing waker but never calling wake()**:
```rust
// WRONG: Stores waker but doesn't wake on completion
guard.waker = Some(cx.waker().clone());
// ... later, value becomes ready but wake() is never called
```

3. **Calling wake before storing the value**:
```rust
// WRONG: Race condition
waker.wake();  // Executor might poll before value is stored
guard.value = Some(value);
```

**Correct Pattern**:
```rust
// RIGHT: Store value, then wake
guard.value = Some(value);
let waker = guard.waker.take();
drop(guard);  // Release lock before waking
if let Some(waker) = waker {
    waker.wake();
}
```

## Related Patterns

- **Custom Future**: Uses wakers to notify executors
- **Simple Executor**: Creates wakers and responds to wake calls
- **Polling Loop**: The executor side of the waker protocol
- **Spawn Blocking**: Demonstrates waking from background threads
- **Thread Pool for Blocking**: Manages multiple wakers for concurrent operations

## Known Uses

1. **tokio Runtime**: Uses wakers to schedule tasks across thread pools
2. **async-std**: Similar waker-based scheduling
3. **futures::channel**: Channels wake receivers when messages arrive
4. **tokio::time**: Timers wake futures when deadlines expire
5. **tokio::net**: I/O operations wake on readiness (epoll/kqueue events)
6. **Crossbeam Parker**: Thread parking primitive used in simple executors

**Example from block-on tests**:

```rust
use async_std::task::{spawn, sleep};
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

The timer futures wake the executor exactly when their deadlines expire, demonstrating efficient waker use.

**Waker Implementation Types**:
- `Arc<W>` where `W: Wake`: Most common, allows cloning and thread-safe waking
- Thread unparker: Simple executors (like `block-on`)
- Task queue insertion: Work-stealing executors
- Event notification: I/O reactors

**The Waker Guarantee**: If `wake()` is called after a future returns `Poll::Pending` but before it returns `Poll::Ready`, the executor will poll the future at least once more. This guarantee enables correct async execution without races or deadlocks.
