# Tutorial: Building a Queue Data Structure in Rust

## Introduction

In this tutorial, you'll learn how to implement a First-In-First-Out (FIFO) queue data structure in Rust from scratch. This is your first step into understanding how data structures work in Rust, including concepts like ownership, borrowing, and working with Vec as underlying storage.

By the end of this tutorial, you'll have built a working queue that can efficiently add and remove characters, and you'll understand the fundamental differences between Rust and Python when it comes to data structure implementation.

## Prerequisites

Before starting this tutorial, you should:
- Have Rust installed (version 1.56 or later)
- Understand basic Rust syntax (variables, functions, structs)
- Be familiar with Python lists (we'll compare them to Rust's Vec)

## What You'll Build

You'll create a `Queue` struct that stores characters (`char`) and supports these operations:
- `push(c)` - Add a character to the back of the queue
- `pop()` - Remove and return a character from the front
- `is_empty()` - Check if the queue is empty
- `new()` - Create a new empty queue

## Step 1: Understanding the Queue Design

Unlike a simple wrapper around a single Vec, our queue uses **two Vec containers** for efficiency:

```rust
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last
    younger: Vec<char>  // younger elements, youngest last
}
```

### Why Two Vectors?

This clever design gives us O(1) amortized performance for both push and pop operations. Here's how it works:

- **younger**: New elements are pushed here (at the end)
- **older**: When we need to pop, we take from the end of this vector
- When **older** is empty and we need to pop, we move all elements from **younger** to **older** (reversed)

**Comparison to Python:**
In Python, you might use a `collections.deque` or a simple list. Python lists support `append()` (O(1)) but `pop(0)` is O(n). Our Rust implementation achieves better amortized performance.

```python
# Python equivalent (less efficient)
class Queue:
    def __init__(self):
        self.items = []

    def push(self, item):
        self.items.append(item)  # O(1)

    def pop(self):
        if self.items:
            return self.items.pop(0)  # O(n) - expensive!
        return None
```

## Step 2: Creating the Struct

Create a new Rust project:

```bash
cargo new queue --lib
cd queue
```

Open `src/lib.rs` and define the struct with warning attributes:

```rust
#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

/// A first-in, first-out queue of characters.
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last.
    younger: Vec<char>  // younger elements, youngest last.
}
```

**Key Rust Concepts:**

1. **Documentation comments** (`///`) appear in generated docs
2. **Public visibility** (`pub`) makes the struct accessible from other modules
3. **Vec<char>** is Rust's growable array type, similar to Python's list but type-safe

## Step 3: Implementing the Push Method

The `push` method adds elements to the back of the queue:

```rust
impl Queue {
    /// Push a character onto the back of a queue.
    pub fn push(&mut self, c: char) {
        self.younger.push(c);
    }
}
```

**Breaking Down the Syntax:**

- `impl Queue` - Implementation block for Queue methods
- `&mut self` - Mutable reference to self (we need to modify the queue)
- `c: char` - Parameter of type char
- `self.younger.push(c)` - Delegate to Vec's push method

**Ownership and Borrowing:**

The `&mut self` parameter means:
- The method **borrows** the Queue mutably
- The caller retains ownership
- No other references can exist while this method runs
- This prevents data races at compile time!

**Python Comparison:**

```python
def push(self, c):
    self.younger.append(c)
```

Python's `self` is similar, but Rust enforces mutability at compile time.

## Step 4: Implementing the Pop Method

The `pop` method is more complex because it handles the two-vector strategy:

```rust
impl Queue {
    /// Pop a character off the front of a queue. Return `Some(c)` if there
    /// was a character to pop, or `None` if the queue was empty.
    pub fn pop(&mut self) -> Option<char> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }

            // Bring the elements in younger over to older, and put them in
            // the promised order.
            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        // Now older is guaranteed to have something. Vec's pop method
        // already returns an Option, so we're set.
        self.older.pop()
    }
}
```

**Step-by-Step Walkthrough:**

1. **Check if older is empty**: If not, we can pop directly from it
2. **Check if younger is empty**: If both are empty, return `None`
3. **Transfer elements**: Use `std::mem::swap` to efficiently swap the vectors
4. **Reverse older**: Put elements in FIFO order (oldest at the end)
5. **Pop from older**: Return the oldest element

**Understanding Option<char>:**

Rust doesn't have `null`. Instead, it uses `Option<T>`:
- `Some(value)` - Contains a value
- `None` - No value available

```python
# Python equivalent
def pop(self):
    if not self.older:
        if not self.younger:
            return None
        self.older, self.younger = self.younger, self.older
        self.older.reverse()
    return self.older.pop() if self.older else None
```

**Why std::mem::swap?**

`swap` is incredibly efficient - it just swaps the internal pointers of the Vecs, not the actual data. This is an O(1) operation!

## Step 5: Adding Helper Methods

### The is_empty Method

```rust
impl Queue {
    pub fn is_empty(&self) -> bool {
        self.older.is_empty() && self.younger.is_empty()
    }
}
```

**Note:** This uses `&self` (immutable borrow) because it only reads data.

### The new Constructor

```rust
impl Queue {
    pub fn new() -> Queue {
        Queue { older: Vec::new(), younger: Vec::new() }
    }
}
```

**Rust vs Python Constructors:**

```python
# Python
def __init__(self):
    self.older = []
    self.younger = []

# Rust
pub fn new() -> Queue {
    Queue { older: Vec::new(), younger: Vec::new() }
}
```

Rust uses a `new()` associated function (like a static method) by convention, not a special `__init__` method.

## Step 6: Complete Implementation

Here's the full implementation with all methods:

```rust
#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

/// A first-in, first-out queue of characters.
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last.
    younger: Vec<char>  // younger elements, youngest last.
}

impl Queue {
    /// Create a new, empty queue.
    pub fn new() -> Queue {
        Queue { older: Vec::new(), younger: Vec::new() }
    }

    /// Push a character onto the back of a queue.
    pub fn push(&mut self, c: char) {
        self.younger.push(c);
    }

    /// Pop a character off the front of a queue. Return `Some(c)` if there
    /// was a character to pop, or `None` if the queue was empty.
    pub fn pop(&mut self) -> Option<char> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }

            // Bring the elements in younger over to older, and put them in
            // the promised order.
            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        // Now older is guaranteed to have something. Vec's pop method
        // already returns an Option, so we're set.
        self.older.pop()
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.older.is_empty() && self.younger.is_empty()
    }

    /// Split the queue into its two internal vectors.
    /// This consumes the queue.
    pub fn split(self) -> (Vec<char>, Vec<char>) {
        (self.older, self.younger)
    }
}
```

## Step 7: Testing Your Queue

Add these tests to verify your implementation:

```rust
#[test]
fn test_push_pop() {
    let mut q = Queue::new();

    q.push('0');
    q.push('1');
    assert_eq!(q.pop(), Some('0'));

    q.push('∞');
    assert_eq!(q.pop(), Some('1'));
    assert_eq!(q.pop(), Some('∞'));
    assert_eq!(q.pop(), None);
}

#[test]
fn test_is_empty() {
    let mut q = Queue::new();

    assert!(q.is_empty());
    q.push('☉');
    assert!(!q.is_empty());
    q.pop();
    assert!(q.is_empty());
}
```

Run the tests:

```bash
cargo test
```

You should see:

```
running 2 tests
test test_is_empty ... ok
test test_push_pop ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Key Rust Concepts Learned

### 1. Ownership
- Each value has a single owner
- When the owner goes out of scope, the value is dropped
- The `split()` method takes `self` (not `&self`), consuming the queue

### 2. Borrowing
- `&self` - Immutable borrow (read-only)
- `&mut self` - Mutable borrow (can modify)
- Only one mutable borrow OR multiple immutable borrows at a time

### 3. Option<T>
- Rust's type-safe alternative to null
- Forces you to handle the "no value" case
- Used in `pop()` to indicate an empty queue

### 4. Vec<T>
- Growable array type (like Python's list)
- Stores elements contiguously in memory
- Automatically grows when needed

## Common Pitfalls and Solutions

### Pitfall 1: Trying to use after move

```rust
let mut q = Queue::new();
q.push('a');
let vecs = q.split();  // q is consumed here
q.push('b');           // ERROR! q has been moved
```

**Solution:** Don't use a value after it's been moved (consumed).

### Pitfall 2: Multiple mutable borrows

```rust
let mut q = Queue::new();
let r1 = &mut q;
let r2 = &mut q;  // ERROR! Can't have two mutable borrows
```

**Solution:** Only take one mutable reference at a time.

### Pitfall 3: Forgetting to make q mutable

```rust
let q = Queue::new();  // Not mutable!
q.push('x');           // ERROR! Can't call &mut self on immutable value
```

**Solution:** Use `let mut q = Queue::new();`

## Performance Analysis

| Operation | Time Complexity | Explanation |
|-----------|----------------|-------------|
| push      | O(1)          | Just appends to younger Vec |
| pop       | O(1) amortized| Usually O(1), occasionally O(n) for transfer |
| is_empty  | O(1)          | Just checks two boolean conditions |
| new       | O(1)          | Creates two empty Vecs |

The two-vector design gives us excellent average performance. The occasional O(n) transfer is amortized across many O(1) operations.

## Comparison with Python

| Feature | Rust Queue | Python List |
|---------|-----------|-------------|
| Type Safety | Compile-time checked | Runtime checked |
| Memory Safety | Guaranteed by compiler | Developer's responsibility |
| Pop from front | O(1) amortized | O(n) |
| Null handling | Option<char> | None value |
| Mutability | Explicit (&mut) | Implicit |

## Next Steps

Now that you understand the basic queue implementation, you're ready to:

1. **Learn generics**: Move to the next tutorial to make this queue work with any type, not just `char`
2. **Add more methods**: Try implementing `peek()`, `len()`, or `clear()`
3. **Implement traits**: Learn about Display, Debug, and IntoIterator traits

## Additional Exercises

1. **Add a peek method**: Return a reference to the front element without removing it
   ```rust
   pub fn peek(&self) -> Option<&char> {
       // Your implementation here
   }
   ```

2. **Add a len method**: Return the total number of elements
   ```rust
   pub fn len(&self) -> usize {
       // Your implementation here
   }
   ```

3. **Implement FromIterator**: Allow creating a queue from an iterator
   ```rust
   let q: Queue = "hello".chars().collect();
   ```

## Summary

Congratulations! You've built a working queue data structure in Rust. You've learned:

- How to design structs with multiple fields
- The difference between `&self` and `&mut self`
- How to use `Option<T>` for safe null handling
- The importance of ownership and borrowing
- How Vec works as underlying storage
- Why two vectors can be more efficient than one

This foundational knowledge will help you understand more complex data structures and Rust's unique approach to memory safety.

## Resources

- [Rust Book - Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Rust Book - Structs](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [Vec Documentation](https://doc.rust-lang.org/std/vec/struct.Vec.html)
- [Option Documentation](https://doc.rust-lang.org/std/option/enum.Option.html)
