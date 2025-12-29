# Async Trait Patterns

Combining async functions with traits creates powerful abstractions but introduces complexity around lifetimes, dynamic dispatch, and `Send` bounds. Understanding these patterns enables you to design flexible async interfaces while maintaining the zero-cost guarantees that make Rust's async model compelling.

## The Async-in-Trait Challenge

Before Rust 1.75 (and still relevant for understanding the internals), you couldn't write async methods directly in traits:

```rust
// This didn't work before async-fn-in-traits stabilization
trait Fetcher {
    async fn fetch(&self, url: &str) -> Result<String, Error>;  // Error!
}
```

The problem: async functions return `impl Future`, and traits don't support `impl Trait` in return position. The future's concrete type is compiler-generated and unknowable in the trait definition.

### Manual Future Implementation

The workaround involves returning boxed futures manually:

```rust
use std::future::Future;
use std::pin::Pin;

trait Fetcher {
    fn fetch(&self, url: &str)
        -> Pin<Box<dyn Future<Output = Result<String, Error>> + Send + '_>>;
}
```

This signature deserves careful examination:

- **`Pin<Box<...>>`**: The future is heap-allocated and pinned in memory (required by `Future` trait)
- **`dyn Future<...>`**: Dynamic dispatch—the concrete type is erased
- **`+ Send`**: The future can be sent across threads
- **`+ '_`**: The future borrows from `&self` (elided lifetime made explicit)

Implementing this trait requires boxing your async blocks:

```rust
struct HttpFetcher {
    client: HttpClient,
}

impl Fetcher for HttpFetcher {
    fn fetch(&self, url: &str)
        -> Pin<Box<dyn Future<Output = Result<String, Error>> + Send + '_>>
    {
        Box::pin(async move {
            self.client.get(url).await?.text().await
        })
    }
}
```

The `Box::pin` allocates the future on the heap and pins it. This has performance implications: you lose the zero-cost abstraction because heap allocation isn't free.

### The Modern Approach: Async Fn in Traits

Since Rust 1.75, you can use async functions directly in traits:

```rust
trait Fetcher {
    async fn fetch(&self, url: &str) -> Result<String, Error>;
}

impl Fetcher for HttpFetcher {
    async fn fetch(&self, url: &str) -> Result<String, Error> {
        self.client.get(url).await?.text().await
    }
}
```

This compiles to essentially the same code as the manual version but with better ergonomics. However, it still uses dynamic dispatch when used through trait objects, so understanding the manual pattern remains valuable.

## Dynamic Async with Trait Objects

When you need runtime polymorphism over async operations, trait objects combined with async create interesting challenges:

```rust
// Vector of different fetcher implementations
let fetchers: Vec<Box<dyn Fetcher>> = vec![
    Box::new(HttpFetcher::new()),
    Box::new(CachedFetcher::new()),
    Box::new(MockFetcher::new()),
];

// Iterate and call async methods
for fetcher in &fetchers {
    let result = fetcher.fetch("https://example.com").await?;
    println!("{}", result);
}
```

This works, but notice the cost: two levels of indirection (boxed trait object + boxed future), plus dynamic dispatch overhead. For hot paths, this may be unacceptable.

### Static Dispatch Alternative

When types are known at compile time, prefer static dispatch:

```rust
async fn fetch_all<F: Fetcher>(fetcher: &F, urls: &[&str]) -> Vec<Result<String, Error>> {
    let mut results = vec![];
    for url in urls {
        results.push(fetcher.fetch(url).await);
    }
    results
}

// No boxing, no dynamic dispatch
let http_fetcher = HttpFetcher::new();
fetch_all(&http_fetcher, &["url1", "url2"]).await;
```

The generic parameter `F: Fetcher` monomorphizes the function for each concrete type, eliminating runtime overhead.

**Design Principle**: Use trait objects for flexibility, generics for performance. Measure before choosing.

## Lifetime Implications of Async

Async functions complicate lifetimes because the future must capture all borrowed data it needs across `.await` points:

```rust
trait Processor {
    async fn process<'a>(&'a self, data: &'a str) -> String;
}
```

This signature ties the future's lifetime to both `self` and `data`. The future cannot outlive either reference. Attempting to spawn such a future fails:

```rust
impl Processor for MyProcessor {
    async fn process<'a>(&'a self, data: &'a str) -> String {
        format!("Processed: {}", data)
    }
}

async fn spawn_process(processor: &MyProcessor, data: &str) {
    // ERROR: Cannot spawn - future borrows from data
    task::spawn(processor.process(data));  // Won't compile
}
```

The error: `task::spawn` requires `'static` futures, but our future borrows from `data`, which isn't `'static`.

### Solutions

**Solution 1**: Make the future self-contained by taking ownership:

```rust
trait Processor {
    async fn process(&self, data: String) -> String;  // Owned String
}

async fn spawn_process(processor: Arc<MyProcessor>, data: String) {
    // Now it compiles - future owns its data
    task::spawn(async move {
        processor.process(data).await
    });
}
```

**Solution 2**: Use `Arc` to extend lifetimes:

```rust
async fn spawn_with_arc(processor: Arc<MyProcessor>, data: Arc<str>) {
    task::spawn(async move {
        // Arc makes both processor and data 'static
        processor.process(&data).await
    });
}
```

**Solution 3**: Accept the limitation and don't spawn:

```rust
async fn sequential_process(processor: &MyProcessor, data: &str) {
    // Fine - we await directly without spawning
    let result = processor.process(data).await;
}
```

**Design Principle**: For spawnable async, require owned or `Arc`-wrapped data. For non-spawnable async, leverage borrowing.

## Async Trait Patterns for Concurrency

Traits can encode concurrency patterns directly:

### The Streamer Pattern

```rust
use futures::stream::Stream;
use std::pin::Pin;

trait EventSource {
    type Event;
    fn events(&self)
        -> Pin<Box<dyn Stream<Item = Self::Event> + Send + '_>>;
}

impl EventSource for WebSocketSource {
    type Event = Message;

    fn events(&self)
        -> Pin<Box<dyn Stream<Item = Message> + Send + '_>>
    {
        Box::pin(async_stream::stream! {
            while let Some(msg) = self.connection.next().await {
                yield msg;
            }
        })
    }
}

// Usage
async fn consume_events<S: EventSource>(source: &S) {
    let mut events = source.events();
    while let Some(event) = events.next().await {
        handle_event(event).await;
    }
}
```

This pattern enables lazy, on-demand event production with backpressure built-in.

### The Worker Pattern

```rust
trait Worker: Send + Sync {
    type Task: Send;
    type Result: Send;

    async fn execute(&self, task: Self::Task) -> Self::Result;
}

async fn run_worker_pool<W: Worker>(
    worker: Arc<W>,
    tasks: Vec<W::Task>
) -> Vec<W::Result>
where
    W::Task: 'static,
    W::Result: 'static,
{
    let handles: Vec<_> = tasks.into_iter().map(|task| {
        let worker = worker.clone();
        task::spawn(async move {
            worker.execute(task).await
        })
    }).collect();

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }
    results
}
```

The `Send + Sync` bounds on `Worker` and its associated types ensure thread safety. The `'static` bounds allow spawning across threads.

## Object-Safe Async Traits

Not all traits can be made into trait objects. For async traits to be object-safe:

1. **No generic methods** (other than lifetime parameters)
2. **No `Self: Sized` requirement**
3. **Returns must be `dyn`-compatible** (hence the `Box<dyn Future>` pattern)

```rust
// Object-safe
trait ObjectSafeAsync {
    fn compute(&self) -> Pin<Box<dyn Future<Output = i32> + Send + '_>>;
}

// NOT object-safe - generic parameter
trait NotObjectSafe {
    fn compute<T>(&self, arg: T) -> Pin<Box<dyn Future<Output = T> + Send + '_>>;
}
```

When object safety matters (runtime polymorphism needed), constrain your trait design accordingly.

## The async-trait Crate

For codebases not yet on Rust 1.75+, the `async-trait` crate provides a macro that automates the boxing pattern:

```rust
use async_trait::async_trait;

#[async_trait]
trait Fetcher {
    async fn fetch(&self, url: &str) -> Result<String, Error>;
}

#[async_trait]
impl Fetcher for HttpFetcher {
    async fn fetch(&self, url: &str) -> Result<String, Error> {
        self.client.get(url).await?.text().await
    }
}
```

The macro transforms this into the manual `Pin<Box<dyn Future>>` pattern automatically. It's purely syntactic sugar, but dramatically improves readability.

### Performance Considerations

The boxing introduced by async traits (whether manual or via `async-trait`) has measurable cost:

- **Heap allocation**: Every call allocates
- **Dynamic dispatch**: Virtual function call overhead
- **Indirection**: Extra pointer dereference

For performance-critical code, measure the impact and consider:

- **Monomorphization**: Use generics instead of trait objects
- **Enum dispatch**: For a small, fixed set of types
- **Static dispatch**: When types are known at compile time

```rust
// Enum dispatch - no boxing, no dynamic dispatch
enum FetcherType {
    Http(HttpFetcher),
    Cached(CachedFetcher),
}

impl FetcherType {
    async fn fetch(&self, url: &str) -> Result<String, Error> {
        match self {
            FetcherType::Http(f) => f.fetch(url).await,
            FetcherType::Cached(f) => f.fetch(url).await,
        }
    }
}
```

This compiles to efficient match dispatch with no allocation.

## Best Practices

1. **Prefer static dispatch** when types are known at compile time
2. **Use trait objects** only when runtime polymorphism is necessary
3. **Be explicit about Send bounds** on async trait methods for multi-threaded executors
4. **Prefer owned types** in spawnable async traits
5. **Measure performance** when boxing futures in hot paths
6. **Consider enum dispatch** for finite, known type sets

Async traits unlock powerful abstractions in Rust's async ecosystem. Understanding the lifetime, allocation, and dispatch implications ensures you use them effectively without sacrificing the performance that makes async Rust compelling. See **Chapter 3: Advanced Trait Patterns** for more on trait design principles that complement these async-specific patterns.
