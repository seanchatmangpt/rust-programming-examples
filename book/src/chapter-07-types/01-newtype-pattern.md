# The Newtype Pattern for Domain Modeling

The newtype pattern is one of Rust's most powerful architectural tools—a zero-cost abstraction that transforms primitive types into domain-specific semantic types. By wrapping a value in a single-field tuple struct, you gain type safety, enforce invariants, and create self-documenting APIs without any runtime overhead.

## Understanding the Newtype Pattern

At its core, the newtype pattern is elegantly simple: wrap a type in a struct to create a new, distinct type. The compiler treats these as different types, preventing accidental misuse, while optimizations ensure zero runtime cost.

```rust
// A newtype wrapping Vec<u8>
struct Ascii(Vec<u8>);

// These are now distinct types
let user_id = UserId(42);
let product_id = ProductId(42);

// Compiler error: mismatched types
fn get_user(id: UserId) -> User { /* ... */ }
get_user(product_id);  // ❌ Compile error!
```

This compile-time distinction is the foundation of type-driven architecture. The types themselves encode domain rules, making invalid states unrepresentable.

## Enforcing Invariants with Newtypes

The `ascii` project from our repository demonstrates newtype pattern mastery. ASCII text has a strict invariant: all bytes must be in the range `0x00..=0x7f`. Rather than validating this constraint repeatedly throughout the codebase, the newtype pattern encodes it at the type level.

```rust
/// An ASCII-encoded string.
#[derive(Debug, Eq, PartialEq)]
pub struct Ascii(
    // This must hold only well-formed ASCII text:
    // bytes from `0` to `0x7f`.
    Vec<u8>
);

impl Ascii {
    /// Create an `Ascii` from the ASCII text in `bytes`.
    /// Return a `NotAsciiError` if `bytes` contains non-ASCII characters.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError> {
        if bytes.iter().any(|&byte| !byte.is_ascii()) {
            return Err(NotAsciiError(bytes));
        }
        Ok(Ascii(bytes))
    }
}
```

This design has profound architectural implications:

1. **Single Point of Validation**: The invariant is checked once, at construction time
2. **Type System Enforcement**: Once you have an `Ascii` value, it's guaranteed valid
3. **API Clarity**: Functions accepting `Ascii` communicate their requirements in the signature
4. **Composability**: `Ascii` can be safely converted to `String` without re-validation

The key insight is that validation happens at the boundary—when external data enters your system—and the type system maintains the invariant thereafter.

## Zero-Cost Abstraction Guarantees

Rust's zero-cost abstraction principle means newtypes impose no runtime penalty. The `Ascii` wrapper is a compile-time construct; in the generated machine code, it's identical to the underlying `Vec<u8>`.

```rust
// This conversion is zero-cost—no allocation, copying, or scanning
impl From<Ascii> for String {
    fn from(ascii: Ascii) -> String {
        // SAFETY: Well-formed ASCII is also well-formed UTF-8
        unsafe { String::from_utf8_unchecked(ascii.0) }
    }
}
```

The `unsafe` block here is architecturally significant. Because the newtype guarantees the invariant, we can use unchecked conversion. The type system's compile-time guarantee enables a safe-to-use (though unsafe-internally) runtime optimization.

This pattern—safe public API with unsafe optimized internals—is fundamental to systems programming in Rust.

## Domain-Driven Design with Types

Newtypes are natural building blocks for domain-driven design. Consider modeling a financial system:

```rust
// Prevent mixing different monetary values
struct UsdCents(i64);
struct EurCents(i64);
struct BitcoinSatoshis(i64);

// Prevent invalid operations
impl UsdCents {
    pub fn add(self, other: UsdCents) -> UsdCents {
        UsdCents(self.0 + other.0)
    }

    // Cannot add different currencies without explicit conversion
    // pub fn add(self, other: EurCents) -> ??? // ❌ Doesn't compile!
}
```

This is more than type safety—it's encoding business rules in the type system. The compiler becomes a domain expert, rejecting invalid operations before they reach production.

### The Queue Example: From Concrete to Generic

The evolution from `queue` to `generic-queue` in our repository shows newtype thinking at a larger scale. The original `Queue` is essentially a newtype around two `Vec<char>` instances:

```rust
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last
    younger: Vec<char>  // younger elements, youngest last
}
```

This struct is more than data—it's a *semantic type* representing a specific data structure with queue semantics. The invariant is structural: elements are distributed between two vectors to enable efficient FIFO operations.

The public API enforces these invariants:

```rust
impl Queue {
    pub fn push(&mut self, c: char) {
        self.younger.push(c);  // Always add to younger
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }
            // Maintain invariant: swap and reverse
            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }
        self.older.pop()
    }
}
```

Users cannot directly manipulate `older` and `younger`, preventing invariant violations. This is privacy-based encapsulation working with the type system.

## When to Use Newtypes: A Decision Framework

Not every wrapped type needs to be a newtype. Here's a practical decision framework:

### Use Newtypes When:

1. **Type Confusion Risk**: Similar primitives used in different contexts
   ```rust
   // Without newtypes - easy to mix up
   fn transfer(from: i32, to: i32, amount: i32) { /* ... */ }

   // With newtypes - impossible to mix up
   fn transfer(from: AccountId, to: AccountId, amount: UsdCents) { /* ... */ }
   ```

2. **Invariant Enforcement**: The type has validity rules
   ```rust
   struct NonEmptyVec<T>(Vec<T>);  // Guarantees at least one element
   struct SortedVec<T>(Vec<T>);    // Guarantees sorted order
   ```

3. **API Clarity**: The wrapped type's meaning isn't obvious
   ```rust
   struct Meters(f64);
   struct Seconds(f64);
   // Much clearer than: fn velocity(distance: f64, time: f64) -> f64
   fn velocity(distance: Meters, time: Seconds) -> MetersPerSecond
   ```

### Use Simple Validation When:

1. **Temporary Values**: Data that doesn't propagate through the system
2. **Performance-Critical Paths**: Where validation cost matters (rare with newtypes)
3. **Simple Constraints**: A single `assert!` or early return suffices

### The Ascii Decision

Why is `Ascii` a newtype rather than just a validated `Vec<u8>`?

1. **Propagation**: ASCII strings flow through many functions
2. **Conversion Safety**: Enables unsafe-but-sound `String` conversion
3. **API Documentation**: `fn process(text: Ascii)` is self-documenting
4. **Future Extension**: Can add ASCII-specific methods without changing call sites

## Deriving Traits for Newtypes

Rust's derive macros work seamlessly with newtypes:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UserId(u64);
```

But automatic derivation has limits. For `Ascii`, manual implementation of `From<Ascii> for String` enables optimized conversion. The decision to derive versus manually implement depends on:

- **Correctness**: Does the derived implementation match domain semantics?
- **Performance**: Can manual implementation optimize using invariants?
- **API Surface**: What operations should be exposed?

## Newtypes in Larger Architectures

In production systems, newtypes form hierarchies of trust:

```rust
// Untrusted external data
struct RawInput(String);

// Validated but not sanitized
struct ValidatedInput(String);

// Safe for database insertion
struct SanitizedInput(String);

// Each conversion enforces progressively stronger invariants
impl RawInput {
    pub fn validate(self) -> Result<ValidatedInput, ValidationError> { /* ... */ }
}

impl ValidatedInput {
    pub fn sanitize(self) -> SanitizedInput { /* ... */ }
}
```

This creates "security levels" in your type system. A function accepting `SanitizedInput` documents and enforces its requirements:

```rust
fn insert_into_db(data: SanitizedInput) {
    // Caller MUST have sanitized—enforced by type system
}
```

## Performance Implications and Monomorphization

Despite being zero-cost at runtime, newtypes do affect compilation:

1. **Binary Size**: Each newtype generates separate monomorphized code
2. **Compile Time**: More types mean more type checking
3. **Debug Symbols**: Newtypes appear in debugger output

For most applications, these costs are negligible compared to the benefits. But in embedded systems or compile-time-constrained environments, consider consolidating newtypes that share implementations.

## Cross-References and Further Reading

The newtype pattern is foundational for:

- **Chapter 3: Traits** - Implementing custom behavior for newtypes
- **Chapter 4: Modules** - Visibility and encapsulation strategies
- **Chapter 5: Errors** - Type-safe error handling with custom error newtypes

The `ascii` project demonstrates production-quality newtype design. Study its balance of safety, performance, and usability.

## Conclusion

The newtype pattern transforms Rust's type system from a correctness tool into an architectural framework. By encoding domain invariants as types, you:

- Eliminate entire classes of bugs at compile time
- Create self-documenting, misuse-resistant APIs
- Enable optimizations based on type-level guarantees
- Build systems where invalid states are unrepresentable

This is type-driven architecture in practice: using the type system not merely to prevent errors, but to actively guide correct design.
