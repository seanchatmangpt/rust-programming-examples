# Tutorial: Implementing Operator Overloading with Complex Numbers

## Introduction

In this tutorial, you'll learn how to implement operator overloading in Rust by building a `Complex` number type. You'll discover how Rust's trait system provides a type-safe, explicit approach to operator overloading that's quite different from Python's magic methods.

### What You'll Learn

- Defining generic structs with type parameters
- Implementing arithmetic operators via `std::ops` traits
- Understanding trait bounds and where clauses
- Implementing `Add`, `Sub`, `Mul`, and comparison operators
- Custom formatting with `Display` and `Debug`
- The difference between Rust traits and Python's magic methods

### Prerequisites

- Basic understanding of Rust structs and generics
- Familiarity with traits and trait implementations
- Knowledge of complex number arithmetic (basic)

## Complex Numbers: A Quick Refresher

A complex number has a real and imaginary part: `a + bi`

**Arithmetic rules:**
- Addition: `(a + bi) + (c + di) = (a+c) + (b+d)i`
- Subtraction: `(a + bi) - (c + di) = (a-c) + (b-d)i`
- Multiplication: `(a + bi) × (c + di) = (ac-bd) + (ad+bc)i`

## Step 1: Define the Complex Type

Let's start with a generic `Complex` type that can work with any numeric type:

```rust
#[derive(Clone, Copy, Debug)]
struct Complex<T> {
    /// Real portion of the complex number
    re: T,

    /// Imaginary portion of the complex number
    im: T,
}
```

**Key Points:**

- **Generic Type Parameter `T`**: Works with any numeric type (`i32`, `f64`, etc.)
- **Derive Macros**:
  - `Clone`: Allows copying the complex number
  - `Copy`: Enables pass-by-copy for types like `i32`, `f64`
  - `Debug`: Auto-generates debug formatting

**Python Comparison:**

In Python, you might define a class like this:

```python
class Complex:
    def __init__(self, re, im):
        self.re = re
        self.im = im
```

Python doesn't have generics built into the language (though type hints support `Generic[T]`), but Rust's generics are compile-time checked and generate specialized code for each type.

## Step 2: Implement Addition

To enable the `+` operator, we implement the `Add` trait:

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

**Understanding the Code:**

1. **`impl<T> Add for Complex<T>`**: Implement `Add` for any `Complex<T>`
2. **`where T: Add<Output = T>`**: Constraint - `T` must support addition that returns `T`
3. **`type Output = Self`**: Adding two complex numbers returns a complex number
4. **`fn add(self, rhs: Self)`**: Takes ownership of both operands (consumed by value)

**Using It:**

```rust
let a = Complex { re: 1, im: 2 };
let b = Complex { re: 3, im: 4 };
let c = a + b;  // Complex { re: 4, im: 6 }
```

**Python Comparison:**

```python
class Complex:
    def __add__(self, other):
        return Complex(self.re + other.re, self.im + other.im)
```

Differences:
- Python uses `__add__` magic method; Rust uses `Add` trait
- Rust requires explicit trait bounds (`T: Add`)
- Rust makes ownership explicit (`self` vs `&self`)

## Step 3: Implement Subtraction

Subtraction follows the same pattern:

```rust
use std::ops::Sub;

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
```

**Using It:**

```rust
let a = Complex { re: 5, im: 7 };
let b = Complex { re: 2, im: 3 };
let c = a - b;  // Complex { re: 3, im: 4 }
```

**Python Comparison:**

```python
def __sub__(self, other):
    return Complex(self.re - other.re, self.im - other.im)
```

## Step 4: Implement Multiplication

Multiplication is more complex (pun intended) because we need multiple operations:

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

**Understanding the Trait Bounds:**

- **`T: Clone`**: We need to use values multiple times
- **`T: Add<Output = T>`**: For adding products
- **`T: Sub<Output = T>`**: For subtracting products
- **`T: Mul<Output = T>`**: For the multiplication itself

**Why `clone()`?**

Rust moves values by default. Since we use `self.re` and `self.im` multiple times, we must clone them:

```rust
re: self.re.clone() * rhs.re.clone() - (self.im.clone() * rhs.im.clone())
//         ^clone                ^clone          ^clone          ^clone
```

The last uses don't need `clone()` because they're the final use (moved).

**Using It:**

```rust
let a = Complex { re: 2, im: 3 };
let b = Complex { re: 4, im: 5 };
let c = a * b;
// (2 + 3i) × (4 + 5i) = 8 + 10i + 12i + 15i²
//                      = 8 + 22i - 15
//                      = -7 + 22i
// c == Complex { re: -7, im: 22 }
```

**Python Comparison:**

```python
def __mul__(self, other):
    return Complex(
        self.re * other.re - self.im * other.im,
        self.im * other.re + self.re * other.im
    )
```

Python doesn't have the concept of "move" or "clone" - everything is references.

## Step 5: Chaining Operations

Because we've implemented these traits, we can now chain operations naturally:

```rust
let mut z = Complex { re: 1, im: 2 };
let c = Complex { re: 3, im: 4 };

z = z * z + c;
// First: z * z = (1 + 2i) × (1 + 2i) = -3 + 4i
// Then: -3 + 4i + (3 + 4i) = 0 + 8i
```

**Python Comparison:**

```python
z = Complex(1, 2)
c = Complex(3, 4)
z = z * z + c  # Same syntax!
```

The syntax is identical, but Rust enforces type safety at compile time.

## Step 6: Implement Equality

For comparing complex numbers, implement `PartialEq`:

```rust
impl<T: PartialEq> PartialEq for Complex<T> {
    fn eq(&self, other: &Complex<T>) -> bool {
        self.re == other.re && self.im == other.im
    }
}
```

**Key Points:**

- **Trait Bound**: `T: PartialEq` ensures we can compare the components
- **References**: `&self` and `&other` - we're just checking, not consuming
- **Logic**: Both parts must be equal

For types where equality is total (not partial), we can also implement `Eq`:

```rust
impl<T: Eq> Eq for Complex<T> {}
```

**Using It:**

```rust
let x = Complex { re: 5, im: 2 };
let y = Complex { re: 2, im: 5 };
let z = x * y;

assert_eq!(z, Complex { re: 0, im: 29 });
```

**Python Comparison:**

```python
def __eq__(self, other):
    return self.re == other.re and self.im == other.im
```

## Step 7: Custom Display Formatting

Let's make our complex numbers print nicely:

```rust
use std::fmt;

impl fmt::Display for Complex {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        let im_sign = if self.im < 0.0 { '-' } else { '+' };
        write!(dest, "{} {} {}i", self.re, im_sign, f64::abs(self.im))
    }
}
```

**Note**: This example uses a non-generic `Complex` with `f64` for simplicity.

**Using It:**

```rust
let z = Complex { re: -0.5, im: 0.866 };
println!("{}", z);  // Prints: -0.5 + 0.866i

let w = Complex { re: -0.5, im: -0.866 };
println!("{}", w);  // Prints: -0.5 - 0.866i
```

**Advanced: Alternate Formatting**

We can support polar format with the `{:#}` specifier:

```rust
impl fmt::Display for Complex {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        let (re, im) = (self.re, self.im);
        if dest.alternate() {
            // Polar form: magnitude ∠ angle
            let abs = f64::sqrt(re * re + im * im);
            let angle = f64::atan2(im, re) / std::f64::consts::PI * 180.0;
            write!(dest, "{} ∠ {}°", abs, angle)
        } else {
            // Rectangular form: a + bi
            let im_sign = if im < 0.0 { '-' } else { '+' };
            write!(dest, "{} {} {}i", re, im_sign, f64::abs(im))
        }
    }
}
```

**Using It:**

```rust
let z = Complex { re: 0.0, im: 2.0 };
println!("{}", z);    // Prints: 0 + 2i
println!("{:#}", z);  // Prints: 2 ∠ 90°
```

**Python Comparison:**

```python
def __str__(self):
    sign = '-' if self.im < 0 else '+'
    return f"{self.re} {sign} {abs(self.im)}i"

def __repr__(self):
    return f"Complex({self.re}, {self.im})"
```

Rust separates `Display` (user-facing) from `Debug` (programmer-facing).

## Step 8: Implement Negation

For unary minus, implement the `Neg` trait:

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

**Using It:**

```rust
let z = Complex { re: 3, im: 4 };
let w = -z;  // Complex { re: -3, im: -4 }
```

**Python Comparison:**

```python
def __neg__(self):
    return Complex(-self.re, -self.im)
```

## Step 9: Compound Assignment Operators

For `+=`, implement `AddAssign`:

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

**Key Difference:**

- `Add`: `fn add(self, rhs: Self) -> Self` - consumes and returns
- `AddAssign`: `fn add_assign(&mut self, rhs: Self)` - mutates in place

**Using It:**

```rust
let mut z = Complex { re: 1, im: 2 };
z += Complex { re: 3, im: 4 };
// z is now Complex { re: 4, im: 6 }
```

**Python Comparison:**

```python
def __iadd__(self, other):
    self.re += other.re
    self.im += other.im
    return self
```

Python's `__iadd__` must return `self`; Rust's `add_assign` returns nothing.

## Complete Working Example

Let's put it all together:

```rust
use std::ops::{Add, Sub, Mul};

#[derive(Clone, Copy, Debug)]
struct Complex<T> {
    re: T,
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

fn main() {
    // Integer complex numbers
    let a = Complex { re: 1, im: 2 };
    let b = Complex { re: 3, im: 4 };
    let c = a + b;
    println!("{:?}", c);  // Complex { re: 4, im: 6 }

    // Float complex numbers
    let x = Complex { re: 1.5, im: 2.5 };
    let y = Complex { re: 0.5, im: 1.5 };
    let z = x * y;
    println!("{:?}", z);  // Complex { re: -3.0, im: 3.5 }
}
```

## Key Takeaways

### Rust's Approach to Operator Overloading

1. **Explicit Traits**: Operators are just traits (`Add`, `Mul`, etc.)
2. **Type Safety**: Trait bounds ensure operations are valid
3. **Ownership**: Clear semantics for value vs. reference operations
4. **No Surprises**: Can't overload operators in unexpected ways

### Compared to Python

| Aspect | Rust | Python |
|--------|------|--------|
| Syntax | Implement traits | Define magic methods |
| Type checking | Compile-time with trait bounds | Runtime duck typing |
| Ownership | Explicit (value vs. reference) | Everything is references |
| Discoverability | Traits are documented | Magic methods by convention |
| Performance | Zero-cost abstraction | Dynamic dispatch overhead |

### Design Patterns

1. **Generic types with trait bounds**: `T: Add<Output = T>`
2. **Associated types**: `type Output = Self`
3. **Derived traits**: `#[derive(Clone, Copy, Debug)]`
4. **Multiple trait bounds**: `where T: Clone + Add + Sub + Mul`
5. **Ownership patterns**: Value consumption vs. mutation

## Exercises

### Exercise 1: Implement Division

Implement the `Div` trait for complex division:

```rust
// Formula: (a + bi) / (c + di) = [(ac + bd) + (bc - ad)i] / (c² + d²)
use std::ops::Div;

impl<T> Div for Complex<T>
where
    T: /* What bounds do you need? */,
{
    type Output = Complex<T>;
    fn div(self, rhs: Self) -> Self {
        // Your implementation here
    }
}
```

### Exercise 2: Implement Conjugate

Add a method to compute the complex conjugate:

```rust
impl<T: Neg<Output = T>> Complex<T> {
    pub fn conjugate(self) -> Complex<T> {
        // a + bi -> a - bi
    }
}
```

### Exercise 3: Implement Magnitude

For floating-point complex numbers, compute the magnitude:

```rust
impl Complex<f64> {
    pub fn magnitude(&self) -> f64 {
        // |a + bi| = sqrt(a² + b²)
    }
}
```

### Exercise 4: Very Generic Add

Implement `Add` so you can add `Complex<i32>` to `Complex<f64>`:

```rust
impl<L, R> Add<Complex<R>> for Complex<L>
where
    L: Add<R>,
{
    type Output = Complex<L::Output>;
    fn add(self, rhs: Complex<R>) -> Self::Output {
        // Implementation
    }
}
```

### Exercise 5: Derive Everything

Instead of manual `PartialEq`, use derive macros:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Complex<T> {
    re: T,
    im: T,
}
```

When does this work? What trait bounds does `T` need?

## Advanced Topics

### Move Semantics and Performance

```rust
// This consumes both operands
let z = a + b;  // a and b moved, can't use them again

// To keep using them, use Copy types or references
let z = &a + &b;  // Would need impl Add for &Complex
```

### Working with References

You could implement `Add` for references:

```rust
impl<'a, 'b, T> Add<&'b Complex<T>> for &'a Complex<T>
where
    T: Add<Output = T> + Clone,
{
    type Output = Complex<T>;
    fn add(self, rhs: &'b Complex<T>) -> Complex<T> {
        Complex {
            re: self.re.clone() + rhs.re.clone(),
            im: self.im.clone() + rhs.im.clone(),
        }
    }
}
```

Now you can add references: `&a + &b`

## Next Steps

- Explore the `num_complex` crate for a production-ready implementation
- Learn about more `std::ops` traits: `Rem`, `BitAnd`, `Shl`, etc.
- Study trait coherence rules and the orphan rule
- Understand blanket implementations and specialization

## Further Reading

- [std::ops Module Documentation](https://doc.rust-lang.org/std/ops/)
- [The Rust Book - Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [The Rust Book - Advanced Traits](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html)
- [num_complex Crate](https://docs.rs/num-complex/)

## Reference: Complete Code

The complete implementation can be found at:
`/home/user/rust-programming-examples/complex/src/lib.rs`

Run the tests with:
```bash
cd /home/user/rust-programming-examples/complex
cargo test
```
