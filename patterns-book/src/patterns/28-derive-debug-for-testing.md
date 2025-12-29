# 28. DERIVE DEBUG FOR TESTING

*A type that can show its internals when inspected, like a building with glass walls for construction review.*

...within any **STRUCT DEFINITION** (1) or **ENUM WITH VARIANTS** (4), when you need to print values during testing or debugging...

◆ ◆ ◆

**How can you see what's inside your types when tests fail or code misbehaves?**

You write a test. It fails. The assertion shows:

```
thread 'tests::test_complex_eq' panicked at 'assertion failed: `(left == right)`
  left: Complex { ... },
 right: Complex { ... }'
```

But the `...` tells you nothing. You cannot see the values inside. To debug, you must add temporary println! statements, recompile, rerun, then remove them. This cycle wastes time.

Rust's `Debug` trait exists for exactly this purpose—it provides a programmer-facing representation of your type. The compiler can automatically implement it via `#[derive(Debug)]`. With Debug, test failures show the actual values. The `dbg!` macro can print values mid-execution. The `{:?}` format specifier works in all your debug output.

Every type you create should derive Debug unless it cannot (contains raw pointers, has special security requirements, or cannot be meaningfully displayed). The cost is zero at runtime—debug formatting code is only called when you explicitly ask for it.

**Therefore:**

**Add #[derive(Debug)] to every struct and enum definition unless there's a specific reason not to.**

```rust
#[derive(Clone, Copy, Debug)]
struct Complex<T> {
    /// Real portion of the complex number
    re: T,

    /// Imaginary portion of the complex number
    im: T,
}

#[test]
fn test_complex_eq() {
    let x = Complex { re: 5, im: 2 };
    let y = Complex { re: 2, im: 5 };
    assert_eq!(x * y, Complex { re: 0, im: 29 });
    // If this fails, Debug shows: "Complex { re: 5, im: 2 }"
}
```

*Debug is like adding inspection windows to your types—you can see inside whenever needed, but they don't affect normal operation.*

◆ ◆ ◆

Debug often appears alongside **DERIVE COPY FOR STACK TYPES** (20) and enables the `dbg!` macro. For user-facing output, implement **DISPLAY TRAIT FOR TO_STRING** (31) instead.
