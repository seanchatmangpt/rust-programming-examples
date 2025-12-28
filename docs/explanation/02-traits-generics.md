# Traits and Generics

## Understanding Rust's Type System

Coming from Python, you're used to duck typing ("if it walks like a duck...") and abstract base classes. Rust's trait system combines the best of both worlds: the flexibility of interfaces with compile-time guarantees. Unlike Python's runtime type checking, Rust verifies at compile time that your code works for all possible types.

## Traits: Rust's Answer to Interfaces

A trait defines behavior that types can implement. Think of traits as a contract: "Any type implementing this trait must provide these methods."

### Traits vs Python's Duck Typing

In Python, you might write:

```python
def add_numbers(a, b):
    return a + b  # Works for anything with __add__

result = add_numbers(5, 3)        # Fine
result = add_numbers("Hello, ", "World")  # Also fine
result = add_numbers(None, 5)     # Runtime error!
```

Python doesn't check if `a` and `b` support addition until runtime. Rust checks at compile time using traits.

From `/home/user/rust-programming-examples/complex/src/lib.rs`:

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

This says: "Complex numbers can be added, **but only if** their component type `T` can be added." The compiler verifies this. You can't compile `Complex<NotAddable> + Complex<NotAddable>`.

### Traits vs Python's Abstract Base Classes

Python's ABCs provide a middle ground:

```python
from abc import ABC, abstractmethod

class Drawable(ABC):
    @abstractmethod
    def draw(self):
        pass

class Circle(Drawable):
    def draw(self):
        print("Drawing circle")
```

Rust traits are similar but more powerful:

```rust
trait Drawable {
    fn draw(&self);
}

struct Circle { radius: f64 }

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius);
    }
}
```

Key differences:

1. **Compile-time checking** - Rust verifies implementations at compile time
2. **No inheritance hierarchy** - Types implement traits independently
3. **Orphan rule** - You can't implement external traits on external types (prevents conflicts)
4. **Zero runtime cost** - Trait dispatch can be statically resolved

## Generics: Type Parameters with Guarantees

Python 3.5+ added type hints, but they're optional and not enforced:

```python
from typing import TypeVar, Generic, List

T = TypeVar('T')

class Queue(Generic[T]):
    def __init__(self) -> None:
        self.items: List[T] = []

    def push(self, item: T) -> None:
        self.items.append(item)
```

This is documentation; Python doesn't enforce it at runtime.

Rust generics are real, enforced, and optimized. From `/home/user/rust-programming-examples/generic-queue/src/lib.rs`:

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { older: Vec::new(), younger: Vec::new() }
    }

    pub fn push(&mut self, t: T) {
        self.younger.push(t);
    }

    pub fn pop(&mut self) -> Option<T> {
        // ...
    }
}
```

This defines a queue that works with **any** type `T`. The compiler generates specialized code for each concrete type you use:

```rust
let mut char_queue = Queue::new();
char_queue.push('a');  // Queue<char>

let mut int_queue = Queue::new();
int_queue.push(42);    // Queue<i32>
```

Rust generates two separate implementations, each optimized for its type. This is called **monomorphization** - no runtime type checks, no boxing, just fast native code.

## Trait Bounds: Constraining Generic Types

Not all operations work on all types. Trait bounds let you constrain generic parameters.

The binary tree's `walk` method from `/home/user/rust-programming-examples/binary-tree/src/lib.rs`:

```rust
impl<T: Clone> BinaryTree<T> {
    fn walk(&self) -> Vec<T> {
        match *self {
            BinaryTree::Empty => vec![],
            BinaryTree::NonEmpty(ref boxed) => {
                let mut result = boxed.left.walk();
                result.push(boxed.element.clone());  // Requires Clone!
                result.extend(boxed.right.walk());
                result
            }
        }
    }
}
```

`T: Clone` is a trait bound. It says: "This implementation only exists for types that implement `Clone`." You can't call `walk()` on a `BinaryTree<NotCloneable>`.

Compare this to the `add` method:

```rust
impl<T: Ord> BinaryTree<T> {
    fn add(&mut self, value: T) {
        match *self {
            BinaryTree::NonEmpty(ref mut node) => {
                if value <= node.element {  // Requires Ord!
                    node.left.add(value);
                }
            }
            // ...
        }
    }
}
```

`T: Ord` means `T` must be orderable. You can't add elements to a `BinaryTree<Unorderable>`.

These are **different implementations** for the same type! A `BinaryTree<i32>` has both `walk()` and `add()` because `i32` implements both `Clone` and `Ord`. But `BinaryTree<SomeType>` might only have one or neither.

## Where Clauses: Complex Constraints

For complex bounds, `where` clauses improve readability:

```rust
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

This requires `T` to implement four traits: `Clone`, `Add`, `Sub`, and `Mul`. The `where` clause makes these constraints clear and readable.

## Common Traits: The Standard Library's Building Blocks

Rust's standard library defines many useful traits. Here are the most important for Python developers:

### Clone and Copy

```rust
#[derive(Clone, Copy, Debug)]
struct Complex<T> {
    re: T,
    im: T,
}
```

- **Clone**: Explicit deep copying via `.clone()`
- **Copy**: Implicit copying for cheap-to-copy types

In Python, everything is a reference:

```python
a = [1, 2, 3]
b = a        # b is a reference to the same list
b.append(4)  # a is now [1, 2, 3, 4]
```

In Rust, assignment moves ownership by default:

```rust
let a = vec![1, 2, 3];
let b = a;  // a is moved; can't use it anymore

// To copy, you must be explicit:
let a = vec![1, 2, 3];
let b = a.clone();  // a is still valid
```

`Copy` types are implicitly copied:

```rust
let a = 5;
let b = a;  // a is copied; both are valid
```

Types can implement `Copy` only if all their fields are `Copy`. `Vec<T>` can't be `Copy` because it owns heap data.

### Debug and Display

```rust
use std::fmt;

impl fmt::Display for Complex {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        let im_sign = if self.im < 0.0 { '-' } else { '+' };
        write!(dest, "{} {} {}i", self.re, im_sign, f64::abs(self.im))
    }
}
```

- **Debug**: For programmer-facing output (derived automatically)
- **Display**: For user-facing output (must implement manually)

Python equivalents:

```python
class Complex:
    def __repr__(self):  # Like Debug
        return f"Complex({self.re}, {self.im})"

    def __str__(self):   # Like Display
        im_sign = '-' if self.im < 0 else '+'
        return f"{self.re} {im_sign} {abs(self.im)}i"
```

### PartialEq and Eq

```rust
impl<T: PartialEq> PartialEq for Complex<T> {
    fn eq(&self, other: &Complex<T>) -> bool {
        self.re == other.re && self.im == other.im
    }
}

impl<T: Eq> Eq for Complex<T> {}
```

- **PartialEq**: Types that can be compared for equality (==, !=)
- **Eq**: Marker trait for types with total equality (reflexive, symmetric, transitive)

Why two traits? Some types have partial equality. For example, `f64` implements `PartialEq` but not `Eq` because `NaN != NaN`.

In Python:

```python
class Complex:
    def __eq__(self, other):  # Both PartialEq and Eq combined
        return self.re == other.re and self.im == other.im
```

### Ord and PartialOrd

For ordering comparisons (<, >, <=, >=):

```rust
impl<T: Ord> BinaryTree<T> {
    fn add(&mut self, value: T) {
        // Can compare values because T: Ord
    }
}
```

Python equivalent:

```python
class Point:
    def __lt__(self, other):  # Less than
        return self.x < other.x

    # Python requires you to implement __eq__, __le__, __gt__, etc.
    # or use @total_ordering decorator
```

## Operator Overloading Traits

Rust uses traits for operator overloading. From `/home/user/rust-programming-examples/complex/src/lib.rs`:

```rust
use std::ops::{Add, Sub, Mul, Neg};

impl<T> Add for Complex<T>
where T: Add<Output = T>
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

Common operator traits:

| Trait | Operator | Python Equivalent |
|-------|----------|-------------------|
| `Add` | `+` | `__add__` |
| `Sub` | `-` | `__sub__` |
| `Mul` | `*` | `__mul__` |
| `Div` | `/` | `__truediv__` |
| `Neg` | `-x` | `__neg__` |
| `Index` | `[]` | `__getitem__` |
| `Deref` | `*x` | (no equivalent) |

### Compound Assignment

```rust
use std::ops::AddAssign;

impl<T> AddAssign for Complex<T>
where T: AddAssign<T>
{
    fn add_assign(&mut self, rhs: Complex<T>) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

// Now you can use +=
let mut z = Complex { re: 1, im: 2 };
z += Complex { re: 3, im: 4 };
```

Python equivalent:

```python
def __iadd__(self, other):
    self.re += other.re
    self.im += other.im
    return self
```

## Deriving Traits Automatically

Rust can auto-generate implementations for common traits:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Complex<T> {
    re: T,
    im: T,
}
```

This generates implementations that work field-by-field. Only works if all fields implement the derived trait.

Python's closest equivalent is `@dataclass`:

```python
from dataclasses import dataclass

@dataclass
class Complex:
    re: float
    im: float
    # Automatically gets __init__, __repr__, __eq__
```

## Advanced: Very Generic Implementations

You can make traits extremely flexible:

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

This allows adding `Complex<i32>` to `Complex<f64>`, producing `Complex<f64>`. The type system infers the output type from the `Add` implementation of the component types.

## Generic Queue Example: Putting It All Together

The queue from `/home/user/rust-programming-examples/generic-queue/src/lib.rs` demonstrates clean generic design:

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { older: Vec::new(), younger: Vec::new() }
    }

    pub fn push(&mut self, t: T) {
        self.younger.push(t);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.older.is_empty() {
            use std::mem::swap;

            if self.younger.is_empty() {
                return None;
            }

            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        self.older.pop()
    }

    pub fn split(self) -> (Vec<T>, Vec<T>) {
        (self.older, self.younger)
    }
}
```

Notice:
- No trait bounds on the type definition (works with any `T`)
- Methods like `push` and `pop` work with any type
- The implementation is clean and generic
- Rust generates optimized code for each concrete type

## Key Differences from Python

| Aspect | Python | Rust |
|--------|--------|------|
| Type checking | Runtime | Compile-time |
| Generics | Type hints (optional) | Real generic code |
| Duck typing | Default behavior | Explicit via traits |
| Performance | Generic code is slow | Monomorphization (fast) |
| Operator overloading | Magic methods | Trait implementation |
| Multiple dispatch | Not built-in | Via trait bounds |

## Key Takeaways

1. **Traits define behavior** - Like interfaces, but verified at compile time
2. **Generics are real** - Not just documentation; the compiler enforces them
3. **Trait bounds constrain types** - Only types implementing required traits can be used
4. **Common traits are powerful** - Clone, Copy, Debug, Display, PartialEq, Ord
5. **Operator overloading uses traits** - Add, Sub, Mul, etc.
6. **Derive for free implementations** - Auto-generate common trait implementations
7. **Zero-cost abstractions** - Generic code is as fast as hand-written specific code

Traits and generics give you Python's flexibility with C++'s performance. The type system catches errors at compile time that would be runtime crashes in Python, and the generated code runs without any type-checking overhead.
