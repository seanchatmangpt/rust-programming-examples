# Advanced Generic Patterns

## Learning Objectives

By the end of this chapter, you will:
- Understand advanced generic patterns including GATs and const generics
- Learn workarounds for higher-kinded types in Rust
- Master specialization techniques for performance optimization
- Apply compile-time configuration patterns effectively

## Introduction

Generic programming in Rust extends far beyond basic type parameters. This chapter explores advanced generic patterns that enable powerful abstractions while maintaining zero-cost guarantees. We'll examine techniques that push the boundaries of Rust's type system, drawing from real implementations across our example projects.

## Higher-Kinded Types and Workarounds

Rust doesn't directly support higher-kinded types (HKT) - types that abstract over type constructors. However, several workarounds enable HKT-like patterns:

### The Associated Type Pattern

The `binary-tree` project demonstrates this through its iterator implementation:

```rust
pub struct BinaryTree<T> {
    value: T,
    left: Option<Box<BinaryTree<T>>>,
    right: Option<Box<BinaryTree<T>>>,
}

impl<T> BinaryTree<T> {
    pub fn iter(&self) -> TreeIter<'_, T> {
        TreeIter {
            unvisited: vec![self],
        }
    }
}

pub struct TreeIter<'a, T> {
    unvisited: Vec<&'a BinaryTree<T>>,
}

impl<'a, T> Iterator for TreeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.unvisited.pop()?;
        if let Some(ref left) = node.left {
            self.unvisited.push(left);
        }
        if let Some(ref right) = node.right {
            self.unvisited.push(right);
        }
        Some(&node.value)
    }
}
```

This pattern abstracts over the container type (`BinaryTree`) while producing a specific iterator type. The key insight: use associated types to simulate type constructors.

### Generic Associated Types (GATs)

Stabilized in Rust 1.65, GATs allow associated types with their own generic parameters:

```rust
trait LendingIterator {
    type Item<'a> where Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

// Without GATs, you'd need separate traits for each lifetime
// GATs enable self-referential iteration patterns
```

The `gap-buffer` project could leverage GATs for efficient text editing operations:

```rust
trait GapBufferOps {
    type View<'a> where Self: 'a;

    fn view_before_gap<'a>(&'a self) -> Self::View<'a>;
    fn view_after_gap<'a>(&'a self) -> Self::View<'a>;
}
```

### The Defunctionalization Pattern

When HKT is unavoidable, defunctionalization provides an escape hatch:

```rust
// Instead of abstracting over F<T>
// Define trait for operations on F
trait Mappable<A> {
    type Mapped<B>;
    fn map<B, F>(self, f: F) -> Self::Mapped<B>
    where
        F: FnMut(A) -> B;
}

impl<A> Mappable<A> for Vec<A> {
    type Mapped<B> = Vec<B>;

    fn map<B, F>(self, f: F) -> Vec<B>
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(f).collect()
    }
}
```

This transforms the abstraction from "container types" to "operations on container types."

## Const Generics for Compile-Time Configuration

Const generics enable type-level programming with values, not just types:

```rust
struct FixedBuffer<T, const N: usize> {
    data: [T; N],
    len: usize,
}

impl<T, const N: usize> FixedBuffer<T, N> {
    const fn capacity(&self) -> usize {
        N
    }

    fn push(&mut self, value: T) -> Result<(), T> {
        if self.len < N {
            self.data[self.len] = value;
            self.len += 1;
            Ok(())
        } else {
            Err(value)
        }
    }
}

// Zero runtime overhead - capacity is known at compile time
let mut buffer: FixedBuffer<i32, 128> = FixedBuffer::new();
```

### Const Generic Patterns from Complex Numbers

The `complex` project could benefit from const generics for SIMD optimization:

```rust
use std::ops::Add;

#[repr(C, align(16))]
struct ComplexArray<const N: usize> {
    reals: [f64; N],
    imags: [f64; N],
}

impl<const N: usize> ComplexArray<N> {
    // Compile-time guarantee of power-of-2 size for SIMD
    const fn assert_power_of_two() {
        assert!(N.is_power_of_two(), "N must be power of 2");
    }

    fn add_simd(&mut self, other: &Self)
    where
        [(); N / 4]: Sized, // Requires N divisible by 4
    {
        // SIMD operations guaranteed safe by const generic bounds
    }
}
```

## Specialization for Performance

While full specialization remains unstable, min_specialization enables critical optimizations:

```rust
// Generic default implementation
trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl<T> ToBytes for T {
    default fn to_bytes(&self) -> Vec<u8> {
        // Slow generic serialization
        bincode::serialize(self).unwrap()
    }
}

// Specialized for Copy types
impl<T: Copy> ToBytes for T {
    fn to_bytes(&self) -> Vec<u8> {
        // Fast memcpy for Copy types
        unsafe {
            let ptr = self as *const T as *const u8;
            let size = std::mem::size_of::<T>();
            std::slice::from_raw_parts(ptr, size).to_vec()
        }
    }
}
```

### Specialization in Binary Trees

For `binary-tree`, specialization enables optimized operations:

```rust
trait Summable {
    fn sum(&self) -> i64;
}

impl<T> Summable for BinaryTree<T>
where
    T: Into<i64> + Clone,
{
    default fn sum(&self) -> i64 {
        // Generic recursive traversal
        let mut total = self.value.clone().into();
        if let Some(ref left) = self.left {
            total += left.sum();
        }
        if let Some(ref right) = self.right {
            total += right.sum();
        }
        total
    }
}

// Specialized for i64 - direct accumulation
impl Summable for BinaryTree<i64> {
    fn sum(&self) -> i64 {
        // Tail-recursive optimization possible
        self.iter().copied().sum()
    }
}
```

## Advanced Type-Level Programming

### Type-Level Peano Numbers

Encode natural numbers in types for compile-time verification:

```rust
trait Nat {}

struct Zero;
struct Succ<N: Nat>(PhantomData<N>);

impl Nat for Zero {}
impl<N: Nat> Nat for Succ<N> {}

// Type-level addition
trait Add<N: Nat>: Nat {
    type Output: Nat;
}

impl<N: Nat> Add<N> for Zero {
    type Output = N;
}

impl<N: Nat, M: Nat> Add<M> for Succ<N>
where
    N: Add<M>,
{
    type Output = Succ<<N as Add<M>>::Output>;
}
```

This enables compile-time dimensional analysis, matrix size verification, and protocol state machines.

## Generic Bounds and Where Clauses

### Complex Bounds Management

From `generic-queue`:

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>,
}

impl<T> Queue<T>
where
    T: Clone + Send + 'static,
{
    // Bounds required for thread-safe queue operations
    pub fn spawn_processor<F>(&self, f: F) -> JoinHandle<()>
    where
        F: FnMut(T) + Send + 'static,
    {
        // Implementation
    }
}
```

### Negative Bounds (Future)

While not yet stable, negative trait bounds would enable:

```rust
// Hypothetical future syntax
fn process<T: !Copy>(value: T) {
    // Guaranteed value is moved, enabling optimizations
}
```

## Performance Considerations

### Monomorphization Overhead

Each generic instantiation creates a separate compiled function:

```rust
fn process<T: Display>(value: T) {
    println!("{}", value);
}

// Generates three separate functions in binary
process(42_i32);        // process::<i32>
process("hello");       // process::<&str>
process(3.14_f64);      // process::<f64>
```

**Trade-off**: Code size vs. performance. Solutions:
1. Use trait objects for rarely-called code paths
2. Factor out non-generic helper functions
3. Use `#[inline(always)]` judiciously

### Zero-Cost Abstractions

Generics maintain zero-cost guarantees:

```rust
// From binary-tree
let tree = BinaryTree::new(10);
let sum: i32 = tree.iter().sum();

// Compiles to equivalent of:
let mut sum = 0;
// ... manual tree traversal ...
// No iterator overhead in release builds
```

## Decision Framework: When to Use Advanced Generics

| Pattern | Use When | Avoid When | Example Project |
|---------|----------|------------|-----------------|
| GATs | Self-referential iterators | Simple iteration | gap-buffer |
| Const generics | Fixed-size arrays, SIMD | Dynamic sizing | complex |
| Specialization | Performance-critical paths | General libraries | binary-tree |
| Type-level programming | Compile-time verification | Runtime flexibility | N/A (advanced) |

## Practical Guidelines

1. **Start Simple**: Begin with basic generics, add complexity only when needed
2. **Measure First**: Profile before specializing - don't optimize prematurely
3. **Document Bounds**: Complex where clauses need clear explanations
4. **Test Instantiations**: Verify common type parameter combinations
5. **Consider Compile Time**: Each generic instantiation adds to build time

## Summary

Advanced generic patterns unlock powerful abstractions in Rust:
- GATs enable self-referential types previously impossible
- Const generics bring compile-time configuration with zero overhead
- Specialization optimizes hot paths while maintaining generic interfaces
- Type-level programming provides compile-time verification

These techniques appear throughout high-performance Rust codebases. Master them to build libraries that are both ergonomic and blazingly fast.

## Further Reading

- Chapter 10.3: Advanced trait patterns building on these foundations
- Chapter 8: Async/await uses GATs extensively in Stream trait
- Chapter 6: Memory safety principles underlying generic programming
