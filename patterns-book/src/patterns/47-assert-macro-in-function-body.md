# 47. ASSERT MACRO IN FUNCTION BODY

*A function stands at its boundary, uncertain whether its caller will honor the contract required for correct operation.*

...within a **FUNCTION INVARIANTS** context, when you need to verify critical assumptions about program state...

◆ ◆ ◆

**How do you enforce preconditions and invariants in code that must always hold true?**

Functions often depend on implicit contracts: a pointer must not be null, an index must be within bounds, two values must have a specific relationship. When these assumptions are violated, the function cannot proceed safely. Yet in production code, checking every assumption with manual error handling creates verbose, defensive code that obscures the essential logic.

The tension is between robustness and clarity. Returning `Result` for every precondition transforms simple functions into error-handling ceremonies. But ignoring preconditions leads to undefined behavior, memory corruption, or silent data corruption.

In the actix-gcd web server, the GCD function requires both inputs to be non-zero—division by zero or GCD of zero has no mathematical meaning. This is not a recoverable error condition; it represents a programming mistake that should never reach production. The function cannot proceed if violated, yet wrapping it in Result obscures the mathematical purity of the algorithm.

Assertions serve a different purpose than error handling: they document and enforce invariants that should always be true if the program is correct. They fail loudly during development and testing, catching bugs before production. In release builds, they can be compiled away for performance-critical code, though by default they remain.

**Therefore:**

**Use `assert!` or `assert_eq!` to verify critical invariants that must hold true, documenting assumptions and catching violations during development.**

```rust
// From actix-gcd/src/main.rs - enforce mathematical precondition
fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);
    while m != 0 {
        if m < n {
            let t = m;
            m = n;
            n = t;
        }
        m = m % n;
    }
    n
}

// From queue tests - verify invariants about data structure state
#[test]
fn test_is_empty() {
    let mut q = Queue::new();

    assert!(q.is_empty());
    q.push('☉');
    assert!(!q.is_empty());
    q.pop();
    assert!(q.is_empty());
}

// From unsafe code - enforce alignment requirements
unsafe fn new_ref_with_flag(ptr: &T, flag: bool) -> RefWithFlag<T> {
    assert!(align_of::<T>() % 2 == 0,
            "RefWithFlag requires types with alignment >= 2");
    // ... unsafe operations that depend on this invariant
}
```

*The assertion stands as a guard at the function's threshold: when the condition holds, execution flows smoothly past; when violated, the program halts immediately with a clear message, pointing directly to the broken assumption.*

◆ ◆ ◆

Use `debug_assert!` for expensive checks that should only run in development; use `panic!` with a message when the failure needs context; combine with **CUSTOM ERROR TYPES** (31) when the condition might legitimately fail due to external input.
