# Partial Ordering

## Pattern Name
**Partial Ordering for Domain Comparisons**

## Context

You are defining a type that represents values that can sometimes be compared, but where not all pairs of values have a defined ordering relationship. Examples include:
- Intervals that may overlap (neither less than nor greater than each other)
- Floating-point numbers (with NaN values that can't be compared)
- Sets (where overlap means incomparability)
- Complex numbers (no standard ordering for complex numbers)
- Partial orders in mathematics or type theory

Rust distinguishes between `PartialOrd` (partial ordering) and `Ord` (total ordering), allowing you to model these domain-specific comparison semantics accurately.

## Problem

**How do you implement comparison operations for types where some values are incomparable, while still enabling users to use standard comparison operators (`<`, `>`, `<=`, `>=`)?**

You need to:
- Express that some pairs of values have no ordering relationship
- Use standard Rust comparison operators where comparisons are defined
- Integrate with Rust's type system and generic algorithms
- Avoid runtime panics when comparing incomparable values
- Maintain semantic correctness for domain-specific comparison rules

## Forces

- **Mathematical Correctness vs API Convenience**: Partial orders are mathematically precise, but `Option<Ordering>` is less convenient than `bool`
- **Total vs Partial**: Most developers expect all values to be comparable (total order), but some domains require partial orders
- **Generic Algorithms**: Many algorithms assume total ordering; partial orders may not work with all standard library functions
- **Surprising Behavior**: `!(a < b)` doesn't imply `a >= b` in partial orders (both could be false)
- **Error Handling**: No obvious way to handle "incomparable" results in boolean comparison operators
- **Consistency**: Must maintain consistency between `partial_cmp`, `lt`, `le`, `gt`, `ge` methods

## Solution

**Implement the `PartialOrd` trait for your type, with `partial_cmp` returning `Option<Ordering>` where `None` indicates incomparable values.**

### Core Implementation

From `/home/user/rust-programming-examples/interval/src/lib.rs`:

```rust
use std::cmp::{Ordering, PartialOrd};

#[derive(Debug, PartialEq)]
struct Interval<T> {
    lower: T, // inclusive
    upper: T, // exclusive
}

impl<T: PartialOrd> PartialOrd<Interval<T>> for Interval<T> {
    fn partial_cmp(&self, other: &Interval<T>) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.lower >= other.upper {
            // self is completely after other
            Some(Ordering::Greater)
        } else if self.upper <= other.lower {
            // self is completely before other
            Some(Ordering::Less)
        } else {
            // Intervals overlap - incomparable
            None
        }
    }
}
```

### Understanding the Semantics

The interval ordering defines:
- `[10, 20) < [20, 40)` ✓ (first ends before second starts)
- `[7, 8) > [0, 1)` ✓ (first starts after second ends)
- `[7, 8) == [7, 8)` ✓ (same interval)
- `[10, 30)` vs `[20, 40)` → **None** (overlapping, incomparable)

### Using Partial Orderings

```rust
#[test]
fn test_interval_comparisons() {
    // Non-overlapping intervals can be compared
    assert!(Interval { lower: 10, upper: 20 } <  Interval { lower: 20, upper: 40 });
    assert!(Interval { lower: 7,  upper: 8  } >= Interval { lower: 0,  upper: 1  });
    assert!(Interval { lower: 7,  upper: 8  } <= Interval { lower: 7,  upper: 8  });

    // Overlapping intervals are incomparable
    let left  = Interval { lower: 10, upper: 30 };
    let right = Interval { lower: 20, upper: 40 };

    // Both comparisons return false!
    assert!(!(left < right));
    assert!(!(left >= right));
}
```

### Key Insight: Comparison Operators with Partial Orders

When `partial_cmp` returns `None`, the comparison operators behave as follows:
- `<`, `<=`, `>`, `>=` all return **false**
- This means `!(a < b)` does NOT imply `a >= b`
- You cannot assume trichotomy (exactly one of `<`, `==`, `>` is true)

### Checking for Comparability

```rust
fn are_comparable<T: PartialOrd>(a: &T, b: &T) -> bool {
    a.partial_cmp(b).is_some()
}

fn compare_intervals<T: PartialOrd>(a: &Interval<T>, b: &Interval<T>) -> String {
    match a.partial_cmp(b) {
        Some(Ordering::Less) => format!("{:?} < {:?}", a, b),
        Some(Ordering::Equal) => format!("{:?} == {:?}", a, b),
        Some(Ordering::Greater) => format!("{:?} > {:?}", a, b),
        None => format!("{:?} and {:?} are incomparable", a, b),
    }
}
```

### PartialOrd Trait Definition

For reference, here's what you're implementing:

```rust
pub trait PartialOrd<Rhs = Self>: PartialEq<Rhs> {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;

    // Default implementations based on partial_cmp
    fn lt(&self, other: &Rhs) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Less))
    }
    fn le(&self, other: &Rhs) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Less | Ordering::Equal))
    }
    fn gt(&self, other: &Rhs) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Greater))
    }
    fn ge(&self, other: &Rhs) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Greater | Ordering::Equal))
    }
}
```

You typically only implement `partial_cmp`; the comparison operators are automatically derived.

## Resulting Context

### Benefits

- **Mathematical Precision**: Accurately models domains where not all values are comparable
- **Type Safety**: Incomparability is represented in the type system (via `Option`)
- **Standard Operators**: Users can still use `<`, `>`, `<=`, `>=` where defined
- **Explicit Semantics**: `None` makes incomparability explicit rather than panicking
- **Generic Integration**: Works with generic code that accepts `PartialOrd` bounds

### Liabilities

- **Unexpected Behavior**: `!(a < b)` doesn't mean `a >= b`, which surprises developers expecting total orders
- **Limited Algorithm Support**: Sorting and binary search expect total orders
- **Boolean Confusion**: Comparison operators return `false` for incomparable values, hiding information
- **No Panic**: Unlike `Ord`, there's no way to signal an error when comparing incomparable values
- **Testing Complexity**: Must test incomparable cases explicitly

### Comparison with Ord (Total Ordering)

**When to use `Ord`** (total ordering):
- Every pair of values has a defined ordering relationship
- Examples: integers, strings, dates/times
- Required for sorting, binary search, BTreeMap keys

**When to use `PartialOrd`** (partial ordering):
- Some pairs of values are incomparable
- Examples: floating-point (NaN), intervals, sets, complex numbers
- Comparison operators may return `false` for both directions

```rust
// Total order (Ord)
fn sort_list<T: Ord>(items: &mut [T]) {
    items.sort();  // Always works
}

// Partial order (PartialOrd)
fn try_sort<T: PartialOrd>(items: &mut [T]) {
    items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    // Incomparable items treated as equal (unstable sort!)
}
```

## Related Patterns

- **PartialEq and Eq**: Equality is also split into partial (with `NaN`) and total
- **Newtype Wrapper**: Can wrap types to provide different ordering semantics
- **Operator Overloading**: Comparison operators are overloadable traits
- **Option for Absence**: `Option<Ordering>` represents "no ordering exists"

## Known Uses

- **f32/f64**: Standard library floating-point types implement `PartialOrd` but not `Ord` (because of NaN)
- **Intervals**: As shown in the example, intervals have a natural partial order
- **Sets**: Set inclusion forms a partial order (overlapping sets are incomparable)
- **Option<T>**: Implements `PartialOrd` when `T: PartialOrd`
- **Vectors**: Lexicographic ordering with partial comparison of elements

## Example: Set Inclusion as Partial Order

Another domain where partial ordering is natural:

```rust
use std::collections::HashSet;
use std::cmp::{Ordering, PartialOrd};

#[derive(Debug, PartialEq)]
struct Set<T>(HashSet<T>);

impl<T: Eq + std::hash::Hash> PartialOrd for Set<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let is_subset = self.0.is_subset(&other.0);
        let is_superset = self.0.is_superset(&other.0);

        match (is_subset, is_superset) {
            (true, true) => Some(Ordering::Equal),     // Same sets
            (true, false) => Some(Ordering::Less),      // Proper subset
            (false, true) => Some(Ordering::Greater),   // Proper superset
            (false, false) => None,                     // Incomparable
        }
    }
}

#[test]
fn test_set_ordering() {
    let small = Set(HashSet::from([1, 2]));
    let large = Set(HashSet::from([1, 2, 3]));
    let different = Set(HashSet::from([2, 3, 4]));

    assert!(small < large);           // {1,2} ⊂ {1,2,3}
    assert!(!(small < different));    // {1,2} ⊄ {2,3,4}
    assert!(!(small >= different));   // Neither subset nor superset
}
```

## Guidelines

1. **Document Incomparability**: Clearly explain when values are incomparable
2. **Test All Cases**: Test less-than, equal, greater-than, AND incomparable cases
3. **Be Consistent**: Ensure `partial_cmp` agrees with `PartialEq`
4. **Consider Ord**: If all values ARE comparable, implement `Ord` instead
5. **Handle None**: In generic code, decide how to handle `None` results
6. **Avoid Panicking**: Don't call `unwrap()` on `partial_cmp` results without checking

## Common Mistakes

### Mistake 1: Assuming Trichotomy

```rust
// ❌ WRONG: Assumes one of these is always true
if a < b {
    println!("a is less");
} else if a > b {
    println!("a is greater");
} else {
    println!("a equals b");  // WRONG: Could be incomparable!
}

// ✅ RIGHT: Check explicitly
match a.partial_cmp(&b) {
    Some(Ordering::Less) => println!("a is less"),
    Some(Ordering::Greater) => println!("a is greater"),
    Some(Ordering::Equal) => println!("a equals b"),
    None => println!("a and b are incomparable"),
}
```

### Mistake 2: Using Partial Orders with Sorting

```rust
let mut intervals = vec![
    Interval { lower: 10, upper: 30 },
    Interval { lower: 20, upper: 40 },
];

// ❌ WRONG: Panics if partial_cmp returns None
// intervals.sort();

// ✅ OPTION 1: Handle None explicitly
intervals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

// ✅ OPTION 2: Define a total order wrapper
struct TotalInterval(Interval<i32>);
impl Ord for TotalInterval {
    fn cmp(&self, other: &Self) -> Ordering {
        // Define how to handle overlapping intervals
        self.0.lower.cmp(&other.0.lower)
    }
}
```

### Mistake 3: Inconsistent with PartialEq

```rust
// ❌ WRONG: partial_cmp says Equal, but PartialEq says not equal
impl<T: PartialOrd> PartialOrd for Wrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)  // Always equal
    }
}
impl<T> PartialEq for Wrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        false  // Never equal - INCONSISTENT!
    }
}

// ✅ RIGHT: Maintain consistency
impl<T: PartialOrd> PartialOrd for Wrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else {
            // ... other comparisons
        }
    }
}
```

## Why This Pattern Works in Rust

Rust's approach to ordering is more principled than many languages:

1. **Separation of Concerns**: `PartialOrd` vs `Ord` separates partial from total orders
2. **Option Type**: `Option<Ordering>` explicitly represents "no ordering exists"
3. **Type Safety**: Generic algorithms can require `Ord` when they need total ordering
4. **Mathematical Rigor**: Models mathematical partial orders accurately
5. **No Implicit Behavior**: Incomparability is explicit, not hidden

This pattern demonstrates Rust's philosophy of **encoding semantic properties in the type system**, allowing the compiler to prevent misuse of partial orders in contexts requiring total orders.

## Advanced: Floating-Point Considerations

Floating-point numbers are the most common example of partial ordering:

```rust
let nan = f64::NAN;
let zero = 0.0;

// All comparisons with NaN return false
assert!(!(nan < zero));
assert!(!(nan > zero));
assert!(!(nan == zero));
assert!(!(nan >= zero));

// This is why f64 implements PartialOrd but not Ord
fn example<T: PartialOrd>(a: T, b: T) { /* ... */ }  // OK for f64
// fn example<T: Ord>(a: T, b: T) { /* ... */ }      // ERROR: f64 not Ord
```

For algorithms that need total ordering on floats, use wrapper types:

```rust
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct TotalF64(f64);

impl Eq for TotalF64 {}

impl Ord for TotalF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .unwrap_or_else(|| {
                // Define NaN ordering (e.g., NaN > all numbers)
                match (self.0.is_nan(), other.0.is_nan()) {
                    (true, true) => Ordering::Equal,
                    (true, false) => Ordering::Greater,
                    (false, true) => Ordering::Less,
                    _ => unreachable!(),
                }
            })
    }
}

// Now can sort Vec<TotalF64>
let mut floats = vec![TotalF64(1.0), TotalF64(f64::NAN), TotalF64(0.5)];
floats.sort();  // OK: TotalF64 implements Ord
```

This pattern allows you to use partial orders where mathematically correct, while still providing escape hatches for algorithms that require total ordering.
