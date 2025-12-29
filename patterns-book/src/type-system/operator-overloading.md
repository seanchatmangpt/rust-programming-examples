# Operator Overloading

## Pattern Name
**Operator Overloading for Domain Types**

## Context

You are building a Rust library that models mathematical, geometric, or domain-specific concepts (such as complex numbers, vectors, matrices, rational numbers, or physical quantities). Your users need to perform natural algebraic operations on these types using familiar syntax like `a + b`, `a * b`, or `-a`.

Rust's type system provides operator traits (`Add`, `Sub`, `Mul`, `Neg`, etc.) in the `std::ops` module that enable custom types to work with built-in operator syntax.

## Problem

**How do you make custom types work naturally with Rust's operators while maintaining type safety and composability?**

Domain-specific types need to support arithmetic operations in a way that:
- Feels natural to users (using `+`, `-`, `*` instead of verbose method names)
- Maintains type safety (prevents nonsensical operations)
- Works with generic code
- Composes with other traits (like `Clone`, `Copy`)
- Supports both owned and borrowed values where appropriate

## Forces

- **Ergonomics vs Explicitness**: Operators are concise and intuitive, but can hide complexity or expensive operations
- **Type Safety**: Operations should only be defined when they make semantic sense
- **Generic Constraints**: Implementing operators for generic types requires careful trait bounds
- **Ownership Semantics**: Operators consume their operands by default; must decide when to require `Copy` or accept references
- **Consistency**: Operators should follow mathematical conventions and user expectations
- **Performance**: Operator implementations should avoid unnecessary cloning when possible
- **Composability**: Operator implementations should work with Rust's type system (generics, associated types)

## Solution

**Implement the appropriate operator traits from `std::ops` for your custom types, using trait bounds to constrain generic implementations.**

### Core Technique

From `/home/user/rust-programming-examples/complex/src/lib.rs`:

```rust
use std::ops::Add;

#[derive(Clone, Copy, Debug)]
struct Complex<T> {
    /// Real portion of the complex number
    re: T,
    /// Imaginary portion of the complex number
    im: T,
}

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

### Key Elements

1. **Import the operator trait**: `use std::ops::Add;`
2. **Specify generic constraints**: `where T: Add<Output = T>` ensures the component type supports addition
3. **Define the associated type**: `type Output = Self;` specifies what the operation produces
4. **Implement the operation**: Perform the domain-specific computation

### Multiple Operators

```rust
use std::ops::{Sub, Mul};

impl<T> Sub for Complex<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Complex {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl<T> Mul for Complex<T>
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        // (a + bi)(c + di) = (ac - bd) + (ad + bc)i
        Complex {
            re: self.re.clone() * rhs.re.clone()
                - (self.im.clone() * rhs.im.clone()),
            im: self.im * rhs.re + self.re * rhs.im,
        }
    }
}
```

**Note**: Multiplication requires `Clone` because we need to use `self.re` and `self.im` multiple times.

### Unary Operators

```rust
use std::ops::Neg;

impl<T> Neg for Complex<T>
where
    T: Neg<Output = T>,
{
    type Output = Complex<T>;
    fn neg(self) -> Complex<T> {
        Complex {
            re: -self.re,
            im: -self.im,
        }
    }
}
```

### Compound Assignment Operators

```rust
use std::ops::AddAssign;

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

Compound assignment operators (`+=`, `-=`, etc.) take `&mut self` and don't return a value, enabling efficient in-place modification.

### Advanced: Heterogeneous Operations

For maximum flexibility, allow operations between different generic types:

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

This allows `Complex<i32> + Complex<f64>` to produce `Complex<f64>` (following standard type promotion rules).

### Usage Example

```rust
let mut z = Complex { re: 1, im: 2 };
let c = Complex { re: 3, im: 4 };

z = z * z + c;  // Natural mathematical syntax!
```

## Resulting Context

### Benefits

- **Natural Syntax**: Users can write mathematical expressions that closely resemble mathematical notation
- **Type Safety**: The type system ensures operations are only available when semantically valid
- **Generic Code**: Operator implementations work with any type that satisfies the constraints
- **Compiler Optimization**: Operators are just trait methods, so they can be inlined and optimized
- **Composability**: Works seamlessly with Rust's type system, including generic functions and trait bounds

### Liabilities

- **Implicit Costs**: Operators can hide expensive operations (multiplication of large matrices looks like `a * b`)
- **Ownership Rules**: Binary operators consume their operands by default, requiring `Copy` or cloning
- **Trait Bound Complexity**: Generic implementations can accumulate many trait bounds
- **Limited Operator Set**: Rust only allows overloading specific operators, not creating new ones
- **No Chaining**: Can't directly implement chained comparisons like `a < b < c`

### Common Mistakes

1. **Forgetting Clone Bounds**: If you need to use a field multiple times, add `T: Clone` to bounds
2. **Wrong Associated Type**: Ensure `Output` type makes semantic sense for your operation
3. **Inconsistent Semantics**: Follow mathematical conventions (e.g., addition should be commutative if mathematically it is)
4. **Missing Compound Assignment**: If you implement `Add`, consider also implementing `AddAssign` for efficiency

## Related Patterns

- **Display Formatting**: Often used together to provide human-readable output for types with operator overloading
- **PartialEq and Eq**: Equality checking complements arithmetic operations
- **From/Into Conversions**: Enable type conversions that work seamlessly with operators
- **Newtype Wrapper**: Restrict which operators are available by wrapping primitives

## Known Uses

- **num_complex**: Production-quality complex number library using this pattern
- **nalgebra**: Linear algebra library with extensive operator overloading for vectors and matrices
- **rust-decimal**: Decimal number library implementing all arithmetic operators
- **euclid**: Geometry library with operators for points, vectors, and transformations
- **Standard Library**: `std::ops` documentation provides canonical examples

## Example: Full Implementation

From the actual codebase (`/home/user/rust-programming-examples/complex/src/lib.rs`):

```rust
#[derive(Clone, Copy, Debug)]
struct Complex<T> {
    re: T,
    im: T,
}

use std::ops::{Add, Sub, Mul, Neg, AddAssign};

impl<T: Add<Output = T>> Add for Complex<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Complex { re: self.re + rhs.re, im: self.im + rhs.im }
    }
}

impl<T: Sub<Output = T>> Sub for Complex<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Complex { re: self.re - rhs.re, im: self.im - rhs.im }
    }
}

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

impl<T: Neg<Output = T>> Neg for Complex<T> {
    type Output = Complex<T>;
    fn neg(self) -> Complex<T> {
        Complex { re: -self.re, im: -self.im }
    }
}

impl<T: AddAssign<T>> AddAssign for Complex<T> {
    fn add_assign(&mut self, rhs: Complex<T>) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

#[test]
fn test_complex_arithmetic() {
    let x = Complex { re: 5, im: 2 };
    let y = Complex { re: 2, im: 5 };
    assert_eq!(x * y, Complex { re: 0, im: 29 });
}
```

## Guidelines

1. **Start Simple**: Begin with concrete types (like `Complex<i32>`), then generalize
2. **Follow Mathematics**: Honor mathematical conventions and properties (commutativity, associativity)
3. **Consider Copy**: If your type is small, derive `Copy` to make operators more ergonomic
4. **Document Complexity**: If an operation is expensive (O(n²)), document it
5. **Test Edge Cases**: Test with different types (integers, floats) and edge values
6. **Provide Examples**: Show idiomatic usage in documentation

## Why This Pattern Works in Rust

Rust's operator overloading is safer than C++'s because:
- **No Hidden Conversions**: Rust doesn't do implicit type conversions
- **Explicit Trait Bounds**: Generic constraints make requirements clear
- **Move Semantics**: Default move semantics prevent accidental copies
- **Type Safety**: Associated types ensure type consistency

This pattern demonstrates Rust's philosophy of **zero-cost abstractions**: the abstraction (using `+` instead of `.add()`) has no runtime cost, while maintaining complete type safety.
