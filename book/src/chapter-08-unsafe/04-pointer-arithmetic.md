# Pointer Arithmetic and Memory Manipulation

## Raw Pointers: The Foundation

Raw pointers (`*const T` and `*mut T`) are Rust's escape hatch from the borrow checker. Unlike references (`&T` and `&mut T`), raw pointers:

- Can be null
- Don't enforce aliasing rules
- Don't track lifetimes
- Can point to invalid or uninitialized memory
- Don't automatically dereference

This flexibility comes at a cost: you must manually ensure safety when using them.

The `gap-buffer` project demonstrates sophisticated pointer arithmetic for implementing an efficient text editor data structure. Understanding its techniques illuminates the careful reasoning required for correct unsafe code.

## Pointer Arithmetic Basics

Pointer arithmetic in Rust uses the `offset` method:

```rust
unsafe fn space(&self, index: usize) -> *const T {
    self.storage.as_ptr().offset(index as isize)
}

unsafe fn space_mut(&mut self, index: usize) -> *mut T {
    self.storage.as_mut_ptr().offset(index as isize)
}
```

The `offset` method moves the pointer forward (positive offset) or backward (negative offset) by the specified number of elements, not bytes. If `T` is 4 bytes, `offset(2)` moves 8 bytes forward.

### Safety Requirements for offset

The documentation for `offset` specifies strict requirements:

1. Both the starting pointer and the result must be in bounds of the same allocated object
2. The computed offset cannot overflow `isize`
3. The resulting pointer must be properly aligned

Violating any of these is undefined behavior. The `space` and `space_mut` methods document their requirement:

```rust
/// Return a pointer to the `index`'th element of the underlying storage,
/// regardless of the gap.
///
/// Safety: `index` must be a valid index into `self.storage`.
```

Callers must ensure `index` is valid. The methods trust this contract.

## Manual Memory Management

The `GapBuffer` manages memory manually, storing elements in a Vec's "unused" capacity:

```rust
pub struct GapBuffer<T> {
    // Storage for elements. This has the capacity we need, but its length
    // always remains zero. GapBuffer puts its elements and the gap in this
    // `Vec`'s "unused" capacity.
    storage: Vec<T>,

    // Range of uninitialized elements in the middle of `storage`.
    // Elements before and after this range are always initialized.
    gap: Range<usize>
}
```

This design means:
- `storage.len()` is always 0
- `storage.capacity()` is the total space
- Elements before `gap.start` are initialized
- Elements in `gap` are uninitialized
- Elements after `gap.end` are initialized

### Tracking Initialization State

This is critical: Rust doesn't track which memory is initialized. You must.

When inserting an element:

```rust
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
```

We use `std::ptr::write` instead of assignment because:
1. The memory is uninitialized, so "dropping" the old value would be UB
2. `write` just writes bytes without attempting to drop

The gap start moves forward, excluding the newly initialized element.

When removing:

```rust
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
```

We use `std::ptr::read` to move the value out without dropping the source. The gap end moves forward, including the now-uninitialized memory.

### The Drop Implementation

Manual memory management requires manual cleanup:

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

We must drop all initialized elements but skip the gap (which is uninitialized). `drop_in_place` runs the destructor without moving the value. This is essential for types with non-trivial drops.

If we forgot to drop elements, we'd leak memory. If we dropped uninitialized elements, we'd have undefined behavior.

## Buffer Management and Bounds Checking

Correct bounds checking prevents the most common unsafe code bugs: buffer overflows and out-of-bounds access.

### Bounds Checking Strategy

The `GapBuffer::get` method shows defensive bounds checking:

```rust
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
```

The bounds check happens *before* the unsafe block. If `raw` is out of bounds, we return `None` safely. Only in the verified case do we dereference the pointer.

### Index Translation

The gap complicates indexing. A logical index must be translated to a physical index:

```rust
fn index_to_raw(&self, index: usize) -> usize {
    if index < self.gap.start {
        index
    } else {
        index + self.gap.len()
    }
}
```

Indices before the gap map directly. Indices after the gap skip over the gap. This function is safe—it just returns a number. The caller must verify the result is in bounds.

### Enlarge Gap: Complex Pointer Manipulation

When the gap is full, we need to resize:

```rust
fn enlarge_gap(&mut self) {
    let mut new_capacity = self.capacity() * 2;
    if new_capacity == 0 {
        new_capacity = 4;
    }

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

    self.storage = new;
    self.gap = new_gap;
}
```

This function demonstrates several unsafe operations:

1. `copy_nonoverlapping`: Like C's `memcpy`, copies bytes without overlap checking
2. Computing destination pointers via `offset`
3. Reasoning about initialization states across two buffers

The safety argument:
- Source pointers are valid (from `self.space`)
- Destination pointers are valid (within `new`'s capacity)
- Counts are correct (`self.gap.start` and `after_gap`)
- Ranges don't overlap (using `copy_nonoverlapping`)
- Old buffer freed without dropping elements (its length is 0)

## Alignment and Layout Considerations

Memory alignment is often invisible in safe Rust but critical in unsafe code.

### Alignment Requirements

From `ref-with-flag`:

```rust
pub fn new(ptr: &'a T, flag: bool) -> RefWithFlag<T> {
    assert!(align_of::<T>() % 2 == 0);
    RefWithFlag {
        ptr_and_bit: ptr as *const T as usize | flag as usize,
        behaves_like: PhantomData
    }
}
```

The assertion checks that `T` has at least 2-byte alignment. Why?

On most architectures, pointers to aligned types have their lower bits set to zero. For 2-byte alignment, the lowest bit is always 0. This means we can steal that bit to store a boolean flag.

```rust
pub fn get_ref(&self) -> &'a T {
    unsafe {
        let ptr = (self.ptr_and_bit & !1) as *const T;
        &*ptr
    }
}
```

We clear the lowest bit (`& !1`) to recover the pointer. This works only if the bit was 0 in the original pointer, which the alignment guarantees.

### Alignment Misuse

What happens if you violate alignment?

```rust
#[repr(C, packed)]
struct Unaligned {
    x: u8,
    y: u32,  // Might be at an odd address
}

let u = Unaligned { x: 1, y: 2 };
let ptr = &u.y as *const u32;
// ptr might not be 4-byte aligned

unsafe {
    let value = *ptr;  // Possible undefined behavior!
}
```

On some architectures, unaligned loads cause crashes. On others, they're slower or silently wrong. Rust references are always aligned, so this is mostly a concern in FFI or when reinterpreting memory.

### Layout Compatibility

The `repr(C)` attribute ensures compatible layout with C:

```rust
#[repr(C)]
pub struct git_signature {
    pub name: *const c_char,
    pub email: *const c_char,
    pub when: git_time
}
```

Without `repr(C)`, Rust can reorder fields. With it, the layout matches the C struct, allowing safe interop.

## Advanced Pattern: Copy vs Move

Pointer manipulation often involves choosing between copying and moving:

### ptr::read - Move Out

```rust
let element = unsafe {
    std::ptr::read(self.space(self.gap.end))
};
```

This *moves* the value, leaving the source uninitialized. The source must not be dropped or accessed again.

### ptr::write - Move In

```rust
unsafe {
    std::ptr::write(self.space_mut(index), elt);
}
```

This *moves* the value to uninitialized memory. The destination must not be initialized (or you'll skip the drop).

### ptr::copy / ptr::copy_nonoverlapping - Bitwise Copy

```rust
unsafe {
    std::ptr::copy_nonoverlapping(self.space(0),
                                  new.as_mut_ptr(),
                                  self.gap.start);
}
```

This copies bytes without calling drop or clone. Use for:
- Moving elements in bulk
- Implementing custom collections
- When T is Copy (bytes are identical)

These are the primitives for manual memory management. Safe Rust hides them inside `Vec`, `Box`, and other abstractions.

## Practical Advice

When working with pointers:

1. **Verify bounds explicitly**: Check before dereferencing
2. **Track initialization carefully**: Know which memory is valid
3. **Document pointer ownership**: Who frees the memory?
4. **Use helpers for clarity**: `space()` is clearer than raw offsets
5. **Test edge cases**: Empty buffers, full buffers, gap at ends
6. **Use Miri**: Catch alignment and initialization bugs

## Conclusion

Pointer arithmetic and manual memory management are powerful tools for building efficient data structures. The `gap-buffer` project shows that complex pointer manipulation can be safe when:

- Invariants are clearly stated
- Bounds are checked before unsafe operations
- Initialization states are tracked precisely
- Drop implementations clean up correctly
- Testing covers edge cases

These techniques underlie all of Rust's collection types. Understanding them demystifies the standard library and enables building your own safe, efficient abstractions over raw memory.
