# Advanced Trait Patterns

## Learning Objectives

By the end of this chapter, you will:
- Master marker traits for compile-time guarantees
- Implement sealed traits to control API surface
- Use blanket implementations effectively at scale
- Understand trait object performance implications

## Introduction

Traits are Rust's mechanism for abstraction, polymorphism, and compile-time verification. Beyond basic trait implementation, advanced patterns enable sophisticated API design, performance optimization, and compile-time guarantees. This chapter explores techniques used in production Rust systems.

## Marker Traits for Compile-Time Guarantees

Marker traits carry no methods but encode properties verified at compile time. They're Rust's version of "phantom types."

### Standard Library Markers

The most important markers come from `std::marker`:

```rust
pub unsafe trait Send {}
pub unsafe trait Sync {}
pub trait Copy: Clone {}
pub trait Sized {}
pub struct PhantomData<T: ?Sized>;
```

These traits enable the type system to verify concurrency safety and memory layout.

### Custom Marker Traits

From `gap-buffer` principles:

```rust
/// Marker: Type can be efficiently moved in memory
pub unsafe trait Relocatable {}

// Safe for simple types
unsafe impl Relocatable for i32 {}
unsafe impl Relocatable for f64 {}
unsafe impl<T: Relocatable> Relocatable for Option<T> {}

// Unsafe for types with self-referential pointers
// (Don't implement for Pin, MutexGuard, etc.)

pub struct GapBuffer<T: Relocatable> {
    storage: Vec<T>,
    gap_start: usize,
    gap_end: usize,
}

impl<T: Relocatable> GapBuffer<T> {
    // Move gap efficiently - guaranteed safe by Relocatable bound
    fn move_gap(&mut self, new_pos: usize) {
        if new_pos < self.gap_start {
            let src = new_pos;
            let dst = self.gap_end - (self.gap_start - new_pos);
            let count = self.gap_start - new_pos;
            // Safe: T is Relocatable
            unsafe {
                std::ptr::copy(
                    self.storage.as_ptr().add(src),
                    self.storage.as_mut_ptr().add(dst),
                    count,
                );
            }
            self.gap_end = dst + count;
            self.gap_start = new_pos;
        }
    }
}
```

The `Relocatable` marker prevents misuse at compile time—attempting to use non-relocatable types in `GapBuffer` causes a compile error.

### Capability Traits

Encode what operations a type supports:

```rust
/// Marker: Type has cheap clone (Copy or Rc/Arc)
pub trait CheapClone: Clone {}

impl<T: Copy> CheapClone for T {}
impl<T> CheapClone for Rc<T> {}
impl<T> CheapClone for Arc<T> {}

// Use to enable optimizations
pub struct BinaryTree<T: CheapClone> {
    value: T,
    left: Option<Box<BinaryTree<T>>>,
    right: Option<Box<BinaryTree<T>>>,
}

impl<T: CheapClone> BinaryTree<T> {
    // Can clone freely without performance penalty
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }
}
```

## Sealed Traits: Controlling the API Surface

Sealed traits prevent external crates from implementing a trait, enabling future changes without breaking compatibility.

### The Sealed Pattern

From Rust's standard library pattern:

```rust
mod private {
    pub trait Sealed {}
}

pub trait IteratorExt: private::Sealed {
    fn custom_collect<B>(self) -> B
    where
        Self: Sized;
}

// Only types in this crate can implement IteratorExt
impl<T> private::Sealed for T where T: Iterator {}

impl<T> IteratorExt for T
where
    T: Iterator,
{
    fn custom_collect<B>(self) -> B {
        // Implementation
        unimplemented!()
    }
}
```

External crates can **use** `IteratorExt` but can't **implement** it—the private `Sealed` supertrait is inaccessible.

### When to Seal Traits

Decision matrix:

| Scenario | Seal? | Rationale |
|----------|-------|-----------|
| Public library API | Yes | Future flexibility, prevent misuse |
| Internal trait | No | Not exposed outside crate |
| Extension trait | Yes | Control implementations |
| Core abstraction | No | Designed for user implementation |

### Example: Sealed Number Trait

From `complex` project principles:

```rust
mod sealed {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

pub trait Float: sealed::Sealed + Copy {
    fn sqrt(self) -> Self;
    fn abs(self) -> Self;
    // ... more operations
}

impl Float for f32 {
    fn sqrt(self) -> Self {
        f32::sqrt(self)
    }
    fn abs(self) -> Self {
        f32::abs(self)
    }
}

impl Float for f64 {
    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }
    fn abs(self) -> Self {
        f64::abs(self)
    }
}

// Users can't add Float for i32 or custom types
// Guarantees Float = f32 or f64
```

This enables library evolution—adding methods to `Float` won't break external code.

## Blanket Implementations at Scale

Blanket implementations provide trait implementations for all types matching a bound.

### The ToString Pattern

From standard library:

```rust
pub trait Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

// Blanket impl: all Display types get ToString
impl<T: Display + ?Sized> ToString for T {
    fn to_string(&self) -> String {
        format!("{}", self)
    }
}
```

This is powerful: implement one trait (`Display`), get another (`ToString`) for free.

### Binary Tree Iterator Blanket

```rust
pub trait Tree<T> {
    fn left(&self) -> Option<&Self>;
    fn right(&self) -> Option<&Self>;
    fn value(&self) -> &T;
}

// Blanket: all Tree types get traversal
impl<T, Tr: Tree<T>> Tr {
    pub fn in_order(&self) -> impl Iterator<Item = &T> {
        let mut stack = Vec::new();
        let mut current = Some(self);

        std::iter::from_fn(move || {
            while let Some(node) = current {
                stack.push(node);
                current = node.left();
            }

            let node = stack.pop()?;
            current = node.right();
            Some(node.value())
        })
    }
}
```

### Coherence and Overlap

Blanket impls must not overlap:

```rust
// ERROR: Overlapping implementations
impl<T: Clone> MyTrait for T {}
impl<T: Copy> MyTrait for T {}  // Copy extends Clone - overlap!

// OK: Disjoint bounds
impl<T: Clone> MyTrait for Vec<T> {}
impl<T: Copy> MyTrait for &[T] {}  // Different types
```

### Generic From/Into Conversions

From `queue` conversion patterns:

```rust
impl<T> From<Vec<T>> for Queue<T> {
    fn from(vec: Vec<T>) -> Self {
        Queue {
            older: vec,
            younger: Vec::new(),
        }
    }
}

// Blanket impl in std gives us Into for free:
// impl<T, U> Into<U> for T where U: From<T>

let vec = vec![1, 2, 3];
let queue: Queue<_> = vec.into();  // Uses Into from From
```

## Trait Objects and Dynamic Dispatch

Trait objects enable runtime polymorphism at the cost of performance.

### Static vs Dynamic Dispatch

```rust
// Static dispatch - monomorphization
fn process_static<T: Display>(item: &T) {
    println!("{}", item);
}
// Each type gets its own compiled function
// No runtime overhead, but code bloat

// Dynamic dispatch - trait object
fn process_dynamic(item: &dyn Display) {
    println!("{}", item);
}
// One function, vtable lookup at runtime
// Small overhead, reduced binary size
```

### Object Safety Requirements

Not all traits can be trait objects:

```rust
// Object-safe: can be trait object
pub trait Drawable {
    fn draw(&self);  // Takes &self
}

// NOT object-safe: can't be trait object
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;

    fn collect<B>(self) -> B  // Takes self by value - not allowed
    where
        Self: Sized;
}
```

**Object-safe rules**:
1. No methods with `Self: Sized` bound
2. No associated functions (no `self` parameter)
3. No generic methods
4. Return type can't be `Self`
5. No associated constants

### Performance Implications

From benchmarking trait objects:

```rust
use criterion::{black_box, Criterion};

trait Process {
    fn process(&self, x: i32) -> i32;
}

struct Doubler;
impl Process for Doubler {
    fn process(&self, x: i32) -> i32 {
        x * 2
    }
}

fn bench_static(c: &mut Criterion) {
    let doubler = Doubler;
    c.bench_function("static dispatch", |b| {
        b.iter(|| black_box(doubler.process(black_box(42))))
    });
}

fn bench_dynamic(c: &mut Criterion) {
    let doubler: &dyn Process = &Doubler;
    c.bench_function("dynamic dispatch", |b| {
        b.iter(|| black_box(doubler.process(black_box(42))))
    });
}

// Typical results:
// static dispatch:  1.2 ns
// dynamic dispatch: 2.8 ns
// ~2.3x overhead from vtable indirection
```

**When to use trait objects**:
- Collections of heterogeneous types
- Plugin systems
- When binary size matters more than max performance
- Abstractions across FFI boundaries

### Thin vs Fat Pointers

```rust
// Thin pointer: 8 bytes (64-bit)
let x: &i32 = &42;

// Fat pointer: 16 bytes (pointer + vtable)
let y: &dyn Display = &42;

// Fat pointer: 16 bytes (pointer + length)
let z: &[i32] = &[1, 2, 3];
```

Fat pointers affect struct sizes and cache efficiency.

## Advanced Trait Patterns

### The Newtype Pattern for Trait Coherence

```rust
// Can't impl foreign trait on foreign type
// impl Display for Vec<i32> {}  // ERROR: orphan rule

// Newtype wrapper
pub struct DisplayableVec(Vec<i32>);

impl Display for DisplayableVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, val) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", val)?;
        }
        write!(f, "]")
    }
}

// Deref for transparent access
impl Deref for DisplayableVec {
    type Target = Vec<i32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
```

### Associated Types vs Generic Parameters

```rust
// Associated type - one implementation per type
trait Container {
    type Item;
    fn get(&self, index: usize) -> Option<&Self::Item>;
}

impl Container for Vec<i32> {
    type Item = i32;  // Only one Item type possible
    fn get(&self, index: usize) -> Option<&i32> {
        self.get(index)
    }
}

// Generic parameter - multiple implementations possible
trait Convert<T> {
    fn convert(&self) -> T;
}

impl Convert<String> for i32 {
    fn convert(&self) -> String {
        self.to_string()
    }
}

impl Convert<f64> for i32 {
    fn convert(&self) -> f64 {
        *self as f64
    }
}
```

**Guideline**: Use associated types when there's one obvious output type. Use generic parameters for multiple possible conversions.

### Supertraits for Refinement

```rust
// From binary-tree principles
pub trait Tree {
    type Item;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// Refine with additional capabilities
pub trait SortedTree: Tree
where
    Self::Item: Ord,
{
    fn min(&self) -> Option<&Self::Item>;
    fn max(&self) -> Option<&Self::Item>;
    fn contains(&self, value: &Self::Item) -> bool;
}
```

## Decision Framework

| Pattern | Use When | Complexity | Example |
|---------|----------|------------|---------|
| Marker trait | Compile-time properties | Low | Send, Sync, Relocatable |
| Sealed trait | Library API control | Medium | Extension traits |
| Blanket impl | Provide derived behavior | High (coherence) | ToString from Display |
| Trait object | Runtime polymorphism | Medium | Plugin systems |
| Associated type | Single output type | Low | Iterator::Item |
| Generic parameter | Multiple implementations | Medium | From<T> |

## Practical Guidelines

1. **Prefer Static Dispatch**: Use generics unless you need heterogeneous collections
2. **Seal Extension Traits**: Keep control of your API surface
3. **Document Object Safety**: Clearly mark traits as object-safe or not
4. **Use Markers Sparingly**: Only for properties the type system can't otherwise express
5. **Blanket Carefully**: Ensure implementations don't overlap

## Common Pitfalls

### Over-Abstraction

```rust
// BAD: Trait where concrete type suffices
trait Number {
    fn add(&self, other: &Self) -> Self;
}

// GOOD: Just use concrete types or std traits
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Breaking Object Safety Accidentally

```rust
pub trait Plugin {
    fn execute(&self);

    // Oops! Added generic method - trait no longer object-safe
    fn configure<T: Config>(&self, config: T);  // BREAKS object safety
}

// Fix: Move generic to associated type
pub trait Plugin {
    type Config: Config;

    fn execute(&self);
    fn configure(&self, config: &Self::Config);  // Object-safe again
}
```

## Summary

Advanced trait patterns enable powerful abstractions:
- **Marker traits** encode compile-time properties
- **Sealed traits** control API evolution
- **Blanket implementations** provide automatic derived behavior
- **Trait objects** enable runtime polymorphism with measured overhead

Master these patterns to design APIs that are both flexible and performant. The binary-tree, complex, and gap-buffer projects demonstrate these principles in real systems.

## Further Reading

- Chapter 10.1: Generics interact deeply with trait bounds
- Chapter 10.4: Trade-offs between abstraction and performance
- Chapter 4: Trait-based architecture from the ground up
