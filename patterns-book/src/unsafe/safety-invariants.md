# Safety Invariants Pattern

## Context

You are designing a type that contains unsafe code or maintains properties that cannot be verified by the Rust compiler. The type needs to expose a safe API while relying on internal invariants that, if violated, would cause undefined behavior.

The `Ascii` type exemplifies this: it wraps a `Vec<u8>` but maintains the invariant that all bytes are valid ASCII (0x00-0x7f). This invariant enables zero-cost conversion to `String`.

## Problem

**How do you define, document, and maintain safety invariants that unsafe code depends on, ensuring they cannot be violated through the public API?**

Unsafe code is safe only when certain conditions (invariants) hold true. These invariants exist in the programmer's mind and in comments, but the compiler cannot verify them. If an invariant is violated—even by safe code elsewhere in the module—undefined behavior results.

The challenge is to make invariants explicit, enforceable, and maintainable across the lifetime of the type.

## Forces

- **Explicitness**: Invariants must be documented so reviewers can verify safety
- **Enforcement**: The public API must make violating invariants impossible
- **Visibility**: Internal fields must be private to prevent external violation
- **Maintainability**: Future code changes must preserve invariants
- **Performance**: Checking invariants at runtime may be too expensive
- **Correctness**: Invariants must be sufficient to guarantee memory safety

These forces conflict: strong enforcement requires runtime checks (expensive); weak enforcement risks undefined behavior (catastrophic).

## Solution

**Define clear safety invariants, document them explicitly, enforce them through API design, and use assertions in debug builds to catch violations early.**

Follow this pattern:

1. **Document invariants**: Use comments in the type definition
2. **Private fields**: Make all fields private to control access
3. **Validated constructors**: Check invariants in all constructors
4. **Preserve in methods**: Ensure methods maintain invariants
5. **Debug assertions**: Add runtime checks in debug builds
6. **Audit unsafe code**: Review all unsafe code that depends on invariants

### Example from ascii

The `Ascii` type maintains the invariant "all bytes are valid ASCII":

```rust
mod my_ascii {
    /// An ASCII-encoded string.
    #[derive(Debug, Eq, PartialEq)]
    pub struct Ascii(
        // INVARIANT: This must hold only well-formed ASCII text:
        // bytes from `0` to `0x7f`.
        Vec<u8>
    );

    impl Ascii {
        /// Create an `Ascii` from the ASCII text in `bytes`.
        /// Return a `NotAsciiError` if `bytes` contains non-ASCII characters.
        pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError> {
            // Validate the invariant before construction
            if bytes.iter().any(|&byte| !byte.is_ascii()) {
                return Err(NotAsciiError(bytes));
            }
            Ok(Ascii(bytes))
        }

        /// Construct an `Ascii` value from `bytes`, without checking.
        ///
        /// # Safety
        ///
        /// The caller must ensure that `bytes` contains only ASCII
        /// characters: bytes no greater than 0x7f. Otherwise, the effect is
        /// undefined.
        pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Ascii {
            // Debug assertion to catch violations during testing
            debug_assert!(bytes.iter().all(|&b| b.is_ascii()),
                         "Invariant violated: non-ASCII bytes in Ascii type");
            Ascii(bytes)
        }
    }

    // Unsafe code that depends on the invariant
    impl From<Ascii> for String {
        fn from(ascii: Ascii) -> String {
            // SAFETY: The Ascii type's invariant guarantees all bytes are
            // valid ASCII, which is a subset of valid UTF-8.
            unsafe { String::from_utf8_unchecked(ascii.0) }
        }
    }
}
```

Key points:
- **Documented**: The invariant is stated in a comment on the field
- **Enforced**: The field is private; only validated constructors can create instances
- **Checked**: The safe constructor validates before construction
- **Unsafe option**: Provides an unchecked constructor for performance when the caller can guarantee the invariant
- **Used**: The unsafe conversion relies explicitly on the documented invariant

### Example from gap-buffer

The `GapBuffer` maintains multiple invariants:

```rust
pub struct GapBuffer<T> {
    // INVARIANT 1: storage.len() is always 0 (we use capacity only)
    storage: Vec<T>,

    // INVARIANT 2: gap.start <= gap.end <= storage.capacity()
    // INVARIANT 3: Elements outside gap range [0..gap.start) and
    //              [gap.end..capacity()) are initialized
    // INVARIANT 4: Elements in gap range [gap.start..gap.end) are
    //              uninitialized
    gap: Range<usize>
}

impl<T> GapBuffer<T> {
    unsafe fn space(&self, index: usize) -> *const T {
        // Debug assertion to verify invariant in development
        debug_assert!(index < self.storage.capacity(),
                     "index {} out of bounds for capacity {}",
                     index, self.storage.capacity());
        self.storage.as_ptr().offset(index as isize)
    }

    pub fn set_position(&mut self, pos: usize) {
        // Validate precondition (part of maintaining invariants)
        if pos > self.len() {
            panic!("index {} out of range for GapBuffer", pos);
        }

        unsafe {
            // SAFETY: We just validated pos <= self.len(), and the gap
            // invariants ensure we can safely move elements.
            // Invariant preserved: gap moves but size remains constant
            let gap = self.gap.clone();
            if pos > gap.start {
                let distance = pos - gap.start;
                std::ptr::copy(self.space(gap.end),
                               self.space_mut(gap.start),
                               distance);
            }
            self.gap = pos .. pos + gap.len();
        }
    }
}

impl<T> Drop for GapBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: Invariants 3 and 4 ensure that elements outside
            // the gap are initialized and must be dropped
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

The invariants are:
- **Explicit**: Each invariant is documented
- **Necessary**: The unsafe code in Drop depends on invariants 3 and 4
- **Maintained**: Every method preserves the invariants
- **Checked**: Debug assertions catch violations during testing

### Example from ref-with-flag

```rust
/// A `&T` and a `bool`, wrapped up in a single word.
/// The type `T` must require at least two-byte alignment.
pub struct RefWithFlag<'a, T> {
    // INVARIANT: The low bit contains the boolean flag.
    // INVARIANT: The upper bits contain a valid pointer to T.
    // INVARIANT: T must have alignment >= 2 (verified in constructor).
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>
}

impl<'a, T: 'a> RefWithFlag<'a, T> {
    pub fn new(ptr: &'a T, flag: bool) -> RefWithFlag<T> {
        // Enforce the alignment invariant at construction
        assert!(align_of::<T>() % 2 == 0,
               "RefWithFlag requires T to have alignment >= 2");
        RefWithFlag {
            ptr_and_bit: ptr as *const T as usize | flag as usize,
            behaves_like: PhantomData
        }
    }

    pub fn get_ref(&self) -> &'a T {
        unsafe {
            // SAFETY: The constructor ensures T has alignment >= 2,
            // so the low bit is always 0 in the original pointer.
            // Masking with !1 recovers the original pointer.
            let ptr = (self.ptr_and_bit & !1) as *const T;
            &*ptr
        }
    }
}
```

## Resulting Context

### Benefits

- **Auditable safety**: Reviewers can verify that unsafe code is sound given the invariants
- **Maintainability**: Future maintainers know what properties must be preserved
- **Early detection**: Debug assertions catch violations during development
- **Clear contracts**: Each unsafe block references specific invariants
- **Documentation**: Invariants serve as both comments and verification targets

### Liabilities

- **Complexity**: More invariants mean more to track and maintain
- **Runtime cost**: Debug assertions add overhead (mitigated by only running in debug builds)
- **False confidence**: Documented invariants can still be violated if enforcement is weak
- **Verbosity**: Requires detailed comments and careful API design

### Enforcement Levels

1. **Type system** (strongest): Use types to make invalid states unrepresentable
2. **Privacy**: Make fields private to control mutation
3. **Validation**: Check invariants in constructors and public methods
4. **Runtime assertions**: Use `debug_assert!` to catch violations in testing
5. **Documentation** (weakest): Document invariants for unsafe code reviewers

Use the strongest enforcement practical for each invariant.

## Related Patterns

- **Unsafe Block**: Uses documented invariants to justify unsafe operations
- **Safe Wrapper**: Maintains invariants while wrapping unsafe FFI
- **Transmute With Care**: Relies on layout invariants for safe type punning

## Known Uses

- **ascii**: Maintains "all bytes are ASCII" invariant
- **gap-buffer**: Maintains gap position and initialization invariants
- **ref-with-flag**: Maintains alignment and pointer validity invariants
- **libgit2-rs-safe::Repository**: Maintains "raw pointer is valid" invariant
- **std::vec::Vec**: Maintains "len <= capacity" and "elements [0..len) are initialized" invariants
- **std::string::String**: Maintains "bytes are valid UTF-8" invariant

## Implementation Notes

### Invariant Documentation Template

```rust
pub struct MyType {
    // INVARIANT: [Description of what must always be true]
    // DEPENDS ON: [What other invariants this relies on]
    // ENFORCED BY: [How this is maintained - constructor, privacy, etc.]
    // USED BY: [What unsafe code relies on this invariant]
    field: FieldType,
}
```

### Debug Assertions

```rust
impl MyType {
    fn method(&mut self) {
        // Check invariant before operation
        #[cfg(debug_assertions)]
        self.check_invariants();

        // Perform operation...

        // Check invariant after operation
        #[cfg(debug_assertions)]
        self.check_invariants();
    }

    #[cfg(debug_assertions)]
    fn check_invariants(&self) {
        assert!(self.invariant_1(), "Invariant 1 violated");
        assert!(self.invariant_2(), "Invariant 2 violated");
    }
}
```

### Testing Invariants

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn valid_construction_maintains_invariants() {
        let ascii = Ascii::from_bytes(b"hello".to_vec()).unwrap();
        // If we get here, invariants were checked in constructor
    }

    #[test]
    #[should_panic(expected = "non-ASCII")]
    fn invalid_construction_rejected() {
        // This should panic, not violate invariants
        unsafe { Ascii::from_bytes_unchecked(vec![0xff]) };
    }
}
```

## Further Reading

- *The Rustonomicon* - Chapter on "Safe Wrappers Around Unsafe Code"
- Blog post: "How to Safely Think in Unsafe Rust" by Aria Beingessner
- Rust RFC 1643 - Type privacy and sealed traits
