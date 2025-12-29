# Performance-Critical Architecture

## Learning Objectives

By the end of this chapter, you will:
- Master zero-copy patterns for maximum performance
- Understand SIMD vectorization in Rust
- Profile applications to identify bottlenecks
- Optimize memory allocation strategies

## Introduction

Performance-critical systems demand more than clean architecture—they require intimate knowledge of hardware, memory hierarchies, and compiler optimizations. This chapter explores techniques for building systems where every nanosecond counts, drawing from real-world patterns in our example projects.

## Zero-Copy Patterns

Zero-copy programming eliminates unnecessary memory allocations and copies, critical for high-throughput systems.

### String Processing Without Allocation

From the `grep` project pattern:

```rust
use std::io::{BufRead, BufReader};
use std::fs::File;

fn search_lines_zero_copy(path: &str, pattern: &str) -> std::io::Result<Vec<usize>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut matches = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        // Only allocate when pattern found
        if line.contains(pattern) {
            matches.push(line_num);
        }
        // line String dropped here - no accumulation
    }

    Ok(matches)
}
```

### Avoiding Intermediate Allocations

The `gap-buffer` project demonstrates efficient text manipulation:

```rust
pub struct GapBuffer<T> {
    storage: Vec<T>,
    gap_start: usize,
    gap_end: usize,
}

impl GapBuffer<char> {
    // Zero-copy view into buffer segments
    pub fn before_gap(&self) -> &[char] {
        &self.storage[..self.gap_start]
    }

    pub fn after_gap(&self) -> &[char] {
        &self.storage[self.gap_end..]
    }

    // Insert without moving data far from gap
    pub fn insert(&mut self, ch: char) {
        if self.gap_start == self.gap_end {
            self.enlarge_gap();
        }
        self.storage[self.gap_start] = ch;
        self.gap_start += 1;
        // O(1) insertion at cursor - no memcpy
    }
}
```

**Key insight**: Structure data to minimize movement. The gap buffer keeps a hole at the editing point, making insertions O(1) instead of O(n).

### Borrowed Iterators

From `binary-tree`, zero-copy traversal:

```rust
impl<T> BinaryTree<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        TreeIter {
            stack: vec![self],
        }
    }
}

// Returns references, never clones values
pub struct TreeIter<'a, T> {
    stack: Vec<&'a BinaryTree<T>>,
}

impl<'a, T> Iterator for TreeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let node = self.stack.pop()?;
        // Push references to children
        if let Some(ref left) = node.left {
            self.stack.push(left);
        }
        if let Some(ref right) = node.right {
            self.stack.push(right);
        }
        Some(&node.value)
    }
}
```

## SIMD and Vectorization

Single Instruction Multiple Data (SIMD) operations process multiple data points in one CPU instruction.

### Portable SIMD (std::simd)

The `complex` project is perfect for SIMD optimization:

```rust
#![feature(portable_simd)]
use std::simd::*;

#[derive(Clone, Copy)]
pub struct ComplexSIMD {
    pub re: f64x4,  // 4 real components
    pub im: f64x4,  // 4 imaginary components
}

impl ComplexSIMD {
    // Process 4 complex numbers at once
    pub fn add(self, other: Self) -> Self {
        ComplexSIMD {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    pub fn mul(self, other: Self) -> Self {
        // (a + bi)(c + di) = (ac - bd) + (ad + bc)i
        ComplexSIMD {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

// Batch process array of complex numbers
fn mandelbrot_simd(coords: &[(f64, f64)]) -> Vec<u32> {
    coords
        .chunks_exact(4)
        .map(|chunk| {
            let re = f64x4::from_array([
                chunk[0].0, chunk[1].0, chunk[2].0, chunk[3].0
            ]);
            let im = f64x4::from_array([
                chunk[0].1, chunk[1].1, chunk[2].1, chunk[3].1
            ]);

            let c = ComplexSIMD { re, im };
            // Process 4 points in parallel
            iterate_simd(c)
        })
        .flatten()
        .collect()
}
```

**Performance gain**: 4× speedup for operations that vectorize well.

### Auto-Vectorization

The compiler can auto-vectorize simple loops:

```rust
// Likely auto-vectorized by LLVM
fn sum_array(arr: &[f64]) -> f64 {
    arr.iter().sum()  // LLVM converts to SIMD
}

// Verify with:
// cargo rustc --release -- --emit asm
```

**Guidelines for auto-vectorization**:
1. Use iterators over manual loops
2. Avoid branches in hot loops
3. Keep loop bodies simple
4. Use powers-of-two array sizes
5. Align data to 16/32 byte boundaries

## Profiling and Bottleneck Identification

Optimization without measurement is guesswork. Systematic profiling reveals true bottlenecks.

### CPU Profiling with perf

```bash
# Build with symbols
cargo build --release
export RUSTFLAGS="-C force-frame-pointers=yes"
cargo build --release

# Profile binary
perf record --call-graph dwarf ./target/release/grep pattern file.txt
perf report

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```

### Criterion for Microbenchmarks

From `binary-tree` benchmarking:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_tree_iteration(c: &mut Criterion) {
    let tree = create_large_tree(10000);

    c.bench_function("tree iter", |b| {
        b.iter(|| {
            let sum: i32 = black_box(&tree).iter().sum();
            black_box(sum)
        })
    });
}

criterion_group!(benches, bench_tree_iteration);
criterion_main!(benches);
```

**Key technique**: `black_box` prevents compiler from optimizing away benchmarked code.

### Memory Profiling

```bash
# Heap profiling with valgrind
cargo build --release
valgrind --tool=massif ./target/release/binary

# Analyze with ms_print
ms_print massif.out.* > memory-profile.txt

# Alternative: heaptrack
heaptrack ./target/release/binary
heaptrack_gui heaptrack.binary.*.gz
```

### Identifying Hot Paths

Use `#[inline]` attributes strategically:

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>,
}

impl<T> Queue<T> {
    #[inline(always)]  // Force inline for hot path
    pub fn push(&mut self, value: T) {
        self.younger.push(value);
    }

    #[inline]  // Suggest inline
    pub fn pop(&mut self) -> Option<T> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }
            // Cold path - reversal happens rarely
            std::mem::swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }
        self.older.pop()
    }

    #[inline(never)]  // Never inline - debugging or rare path
    fn enlarge_capacity(&mut self) {
        // Complex logic, called rarely
    }
}
```

## Allocation Strategies

Memory allocation is expensive. Minimize it through strategic patterns.

### Pre-allocation and Capacity Hints

```rust
// Bad: repeated allocations
let mut vec = Vec::new();
for i in 0..10000 {
    vec.push(i);  // May reallocate 14+ times
}

// Good: pre-allocate
let mut vec = Vec::with_capacity(10000);
for i in 0..10000 {
    vec.push(i);  // Zero reallocations
}

// From iterator size_hint
let vec: Vec<_> = (0..10000).collect();  // Uses size_hint
```

### Object Pools

Reuse expensive-to-allocate objects:

```rust
use std::sync::Mutex;

pub struct BufferPool {
    pool: Mutex<Vec<Vec<u8>>>,
    capacity: usize,
}

impl BufferPool {
    pub fn new(buffer_size: usize, pool_size: usize) -> Self {
        let pool = (0..pool_size)
            .map(|_| Vec::with_capacity(buffer_size))
            .collect();

        BufferPool {
            pool: Mutex::new(pool),
            capacity: buffer_size,
        }
    }

    pub fn acquire(&self) -> Vec<u8> {
        self.pool
            .lock()
            .unwrap()
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(self.capacity))
    }

    pub fn release(&self, mut buffer: Vec<u8>) {
        buffer.clear();
        if buffer.capacity() == self.capacity {
            self.pool.lock().unwrap().push(buffer);
        }
    }
}
```

### Arena Allocation

From `gap-buffer` style patterns:

```rust
pub struct Arena<T> {
    chunks: Vec<Vec<T>>,
    current: usize,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena {
            chunks: vec![Vec::with_capacity(1024)],
            current: 0,
        }
    }

    pub fn alloc(&mut self, value: T) -> &mut T {
        if self.chunks[self.current].len() == self.chunks[self.current].capacity() {
            self.chunks.push(Vec::with_capacity(1024));
            self.current += 1;
        }

        let chunk = &mut self.chunks[self.current];
        chunk.push(value);
        chunk.last_mut().unwrap()
    }

    // All allocations freed at once when Arena dropped
}
```

**Use case**: Parsers, compilers, short-lived batch processing.

### SmallVec for Small Optimizations

```rust
use smallvec::SmallVec;

// Store up to 4 items inline, heap allocate beyond
type Stack<T> = SmallVec<[T; 4]>;

fn process_small_lists() {
    let mut stack: Stack<i32> = SmallVec::new();
    stack.push(1);
    stack.push(2);
    // No heap allocation for small stacks
}
```

## Cache-Friendly Data Structures

Modern CPUs spend more time waiting for memory than computing. Design for cache efficiency.

### Structure of Arrays (SoA)

```rust
// Array of Structures (bad for cache)
struct Particle {
    x: f32,
    y: f32,
    z: f32,
    vx: f32,
    vy: f32,
    vz: f32,
}

let particles: Vec<Particle> = vec![/* ... */];

// Iterating over positions: loads entire struct, wastes cache lines

// Structure of Arrays (good for cache)
struct Particles {
    positions_x: Vec<f32>,
    positions_y: Vec<f32>,
    positions_z: Vec<f32>,
    velocities_x: Vec<f32>,
    velocities_y: Vec<f32>,
    velocities_z: Vec<f32>,
}

// Iterating positions: all data needed is contiguous
```

### Data Alignment

```rust
#[repr(align(64))]  // Cache line alignment
pub struct CacheAligned<T> {
    data: T,
}

// Prevents false sharing in multi-threaded code
```

## Performance Decision Framework

| Technique | Best For | Overhead | Example |
|-----------|----------|----------|---------|
| Zero-copy | String/buffer processing | Design complexity | grep, gap-buffer |
| SIMD | Numeric arrays, data-parallel ops | Portability concerns | complex |
| Pre-allocation | Known-size collections | Memory waste if over-allocated | queue |
| Object pools | Expensive objects, high churn | Lock contention | Connection pools |
| Arena allocation | Batch processing, parsers | All-or-nothing deallocation | AST parsing |

## Practical Guidelines

1. **Profile First**: Don't optimize without data
2. **Optimize the Hot Path**: 90% of time spent in 10% of code
3. **Measure Twice, Code Once**: Verify optimizations improve performance
4. **Trade Space for Time**: Pre-compute, cache, or use larger buffers
5. **Know Your Hardware**: Cache sizes, SIMD width, alignment requirements

## Anti-Patterns

### Premature Optimization

```rust
// BAD: Complex before necessary
fn process(data: &[i32]) -> i32 {
    unsafe {
        // Unsafe SIMD before profiling shows need
    }
}

// GOOD: Start simple
fn process(data: &[i32]) -> i32 {
    data.iter().sum()  // Let compiler optimize
}
```

### Micro-Optimizing Cold Paths

```rust
// Don't optimize error paths that run 0.001% of the time
fn validate_input(s: &str) -> Result<Data, Error> {
    // This is an error path - clarity over speed
    Err(Error::new(&format!("Invalid input: {}", s)))
}
```

## Summary

Performance-critical architecture in Rust leverages:
- **Zero-copy patterns** to eliminate unnecessary allocations
- **SIMD** for data-parallel operations
- **Strategic profiling** to identify true bottlenecks
- **Allocation strategies** from pre-allocation to object pools
- **Cache-aware design** for maximum throughput

The projects in this repository—from `gap-buffer`'s efficient editing to `complex`'s arithmetic—demonstrate these principles in action. Master these techniques to build systems that fully exploit modern hardware.

## Further Reading

- Chapter 10.1: Advanced generics enable zero-cost abstractions
- Chapter 10.4: Design trade-offs when performance conflicts with safety
- Chapter 6: Memory management foundations underlying these optimizations
