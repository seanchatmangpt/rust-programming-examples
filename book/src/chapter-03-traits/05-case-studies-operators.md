# Case Studies: Operator Overloading and Comparison

Real-world projects demonstrate how trait-based design principles translate into practical implementations. This chapter examines three projects from the repository—`complex`, `interval`, and `basic-router`—analyzing their architectural decisions, trade-offs, and design patterns. These case studies reveal how experienced Rust developers apply trait-based thinking to solve real problems.

## Case Study 1: Complex Numbers and Arithmetic Traits

The `complex` project provides multiple implementations of complex number arithmetic, each demonstrating different architectural approaches to operator overloading.

### Evolution from Specific to Generic

The project presents a progression showing how trait implementations evolve:

#### Stage 1: Non-Generic Implementation

```rust
impl Add for Complex<i32> {
    type Output = Complex<i32>;
    fn add(self, rhs: Self) -> Self {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}
```

This implementation only works for `Complex<i32>`. To support `Complex<f64>`, you'd need to duplicate the entire implementation—clearly not scalable.

**Architectural Issue**: Code duplication violates DRY principle and makes maintenance difficult.

#### Stage 2: Somewhat Generic Implementation

```rust
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

Now the implementation works for any type `T` that supports addition. This single implementation covers `Complex<i32>`, `Complex<f64>`, `Complex<i64>`, and any other numeric type.

**Architectural Win**: Generic implementation reuses code across all compatible types through monomorphization.

#### Stage 3: Very Generic Implementation

```rust
impl<L, R> Add<Complex<R>> for Complex<L>
where
    L: Add<R>,
{
    type Output = Complex<L::Output>;
    fn add(self, rhs: Complex<R>) -> Self::Output {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}
```

This implementation allows adding complex numbers with different component types:

```rust
let a = Complex { re: 1i32, im: 2i32 };
let b = Complex { re: 3.0f64, im: 4.0f64 };
let result: Complex<f64> = a + b;  // Automatic type promotion
```

**Architectural Decision**: This provides maximum flexibility but increases complexity. The output type is determined by the component types' `Add` implementation, leveraging Rust's type inference.

### Architectural Lesson: Progressive Generalization

The progression demonstrates a key principle: **start specific, generalize as needed**. The non-generic version proves the concept, the somewhat-generic version handles most use cases, and the very-generic version addresses edge cases.

### Multiplication: Managing Complexity

Complex number multiplication requires multiple operations:

```rust
impl<T> Mul for Complex<T>
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Complex {
            // (a + bi)(c + di) = (ac - bd) + (ad + bc)i
            re: self.re.clone() * rhs.re.clone()
                - (self.im.clone() * rhs.im.clone()),
            im: self.im * rhs.re + self.re * rhs.im,
        }
    }
}
```

**Architectural Observations**:

1. **Trait Bound Composition**: Requires four traits (`Clone`, `Add`, `Sub`, `Mul`) composed through intersection
2. **Clone Requirement**: Necessary because values are used multiple times in the formula
3. **Ownership Semantics**: The method consumes `self` and `rhs`, returning a new value—appropriate for numeric types

**Design Alternative**: Could implement `Mul<&Complex<T>>` to work with references:

```rust
impl<T> Mul<&Complex<T>> for Complex<T>
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    type Output = Complex<T>;
    fn mul(self, rhs: &Complex<T>) -> Complex<T> {
        Complex {
            re: self.re.clone() * rhs.re.clone()
                - (self.im.clone() * rhs.im.clone()),
            im: self.im * rhs.re.clone() + self.re * rhs.im.clone(),
        }
    }
}
```

This allows `a * &b`, useful when `b` needs to be reused. The choice depends on expected usage patterns.

### Compound Assignment: Building on Fundamentals

```rust
impl<T> AddAssign for Complex<T>
where
    T: AddAssign<T>,
{
    fn add_assign(&mut self, rhs: Complex<T>) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}
```

**Architectural Pattern**: `AddAssign` mirrors `Add` but operates in-place, avoiding allocation. Notice it doesn't require `Add`—the traits are independent. However, conventionally, types implementing `AddAssign` also implement `Add` to maintain user expectations.

### Equality and Comparison

```rust
impl<T: PartialEq> PartialEq for Complex<T> {
    fn eq(&self, other: &Complex<T>) -> bool {
        self.re == other.re && self.im == other.im
    }
}

impl<T: Eq> Eq for Complex<T> {}
```

**Architectural Decision**: `PartialEq` is implemented manually (component-wise equality), while `Eq` is a marker trait with no methods—it just asserts that equality is reflexive, symmetric, and transitive.

**Why Both?** Complex numbers over `f64` implement `PartialEq` but not `Eq` because `NaN != NaN`. Complex numbers over `i32` implement both because integer equality is total.

## Case Study 2: Intervals and Partial Ordering

The `interval` project demonstrates a sophisticated use of `PartialOrd`, showing how to implement ordering for types where not all values are comparable.

### The Problem: Overlapping Intervals

```rust
#[derive(Debug, PartialEq)]
struct Interval<T> {
    lower: T,  // inclusive
    upper: T,  // exclusive
}
```

Intervals `[10, 20)` and `[20, 30)` have a clear ordering relationship: the first is entirely less than the second. However, intervals `[10, 30)` and `[20, 40)` overlap—neither is clearly less than or greater than the other.

### Implementing PartialOrd

```rust
impl<T: PartialOrd> PartialOrd<Interval<T>> for Interval<T> {
    fn partial_cmp(&self, other: &Interval<T>) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.lower >= other.upper {
            Some(Ordering::Greater)  // self is entirely after other
        } else if self.upper <= other.lower {
            Some(Ordering::Less)  // self is entirely before other
        } else {
            None  // Intervals overlap - no ordering
        }
    }
}
```

**Architectural Brilliance**: Returning `None` for overlapping intervals correctly models the mathematical reality that not all intervals are comparable. This is a perfect use of `PartialOrd` rather than `Ord`.

### Why Not Ord?

The `Ord` trait requires total ordering—every pair of values must have a defined relationship. For intervals, this doesn't hold:

```rust
let left = Interval { lower: 10, upper: 30 };
let right = Interval { lower: 20, upper: 40 };

assert!(!(left < right));   // Not less than
assert!(!(left >= right));  // Not greater than or equal
// This violates Ord's requirement: a < b or a >= b must be true
```

**Architectural Lesson**: Choose `PartialOrd` when your type has a natural partial ordering. Don't force a total ordering where none exists—it leads to arbitrary decisions and confusing semantics.

### Testing Partial Ordering

```rust
#[test]
fn test() {
    assert!(Interval { lower: 10, upper: 20 } < Interval { lower: 20, upper: 40 });
    assert!(Interval { lower: 7, upper: 8 } >= Interval { lower: 0, upper: 1 });

    // Overlapping intervals aren't ordered
    let left = Interval { lower: 10, upper: 30 };
    let right = Interval { lower: 20, upper: 40 };
    assert!(!(left < right));
    assert!(!(left >= right));
}
```

The tests verify both the positive cases (non-overlapping intervals) and the negative cases (overlapping intervals have no ordering).

### Design Alternative: Custom Comparison

An alternative approach would define a custom comparison method:

```rust
impl<T: PartialOrd> Interval<T> {
    fn overlaps(&self, other: &Interval<T>) -> bool {
        !(self.lower >= other.upper || self.upper <= other.lower)
    }

    fn compare(&self, other: &Interval<T>) -> IntervalOrdering {
        if self == other {
            IntervalOrdering::Equal
        } else if self.overlaps(other) {
            IntervalOrdering::Overlapping
        } else if self.lower >= other.upper {
            IntervalOrdering::After
        } else {
            IntervalOrdering::Before
        }
    }
}

enum IntervalOrdering {
    Before,
    After,
    Equal,
    Overlapping,
}
```

This provides more semantic information than `Option<Ordering>`, but it doesn't integrate with Rust's comparison operators. The `PartialOrd` implementation is the idiomatic choice.

## Case Study 3: Basic Router and Function Traits

The `basic-router` project demonstrates trait objects and closure traits, showing how function traits enable flexible callback systems.

### The Architecture

```rust
type BoxedCallback = Box<dyn Fn(&Request) -> Response>;

struct BasicRouter {
    routes: HashMap<String, BoxedCallback>
}

impl BasicRouter {
    fn add_route<C>(&mut self, url: &str, callback: C)
    where
        C: Fn(&Request) -> Response + 'static
    {
        self.routes.insert(url.to_string(), Box::new(callback));
    }

    fn handle_request(&self, request: &Request) -> Response {
        match self.routes.get(&request.url) {
            None => not_found_response(),
            Some(callback) => callback(request)
        }
    }
}
```

**Architectural Analysis**:

1. **Trait Object Storage**: `Box<dyn Fn(&Request) -> Response>` enables storing different closure types in the same HashMap
2. **Generic Addition**: `add_route<C>` accepts any callable with the correct signature
3. **Lifetime Bound**: `'static` ensures callbacks don't reference temporary data
4. **Dynamic Dispatch**: Calling `callback(request)` uses vtable dispatch (see Section 3.3)

### Why Trait Objects?

Consider the usage:

```rust
let mut router = BasicRouter::new();
router.add_route("/", |_| get_form_response());
router.add_route("/gcd", |req| get_gcd_response(req));
```

These two closures have **different types**—they capture different environments and have different implementations. Without trait objects, the router couldn't store them together:

```rust
// This doesn't work - different types!
struct BadRouter<F1, F2> {
    route1: F1,
    route2: F2,
}
```

For a fixed number of routes known at compile time, you could use generics. But for a dynamic routing table, trait objects are necessary.

### Function Trait Hierarchy

Rust has three function traits, forming a hierarchy:

```rust
trait FnOnce {
    // Can be called once (moves captured values)
    fn call_once(self) -> Output;
}

trait FnMut: FnOnce {
    // Can be called multiple times (mutates captured values)
    fn call_mut(&mut self) -> Output;
}

trait Fn: FnMut {
    // Can be called multiple times (only borrows captured values)
    fn call(&self) -> Output;
}
```

The router uses `Fn` because it needs to call the callback multiple times without consuming it:

```rust
// First request
router.handle_request(&req1);
// Second request - needs same callback
router.handle_request(&req2);
```

If the router used `FnOnce`, callbacks would be consumed after the first use.

### Design Trade-off: Performance vs Flexibility

The trait object approach provides flexibility but incurs vtable overhead:

```rust
// Dynamic dispatch - flexible but slower
type BoxedCallback = Box<dyn Fn(&Request) -> Response>;

// Static dispatch alternative - faster but less flexible
struct StaticRouter<F>
where
    F: Fn(&Request) -> Response
{
    routes: HashMap<String, F>
}
```

The static version requires all callbacks to have the same type, severely limiting flexibility. For a routing system, the flexibility of trait objects is worth the minor performance cost.

### Memory Management

The `Box<dyn Fn>` combines several architectural concerns:

1. **Heap Allocation**: `Box` puts the closure on the heap
2. **Ownership**: The router owns the closures
3. **Type Erasure**: `dyn Fn` erases the concrete closure type
4. **Lifetime**: `'static` ensures no dangling references

This demonstrates how Rust's ownership system integrates with trait-based abstraction.

## Architectural Decision Summary

| Project | Trait Used | Decision | Rationale |
|---------|-----------|----------|-----------|
| **complex** | `Add`, `Mul`, `Neg` | Generic with trait bounds | Code reuse across numeric types |
| **complex** | `PartialEq` vs `Eq` | Both, conditionally | `Eq` only when `T: Eq` (not for `f64`) |
| **interval** | `PartialOrd` not `Ord` | Partial ordering | Models mathematical reality of overlaps |
| **basic-router** | `Fn` (trait object) | Dynamic dispatch | Enable heterogeneous callback storage |

## Common Patterns and Anti-Patterns

### Pattern: Trait Bound Progression

Start minimal, add constraints as needed:

```rust
// Stage 1: Works for any T
impl<T> Complex<T> {
    fn new(re: T, im: T) -> Self { ... }
}

// Stage 2: Add constraints for specific methods
impl<T: Add<Output = T>> Complex<T> {
    fn add(self, rhs: Self) -> Self { ... }
}

// Stage 3: More constraints for complex operations
impl<T: Clone + Add + Sub + Mul> Complex<T> {
    fn mul(self, rhs: Self) -> Self { ... }
}
```

### Anti-Pattern: Over-Constraining Type Definitions

```rust
// BAD: Constraints on type definition
struct Complex<T: Add + Sub + Mul> {
    re: T,
    im: T,
}

// GOOD: Constraints on implementations
struct Complex<T> {
    re: T,
    im: T,
}

impl<T: Add> Complex<T> { /* ... */ }
impl<T: Mul> Complex<T> { /* ... */ }
```

Over-constraining the type definition prevents `Complex<NonMultipliable>` from existing, even though it could support addition. Keep type definitions maximally generic.

### Pattern: Semantic Trait Implementation

```rust
// GOOD: PartialOrd models actual semantics
impl<T: PartialOrd> PartialOrd for Interval<T> {
    fn partial_cmp(&self, other: &Interval<T>) -> Option<Ordering> {
        // Returns None for overlapping intervals
    }
}

// BAD: Forced total ordering with arbitrary decision
impl<T: Ord> Ord for Interval<T> {
    fn cmp(&self, other: &Interval<T>) -> Ordering {
        // Arbitrarily compare lower bounds when overlapping
        // This creates confusing semantics!
    }
}
```

Choose traits that accurately represent your type's semantics.

## Key Takeaways

1. **Progressive generalization**: Start specific, generalize as proven useful
2. **Trait composition**: Complex requirements emerge from simple trait combinations
3. **Semantic accuracy**: Choose `PartialOrd` vs `Ord` based on mathematical properties
4. **Trait objects for heterogeneity**: Use when storing different types together
5. **Constraint placement**: Constrain implementations, not type definitions
6. **Function traits**: Choose `Fn`, `FnMut`, or `FnOnce` based on usage requirements

These case studies demonstrate that trait-based design isn't academic—it's practical architectural thinking that produces flexible, maintainable, high-performance code.

## Cross-References

- **Section 3.1**: Trait fundamentals underlying these implementations
- **Section 3.2**: Composition patterns used throughout the case studies
- **Section 3.3**: Dynamic dispatch in the router example
- **Chapter 2**: Generic types provide the foundation for these trait implementations
