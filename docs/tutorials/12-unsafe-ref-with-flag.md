# Tutorial: Unsafe Rust and Pointer Manipulation

## Introduction

In this tutorial, you'll venture into Rust's `unsafe` territory by building a `RefWithFlag` type that packs a reference and a boolean flag into a single word. This demonstrates advanced low-level programming techniques while maintaining Rust's safety guarantees where possible.

### What You'll Learn

- Understanding `unsafe` Rust and when to use it
- Pointer representation and memory alignment
- Bit manipulation and pointer tagging
- The `PhantomData` type and its purpose
- Documenting safety invariants
- Creating safe abstractions over unsafe code
- Memory layout and alignment guarantees

### Prerequisites

- Solid understanding of Rust ownership and references
- Basic knowledge of binary representation and bitwise operations
- Familiarity with memory addresses and pointers
- Understanding of Rust lifetimes

### Warning

This tutorial covers advanced, unsafe code. In production, prefer safe abstractions. The techniques shown here are typically used in:
- Low-level data structure implementations
- Memory allocators
- High-performance libraries
- Systems programming

**Python developers:** This has no direct Python equivalent - Python manages all memory for you.

## The Problem: Wasted Space

Consider storing a reference and a boolean:

```rust
struct RefAndFlag<'a, T> {
    reference: &'a T,
    flag: bool,
}
```

**On a 64-bit system:**
- `&T` (reference): 8 bytes
- `bool`: 1 byte, but padded to 8 bytes for alignment
- **Total: 16 bytes**

But we only need 9 bits of information! Can we do better?

## The Insight: Exploiting Alignment

**Key fact:** Most types in Rust have alignment requirements:
- `i32`: 4-byte aligned (addresses are multiples of 4)
- `i64`: 8-byte aligned (addresses are multiples of 8)
- Structs: aligned to their largest field

**This means:** For 2-byte aligned types, addresses always end in binary `0`. We can steal that last bit to store our boolean flag!

**Example addresses for 2-byte aligned values:**
```
0x1000  (binary: ...0000)  ✓ Valid address
0x1002  (binary: ...0010)  ✓ Valid address
0x1004  (binary: ...0100)  ✓ Valid address
0x1001  (binary: ...0001)  ✗ Invalid (odd address)
0x1003  (binary: ...0011)  ✗ Invalid (odd address)
```

The last bit is always 0, so we can use it for our flag!

## Step 1: Define the Type

```rust
use std::marker::PhantomData;

/// A `&T` and a `bool`, wrapped up in a single word.
/// The type `T` must require at least two-byte alignment.
pub struct RefWithFlag<'a, T> {
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>,  // occupies no space
}
```

**Understanding the fields:**

1. **`ptr_and_bit: usize`**: Stores both the pointer and the flag
   - Most significant bits: pointer address
   - Least significant bit: boolean flag
   - Size: 8 bytes on 64-bit systems

2. **`behaves_like: PhantomData<&'a T>`**: A zero-sized marker
   - Tells the compiler we "act like" we hold `&'a T`
   - Ensures lifetime `'a` is checked
   - Ensures variance is correct
   - **Takes zero bytes** - pure compile-time information

**Why PhantomData?**

Without it, the compiler doesn't know this type "uses" `T` and `'a`:

```rust
// Without PhantomData - WRONG!
struct RefWithFlag<'a, T> {
    ptr_and_bit: usize,  // Compiler: "Where's the T? Where's 'a?"
}
```

The compiler would:
- Not ensure `T` is used correctly
- Not check lifetime `'a` properly
- Allow wrong variance (e.g., allowing covariance where it shouldn't)

**Python Comparison:**

Python has no concept of alignment, pointers, or this kind of optimization. Everything is a reference to a heap object, and Python manages all memory layout.

## Step 2: The Constructor

```rust
use std::mem::align_of;

impl<'a, T: 'a> RefWithFlag<'a, T> {
    pub fn new(ptr: &'a T, flag: bool) -> RefWithFlag<'a, T> {
        // Ensure T has at least 2-byte alignment
        assert!(align_of::<T>() % 2 == 0);

        RefWithFlag {
            ptr_and_bit: ptr as *const T as usize | flag as usize,
            behaves_like: PhantomData,
        }
    }
}
```

**Understanding each part:**

### The Assertion

```rust
assert!(align_of::<T>() % 2 == 0);
```

- `align_of::<T>()`: Returns the alignment requirement of `T`
- `% 2 == 0`: Checks if it's a multiple of 2 (even number)
- **Ensures** the last bit of addresses is always 0

**Examples:**
- `align_of::<i32>()` = 4 ✓ (4 % 2 == 0)
- `align_of::<u64>()` = 8 ✓ (8 % 2 == 0)
- `align_of::<u8>()` = 1 ✗ (1 % 2 != 0) - Would panic!

### The Conversion Chain

```rust
ptr as *const T as usize | flag as usize
```

Let's break this down step by step:

1. **`ptr: &'a T`**: A reference (safe)
2. **`as *const T`**: Convert to a raw pointer (entering unsafe territory)
3. **`as usize`**: Convert pointer to an integer (its memory address)
4. **`flag as usize`**: Convert `bool` to `usize` (0 or 1)
5. **`|` (bitwise OR)**: Combine pointer and flag

**Example:**
```rust
// Suppose ptr's address is 0x1000 and flag is true
ptr = 0x1000  (binary: ...0001_0000_0000_0000)
flag = true   (binary: ...0000_0000_0000_0001)

ptr_and_bit = 0x1000 | 0x0001
            = 0x1001  (binary: ...0001_0000_0000_0001)
                                                    ^-- flag bit
```

**Why this works:** Since the last bit of `ptr` is always 0 (due to alignment), OR-ing with the flag bit sets it without affecting the pointer.

## Step 3: Extracting the Reference

```rust
impl<'a, T: 'a> RefWithFlag<'a, T> {
    pub fn get_ref(&self) -> &'a T {
        unsafe {
            // Clear the flag bit to get back the original pointer
            let ptr = (self.ptr_and_bit & !1) as *const T;
            &*ptr
        }
    }
}
```

**Understanding the extraction:**

### Clearing the Flag Bit

```rust
self.ptr_and_bit & !1
```

- **`!1`**: Bitwise NOT of 1
  - `1` = `...0000_0001`
  - `!1` = `...1111_1110`
- **`&` (bitwise AND)**: Clears the last bit

**Example:**
```rust
ptr_and_bit = 0x1001  (binary: ...0001_0000_0000_0001)
!1          = 0xFFFE  (binary: ...1111_1111_1111_1110)

result      = 0x1000  (binary: ...0001_0000_0000_0000)
                                                    ^-- cleared!
```

### Converting Back to Reference

```rust
as *const T    // Integer to raw pointer
&*ptr          // Dereference and re-borrow
```

**This is unsafe because:**
- We're creating a pointer from an integer
- We're dereferencing a raw pointer
- We must ensure the pointer is valid

**Safety requirements we must uphold:**
1. The address is still valid (T hasn't been dropped)
2. The address is properly aligned
3. The lifetime `'a` is still valid

## Step 4: Extracting the Flag

```rust
impl<'a, T: 'a> RefWithFlag<'a, T> {
    pub fn get_flag(&self) -> bool {
        self.ptr_and_bit & 1 != 0
    }
}
```

**Understanding the extraction:**

```rust
self.ptr_and_bit & 1
```

- **`& 1`**: Isolates the last bit
- **`!= 0`**: Converts to `bool`

**Example:**
```rust
ptr_and_bit = 0x1001  (binary: ...0001_0000_0000_0001)
& 1         = 0x0001  (binary: ...0000_0000_0000_0001)

result = 1 != 0 = true
```

**This is safe!** We're just reading bits, no pointer dereferencing.

## Step 5: Understanding PhantomData

```rust
use std::marker::PhantomData;

pub struct RefWithFlag<'a, T> {
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>,
}
```

**What PhantomData does:**

### 1. Lifetime Checking

```rust
fn example<'a, 'b>(x: &'a i32, y: &'b i32) -> RefWithFlag<'a, i32> {
    RefWithFlag::new(y, true)  // Error! 'b doesn't outlive 'a
}
```

Without `PhantomData<&'a T>`, the compiler wouldn't catch this lifetime error.

### 2. Variance

PhantomData ensures proper variance:

```rust
// &'a T is covariant in 'a
// (if 'long outlives 'short, &'long T can be used as &'short T)

let long_lived = 42;
{
    let ref_with_flag = RefWithFlag::new(&long_lived, true);
    // Can treat as if it has a shorter lifetime
}
```

### 3. Drop Check

PhantomData helps the compiler understand drop order:

```rust
impl<'a, T> Drop for RefWithFlag<'a, T> {
    fn drop(&mut self) {
        // Compiler knows we're "using" &'a T here
    }
}
```

### 4. Zero Size

```rust
use std::mem::size_of;

assert_eq!(size_of::<PhantomData<&i32>>(), 0);
assert_eq!(size_of::<RefWithFlag<i32>>(), size_of::<usize>());
```

**PhantomData takes no space** - it's purely for compile-time checks.

## Step 6: Using RefWithFlag

Here's a complete example:

```rust
fn main() {
    let vec = vec![10, 20, 30];

    // Create a RefWithFlag pointing to vec with flag=true
    let flagged = RefWithFlag::new(&vec, true);

    // Extract the reference
    let vec_ref = flagged.get_ref();
    println!("Second element: {}", vec_ref[1]);  // 20

    // Extract the flag
    println!("Flag is: {}", flagged.get_flag());  // true

    // Verify size savings
    use std::mem::size_of;
    println!("Size of RefWithFlag: {}", size_of::<RefWithFlag<Vec<i32>>>());  // 8
    println!("Size of (&Vec<i32>, bool): {}", size_of::<(&Vec<i32>, bool)>()); // 16
}
```

**Output:**
```
Second element: 20
Flag is: true
Size of RefWithFlag: 8
Size of (&Vec<i32>, bool): 16
```

We've saved 8 bytes (50% space savings)!

## Step 7: Safety Invariants

When writing `unsafe` code, document your invariants:

```rust
/// # Safety Invariants
///
/// This type maintains the following invariants:
///
/// 1. **Alignment**: `T` must have at least 2-byte alignment.
///    Enforced by runtime check in `new()`.
///
/// 2. **Pointer validity**: The pointer stored in `ptr_and_bit` must remain
///    valid for the lifetime `'a`. This is ensured by:
///    - Taking `&'a T` as input (guaranteed valid for 'a)
///    - PhantomData maintaining lifetime relationship
///
/// 3. **Bit 0**: The least significant bit of `ptr_and_bit` is the flag.
///    The remaining bits form the pointer address.
///
/// 4. **No aliasing**: We only create shared references (&'a T), never mutable
///    ones, so we don't violate aliasing rules.
```

**Why document invariants?**

- Future maintainers know what the code assumes
- Reviewers can verify safety arguments
- Changes that break invariants are caught
- Makes unsafe code auditable

## Step 8: When Unsafe is Safe

Our `RefWithFlag` has **safe public API**:

```rust
// All these methods are safe to call
let flagged = RefWithFlag::new(&value, true);  // Safe
let r = flagged.get_ref();                      // Safe
let f = flagged.get_flag();                     // Safe
```

**The unsafe code is hidden inside**, and we've proven it's safe by:

1. **Checking alignment** at runtime
2. **Preserving lifetime** with PhantomData
3. **Documenting invariants** clearly
4. **Testing thoroughly**

This is called a **safe abstraction over unsafe code**.

## Step 9: Common Pitfalls

### Pitfall 1: Wrong Alignment

```rust
let byte: u8 = 42;
let flagged = RefWithFlag::new(&byte, true);  // PANIC!
// u8 has 1-byte alignment, so last bit might not be 0
```

**Solution:** The runtime assertion catches this.

### Pitfall 2: Lifetime Errors

```rust
fn oops<'a>() -> RefWithFlag<'a, i32> {
    let x = 42;
    RefWithFlag::new(&x, true)  // Error: x doesn't live long enough
}
```

**Solution:** PhantomData ensures the compiler catches this.

### Pitfall 3: Reading Wrong Bits

```rust
// WRONG: Reading bit 1 instead of bit 0
pub fn get_flag(&self) -> bool {
    self.ptr_and_bit & 2 != 0  // Wrong bit mask!
}
```

**Solution:** Careful code review and testing.

### Pitfall 4: Forgetting to Clear the Bit

```rust
// WRONG: Not clearing the flag bit
pub fn get_ref(&self) -> &'a T {
    unsafe {
        let ptr = self.ptr_and_bit as *const T;  // BUG!
        &*ptr
    }
}
```

This would create a pointer to an invalid address (off by 1).

## Real-World Applications

### 1. Option Optimization

The standard library uses similar tricks for `Option<&T>`:

```rust
// Internally, Option<&T> uses null pointer optimization
pub enum Option<T> {
    None,    // Represented as null pointer
    Some(T), // Represented as non-null pointer
}
```

Size of `Option<&T>` equals size of `&T` (8 bytes, not 16)!

### 2. Tagged Pointers

Many data structures use tagged pointers:

```rust
// Low bits indicate type of node
enum NodePtr {
    Leaf,    // tag = 0b00
    Branch,  // tag = 0b01
    Empty,   // tag = 0b10
}
```

### 3. Lock-Free Data Structures

Atomic operations on tagged pointers enable lock-free programming:

```rust
// Compare-and-swap with version counter in low bits
struct VersionedPtr {
    ptr: usize,  // Pointer + version in low bits
}
```

## Complete Working Example

Here's a complete, documented implementation:

```rust
use std::marker::PhantomData;
use std::mem::align_of;

/// A reference `&'a T` and a boolean flag, packed into a single word.
///
/// This is only possible for types `T` with at least 2-byte alignment,
/// as we steal the least significant bit of the pointer to store the flag.
pub struct RefWithFlag<'a, T> {
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>,
}

impl<'a, T: 'a> RefWithFlag<'a, T> {
    /// Create a new `RefWithFlag` from a reference and a boolean flag.
    ///
    /// # Panics
    ///
    /// Panics if `T` does not have at least 2-byte alignment.
    pub fn new(ptr: &'a T, flag: bool) -> RefWithFlag<'a, T> {
        assert!(align_of::<T>() % 2 == 0,
                "RefWithFlag requires T to have 2-byte alignment");

        RefWithFlag {
            ptr_and_bit: ptr as *const T as usize | flag as usize,
            behaves_like: PhantomData,
        }
    }

    /// Extract the reference.
    pub fn get_ref(&self) -> &'a T {
        unsafe {
            // Clear the flag bit (bit 0) to get the original pointer
            let ptr = (self.ptr_and_bit & !1) as *const T;

            // Safety: We know this pointer is valid because:
            // 1. It came from a valid &'a T reference
            // 2. We only cleared bit 0, which was 0 in the original pointer
            // 3. PhantomData ensures 'a is still valid
            &*ptr
        }
    }

    /// Extract the flag.
    pub fn get_flag(&self) -> bool {
        // Isolate bit 0
        self.ptr_and_bit & 1 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_usage() {
        let vec = vec![10, 20, 30];
        let flagged = RefWithFlag::new(&vec, true);

        assert_eq!(flagged.get_ref()[1], 20);
        assert_eq!(flagged.get_flag(), true);
    }

    #[test]
    fn flag_false() {
        let value = 42i32;
        let flagged = RefWithFlag::new(&value, false);

        assert_eq!(*flagged.get_ref(), 42);
        assert_eq!(flagged.get_flag(), false);
    }

    #[test]
    #[should_panic]
    fn wrong_alignment() {
        let byte: u8 = 42;
        RefWithFlag::new(&byte, true);  // Should panic
    }
}

fn main() {
    let data = vec![1, 2, 3, 4, 5];
    let rf = RefWithFlag::new(&data, true);

    println!("Data: {:?}", rf.get_ref());
    println!("Flag: {}", rf.get_flag());

    use std::mem::size_of;
    println!("\nSize comparison:");
    println!("  RefWithFlag<Vec<i32>>: {} bytes",
             size_of::<RefWithFlag<Vec<i32>>>());
    println!("  (&Vec<i32>, bool):     {} bytes",
             size_of::<(&Vec<i32>, bool)>());
}
```

## Key Takeaways

### When to Use Unsafe

Use `unsafe` when:
1. You need performance optimizations the compiler can't make
2. You're implementing low-level abstractions
3. You're interfacing with foreign code (FFI)
4. You understand and can maintain the invariants

**Don't use unsafe when:**
- Safe code would work just as well
- You're not sure about the safety requirements
- The performance gain is negligible
- You can't clearly document the invariants

### Safety Principles

1. **Minimize unsafe**: Make the unsafe region as small as possible
2. **Safe API**: Expose safe functions that wrap unsafe code
3. **Document invariants**: Explain what must be true for safety
4. **Runtime checks**: Use assertions where possible
5. **Test thoroughly**: Unsafe code needs extensive testing

### Compared to Python

Python developers don't typically worry about:
- Memory layout or alignment
- Pointer manipulation
- Manual lifetime management
- Unsafe code boundaries

Rust gives you control over these, but requires you to prove safety.

## Exercises

### Exercise 1: Two-Bit Flag

Modify `RefWithFlag` to store 2 bits of information (0-3 instead of just 0-1). You'll need 4-byte alignment.

```rust
pub struct RefWithTwoBits<'a, T> {
    // Store 2 bits in the lowest positions
}
```

### Exercise 2: Null Pointer Optimization

Implement `Option<RefWithFlag<T>>` and verify it's the same size as `RefWithFlag<T>`:

```rust
assert_eq!(
    size_of::<Option<RefWithFlag<i32>>>(),
    size_of::<RefWithFlag<i32>>()
);
```

### Exercise 3: Mut Variant

Create a mutable variant `RefWithFlagMut` that stores `&'a mut T`:

```rust
pub struct RefWithFlagMut<'a, T> {
    // Challenge: What needs to change?
}
```

Hint: You'll need to think about aliasing rules!

### Exercise 4: Generic Alignment

Make the assertion generic over alignment:

```rust
impl<'a, T, const ALIGN: usize> RefWithFlag<'a, T, ALIGN>
where
    T: 'a,
{
    pub fn new(ptr: &'a T, bits: usize) -> Self {
        assert!(align_of::<T>() % ALIGN == 0);
        assert!(bits < ALIGN);
        // ...
    }
}
```

## Further Reading

- [The Rustonomicon - Advanced Unsafe Rust](https://doc.rust-lang.org/nomicon/)
- [std::mem::align_of Documentation](https://doc.rust-lang.org/std/mem/fn.align_of.html)
- [std::marker::PhantomData](https://doc.rust-lang.org/std/marker/struct.PhantomData.html)
- [Unsafe Code Guidelines](https://rust-lang.github.io/unsafe-code-guidelines/)
- [Tagged Pointers](https://en.wikipedia.org/wiki/Tagged_pointer)

## Reference: Complete Code

The complete implementation can be found at:
`/home/user/rust-programming-examples/ref-with-flag/src/lib.rs`

Run the tests with:
```bash
cd /home/user/rust-programming-examples/ref-with-flag
cargo test
```

## Final Words

You've now seen how Rust allows you to write extremely efficient, low-level code while still providing safety guarantees through its type system. The techniques demonstrated here - pointer tagging, PhantomData, and safe abstraction over unsafe code - are fundamental to many high-performance Rust libraries.

Remember: **With great power comes great responsibility.** Use `unsafe` sparingly, document thoroughly, and test extensively.
