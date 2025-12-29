# 34. METHOD TAKING SELF BY MUT REFERENCE

*[Illustration: A Queue struct being modified by a single caller holding exclusive mutable access, with other potential callers blocked, ensuring the modification is atomic and safe from concurrent interference]*

...within any **STRUCT WITH METHODS (20)**, when you need to modify the internal state of an object while keeping it usable afterward...

◆ ◆ ◆

**Most operations on data structures modify state—pushing to a queue, updating a field, removing an element. These operations need exclusive access to ensure safety, but they shouldn't consume the value since the object remains valid afterward.**

Consider adding an element to a queue. The operation must modify the internal vectors, but afterward the queue still exists and should be usable:

```rust
// ❌ Wrong: consuming the queue to push one element
fn push(self, c: char) -> Self {
    let mut new_self = self;
    new_self.younger.push(c);
    new_self
}

// Terrible usage pattern:
q = q.push('a');  // Have to reassign every time
q = q.push('b');  // Awkward!
```

This is the "builder pattern" approach, but it's wrong for mutable operations on existing objects. It forces awkward reassignment and makes method chaining painful.

Taking an immutable reference won't work—you can't modify through `&self`:

```rust
// ❌ Won't compile: can't modify through immutable reference
fn push(&self, c: char) {
    self.younger.push(c);  // ERROR: cannot borrow as mutable
}
```

The solution is `&mut self`—a mutable borrow. This gives the method **exclusive access** to the value for the duration of the call. The caller temporarily gives up all other access (no other borrows can exist), allowing the method to safely modify internal state.

After the method returns, the mutable borrow ends, and the caller regains full access. The object still exists and can be used normally. This is perfect for stateful operations that transform the object in place.

A method taking `&mut self` can:
- Modify any field
- Call other `&mut self` or `&self` methods
- Move values out of fields (replacing with new values)
- Maintain invariants across multiple fields

It requires:
- Exclusive access (no other borrows during the call)
- The caller to have a mutable binding or reference

**Therefore:**

**For methods that modify state but leave the object valid, declare the receiver as `&mut self`. This grants exclusive access for the duration of the call, allowing safe modification, then returns control to the caller with the object still usable.**

```rust
pub struct Queue {
    older: Vec<char>,
    younger: Vec<char>
}

impl Queue {
    /// Add an element to the back of the queue.
    /// Takes &mut self because it modifies internal state.
    pub fn push(&mut self, c: char) {
        self.younger.push(c);
    }

    /// Remove and return the front element.
    /// Takes &mut self - modifies the queue.
    pub fn pop(&mut self) -> Option<char> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }

            // Modify both fields to maintain invariant
            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        self.older.pop()
    }
}

// Usage: clean, natural modification
let mut q = Queue::new();

q.push('a');  // Borrows mutably, then releases
q.push('b');  // Can borrow mutably again
q.push('c');  // Each call is independent

assert_eq!(q.pop(), Some('a'));  // Mutable borrow
assert_eq!(q.pop(), Some('b'));  // Another mutable borrow

// Mutable borrow enforces exclusive access
let item = q.pop();  // OK: single mutable borrow
// let r = &q;       // ERROR: cannot borrow immutably while mutably borrowed
println!("{:?}", item);  // Mutable borrow ends here

// Now immutable borrows work again
let empty = q.is_empty();
```

*[Diagram: Borrowing timeline for &mut self methods:

```
Time →

q.push('a')
  ├─ &mut q borrowed ─┤  (exclusive access)
                      └─ borrow ends

q.push('b')
  ├─ &mut q borrowed ─┤  (exclusive access again)
                      └─ borrow ends

q.is_empty()
  ├─ &q borrowed ─┤  (shared access OK now)
                  └─ borrow ends
```

Exclusive access guarantees:
```
DURING &mut self CALL:
  ✓ Can modify all fields
  ✓ Can call other methods
  ✗ No other borrows exist
  ✗ No data races possible

AFTER &mut self CALL:
  ✓ Object still exists
  ✓ Can borrow again (mut or immut)
  ✓ Invariants maintained
```
]*

The `&mut self` pattern is central to Rust's approach to mutation. Unlike garbage-collected languages where any method might mutate anything at any time, Rust makes mutation **explicit** and **exclusive**.

This exclusivity prevents entire classes of bugs:
- **No iterator invalidation**: Can't modify a collection while iterating (borrow checker prevents it)
- **No concurrent modification**: Can't have two threads mutating the same data
- **No observation during mutation**: Can't read inconsistent state mid-update

The mutable reference is **temporary**. After the method call, the caller can choose to borrow again (mutably or immutably), move the value, or let it go out of scope. The flexibility comes from the fact that the value isn't consumed—just borrowed.

◆ ◆ ◆

Use this for any operation that changes state: `push`, `pop`, `insert`, `remove`, `update`, `clear`, etc. Contrast with **METHOD TAKING SELF BY REFERENCE (33)** for read-only operations. If the operation consumes the value and transforms it into something else, use **METHOD CONSUMING SELF (35)** instead. Methods taking `&mut self` should maintain the object's invariants—don't leave it in an invalid state. Use **MEM SWAP FOR MOVING VALUES (48)** when you need to move values out of fields while mutating. The caller must have a mutable binding: `let mut q = Queue::new();` to call these methods.
