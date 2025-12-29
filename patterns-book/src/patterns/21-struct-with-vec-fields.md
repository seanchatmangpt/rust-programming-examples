# 21. STRUCT WITH VEC FIELDS

*A container with dynamic collections inside, like a basket holding bags of varying size*

...within a **[CUSTOM DATA TYPE](./20-custom-data-type.md)**, when you need to store a varying number of elements as part of your struct's state...

◆ ◆ ◆

**How do you store collections of elements within a struct without knowing the size at compile time?**

A struct's size must be known at compile time, yet real programs need to store variable-length data. You might need a queue that grows and shrinks, a cache that accumulates entries, or a buffer that receives incoming data. Fixed-size arrays are too rigid—you waste space or run out of room.

The tension is between Rust's stack-allocation preference and the need for growth. Stack values have compile-time sizes. Vec solves this by storing a pointer, capacity, and length on the stack (three words), while the actual data lives on the heap. This gives you the best of both worlds: the struct has a known size, but its contents can grow.

Collections as struct fields also raise ownership questions. When you store a Vec inside a struct, the struct owns the Vec, which owns its elements. This creates a clear ownership tree. When the struct is dropped, the Vec is dropped, which drops its elements. Memory management follows naturally from the ownership rules.

**Therefore:**

**Store dynamic collections in your struct as Vec<T> fields, making the struct the owner of the collection and all its elements.**

```rust
/// A first-in, first-out queue of characters.
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last.
    younger: Vec<char>  // younger elements, youngest last.
}

impl Queue {
    pub fn new() -> Queue {
        Queue {
            older: Vec::new(),
            younger: Vec::new()
        }
    }

    pub fn push(&mut self, c: char) {
        self.younger.push(c);
    }

    pub fn is_empty(&self) -> bool {
        self.older.is_empty() && self.younger.is_empty()
    }
}
```

*The struct contains Vec fields that grow and shrink as needed, while the struct itself has a fixed size of a few words*

◆ ◆ ◆

This pattern leads naturally to **[STRUCT WITH TWO VECS FOR QUEUE](./22-struct-with-two-vecs-for-queue.md)** when you need efficient push and pop operations, and to **[GENERIC TYPE WITH PARAMETER T](./24-generic-type-with-parameter-t.md)** when you want to work with any element type.
