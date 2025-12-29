# Generic Programming for Flexibility

Generic programming in Rust is more than code reuse—it's an architectural strategy for building flexible, composable systems. The journey from concrete types to generics, exemplified by our `queue` and `generic-queue` projects, reveals how type parameters transform rigid implementations into adaptable frameworks. Understanding generics means understanding how Rust balances compile-time flexibility with runtime performance.

## Generic Types as Architectural Templates

The evolution from `Queue` to `Queue<T>` demonstrates generics as architectural abstraction. The original concrete implementation:

```rust
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last
    younger: Vec<char>  // younger elements, youngest last
}

impl Queue {
    pub fn push(&mut self, c: char) {
        self.younger.push(c);
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }
            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }
        self.older.pop()
    }
}
```

This implementation is correct, efficient, and inflexible. It works only for `char`. To support integers, you'd need to duplicate the entire implementation. This violates DRY (Don't Repeat Yourself) and creates maintenance burden—bug fixes must be replicated across implementations.

The generic version transforms the type into a template:

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}

impl<T> Queue<T> {
    pub fn push(&mut self, t: T) {
        self.younger.push(t);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }
            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }
        self.older.pop()
    }
}
```

The algorithm is identical—only the type has been abstracted. This single implementation now works for any type:

```rust
let mut char_queue = Queue::<char>::new();
char_queue.push('A');

let mut int_queue = Queue::<i32>::new();
int_queue.push(42);

let mut string_queue = Queue::<String>::new();
string_queue.push("hello".to_string());
```

This is architectural leverage: write once, use everywhere.

## Type Parameters as Configuration Points

Type parameters are more than placeholders—they're configuration points that define how a type behaves. The `Complex<T>` type from our repository demonstrates this elegantly:

```rust
#[derive(Clone, Copy, Debug)]
struct Complex<T> {
    /// Real portion of the complex number
    re: T,
    /// Imaginary portion of the complex number
    im: T,
}
```

The `T` parameter configures the numeric type. But to enable arithmetic, `T` must support arithmetic operations. This is where trait bounds become architectural constraints:

```rust
use std::ops::Add;

impl<T> Add for Complex<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}
```

The `where T: Add<Output = T>` clause is an architectural declaration: "This implementation exists for any type `T` that supports addition." The type parameter becomes a contract—the generic code promises to work for any type meeting the contract.

This enables remarkable flexibility:

```rust
let a = Complex { re: 1.0_f64, im: 2.0 };
let b = Complex { re: 3.0, im: 4.0 };
let c = a + b;  // Complex<f64>

let a = Complex { re: 1_i32, im: 2 };
let b = Complex { re: 3, im: 4 };
let c = a + b;  // Complex<i32>
```

Same code, different types, zero runtime overhead.

## The Complexity of Generic Multiplication

Multiplication of complex numbers reveals the nuance of trait bounds:

```rust
use std::ops::Mul;

impl<T> Mul for Complex<T>
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Complex {
            re: self.re.clone() * rhs.re.clone()
                - (self.im.clone() * rhs.im.clone()),
            im: self.im * rhs.re + self.re * rhs.im,
        }
    }
}
```

The mathematical formula `(a + bi)(c + di) = (ac - bd) + (ad + bc)i` requires multiple operations. Each intermediate value must be owned, requiring `Clone`. The trait bound grows to match the implementation's needs:

- `Clone`: Needed to reuse values
- `Add<Output = T>`: Addition produces T
- `Sub<Output = T>`: Subtraction produces T
- `Mul<Output = T>`: Multiplication produces T

These bounds form an interface definition. Any type implementing these traits gains complex number support. This is composition at the type level.

## Monomorphization: The Performance Trade-Off

Rust's generics are zero-cost at runtime through **monomorphization**—the compiler generates specialized code for each concrete type:

```rust
// Generic code:
fn print_queue<T: std::fmt::Debug>(q: &Queue<T>) {
    // ...
}

// Called with different types:
print_queue(&char_queue);    // Generates print_queue_char
print_queue(&int_queue);     // Generates print_queue_i32
print_queue(&string_queue);  // Generates print_queue_String
```

Each call produces a separate function in the compiled binary. This has profound implications:

### Benefits:
1. **Zero Runtime Overhead**: No virtual dispatch, no indirection
2. **Optimal Inlining**: Compiler can inline type-specific code
3. **Type-Specific Optimizations**: Each monomorphized function can be optimized independently

### Costs:
1. **Binary Size**: Each instantiation adds code to the binary
2. **Compile Time**: Compiler must generate and optimize each variant
3. **Code Cache**: More code can reduce instruction cache effectiveness

For `Queue<T>`, if you use it with 10 different types, you get 10 copies of the queue implementation in your binary. For small types, this is negligible. For large generic types used with many types, it can bloat binaries significantly.

## When to Use Generics vs Trait Objects

The generics vs trait objects decision is architectural, not just technical:

### Use Generics When:

1. **Performance Critical**: Need zero-cost abstraction
2. **Type Known at Compile Time**: Caller determines concrete type
3. **Few Concrete Types**: Limited monomorphization explosion
4. **Need Associated Types**: Generic algorithms with type-level configuration

```rust
// Generic: Fast, monomorphized
fn process_generic<T: Iterator>(iter: T) {
    for item in iter {
        // Specialized for this iterator type
    }
}
```

### Use Trait Objects When:

1. **Heterogeneous Collections**: Storing different types together
2. **Plugin Architectures**: Unknown types determined at runtime
3. **Binary Size Matters**: Avoid monomorphization explosion
4. **Runtime Polymorphism**: Type not known until runtime

```rust
// Trait object: Dynamic dispatch, single implementation
fn process_dynamic(iter: &mut dyn Iterator<Item = i32>) {
    for item in iter {
        // Single code path, runtime dispatch
    }
}
```

The `Queue<T>` naturally uses generics because:
- Each queue stores a single type
- Performance matters for data structures
- Type is known when queue is created

But a message queue handling different message types might use trait objects:

```rust
trait Message {
    fn handle(&self);
}

struct MessageQueue {
    messages: Vec<Box<dyn Message>>,  // Heterogeneous collection
}
```

## Generic Constraints and API Design

Trait bounds define your API surface. The difference between:

```rust
impl<T> Queue<T> {
    pub fn new() -> Self { /* ... */ }
}
```

and

```rust
impl<T: Clone> Queue<T> {
    pub fn new() -> Self { /* ... */ }
}
```

is architectural. The first works for any type; the second only for cloneable types. Over-constraining limits reusability; under-constraining prevents useful operations.

Our `Queue<T>` has no constraints on `new()`, `push()`, or `is_empty()`:

```rust
impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { older: Vec::new(), younger: Vec::new() }
    }

    pub fn push(&mut self, t: T) {
        self.younger.push(t);
    }

    pub fn is_empty(&self) -> bool {
        self.older.is_empty() && self.younger.is_empty()
    }
}
```

These operations work for *any* type `T`, even non-clonable, non-comparable types. This is maximally reusable. Only operations that genuinely require specific capabilities add bounds.

## Generic Associated Types and Complex APIs

The `Complex<T>` type demonstrates how generics propagate through APIs. Every trait implementation is generic over `T`:

```rust
impl<T: PartialEq> PartialEq for Complex<T> {
    fn eq(&self, other: &Complex<T>) -> bool {
        self.re == other.re && self.im == other.im
    }
}
```

This creates a coherent API: if `T` supports equality, so does `Complex<T>`. The type system ensures consistency across the entire implementation.

But bounds must be minimal. Using:

```rust
impl<T: PartialEq + PartialOrd> PartialEq for Complex<T> {
    // ...
}
```

would unnecessarily require `T: PartialOrd` for equality, limiting usability without benefit.

## Turbofish Syntax and Type Inference

Rust's type inference usually eliminates the need for explicit type parameters:

```rust
let mut q = Queue::new();
q.push("CAD");  // Inferred as Queue<&str>
q.push("BTC");  // Consistent with first push
```

But sometimes inference is ambiguous or undesirable. The turbofish syntax (`::<T>`) provides explicit control:

```rust
let q = Queue::<char>::new();  // Explicit type
let q = Queue::new();           // Inferred later

// Ambiguous without turbofish:
let parsed = "42".parse::<i32>()?;  // Which numeric type?
```

This is particularly useful in API design:

```rust
// User can specify or let it be inferred
pub fn from_iter<I>(iter: I) -> Queue<T>
where
    I: IntoIterator<Item = T>
{
    // ...
}

// Explicit:
let q = Queue::from_iter::<Vec<i32>>(vec![1, 2, 3]);

// Inferred:
let q: Queue<i32> = Queue::from_iter(vec![1, 2, 3]);
```

## Generic Programming Patterns

### The NewType Pattern with Generics

Generics combine with newtypes for powerful abstractions:

```rust
struct Validated<T> {
    value: T,
}

impl<T> Validated<T> {
    pub fn new(value: T, validate: impl Fn(&T) -> bool) -> Option<Self> {
        if validate(&value) {
            Some(Validated { value })
        } else {
            None
        }
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}
```

This creates a validated wrapper for *any* type, with validation logic provided at construction.

### The Iterator Pattern

The `binary-tree` project's iterator implementation shows generics at scale:

```rust
pub struct InorderIter<'a, T> {
    stack: Vec<&'a TreeNode<T>>,
    current: Option<&'a TreeNode<T>>,
}

impl<'a, T> Iterator for InorderIter<'a, T> {
    type Item = &'a T;
    // ...
}
```

The iterator is generic over both the element type `T` and the lifetime `'a`. This composition of generics creates flexible, reusable iteration logic.

## Performance Considerations in Practice

For `Queue<T>`, generic performance is excellent because:

1. **Simple Operations**: Push/pop compile to simple vector operations
2. **No Allocation**: Generic code doesn't add allocations
3. **Inlining**: Small methods like `is_empty()` inline completely

But complex generic algorithms may face:

- **Compile Time**: Deep generic hierarchies slow compilation
- **Binary Bloat**: Many instantiations increase binary size
- **Optimization Challenges**: Compiler must optimize each variant independently

The `Complex<T>` multiplication shows this. For `Complex<f64>`, the compiler generates optimized floating-point code. For `Complex<BigInt>`, it generates arbitrary-precision arithmetic. Same source, completely different machine code.

## Decision Framework: Generics in Architecture

When designing a generic type, ask:

1. **Does the algorithm truly not depend on the concrete type?**
   - Yes: Use generics (e.g., `Queue<T>`)
   - No: Use concrete types

2. **How many concrete types will be used?**
   - Few (<10): Generics are fine
   - Many: Consider trait objects or consolidation

3. **Is performance critical?**
   - Yes: Generics enable monomorphization
   - No: Trait objects may be simpler

4. **What constraints does the algorithm require?**
   - Minimal: Keep trait bounds minimal
   - Extensive: Consider if generics are appropriate

## Cross-References

Generic programming builds on:

- **Chapter 1: Newtypes** - Generic newtypes for validated types
- **Chapter 3: Traits** - Trait bounds define generic constraints
- **Chapter 6: Async** - Generic futures and async abstractions

## Conclusion

Generic programming in Rust is an architectural tool. The transformation from `Queue` to `Queue<T>` isn't just about reusability—it's about creating flexible, composable abstractions that maintain zero-cost performance guarantees.

Type parameters are configuration points, trait bounds are contracts, and monomorphization ensures that abstraction never compromises performance. By understanding these mechanisms, you can architect systems that are simultaneously flexible and fast, abstract and concrete, reusable and optimized.

The key is balance: use generics where they add genuine flexibility without undue complexity, constrain them minimally to maximize reusability, and let monomorphization handle the rest.
