# Types API Reference

This reference provides complete API documentation for custom type implementations demonstrating Rust's type system features.

## Ascii - ASCII String Type

A validated ASCII-encoded string type that guarantees all bytes are valid ASCII (0x00-0x7f).

### Struct Definition

```rust
pub struct Ascii(Vec<u8>)
```

**Invariant:**
- The inner `Vec<u8>` must contain only bytes in the range `0x00..=0x7f`

**Derived Traits:**
- `Debug`
- `Eq`
- `PartialEq`

### Methods

#### `from_bytes`

Creates an `Ascii` from a byte vector, validating that all bytes are ASCII.

**Signature:**
```rust
pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `bytes` | `Vec<u8>` | Byte vector to validate and convert |

**Returns:**
- `Ok(Ascii)`: Successfully validated ASCII string
- `Err(NotAsciiError)`: Contains non-ASCII bytes (original vector returned in error)

**Complexity:**
- Time: O(n) - validates all bytes
- Space: O(1) - takes ownership of input

**Example:**
```rust
let bytes = b"ASCII and ye shall receive".to_vec();
let ascii = Ascii::from_bytes(bytes).unwrap();
let string = String::from(ascii);
assert_eq!(string, "ASCII and ye shall receive");
```

---

#### `from_bytes_unchecked` (unsafe)

Creates an `Ascii` without validating the bytes.

**Signature:**
```rust
pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Ascii
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `bytes` | `Vec<u8>` | Byte vector assumed to contain only ASCII |

**Returns:**
- `Ascii`: Ascii instance (no validation performed)

**Safety:**
- Caller must ensure `bytes` contains only ASCII characters (bytes ≤ 0x7f)
- Violating this invariant leads to undefined behavior when converting to `String`

**Complexity:**
- Time: O(1) - no validation
- Space: O(1)

**Example:**
```rust
// UNSAFE: Caller guarantees these bytes are ASCII
let ascii = unsafe {
    Ascii::from_bytes_unchecked(b"hello".to_vec())
};
```

---

### Error Types

#### `NotAsciiError`

Error returned when byte vector contains non-ASCII bytes.

**Definition:**
```rust
pub struct NotAsciiError(pub Vec<u8>)
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `0` | `Vec<u8>` | The original byte vector that failed validation |

**Purpose:**
- Allows caller to recover the input vector on validation failure
- Implements `Debug`, `Eq`, `PartialEq`

**Example:**
```rust
let bytes = vec![0xf7, 0xbf, 0xbf, 0xbf];
match Ascii::from_bytes(bytes) {
    Ok(ascii) => { /* ... */ },
    Err(NotAsciiError(original)) => {
        println!("Invalid bytes: {:?}", original);
    }
}
```

---

### Trait Implementations

#### `From<Ascii> for String`

Zero-cost conversion from `Ascii` to `String`.

**Signature:**
```rust
impl From<Ascii> for String {
    fn from(ascii: Ascii) -> String
}
```

**Safety:**
- Uses `String::from_utf8_unchecked` internally
- Safe because well-formed ASCII is also well-formed UTF-8

**Complexity:**
- Time: O(1) - just moves the vector
- Space: O(1) - no allocation or copy

**Example:**
```rust
let ascii = Ascii::from_bytes(b"hello".to_vec())?;
let string: String = ascii.into();
```

---

## Complex&lt;T&gt; - Complex Numbers

Generic complex number type with operator overloading.

### Struct Definition

```rust
#[derive(Clone, Copy, Debug)]
struct Complex<T> {
    /// Real portion of the complex number
    re: T,
    /// Imaginary portion of the complex number
    im: T,
}
```

**Type Parameters:**
| Parameter | Bounds | Description |
|-----------|--------|-------------|
| `T` | Varies by operation | Numeric type (typically `f64`, `i32`, etc.) |

**Derived Traits:**
- `Clone`
- `Copy`
- `Debug`

### Operator Implementations

#### `Add` - Addition

**Signature:**
```rust
impl<T> Add for Complex<T>
where
    T: Add<Output = T>
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self
}
```

**Trait Bounds:**
- `T: Add<Output = T>` - Component type must support addition

**Formula:**
```
(a + bi) + (c + di) = (a + c) + (b + d)i
```

**Example:**
```rust
let a = Complex { re: 1, im: 2 };
let b = Complex { re: 3, im: 4 };
let c = a + b;
assert_eq!(c.re, 4);
assert_eq!(c.im, 6);
```

---

#### `Sub` - Subtraction

**Signature:**
```rust
impl<T> Sub for Complex<T>
where
    T: Sub<Output = T>
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self
}
```

**Trait Bounds:**
- `T: Sub<Output = T>` - Component type must support subtraction

**Formula:**
```
(a + bi) - (c + di) = (a - c) + (b - d)i
```

---

#### `Mul` - Multiplication

**Signature:**
```rust
impl<T> Mul for Complex<T>
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T>
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self
}
```

**Trait Bounds:**
- `T: Clone` - Components need to be cloned
- `T: Add<Output = T>` - For adding products
- `T: Sub<Output = T>` - For subtracting imaginary part
- `T: Mul<Output = T>` - For multiplying components

**Formula:**
```
(a + bi) * (c + di) = (ac - bd) + (ad + bc)i
```

**Example:**
```rust
let a = Complex { re: 5, im: 2 };
let b = Complex { re: 2, im: 5 };
assert_eq!(a * b, Complex { re: 0, im: 29 });
```

---

#### `Neg` - Negation

**Signature:**
```rust
impl<T> Neg for Complex<T>
where
    T: Neg<Output = T>
{
    type Output = Complex<T>;
    fn neg(self) -> Complex<T>
}
```

**Trait Bounds:**
- `T: Neg<Output = T>` - Component type must support negation

**Formula:**
```
-(a + bi) = -a + (-b)i
```

---

#### `AddAssign` - Compound Addition

**Signature:**
```rust
impl<T> AddAssign for Complex<T>
where
    T: AddAssign<T>
{
    fn add_assign(&mut self, rhs: Complex<T>)
}
```

**Trait Bounds:**
- `T: AddAssign<T>` - Component type must support `+=`

**Example:**
```rust
let mut z = Complex { re: 1, im: 2 };
z += Complex { re: 3, im: 4 };
```

---

### Comparison Implementations

#### `PartialEq`

**Signature:**
```rust
impl<T: PartialEq> PartialEq for Complex<T> {
    fn eq(&self, other: &Complex<T>) -> bool
}
```

**Trait Bounds:**
- `T: PartialEq` - Components must be comparable

**Behavior:**
- Two complex numbers are equal if both components are equal

---

#### `Eq`

**Signature:**
```rust
impl<T: Eq> Eq for Complex<T> {}
```

**Trait Bounds:**
- `T: Eq` - Components must have full equivalence

---

### Generic Variations

#### Very Generic Addition

Allows adding complex numbers with different component types:

**Signature:**
```rust
impl<L, R> Add<Complex<R>> for Complex<L>
where
    L: Add<R>
{
    type Output = Complex<L::Output>;
    fn add(self, rhs: Complex<R>) -> Self::Output
}
```

**Example:**
```rust
let a = Complex { re: 1i32, im: 2i32 };
let b = Complex { re: 3.0f64, im: 4.0f64 };
// Can add different numeric types
```

---

### Display Formatting

#### `Display` (for `Complex<f64>`)

**Signature:**
```rust
impl fmt::Display for Complex {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result
}
```

**Format:**
- Standard: `"{re} + {im}i"` or `"{re} - {im}i"`
- Alternate (`{:#}`): Polar form `"{abs} ∠ {angle}°"`

**Example:**
```rust
let c = Complex { re: -0.5, im: 0.866 };
assert_eq!(format!("{}", c), "-0.5 + 0.866i");

let c = Complex { re: 0.0, im: 2.0 };
assert_eq!(format!("{:#}", c), "2 ∠ 90°");
```

---

## Interval&lt;T&gt; - Half-Open Intervals

Represents a half-open interval [lower, upper) with custom partial ordering.

### Struct Definition

```rust
#[derive(Debug, PartialEq)]
struct Interval<T> {
    lower: T,  // inclusive
    upper: T,  // exclusive
}
```

**Type Parameters:**
| Parameter | Bounds | Description |
|-----------|--------|-------------|
| `T` | `PartialOrd` (for ordering) | Type of interval bounds |

**Invariant:**
- Conventionally `lower < upper`, but not enforced

**Derived Traits:**
- `Debug`
- `PartialEq`

### Trait Implementations

#### `PartialOrd` - Partial Ordering

**Signature:**
```rust
impl<T: PartialOrd> PartialOrd<Interval<T>> for Interval<T> {
    fn partial_cmp(&self, other: &Interval<T>) -> Option<Ordering>
}
```

**Trait Bounds:**
- `T: PartialOrd` - Bounds must be comparable

**Ordering Rules:**
| Condition | Result |
|-----------|--------|
| `self == other` | `Some(Ordering::Equal)` |
| `self.lower >= other.upper` | `Some(Ordering::Greater)` |
| `self.upper <= other.lower` | `Some(Ordering::Less)` |
| Intervals overlap | `None` (incomparable) |

**Key Property:**
- Overlapping intervals are **not ordered** with respect to each other
- This is a partial order, not a total order

**Example:**
```rust
assert!(Interval { lower: 10, upper: 20 } < Interval { lower: 20, upper: 40 });
assert!(Interval { lower: 7, upper: 8 } >= Interval { lower: 0, upper: 1 });

// Overlapping intervals aren't ordered
let left = Interval { lower: 10, upper: 30 };
let right = Interval { lower: 20, upper: 40 };
assert!(!(left < right));
assert!(!(left >= right));
```

---

## RefWithFlag&lt;'a, T&gt; - Reference with Boolean Flag

A reference and a boolean flag packed into a single word by stealing the low bit of the pointer.

### Struct Definition

```rust
pub struct RefWithFlag<'a, T> {
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>  // occupies no space
}
```

**Type Parameters:**
| Parameter | Bounds | Description |
|-----------|--------|-------------|
| `'a` | (lifetime) | Lifetime of the borrowed reference |
| `T` | Must have 2+ byte alignment | Type of the referenced value |

**Memory Layout:**
- Size: 1 word (same as `&T`)
- The low bit stores the flag
- Remaining bits store the pointer

**Requirements:**
- `T` must have at least 2-byte alignment
- This ensures the low bit of valid pointers is always 0

### Methods

#### `new`

Creates a new `RefWithFlag` from a reference and boolean.

**Signature:**
```rust
pub fn new(ptr: &'a T, flag: bool) -> RefWithFlag<T>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `ptr` | `&'a T` | Reference to store |
| `flag` | `bool` | Boolean flag to store |

**Returns:**
- `RefWithFlag<'a, T>`: Packed reference and flag

**Panics:**
- If `align_of::<T>() % 2 != 0` (odd alignment)

**Example:**
```rust
let vec = vec![10, 20, 30];
let flagged = RefWithFlag::new(&vec, true);
```

---

#### `get_ref`

Extracts the reference from the packed representation.

**Signature:**
```rust
pub fn get_ref(&self) -> &'a T
```

**Returns:**
- `&'a T`: The stored reference

**Safety:**
- Uses unsafe code to reconstruct pointer
- Safe because construction ensures valid pointer alignment

**Example:**
```rust
let vec = vec![10, 20, 30];
let flagged = RefWithFlag::new(&vec, true);
assert_eq!(flagged.get_ref()[1], 20);
```

---

#### `get_flag`

Extracts the boolean flag from the packed representation.

**Signature:**
```rust
pub fn get_flag(&self) -> bool
```

**Returns:**
- `bool`: The stored flag value

**Example:**
```rust
let vec = vec![10, 20, 30];
let flagged = RefWithFlag::new(&vec, true);
assert_eq!(flagged.get_flag(), true);
```

---

### Safety Considerations

**Unsafe Code:**
- `get_ref` uses unsafe pointer manipulation
- Safe wrapper ensures invariants are maintained

**Requirements:**
- Type `T` must have even alignment (≥2 bytes)
- Checked at runtime in `new`

**Valid Types:**
- Most Rust types (structs, enums, primitives except `u8`/`i8`)
- `Vec<T>`, `String`, `Box<T>`, etc.

**Invalid Types:**
- `u8`, `i8`, `bool` (1-byte alignment)

**Example:**
```rust
// Valid: Vec has pointer-size alignment
let v = vec![1, 2, 3];
let r = RefWithFlag::new(&v, true);

// Invalid: Would panic
// let b: u8 = 42;
// let r = RefWithFlag::new(&b, true);  // Panics!
```

---

## Type Safety Summary

| Type | Key Safety Feature | Validation Strategy |
|------|-------------------|-------------------|
| `Ascii` | Only valid ASCII bytes | Runtime check in constructor |
| `Complex<T>` | Type-safe arithmetic | Compile-time trait bounds |
| `Interval<T>` | Partial ordering semantics | Type system enforces rules |
| `RefWithFlag<'a, T>` | Pointer bit manipulation | Runtime alignment check + unsafe |

## Common Patterns

### Validated Types (Ascii)

```rust
// Parse-don't-validate pattern
pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError>

// Zero-cost extraction
impl From<Ascii> for String

// Unsafe escape hatch with documentation
pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Ascii
```

### Generic Types with Trait Bounds (Complex, Interval)

```rust
// Different bounds for different operations
impl<T: Add<Output = T>> Add for Complex<T>
impl<T: Mul + Add + Sub + Clone> Mul for Complex<T>

// Deriving vs manual implementation
#[derive(Clone, Copy, Debug)]  // Automatic
impl<T: PartialEq> PartialEq for Complex<T>  // Manual
```

### Unsafe Types with Safe Wrappers (RefWithFlag)

```rust
// Public safe constructor with runtime checks
pub fn new(ptr: &'a T, flag: bool) -> RefWithFlag<T>

// Public safe accessors wrapping unsafe code
pub fn get_ref(&self) -> &'a T  // Uses unsafe internally

// PhantomData for proper variance and drop check
behaves_like: PhantomData<&'a T>
```
