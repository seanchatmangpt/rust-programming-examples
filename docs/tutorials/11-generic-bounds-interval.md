# Tutorial: Advanced Generics with the Interval Type

## Introduction

In this tutorial, you'll learn advanced generic programming techniques in Rust by implementing an `Interval` type that represents numeric ranges. You'll discover how trait bounds work, how to implement comparison traits, and how Rust's type system enables partial ordering - a concept that doesn't exist in most programming languages.

### What You'll Learn

- Creating generic types with meaningful constraints
- Understanding and implementing `PartialOrd` and `Ordering`
- The difference between partial and total ordering
- Working with inclusive/exclusive ranges
- Advanced trait bounds and `where` clauses
- Why some values can't be ordered

### Prerequisites

- Understanding of Rust generics basics
- Familiarity with traits and trait implementations
- Basic knowledge of comparison operations

## Understanding Intervals

An **interval** represents a range of values from a lower bound to an upper bound:

- `[10, 20)` means "all values from 10 (inclusive) to 20 (exclusive)"
- `[0, 100)` represents 0, 1, 2, ..., 99

**Real-world uses:**
- Time ranges: "meetings from 2pm to 3pm"
- Version ranges: "compatible with versions 1.5 to 2.0"
- Number ranges: "valid ages from 0 to 150"

## Step 1: Define the Interval Type

Let's create a generic `Interval` type:

```rust
#[derive(Debug, PartialEq)]
struct Interval<T> {
    lower: T,  // inclusive
    upper: T,  // exclusive
}
```

**Key Points:**

- **Generic `T`**: Works with any type (integers, floats, dates, etc.)
- **Half-open interval**: `[lower, upper)` includes `lower`, excludes `upper`
- **Derived traits**: `Debug` for printing, `PartialEq` for equality checking

**Python Comparison:**

Python's `range` is similar but limited to integers:

```python
# Python range is always half-open
r = range(10, 20)  # 10 to 19
```

Rust's `Interval` can work with any comparable type!

## Step 2: Understanding Ordering

Before implementing comparisons, we need to understand **partial ordering**.

### Total Ordering

In a **total ordering**, every pair of values can be compared:

- For integers: `5 < 10`, `10 > 5`, `5 == 5`
- Every pair has a relationship: less than, greater than, or equal

### Partial Ordering

In a **partial ordering**, some pairs can't be compared:

- Intervals `[10, 30)` and `[20, 40)` **overlap** - which is "greater"?
- They're not equal, not clearly less than, not clearly greater than
- The comparison is **undefined** for overlapping intervals

**Python Comparison:**

Python doesn't have built-in partial ordering. Everything either compares or raises an exception:

```python
# Python floats
float('nan') < 5  # Returns False (surprising!)
float('nan') > 5  # Also False
float('nan') == float('nan')  # False!
```

Rust handles this more explicitly with `Option<Ordering>`.

## Step 3: Defining Interval Ordering

Let's define when intervals are ordered:

1. **Equal**: Same lower and upper bounds
2. **Less than**: One interval completely before another
3. **Greater than**: One interval completely after another
4. **Unordered**: Intervals overlap

**Examples:**

```
[10, 20) < [20, 30)  ✓ (first entirely before second)
[10, 20) < [15, 30)  ✗ (they overlap - unordered!)
[10, 20) < [10, 20)  ✗ (they're equal)
```

**Visual representation:**

```
[10, 20) < [20, 30)
|-----|
        |-----|    ✓ Less than (no overlap)

[10, 20) ? [15, 30)
|-----|
    |-----|        ✗ Unordered (overlap)
```

## Step 4: Implement PartialOrd

Now let's implement the `PartialOrd` trait:

```rust
use std::cmp::{Ordering, PartialOrd};

impl<T: PartialOrd> PartialOrd<Interval<T>> for Interval<T> {
    fn partial_cmp(&self, other: &Interval<T>) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.lower >= other.upper {
            Some(Ordering::Greater)
        } else if self.upper <= other.lower {
            Some(Ordering::Less)
        } else {
            None  // Intervals overlap - can't order them
        }
    }
}
```

**Understanding the Code:**

1. **Trait Bound**: `T: PartialOrd` - we need to compare `T` values
2. **Return Type**: `Option<Ordering>` - `Some` if orderable, `None` if not
3. **Logic**:
   - If equal → `Some(Ordering::Equal)`
   - If `self` completely after `other` → `Some(Ordering::Greater)`
   - If `self` completely before `other` → `Some(Ordering::Less)`
   - If overlapping → `None` (unordered)

**Key Insight:**

The return type `Option<Ordering>` explicitly represents "maybe comparable":
- `Some(Ordering::Less)` means definitely less than
- `None` means "can't determine ordering"

## Step 5: Using the Interval Type

Let's see it in action:

```rust
fn main() {
    let a = Interval { lower: 10, upper: 20 };
    let b = Interval { lower: 20, upper: 40 };

    // These intervals don't overlap, so they're ordered
    println!("{}", a < b);   // true
    println!("{}", a > b);   // false
    println!("{}", a <= b);  // true
}
```

**Non-overlapping intervals:**

```rust
let morning = Interval { lower: 8, upper: 12 };    // 8am to 12pm
let afternoon = Interval { lower: 12, upper: 17 }; // 12pm to 5pm

assert!(morning < afternoon);  // ✓ Morning comes before afternoon
```

**Overlapping intervals:**

```rust
let early = Interval { lower: 10, upper: 30 };
let late = Interval { lower: 20, upper: 40 };

// These overlap, so comparisons return false
assert!(!(early < late));   // NOT less than
assert!(!(early > late));   // NOT greater than
assert!(!(early >= late));  // NOT greater-or-equal
assert!(!(early <= late));  // NOT less-or-equal
```

**Important:** When intervals overlap, **all comparison operators return false** (except `==` which also returns false). They're simply unordered.

## Step 6: Understanding the Comparison Operators

The `PartialOrd` trait provides these operators:

```rust
let a = Interval { lower: 10, upper: 20 };
let b = Interval { lower: 20, upper: 40 };
let c = Interval { lower: 15, upper: 30 };

// Non-overlapping: fully ordered
a < b   // true (a entirely before b)
b > a   // true (b entirely after a)
a <= a  // true (equal intervals)

// Overlapping: unordered
a < c   // false
a > c   // false
a <= c  // false
a >= c  // false
```

**Python Comparison:**

Python doesn't have partial ordering built-in:

```python
class Interval:
    def __init__(self, lower, upper):
        self.lower = lower
        self.upper = upper

    def __lt__(self, other):
        if self.upper <= other.lower:
            return True
        elif self.lower >= other.upper:
            return False
        else:
            # Overlap - what to do?
            raise ValueError("Intervals overlap - can't compare")
```

Rust's approach is cleaner: unordered comparisons just return `false`, and you can check explicitly with `partial_cmp()`.

## Step 7: Explicitly Checking Ordering

You can use `partial_cmp()` directly to handle all cases:

```rust
let a = Interval { lower: 10, upper: 20 };
let b = Interval { lower: 15, upper: 30 };

match a.partial_cmp(&b) {
    Some(Ordering::Less) => println!("a is less than b"),
    Some(Ordering::Greater) => println!("a is greater than b"),
    Some(Ordering::Equal) => println!("a equals b"),
    None => println!("a and b are unordered (they overlap)"),
}
```

This is more explicit and handles all cases including the unordered case.

## Step 8: Adding Useful Methods

Let's add methods to work with intervals:

```rust
impl<T: PartialOrd> Interval<T> {
    /// Check if a value is in the interval
    pub fn contains(&self, value: &T) -> bool {
        &self.lower <= value && value < &self.upper
    }

    /// Check if two intervals overlap
    pub fn overlaps(&self, other: &Interval<T>) -> bool {
        self.lower < other.upper && other.lower < self.upper
    }

    /// Check if this interval is empty
    pub fn is_empty(&self) -> bool
    where
        T: PartialOrd,
    {
        self.lower >= self.upper
    }
}
```

**Using these methods:**

```rust
let interval = Interval { lower: 10, upper: 20 };

assert!(interval.contains(&15));    // 15 is in [10, 20)
assert!(!interval.contains(&20));   // 20 is excluded
assert!(!interval.contains(&5));    // 5 is before the interval

let other = Interval { lower: 15, upper: 25 };
assert!(interval.overlaps(&other)); // [10,20) and [15,25) overlap
```

**Python Comparison:**

```python
class Interval:
    def contains(self, value):
        return self.lower <= value < self.upper

    def overlaps(self, other):
        return self.lower < other.upper and other.lower < self.upper
```

Similar methods, but Rust's trait bounds ensure type safety.

## Step 9: Generic Bounds in Practice

Let's see how trait bounds work with different types:

```rust
// Integer intervals
let int_interval = Interval { lower: 1, upper: 10 };
assert!(int_interval.contains(&5));

// Float intervals
let float_interval = Interval { lower: 0.0, upper: 1.0 };
assert!(float_interval.contains(&0.5));

// Character intervals (lexicographic order)
let char_interval = Interval { lower: 'a', upper: 'z' };
assert!(char_interval.contains(&'m'));
```

**The power of generics:** One implementation works for all comparable types!

**What happens with NaN?**

```rust
let weird = Interval { lower: 0.0, upper: f64::NAN };
let value = 5.0;

// This might behave unexpectedly because NaN comparisons are tricky
// NaN is not less than, greater than, or equal to anything (including itself)
println!("{}", weird.contains(&value));  // May be false
```

This is why we use `PartialOrd` instead of `Ord` - floats don't have total ordering because of NaN.

## Step 10: Total vs Partial Ordering

**Ord (Total Ordering):**
- Every pair of values can be compared
- Types: `i32`, `String`, `char`
- Required methods: `fn cmp(&self, other: &Self) -> Ordering`

**PartialOrd (Partial Ordering):**
- Some pairs might not be comparable
- Types: `f64` (due to NaN), our `Interval`
- Required methods: `fn partial_cmp(&self, other: &Self) -> Option<Ordering>`

**When to use each:**

```rust
// Ord: All integers can be compared
fn find_max_int(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

// PartialOrd: Floats need careful handling
fn find_max_float(a: f64, b: f64) -> Option<f64> {
    match a.partial_cmp(&b) {
        Some(Ordering::Greater) => Some(a),
        Some(_) => Some(b),
        None => None,  // One is NaN
    }
}
```

## Complete Working Example

Let's build a program that works with time intervals:

```rust
use std::cmp::{Ordering, PartialOrd};

#[derive(Debug, PartialEq)]
struct Interval<T> {
    lower: T,
    upper: T,
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
            None
        }
    }
}

impl<T: PartialOrd> Interval<T> {
    fn overlaps(&self, other: &Interval<T>) -> bool {
        self.lower < other.upper && other.lower < self.upper
    }
}

fn main() {
    // Meeting times (in 24-hour format)
    let meeting1 = Interval { lower: 9, upper: 11 };   // 9am-11am
    let meeting2 = Interval { lower: 11, upper: 13 };  // 11am-1pm
    let meeting3 = Interval { lower: 10, upper: 12 };  // 10am-12pm

    // Check if meetings conflict
    println!("Meeting 1 and 2 conflict: {}", meeting1.overlaps(&meeting2));
    println!("Meeting 1 and 3 conflict: {}", meeting1.overlaps(&meeting3));

    // Try ordering
    if meeting1 < meeting2 {
        println!("Meeting 1 comes before meeting 2");
    }

    if meeting1.partial_cmp(&meeting3).is_none() {
        println!("Meeting 1 and 3 overlap - can't order them");
    }
}
```

**Output:**
```
Meeting 1 and 2 conflict: false
Meeting 1 and 3 conflict: true
Meeting 1 comes before meeting 2
Meeting 1 and 3 overlap - can't order them
```

## Key Takeaways

### Partial Ordering Concepts

1. **Not everything can be compared**: Some values are fundamentally unorderable
2. **Option<Ordering>**: Explicitly represents "maybe comparable"
3. **PartialOrd vs Ord**: Partial is more general, Ord is more restrictive
4. **Type safety**: Rust prevents invalid comparisons at compile time

### Compared to Python

| Aspect | Rust | Python |
|--------|------|--------|
| Partial ordering | Built-in with `PartialOrd` | Not natively supported |
| Unorderable values | Return `None` or `false` | Raise exception or return `False` |
| Type safety | Compile-time trait bounds | Runtime duck typing |
| NaN handling | Explicit with `partial_cmp` | Surprising behavior |
| Generic constraints | `T: PartialOrd` | No constraints (duck typing) |

### Advanced Generics Patterns

1. **Trait bounds**: `<T: PartialOrd>` constrains generic types
2. **Where clauses**: Alternative syntax for complex bounds
3. **Associated types**: `Option<Ordering>` as return type
4. **Derived traits**: Automatic implementations
5. **Conditional methods**: `where T: PartialOrd` on specific methods

## Exercises

### Exercise 1: Implement Intersection

Add a method to find the intersection of two intervals:

```rust
impl<T: PartialOrd + Clone> Interval<T> {
    pub fn intersection(&self, other: &Interval<T>) -> Option<Interval<T>> {
        // Return Some(interval) if they overlap, None otherwise
        // The intersection is the overlapping region
    }
}
```

**Hint:** The intersection's lower bound is the max of the two lower bounds, and the upper bound is the min of the two upper bounds.

### Exercise 2: Implement Union

For non-overlapping intervals, create a method that returns an error:

```rust
pub fn union(&self, other: &Interval<T>) -> Result<Interval<T>, &str> {
    // If intervals are adjacent or overlapping, return their union
    // Otherwise, return an error
}
```

### Exercise 3: Add Length Method

For intervals where subtraction is defined:

```rust
impl<T: Sub<Output = T> + Clone> Interval<T> {
    pub fn length(&self) -> T {
        // Return upper - lower
    }
}
```

### Exercise 4: Implement Iterator

Make intervals iterable for integer types:

```rust
impl Iterator for Interval<i32> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        // Yield values from lower to upper-1
    }
}
```

Usage: `for x in Interval { lower: 0, upper: 10 } { println!("{}", x); }`

### Exercise 5: Add Validation

Create a constructor that ensures `lower < upper`:

```rust
impl<T: PartialOrd> Interval<T> {
    pub fn new(lower: T, upper: T) -> Result<Interval<T>, &'static str> {
        if lower >= upper {
            Err("Lower bound must be less than upper bound")
        } else {
            Ok(Interval { lower, upper })
        }
    }
}
```

## Advanced Topics

### Working with Different Bound Types

You could make the interval's bounds generic:

```rust
struct Interval<T, L, U> {
    lower: L,
    upper: U,
    _phantom: PhantomData<T>,
}
```

This allows mixed bounds like `Interval<i32, i32, f64>`.

### Infinite Intervals

Add support for unbounded intervals:

```rust
enum Bound<T> {
    Unbounded,
    Included(T),
    Excluded(T),
}

struct Interval<T> {
    lower: Bound<T>,
    upper: Bound<T>,
}
```

### Custom Ordering Logic

You could define different ordering semantics:

```rust
// Order by interval length instead of position
impl<T: Sub<Output = U>, U: PartialOrd> PartialOrd for Interval<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.length().partial_cmp(&other.length())
    }
}
```

## Real-World Applications

1. **Date ranges**: Schedule management, booking systems
2. **Version constraints**: Cargo dependency ranges
3. **Numeric ranges**: Input validation, bounded values
4. **Time intervals**: Duration tracking, timeline management
5. **IP ranges**: Network address management

## Next Steps

- Study the `std::ops::Range` type in the standard library
- Learn about the `Bound` type in `std::collections`
- Explore interval tree data structures
- Investigate the `chrono` crate for date/time intervals
- Read about lattice theory and partial orders

## Further Reading

- [std::cmp::PartialOrd Documentation](https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html)
- [The Rust Book - Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [std::ops::Range](https://doc.rust-lang.org/std/ops/struct.Range.html)
- [Partial Order on Wikipedia](https://en.wikipedia.org/wiki/Partially_ordered_set)

## Reference: Complete Code

The complete implementation can be found at:
`/home/user/rust-programming-examples/interval/src/lib.rs`

Run the tests with:
```bash
cd /home/user/rust-programming-examples/interval
cargo test
```
