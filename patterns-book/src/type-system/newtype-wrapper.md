# Newtype Wrapper

## Pattern Name
**Newtype Wrapper for Type Safety**

## Context

You are working with primitive types (integers, strings, booleans) or existing types that can represent multiple distinct concepts in your domain. For example:
- User IDs vs Product IDs (both `u64`)
- Distances in meters vs miles (both `f64`)
- HTML strings vs plain text strings (both `String`)
- File paths vs URLs (both `String`)

Rust's strong type system can prevent logical errors, but only if different concepts use different types. The newtype pattern creates distinct types with zero runtime cost.

## Problem

**How do you create distinct types for different semantic concepts while avoiding the overhead and boilerplate of full wrapper types?**

You need to:
- Prevent mixing up semantically different values (e.g., adding meters to miles)
- Make the type system enforce domain invariants
- Avoid runtime overhead (boxing, indirection, extra allocations)
- Selectively expose only appropriate operations for each type
- Maintain ergonomics for legitimate operations

## Forces

- **Type Safety vs Ergonomics**: More distinct types mean more safety but also more conversions
- **Zero-Cost Abstraction**: Wrappers should have no runtime overhead
- **API Surface**: Need to decide which operations to expose (some primitives support operations that don't make sense semantically)
- **Conversions**: Must provide controlled ways to convert between wrapped and unwrapped values
- **Trait Implementation**: Need to selectively implement traits that make semantic sense
- **Pattern Matching**: Should be able to destructure and pattern match on wrapped values
- **Serialization**: Often need to serialize as the underlying type, not as a struct

## Solution

**Create a tuple struct with a single field holding the wrapped value, implementing only the traits and operations that make semantic sense for your domain concept.**

### Basic Newtype Pattern

```rust
// Prevent mixing different ID types
struct UserId(u64);
struct ProductId(u64);
struct OrderId(u64);

// This won't compile - type safety!
fn get_user(id: UserId) -> User { /* ... */ }
let product_id = ProductId(42);
// get_user(product_id);  // ERROR: expected UserId, found ProductId
```

### With Appropriate Trait Implementations

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UserId(u64);

impl UserId {
    fn new(id: u64) -> Self {
        UserId(id)
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

// Only implement Display, not arithmetic operations
use std::fmt;

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User#{}", self.0)
    }
}
```

### Enforcing Invariants

Newtypes can enforce domain constraints:

```rust
/// Temperature in Kelvin (always non-negative)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Kelvin(f64);

impl Kelvin {
    /// Create temperature in Kelvin
    /// Returns None if temperature is below absolute zero
    fn new(k: f64) -> Option<Self> {
        if k >= 0.0 {
            Some(Kelvin(k))
        } else {
            None
        }
    }

    fn from_celsius(c: f64) -> Option<Self> {
        Self::new(c + 273.15)
    }

    fn as_celsius(&self) -> f64 {
        self.0 - 273.15
    }
}

// Users can't create invalid temperatures
let valid = Kelvin::new(300.0);     // Some(Kelvin(300.0))
let invalid = Kelvin::new(-10.0);   // None
```

### Selective Operator Implementation

From the complex number example, we can apply this to domain-specific quantities:

```rust
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Meters(f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Seconds(f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct MetersPerSecond(f64);

use std::ops::{Add, Sub, Div};

// Meters can be added to meters
impl Add for Meters {
    type Output = Meters;
    fn add(self, rhs: Meters) -> Meters {
        Meters(self.0 + rhs.0)
    }
}

// Meters divided by seconds gives velocity
impl Div<Seconds> for Meters {
    type Output = MetersPerSecond;
    fn div(self, rhs: Seconds) -> MetersPerSecond {
        MetersPerSecond(self.0 / rhs.0)
    }
}

// But meters + seconds is nonsensical and won't compile!
// let nonsense = Meters(10.0) + Seconds(5.0);  // ERROR
```

### Transparent Wrapper (Advanced)

For zero-cost conversion without runtime checks:

```rust
/// ASCII-only string (guaranteed valid ASCII)
#[repr(transparent)]
struct AsciiStr(str);

impl AsciiStr {
    /// SAFETY: Caller must ensure bytes are valid ASCII
    unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &AsciiStr {
        std::mem::transmute(bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Option<&AsciiStr> {
        if bytes.iter().all(|&b| b < 128) {
            Some(unsafe { AsciiStr::from_bytes_unchecked(bytes) })
        } else {
            None
        }
    }
}
```

The `#[repr(transparent)]` attribute ensures the newtype has the same memory layout as the wrapped type.

### Pattern Matching and Destructuring

```rust
struct Point(i32, i32);

let p = Point(10, 20);

// Destructure in pattern match
match p {
    Point(x, y) => println!("x: {}, y: {}", x, y),
}

// Destructure in let binding
let Point(x, y) = p;
```

### Integration with Serialization

Using serde (common pattern):

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]  // Serialize as u64, not as struct
struct UserId(u64);

// JSON: 42  (not {"UserId": 42})
```

## Resulting Context

### Benefits

- **Type Safety**: Compiler prevents mixing semantically different values
- **Zero Runtime Cost**: Newtype wrapper is optimized away completely
- **Self-Documenting**: Function signatures clearly show intent (`UserId` vs `ProductId`)
- **Controlled API**: Only expose operations that make semantic sense
- **Invariant Enforcement**: Constructor can validate domain constraints
- **No Name Collisions**: Different types can have different method names for same operation

### Liabilities

- **Conversion Boilerplate**: Need to wrap/unwrap when interfacing with external APIs
- **Trait Implementation**: Must manually implement traits (can't always use derive)
- **Discoverability**: IDEs may not suggest operations on the underlying type
- **Ecosystem Integration**: Some libraries expect primitive types and require unwrapping
- **Proliferation**: Can lead to many small types that need conversions

### When to Use

✅ **Use Newtype When:**
- Different primitive values have different semantic meanings
- You need to prevent mixing incompatible values (IDs, units of measure)
- You want to restrict operations (IDs shouldn't be added)
- You need to enforce invariants (non-negative values, valid ranges)

❌ **Don't Use Newtype When:**
- The wrapper would have no behavioral difference
- All operations of the underlying type are valid
- The cognitive overhead outweighs the safety benefits

## Related Patterns

- **Operator Overloading**: Newtypes can implement operators selectively for domain-appropriate operations
- **Type State Pattern**: Newtypes can represent different states in a state machine
- **Phantom Types**: Combine with phantom types for additional compile-time guarantees
- **Builder Pattern**: Constructors can enforce complex invariants before creating the newtype

## Known Uses

- **std::io::Error**: Wraps error kinds with additional context
- **PathBuf/Path**: Distinguish paths from generic strings
- **NonZeroU32**: Standard library newtype guaranteeing non-zero integers (enables niche optimization)
- **Bytes** crate: `Bytes` wraps `Vec<u8>` with reference-counted semantics
- **chrono**: `DateTime<Utc>` vs `DateTime<Local>` prevent timezone mixing
- **diesel**: Uses newtypes extensively to prevent SQL injection and type errors

## Example: Preventing Unit Confusion

A real-world example inspired by domain modeling:

```rust
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Meters(f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Feet(f64);

impl Meters {
    fn new(m: f64) -> Self { Meters(m) }

    fn to_feet(self) -> Feet {
        Feet(self.0 * 3.28084)
    }
}

impl Feet {
    fn new(ft: f64) -> Self { Feet(ft) }

    fn to_meters(self) -> Meters {
        Meters(self.0 / 3.28084)
    }
}

use std::ops::Add;

impl Add for Meters {
    type Output = Meters;
    fn add(self, rhs: Meters) -> Meters {
        Meters(self.0 + rhs.0)
    }
}

impl Add for Feet {
    type Output = Feet;
    fn add(self, rhs: Feet) -> Feet {
        Feet(self.0 + rhs.0)
    }
}

// Type-safe distance calculation
fn total_distance(d1: Meters, d2: Meters) -> Meters {
    d1 + d2
}

// This prevents the Mars Climate Orbiter disaster!
let metric = Meters::new(100.0);
let imperial = Feet::new(328.0);

// total_distance(metric, imperial);  // ERROR: type mismatch!
total_distance(metric, imperial.to_meters());  // OK: explicit conversion
```

## Guidelines

1. **Name Meaningfully**: Use domain terminology (`UserId`, not `U64Wrapper`)
2. **Derive Common Traits**: Usually want `Debug`, `Clone`, `Copy` (if underlying is Copy)
3. **Provide Conversions**: Offer `new()`, `into_inner()`, `as_ref()` methods
4. **Document Invariants**: Clearly state what the newtype guarantees
5. **Implement Only Valid Operations**: Don't implement `Add` for IDs just because the underlying type supports it
6. **Consider Serde**: Add `#[serde(transparent)]` if serializing
7. **Use Type Aliases Sparingly**: `type` aliases don't create new types (just synonyms)

## Why This Pattern Works in Rust

Rust's zero-cost abstractions make newtypes practical:

1. **No Runtime Cost**: The wrapper is optimized away entirely
2. **Strong Type System**: No implicit conversions, preventing accidental misuse
3. **Trait System**: Selective trait implementation controls available operations
4. **Repr Transparent**: Guarantees same memory layout as wrapped type
5. **Pattern Matching**: Easy to extract wrapped values when needed

The newtype pattern embodies Rust's principle of **making illegal states unrepresentable** in the type system, catching errors at compile time rather than runtime.

## Common Patterns

### Smart Constructor

```rust
struct PositiveInt(i32);

impl PositiveInt {
    fn new(value: i32) -> Result<Self, &'static str> {
        if value > 0 {
            Ok(PositiveInt(value))
        } else {
            Err("value must be positive")
        }
    }
}
```

### Deref for Convenience

```rust
use std::ops::Deref;

struct MyString(String);

impl Deref for MyString {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

// Now MyString can use all &String methods
let s = MyString(String::from("hello"));
let len = s.len();  // Deref coercion to &String
```

### Multiple Newtypes for Same Underlying Type

```rust
struct Html(String);
struct PlainText(String);
struct Markdown(String);

impl Html {
    fn escape(text: PlainText) -> Html {
        // Escape HTML entities
        Html(text.0.replace("<", "&lt;").replace(">", "&gt;"))
    }
}

// Prevents accidentally rendering unescaped text as HTML
fn render(html: Html) { /* ... */ }
let user_input = PlainText(String::from("<script>"));
// render(user_input);  // ERROR
render(Html::escape(user_input));  // OK
```

This pattern is extensively used in web frameworks to prevent XSS vulnerabilities.
