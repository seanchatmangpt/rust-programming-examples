# Tutorial: Creating Type-Safe Custom String Types in Rust

## Introduction

In this tutorial, you'll learn how to create custom string types in Rust that provide compile-time guarantees about their contents. We'll build an `Ascii` type that ensures strings contain only valid ASCII characters, demonstrating Rust's powerful type system features.

### What You'll Learn

- Creating wrapper types with newtype pattern
- Implementing validation at construction time
- Working with `From` and `Into` conversion traits
- Designing fallible constructors with `Result`
- Understanding when and why to use `unsafe` code
- Comparing Rust's approach to Python's string encoding

### Prerequisites

- Basic understanding of Rust structs and enums
- Familiarity with `Result` type and error handling
- Understanding of UTF-8 vs ASCII encoding

## Why Custom String Types?

In Python, you might handle encoding like this:

```python
def process_ascii(text: str) -> str:
    # Hope that text is ASCII, or check at runtime
    encoded = text.encode('ascii')  # May raise UnicodeEncodeError
    return encoded.decode('ascii')
```

In Rust, we can encode this constraint in the type system itself, catching errors at construction time and eliminating runtime checks later.

## Step 1: Define the Ascii Type

Let's start by defining our custom type. We'll use the **newtype pattern** - wrapping an existing type in a struct to add new semantics:

```rust
/// An ASCII-encoded string.
#[derive(Debug, Eq, PartialEq)]
pub struct Ascii(Vec<u8>);
```

**Key Points:**

- The `Ascii` type wraps a `Vec<u8>` (byte vector)
- The inner `Vec<u8>` is private - users cannot access it directly
- We derive common traits: `Debug` for printing, `Eq` and `PartialEq` for comparisons
- The privacy ensures we maintain the invariant: "only valid ASCII bytes inside"

**Python Comparison:**

Python doesn't have this concept of "newtype". You'd typically use a class with validation:

```python
class Ascii:
    def __init__(self, data: bytes):
        if not all(b <= 127 for b in data):
            raise ValueError("Not ASCII")
        self._data = data
```

But Python can't enforce that `_data` won't be modified directly - it's just a convention.

## Step 2: Create a Validated Constructor

Now let's add a constructor that validates input:

```rust
/// Custom error type for ASCII validation failures
#[derive(Debug, Eq, PartialEq)]
pub struct NotAsciiError(pub Vec<u8>);

impl Ascii {
    /// Create an `Ascii` from the ASCII text in `bytes`.
    /// Returns a `NotAsciiError` if `bytes` contains any non-ASCII characters.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError> {
        if bytes.iter().any(|&byte| !byte.is_ascii()) {
            return Err(NotAsciiError(bytes));
        }
        Ok(Ascii(bytes))
    }
}
```

**Understanding the Code:**

1. **Return Type**: `Result<Ascii, NotAsciiError>` - either success or an error
2. **Validation**: `bytes.iter().any(|&byte| !byte.is_ascii())` checks each byte
3. **Error Recovery**: The error contains the original `Vec<u8>`, so the caller can recover their data
4. **Ownership**: `bytes` is moved into either `Ascii` or `NotAsciiError`

**Using the Constructor:**

```rust
let valid = b"Hello, ASCII!".to_vec();
let ascii = Ascii::from_bytes(valid).unwrap();

let invalid = vec![0xf7, 0xbf, 0xbf, 0xbf];  // Invalid UTF-8 and non-ASCII
match Ascii::from_bytes(invalid) {
    Ok(_) => println!("Success"),
    Err(NotAsciiError(original)) => {
        println!("Failed validation, got back original bytes: {:?}", original);
    }
}
```

**Python Comparison:**

```python
# Python equivalent
try:
    ascii_str = Ascii(b"Hello, ASCII!")
except ValueError as e:
    print(f"Validation failed: {e}")
```

Python raises exceptions; Rust uses `Result` for explicit error handling without exceptions.

## Step 3: Implement Safe Conversion to String

Once we have valid ASCII, converting to a Rust `String` is trivial - ASCII is a subset of UTF-8! We'll implement the `From` trait:

```rust
impl From<Ascii> for String {
    fn from(ascii: Ascii) -> String {
        // Safety: Well-formed ASCII text is also well-formed UTF-8
        unsafe { String::from_utf8_unchecked(ascii.0) }
    }
}
```

**Why `unsafe` Here?**

- `String::from_utf8_unchecked` bypasses UTF-8 validation for performance
- This is safe because we already validated that bytes are ASCII
- ASCII (0x00-0x7F) is always valid UTF-8
- The `unsafe` block promises the compiler we've upheld this invariant

**Using the Conversion:**

```rust
let bytes: Vec<u8> = b"ASCII and ye shall receive".to_vec();
let ascii: Ascii = Ascii::from_bytes(bytes).unwrap();
let string: String = ascii.into();  // or String::from(ascii)

assert_eq!(string, "ASCII and ye shall receive");
```

**Performance Benefits:**

- `from_bytes`: Scans once to validate ASCII
- `into`: Zero cost! No allocation, copies, or validation
- Total: One scan, versus two if we used `String::from_utf8()`

**Python Comparison:**

```python
# Python always validates
ascii_bytes = b"Hello"
text = ascii_bytes.decode('ascii')  # Validates at decode time
```

## Step 4: Understanding the Unsafe Constructor

For advanced users who can guarantee ASCII validity, we can provide an unchecked constructor:

```rust
impl Ascii {
    /// Construct an `Ascii` value from `bytes`, without checking
    /// whether `bytes` actually contains well-formed ASCII.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains only ASCII
    /// characters: bytes no greater than 0x7f. Otherwise, the effect
    /// is undefined behavior.
    pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Ascii {
        Ascii(bytes)
    }
}
```

**When to Use This:**

- You're reading from a trusted source (e.g., a library that guarantees ASCII)
- You've already validated the data yourself
- You're optimizing a hot path and profiling shows validation is expensive

**How to Use It:**

```rust
let bytes = b"Known ASCII".to_vec();
let ascii = unsafe {
    // SAFETY: We know this literal contains only ASCII
    Ascii::from_bytes_unchecked(bytes)
};
```

**The Danger:**

```rust
let invalid = vec![0xf7, 0xbf, 0xbf, 0xbf];
let ascii = unsafe {
    // DANGER: Contract violated! Undefined behavior ahead!
    Ascii::from_bytes_unchecked(invalid)
};
let string: String = ascii.into();
// `string` now contains invalid UTF-8 - undefined behavior if used!
```

## Complete Example: Building a CLI Tool

Let's put it all together in a practical example:

```rust
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer)?;

    match Ascii::from_bytes(buffer) {
        Ok(ascii) => {
            let string = String::from(ascii);
            println!("Valid ASCII string: {}", string);
            Ok(())
        }
        Err(NotAsciiError(bytes)) => {
            eprintln!("Error: Input contains non-ASCII bytes");
            eprintln!("First invalid byte at position: {}",
                bytes.iter().position(|&b| !b.is_ascii()).unwrap());
            std::process::exit(1);
        }
    }
}
```

**What This Demonstrates:**

1. Reading binary data from stdin
2. Attempting to validate as ASCII
3. On success: zero-cost conversion to `String`
4. On failure: recovering the original data to report helpful errors

## Key Takeaways

### Type Safety Benefits

1. **Compile-time guarantees**: Once you have an `Ascii`, it's guaranteed valid
2. **No defensive coding**: Functions accepting `Ascii` don't need to validate
3. **Zero-cost abstractions**: After validation, conversions are free
4. **Clear APIs**: The type signature documents the requirement

### Compared to Python

| Aspect | Rust `Ascii` Type | Python `str` |
|--------|------------------|--------------|
| Validation | At construction | At encode/decode |
| Runtime checks | Once (construction) | Every operation |
| Type safety | Compile-time | Runtime |
| Performance | Zero-cost after validation | Repeated validation |
| Error handling | `Result` type | Exceptions |

### Design Patterns Learned

1. **Newtype pattern**: Wrapping types to add semantics
2. **Invariant enforcement**: Private fields + validated constructors
3. **Safe abstractions**: Hiding `unsafe` behind safe APIs
4. **Error recovery**: Returning original data in errors
5. **Trait implementations**: `From` for idiomatic conversions

## Exercises

### Exercise 1: Add Methods

Add these methods to `Ascii`:

```rust
impl Ascii {
    pub fn len(&self) -> usize {
        // Return the length of the ASCII string
    }

    pub fn is_empty(&self) -> bool {
        // Check if the string is empty
    }

    pub fn as_bytes(&self) -> &[u8] {
        // Return a reference to the underlying bytes
    }
}
```

### Exercise 2: Implement Display

Implement `std::fmt::Display` so you can print `Ascii` values directly:

```rust
use std::fmt;

impl fmt::Display for Ascii {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Hint: You need to convert bytes to a str
        // Use std::str::from_utf8 or from_utf8_unchecked
    }
}
```

### Exercise 3: Create an AsciiStr Type

Create a borrowed version using string slices:

```rust
pub struct AsciiStr<'a>(&'a [u8]);

impl<'a> AsciiStr<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<AsciiStr<'a>, NotAsciiError> {
        // Validate and return AsciiStr
    }
}
```

### Exercise 4: Error Trait Implementation

Implement the `std::error::Error` trait for `NotAsciiError`:

```rust
impl std::error::Error for NotAsciiError {
    fn description(&self) -> &str {
        "byte sequence is not ASCII"
    }
}

impl fmt::Display for NotAsciiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Provide a helpful error message
    }
}
```

## Next Steps

Now that you understand custom string types, you can:

- Explore the `From` and `TryFrom` traits in more depth
- Learn about other newtype patterns (e.g., `TypedId`, `Meters`, `Seconds`)
- Study the `std::str` and `std::string` modules
- Investigate other string types like `OsString`, `PathBuf`, `CString`

## Further Reading

- [The Rust Book - Type Aliases and Newtype Pattern](https://doc.rust-lang.org/book/ch19-04-advanced-types.html)
- [std::convert Module Documentation](https://doc.rust-lang.org/std/convert/)
- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Working with UTF-8](https://doc.rust-lang.org/std/primitive.str.html)

## Reference: Complete Code

The complete implementation can be found at:
`/home/user/rust-programming-examples/ascii/src/lib.rs`

Run the tests with:
```bash
cd /home/user/rust-programming-examples/ascii
cargo test
```
