# 40. UNSAFE FUNCTION WITH SAFETY COMMENT

*A high-voltage cabinet with warning labels that explain exactly what voltage flows through each wire and what you must do to avoid electrocution—the danger explicit, the precautions documented*

...within a **PUBLIC FUNCTION (35)** or **PRIVATE HELPER FUNCTION**, when you must bypass Rust's safety guarantees for performance or FFI, but need to document the contract that makes the operation safe...

◆ ◆ ◆

**How do you provide operations that the compiler cannot verify as safe, while ensuring callers understand what invariants they must maintain?**

Sometimes you need to step outside Rust's safety guarantees: interfacing with C libraries, optimizing hot paths, implementing low-level data structures. The compiler cannot verify these operations—it cannot know that a raw pointer points to valid memory, that an FFI call maintains Rust's invariants, or that a transmute produces a valid value.

The `unsafe` keyword marks these boundaries. An `unsafe fn` declares that the function has preconditions the compiler cannot check. Calling such a function is an assertion: "I, the caller, guarantee these invariants hold." But how does the caller know what to guarantee?

This is where documentation becomes part of the contract. The Safety comment explains exactly what the caller must ensure: which pointers must be non-null, which memory must be initialized, which invariants must hold. It transforms unsafe from a compiler bypass into a documented contract.

The standard library uses this pattern extensively. Every unsafe function in std has a Safety section explaining the preconditions. This creates an audit trail: when something goes wrong, you can trace backward through Safety comments to find where the contract was violated.

**Therefore:**

**Mark functions with undefined behavior preconditions as `unsafe fn`, and document the exact safety requirements in a `# Safety` section that explains what callers must guarantee.**

```rust
// From ascii/src/lib.rs
pub struct Ascii(Vec<u8>);

impl Ascii {
    /// Construct an `Ascii` value from `bytes`, without checking
    /// whether `bytes` actually contains well-formed ASCII.
    ///
    /// This constructor is infallible, and returns an `Ascii` directly,
    /// rather than a `Result<Ascii, NotAsciiError>` as the `from_bytes`
    /// constructor does.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains only ASCII
    /// characters: bytes no greater than 0x7f. Otherwise, the effect is
    /// undefined.
    pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Ascii {
        Ascii(bytes)
    }
}

// Safe wrapper for comparison
impl Ascii {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError> {
        if bytes.iter().any(|&byte| !byte.is_ascii()) {
            return Err(NotAsciiError(bytes));
        }
        Ok(Ascii(bytes))
    }
}

// Safe usage:
let ascii = Ascii::from_bytes(b"hello".to_vec())?;

// Unsafe usage (caller must guarantee ASCII):
let ascii = unsafe {
    // SAFETY: This byte slice contains only ASCII characters
    Ascii::from_bytes_unchecked(b"hello".to_vec())
};
```

*The unsafe boundary clearly marked, the contract explicitly stated—trust built through transparency rather than hidden through obscurity*

◆ ◆ ◆

This connects to **FUNCTION RETURNING RESULT (38)** by providing a safe alternative that validates preconditions, works with **PUBLIC FUNCTION (35)** by documenting the API contract, and supports **SAFE WRAPPER (future pattern)** by enabling encapsulated unsafe code with safe interfaces.
