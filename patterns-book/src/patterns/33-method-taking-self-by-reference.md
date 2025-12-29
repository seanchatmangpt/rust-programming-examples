# 33. METHOD TAKING SELF BY REFERENCE

*[Illustration: A Queue struct being inspected by multiple callers simultaneously through shared references, each caller able to check if the queue is empty without preventing others from reading]*

...within any **STRUCT WITH METHODS (20)**, when you need to inspect the state of an object without modifying it or taking ownership...

◆ ◆ ◆

**Methods that only read data should not prevent other reads, and they should not consume the value. But Rust's ownership system makes you choose between taking ownership, exclusive access, or shared access.**

Consider a queue that provides an `is_empty()` check. If the method took ownership (`self`), checking whether the queue is empty would *destroy the queue*:

```rust
// ❌ Terrible: checking consumes the queue
fn is_empty(self) -> bool {
    self.older.is_empty() && self.younger.is_empty()
}

let mut q = Queue::new();
q.push('a');
if q.is_empty() {  // q is moved here
    // ...
}
// q is now unusable! Just checking destroyed it.
```

If the method took a mutable reference (`&mut self`), checking would require exclusive access. No other code could read the queue at the same time:

```rust
// ❌ Too restrictive: checking requires exclusive access
fn is_empty(&mut self) -> bool {
    self.older.is_empty() && self.younger.is_empty()
}

let r1 = &q;
let r2 = &q;
if q.is_empty() {  // ERROR: can't borrow mutably while immutably borrowed
    // ...
}
```

The method doesn't modify the queue, so requiring exclusive access is too strong. It prevents natural patterns like checking emptiness while iterating, or having multiple threads read the same queue.

The solution is to take *shared, immutable access* via `&self`. This is Rust's "reader" access: many references can exist simultaneously, as long as none of them mutate the data.

A method taking `&self` can:
- Read any field
- Call other `&self` methods
- Return references to internal data (with proper lifetimes)
- Be called multiple times without consuming the value

It cannot:
- Modify any field
- Call `&mut self` methods
- Move out of fields
- Violate the immutability guarantee

**Therefore:**

**For methods that only inspect state without modifying it, declare the receiver as `&self`. This allows multiple simultaneous readers and doesn't consume the value. The method can access all fields immutably.**

```rust
pub struct Queue {
    older: Vec<char>,
    younger: Vec<char>
}

impl Queue {
    /// Check if the queue contains no elements.
    /// Takes &self because it only reads, doesn't modify.
    pub fn is_empty(&self) -> bool {
        self.older.is_empty() && self.younger.is_empty()
    }

    /// Get the total number of elements.
    /// Takes &self - pure inspection.
    pub fn len(&self) -> usize {
        self.older.len() + self.younger.len()
    }

    /// Peek at the front element without removing it.
    /// Returns Option<&char> - reference to internal data.
    pub fn peek(&self) -> Option<&char> {
        if !self.older.is_empty() {
            self.older.last()
        } else {
            self.younger.first()
        }
    }
}

// Usage: can call many times, can call while borrowed
let mut q = Queue::new();
q.push('a');
q.push('b');

// Multiple calls to &self methods
let empty = q.is_empty();  // Borrows q immutably
let count = q.len();       // Borrows q immutably again
let front = q.peek();      // And again

// Can still use q afterward
println!("Empty: {}, Len: {}, Front: {:?}", empty, count, front);

// Multiple references can check simultaneously
let r1 = &q;
let r2 = &q;
if r1.is_empty() && r2.is_empty() {
    println!("Both references see empty queue");
}
```

*[Diagram: Method receiver types and their effects:

```
fn method(self)        → Consumes value
                         Can only call once
                         Transfers ownership

fn method(&mut self)   → Borrows mutably
                         Exclusive access required
                         Can modify fields
                         Can call once at a time

fn method(&self)       → Borrows immutably
                         Shared access allowed
                         Cannot modify fields
                         Can call many times simultaneously
                         ↑ USE THIS FOR READ-ONLY OPERATIONS
```

Calling pattern:
```
let q = Queue::new();

q.is_empty()    // &self - borrows immutably
q.is_empty()    // Can call again immediately
q.len()         // Another &self - no problem

let r1 = &q;
let r2 = &q;
r1.is_empty() && r2.is_empty()  // Both can read simultaneously
```
]*

Methods taking `&self` are the most permissive for callers—they impose the fewest restrictions. They represent **read-only operations**, which cannot break invariants since they don't modify state.

The borrow checker ensures safety: while any `&self` borrow exists, no `&mut self` borrow can exist. This prevents the classic reader-writer race where one thread reads while another modifies. At compile time, Rust guarantees no data race.

When the method returns a reference to internal data (like `peek()` returning `Option<&char>`), the lifetime is automatically tied to the `&self` lifetime. This prevents dangling references—callers cannot use the returned reference after the original value is modified or dropped.

◆ ◆ ◆

Use this for getter methods, predicate methods (`is_empty`, `contains`), and any inspection operation. Pair with **METHOD TAKING SELF BY MUT REFERENCE (34)** for modifying operations. If the method needs to modify state, use `&mut self` instead. If the method needs to consume the value and transform it, use **METHOD CONSUMING SELF (35)**. Methods taking `&self` should be const-friendly where possible. They should never mutate through interior mutability (RefCell, etc.) unless clearly documented—that violates the `&self` contract.
