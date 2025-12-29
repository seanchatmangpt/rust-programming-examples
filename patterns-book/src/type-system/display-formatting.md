# Display Formatting

## Pattern Name
**Display Formatting for Human-Readable Output**

## Context

You have defined custom types (structs, enums, newtypes) that need to be presented to users in a readable format. Common scenarios include:
- Domain types that should print in natural notation (complex numbers, dates, quantities)
- Types that have multiple representation formats (Cartesian vs polar for complex numbers)
- Error types that need user-friendly messages
- Configuration objects that should display their settings
- Log messages that need consistent formatting

Rust provides two main formatting traits: `Debug` (for programmer-facing output) and `Display` (for user-facing output). Understanding when and how to implement each is essential for professional library design.

## Problem

**How do you provide human-readable string representations of custom types that integrate seamlessly with Rust's formatting system while offering appropriate detail for different audiences?**

You need to:
- Distinguish between programmer-facing debug output and user-facing display output
- Integrate with Rust's `format!`, `println!`, and string formatting macros
- Support multiple formatting styles when appropriate (e.g., alternate formatting)
- Avoid string allocation when not needed (write directly to formatter)
- Maintain consistency with standard library formatting conventions
- Enable composition (types containing your type should format correctly)

## Forces

- **Debug vs Display**: Debug is for developers, Display is for end users
- **Allocation**: Formatting should write directly to output, not allocate intermediate strings
- **Conventions**: Mathematical/domain notation vs programmer-friendly representation
- **Flexibility**: Some types have multiple valid string representations
- **Error Handling**: Formatting rarely fails, but trait requires Result return
- **Derivability**: Debug can be auto-derived, Display cannot
- **Composition**: Formatted output should compose with other formatted output

## Solution

**Implement the `Display` trait for user-facing output and derive or implement `Debug` for programmer-facing diagnostics, using the `write!` macro to format directly to the provided formatter.**

### Core Display Implementation

From `/home/user/rust-programming-examples/complex/src/lib.rs`:

```rust
use std::fmt;

#[derive(Copy, Clone, Debug)]
struct Complex {
    re: f64,
    im: f64
}

impl fmt::Display for Complex {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        let im_sign = if self.im < 0.0 { '-' } else { '+' };
        write!(dest, "{} {} {}i", self.re, im_sign, f64::abs(self.im))
    }
}

// Usage
let c = Complex { re: -0.5, im: 0.866 };
assert_eq!(format!("{}", c), "-0.5 + 0.866i");

let c2 = Complex { re: -0.5, im: -0.866 };
assert_eq!(format!("{}", c2), "-0.5 - 0.866i");
```

### Key Elements

1. **Import fmt module**: `use std::fmt;`
2. **Implement Display trait**: `impl fmt::Display for Complex`
3. **Signature**: `fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result`
4. **write! macro**: Writes directly to formatter without allocating
5. **Return Result**: Always return `Ok(())` unless write fails

### Debug vs Display

```rust
#[derive(Debug)]  // Auto-generated Debug implementation
struct Complex {
    re: f64,
    im: f64
}

let c = Complex { re: 3.0, im: 4.0 };

// Debug format (for programmers)
println!("{:?}", c);     // Complex { re: 3.0, im: 4.0 }
println!("{:#?}", c);    // Pretty-printed multi-line (for complex structs)

// Display format (for users) - requires manual implementation
println!("{}", c);       // 3 + 4i
```

**When to use each**:
- **Debug (`{:?}`)**: Development, logging, error messages, assert failures
- **Display (`{}`)**: User-facing output, formatted messages, reports

### Alternate Formatting

Support multiple display formats using the alternate flag:

```rust
impl fmt::Display for Complex {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        let (re, im) = (self.re, self.im);

        if dest.alternate() {
            // Alternate format: polar notation
            let abs = f64::sqrt(re * re + im * im);
            let angle = f64::atan2(im, re) / std::f64::consts::PI * 180.0;
            write!(dest, "{} ∠ {}°", abs, angle)
        } else {
            // Standard format: Cartesian notation
            let im_sign = if im < 0.0 { '-' } else { '+' };
            write!(dest, "{} {} {}i", re, im_sign, f64::abs(im))
        }
    }
}

let c = Complex { re: 0.0, im: 2.0 };
assert_eq!(format!("{}", c),   "0 + 2i");      // Standard
assert_eq!(format!("{:#}", c), "2 ∠ 90°");     // Alternate
```

### Forwarding to Inner Types

For wrapper types, you can delegate formatting:

```rust
struct UserId(u64);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User#{}", self.0)
    }
}

impl fmt::Debug for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UserId({})", self.0)
    }
}

let id = UserId(42);
println!("{}", id);    // User#42
println!("{:?}", id);  // UserId(42)
```

### Formatting with Precision and Width

Respect formatter parameters:

```rust
impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let im_sign = if self.im < 0.0 { '-' } else { '+' };

        // Use formatter's precision/width if specified
        match f.precision() {
            Some(prec) => write!(
                f,
                "{:.prec$} {} {:.prec$}i",
                self.re,
                im_sign,
                f64::abs(self.im),
                prec = prec
            ),
            None => write!(f, "{} {} {}i", self.re, im_sign, f64::abs(self.im)),
        }
    }
}

let c = Complex { re: 1.0/3.0, im: 1.0/7.0 };
println!("{}", c);       // 0.3333333333333333 + 0.14285714285714285i
println!("{:.2}", c);    // 0.33 + 0.14i
```

### Error Handling in Formatting

The `fmt` method returns `fmt::Result`, which is `Result<(), fmt::Error>`. The `write!` macro propagates errors automatically:

```rust
impl fmt::Display for MyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Part 1")?;  // ? propagates errors
        write!(f, " Part 2")?;
        Ok(())  // Explicit success
    }
}
```

In practice, formatting rarely fails (only if writing to the underlying stream fails).

## Resulting Context

### Benefits

- **Standard Integration**: Works with `format!`, `println!`, `to_string()`, etc.
- **No Allocation**: Writes directly to output buffer (zero-copy when possible)
- **User-Friendly**: Display provides human-readable output
- **Developer-Friendly**: Debug provides detailed diagnostic information
- **Flexible**: Alternate formatting enables multiple representations
- **Composable**: Types containing your type inherit formatting capability

### Liabilities

- **Manual Implementation**: Display cannot be auto-derived (requires custom logic)
- **Convention Knowledge**: Must know domain-specific notation
- **Limited Customization**: Only one Display implementation per type (can't have multiple without newtypes)
- **Error Handling**: Must return Result even when errors are impossible
- **Internationalization**: Display doesn't support localization directly

### Standard Library Conventions

- **Numbers**: Right-aligned by default, can use `{:<}` for left-align
- **Padding**: Use `{:5}` for width, `{:05}` for zero-padding
- **Precision**: Use `{:.2}` for decimal precision
- **Alternate**: Use `{:#}` for alternate representation
- **Debug**: Use `{:?}` for Debug, `{:#?}` for pretty Debug

## Related Patterns

- **Newtype Wrapper**: Newtypes often need custom Display implementations
- **Operator Overloading**: Display complements arithmetic operations
- **Error Handling**: Custom error types should implement Display for error messages
- **Builder Pattern**: Builders often implement Debug for configuration inspection

## Known Uses

- **Standard Library**: All primitive types implement both Debug and Display
- **chrono**: Date/time types support multiple display formats
- **url**: URL type displays as string
- **std::net**: IP addresses, socket addresses implement Display
- **std::path**: Path types display as string slices
- **error types**: `std::io::Error`, `std::fmt::Error`, etc.

## Example: Error Type with Display

```rust
use std::fmt;
use std::error::Error;

#[derive(Debug)]
enum ConfigError {
    MissingField(String),
    InvalidValue { field: String, value: String },
    ParseError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
            ConfigError::InvalidValue { field, value } => {
                write!(f, "Invalid value '{}' for field '{}'", value, field)
            }
            ConfigError::ParseError(msg) => {
                write!(f, "Parse error: {}", msg)
            }
        }
    }
}

impl Error for ConfigError {}

// Usage
let err = ConfigError::MissingField("database_url".to_string());
println!("{}", err);  // Missing required field: database_url
eprintln!("Error: {}", err);  // User-friendly error message
```

## Example: Multiple Format Representations

```rust
use std::fmt;

struct Duration {
    seconds: u64,
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            // Alternate: human-readable
            let hours = self.seconds / 3600;
            let minutes = (self.seconds % 3600) / 60;
            let seconds = self.seconds % 60;
            write!(f, "{}h {}m {}s", hours, minutes, seconds)
        } else {
            // Standard: seconds only
            write!(f, "{}s", self.seconds)
        }
    }
}

let duration = Duration { seconds: 3661 };
println!("{}", duration);     // 3661s
println!("{:#}", duration);   // 1h 1m 1s
```

## Guidelines

### Implementing Display

1. **Use write! macro**: Don't allocate intermediate strings
   ```rust
   // ❌ AVOID
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       let s = format!("{} + {}i", self.re, self.im);
       write!(f, "{}", s)
   }

   // ✅ PREFER
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       write!(f, "{} + {}i", self.re, self.im)
   }
   ```

2. **Follow domain conventions**: Use standard notation for your domain
   - Complex numbers: `a + bi`
   - Dates: `YYYY-MM-DD` or locale-specific
   - Quantities: `10 meters`, `5.5 kg`

3. **Keep it simple**: Display should be concise; Debug can be verbose

4. **Be consistent**: Similar types should format similarly

5. **Handle edge cases**: Zero, negative, special values

### Implementing Debug

1. **Prefer derive**: Use `#[derive(Debug)]` when possible
   ```rust
   #[derive(Debug)]
   struct Point { x: i32, y: i32 }
   ```

2. **Manual Debug for custom format**:
   ```rust
   impl fmt::Debug for Point {
       fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
           f.debug_struct("Point")
               .field("x", &self.x)
               .field("y", &self.y)
               .finish()
       }
   }
   ```

3. **DebugStruct, DebugTuple helpers**: For struct-like formatting
   ```rust
   impl fmt::Debug for Complex {
       fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
           f.debug_struct("Complex")
               .field("re", &self.re)
               .field("im", &self.im)
               .finish()
       }
   }
   ```

### Testing Format Output

```rust
#[test]
fn test_complex_display() {
    let c1 = Complex { re: -0.5, im: 0.866 };
    assert_eq!(format!("{}", c1), "-0.5 + 0.866i");

    let c2 = Complex { re: 3.0, im: -4.0 };
    assert_eq!(format!("{}", c2), "3 - 4i");

    let c3 = Complex { re: 0.0, im: 2.0 };
    assert_eq!(format!("{:#}", c3), "2 ∠ 90°");
}
```

## Common Mistakes

### Mistake 1: Allocating Unnecessarily

```rust
// ❌ WRONG: Creates intermediate String
impl fmt::Display for MyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self.to_custom_string();  // Allocates!
        write!(f, "{}", s)
    }
}

// ✅ RIGHT: Write directly to formatter
impl fmt::Display for MyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.field1)?;
        write!(f, " - {}", self.field2)?;
        Ok(())
    }
}
```

### Mistake 2: Ignoring Formatter Options

```rust
// ❌ WRONG: Ignores precision, width, etc.
impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)  // Loses formatter options
    }
}

// ✅ RIGHT: Forward formatter options to inner type
impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Forwards precision, width, etc. to f64 formatter
        fmt::Display::fmt(&self.value, f)
    }
}
```

### Mistake 3: Display and Debug Inconsistency

```rust
// ❌ CONFUSING: Debug shows different value than Display
impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0 * 2)  // Multiplies!?
    }
}

// ✅ RIGHT: Debug and Display should show same logical value
impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wrapper({})", self.0)
    }
}

impl fmt::Debug for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Wrapper").field(&self.0).finish()
    }
}
// Both show the actual value, just different styles
```

## Advanced: Custom Format Traits

For specialized formatting needs, you can define custom traits:

```rust
trait ToHex {
    fn to_hex(&self) -> String;
}

impl ToHex for u32 {
    fn to_hex(&self) -> String {
        format!("{:#x}", self)
    }
}

// Or better, define a formatting trait
trait HexDisplay {
    fn fmt_hex(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl HexDisplay for u32 {
    fn fmt_hex(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self)
    }
}
```

But for most cases, using Display with alternate formatting (`{:#}`) is sufficient.

## Why This Pattern Works in Rust

Rust's formatting system is more principled than many languages:

1. **Separation of Concerns**: Debug vs Display clearly separates audiences
2. **Trait-Based**: Formatting is a trait, enabling generic code
3. **Zero-Copy**: Formatter writes directly, avoiding allocations
4. **Type Safety**: Format string is checked at compile time (with `format_args!` macro)
5. **Composability**: Formatted types compose naturally
6. **Extension**: Custom types integrate seamlessly with formatting macros

### Comparison with Other Languages

**Python's `__str__` and `__repr__`**:
```python
class Complex:
    def __str__(self):   # Like Display
        return f"{self.re} + {self.im}i"
    def __repr__(self):  # Like Debug
        return f"Complex({self.re}, {self.im})"
```

**Java's `toString()`**:
```java
public String toString() {  // Like Display
    return re + " + " + im + "i";
}
```

Rust's approach is more flexible because:
- No allocation required (writes to buffer)
- Separate Debug and Display (Python conflates in `repr`)
- Generic formatting (precision, width, alignment)
- Multiple format styles via alternate flag

## Performance Considerations

```rust
// Fast: No allocation
let s = format!("{}", my_complex);

// Faster: Reuses buffer
let mut buffer = String::new();
write!(&mut buffer, "{}", my_complex).unwrap();

// Fastest: Write to output stream directly
println!("{}", my_complex);  // No intermediate String
```

For performance-critical code, prefer writing directly to output rather than creating intermediate strings.

## Complete Example

From the actual codebase (`/home/user/rust-programming-examples/complex/src/lib.rs`):

```rust
#[derive(Copy, Clone, Debug)]
struct Complex { re: f64, im: f64 }

use std::fmt;

impl fmt::Display for Complex {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        let im_sign = if self.im < 0.0 { '-' } else { '+' };
        write!(dest, "{} {} {}i", self.re, im_sign, f64::abs(self.im))
    }
}

#[test]
fn test_complex_display() {
    let one_twenty = Complex { re: -0.5, im: 0.866 };
    assert_eq!(format!("{}", one_twenty), "-0.5 + 0.866i");

    let two_forty = Complex { re: -0.5, im: -0.866 };
    assert_eq!(format!("{}", two_forty), "-0.5 - 0.866i");
}
```

### With Alternate Formatting

```rust
impl fmt::Display for Complex {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        let (re, im) = (self.re, self.im);
        if dest.alternate() {
            // Polar notation
            let abs = f64::sqrt(re * re + im * im);
            let angle = f64::atan2(im, re) / std::f64::consts::PI * 180.0;
            write!(dest, "{} ∠ {}°", abs, angle)
        } else {
            // Cartesian notation
            let im_sign = if im < 0.0 { '-' } else { '+' };
            write!(dest, "{} {} {}i", re, im_sign, f64::abs(im))
        }
    }
}

let ninety = Complex { re: 0.0, im: 2.0 };
assert_eq!(format!("{}", ninety),  "0 + 2i");
assert_eq!(format!("{:#}", ninety), "2 ∠ 90°");
```

This pattern demonstrates how Rust's formatting system enables **expressive, efficient, type-safe string formatting** that integrates seamlessly with the standard library and user code.
