# Raw Pointer Manipulation Pattern

## Context

You are implementing a low-level data structure or optimization that requires direct memory access without Rust's ownership and borrowing rules. You need to work with addresses as integers, perform pointer arithmetic, or temporarily bypass the borrow checker.

The `ref-with-flag` example demonstrates this: packing a reference and a boolean flag into a single machine word by exploiting pointer alignment and manipulating the raw pointer bits.

## Problem

**How do you safely manipulate raw pointers—casting to integers, performing bit operations, and converting back to references—without causing undefined behavior?**

Raw pointers (`*const T` and `*mut T`) allow operations the borrow checker cannot verify: dereferencing potentially invalid addresses, violating aliasing rules, and accessing uninitialized memory. Yet certain optimizations and data structures require these capabilities.

The challenge is to use raw pointers correctly, maintaining the invariants that Rust's type system normally enforces automatically.

## Forces

- **Performance**: Raw pointers enable optimizations impossible with safe references
- **Flexibility**: Can temporarily break borrowing rules when you know it's safe
- **Danger**: Dereferencing invalid pointers causes undefined behavior
- **Aliasing**: Raw pointers don't enforce Rust's aliasing rules
- **Lifetimes**: Raw pointers lack lifetime information, risking dangling references
- **Complexity**: Pointer manipulation is error-prone and hard to review

These forces conflict: maximum performance requires raw pointers, but maximum safety requires avoiding them.

## Solution

**Use raw pointers only when necessary, validate all invariants before dereferencing, maintain clear ownership semantics, and convert back to safe references at the earliest opportunity.**

Follow this pattern:

1. **Convert to raw**: Use `as *const T` or `as *mut T` from references
2. **Maintain validity**: Ensure pointers remain valid (point to live memory)
3. **Check null**: Verify pointers are non-null before dereferencing
4. **Check alignment**: Ensure proper alignment for the target type
5. **Dereference minimally**: Keep unsafe blocks small
6. **Document**: Explain what makes the pointer dereference safe

### Example from ref-with-flag

This type packs a reference and boolean into one word using pointer bit manipulation:

```rust
use std::marker::PhantomData;
use std::mem::align_of;

/// A `&T` and a `bool`, wrapped up in a single word.
/// The type `T` must require at least two-byte alignment.
pub struct RefWithFlag<'a, T> {
    // INVARIANT: The upper bits contain a valid pointer to T.
    // INVARIANT: The low bit contains the boolean flag.
    // INVARIANT: T must have alignment >= 2.
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>  // For lifetime tracking
}

impl<'a, T: 'a> RefWithFlag<'a, T> {
    pub fn new(ptr: &'a T, flag: bool) -> RefWithFlag<T> {
        // Enforce alignment invariant - this must be checked!
        assert!(align_of::<T>() % 2 == 0,
               "RefWithFlag requires types with 2-byte alignment");

        RefWithFlag {
            // Convert reference to raw pointer, then to integer
            // OR in the flag bit (safe because alignment guarantees bit 0 is free)
            ptr_and_bit: ptr as *const T as usize | flag as usize,
            behaves_like: PhantomData
        }
    }

    pub fn get_ref(&self) -> &'a T {
        unsafe {
            // SAFETY: The constructor ensures T has at least 2-byte alignment,
            // so the low bit of the original pointer was 0. Masking with !1
            // recovers the original pointer value. The PhantomData maintains
            // the correct lifetime 'a, and the pointer is valid for 'a because
            // it was derived from &'a T.
            let ptr = (self.ptr_and_bit & !1) as *const T;
            &*ptr
        }
    }

    pub fn get_flag(&self) -> bool {
        // Extract low bit - no unsafe needed, just bit manipulation
        self.ptr_and_bit & 1 != 0
    }
}
```

Key safety features:
- **Alignment check**: Verifies T's alignment allows stealing the low bit
- **Valid origin**: Pointer comes from a valid reference with lifetime 'a
- **Reconstruction**: Correctly masks out the flag bit before dereferencing
- **Lifetime tracking**: PhantomData preserves the lifetime relationship

### Example: Pointer Arithmetic from gap-buffer

Working with raw pointers to perform pointer arithmetic:

```rust
impl<T> GapBuffer<T> {
    /// Return a pointer to the `index`'th element of storage.
    ///
    /// Safety: `index` must be a valid index into `self.storage`.
    unsafe fn space(&self, index: usize) -> *const T {
        // Convert Vec's internal pointer to raw, add offset
        self.storage.as_ptr().offset(index as isize)
    }

    /// Return a mutable pointer to the `index`'th element.
    ///
    /// Safety: `index` must be a valid index into `self.storage`.
    unsafe fn space_mut(&mut self, index: usize) -> *mut T {
        self.storage.as_mut_ptr().offset(index as isize)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let raw = self.index_to_raw(index);
        // Bounds check BEFORE dereferencing
        if raw < self.capacity() {
            unsafe {
                // SAFETY: We just checked `raw` is within capacity,
                // and index_to_raw skips the gap, so the element
                // at this position is initialized.
                Some(&*self.space(raw))
            }
        } else {
            None
        }
    }
}
```

The pattern:
- **Helper functions**: Wrap pointer arithmetic in small, documented functions
- **Bounds checking**: Always validate indices before dereferencing
- **Minimal unsafe**: Unsafe is only at the point of dereference

### Converting Between Raw Pointers and References

```rust
// Safe reference to raw pointer (always safe)
let r: &T = &value;
let ptr: *const T = r as *const T;

// Raw pointer back to reference (UNSAFE - must verify validity)
unsafe {
    // SAFETY: ptr was derived from a valid reference that
    // is still alive, so this is safe.
    let r2: &T = &*ptr;
}
```

### Null Pointer Checks

Always check for null before dereferencing:

```rust
unsafe fn use_pointer(ptr: *const T) -> Option<&T> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: We just checked ptr is non-null, and the caller
        // guarantees it points to valid memory.
        Some(&*ptr)
    }
}
```

## Resulting Context

### Benefits

- **Space efficiency**: Pack multiple values into single words
- **Performance**: Avoid bounds checks when you've already validated
- **Low-level control**: Implement data structures impossible with safe code
- **Explicit**: Raw pointer operations make memory access patterns visible

### Liabilities

- **Unsafe**: All raw pointer dereferences require unsafe blocks
- **No borrow checking**: Must manually track aliasing and mutability
- **Easy to break**: Small mistakes cause memory corruption
- **Platform-specific**: Pointer size and alignment vary by platform
- **Hard to audit**: Reviewers must verify complex invariants

### Safety Checklist for Raw Pointers

Before dereferencing `ptr`:

- [ ] Is `ptr` non-null?
- [ ] Does `ptr` point to a valid instance of T?
- [ ] Is `ptr` properly aligned for T?
- [ ] Is the memory at `ptr` initialized (if reading)?
- [ ] Am I respecting aliasing rules (no `&mut` aliases)?
- [ ] Will the memory remain valid for the reference lifetime?

## Related Patterns

- **Safety Invariants**: Raw pointers require explicit invariants about validity
- **Pointer Arithmetic**: Extends this pattern with offset operations
- **FFI Bindings**: C interop requires extensive raw pointer manipulation

## Known Uses

- **ref-with-flag**: Packs reference and boolean using pointer bit manipulation
- **gap-buffer**: Uses raw pointers for uninitialized memory management
- **std::vec::Vec**: Manages raw pointer to heap allocation
- **std::collections::LinkedList**: Uses raw pointers for node links
- **std::rc::Rc**: Uses raw pointers for reference counting
- **parking_lot**: Uses raw pointers for lock-free algorithms

## Implementation Notes

### PhantomData for Lifetimes

When storing raw pointers, use `PhantomData` to track lifetimes:

```rust
use std::marker::PhantomData;

struct MyPointer<'a, T> {
    ptr: *const T,
    // Acts as if we're storing &'a T, for lifetime checking
    _marker: PhantomData<&'a T>
}
```

Without `PhantomData`, the compiler doesn't track that `MyPointer` holds a reference to data with lifetime `'a`.

### Variance and PhantomData

```rust
// Covariant in 'a (can accept shorter lifetimes)
struct Covariant<'a, T> {
    ptr: *const T,
    _marker: PhantomData<&'a T>
}

// Invariant in 'a (must match exactly)
struct Invariant<'a, T> {
    ptr: *mut T,
    _marker: PhantomData<&'a mut T>
}
```

Match the `PhantomData` type to your intended semantics.

### Alignment Utilities

```rust
use std::mem::align_of;

/// Check if a pointer is properly aligned for T.
fn is_aligned<T>(ptr: *const T) -> bool {
    (ptr as usize) % align_of::<T>() == 0
}

/// Get the number of low bits guaranteed to be zero.
fn alignment_bits<T>() -> u32 {
    align_of::<T>().trailing_zeros()
}

// Example: u64 has 8-byte alignment, so 3 low bits are always 0
assert_eq!(alignment_bits::<u64>(), 3);
```

### Safe Pointer Creation

```rust
use std::ptr;

// Create null pointer (safe, but dereferencing it is UB)
let ptr: *const i32 = ptr::null();
let ptr_mut: *mut i32 = ptr::null_mut();

// Create from reference (safe)
let value = 42;
let ptr: *const i32 = &value;

// Create from Box (safe)
let boxed = Box::new(42);
let ptr: *const i32 = Box::into_raw(boxed);
// Must eventually: unsafe { Box::from_raw(ptr) } to free
```

### Common Operations

```rust
use std::ptr;

// Safe operations (no unsafe needed):
ptr.is_null()                    // Check for null
ptr1 == ptr2                     // Compare pointers
ptr.cast::<U>()                  // Cast to different type
ptr::null::<T>()                 // Create null pointer

// Unsafe operations:
unsafe { *ptr }                  // Dereference
unsafe { &*ptr }                 // Convert to reference
unsafe { ptr.offset(n) }         // Pointer arithmetic
unsafe { ptr.write(value) }      // Write through pointer
unsafe { ptr.read() }            // Read through pointer
```

## Common Pitfalls

### Pitfall 1: Dereferencing After Free

```rust
let ptr = {
    let value = 42;
    &value as *const i32
}; // value dropped here

unsafe {
    // ❌ WRONG: ptr now points to freed stack memory
    println!("{}", *ptr);
}
```

### Pitfall 2: Breaking Aliasing Rules

```rust
let mut value = 42;
let ptr1 = &mut value as *mut i32;
let ptr2 = &mut value as *mut i32;

unsafe {
    // ❌ WRONG: Creating two mutable references to same location
    *ptr1 = 1;
    *ptr2 = 2;  // Undefined behavior!
}
```

### Pitfall 3: Misaligned Pointers

```rust
let bytes = [0u8; 8];
let ptr = bytes.as_ptr() as *const u64;

unsafe {
    // ❌ WRONG: ptr might not be 8-byte aligned
    // (u64 requires 8-byte alignment)
    let _value = *ptr;  // May crash or silently corrupt
}
```

### Pitfall 4: Wrong Pointer Arithmetic

```rust
let array = [1, 2, 3, 4, 5];
let ptr = array.as_ptr();

unsafe {
    // ❌ WRONG: offset is in elements, not bytes
    let ptr2 = (ptr as usize + 1) as *const i32;  // Only advances 1 byte!

    // ✅ RIGHT: use .offset() for element-wise arithmetic
    let ptr2 = ptr.offset(1);  // Correctly advances sizeof(i32) bytes
}
```

## Testing Raw Pointer Code

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_with_flag() {
        let vec = vec![10, 20, 30];
        let flagged = RefWithFlag::new(&vec, true);

        // Verify pointer round-trips correctly
        assert_eq!(flagged.get_ref(), &vec);
        assert_eq!(flagged.get_flag(), true);

        // Test with false flag
        let unflagged = RefWithFlag::new(&vec, false);
        assert_eq!(unflagged.get_flag(), false);
    }

    #[test]
    fn test_alignment() {
        // These types have sufficient alignment
        let _: RefWithFlag<u16> = RefWithFlag::new(&42u16, true);
        let _: RefWithFlag<String> = RefWithFlag::new(&String::new(), false);
    }

    #[test]
    #[should_panic(expected = "alignment")]
    fn test_insufficient_alignment() {
        // u8 has 1-byte alignment, should panic
        let _: RefWithFlag<u8> = RefWithFlag::new(&42u8, true);
    }
}
```

## Further Reading

- *The Rustonomicon* - "Working with Uninitialized Memory"
- Rust Reference - "Raw Pointers" section
- Blog: "Learning Rust With Entirely Too Many Linked Lists" - Raw pointer patterns
- RFC 2585 - Unsafe code guidelines
