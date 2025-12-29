# Thread Pool for Blocking Operations

## Context

You're using the Spawn Blocking pattern to run blocking operations from async code. Your application spawns many blocking tasks—database queries, file I/O, CPU-intensive computations, password hashing. Each call to `spawn_blocking` creates a new OS thread, which works but has serious overhead.

Creating a thread involves:
- System call overhead (~10-100μs)
- Stack allocation (typically 2MB per thread)
- OS scheduler registration
- Context switch costs

For high-frequency blocking operations, this overhead is unacceptable. Spawning 10,000 threads would allocate 20GB of memory just for stacks and likely crash the system.

## Problem

**How do you efficiently execute many blocking operations without the overhead of creating a thread per operation?**

The challenges:
- **Thread Creation Cost**: Spawning threads is expensive (time and memory)
- **System Limits**: Operating systems limit the number of threads (typically 1000-10000)
- **Resource Waste**: Short-lived threads create and destroy stacks repeatedly
- **Scalability**: Need to handle thousands of blocking operations without proportional resource growth
- **Responsiveness**: Must still maintain low latency for individual operations
- **Load Balancing**: Should distribute work evenly across available resources

Naive approaches fail:
```rust
// WRONG: Spawns 10,000 threads (likely crashes)
for i in 0..10_000 {
    std::thread::spawn(move || {
        blocking_operation(i);
    });
}
```

## Forces

- **Thread Reuse**: Amortize thread creation cost across many operations
- **Bounded Resources**: Limit total thread count regardless of work submitted
- **Work Queue**: Operations must wait if all threads are busy
- **Fairness**: Operations should execute in roughly FIFO order
- **Backpressure**: System should apply pushback when overloaded
- **Shutdown**: Must cleanly terminate all threads and pending work
- **Task Ownership**: Work must be movable between threads (Send + 'static)

## Solution

**Maintain a fixed-size pool of worker threads that pull blocking operations from a shared queue.**

While the example `spawn-blocking` code spawns threads directly, production implementations use thread pools. Here's the architecture:

**Conceptual Thread Pool Design**:

```rust
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let job = receiver.lock().unwrap().recv();

                match job {
                    Ok(job) => {
                        println!("Worker {} got a job", id);
                        job();
                    }
                    Err(_) => {
                        println!("Worker {} shutting down", id);
                        break;
                    }
                }
            }
        });

        Worker { id, thread }
    }
}
```

**Integrating with spawn_blocking**:

```rust
use std::sync::LazyLock;

static BLOCKING_POOL: LazyLock<ThreadPool> = LazyLock::new(|| {
    let size = num_cpus::get().max(4);  // At least 4 threads
    ThreadPool::new(size)
});

pub fn spawn_blocking<T, F>(closure: F) -> SpawnBlocking<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let inner = Arc::new(Mutex::new(Shared {
        value: None,
        waker: None,
    }));

    // Submit to thread pool instead of spawning thread
    BLOCKING_POOL.execute({
        let inner = inner.clone();
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

**The Architecture**:

1. **Fixed Thread Count**: Create N worker threads at startup (e.g., num_cpus)
2. **Work Queue**: Shared MPSC channel holds pending work
3. **Worker Loop**: Each thread loops: receive job → execute → repeat
4. **Job Submission**: `execute()` sends work to the channel
5. **Automatic Distribution**: First available worker picks up the next job

**Example Usage** (from tests, now efficient):

```rust
// This now uses a thread pool - efficient!
let futures: Vec<_> = (0..100)
    .map(|i| (i, spawn_blocking(move || {
        expensive_computation(i)
    })))
    .collect();

for (i, f) in futures {
    assert_eq!(f.await, i);
}
```

**With Thread Pool**:
- Creates pool of ~8 threads (based on CPU count)
- 100 jobs queued
- Workers execute jobs in parallel (8 at a time)
- Total threads: 8 (not 100)
- Memory: 16MB (not 200MB)

**Queue Behavior**:

```rust
// If pool has 4 threads and we submit 10 jobs:
for i in 0..10 {
    spawn_blocking(move || {
        println!("Job {}", i);
        expensive_work(i);
    });
}

// Execution timeline:
// t=0:    Workers 0-3 start jobs 0-3
// t=100:  Job 0 completes, worker 0 takes job 4
// t=150:  Job 2 completes, worker 2 takes job 5
// ...
// Jobs queue until workers are available
```

**Production Enhancements**:

1. **Dynamic Sizing**:
```rust
let pool = ThreadPool::new_adaptive(
    min_threads: 2,
    max_threads: 100,
    idle_timeout: Duration::from_secs(60),
);
```

2. **Priority Queues**:
```rust
pub enum Priority {
    High,
    Normal,
    Low,
}

pool.execute_with_priority(Priority::High, || {
    critical_operation();
});
```

3. **Bounded Queues** (backpressure):
```rust
let pool = ThreadPool::new_bounded(
    threads: 8,
    queue_size: 1000,
);

// Blocks if queue is full
pool.execute(|| work());

// Returns error if queue is full
pool.try_execute(|| work())?;
```

4. **Graceful Shutdown**:
```rust
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Close channel - workers will exit
        drop(self.sender.clone());

        // Wait for workers to finish
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
```

**Real-World Configuration**:

```rust
// tokio uses a blocking thread pool
#[tokio::main]
async fn main() {
    // Configured via runtime builder
    let rt = tokio::runtime::Builder::new_multi_thread()
        .max_blocking_threads(512)  // Default: 512
        .build()
        .unwrap();
}
```

## Resulting Context

**Benefits**:

1. **Resource Efficiency**: Fixed memory usage regardless of job count
   - 8 threads × 2MB = 16MB (vs potentially GBs without pooling)

2. **Bounded Thread Count**: Never exceeds pool size
   ```rust
   // Safe: only creates 8 threads
   for i in 0..1_000_000 {
       spawn_blocking(move || work(i));
   }
   ```

3. **Amortized Overhead**: Thread creation cost paid once, not per job
   - Thread creation: 100μs × 8 = 800μs total
   - vs 100μs × 1,000,000 = 100 seconds without pooling

4. **Automatic Load Balancing**: Workers pull from shared queue
   - Fast jobs finish quickly, worker takes next job
   - Slow jobs don't block other workers

5. **Backpressure** (with bounded queue): System can't be overloaded
   ```rust
   // Queue full - caller must wait
   pool.execute(|| work());  // Blocks until space available
   ```

**Trade-offs**:

1. **Queueing Delay**: Jobs wait if all workers busy
   ```rust
   // 100 jobs, 8 workers
   // Job 9 waits until a worker is free
   // Latency = queue_time + execution_time
   ```

2. **No Cancellation**: Submitted jobs execute eventually (can't cancel queued work easily)
   ```rust
   let f = spawn_blocking(|| slow_work());
   drop(f);  // Future dropped, but work still executes
   ```

3. **Memory for Queue**: Unbounded queue can grow indefinitely
   ```rust
   // Bad: Can exhaust memory
   for i in 0..usize::MAX {
       spawn_blocking(move || work(i));
   }
   ```

4. **Worker Starvation**: Long-running jobs block workers
   ```rust
   // Bad: Blocks all 8 workers for 1 hour each
   for _ in 0..8 {
       spawn_blocking(|| {
           std::thread::sleep(Duration::from_secs(3600));
       });
   }
   // All other jobs queue for an hour
   ```

**Sizing Guidelines**:

```rust
// CPU-bound work (computation, hashing)
let pool_size = num_cpus::get();

// I/O-bound work (database, file I/O)
let pool_size = num_cpus::get() * 4;  // Higher ratio

// Mixed workload
let pool_size = num_cpus::get() * 2;  // Conservative

// Example from tokio
tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_cpus::get())      // Async workers
    .max_blocking_threads(512)            // Blocking pool (large)
    .build()
```

**Monitoring**:

```rust
struct PoolMetrics {
    active_threads: usize,
    queued_jobs: usize,
    total_executed: u64,
    avg_queue_time: Duration,
}

impl ThreadPool {
    pub fn metrics(&self) -> PoolMetrics {
        // Implementation
    }
}

// Alert if queue is backing up
if pool.metrics().queued_jobs > 1000 {
    eprintln!("Warning: Thread pool saturated");
}
```

## Related Patterns

- **Spawn Blocking**: What this pattern optimizes
- **Custom Future**: The underlying async primitive
- **Waker Mechanism**: How completed work notifies async tasks
- **Simple Executor**: Consumer of the futures produced
- **Work Stealing**: Advanced load balancing technique

## Known Uses

1. **tokio::task::spawn_blocking**: Uses a thread pool with 512 max threads
   ```rust
   tokio::task::spawn_blocking(|| {
       // Runs on tokio's blocking pool
   }).await?;
   ```

2. **async-std::task::spawn_blocking**: Similar thread pool
   ```rust
   async_std::task::spawn_blocking(|| {
       // Runs on async-std's blocking pool
   }).await;
   ```

3. **rayon**: Data-parallelism library with thread pool
   ```rust
   rayon::ThreadPoolBuilder::new()
       .num_threads(8)
       .build_global()
       .unwrap();
   ```

4. **actix-web::web::block**: Web framework blocking pool
   ```rust
   web::block(|| {
       // Database operation
   }).await?;
   ```

5. **Database connection pools**: Similar pattern for connections
   ```rust
   let pool = r2d2::Pool::builder()
       .max_size(15)
       .build(manager)?;
   ```

**Real-World Example** (from spawn-blocking tests, now efficient):

```rust
// Password verification: CPU-intensive
async fn verify_password(password: &str, hash: &str, key: &str)
                        -> Result<bool, argonautica::Error>
{
    let password = password.to_string();
    let hash = hash.to_string();
    let key = key.to_string();

    // Uses thread pool - efficient even with many calls
    spawn_blocking(move || {
        argonautica::Verifier::default()
            .with_hash(hash)
            .with_password(password)
            .with_secret_key(key)
            .verify()
    }).await
}

// Efficient: reuses threads across 1000 verifications
for i in 0..1000 {
    verify_password("pass", "hash", "key").await?;
}
```

**Performance Comparison**:

| Operation | No Pool (spawn thread) | Thread Pool (8 threads) |
|-----------|------------------------|-------------------------|
| 1 job | 100μs | 100μs |
| 100 jobs | 10ms | 1.25ms (8x parallel) |
| 1000 jobs | 100ms | 12.5ms (8x parallel) |
| 10000 jobs | 1s + likely crash | 125ms (8x parallel) |
| Memory (10000 jobs) | 20GB | 16MB |

The thread pool pattern is essential for production async Rust, making blocking operations practical at scale.
