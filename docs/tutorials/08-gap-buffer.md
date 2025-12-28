# Tutorial: Building a Gap Buffer for Text Editing

## Introduction

In this tutorial, you'll learn how to implement a gap buffer - a sophisticated data structure used by text editors like Emacs and many others. Gap buffers provide extremely efficient insertions and deletions at a cursor position, making them ideal for interactive text editing.

This tutorial will challenge your understanding of Rust by working with:
- Unsafe code and raw pointers
- Manual memory management
- Uninitialized memory
- Complex ownership scenarios
- Custom Drop implementations

By the end, you'll have built a production-quality data structure and gained deep insights into Rust's memory model.

## Prerequisites

Before starting this tutorial, you should:
- Have completed the previous data structure tutorials
- Understand Rust's ownership, borrowing, and lifetimes
- Be familiar with Vec<T> internals
- Know basic unsafe Rust concepts
- Understand how text editors work at a high level

## What You'll Build

You'll create a `GapBuffer<T>` that supports:
- **Constant-time insertion** at the cursor position
- **Constant-time deletion** at the cursor position
- **Fast cursor movement** within the buffer
- **Generic types** - works with any type T
- **Safe public API** - unsafe code is encapsulated

## Step 1: Understanding Gap Buffers

### What is a Gap Buffer?

A gap buffer is a dynamic array with an empty "gap" in the middle. The cursor position is always at the gap's start. This design makes insertions and deletions at the cursor extremely fast.

```
Text: "Hello World"
Cursor position: between "Hello" and "World"

Memory layout:
┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐
│ 'H' │ 'e' │ 'l' │ 'l' │ 'o' │ gap │ gap │ gap │ 'W' │ 'o' │ 'r' │ 'l' │ 'd'
└─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘
       ↑                       ↑                   ↑
     before gap            gap start            gap end
```

### Why Use a Gap Buffer?

**Advantages:**
1. **O(1) insertion/deletion at cursor** - Just adjust the gap boundaries
2. **Good cache locality** - Contiguous memory
3. **Simple implementation** - One allocation, no linked lists
4. **Memory efficient** - Only one gap, not many small allocations

**Disadvantages:**
1. **O(n) cursor movement** - May need to shift many elements
2. **Wasted space** - The gap takes memory
3. **Poor for random access editing** - Best when edits are localized

### Comparison to Other Structures

| Structure | Insert at Cursor | Move Cursor | Random Access |
|-----------|------------------|-------------|---------------|
| Gap Buffer | O(1) | O(n) | O(1) |
| Rope | O(log n) | O(log n) | O(log n) |
| Vec | O(n) | O(1) | O(1) |
| Linked List | O(1) | O(n) | O(n) |

Gap buffers are ideal when edits are localized (typical in text editing).

## Step 2: Designing the Structure

### The Basic Structure

```rust
use std::ops::Range;

pub struct GapBuffer<T> {
    // Storage for elements. This has the capacity we need, but its length
    // always remains zero. GapBuffer puts its elements and the gap in this
    // Vec's "unused" capacity.
    storage: Vec<T>,

    // Range of uninitialized elements in the middle of `storage`.
    // Elements before and after this range are always initialized.
    gap: Range<usize>
}
```

**Key Design Decisions:**

1. **Vec with zero length**: We use Vec's capacity but keep length at 0
2. **Raw pointer access**: We manually manage which slots are initialized
3. **Range for gap**: Stores `gap.start..gap.end` indices

### Why Keep Vec Length at Zero?

```rust
// If we used Vec normally:
let mut v = vec![1, 2, 3, 4, 5];
// Inserting in middle requires shifting: O(n)
v.insert(2, 99);  // Shifts [3, 4, 5] right

// With gap buffer:
// The gap is already there, just write into it: O(1)
```

By keeping Vec's length at 0, we prevent Vec from:
- Trying to drop uninitialized memory
- Interfering with our manual memory management
- Running its own logic for insert/remove

### Memory Invariants

Our gap buffer maintains these invariants:

1. Elements in `0..gap.start` are initialized
2. Elements in `gap.start..gap.end` are uninitialized
3. Elements in `gap.end..capacity` are initialized
4. `storage.len()` is always 0
5. `gap.start <= gap.end <= capacity`

## Step 3: Basic Methods

### Constructor

```rust
impl<T> GapBuffer<T> {
    pub fn new() -> GapBuffer<T> {
        GapBuffer {
            storage: Vec::new(),
            gap: 0..0
        }
    }
}
```

Start with an empty Vec and empty gap (0..0).

### Capacity and Length

```rust
impl<T> GapBuffer<T> {
    /// Return the number of elements this GapBuffer could hold without
    /// reallocation.
    pub fn capacity(&self) -> usize {
        self.storage.capacity()
    }

    /// Return the number of elements this GapBuffer currently holds.
    pub fn len(&self) -> usize {
        self.capacity() - self.gap.len()
    }

    /// Return the current insertion position.
    pub fn position(&self) -> usize {
        self.gap.start
    }
}
```

**Understanding len():**
- Total capacity - gap size = actual elements
- Example: capacity=10, gap=3..7 → len = 10 - 4 = 6 elements

### Index Conversion

We need to convert logical indices (0..len) to physical indices (0..capacity), skipping the gap:

```rust
impl<T> GapBuffer<T> {
    /// Return the offset in the buffer of the `index`'th element, taking
    /// the gap into account.
    fn index_to_raw(&self, index: usize) -> usize {
        if index < self.gap.start {
            index
        } else {
            index + self.gap.len()
        }
    }
}
```

**Example:**

```
Logical:  0   1   2   3   4   5
          ↓   ↓   ↓       ↓   ↓   ↓
Physical: 0   1   2  gap  6   7   8
                      3   4   5
```

If `gap = 3..6`:
- Logical index 0 → Physical index 0
- Logical index 2 → Physical index 2
- Logical index 3 → Physical index 6 (skip gap)
- Logical index 5 → Physical index 8

## Step 4: Working with Unsafe Code

### Why Unsafe?

Gap buffers require unsafe code because:
1. We access uninitialized memory
2. We use raw pointers for efficiency
3. We manually track which memory is initialized

**Safety principle:** Encapsulate all unsafe code in private methods with safe public APIs.

### Raw Pointer Helpers

```rust
impl<T> GapBuffer<T> {
    /// Return a pointer to the `index`'th element of the underlying storage,
    /// regardless of the gap.
    ///
    /// Safety: `index` must be a valid index into `self.storage`.
    unsafe fn space(&self, index: usize) -> *const T {
        self.storage.as_ptr().offset(index as isize)
    }

    /// Return a mutable pointer to the `index`'th element of the underlying
    /// storage, regardless of the gap.
    ///
    /// Safety: `index` must be a valid index into `self.storage`.
    unsafe fn space_mut(&mut self, index: usize) -> *mut T {
        self.storage.as_mut_ptr().offset(index as isize)
    }
}
```

**Understanding the Code:**

- `as_ptr()` / `as_mut_ptr()` - Get raw pointer to Vec's buffer
- `.offset(index as isize)` - Pointer arithmetic (unsafe!)
- Returns `*const T` or `*mut T` - Raw pointers

**Safety Contract:**

These methods are `unsafe` because the caller must ensure `index < capacity`. We document this in the safety comment.

## Step 5: Implementing Get

### Safe Element Access

```rust
impl<T> GapBuffer<T> {
    /// Return a reference to the `index`'th element,
    /// or `None` if `index` is out of bounds.
    pub fn get(&self, index: usize) -> Option<&T> {
        let raw = self.index_to_raw(index);
        if raw < self.capacity() {
            unsafe {
                // We just checked `raw` against self.capacity(),
                // and index_to_raw skips the gap, so this is safe.
                Some(&*self.space(raw))
            }
        } else {
            None
        }
    }
}
```

**Breaking Down the Safety:**

1. Convert logical index to physical index
2. Check bounds
3. In unsafe block:
   - Get raw pointer with `space(raw)`
   - Dereference it with `*`
   - Create reference with `&`
4. We know it's safe because:
   - `raw < capacity` (bounds check)
   - `index_to_raw` never returns a gap index
   - Gap indices are uninitialized; non-gap indices are initialized

## Step 6: Moving the Gap

### The Set Position Method

Moving the cursor requires moving the gap:

```rust
impl<T> GapBuffer<T> {
    /// Set the current insertion position to `pos`.
    /// If `pos` is out of bounds, panic.
    pub fn set_position(&mut self, pos: usize) {
        if pos > self.len() {
            panic!("index {} out of range for GapBuffer", pos);
        }

        unsafe {
            let gap = self.gap.clone();
            if pos > gap.start {
                // `pos` falls after the gap. Move the gap right
                // by shifting elements after the gap to before it.
                let distance = pos - gap.start;
                std::ptr::copy(self.space(gap.end),
                               self.space_mut(gap.start),
                               distance);
            } else if pos < gap.start {
                // `pos` falls before the gap. Move the gap left
                // by shifting elements before the gap to after it.
                let distance = gap.start - pos;
                std::ptr::copy(self.space(pos),
                               self.space_mut(gap.end - distance),
                               distance);
            }

            self.gap = pos .. pos + gap.len();
        }
    }
}
```

### Understanding Gap Movement

**Moving gap right:**

```
Before: [a b c _ _ d e f]
            ↑ gap ↑
Move to position 5:
Step 1: Copy "d e" to gap location
        [a b c d e _ _ f]
               ↑ gap ↑
```

**Moving gap left:**

```
Before: [a b c _ _ d e f]
            ↑ gap ↑
Move to position 1:
Step 1: Copy "b c" after the gap
        [a _ _ b c d e f]
           ↑ gap ↑
```

### Using std::ptr::copy

```rust
std::ptr::copy(src, dst, count)
```

This is like C's `memmove`:
- Copies `count` elements from `src` to `dst`
- Handles overlapping regions correctly
- Very fast (optimized to memcpy when possible)

**Safety requirements:**
- Both pointers must be valid
- Must not violate aliasing rules
- Source elements must be initialized
- Destination may be uninitialized

## Step 7: Insertion and Deletion

### Insert Method

```rust
impl<T> GapBuffer<T> {
    /// Insert `elt` at the current insertion position,
    /// and leave the insertion position after it.
    pub fn insert(&mut self, elt: T) {
        if self.gap.len() == 0 {
            self.enlarge_gap();
        }

        unsafe {
            let index = self.gap.start;
            std::ptr::write(self.space_mut(index), elt);
        }
        self.gap.start += 1;
    }
}
```

**Step-by-step:**

1. Check if gap is full (length == 0)
2. If full, enlarge the gap (reallocate)
3. Write element at gap start using `ptr::write`
4. Move gap start forward (shrinking the gap)

**Why std::ptr::write?**

```rust
std::ptr::write(ptr, value)
```

This writes a value to memory without:
- Reading the old value first (it's uninitialized!)
- Dropping the old value (there is none!)

Similar to C's placement new.

### Insert Iterator

```rust
impl<T> GapBuffer<T> {
    /// Insert the elements produced by `iter` at the current insertion
    /// position, and leave the insertion position after them.
    pub fn insert_iter<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item=T>
    {
        for item in iterable {
            self.insert(item)
        }
    }
}
```

This allows convenient insertion of strings:

```rust
let mut buf = GapBuffer::new();
buf.insert_iter("Hello".chars());
```

### Remove Method

```rust
impl<T> GapBuffer<T> {
    /// Remove the element just after the insertion position
    /// and return it, or return `None` if the insertion position
    /// is at the end of the GapBuffer.
    pub fn remove(&mut self) -> Option<T> {
        if self.gap.end == self.capacity() {
            return None;
        }

        let element = unsafe {
            std::ptr::read(self.space(self.gap.end))
        };
        self.gap.end += 1;
        Some(element)
    }
}
```

**Why std::ptr::read?**

```rust
std::ptr::read(ptr) -> T
```

This reads a value from memory without dropping what's there:
- Copies the bytes
- Returns ownership of the value
- Leaves memory uninitialized (caller's responsibility)

After reading, we expand the gap to cover that position, marking it uninitialized.

## Step 8: Enlarging the Gap

### The Enlarge Gap Method

When the gap is full, we need to reallocate:

```rust
impl<T> GapBuffer<T> {
    /// Double the capacity of `self.storage`.
    fn enlarge_gap(&mut self) {
        let mut new_capacity = self.capacity() * 2;
        if new_capacity == 0 {
            // The existing vector is empty.
            // Choose a reasonable starting capacity.
            new_capacity = 4;
        }

        // We have no idea what resizing a Vec does with its "unused"
        // capacity. So just create a new vector and move over the elements.
        let mut new = Vec::with_capacity(new_capacity);
        let after_gap = self.capacity() - self.gap.end;
        let new_gap = self.gap.start .. new.capacity() - after_gap;

        unsafe {
            // Move the elements that fall before the gap.
            std::ptr::copy_nonoverlapping(self.space(0),
                                          new.as_mut_ptr(),
                                          self.gap.start);

            // Move the elements that fall after the gap.
            let new_gap_end = new.as_mut_ptr().offset(new_gap.end as isize);
            std::ptr::copy_nonoverlapping(self.space(self.gap.end),
                                          new_gap_end,
                                          after_gap);
        }

        // This frees the old Vec, but drops no elements,
        // because the Vec's length is zero.
        self.storage = new;
        self.gap = new_gap;
    }
}
```

### Understanding the Algorithm

**Before enlargement:**
```
[a b c _ d e]  capacity=6, gap=3..4
       ↑
```

**After enlargement:**
```
[a b c _ _ _ _ _ _ d e]  capacity=12, gap=3..9
       ↑           ↑
```

**Steps:**

1. Calculate new capacity (double, minimum 4)
2. Create new Vec with that capacity
3. Calculate where the new gap should be
4. Copy elements before gap to new Vec
5. Copy elements after gap to new Vec (at the end)
6. Replace old Vec with new Vec

**Why copy_nonoverlapping?**

```rust
std::ptr::copy_nonoverlapping(src, dst, count)
```

Like `copy`, but assumes no overlap. This is faster (can use memcpy directly) and we know the source and destination don't overlap because they're different allocations.

## Step 9: Implementing Drop

### Manual Cleanup

Since we manage memory manually, we must implement Drop:

```rust
impl<T> Drop for GapBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0 .. self.gap.start {
                std::ptr::drop_in_place(self.space_mut(i));
            }
            for i in self.gap.end .. self.capacity() {
                std::ptr::drop_in_place(self.space_mut(i));
            }
        }
    }
}
```

**Why is this necessary?**

- `storage` is a `Vec<T>` with length 0
- When Vec drops, it only drops elements up to `len` (which is 0)
- Our actual elements are in Vec's "unused" capacity
- We must manually drop them to prevent memory leaks

**What std::ptr::drop_in_place does:**

```rust
std::ptr::drop_in_place(ptr)
```

Runs the destructor for the value at `ptr` without deallocating:
- Calls `T::drop()` if T implements Drop
- Recursively drops fields
- Leaves memory allocated but uninitialized

**Safety:**

- Only drop initialized memory (skip the gap!)
- Drop exactly once (no double-free)

## Step 10: Iterator Implementation

### The Iterator State

```rust
pub struct Iter<'a, T> {
    buffer: &'a GapBuffer<T>,
    pos: usize
}
```

Simple design: store a reference to the buffer and current position.

### Implementing Iterator

```rust
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.pos >= self.buffer.len() {
            None
        } else {
            self.pos += 1;
            self.buffer.get(self.pos - 1)
        }
    }
}
```

This reuses our safe `get` method!

### Implementing IntoIterator

```rust
impl<'a, T: 'a> IntoIterator for &'a GapBuffer<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        Iter { buffer: self, pos: 0 }
    }
}
```

Now you can use `for` loops:

```rust
for element in &buffer {
    println!("{:?}", element);
}
```

## Step 11: Specialized Methods

### String Method for GapBuffer<char>

For char buffers, provide a convenient string method:

```rust
impl GapBuffer<char> {
    pub fn get_string(&self) -> String {
        let mut text = String::new();
        text.extend(self);
        text
    }
}
```

This uses our iterator to collect chars into a String.

### Debug Implementation

```rust
use std::fmt;

impl<T: fmt::Debug> GapBuffer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indices = (0..self.gap.start).chain(self.gap.end .. self.capacity());
        let elements = indices.map(|i| unsafe { &*self.space(i) });
        f.debug_list().entries(elements).finish()
    }
}
```

This creates a debug representation that skips the gap:

```rust
let buf = GapBuffer::from_iter(vec![1, 2, 3]);
println!("{:?}", buf);  // [1, 2, 3]
```

## Step 12: Complete Implementation

Here's the full gap buffer implementation:

```rust
mod gap {
    use std::ops::Range;

    pub struct GapBuffer<T> {
        storage: Vec<T>,
        gap: Range<usize>
    }

    impl<T> GapBuffer<T> {
        pub fn new() -> GapBuffer<T> {
            GapBuffer { storage: Vec::new(), gap: 0..0 }
        }

        pub fn capacity(&self) -> usize {
            self.storage.capacity()
        }

        pub fn len(&self) -> usize {
            self.capacity() - self.gap.len()
        }

        pub fn position(&self) -> usize {
            self.gap.start
        }

        unsafe fn space(&self, index: usize) -> *const T {
            self.storage.as_ptr().offset(index as isize)
        }

        unsafe fn space_mut(&mut self, index: usize) -> *mut T {
            self.storage.as_mut_ptr().offset(index as isize)
        }

        fn index_to_raw(&self, index: usize) -> usize {
            if index < self.gap.start {
                index
            } else {
                index + self.gap.len()
            }
        }

        pub fn get(&self, index: usize) -> Option<&T> {
            let raw = self.index_to_raw(index);
            if raw < self.capacity() {
                unsafe {
                    Some(&*self.space(raw))
                }
            } else {
                None
            }
        }

        pub fn set_position(&mut self, pos: usize) {
            if pos > self.len() {
                panic!("index {} out of range for GapBuffer", pos);
            }

            unsafe {
                let gap = self.gap.clone();
                if pos > gap.start {
                    let distance = pos - gap.start;
                    std::ptr::copy(self.space(gap.end),
                                   self.space_mut(gap.start),
                                   distance);
                } else if pos < gap.start {
                    let distance = gap.start - pos;
                    std::ptr::copy(self.space(pos),
                                   self.space_mut(gap.end - distance),
                                   distance);
                }

                self.gap = pos .. pos + gap.len();
            }
        }

        pub fn insert(&mut self, elt: T) {
            if self.gap.len() == 0 {
                self.enlarge_gap();
            }

            unsafe {
                let index = self.gap.start;
                std::ptr::write(self.space_mut(index), elt);
            }
            self.gap.start += 1;
        }

        pub fn insert_iter<I>(&mut self, iterable: I)
        where
            I: IntoIterator<Item=T>
        {
            for item in iterable {
                self.insert(item)
            }
        }

        pub fn remove(&mut self) -> Option<T> {
            if self.gap.end == self.capacity() {
                return None;
            }

            let element = unsafe {
                std::ptr::read(self.space(self.gap.end))
            };
            self.gap.end += 1;
            Some(element)
        }

        fn enlarge_gap(&mut self) {
            let mut new_capacity = self.capacity() * 2;
            if new_capacity == 0 {
                new_capacity = 4;
            }

            let mut new = Vec::with_capacity(new_capacity);
            let after_gap = self.capacity() - self.gap.end;
            let new_gap = self.gap.start .. new.capacity() - after_gap;

            unsafe {
                std::ptr::copy_nonoverlapping(self.space(0),
                                              new.as_mut_ptr(),
                                              self.gap.start);

                let new_gap_end = new.as_mut_ptr().offset(new_gap.end as isize);
                std::ptr::copy_nonoverlapping(self.space(self.gap.end),
                                              new_gap_end,
                                              after_gap);
            }

            self.storage = new;
            self.gap = new_gap;
        }
    }

    impl<T> Drop for GapBuffer<T> {
        fn drop(&mut self) {
            unsafe {
                for i in 0 .. self.gap.start {
                    std::ptr::drop_in_place(self.space_mut(i));
                }
                for i in self.gap.end .. self.capacity() {
                    std::ptr::drop_in_place(self.space_mut(i));
                }
            }
        }
    }

    pub struct Iter<'a, T> {
        buffer: &'a GapBuffer<T>,
        pos: usize
    }

    impl<'a, T> Iterator for Iter<'a, T> {
        type Item = &'a T;
        fn next(&mut self) -> Option<&'a T> {
            if self.pos >= self.buffer.len() {
                None
            } else {
                self.pos += 1;
                self.buffer.get(self.pos - 1)
            }
        }
    }

    impl<'a, T: 'a> IntoIterator for &'a GapBuffer<T> {
        type Item = &'a T;
        type IntoIter = Iter<'a, T>;
        fn into_iter(self) -> Iter<'a, T> {
            Iter { buffer: self, pos: 0 }
        }
    }

    impl GapBuffer<char> {
        pub fn get_string(&self) -> String {
            let mut text = String::new();
            text.extend(self);
            text
        }
    }

    use std::fmt;
    impl<T: fmt::Debug> fmt::Debug for GapBuffer<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let indices = (0..self.gap.start).chain(self.gap.end .. self.capacity());
            let elements = indices.map(|i| unsafe { &*self.space(i) });
            f.debug_list().entries(elements).finish()
        }
    }
}
```

## Step 13: Testing Your Gap Buffer

### Basic Functionality Test

```rust
#[test]
fn test() {
    use gap::GapBuffer;

    let mut buf = GapBuffer::new();
    buf.insert_iter("Lord of the Rings".chars());
    buf.set_position(12);

    buf.insert_iter("Onion ".chars());

    assert_eq!(buf.get_string(), "Lord of the Onion Rings");
}
```

This test:
1. Creates a buffer
2. Inserts "Lord of the Rings"
3. Moves cursor to position 12 (after "the ")
4. Inserts "Onion "
5. Verifies the result

### More Complex Tests

```rust
#[test]
fn test_misc() {
    use gap::GapBuffer;

    let mut gb = GapBuffer::new();
    println!("{:?}", gb);  // []

    gb.insert("foo".to_string());
    gb.insert("bar".to_string());
    gb.insert("baz".to_string());
    gb.insert("qux".to_string());
    gb.insert("quux".to_string());
    println!("{:?}", gb);  // ["foo", "bar", "baz", "qux", "quux"]

    gb.set_position(2);

    assert_eq!(gb.remove(), Some("baz".to_string()));
    assert_eq!(gb.remove(), Some("qux".to_string()));
    assert_eq!(gb.remove(), Some("quux".to_string()));
    assert_eq!(gb.remove(), None);

    gb.insert("quuux".to_string());

    gb.set_position(0);
    assert_eq!(gb.remove(), Some("foo".to_string()));
    assert_eq!(gb.remove(), Some("bar".to_string()));
    assert_eq!(gb.remove(), Some("quuux".to_string()));
    assert_eq!(gb.remove(), None);
}
```

## Key Concepts Learned

### 1. Unsafe Rust

You learned how to:
- Use raw pointers safely
- Work with uninitialized memory
- Document safety contracts
- Encapsulate unsafety

### 2. Manual Memory Management

```rust
std::ptr::write(ptr, value)  // Initialize memory
std::ptr::read(ptr)          // Read without dropping
std::ptr::copy(src, dst, n)  // Copy memory (may overlap)
std::ptr::copy_nonoverlapping(src, dst, n)  // Copy (no overlap)
std::ptr::drop_in_place(ptr) // Drop without deallocating
```

### 3. Vec Internals

- Capacity vs length
- Working with "unused" capacity
- Why we keep length at 0
- Manual allocation with `Vec::with_capacity`

### 4. Custom Drop Implementation

When managing resources manually, you must:
- Track what's initialized
- Drop everything exactly once
- Clean up in the right order

## Performance Analysis

### Time Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| insert | O(1) | Amortized (reallocation) |
| remove | O(1) | Just adjust gap boundary |
| get | O(1) | Index calculation |
| set_position | O(n) | May shift many elements |
| enlarge_gap | O(n) | Copy all elements |

### Space Complexity

- Memory overhead: Gap size (wasted space)
- Typical gap size: 10-50% of capacity
- Can be tuned based on usage patterns

### Cache Performance

Gap buffers have excellent cache locality:
- Contiguous memory layout
- Sequential access during insertion
- Good prefetching behavior

## Comparison to Other Text Structures

### Gap Buffer

**Pros:**
- Simple implementation
- Fast local edits
- Good cache performance

**Cons:**
- Slow cursor movement
- Wasted gap space
- Poor for many cursors

### Rope (Tree of Strings)

**Pros:**
- Fast everywhere
- Handles large files
- Supports multiple cursors

**Cons:**
- Complex implementation
- More memory overhead
- Worse cache performance

### Simple Vec

**Pros:**
- Simplest implementation
- Perfect cache performance

**Cons:**
- O(n) insertion anywhere
- O(n) deletion anywhere
- Not suitable for text editing

## Common Pitfalls and Solutions

### Pitfall 1: Dropping Uninitialized Memory

```rust
// WRONG - tries to drop gap!
impl<T> Drop for GapBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0 .. self.capacity() {  // Includes gap!
                std::ptr::drop_in_place(self.space_mut(i));
            }
        }
    }
}

// CORRECT - skip the gap
impl<T> Drop for GapBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0 .. self.gap.start {
                std::ptr::drop_in_place(self.space_mut(i));
            }
            for i in self.gap.end .. self.capacity() {
                std::ptr::drop_in_place(self.space_mut(i));
            }
        }
    }
}
```

### Pitfall 2: Reading Uninitialized Memory

```rust
// WRONG - might read from gap
pub fn get(&self, index: usize) -> Option<&T> {
    if index < self.capacity() {
        unsafe { Some(&*self.space(index)) }  // Might be gap!
    } else {
        None
    }
}

// CORRECT - use index_to_raw
pub fn get(&self, index: usize) -> Option<&T> {
    let raw = self.index_to_raw(index);  // Skips gap
    if raw < self.capacity() {
        unsafe { Some(&*self.space(raw)) }
    } else {
        None
    }
}
```

### Pitfall 3: Overlapping Copies

```rust
// WRONG - undefined behavior if regions overlap
std::ptr::copy_nonoverlapping(src, dst, count);

// CORRECT - handles overlap
std::ptr::copy(src, dst, count);
```

Use `copy` when source and destination might overlap, `copy_nonoverlapping` only when you're certain they don't.

## Exercises

### Exercise 1: Implement Clear

Clear all elements from the buffer:

```rust
impl<T> GapBuffer<T> {
    pub fn clear(&mut self) {
        // Hint: Drop all elements and reset gap
        todo!()
    }
}
```

### Exercise 2: Optimize Cursor Movement

Improve `set_position` to move the smaller amount:

```rust
pub fn set_position(&mut self, pos: usize) {
    // Hint: Compare distance to move gap left vs right
    // Choose the shorter movement
    todo!()
}
```

### Exercise 3: Implement Backspace

Delete the character before the cursor:

```rust
impl<T> GapBuffer<T> {
    pub fn backspace(&mut self) -> Option<T> {
        // Hint: Move gap start backward and return that element
        todo!()
    }
}
```

### Exercise 4: Add a Peek Method

Look at the character at the cursor without removing it:

```rust
impl<T> GapBuffer<T> {
    pub fn peek(&self) -> Option<&T> {
        // Hint: Return element at gap.end if it exists
        todo!()
    }
}
```

## Real-World Usage

Gap buffers are used in:

- **GNU Emacs** - The classic text editor
- **Vim** - In some configurations
- **Text editing libraries** - Many use gap buffers internally
- **Terminal emulators** - For scroll-back buffers

## Next Steps

After mastering gap buffers, you can:

1. **Study ropes**: Tree-based text structures
2. **Learn piece tables**: Used in VS Code
3. **Explore CRDT**: For collaborative editing
4. **Implement syntax highlighting**: Using your gap buffer

## Summary

Congratulations! You've built a sophisticated, production-quality data structure using unsafe Rust. You've learned:

- How to design data structures with uninitialized memory
- When and how to use unsafe code responsibly
- Manual memory management techniques
- The trade-offs between different text editing data structures
- How to implement Drop for custom cleanup
- Working with raw pointers and Vec internals

This knowledge gives you deep insight into how Rust's memory model works and prepares you for building high-performance data structures and systems programming.

## Resources

- [The Rustonomicon - Uninitialized Memory](https://doc.rust-lang.org/nomicon/uninitialized.html)
- [std::ptr documentation](https://doc.rust-lang.org/std/ptr/)
- [Emacs Buffer Implementation](https://www.gnu.org/software/emacs/manual/html_node/elisp/Buffer-Internals.html)
- [Text Editor Data Structures](https://cdacamar.github.io/data%20structures/algorithms/benchmarking/text%20editors/c++/editor-data-structures/)
