# Unsafe Code Patterns

## The Spectrum of Unsafe

Unsafe code in Rust exists on a spectrum from "slightly relaxed safety checks" to "complete manual memory management." Understanding where your code falls on this spectrum helps you choose appropriate patterns and verification strategies.

At one end, we have simple operations like `from_bytes_unchecked` in the `ascii` project—a thin wrapper that skips validation. At the other end, we have complex manual memory management like in `gap-buffer`, which manipulates raw pointers and manages initialization states explicitly.

The patterns you choose should match the complexity of your unsafe operations.

## Unsafe Functions vs Unsafe Blocks

Rust provides two ways to mark code as unsafe: `unsafe fn` and `unsafe` blocks. Choosing between them is a crucial design decision.

### Unsafe Functions: Declaring Contracts

An `unsafe fn` declares that the function itself has safety requirements. Callers must uphold a contract, or undefined behavior results:

```rust
/// Construct an `Ascii` value from `bytes`, without checking
/// whether `bytes` actually contains well-formed ASCII.
///
/// # Safety
///
/// The caller must ensure that `bytes` contains only ASCII
/// characters: bytes no greater than 0x7f. Otherwise, the effect is
/// undefined.
pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Ascii {
    Ascii(bytes)
}
```

The function body doesn't contain `unsafe` operations—the unsafety comes from the contract. The caller must ensure `bytes` contains only ASCII. If this invariant is violated, later code that assumes ASCII will exhibit undefined behavior:

```rust
impl From<Ascii> for String {
    fn from(ascii: Ascii) -> String {
        // SAFETY: Ascii type guarantees well-formed ASCII,
        // which is a subset of UTF-8
        unsafe { String::from_utf8_unchecked(ascii.0) }
    }
}
```

Here's the contract chain:
1. `from_bytes_unchecked` requires the caller to provide ASCII
2. The `Ascii` type maintains this as an invariant
3. `from_utf8_unchecked` assumes UTF-8, which ASCII satisfies

If step 1 is violated, step 3 produces undefined behavior.

### Unsafe Blocks: Localizing Operations

An `unsafe` block indicates that the code inside performs unsafe operations, but the function as a whole is safe to call. All safety requirements are met within the function:

```rust
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

This function is safe to call because it checks bounds before dereferencing the pointer. The unsafe operation is encapsulated. Callers don't need to worry about safety—the function maintains invariants internally.

### Choosing Between Them

**Use `unsafe fn` when:**
- Callers must maintain invariants you cannot check
- The contract extends beyond the function call
- You're building low-level primitives

**Use `unsafe` blocks when:**
- You can verify all safety requirements internally
- You're encapsulating unsafe operations
- You're providing a safe high-level API

## Minimal Scope Principle

Unsafe blocks should be as small as possible. This makes them easier to review and reduces the chance of mistakes.

### Bad: Large Unsafe Blocks

```rust
// Anti-pattern: large unsafe block
pub fn process(&mut self) {
    unsafe {
        let ptr1 = self.get_ptr();
        let value = *ptr1;

        let result = some_safe_computation(value);

        let ptr2 = self.get_other_ptr();
        *ptr2 = result;

        another_safe_function();
    }
}
```

This block mixes safe and unsafe operations. If `another_safe_function()` panics, how does that affect the unsafe code? It's hard to reason about.

### Good: Minimal Unsafe Blocks

```rust
// Better: separate unsafe operations
pub fn process(&mut self) {
    let value = unsafe {
        // SAFETY: get_ptr returns a valid pointer to initialized memory
        *self.get_ptr()
    };

    let result = some_safe_computation(value);

    unsafe {
        // SAFETY: get_other_ptr returns a valid, writable pointer
        *self.get_other_ptr() = result;
    };

    another_safe_function();
}
```

Each unsafe block is focused. If `another_safe_function()` panics, the unsafe operations have already completed. The code is easier to review—each SAFETY comment justifies just a few lines.

### Example from gap-buffer

The `insert` method shows minimal scope:

```rust
pub fn insert(&mut self, elt: T) {
    if self.gap.len() == 0 {
        self.enlarge_gap();
    }

    unsafe {
        // SAFETY: We just ensured gap has space
        let index = self.gap.start;
        std::ptr::write(self.space_mut(index), elt);
    }
    self.gap.start += 1;
}
```

The unsafe block contains only the pointer write. The gap expansion and index update are outside, where they can't interfere with the unsafe operation.

## Documenting Safety Invariants

Clear documentation is not optional for unsafe code. Future maintainers (including yourself) need to understand what must remain true.

### Documenting Unsafe Functions

For unsafe functions, document:
1. **What the caller must ensure** (preconditions)
2. **What the function guarantees** (postconditions)
3. **What happens if violated** (usually "undefined behavior")

From `ref-with-flag`:

```rust
/// A `&T` and a `bool`, wrapped up in a single word.
/// The type `T` must require at least two-byte alignment.
pub struct RefWithFlag<'a, T> {
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>
}
```

The type-level documentation states the alignment requirement. The constructor enforces it:

```rust
impl<'a, T: 'a> RefWithFlag<'a, T> {
    pub fn new(ptr: &'a T, flag: bool) -> RefWithFlag<T> {
        assert!(align_of::<T>() % 2 == 0);
        // ...
    }
}
```

Even though this is a safe function, it documents the invariant (alignment) and checks it at runtime.

### Documenting Unsafe Blocks

For unsafe blocks, explain:
1. **Which unsafe operation is performed**
2. **Why it's safe in this context**
3. **Which invariants guarantee safety**

From `gap-buffer`:

```rust
unsafe {
    // SAFETY: We just checked `raw` against self.capacity(),
    // and index_to_raw skips the gap, so this is safe.
    Some(&*self.space(raw))
}
```

This comment explains:
- The operation: dereferencing a raw pointer
- Why it's safe: bounds checked, gap skipped
- The guarantees: `raw < capacity()`, `index_to_raw` correctness

### State Invariants

Complex types should document their invariants at the type level:

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

This documents two critical invariants:
1. The `storage` Vec always has length zero (we use its capacity)
2. Elements outside the gap range are initialized

Every unsafe operation in `GapBuffer` relies on these invariants. They're verified in code reviews and tests, not by the compiler.

## Testing Unsafe Code

Unsafe code requires more thorough testing than safe code. The compiler can't catch mistakes, so testing becomes critical.

### Test Boundary Conditions

Test the edges where unsafe code is most likely to fail:

```rust
#[test]
fn test_gap_buffer_edges() {
    let mut buf = GapBuffer::new();

    // Empty buffer
    assert_eq!(buf.get(0), None);

    // Insert at beginning
    buf.insert('a');
    assert_eq!(buf.get(0), Some(&'a'));

    // Move gap to different positions
    buf.set_position(0);
    buf.insert('b');
    assert_eq!(buf.get(0), Some(&'b'));
    assert_eq!(buf.get(1), Some(&'a'));

    // Remove elements
    buf.set_position(0);
    assert_eq!(buf.remove(), Some('b'));
}
```

This tests insertion at various positions, gap movement, and removal—all operations that involve unsafe pointer manipulation.

### Test Invariant Violations

Use test builds to detect invariant violations:

```rust
#[test]
#[should_panic]
fn test_alignment_requirement() {
    // This should panic due to alignment check
    let value: u8 = 42;  // u8 is 1-byte aligned
    RefWithFlag::new(&value, true);
}
```

This verifies the alignment check works. If it doesn't panic, the invariant isn't enforced.

### Use Miri for UB Detection

Miri is Rust's interpreter that detects undefined behavior:

```bash
cargo +nightly miri test
```

Miri catches:
- Use-after-free
- Out-of-bounds memory access
- Uninitialized memory reads
- Data races

From the CLAUDE.md documentation:

```bash
# Install miri
rustup +nightly component add miri

# Run tests under miri
cargo +nightly miri test
```

Miri will catch this error:

```rust
unsafe {
    let x = 5;
    let ptr = &x as *const i32;
    // x is dropped here
    println!("{}", *ptr);  // Use-after-free - Miri catches this!
}
```

### Test Resource Cleanup

For types with `Drop` implementations, verify resources are freed:

```rust
#[test]
fn drop_elements() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct Counted;
    impl Drop for Counted {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    {
        let mut buf = GapBuffer::new();
        buf.insert(Counted);
        buf.insert(Counted);
        buf.insert(Counted);
    } // buf dropped here

    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 3);
}
```

This verifies that `GapBuffer::drop` correctly drops all elements, even those in the gap buffer's complex memory layout.

## Common Unsafe Patterns

### Pattern 1: Transmute for Type Conversion

```rust
unsafe { std::mem::transmute(bytes) }
```

Use when converting between types with identical memory layout. Document why layouts match.

### Pattern 2: Pointer Offset for Indexing

```rust
unsafe fn space(&self, index: usize) -> *const T {
    self.storage.as_ptr().offset(index as isize)
}
```

Always verify bounds before calling. Document the safety contract.

### Pattern 3: PhantomData for Lifetime Tracking

```rust
pub struct RefWithFlag<'a, T> {
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>
}
```

Use when you store raw pointers but need lifetime checking. `PhantomData` teaches the compiler about ownership relationships.

## Conclusion

Unsafe code patterns follow from a simple principle: make safety invariants explicit, check them where possible, document them everywhere, and test thoroughly. Choose between `unsafe fn` and `unsafe` blocks based on who must maintain invariants. Keep unsafe blocks small and focused. Document what must be true and why operations are safe. Test edge cases and use tools like Miri to catch mistakes.

Well-structured unsafe code is not dangerous—it's powerful. It enables zero-cost abstractions while maintaining Rust's safety guarantees at higher levels.
