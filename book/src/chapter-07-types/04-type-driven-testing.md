# Type-Driven Testing

Type-driven testing inverts the traditional testing pyramid. When types encode correctness properties, entire classes of tests become unnecessary—the compiler validates what you would have tested manually. This doesn't eliminate testing, but transforms it: instead of testing that functions reject invalid inputs, you test that types prevent invalid inputs from existing. The result is higher confidence with fewer tests.

## Types as Specifications

In type-driven architecture, the type signature is a specification. Consider these two functions:

```rust
// Specification unclear - what inputs are valid?
fn process(data: Vec<u8>) -> String {
    // What if data isn't ASCII?
}

// Specification encoded in types
fn process(data: Ascii) -> String {
    String::from(data)  // Guaranteed safe
}
```

The second signature is a contract: "Give me valid ASCII, I'll give you a String." The type system enforces this contract—you can't call the function with invalid data because invalid `Ascii` values don't exist.

This is the essence of type-driven testing: **make invalid states unrepresentable, then test what remains.**

## The Testing Pyramid Transformed

Traditional testing pyramid:

```
    Unit Tests (many) - Test individual functions
         |
    Integration Tests (fewer) - Test component interaction
         |
    System Tests (fewest) - Test entire system
```

Type-driven testing pyramid:

```
    Type-Level Guarantees (compile-time) - Invalid states impossible
         |
    Property Tests (generated) - Verify type invariants hold
         |
    Integration Tests (focused) - Test business logic
         |
    Example-Based Tests (minimal) - Document edge cases
```

The burden shifts upward—from runtime tests to compile-time guarantees.

## Compile-Time Guarantees Reducing Test Burden

Our `Ascii` type demonstrates this shift. Without newtypes:

```rust
fn to_uppercase(data: Vec<u8>) -> Result<String, Error> {
    // Test case 1: Valid ASCII
    assert_eq!(to_uppercase(b"hello".to_vec())?, "HELLO");

    // Test case 2: Invalid ASCII - should return error
    assert!(to_uppercase(vec![0xFF]).is_err());

    // Test case 3: Empty vector
    assert_eq!(to_uppercase(vec![])?, "");

    // Test case 4: Mixed valid/invalid
    assert!(to_uppercase(vec![b'A', 0xFF, b'B']).is_err());
}
```

Four test cases just to validate input handling. With newtypes:

```rust
fn to_uppercase(data: Ascii) -> String {
    // Test case 1: Valid ASCII becomes uppercase
    let input = Ascii::from_bytes(b"hello".to_vec()).unwrap();
    assert_eq!(to_uppercase(input), "HELLO");

    // No need for invalid input tests - type system prevents them!
}
```

Two test cases eliminated. The type system proves they're impossible.

## Property-Based Testing with Types

Types enable property-based testing—generating random inputs and verifying invariants hold. For `Queue<T>`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn push_pop_order(elements: Vec<char>) {
        let mut q = Queue::new();

        // Property: pushing N elements then popping N times
        // returns elements in FIFO order
        for &elem in &elements {
            q.push(elem);
        }

        let mut popped = Vec::new();
        while let Some(elem) = q.pop() {
            popped.push(elem);
        }

        assert_eq!(popped, elements);
    }

    #[test]
    fn is_empty_invariant(ops: Vec<QueueOp>) {
        let mut q = Queue::new();

        // Property: is_empty() matches actual emptiness
        for op in ops {
            match op {
                QueueOp::Push(c) => {
                    q.push(c);
                    assert!(!q.is_empty());
                }
                QueueOp::Pop => {
                    let was_empty = q.is_empty();
                    let result = q.pop();
                    assert_eq!(result.is_none(), was_empty);
                }
            }
        }
    }
}
```

Property tests verify invariants across thousands of generated inputs. The type system ensures generated inputs are *valid*—you're testing the algorithm, not input validation.

## Types Guiding Test Design

Type signatures guide what to test. For our `Interval<T>` type:

```rust
#[derive(Debug, PartialEq)]
struct Interval<T> {
    lower: T,  // inclusive
    upper: T,  // exclusive
}

impl<T: PartialOrd> PartialOrd<Interval<T>> for Interval<T> {
    fn partial_cmp(&self, other: &Interval<T>) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.lower >= other.upper {
            Some(Ordering::Greater)
        } else if self.upper <= other.lower {
            Some(Ordering::Less)
        } else {
            None  // Overlapping intervals
        }
    }
}
```

The type signature tells us:
1. `T` must implement `PartialOrd` (test with types that do/don't)
2. Result is `Option<Ordering>` (test `Some` and `None` cases)
3. Comparison is partial (test overlapping intervals)

Tests follow directly from the type:

```rust
#[test]
fn test_interval_ordering() {
    // Non-overlapping: should be ordered
    assert!(Interval { lower: 10, upper: 20 } < Interval { lower: 20, upper: 40 });
    assert!(Interval { lower: 7, upper: 8 } > Interval { lower: 0, upper: 1 });

    // Equal intervals
    let interval = Interval { lower: 7, upper: 8 };
    assert_eq!(interval.partial_cmp(&interval), Some(Ordering::Equal));

    // Overlapping: should be None
    let left = Interval { lower: 10, upper: 30 };
    let right = Interval { lower: 20, upper: 40 };
    assert_eq!(left.partial_cmp(&right), None);
    assert!(!(left < right));
    assert!(!(left >= right));
}
```

Every test corresponds to a branch in the type-driven logic. The type signature is the test specification.

## Testing Generic Code

Generic code presents unique testing challenges. For `Complex<T>`:

```rust
impl<T> Add for Complex<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}
```

Tests should verify the implementation works for multiple types:

```rust
#[test]
fn test_complex_add_f64() {
    let a = Complex { re: 1.0_f64, im: 2.0 };
    let b = Complex { re: 3.0, im: 4.0 };
    assert_eq!(a + b, Complex { re: 4.0, im: 6.0 });
}

#[test]
fn test_complex_add_i32() {
    let a = Complex { re: 1_i32, im: 2 };
    let b = Complex { re: 3, im: 4 };
    assert_eq!(a + b, Complex { re: 4, im: 6 });
}

#[test]
fn test_complex_add_generic() {
    fn test_add<T>(a_re: T, a_im: T, b_re: T, b_im: T, expected_re: T, expected_im: T)
    where
        T: Add<Output = T> + PartialEq + Copy,
    {
        let a = Complex { re: a_re, im: a_im };
        let b = Complex { re: b_re, im: b_im };
        let c = a + b;
        assert_eq!(c.re, expected_re);
        assert_eq!(c.im, expected_im);
    }

    test_add(1.0_f64, 2.0, 3.0, 4.0, 4.0, 6.0);
    test_add(1_i32, 2, 3, 4, 4, 6);
}
```

The generic test function ensures the algorithm works across types, not just for specific instantiations.

## Testing Type-State Transitions

For phantom types and type-state patterns, tests verify that valid transitions compile and invalid ones don't:

```rust
// This test verifies valid transitions compile
#[test]
fn connection_state_transitions() {
    let conn = Connection::<Disconnected>::new();
    let conn = conn.connect("127.0.0.1:8080").unwrap();
    let conn = conn.authenticate(&credentials).unwrap();
    conn.send(data).unwrap();
}

// Invalid transitions should not compile
// These are "compile-fail" tests
#[test]
#[should_not_compile]
fn cannot_send_while_disconnected() {
    let conn = Connection::<Disconnected>::new();
    conn.send(data).unwrap();  // Should not compile
}
```

Rust doesn't have built-in compile-fail tests, but tools like `trybuild` enable them:

```rust
#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/invalid_state_transitions.rs");
}
```

## Reducing Test Burden: What Not to Test

Type-driven design eliminates certain test categories:

### Don't Test What the Compiler Guarantees

```rust
// ❌ Unnecessary test - compiler enforces this
#[test]
fn ascii_only_accepts_valid_ascii() {
    let ascii = Ascii::from_bytes(vec![0xFF]);
    assert!(ascii.is_err());
}

// The type guarantees this - construction is the test
```

### Don't Test Impossible States

```rust
// ❌ Unnecessary - this state can't exist
#[test]
fn builder_with_missing_fields() {
    let builder = Builder::new();
    // Can't call build() without required fields - won't compile
}
```

### Do Test Business Logic

```rust
// ✅ Necessary - tests algorithm, not types
#[test]
fn queue_maintains_fifo_order() {
    let mut q = Queue::new();
    q.push('A');
    q.push('B');
    q.push('C');
    assert_eq!(q.pop(), Some('A'));
    assert_eq!(q.pop(), Some('B'));
    assert_eq!(q.pop(), Some('C'));
}
```

## Using Types to Prevent Bugs

The `Ascii` project shows how types prevent entire bug classes. Consider this unsafe conversion:

```rust
impl From<Ascii> for String {
    fn from(ascii: Ascii) -> String {
        // SAFETY: Well-formed ASCII is also well-formed UTF-8
        unsafe { String::from_utf8_unchecked(ascii.0) }
    }
}
```

This uses `unsafe`, but is it safe? The type system proves it:

1. `Ascii` can only be constructed via `from_bytes()` which validates ASCII
2. ASCII (bytes 0x00-0x7F) is a subset of UTF-8
3. Therefore, all `Ascii` values are valid UTF-8

The unsafe code is safe because the type invariant guarantees the precondition. Tests verify the invariant is maintained:

```rust
#[test]
fn ascii_from_bytes_validates() {
    // Valid ASCII
    let result = Ascii::from_bytes(b"hello".to_vec());
    assert!(result.is_ok());

    // Invalid ASCII
    let result = Ascii::from_bytes(vec![0xFF]);
    assert!(result.is_err());
}

#[test]
fn ascii_to_string_conversion() {
    let ascii = Ascii::from_bytes(b"Hello, world!".to_vec()).unwrap();
    let s = String::from(ascii);
    assert_eq!(s, "Hello, world!");
}
```

We test the *construction* validates and the *conversion* succeeds. We don't test that `from_utf8_unchecked` works—that's the standard library's responsibility.

## Testing Invariants with Debug Assertions

Type invariants can be verified with debug assertions:

```rust
impl<T> Queue<T> {
    fn check_invariants(&self) {
        debug_assert!(
            self.older.is_empty() || self.younger.is_empty()
            || !self.older.is_empty(),
            "Queue invariant violated: both older and younger contain elements \
             but older is empty during check"
        );
    }

    pub fn push(&mut self, t: T) {
        self.younger.push(t);
        self.check_invariants();
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }
            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }
        let result = self.older.pop();
        self.check_invariants();
        result
    }
}
```

These assertions are active in debug builds but compiled out in release. They catch invariant violations during testing without affecting production performance.

## Integration Testing Type-Safe APIs

Integration tests verify that type-safe components compose correctly:

```rust
#[test]
fn process_ascii_pipeline() {
    // Integration test: read ASCII from file, process, write back
    let input = std::fs::read("input.txt").unwrap();

    let ascii = match Ascii::from_bytes(input) {
        Ok(a) => a,
        Err(e) => panic!("Input file contains non-ASCII: {:?}", e),
    };

    let processed = process_ascii(ascii);  // Type-safe processing
    let output = String::from(processed);  // Type-safe conversion

    std::fs::write("output.txt", output).unwrap();
}
```

The types guide the test: we test that the pipeline composes, not that each component rejects invalid inputs—the types already do that.

## Measuring Type-Driven Test Coverage

Traditional code coverage metrics are less meaningful for type-driven code. Instead, focus on:

1. **Invariant Coverage**: Are all type invariants verified?
2. **Branch Coverage**: Are all logical branches tested?
3. **Property Coverage**: Do property tests explore the input space?
4. **Integration Coverage**: Do tests verify component composition?

For `Ascii`, this means:
- ✅ Invariant: ASCII bytes are validated on construction
- ✅ Branches: Valid and invalid inputs tested
- ✅ Properties: Round-trip conversions work
- ✅ Integration: Ascii → String conversion tested

## Cross-References

Type-driven testing builds on:

- **Chapter 1: Newtypes** - Encoding invariants in types
- **Chapter 2: Phantom Types** - Testing state transitions
- **Chapter 3: Generics** - Testing generic algorithms
- **Chapter 5: Errors** - Type-safe error handling

## Conclusion

Type-driven testing shifts verification from runtime to compile time. The compiler becomes your first line of defense, proving properties that would otherwise require extensive test suites. This doesn't eliminate testing—it elevates it.

Instead of testing that your code handles invalid inputs, you test that your types prevent invalid inputs from existing. Instead of testing state transitions, you make invalid transitions unrepresentable. Instead of testing edge cases, you use property tests to verify invariants across the entire input space.

The result is higher confidence with less code: fewer tests, stronger guarantees, and architectural correctness enforced by the type system itself.
