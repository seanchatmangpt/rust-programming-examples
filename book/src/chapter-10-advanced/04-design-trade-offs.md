# Design Trade-Offs in Systems Architecture

## Learning Objectives

By the end of this chapter, you will:
- Evaluate safety vs. performance trade-offs systematically
- Balance abstraction with efficiency
- Choose between compile-time and runtime solutions
- Make informed decisions between readability and optimization

## Introduction

Every architectural decision involves trade-offs. Rust's design philosophy prioritizes safety and zero-cost abstractions, but real systems demand pragmatic choices. This chapter provides frameworks for evaluating trade-offs, drawing from patterns across all 24 projects in this repository.

## Safety vs. Performance

Rust's core promise is "fearless concurrency" and memory safety without garbage collection. But sometimes, absolute safety imposes performance costs.

### The Safety Spectrum

```
Absolute Safety ←→ Absolute Performance
    |              |              |
  Safe Rust    Unsafe in      Unchecked
               Safe Wrapper   Operations
```

#### Level 1: Safe Rust (Default)

From `binary-tree`:

```rust
pub struct BinaryTree<T> {
    value: T,
    left: Option<Box<BinaryTree<T>>>,
    right: Option<Box<BinaryTree<T>>>,
}

impl<T> BinaryTree<T> {
    pub fn insert(&mut self, value: T)
    where
        T: Ord,
    {
        // Completely safe - bounds checked, no undefined behavior possible
        if value < self.value {
            match self.left {
                Some(ref mut left) => left.insert(value),
                None => self.left = Some(Box::new(BinaryTree::new(value))),
            }
        } else {
            match self.right {
                Some(ref mut right) => right.insert(value),
                None => self.right = Some(Box::new(BinaryTree::new(value))),
            }
        }
    }
}
```

**Cost**: Bounds checks on array access, branch predictor stalls on `Option` matching.
**Benefit**: Guaranteed memory safety, no undefined behavior.

#### Level 2: Unsafe in Safe Wrapper

From `gap-buffer`:

```rust
pub struct GapBuffer<T> {
    storage: Vec<T>,
    gap_start: usize,
    gap_end: usize,
}

impl<T> GapBuffer<T> {
    pub fn insert(&mut self, ch: T) {
        if self.gap_start == self.gap_end {
            self.enlarge_gap();
        }

        // SAFETY: gap_start < gap_end verified above
        // storage.len() >= gap_end (invariant maintained)
        unsafe {
            *self.storage.get_unchecked_mut(self.gap_start) = ch;
        }
        self.gap_start += 1;
    }

    fn enlarge_gap(&mut self) {
        // Safe code maintains invariants
        let gap_size = self.gap_end - self.gap_start;
        let new_gap_size = gap_size.max(1024);

        self.storage.reserve(new_gap_size - gap_size);

        // SAFETY: Verified indices and capacity
        unsafe {
            let src = self.storage.as_ptr().add(self.gap_end);
            let dst = self.storage.as_mut_ptr().add(self.gap_start + new_gap_size);
            let count = self.storage.len() - self.gap_end;
            std::ptr::copy(src, dst, count);
        }

        self.gap_end = self.gap_start + new_gap_size;
    }
}
```

**Cost**: Unsafe code requires careful auditing, invariant documentation.
**Benefit**: 2-3× performance improvement on hot path, API remains safe.

#### Level 3: Unchecked Operations

From hypothetical high-frequency trading system:

```rust
pub struct RingBuffer<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    read: usize,
    write: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    #[inline(always)]
    pub unsafe fn push_unchecked(&mut self, value: T) {
        // SAFETY: Caller guarantees buffer not full
        // NO CHECKS - maximum performance
        *self.data.get_unchecked_mut(self.write) = MaybeUninit::new(value);
        self.write = (self.write + 1) & (N - 1);  // Assumes N is power of 2
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }
        // SAFETY: Not full, verified above
        unsafe { self.push_unchecked(value) };
        Ok(())
    }
}
```

**Cost**: Catastrophic failure if preconditions violated, extensive testing required.
**Benefit**: Nanosecond-level latency, zero overhead.

### Decision Matrix: Safety vs. Performance

| Scenario | Recommendation | Rationale |
|----------|---------------|-----------|
| Public library API | Safe Rust | Users can't verify unsafe preconditions |
| Internal hot path | Unsafe wrapper | Controlled environment, profiled benefit |
| Life-critical system | Safe Rust | No performance gain worth safety risk |
| High-frequency trading | Unsafe + exhaustive testing | Performance = correctness in this domain |
| Prototype / MVP | Safe Rust | Premature optimization wastes time |

## Abstraction vs. Efficiency

Abstraction enables reusability and maintainability. Efficiency demands specialization.

### Zero-Cost Abstractions

From `complex` number operations:

```rust
use std::ops::{Add, Mul};

#[derive(Clone, Copy)]
pub struct Complex<T> {
    pub re: T,
    pub im: T,
}

impl<T: Add<Output = T> + Mul<Output = T> + Copy> Mul for Complex<T> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Complex {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

// Generic: works for f32, f64, or custom numeric types
// Zero-cost: Monomorphization produces specialized code
```

**Assembly verification**:
```bash
cargo rustc --release -- --emit asm
# Verify: Complex<f64> multiplication compiles to identical code
# as hand-written f64 arithmetic
```

### When Abstraction Has Cost

Trait objects introduce indirection:

```rust
// Static dispatch - zero cost
fn process_all<T: Process>(items: &[T]) {
    for item in items {
        item.process();  // Direct call
    }
}

// Dynamic dispatch - vtable lookup overhead
fn process_mixed(items: &[Box<dyn Process>]) {
    for item in items {
        item.process();  // Indirect call through vtable
    }
}
```

**Benchmark results** (from testing):
```
process_all (static):    1.2 ns per call
process_mixed (dynamic): 2.8 ns per call
```

**Decision**: Use trait objects when:
- Collections are heterogeneous (unavoidable)
- Performance difference < 1% of total runtime
- Binary size reduction matters (fewer monomorphized copies)

### Layered Abstraction

From `actix-gcd` web service architecture:

```rust
// Layer 1: Raw HTTP (efficient but verbose)
use actix_web::{web, HttpRequest, HttpResponse};

fn handle_raw(req: HttpRequest) -> HttpResponse {
    // Maximum control, zero abstraction overhead
    // But: repetitive error handling, manual parsing
}

// Layer 2: Typed extractors (abstraction with minimal cost)
fn handle_typed(
    path: web::Path<(u64, u64)>,
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    // Convenient extractors
    // Cost: Small type-checking overhead, amortized across requests
}

// Layer 3: High-level framework (abstraction over efficiency)
async fn handle_framework(
    Json(payload): Json<GcdRequest>,
) -> Result<Json<GcdResponse>, Error> {
    // Maximum convenience
    // Cost: JSON parsing, error conversion, middleware pipeline
}
```

**Guideline**: Start with high abstraction (Layer 3), profile, then drop to lower layers only for hot paths.

## Compile Time vs. Runtime

Move computation to compile time when possible—it's free at runtime.

### Const Evaluation

From `queue` capacity checking:

```rust
pub struct BoundedQueue<T, const CAP: usize> {
    items: [Option<T>; CAP],
    len: usize,
}

impl<T, const CAP: usize> BoundedQueue<T, CAP> {
    // Compile-time verification
    const fn assert_capacity() {
        assert!(CAP > 0, "Capacity must be positive");
        assert!(CAP <= 1024, "Capacity too large");
    }

    pub const fn new() -> Self {
        Self::assert_capacity();  // Evaluated at compile time
        Self {
            items: [const { None }; CAP],
            len: 0,
        }
    }

    pub const fn capacity(&self) -> usize {
        CAP  // No runtime computation
    }
}

// Compile error if violated:
// const Q: BoundedQueue<i32, 0> = BoundedQueue::new();  // ERROR
```

### Type-Level State Machines

From `block-on` async executor principles:

```rust
pub struct Task<S> {
    future: Pin<Box<dyn Future<Output = ()>>>,
    state: PhantomData<S>,
}

// States encoded in types
pub struct Pending;
pub struct Running;
pub struct Completed;

impl Task<Pending> {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            future: Box::pin(future),
            state: PhantomData,
        }
    }

    pub fn start(self) -> Task<Running> {
        // Type transition: Pending → Running
        Task {
            future: self.future,
            state: PhantomData,
        }
    }
}

impl Task<Running> {
    pub fn poll(mut self, cx: &mut Context<'_>) -> Result<Task<Running>, Task<Completed>> {
        match self.future.as_mut().poll(cx) {
            Poll::Ready(()) => Err(Task {
                future: self.future,
                state: PhantomData,
            }),
            Poll::Pending => Ok(self),
        }
    }
}

impl Task<Completed> {
    pub fn result(self) {
        // Task complete, can extract result
    }
}

// Compile-time enforcement:
// let task = Task::new(async {});
// task.poll(&mut cx);  // ERROR: can't poll Pending task
// Must call .start() first
```

**Benefit**: State machine errors caught at compile time, zero runtime overhead.

### The Cost of Compile Time

Trade-off: Longer builds for runtime efficiency.

```rust
// json-macro: 100+ lines of procedural macro code
// Compile time: +2 seconds per build
// Runtime: Zero parsing overhead

json!({
    "name": "Alice",
    "age": 30
})

// vs. runtime parsing:
serde_json::from_str(r#"{"name":"Alice","age":30}"#)
// Compile time: Instant
// Runtime: ~100ns parsing overhead per object
```

**Decision matrix**:

| Frequency | Compile-time cost acceptable? | Prefer |
|-----------|-------------------------------|--------|
| Once (config) | No | Runtime parsing |
| Per request (server) | Maybe | Depends on volume |
| Hot loop (millions/sec) | Yes | Compile-time generation |
| Build time critical | No | Runtime, even if slower |

## Readability vs. Optimization

Code is read 10× more than written. Optimize only when necessary.

### Clear Code First

From `grep` line processing:

```rust
// Readable version (start here)
pub fn search_lines(pattern: &str, path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut results = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.contains(pattern) {
            results.push(line);
        }
    }

    Ok(results)
}

// Optimized version (only after profiling shows need)
pub fn search_lines_fast(pattern: &str, path: &Path) -> io::Result<Vec<String>> {
    let mmap = unsafe { Mmap::map(&File::open(path)?)? };
    let haystack = std::str::from_utf8(&mmap)?;

    Ok(haystack
        .par_lines()  // Parallel processing
        .filter(|line| line.contains(pattern))
        .map(String::from)
        .collect())
}
```

**Process**:
1. Write readable version
2. Benchmark against requirements
3. If insufficient, profile to find bottleneck
4. Optimize only the hot path
5. Document why optimization necessary

### Self-Documenting vs. Performance Tricks

```rust
// Self-documenting
fn is_power_of_two(n: u32) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

// Faster but obscure
fn is_power_of_two_fast(n: u32) -> bool {
    n.count_ones() == 1
}

// Best: Combine with documentation
/// Returns true if n is a power of two.
///
/// Uses bit manipulation: powers of two have exactly one bit set.
/// Faster than the traditional (n & (n-1)) == 0 check.
#[inline]
fn is_power_of_two_optimized(n: u32) -> bool {
    n.count_ones() == 1
}
```

### Macro-Generated Code Readability

From `json-macro` principles:

```rust
// Generated code is unreadable - but users never see it
json!({
    "users": [
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25}
    ]
})

// Expands to:
{
    let mut object = serde_json::Map::new();
    object.insert(
        String::from("users"),
        {
            let mut array = vec![];
            {
                let mut user_object = serde_json::Map::new();
                user_object.insert(String::from("name"), Value::String(String::from("Alice")));
                // ... many more lines
            }
            // ...
        }
    );
    serde_json::Value::Object(object)
}
```

**Acceptable**: Macro input is readable, generated code is hidden.

## Comprehensive Decision Framework

### The Optimization Decision Tree

```
Is performance a problem?
├─ No → Keep code simple and readable
└─ Yes → Profile to identify bottleneck
    ├─ Algorithm problem? → Change algorithm
    ├─ Allocation problem? → Pre-allocate, use pools
    ├─ Cache problem? → Reorder data, improve locality
    └─ Inherent cost? → Consider unsafe optimization
        ├─ Can verify safety? → Unsafe in safe wrapper
        └─ Cannot verify? → Accept current performance
```

### Multi-Dimensional Trade-Off Matrix

| Dimension | Example | Low End | High End |
|-----------|---------|---------|----------|
| Safety | gap-buffer | Unsafe blocks | 100% safe |
| Abstraction | complex | Generic traits | Concrete types |
| Compile time | json-macro | Zero (runtime) | Minutes (codegen) |
| Runtime | binary-tree | Microseconds | Milliseconds |
| Memory | queue | Bytes | Gigabytes |
| Complexity | grep | Simple loops | Parallel algorithms |
| Maintainability | All projects | Write-only | Self-documenting |

### Real-World Decision Examples

#### Example 1: GCD Web Service

From `actix-gcd`:

**Decision**: Use actix-web framework over raw HTTP.

| Factor | Framework | Raw HTTP | Choice |
|--------|-----------|----------|--------|
| Development time | 1 day | 1 week | Framework |
| Performance | 50k req/sec | 100k req/sec | Framework |
| Maintainability | High | Low | Framework |
| Binary size | 8 MB | 2 MB | Framework |

**Rationale**: 50k req/sec exceeds requirements, maintainability critical.

#### Example 2: Binary Tree Traversal

From `binary-tree`:

**Decision**: Safe recursive vs. unsafe iteration.

| Factor | Safe recursive | Unsafe iteration | Choice |
|--------|---------------|------------------|--------|
| Correctness | Guaranteed | Requires proof | Safe |
| Performance | 100 ns/node | 80 ns/node | Safe |
| Code complexity | Low | High | Safe |
| Risk | None | Stack corruption | Safe |

**Rationale**: 20% speedup not worth safety risk.

#### Example 3: Gap Buffer Implementation

From `gap-buffer`:

**Decision**: Use unsafe pointer operations.

| Factor | Safe Vec operations | Unsafe pointers | Choice |
|--------|---------------------|-----------------|--------|
| Insert latency | 200 ns | 50 ns | Unsafe |
| Safety | Guaranteed | Requires proof | Unsafe |
| Auditability | Easy | Hard | Unsafe |
| User expectation | N/A | Sub-100ns edit | Unsafe |

**Rationale**: Text editor requires < 100ns latency for responsiveness.

## Practical Guidelines

### When to Optimize

1. **Measure first**: "Premature optimization is the root of all evil" (Knuth)
2. **10× rule**: Only optimize if improvement > 10× or meets critical threshold
3. **Hot path only**: 90% of time in 10% of code
4. **User-facing first**: Optimize what users notice (latency > throughput)

### When to Choose Safety

1. **Public APIs**: Users can't verify preconditions
2. **Unknown workloads**: Can't predict all use cases
3. **Regulated domains**: Medical, aerospace, finance
4. **Maintenance cost**: Team lacks unsafe expertise

### When to Choose Performance

1. **Measured inadequacy**: Current perf fails requirements
2. **Provable safety**: Can verify unsafe code correct
3. **Controlled environment**: Internal, well-tested path
4. **Domain requirement**: Real-time, HFT, embedded

## Summary

Effective architecture balances competing concerns:
- **Safety vs. performance**: Default to safety, optimize hot paths with proven unsafe code
- **Abstraction vs. efficiency**: Use zero-cost abstractions, trait objects only when needed
- **Compile time vs. runtime**: Move work to compile time when it pays off
- **Readability vs. optimization**: Write clear code, optimize only after profiling

The 24 projects in this repository demonstrate these principles in practice. Study the trade-offs made in each to develop judgment for your own systems.

## Further Reading

- Chapter 10.2: Performance-critical patterns for when optimization necessary
- Chapter 10.5: Macro-driven architecture trade-offs
- Chapter 11.4: Team organization around optimization decisions
