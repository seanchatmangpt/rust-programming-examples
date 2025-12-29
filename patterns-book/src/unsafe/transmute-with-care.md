# Transmute With Care Pattern

## Context

You need to reinterpret data of one type as another type with the same memory layout, or bypass type system checks for a performance-critical operation. The types have compatible representations, but Rust's type system prevents direct conversion.

The `ascii` module demonstrates this: converting `Vec<u8>` to `String` requires transmutation (via `from_utf8_unchecked`), but only when the bytes are known to be valid UTF-8.

## Problem

**How do you safely perform type transmutation—reinterpreting memory as a different type—without introducing undefined behavior?**

`std::mem::transmute` is the most dangerous operation in Rust. It tells the compiler "trust me, I know these types have compatible layouts" and bypasses all type checking. Incorrect use causes undefined behavior that may not manifest until much later in execution or only on certain platforms.

The challenge is to use transmutation when necessary while maintaining absolute certainty about type compatibility.

## Forces

- **Performance**: Transmutation is zero-cost; safe conversions may allocate or copy
- **Correctness**: Mismatched sizes, alignments, or representations cause undefined behavior
- **Maintainability**: Future type changes can silently break transmutation
- **Clarity**: Transmutation hides the actual operation being performed
- **Safety**: Even small mistakes lead to memory corruption
- **Portability**: Layout assumptions may not hold on all platforms

These forces conflict: maximum performance demands transmutation, but maximum safety demands avoiding it entirely.

## Solution

**Use transmutation only when absolutely necessary, verify size and alignment at compile time, document the layout assumptions, and prefer safer alternatives when possible.**

Follow this pattern:

1. **Avoid when possible**: Try `as`, `From`, `Into`, or `TryFrom` first
2. **Document layout**: Explicitly state the representation guarantees
3. **Compile-time checks**: Use `assert_eq!(size_of::<A>(), size_of::<B>())` in const contexts
4. **SAFETY comment**: Explain why the transmutation is sound
5. **Use specific helpers**: Prefer `from_utf8_unchecked` over raw `transmute`
6. **Test thoroughly**: Include tests on all target platforms

### Example: Safe Transmutation in ascii

Rather than using raw `transmute`, the code uses the standard library's purpose-built function:

```rust
impl From<Ascii> for String {
    fn from(ascii: Ascii) -> String {
        // SAFETY: The Ascii type maintains the invariant that all bytes
        // are valid ASCII (0x00-0x7f), which is a subset of valid UTF-8.
        // from_utf8_unchecked performs a transmute internally but is
        // safer than raw transmute because it documents the specific
        // invariant required (valid UTF-8).
        unsafe { String::from_utf8_unchecked(ascii.0) }
    }
}
```

This is safer than:

```rust
// ❌ DANGEROUS - don't do this
unsafe { std::mem::transmute::<Vec<u8>, String>(ascii.0) }
```

Because `from_utf8_unchecked`:
- Documents the exact requirement (valid UTF-8)
- Has the same performance (zero-cost)
- Is maintained by the standard library
- Works if `String` representation changes (transmute would break)

### Example: When You Must Transmute

If you must use `transmute`, follow this pattern:

```rust
use std::mem;

#[repr(C)]
struct MyStruct {
    a: u32,
    b: u32,
}

// Transmute to compatible C representation
fn to_u64(s: MyStruct) -> u64 {
    // Compile-time assertion - fails if sizes don't match
    const _: () = assert!(mem::size_of::<MyStruct>() == mem::size_of::<u64>());
    const _: () = assert!(mem::align_of::<MyStruct>() >= mem::align_of::<u64>());

    unsafe {
        // SAFETY: MyStruct is repr(C) with two u32 fields, which on
        // all platforms we support has the same size and alignment as u64.
        // The bit pattern of two u32s is a valid u64.
        mem::transmute::<MyStruct, u64>(s)
    }
}
```

### What NOT to Transmute

❌ **Never transmute**:
- Between types of different sizes (compile-time error, but check anyway)
- To/from types with different alignment requirements
- References to different lifetimes
- References to types with different drop behavior
- Non-`repr(C)` types across FFI boundaries
- Enums (layout is unspecified unless `repr(C)` or `repr(Int)`)

### Safer Alternatives

#### Use unions instead of transmute

```rust
union FloatOrBits {
    f: f32,
    bits: u32,
}

// Better than transmute for this use case
fn float_to_bits(f: f32) -> u32 {
    unsafe { FloatOrBits { f }.bits }
}
```

Or even better, use the standard library:

```rust
// Best: use the standard library function
fn float_to_bits(f: f32) -> u32 {
    f.to_bits()
}
```

#### Use pointer casts for reference conversions

```rust
// If you need to convert references, use pointer casts with explicit steps
fn bytes_to_ascii(bytes: &[u8]) -> &Ascii {
    // SAFETY: Ascii is a transparent wrapper around [u8]
    unsafe {
        &*(bytes as *const [u8] as *const Ascii)
    }
}
```

This is clearer about what's happening than `transmute`.

## Resulting Context

### Benefits

- **Zero cost**: Transmutation has no runtime overhead
- **Explicit**: The operation clearly states the types involved
- **Verified**: Compile-time size checks catch many errors
- **Necessary evil**: Sometimes the only way to interface with C or optimize critical paths

### Liabilities

- **Dangerous**: Easiest way to write unsafe code that compiles but crashes
- **Fragile**: Breaks silently if type definitions change
- **Platform-specific**: Layout assumptions may not hold everywhere
- **Hard to review**: Reviewers must verify deep type compatibility
- **Delayed failures**: Undefined behavior may not manifest immediately

### When to Use Transmute

✅ **Valid uses**:
- Converting between types with explicitly guaranteed layouts (`repr(C)`, `repr(transparent)`)
- Implementing low-level primitives (like `Vec` or `String` internals)
- FFI with well-documented C structures
- Performance-critical code after profiling shows other methods are too slow

❌ **Invalid uses**:
- Bypassing the borrow checker
- Converting between semantically different types
- "I think these types have the same layout"
- Avoiding writing a proper conversion function

## Related Patterns

- **Unsafe Block**: Transmute always requires unsafe
- **Safety Invariants**: Transmutation depends on layout invariants
- **FFI Bindings**: Often requires transmuting between Rust and C types

## Known Uses

- **ascii**: Uses `from_utf8_unchecked` (transmute internally) for ASCII to String
- **std::string::String**: `from_utf8_unchecked` transmutes `Vec<u8>` to `String`
- **std::vec::Vec**: Uses transmute internally for reallocation and type conversions
- **std::mem::MaybeUninit**: Uses transmute to convert uninit to initialized
- **Serde**: Uses transmute for some zero-copy deserialization
- **bytemuck crate**: Safe transmutation with trait bounds

## Implementation Notes

### Compile-Time Verification

```rust
use std::mem;

// Function that only compiles if types are compatible
const fn assert_transmutable<A, B>() {
    const {
        assert!(mem::size_of::<A>() == mem::size_of::<B>(),
               "sizes don't match");
        assert!(mem::align_of::<A>() >= mem::align_of::<B>(),
               "alignment mismatch");
    }
}

fn safe_transmute<A, B>(a: A) -> B {
    assert_transmutable::<A, B>();
    unsafe { mem::transmute_copy(&a) }
}
```

### Testing Transmutation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn verify_layout() {
        // Test on actual target platform
        assert_eq!(mem::size_of::<MyStruct>(), mem::size_of::<u64>());
        assert_eq!(mem::align_of::<MyStruct>(), mem::align_of::<u64>());
    }

    #[test]
    fn test_transmute_correctness() {
        let s = MyStruct { a: 0x11223344, b: 0x55667788 };
        let n = to_u64(s);
        // Verify the bit pattern is as expected
        #[cfg(target_endian = "little")]
        assert_eq!(n, 0x5566778811223344);
    }
}
```

### Documentation Template

```rust
/// Convert MyStruct to u64.
///
/// # Safety
///
/// This function uses transmute, which requires:
/// - MyStruct is repr(C)
/// - size_of::<MyStruct>() == size_of::<u64>()
/// - All bit patterns are valid for u64 (true for integers)
/// - Alignment requirements are satisfied
///
/// These are verified at compile time via const assertions.
unsafe fn my_transmute(s: MyStruct) -> u64 {
    // ...
}
```

### The Safest Transmutes

These are generally safe:

```rust
// Integers of same size
let a: u32 = 42;
let b: i32 = unsafe { mem::transmute(a) };

// Arrays to same-size arrays
let arr: [u8; 4] = [1, 2, 3, 4];
let n: u32 = unsafe { mem::transmute(arr) };

// repr(C) structs with identical layouts
#[repr(C)]
struct A { x: u32, y: u32 }
#[repr(C)]
struct B { a: u32, b: u32 }
```

### The Bytemuck Approach

For safer transmutation, consider the `bytemuck` crate:

```rust
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct MyStruct {
    a: u32,
    b: u32,
}

// No unsafe needed - traits guarantee safety
let s = MyStruct { a: 1, b: 2 };
let bytes: [u8; 8] = bytemuck::cast(s);
```

## Common Pitfalls

### Pitfall 1: Transmuting References with Different Lifetimes

```rust
// ❌ WRONG - creates dangling reference
let r: &'static str = unsafe {
    let s = String::from("hello");
    mem::transmute::<&str, &'static str>(s.as_str())
}; // s dropped here, r now dangles
```

### Pitfall 2: Transmuting Enums

```rust
// ❌ WRONG - enum layout is unspecified
enum MyEnum { A, B, C }
let n: u8 = unsafe { mem::transmute(MyEnum::B) }; // UB!

// ✅ RIGHT - use repr
#[repr(u8)]
enum MyEnum { A = 0, B = 1, C = 2 }
let n: u8 = unsafe { mem::transmute(MyEnum::B) }; // OK
```

### Pitfall 3: Assuming Struct Layout

```rust
// ❌ WRONG - Rust can reorder fields
struct MyStruct { a: u8, b: u32 }
// Layout is NOT guaranteed to be [u8, u32]

// ✅ RIGHT - use repr(C)
#[repr(C)]
struct MyStruct { a: u8, b: u32 }
// Now layout matches C struct layout
```

## Further Reading

- *The Rustonomicon* - "Transmutes" chapter
- Blog: "How to Panic in Rust" by Alexis Beingessner (transmute examples)
- Crate: `bytemuck` - Safe transmutation
- RFC 2582 - Raw reference operators (`&raw const` / `&raw mut`)
