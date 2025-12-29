# Pointer Arithmetic Pattern

## Context

You are implementing a data structure that requires managing memory layouts directly: moving blocks of memory, working with uninitialized regions, or implementing custom allocation strategies. Safe Rust's slice indexing is insufficient for your needs.

The `gap-buffer` example demonstrates this: a text editor buffer with a "gap" (uninitialized region) that moves as the insertion point changes, requiring precise pointer arithmetic to shift initialized elements around the gap.

## Problem

**How do you safely perform pointer arithmetic—calculating offsets, copying memory regions, and managing initialized vs. uninitialized memory—without causing undefined behavior?**

Pointer arithmetic allows treating memory as a sequence of bytes or elements, adding offsets to pointers to access different locations. But incorrect arithmetic causes:
- Out-of-bounds access (reading/writing past allocation)
- Alignment violations (accessing unaligned addresses)
- Use of uninitialized memory
- Memory leaks (forgetting to drop elements)

The challenge is to perform pointer arithmetic correctly while maintaining all of Rust's safety invariants manually.

## Forces

- **Performance**: Pointer arithmetic avoids bounds checks and enables bulk operations
- **Flexibility**: Allows implementing data structures impossible with safe slices
- **Correctness**: Off-by-one errors cause memory corruption
- **Complexity**: Tracking initialized/uninitialized regions is error-prone
- **Portability**: Pointer size and arithmetic behavior vary by platform
- **Auditability**: Pointer arithmetic is hard to verify by inspection

These forces conflict: the most efficient implementations use raw pointer arithmetic, but the safest avoid it entirely.

## Solution

**Use pointer arithmetic only when necessary, encapsulate it in small well-tested functions, document invariants clearly, and use `std::ptr` utilities instead of manual calculations.**

Follow this pattern:

1. **Use std::ptr utilities**: Prefer `copy`, `copy_nonoverlapping`, `write`, `read` over manual ops
2. **Bounds checking**: Validate all offsets before arithmetic
3. **Helper functions**: Wrap arithmetic in small, documented functions
4. **Maintain invariants**: Track which memory regions are initialized
5. **Test exhaustively**: Include edge cases (empty, full, boundaries)
6. **Drop correctly**: Clean up initialized elements in Drop

### Example from gap-buffer

A text editor buffer with movable gap for efficient insertion:

```rust
use std::ops::Range;

pub struct GapBuffer<T> {
    // INVARIANT: storage.len() is always 0 (we use capacity only)
    storage: Vec<T>,

    // INVARIANT: Elements in [0..gap.start) are initialized
    // INVARIANT: Elements in [gap.start..gap.end) are uninitialized
    // INVARIANT: Elements in [gap.end..capacity) are initialized
    gap: Range<usize>
}

impl<T> GapBuffer<T> {
    /// Return a pointer to the `index`'th element of storage.
    ///
    /// Safety: `index` must be a valid index into `self.storage`.
    unsafe fn space(&self, index: usize) -> *const T {
        // Use offset, not manual pointer arithmetic
        self.storage.as_ptr().offset(index as isize)
    }

    /// Return a mutable pointer to the `index`'th element.
    ///
    /// Safety: `index` must be a valid index into `self.storage`.
    unsafe fn space_mut(&mut self, index: usize) -> *mut T {
        self.storage.as_mut_ptr().offset(index as isize)
    }

    /// Move the gap to position `pos` by shifting elements.
    pub fn set_position(&mut self, pos: usize) {
        // Validate before any pointer arithmetic
        if pos > self.len() {
            panic!("index {} out of range for GapBuffer", pos);
        }

        unsafe {
            let gap = self.gap.clone();

            if pos > gap.start {
                // Move gap right: shift elements after gap to before it
                // [... gap ... elements ...] -> [... elements ... gap ...]
                let distance = pos - gap.start;

                // SAFETY:
                // - source: gap.end is start of initialized region after gap
                // - dest: gap.start is start of uninitialized gap
                // - count: distance elements to move
                // - regions don't overlap (gap.end > gap.start + distance)
                std::ptr::copy(
                    self.space(gap.end),        // source
                    self.space_mut(gap.start),  // destination
                    distance                     // element count
                );
            } else if pos < gap.start {
                // Move gap left: shift elements before gap to after it
                let distance = gap.start - pos;

                // SAFETY:
                // - source: pos is start of region to move
                // - dest: gap.end - distance is where to move them
                // - count: distance elements
                // - regions don't overlap
                std::ptr::copy(
                    self.space(pos),
                    self.space_mut(gap.end - distance),
                    distance
                );
            }

            // Update gap position
            self.gap = pos .. pos + gap.len();
        }
    }

    /// Insert element at gap position.
    pub fn insert(&mut self, elt: T) {
        if self.gap.len() == 0 {
            self.enlarge_gap();
        }

        unsafe {
            let index = self.gap.start;
            // SAFETY: index is within the gap (uninitialized region)
            // We're initializing this slot
            std::ptr::write(self.space_mut(index), elt);
        }
        self.gap.start += 1;
    }

    /// Remove element after gap.
    pub fn remove(&mut self) -> Option<T> {
        if self.gap.end == self.capacity() {
            return None;
        }

        let element = unsafe {
            // SAFETY: gap.end is the first initialized element after gap
            std::ptr::read(self.space(self.gap.end))
        };
        self.gap.end += 1;
        Some(element)
    }

    /// Double the buffer capacity, moving elements around new gap.
    fn enlarge_gap(&mut self) {
        let mut new_capacity = self.capacity() * 2;
        if new_capacity == 0 {
            new_capacity = 4;
        }

        let mut new = Vec::with_capacity(new_capacity);
        let after_gap = self.capacity() - self.gap.end;
        let new_gap = self.gap.start .. new.capacity() - after_gap;

        unsafe {
            // Copy elements before gap to same position in new buffer
            // SAFETY:
            // - copying gap.start initialized elements
            // - source and dest don't overlap (different allocations)
            std::ptr::copy_nonoverlapping(
                self.space(0),
                new.as_mut_ptr(),
                self.gap.start
            );

            // Copy elements after gap to end of new buffer
            // SAFETY:
            // - source: gap.end in old buffer (initialized)
            // - dest: new_gap.end in new buffer (uninitialized)
            // - count: after_gap elements
            // - no overlap (different allocations)
            let new_gap_end = new.as_mut_ptr().offset(new_gap.end as isize);
            std::ptr::copy_nonoverlapping(
                self.space(self.gap.end),
                new_gap_end,
                after_gap
            );
        }

        // Replace old buffer (Vec's drop handles deallocation)
        // No elements dropped because storage.len() is 0
        self.storage = new;
        self.gap = new_gap;
    }
}

impl<T> Drop for GapBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            // Drop all initialized elements (skip the gap)
            // SAFETY: Elements before gap are initialized
            for i in 0 .. self.gap.start {
                std::ptr::drop_in_place(self.space_mut(i));
            }
            // SAFETY: Elements after gap are initialized
            for i in self.gap.end .. self.capacity() {
                std::ptr::drop_in_place(self.space_mut(i));
            }
        }
    }
}
```

Key safety features:
- **Bounds validation**: Checks `pos <= self.len()` before any pointer arithmetic
- **std::ptr utilities**: Uses `copy`, `write`, `read`, `drop_in_place` instead of manual operations
- **Documented regions**: Comments explain which memory is initialized
- **Correct Drop**: Drops only initialized elements, skips gap

### Pointer Arithmetic Utilities

Always prefer these over manual calculations:

```rust
use std::ptr;

// ✅ GOOD: Use these
ptr::copy(src, dst, count)              // memcpy (may overlap)
ptr::copy_nonoverlapping(src, dst, count) // memcpy (must not overlap)
ptr::write(ptr, value)                  // Initialize memory
ptr::read(ptr)                          // Move out of memory
ptr::drop_in_place(ptr)                 // Drop in place
ptr.offset(count)                       // Pointer arithmetic

// ❌ AVOID: Manual arithmetic
let ptr2 = (ptr as usize + offset) as *const T;  // Easy to get wrong!
```

## Resulting Context

### Benefits

- **Performance**: Bulk operations without bounds checks
- **Flexibility**: Implement data structures impossible with safe code
- **Control**: Precise control over memory layout and initialization
- **Zero-cost**: No runtime overhead compared to C

### Liabilities

- **Unsafe**: All pointer arithmetic requires unsafe blocks
- **Complex**: Tracking initialization state is manual and error-prone
- **Testing burden**: Must test every edge case exhaustively
- **Maintenance**: Easy to introduce bugs when modifying
- **Platform-specific**: Pointer sizes and alignment differ across targets

### Common Mistakes

1. **Off-by-one errors**: `offset(len)` vs `offset(len - 1)`
2. **Overlapping copy**: Using `copy_nonoverlapping` with overlapping regions
3. **Forgetting to drop**: Memory leaks from undroped initialized elements
4. **Double drop**: Dropping same element twice
5. **Wrong element count**: Using byte count instead of element count

## Related Patterns

- **Raw Pointer Manipulation**: Foundation for pointer arithmetic
- **Safety Invariants**: Maintaining initialization invariants is critical
- **Resource Cleanup**: Drop implementation must handle initialization state

## Known Uses

- **gap-buffer**: Movable gap in text buffer
- **std::vec::Vec**: Manages uninitialized capacity
- **std::collections::VecDeque**: Ring buffer with pointer arithmetic
- **std::string::String**: Grows capacity with pointer arithmetic
- **smallvec**: Inline storage with pointer arithmetic
- **arrayvec**: Fixed capacity with pointer tracking

## Implementation Notes

### Calculating Offsets Safely

```rust
// ✅ GOOD: Use offset with element count
unsafe {
    let ptr = base_ptr.offset(5);  // Advances 5 * sizeof(T) bytes
}

// ❌ BAD: Manual byte arithmetic
unsafe {
    let ptr = (base_ptr as usize + 5 * std::mem::size_of::<T>()) as *const T;
}
```

### Copy vs. Copy_nonoverlapping

```rust
use std::ptr;

// Use copy when regions might overlap
unsafe {
    // [1, 2, 3, 4, 5] -> [1, 1, 2, 3, 4]
    // source [0..4) overlaps with dest [1..5)
    ptr::copy(arr.as_ptr(), arr.as_mut_ptr().offset(1), 4);
}

// Use copy_nonoverlapping when regions definitely don't overlap
unsafe {
    // Different allocations, so can't overlap
    ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), len);
}

// ❌ WRONG: Using copy_nonoverlapping with overlap is UB!
unsafe {
    ptr::copy_nonoverlapping(arr.as_ptr(), arr.as_mut_ptr().offset(1), 4);
    // Undefined behavior!
}
```

### Tracking Initialization

```rust
struct MyBuffer<T> {
    data: *mut T,
    capacity: usize,
    len: usize,  // Elements [0..len) are initialized
}

impl<T> MyBuffer<T> {
    fn push(&mut self, value: T) {
        assert!(self.len < self.capacity);
        unsafe {
            // Write to uninitialized position
            ptr::write(self.data.offset(self.len as isize), value);
            self.len += 1;  // Now this position is initialized
        }
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;  // This position is now uninitialized
            unsafe {
                // Read from previously initialized position
                Some(ptr::read(self.data.offset(self.len as isize)))
            }
        }
    }
}

impl<T> Drop for MyBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            // Drop all initialized elements
            for i in 0..self.len {
                ptr::drop_in_place(self.data.offset(i as isize));
            }
            // Deallocate (using Vec or similar)
        }
    }
}
```

### Bounds Checking Before Arithmetic

```rust
impl<T> GapBuffer<T> {
    fn get(&self, index: usize) -> Option<&T> {
        // Convert logical index to physical index
        let raw = self.index_to_raw(index);

        // Bounds check BEFORE any pointer arithmetic
        if raw < self.capacity() {
            unsafe {
                // SAFETY: We just verified raw is in bounds
                Some(&*self.space(raw))
            }
        } else {
            None
        }
    }

    fn index_to_raw(&self, index: usize) -> usize {
        // Skip the gap when calculating physical index
        if index < self.gap.start {
            index
        } else {
            index + self.gap.len()
        }
    }
}
```

## Testing Pointer Arithmetic

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_buffer_operations() {
        let mut buf = GapBuffer::new();

        // Test insertion
        buf.insert('a');
        buf.insert('b');
        buf.insert('c');
        assert_eq!(buf.len(), 3);

        // Test gap movement
        buf.set_position(1);
        buf.insert('X');
        // Should be: a X b c

        // Verify all elements
        assert_eq!(buf.get(0), Some(&'a'));
        assert_eq!(buf.get(1), Some(&'X'));
        assert_eq!(buf.get(2), Some(&'b'));
        assert_eq!(buf.get(3), Some(&'c'));
    }

    #[test]
    fn test_gap_at_boundaries() {
        let mut buf = GapBuffer::new();
        buf.insert('a');
        buf.insert('b');

        // Gap at start
        buf.set_position(0);
        buf.insert('X');

        // Gap at end
        buf.set_position(buf.len());
        buf.insert('Y');
    }

    #[test]
    fn test_drop_cleans_up() {
        let mut buf = GapBuffer::new();
        buf.insert(String::from("hello"));
        buf.insert(String::from("world"));
        // Drop should clean up both strings
    }
}
```

## Common Pitfalls

### Pitfall 1: Wrong Offset Type

```rust
// ❌ WRONG: Using usize for offset
unsafe {
    let ptr2 = ptr.offset(index);  // Error if index is usize!
}

// ✅ RIGHT: Cast to isize
unsafe {
    let ptr2 = ptr.offset(index as isize);
}
```

### Pitfall 2: Forgetting Element Size

```rust
// ❌ WRONG: Treating offset as bytes
let offset = 5;  // Want 5th element
unsafe {
    let ptr2 = (ptr as usize + offset) as *const T;  // Only advances 5 bytes!
}

// ✅ RIGHT: Use offset method
unsafe {
    let ptr2 = ptr.offset(5);  // Correctly advances 5 * sizeof(T) bytes
}
```

### Pitfall 3: Overlapping Regions with copy_nonoverlapping

```rust
let mut arr = [1, 2, 3, 4, 5];

// ❌ WRONG: Overlapping regions
unsafe {
    ptr::copy_nonoverlapping(
        arr.as_ptr(),
        arr.as_mut_ptr().offset(1),
        4  // Copies 4 elements, regions overlap!
    );  // UNDEFINED BEHAVIOR
}

// ✅ RIGHT: Use copy for overlapping regions
unsafe {
    ptr::copy(arr.as_ptr(), arr.as_mut_ptr().offset(1), 4);
}
```

### Pitfall 4: Not Dropping Elements

```rust
impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        // ❌ WRONG: Forgetting to drop elements
        unsafe {
            // Just deallocate buffer
            dealloc(self.ptr);
        }  // Elements never dropped - memory leak!

        // ✅ RIGHT: Drop all elements first
        unsafe {
            for i in 0..self.len {
                ptr::drop_in_place(self.ptr.offset(i as isize));
            }
            dealloc(self.ptr);
        }
    }
}
```

## Further Reading

- *The Rustonomicon* - "Working with Uninitialized Memory"
- Rust documentation: `std::ptr` module
- Blog: "Learning Rust With Entirely Too Many Linked Lists"
- Book: "The Rustonomicon" - Chapter on implementing Vec
