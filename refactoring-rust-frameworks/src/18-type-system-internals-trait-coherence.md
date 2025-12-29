# Type System Internals & Trait Coherence Rules

**Target Audience**: AI agents implementing generic code, analyzing trait systems
**Prerequisites**: Understanding of traits, generics, and lifetimes
**Scope**: Trait resolution, coherence, monomorphization, type inference

---

## 1. Trait Resolution & Monomorphization

### Static Dispatch vs Dynamic Dispatch

Rust provides two distinct mechanisms for polymorphism: static dispatch through generics and dynamic dispatch through trait objects.

**Static dispatch** generates specialized code for each concrete type at compile time:

```rust
fn process<T: Display>(value: T) {
    println!("{}", value);
}

// Compiler generates two separate functions:
// process_i32(value: i32) { println!("{}", value); }
// process_String(value: String) { println!("{}", value); }

process(42);           // Calls monomorphized process_i32
process("hello".to_string()); // Calls monomorphized process_String
```

**Dynamic dispatch** uses runtime indirection through vtables:

```rust
fn process_dyn(value: &dyn Display) {
    println!("{}", value);
}

// Single function, runtime vtable lookup
process_dyn(&42);
process_dyn(&"hello");
```

### Monomorphization Process

Monomorphization transforms generic code into concrete implementations during compilation. The compiler:

1. **Collects all instantiations** of generic functions/types across the codebase
2. **Generates specialized versions** for each unique type parameter combination
3. **Optimizes each version** independently (inlining, constant folding)
4. **Links** all instantiations into the final binary

```rust
fn max<T: Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

let x = max(10, 20);        // Generates max::<i32>
let y = max(3.14, 2.71);    // Generates max::<f64>
let z = max("a", "b");      // Generates max::<&str>
```

The compiler produces three distinct functions, each optimized for its specific type. This eliminates runtime type checks but increases binary size.

## 2. Coherence Rules in Depth

### Coherence Fundamentals

**Coherence** ensures that for any given type and trait, there exists **at most one implementation**. This prevents ambiguity in trait resolution:

```rust
trait Greet {
    fn greet(&self) -> String;
}

struct Person;

// OK: One implementation
impl Greet for Person {
    fn greet(&self) -> String { "Hello".to_string() }
}

// ERROR: Conflicting implementation
// impl Greet for Person {
//     fn greet(&self) -> String { "Hi".to_string() }
// }
```

### The Orphan Rule

The **orphan rule** states: you can implement a trait for a type **only if** either the trait or the type is defined in your current crate. This prevents external crates from conflicting implementations.

### Workarounds: The Newtype Pattern

When coherence blocks valid code, use the **newtype pattern**:

```rust
// Want: impl MyTrait for Vec<i32>
// Problem: Both trait and type are external

// Solution: Wrap in local type
struct MyVec(Vec<i32>);

impl MyTrait for MyVec {
    // Now allowed: MyVec is local
}

// Transparent access with Deref
impl std::ops::Deref for MyVec {
    type Target = Vec<i32>;
    fn deref(&self) -> &Vec<i32> { &self.0 }
}
```

## 3. Trait Bounds & Constraints

### Trait Bound Syntax

```rust
// Inline bounds
fn process<T: Clone + Display>(x: T) { /* ... */ }

// Where clause (preferred for complex bounds)
fn process<T>(x: T)
where
    T: Clone + Display,
{
    /* ... */
}
```

### Associated Types in Traits

Associated types bind types to trait implementations:

```rust
trait Iterator {
    type Item;  // Associated type
    fn next(&mut self) -> Option<Self::Item>;
}

// Compare to generic version (could have multiple impls):
trait IteratorGeneric<Item> {
    fn next(&mut self) -> Option<Item>;
}
```

**Key difference**: Associated types are **output types** (determined by implementation), while generic parameters are **input types** (specified by caller).

## 4. Higher-Ranked Trait Bounds (HRTB)

### The `for<'a>` Syntax

HRTBs express "for all lifetimes" constraints:

```rust
// Without HRTB: specific lifetime
fn call_with_ref<'a, F>(f: F, value: &'a str)
where
    F: Fn(&'a str) -> usize,
{
    f(value);
}

// With HRTB: works for any lifetime
fn call_with_any_ref<F>(f: F, value: &str)
where
    F: for<'a> Fn(&'a str) -> usize,  // For ANY lifetime 'a
{
    f(value);
}
```

## 5. Trait Objects & Dynamic Dispatch

### Trait Object Safety Rules

A trait is **object-safe** if:
1. All methods have `&self` or `&mut self` receiver (not `self` by value)
2. No generic methods
3. No associated functions (no `Self` in return position)
4. Trait doesn't require `Sized`

```rust
// Object-safe
trait Draw {
    fn draw(&self);  // OK: &self receiver
}

// NOT object-safe
trait Clone {
    fn clone(&self) -> Self;  // ERROR: Self in return
}
```

## 6. Type Inference & The Solver

### Constraint Generation

The type checker generates constraints during compilation:

```rust
fn example() {
    let x = vec![1, 2, 3];  // Constraint: x: Vec<T> where T: ?
    let y = x[0];           // Constraint: T = i32 (from literal)
    // Solver: x: Vec<i32>, y: i32
}
```

### Ambiguity and Disambiguation

When inference fails, use turbofish (`::<>`) or type annotations:

```rust
// Ambiguous
let v = Vec::new();  // ERROR: Can't infer T

// Disambiguate with turbofish
let v = Vec::<i32>::new();

// Or type annotation
let v: Vec<i32> = Vec::new();
```

## 7. Performance Implications

### Monomorphization Explosion

```rust
// Each combination generates separate code
fn process<T: Clone, U: Display>(t: T, u: U) { /* ... */ }

process(1i32, "a");      // Instantiation 1: <i32, &str>
process(1i32, 2i32);     // Instantiation 2: <i32, i32>
process("a", "b");       // Instantiation 3: <&str, &str>
// Result: 3× code size
```

### Trait Object Performance

```rust
// Static dispatch: ~1ns call overhead (inlined)
fn static_dispatch<T: Trait>(x: &T) {
    x.method();  // Direct call, can inline
}

// Dynamic dispatch: ~2-5ns call overhead (vtable lookup)
fn dynamic_dispatch(x: &dyn Trait) {
    x.method();  // Indirect call through vtable
}
```

## 8. AI Agent Type System Analysis

### Checking Trait Bound Satisfaction

When analyzing code, verify:

```rust
fn needs_bounds<T: Clone + Display>(x: T) { /* ... */ }

// AI check: Does MyType satisfy bounds?
struct MyType;
// impl Clone for MyType { /* ... */ }  ✓
// impl Display for MyType { /* ... */ } ✗ Missing!
```

### Detecting Coherence Violations

```rust
// AI should flag:
impl MyTrait for Vec<i32> {}  // ⚠️ Orphan rule violation
impl<T> MyTrait for T {}
impl MyTrait for String {}    // ⚠️ Overlaps with above blanket impl
```

## Conclusion

Rust's type system achieves zero-cost abstractions through sophisticated compile-time analysis. Understanding monomorphization, coherence rules, and trait resolution enables writing expressive, performant code. For AI agents, recognizing inference failures, coherence violations, and bound requirements is essential for effective analysis.

**Key Takeaways:**
1. Monomorphization provides static dispatch at compile-time cost
2. Coherence prevents ambiguous trait implementations
3. Associated types are output types, generics are input types
4. Trait objects enable dynamic dispatch with vtable overhead
5. HRTBs express "for all" constraints on lifetimes
