# 48. MEM SWAP FOR MOVING VALUES

*Two containers stand side by side, each holding a different collection of values. They need to exchange contents without creating temporary clones.*

...within a **EFFICIENT DATA MOVEMENT** context, when you need to exchange ownership between two locations...

◆ ◆ ◆

**How do you move values between two owned locations without the expense of cloning or the complexity of manual pointer manipulation?**

Rust's ownership system ensures that values have exactly one owner. When you need to move a value from one location to another, naive approaches create problems. You cannot simply assign because that would leave the old location in an invalid state. You cannot clone expensive structures like `Vec<T>` just to swap them—that defeats the purpose of zero-cost abstractions.

The queue implementation demonstrates this challenge perfectly. A queue maintains two vectors: `older` for elements ready to dequeue, and `younger` for newly pushed elements. When `older` empties, the queue must transfer all elements from `younger` to `older`, reversing them to maintain FIFO order.

A naive approach might clone `younger`, clear it, reverse the clone, and assign to `older`—but this allocates unnecessary memory and copies every element. Another approach might use unsafe pointer manipulation, but that's error-prone and defeats Rust's safety guarantees.

The standard library provides `std::mem::swap`, which exchanges the contents of two mutable references in-place, using only stack memory for a temporary. It works because Rust can briefly move both values onto the stack, swap them, and move them back—a constant-time operation regardless of the values' sizes.

**Therefore:**

**Use `std::mem::swap(&mut a, &mut b)` to exchange ownership of two values in-place, avoiding clones and unsafe code.**

```rust
// From generic-queue/src/lib.rs - swap vectors when older empties
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

    // Now older is guaranteed to have something
    self.older.pop()
}

// From queue/src/lib.rs - same pattern for concrete type
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
```

*The swap happens in an instant: two mutable references reach toward their targets, grasp the values, and exchange them in one atomic gesture—no copies made, no memory allocated, just a pure exchange of ownership.*

◆ ◆ ◆

Use `mem::replace(&mut dest, new_value)` when you need the old value while inserting a new one; use `mem::take(&mut value)` to extract a value while leaving a default in its place; avoid these patterns with **COPY TYPES** where simple assignment suffices.
