# Tutorial: Making Your Queue Generic with Rust Generics

## Introduction

In the previous tutorial, you built a queue that works with `char` values. But what if you want a queue of integers? Or strings? Or any other type? In this tutorial, you'll learn how to use **Rust generics** to make your queue work with any type.

By the end of this tutorial, you'll understand:
- How to add type parameters to structs
- How to implement methods for generic types
- How generics compare to Python's typing system
- Type inference in Rust
- Generic constraints and bounds

## Prerequisites

Before starting this tutorial, you should:
- Have completed the basic queue tutorial
- Understand Rust's ownership and borrowing
- Be familiar with basic Rust syntax

## What You'll Build

You'll transform this:
```rust
pub struct Queue {
    older: Vec<char>,
    younger: Vec<char>
}
```

Into this:
```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}
```

Now your queue can hold **any type**: `Queue<i32>`, `Queue<String>`, `Queue<MyStruct>`, etc.

## Step 1: Understanding Rust Generics

### What Are Generics?

Generics allow you to write code that works with multiple types without sacrificing type safety. They're similar to templates in C++ or generics in Java, but with Rust's unique safety guarantees.

**Comparison to Python:**

```python
# Python with type hints (runtime not enforced)
from typing import TypeVar, Generic, Optional

T = TypeVar('T')

class Queue(Generic[T]):
    def __init__(self):
        self.older: list[T] = []
        self.younger: list[T] = []

    def push(self, item: T) -> None:
        self.younger.append(item)

    def pop(self) -> Optional[T]:
        # ... implementation
        pass
```

**Key Differences:**

| Feature | Rust Generics | Python Typing |
|---------|--------------|---------------|
| Enforcement | Compile-time | Runtime (with tools) |
| Performance | Zero-cost abstraction | Same as non-generic |
| Type Checking | Guaranteed | Optional (mypy, etc.) |
| Monomorphization | Yes (separate code per type) | No (single code) |

## Step 2: Converting the Struct to Generic

Let's start by making the struct generic:

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}
```

**Breaking Down the Syntax:**

- `<T>` - Type parameter declaration
- `T` - By convention, single uppercase letters (T, U, V, etc.)
- `Vec<T>` - A Vec that holds elements of type T

You can read `Queue<T>` as "Queue of T" where T is any type.

### Multiple Type Parameters

While our queue only needs one type parameter, you could have multiple:

```rust
// A hypothetical map structure
struct MyMap<K, V> {
    keys: Vec<K>,
    values: Vec<V>
}
```

## Step 3: Implementing Generic Methods

When implementing methods for a generic struct, you must declare the type parameter:

```rust
impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { older: Vec::new(), younger: Vec::new() }
    }
}
```

**Important Points:**

1. `impl<T>` - Declares that this implementation is generic over T
2. `Queue<T>` - Specifies which type we're implementing for
3. `Self` - Shorthand for `Queue<T>`

### The New Constructor

```rust
impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { older: Vec::new(), younger: Vec::new() }
    }
}
```

This creates a new queue for **any type T**. Rust will infer the type based on usage:

```rust
let mut char_queue = Queue::new();
char_queue.push('a');  // Rust infers Queue<char>

let mut int_queue = Queue::new();
int_queue.push(42);    // Rust infers Queue<i32>
```

### The Push Method

```rust
impl<T> Queue<T> {
    pub fn push(&mut self, t: T) {
        self.younger.push(t);
    }
}
```

**Changes from non-generic version:**
- Parameter changed from `c: char` to `t: T`
- Everything else stays the same!

The generic version works because `Vec<T>` already has a generic `push` method.

### The Is_Empty Method

```rust
impl<T> Queue<T> {
    pub fn is_empty(&self) -> bool {
        self.older.is_empty() && self.younger.is_empty()
    }
}
```

No changes needed! This method doesn't depend on the specific type T.

### The Pop Method

```rust
impl<T> Queue<T> {
    pub fn pop(&mut self) -> Option<T> {
        if self.older.is_empty() {
            use std::mem::swap;

            if self.younger.is_empty() {
                return None;
            }

            // Bring the elements in younger over to older, and put them in
            // the promised order.
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        // Now older is guaranteed to have something. Vec's pop method
        // already returns an Option, so we're set.
        self.older.pop()
    }
}
```

**Changes:**
- Return type changed from `Option<char>` to `Option<T>`
- The logic remains identical

### The Split Method

```rust
impl<T> Queue<T> {
    pub fn split(self) -> (Vec<T>, Vec<T>) {
        (self.older, self.younger)
    }
}
```

**Changes:**
- Return type changed from `(Vec<char>, Vec<char>)` to `(Vec<T>, Vec<T>)`

## Step 4: Complete Generic Implementation

Here's the full generic queue implementation:

```rust
#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

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

    pub fn is_empty(&self) -> bool {
        self.older.is_empty() && self.younger.is_empty()
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.older.is_empty() {
            use std::mem::swap;

            if self.younger.is_empty() {
                return None;
            }

            // Bring the elements in younger over to older, and put them in
            // the promised order.
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        // Now older is guaranteed to have something. Vec's pop method
        // already returns an Option, so we're set.
        self.older.pop()
    }

    pub fn split(self) -> (Vec<T>, Vec<T>) {
        (self.older, self.younger)
    }
}
```

## Step 5: Using Your Generic Queue

### Type Inference

Rust's type inference is powerful. You rarely need to explicitly specify types:

```rust
let mut q = Queue::new();
q.push("CAD");      // Rust infers Queue<&str>
q.push("BTC");

let mut r = Queue::new();
r.push(0.74);       // Rust infers Queue<f64>
r.push(13764.0);
```

### Explicit Type Annotation

Sometimes you want to be explicit:

```rust
let mut q = Queue::<char>::new();
q.push('x');

// Or with type annotation
let mut q: Queue<char> = Queue::new();
q.push('x');
```

### Using with Different Types

```rust
// Queue of integers
let mut int_queue = Queue::new();
int_queue.push(1);
int_queue.push(2);
int_queue.push(3);
assert_eq!(int_queue.pop(), Some(1));

// Queue of strings
let mut string_queue = Queue::new();
string_queue.push("hello".to_string());
string_queue.push("world".to_string());
assert_eq!(string_queue.pop(), Some("hello".to_string()));

// Queue of custom structs
struct Point { x: i32, y: i32 }
let mut point_queue = Queue::new();
point_queue.push(Point { x: 0, y: 0 });
point_queue.push(Point { x: 1, y: 1 });
```

## Step 6: Testing Generic Code

Add comprehensive tests for different types:

```rust
#[test]
fn test_generic() {
    // Explicit type annotation
    let mut q = Queue::<char>::new();
    &mut q;  // Take a mutable reference (tests borrowing)
    drop(q); // Drop it explicitly

    // Type inference with &str
    let mut q = Queue::new();
    let mut r = Queue::new();

    q.push("CAD");   // apparently a Queue<&'static str>
    r.push(0.74);    // apparently a Queue<f64>

    q.push("BTC");
    r.push(13764.0);
}

#[test]
fn test() {
    let mut q = Queue::new();

    q.push('*');
    assert_eq!(q.pop(), Some('*'));
    assert_eq!(q.pop(), None);

    q.push('0');
    q.push('1');
    assert_eq!(q.pop(), Some('0'));

    q.push('∞');
    assert_eq!(q.pop(), Some('1'));
    assert_eq!(q.pop(), Some('∞'));
    assert_eq!(q.pop(), None);

    assert!(q.is_empty());
    q.push('☉');
    assert!(!q.is_empty());
    q.pop();
    assert!(q.is_empty());

    let mut q = Queue::new();

    q.push('P');
    q.push('D');
    assert_eq!(q.pop(), Some('P'));
    q.push('X');

    assert_eq!(q.split(), (vec!['D'], vec!['X']));
}
```

Run the tests:

```bash
cargo test
```

## Step 7: Understanding Type Constraints

### The Problem

Not all methods work with all types. What if you wanted to print the queue contents?

```rust
// This won't compile!
impl<T> Queue<T> {
    pub fn print(&self) {
        for item in &self.older {
            println!("{}", item);  // ERROR: T might not be printable!
        }
    }
}
```

### The Solution: Trait Bounds

Use trait bounds to constrain T:

```rust
use std::fmt::Display;

impl<T: Display> Queue<T> {
    pub fn print(&self) {
        for item in &self.older {
            println!("{}", item);  // OK: T implements Display
        }
        for item in &self.younger {
            println!("{}", item);
        }
    }
}
```

**Syntax Breakdown:**

- `T: Display` - "T must implement the Display trait"
- Now you can only call `print()` on queues where T implements Display

### Multiple Trait Bounds

You can require multiple traits:

```rust
impl<T: Display + Clone> Queue<T> {
    pub fn print_cloned(&self) {
        for item in &self.older {
            let cloned = item.clone();
            println!("{}", cloned);
        }
    }
}
```

Or using the `where` clause for better readability:

```rust
impl<T> Queue<T>
where
    T: Display + Clone
{
    pub fn print_cloned(&self) {
        // ...
    }
}
```

## Step 8: Advanced Generic Patterns

### Default Type Parameters

You can provide default types:

```rust
// Not in our example, but possible
struct Buffer<T = u8> {
    data: Vec<T>
}

let buf1 = Buffer { data: vec![1, 2, 3] };        // Buffer<u8>
let buf2: Buffer<i32> = Buffer { data: vec![1] }; // Buffer<i32>
```

### Const Generics

You can also have constant generic parameters:

```rust
struct ArrayQueue<T, const N: usize> {
    items: [Option<T>; N],
    head: usize,
    tail: usize,
}

let q: ArrayQueue<i32, 10> = ArrayQueue::new(); // Queue of max 10 items
```

## Key Concepts Learned

### 1. Type Parameters

```rust
struct Queue<T>     // Declare type parameter
impl<T> Queue<T>   // Use in implementation
```

### 2. Type Inference

Rust can often figure out types automatically:
```rust
let mut q = Queue::new();
q.push(42);  // Rust knows q is Queue<i32>
```

### 3. Trait Bounds

Constrain what types T can be:
```rust
impl<T: Display> Queue<T>  // T must implement Display
```

### 4. Monomorphization

Rust generates specialized code for each concrete type:
- `Queue<i32>` gets its own compiled code
- `Queue<String>` gets different compiled code
- Zero runtime overhead!

## Comparison: Rust vs Python Generics

### Rust Generics

**Pros:**
- Compile-time type checking
- Zero runtime overhead
- Guaranteed type safety
- Excellent error messages

**Cons:**
- More verbose syntax
- Longer compile times (monomorphization)
- Steeper learning curve

### Python Typing

**Pros:**
- Simpler syntax
- No compilation step
- Gradual typing (can add types incrementally)

**Cons:**
- Runtime overhead for type checking
- Not enforced by default
- Can be ignored
- Requires external tools (mypy)

### Example Comparison

```rust
// Rust - Enforced at compile time
let mut q = Queue::new();
q.push(42);
q.push("oops");  // COMPILE ERROR: expected i32, found &str
```

```python
# Python - Only caught with mypy or at runtime
from typing import Generic, TypeVar
T = TypeVar('T')

class Queue(Generic[T]):
    pass

q: Queue[int] = Queue()
q.push(42)
q.push("oops")  # mypy will warn, but Python runs it
```

## Common Pitfalls and Solutions

### Pitfall 1: Forgetting impl<T>

```rust
// WRONG
impl Queue<T> {  // ERROR: T is not defined
    pub fn new() -> Self { ... }
}

// CORRECT
impl<T> Queue<T> {
    pub fn new() -> Self { ... }
}
```

### Pitfall 2: Type Mismatch

```rust
let mut q = Queue::new();
q.push(42);      // q is now Queue<i32>
q.push("text");  // ERROR: expected i32, found &str
```

**Solution:** Create separate queues for different types.

### Pitfall 3: Unnecessary Trait Bounds

```rust
// TOO RESTRICTIVE
impl<T: Clone> Queue<T> {
    pub fn new() -> Self {  // Doesn't need Clone!
        Queue { older: Vec::new(), younger: Vec::new() }
    }
}

// BETTER
impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { older: Vec::new(), younger: Vec::new() }
    }
}
```

**Solution:** Only add trait bounds where necessary.

## Performance Considerations

### Monomorphization

Rust creates specialized versions of generic code for each type:

```rust
let q1 = Queue::<i32>::new();
let q2 = Queue::<String>::new();
```

The compiler generates two separate implementations:
- One optimized for `i32`
- One optimized for `String`

**Benefits:**
- Zero runtime overhead
- Optimal performance for each type
- No vtable lookups

**Tradeoffs:**
- Larger binary size (code duplication)
- Longer compile times

### Comparison to Dynamic Dispatch

```rust
// Static dispatch (generics) - fast, larger binary
fn process<T: Display>(item: T) {
    println!("{}", item);
}

// Dynamic dispatch (trait objects) - smaller binary, runtime cost
fn process(item: &dyn Display) {
    println!("{}", item);
}
```

## Exercises

### Exercise 1: Add a Generic Filter Method

Create a method that returns a new queue with only elements matching a predicate:

```rust
impl<T: Clone> Queue<T> {
    pub fn filter<F>(&self, predicate: F) -> Queue<T>
    where
        F: Fn(&T) -> bool
    {
        // Your implementation here
        // Hint: Iterate through all elements, apply predicate
        todo!()
    }
}

// Usage
let mut q = Queue::new();
q.push(1);
q.push(2);
q.push(3);
let even_q = q.filter(|x| x % 2 == 0);
```

### Exercise 2: Implement From Iterator

Allow creating a queue from an iterator:

```rust
impl<T> FromIterator<T> for Queue<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        // Your implementation here
        todo!()
    }
}

// Usage
let q: Queue<char> = "hello".chars().collect();
```

### Exercise 3: Add a Map Method

Transform elements to a different type:

```rust
impl<T> Queue<T> {
    pub fn map<U, F>(self, f: F) -> Queue<U>
    where
        F: Fn(T) -> U
    {
        // Your implementation here
        todo!()
    }
}

// Usage
let mut q = Queue::new();
q.push(1);
q.push(2);
let string_q = q.map(|x| x.to_string());
```

## Next Steps

Now that you understand generics, you're ready to:

1. **Learn about recursive data structures**: Move to the Binary Tree tutorial
2. **Explore trait objects**: Learn about dynamic dispatch
3. **Study lifetime parameters**: Understand generic lifetimes
4. **Implement standard traits**: Debug, Display, Iterator, etc.

## Summary

Congratulations! You've learned how to use Rust generics to create flexible, reusable data structures. You now understand:

- How to add type parameters to structs and implementations
- The difference between `impl<T>` and `impl Queue<T>`
- How type inference works in Rust
- How to use trait bounds to constrain generic types
- The performance characteristics of monomorphization
- How Rust generics compare to Python's typing system

This knowledge is fundamental to writing idiomatic Rust code and understanding the standard library.

## Resources

- [Rust Book - Generic Types](https://doc.rust-lang.org/book/ch10-00-generics.html)
- [Rust Book - Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Rust by Example - Generics](https://doc.rust-lang.org/rust-by-example/generics.html)
- [The Rustonomicon - Type System](https://doc.rust-lang.org/nomicon/exotic-sizes.html)
