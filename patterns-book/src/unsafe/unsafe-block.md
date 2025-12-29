# Unsafe Block Pattern

## Context

You are writing Rust code that requires operations the compiler cannot verify as safe: raw pointer dereference, calling foreign functions, or accessing mutable statics. You need to perform these operations while maintaining Rust's safety guarantees at the module boundary.

The `ascii` module demonstrates this: converting validated ASCII bytes to UTF-8 strings requires an unsafe operation, but the module can guarantee safety through careful API design.

## Problem

**How do you isolate unsafe operations to maintain verifiable safety boundaries while enabling performance-critical or low-level functionality?**

Rust's safety guarantees come from compile-time verification. However, certain operations—like calling C functions, dereferencing raw pointers, or performing type transmutations—cannot be verified by the compiler. If unsafe code is scattered throughout a codebase, it becomes impossible to audit and maintain safety invariants.

The challenge is to use `unsafe` when necessary while keeping its scope minimal and the invariants clear.

## Forces

- **Safety isolation**: Unsafe code should be contained in the smallest possible scope
- **Auditability**: Reviewers need to verify safety properties without reading the entire codebase
- **Performance**: Some operations require unsafe code to avoid runtime overhead
- **Maintainability**: Future maintainers must understand what invariants the unsafe code depends on
- **Correctness**: The unsafe operation must genuinely be safe given the surrounding context

These forces conflict: broad unsafe scopes are easier to write but harder to audit; narrow scopes require more careful API design but are easier to verify.

## Solution

**Isolate unsafe operations in the smallest possible blocks or functions, and encapsulate them behind safe APIs that maintain necessary invariants.**

Follow this pattern:

1. **Minimize scope**: Keep `unsafe` blocks as small as possible—ideally a single expression or statement
2. **Document invariants**: Add comments explaining why the operation is safe
3. **Encapsulate**: Wrap unsafe operations in safe functions that enforce preconditions
4. **Validate at boundaries**: Perform all necessary checks before entering unsafe code

### Example from ascii

The `ascii` module uses unsafe to convert validated ASCII to UTF-8 without checking again:

```rust
// Safe public API - validates input
pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError> {
    if bytes.iter().any(|&byte| !byte.is_ascii()) {
        return Err(NotAsciiError(bytes));
    }
    Ok(Ascii(bytes))
}

// Safe conversion using unsafe - no validation needed
impl From<Ascii> for String {
    fn from(ascii: Ascii) -> String {
        // SAFETY: Ascii type guarantees bytes are valid ASCII (0x00-0x7f),
        // which is a subset of valid UTF-8. The from_bytes constructor
        // ensures this invariant.
        unsafe { String::from_utf8_unchecked(ascii.0) }
    }
}
```

The unsafe block is minimal (one expression), documented with a SAFETY comment, and safe because:
- The `Ascii` type maintains the invariant that its bytes are valid ASCII
- The safe constructor (`from_bytes`) validates this invariant
- Valid ASCII is always valid UTF-8

### Example from gap-buffer

Pointer operations are isolated to individual helper methods:

```rust
/// Return a pointer to the `index`'th element of the underlying storage.
///
/// Safety: `index` must be a valid index into `self.storage`.
unsafe fn space(&self, index: usize) -> *const T {
    self.storage.as_ptr().offset(index as isize)
}

// Safe wrapper that checks bounds
pub fn get(&self, index: usize) -> Option<&T> {
    let raw = self.index_to_raw(index);
    if raw < self.capacity() {
        unsafe {
            // SAFETY: We just checked `raw` against self.capacity(),
            // and index_to_raw skips the gap, so this is safe.
            Some(&*self.space(raw))
        }
    } else {
        None
    }
}
```

Each unsafe operation is:
- In its own helper function with documented safety requirements
- Called only after bounds checking
- Wrapped in a safe public API

## Resulting Context

### Benefits

- **Focused auditing**: Reviewers can locate and verify all unsafe code quickly
- **Clear contracts**: Safety comments document exactly what must be true
- **Reduced risk**: Minimal unsafe scope limits potential for undefined behavior
- **Maintainable**: Future changes to safe code cannot violate unsafe invariants
- **Performance**: Zero-cost abstractions—no runtime overhead for safety

### Liabilities

- **More verbose**: Requires careful API design and documentation
- **Learning curve**: Developers must understand when unsafe is truly necessary
- **False security**: Even minimal unsafe blocks can cause undefined behavior if invariants are violated

### Usage Guidelines

Use this pattern when:
- Calling C FFI functions (see **FFI Bindings** pattern)
- Implementing performance-critical data structures
- Working with raw pointers or uninitialized memory
- Using compiler intrinsics

Do NOT use unsafe for:
- Avoiding the borrow checker (fix your design instead)
- Premature optimization (measure first)
- Operations that can be done safely (e.g., bounds checking is cheap)

## Related Patterns

- **Safety Invariants**: Documents the invariants that unsafe code depends on
- **Safe Wrapper**: Encapsulates unsafe FFI behind safe Rust APIs
- **Resource Cleanup**: Uses unsafe in Drop implementations to free resources

## Known Uses

- **ascii**: Uses unsafe for validated ASCII to UTF-8 conversion
- **gap-buffer**: Uses unsafe for pointer arithmetic in a text editor buffer
- **ref-with-flag**: Uses unsafe to pack a reference and boolean into one word
- **libgit2-rs**: Uses unsafe extensively for FFI, but each call is isolated
- **std::vec::Vec**: Rust standard library uses unsafe internally but exposes safe API
- **std::collections::HashMap**: Uses unsafe for performance but maintains safe interface

## Implementation Notes

### SAFETY Comment Convention

Always use this format:

```rust
unsafe {
    // SAFETY: [Explain why this is safe]
    // - Invariant 1: [what must be true]
    // - Invariant 2: [what we checked]
    // - Why it's safe: [reasoning]
    actual_unsafe_operation()
}
```

### Verification Checklist

Before writing unsafe:

- [ ] Is there a safe alternative? (Try that first)
- [ ] Can I make the unsafe block smaller?
- [ ] Have I documented the safety invariants?
- [ ] Can I test the invariants in debug builds with assertions?
- [ ] Will future maintainers understand why this is safe?

### Testing Unsafe Code

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariant_maintained() {
        let ascii = Ascii::from_bytes(b"hello".to_vec()).unwrap();
        let s: String = ascii.into();
        assert_eq!(s, "hello");
    }

    #[test]
    #[should_panic]
    fn test_invalid_rejected() {
        // This should fail, not invoke undefined behavior
        Ascii::from_bytes(vec![0xff]).unwrap();
    }
}
```

## Further Reading

- *The Rustonomicon* - Detailed guide to unsafe Rust
- *Rust RFC 2585* - Unsafe code guidelines
- Crate `unsafe-code-guidelines` - Working group documentation
