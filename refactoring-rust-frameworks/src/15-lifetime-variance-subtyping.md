# Lifetime Variance & Subtyping Advanced Semantics

## Introduction: Why Variance Matters

Variance is the cornerstone of Rust's type system soundness. It determines when one type can be substituted for another, particularly around lifetimes and generic parameters. Misunderstanding variance leads to memory unsafety, use-after-free bugs, and API designs that compile but violate invariants. This chapter provides AI agents with a deep understanding of variance semantics to write sound Rust code.

**Core Principle**: Variance rules exist to prevent you from storing short-lived references in long-lived containers, which would create dangling pointers.

---

## 1. Variance Fundamentals

### What Variance Means

Variance describes how subtyping relationships are preserved through type constructors. Given types `T` and `U` where `T` is a subtype of `U` (written `T <: U`), variance determines whether `F<T> <: F<U>` for some type constructor `F`.

**Three Variance Categories**:

```
Covariant:     F<T> <: F<U>  when T <: U  (subtyping preserved)
Contravariant: F<U> <: F<T>  when T <: U  (subtyping reversed)
Invariant:     No relationship between F<T> and F<U>
```

### Lifetime Subtyping

In Rust, `'a: 'b` means "`'a` outlives `'b`", making `'a` a subtype of `'b`:

```
'static: 'a  for any 'a (static lives forever)
'a: 'b       when 'a encompasses 'b (longer-lived)
```

**ASCII Diagram: Lifetime Relationships**
```
'static ─────────────────────────────────────────────►
'a      ────────────────────────►
'b      ──────────►
'c      ───►

'static: 'a: 'b: 'c
(each is a subtype of lifetimes to its right)
```

### Core Variance Rules

```rust
&'a T        // Covariant in both 'a and T
&'a mut T    // Covariant in 'a, INVARIANT in T
*const T     // Covariant in T
*mut T       // INVARIANT in T
fn(T) -> U   // Contravariant in T, covariant in U
Cell<T>      // INVARIANT in T
PhantomData<T>  // Covariant in T
PhantomData<fn(T)>  // Contravariant in T (function argument trick)
```

**Why Invariance Exists**: Mutable references must be invariant to prevent soundness holes:

```rust
// Hypothetical unsound code if &mut T were covariant in T
fn evil<'a>(input: &mut &'static str, shorter: &'a str) {
    // If &mut T were covariant, we could substitute:
    // &mut &'static str  ~>  &mut &'a str
    *input = shorter; // UNSOUND: storing short-lived ref in long-lived slot
}

fn main() {
    let mut s: &'static str = "static";
    {
        let temp = String::from("temporary");
        evil(&mut s, &temp);
        // temp dropped here
    }
    println!("{}", s); // USE-AFTER-FREE: s points to freed memory!
}
```

The compiler rejects this because `&mut T` is **invariant** in `T`.

### PhantomData and Variance Declaration

`PhantomData<T>` allows you to declare variance for types that don't directly use `T`:

```rust
use std::marker::PhantomData;

struct Iter<'a, T> {
    ptr: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>, // Declares covariance in 'a and T
}
```

**Variance Table for PhantomData**:
```
PhantomData<T>         // Covariant in T
PhantomData<&T>        // Covariant in T
PhantomData<&mut T>    // Invariant in T
PhantomData<*const T>  // Covariant in T
PhantomData<*mut T>    // Invariant in T
PhantomData<fn(T)>     // Contravariant in T (function argument position)
PhantomData<fn() -> T> // Covariant in T (return position)
PhantomData<Cell<T>>   // Invariant in T
```

### When Variance Violations Cause Soundness Bugs

**Pattern 1: Storing in Containers**
```rust
// UNSOUND if Vec<T> were invariant (it's covariant):
fn upcast_lifetime<'a, 'b>(v: Vec<&'a str>) -> Vec<&'b str>
where
    'a: 'b,
{
    v // OK: Vec<&'a str> <: Vec<&'b str> via covariance
}
```

**Pattern 2: Interior Mutability**
```rust
use std::cell::Cell;

// Cell<T> is INVARIANT in T for safety:
fn cannot_upcast<'a, 'b>(c: Cell<&'a str>) -> Cell<&'b str>
where
    'a: 'b,
{
    c // ERROR: cannot convert Cell<&'a str> to Cell<&'b str>
}
```

---

## 2. Lifetime Variance in Detail

### Why &'a T is Covariant in 'a

Immutable references can safely shorten lifetimes:

```rust
fn shorten<'a, 'b>(r: &'a str) -> &'b str
where
    'a: 'b, // 'a outlives 'b
{
    r // OK: &'a str <: &'b str (covariant)
}

let s = String::from("hello");
let r1: &'static str = "world";
let r2: &str = shorten(r1); // Shortens 'static to local lifetime
```

**ASCII Diagram: Lifetime Shortening**
```
&'a T                &'b T
│                    │
├─────────►          ├────►
'a lifetime          'b lifetime

Covariance allows:  &'a T  →  &'b T  when 'a: 'b
```

### Why &'a mut T is Invariant in 'a

Mutable references cannot safely change lifetimes:

```rust
// REJECTED: Cannot shorten mutable reference lifetime arbitrarily
fn broken_shorten<'a, 'b>(r: &'a mut i32) -> &'b mut i32
where
    'a: 'b,
{
    r // ERROR: cannot infer appropriate lifetime
}
```

**Soundness Reason**: If you could shorten, you could create aliasing:

```rust
// Hypothetical unsound code:
fn create_alias<'a>(outer: &'a mut i32) {
    let inner: &mut i32 = outer; // Shortened lifetime
    // Now both outer and inner are live - ALIASING!
}
```

### Practical Implications for API Design

**Good API**: Accept immutable references (covariant, flexible):
```rust
fn log_message(msg: &str) {
    println!("{}", msg);
}

// Can call with any lifetime:
log_message("static");
let temp = String::from("temporary");
log_message(&temp);
```

**Restrictive API**: Mutable references (invariant, inflexible):
```rust
fn modify_message(msg: &mut String) {
    msg.push_str("!");
}

// Lifetime must match exactly (no coercion):
let mut s = String::from("hello");
modify_message(&mut s); // Exact match required
```

### Variance in Return Positions vs Argument Positions

```rust
// Function types: fn(T) -> U
// - Contravariant in T (arguments)
// - Covariant in U (return)

type Handler<'a> = fn(&'a str) -> &'a str;

// Contravariant in argument: can pass function expecting longer lifetime
fn needs_short_lived(h: Handler<'_>) { /* ... */ }

fn handler_long(s: &'static str) -> &'static str { s }
needs_short_lived(handler_long); // OK: contravariance

// Covariant in return: can return shorter lifetime
fn returns_handler<'a>() -> Handler<'a> {
    handler_long // OK: covariance in return
}
```

### HRTBs and Variance Interaction

Higher-Ranked Trait Bounds (`for<'a>`) quantify over all possible lifetimes:

```rust
// HRTB: Works for ANY lifetime 'a
fn apply<F>(f: F, s: &str) -> &str
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    f(s)
}

// Non-HRTB: Works for SPECIFIC lifetime
fn apply_specific<'a, F>(f: F, s: &'a str) -> &'a str
where
    F: Fn(&'a str) -> &'a str,
{
    f(s)
}
```

**HRTB Variance Interaction**:
```rust
// HRTB with contravariance:
trait Transform {
    fn transform<'a>(&self, input: &'a str) -> &'a str;
}

// Implementer can use longer lifetimes in implementation:
impl Transform for Identity {
    fn transform<'a>(&self, input: &'a str) -> &'a str {
        input // Exact match
    }
}
```

---

## 3. Type Variance

### Covariance of Immutable References

```rust
// Covariant: Can pass subtype where supertype expected
fn accept_animal(a: &Animal) { /* ... */ }

struct Dog;
impl Animal for Dog { /* ... */ }

let dog = Dog;
accept_animal(&dog); // OK if Dog <: Animal (not typical Rust, but concept)

// More realistically with lifetimes:
fn accept_static(s: &'static str) { /* ... */ }
fn provide<'a>(s: &'a str) {
    // Cannot call: &'a str is NOT <: &'static str
    // accept_static(s); // ERROR
}
```

### Invariance of Mutable References

```rust
use std::fmt::Display;

// FAILS: Cannot pass &mut String where &mut Display expected
fn show(d: &mut dyn Display) {
    println!("{}", d);
}

let mut s = String::from("hello");
// show(&mut s); // ERROR: String is not exactly dyn Display
```

**Workaround**: Use immutable reference (covariant):
```rust
fn show_immut(d: &dyn Display) {
    println!("{}", d);
}

show_immut(&s); // OK: covariance allows trait object coercion
```

### Invariance of &Cell<T> and Interior Mutability

```rust
use std::cell::Cell;

// Cell<T> is invariant to prevent aliasing unsoundness:
fn evil<'a>(c: &Cell<&'a str>, short: &str) {
    // If Cell<&'a str> were covariant, this would compile:
    // c.set(short); // Would store short-lived ref in long-lived cell
}

fn main() {
    let cell: Cell<&'static str> = Cell::new("static");
    let temp = String::from("temp");
    // evil(&cell, &temp); // CORRECTLY REJECTED: lifetime mismatch
}
```

### Variance Rules for Custom Generic Types

```rust
struct MyBox<T> {
    value: T,
}
// MyBox<T> is COVARIANT in T (acts like T)

struct MyMutBox<T> {
    value: *mut T, // Raw pointer
}
// MyMutBox<T> is INVARIANT in T (mutable access)

struct Callback<T> {
    func: fn(T) -> (),
}
// Callback<T> is CONTRAVARIANT in T (function argument)
```

**Checking Variance**:
```rust
// Compile-time variance check:
fn assert_covariance<'a, 'b, T>(x: MyBox<&'a T>) -> MyBox<&'b T>
where
    'a: 'b,
{
    x // OK if MyBox is covariant in first lifetime parameter
}
```

### When to Declare PhantomData

**Pattern 1: Unused Lifetime Parameters**
```rust
struct Iter<'a, T: 'a> {
    ptr: *const T,
    _marker: PhantomData<&'a T>, // Required to declare 'a is used
}
```

**Pattern 2: Drop Check Soundness**
```rust
struct Guard<'a, T> {
    data: &'a mut T,
    _marker: PhantomData<T>, // Ensures dropck knows we own T
}

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        // Can access T here safely
    }
}
```

---

## 4. Subtyping & Lifetime Relationships

### When 'a: 'b (Outlives Relationship)

```
'a: 'b  means  'a ⊇ 'b  (a encompasses b)

Timeline:
─────────────────────────────────────►
    ├──────'a──────┤
         ├──'b──┤

'a: 'b because 'a starts before and ends after 'b
```

### Substituting Longer Lifetimes for Shorter

```rust
// Can always pass longer lifetime where shorter expected:
fn needs_short<'a>(s: &'a str) { /* ... */ }

let static_str: &'static str = "hello";
needs_short(static_str); // OK: 'static: 'a for any 'a
```

### Contravariance in Function Arguments

```rust
// fn(T) is CONTRAVARIANT in T
type Callback<T> = fn(T);

// Can pass function expecting SUPERTYPE where SUBTYPE callback expected:
fn register_callback<T>(cb: Callback<T>) { /* ... */ }

fn handle_specific(s: &'static str) { println!("{}", s); }
fn handle_any(s: &str) { println!("{}", s); }

// This works due to contravariance:
let cb: Callback<&'static str> = handle_any; // fn(&str) <: fn(&'static str)
```

**ASCII Diagram: Function Contravariance**
```
Argument Types:        &'static str  <:  &'a str
                             │              │
Function Types:     fn(&'a str)   <:  fn(&'static str)
                       (accepts more)    (accepts less)

Variance:              CONTRAVARIANT in argument position
```

### Covariance in Return Types

```rust
// fn() -> T is COVARIANT in T
type Producer<T> = fn() -> T;

// Can pass function returning SUBTYPE where SUPERTYPE producer expected:
fn get_static() -> &'static str { "hello" }

let producer: Producer<&'static str> = get_static;
let generic_producer: Producer<&str> = producer; // OK: covariance
```

### Combining Variance Rules

```rust
// Complex example: fn(&'a T) -> &'b U
// - Contravariant in T (argument)
// - Covariant in U (return)
// - Contravariant in 'a (argument lifetime)
// - Covariant in 'b (return lifetime)

type Transformer<'a, 'b, T, U> = fn(&'a T) -> &'b U;

// Given 'long: 'short and SubT <: SuperT:
// Transformer<'short, 'long, SuperT, SubU> <: Transformer<'long, 'short, SubT, SuperU>
//               ↑       ↑        ↑       ↑
//            contra   cov      contra   cov
```

---

## 5. Higher-Ranked Trait Bounds (HRTBs)

### for<'a> Semantics

```rust
// HRTB: Bound must hold for ALL lifetimes
trait Parser {
    fn parse<'a>(&self, input: &'a str) -> &'a str;
}

// Function accepting HRTB:
fn use_parser<P>(parser: P)
where
    P: for<'a> Parser<'a>, // P must work for every possible 'a
{
    let s = String::from("test");
    parser.parse(&s); // Works for specific 'a
}
```

### Compatibility with Variance

```rust
// HRTB interacts with variance:
fn map_strings<F>(items: Vec<String>, f: F) -> Vec<String>
where
    F: for<'a> Fn(&'a str) -> &'a str, // Universal quantification
{
    items.iter().map(|s| f(s).to_string()).collect()
}

// Can pass function with specific lifetime constraints:
fn identity(s: &str) -> &str { s }
map_strings(vec![], identity); // OK: identity satisfies HRTB
```

### When Compiler Infers HRTB Implicitly

```rust
// Implicit HRTB in trait objects:
let closure: Box<dyn Fn(&str) -> &str> = Box::new(|s| s);
// Equivalent to: Box<dyn for<'a> Fn(&'a str) -> &'a str>
```

### HRTB in Closures and Function Traits

```rust
// Closure with HRTB inference:
let closure = |s: &str| -> &str { s };
// Type: impl for<'a> Fn(&'a str) -> &'a str

// Explicitly requiring HRTB:
fn accept_closure<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let temp = String::from("test");
    println!("{}", f(&temp));
}
```

### HRTBs as Workaround for Variance Issues

```rust
// Problem: Cannot abstract over specific lifetime
struct Context<'a> {
    data: &'a str,
}

// FAILS: Cannot name all possible 'a
// fn process<F>(f: F)
// where
//     F: Fn(Context<'a>) -> i32, // What is 'a here?
// { }

// Solution: HRTB abstracts over all lifetimes
fn process<F>(f: F)
where
    F: for<'a> Fn(Context<'a>) -> i32, // Works for ANY 'a
{
    let s = String::from("data");
    let ctx = Context { data: &s };
    f(ctx);
}
```

---

## 6. Self-Referential Structures

### Why They Require Special Handling

```rust
// FAILS: Cannot create self-referential struct safely
struct SelfRef {
    data: String,
    slice: &'??? str, // What lifetime? Cannot reference self.data
}
```

**ASCII Diagram: Self-Reference Problem**
```
Stack Memory:
┌─────────────────┐
│ SelfRef         │
├─────────────────┤
│ data: String    │ ──┐
│   ptr: *───────────►│ Heap: "hello"
│   len: 5        │   │
│   cap: 5        │   │
├─────────────────┤   │
│ slice: &str     │   │
│   ptr: *────────────┘ PROBLEM: Points to data field
│   len: 5        │     (moves invalidate this!)
└─────────────────┘
```

### Invariance in Self-Referential Types

```rust
// Even with Pin, invariance is critical:
use std::pin::Pin;

struct SelfRefPin<'a> {
    data: String,
    slice: &'a str,
    _pin: PhantomData<&'a mut ()>, // Invariant marker
}
```

### Pin<&mut Self> and Self-Referentiality

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

struct SelfReferential {
    data: String,
    ptr: *const str,
    _pin: PhantomPinned, // Prevents moving
}

impl SelfReferential {
    fn new(s: String) -> Pin<Box<Self>> {
        let mut pinned = Box::pin(SelfReferential {
            data: s,
            ptr: std::ptr::null(),
            _pin: PhantomPinned,
        });

        // SAFETY: We never move this after creating the self-reference
        let ptr = unsafe {
            let self_ref: &Self = &*pinned;
            &self_ref.data as *const String as *const str
        };

        unsafe {
            let mut_ref = Pin::as_mut(&mut pinned);
            Pin::get_unchecked_mut(mut_ref).ptr = ptr;
        }

        pinned
    }
}
```

### Problems Variance Creates for Drop

```rust
// Variance issue in drop:
struct Wrapper<'a, T> {
    inner: &'a T,
}

impl<'a, T> Drop for Wrapper<'a, T> {
    fn drop(&mut self) {
        // Can access self.inner here, but 'a might be shortened
        // println!("{}", self.inner); // Potential use-after-free!
    }
}
```

**Solution**: PhantomData enforces drop check:
```rust
struct SafeWrapper<'a, T: 'a> {
    inner: &'a T,
    _marker: PhantomData<&'a T>, // Ensures 'a lives long enough
}
```

### Safe Patterns for Self-Referential Types

**Pattern 1: Use Indices Instead of References**
```rust
struct Arena<T> {
    items: Vec<T>,
}

struct NodeIndex(usize);

struct Node {
    value: i32,
    children: Vec<NodeIndex>, // Indices, not references
}
```

**Pattern 2: Use Rc/Arc for Shared Ownership**
```rust
use std::rc::Rc;

struct Graph {
    nodes: Vec<Rc<Node>>,
}

struct Node {
    value: i32,
    neighbors: Vec<Rc<Node>>, // Shared ownership
}
```

---

## 7. Common Variance Pitfalls

### Incorrectly Allowing Lifetime Upcasting

```rust
// PITFALL: Trying to extend lifetime
fn extend_lifetime<'a>(s: &'a str) -> &'static str {
    // s // ERROR: cannot return &'a str as &'static str
    unimplemented!()
}

// CORRECT: Only shorten lifetimes
fn shorten_lifetime<'a>(s: &'static str) -> &'a str {
    s // OK: covariance allows this
}
```

### Missing Invariance on Mutable References

```rust
// PITFALL: Custom smart pointer without invariance
struct MyBox<T> {
    ptr: *mut T,
}

// PROBLEM: MyBox<T> is covariant in T by default
// Should be invariant for soundness:

struct SafeBox<T> {
    ptr: *mut T,
    _marker: PhantomData<Cell<T>>, // Enforces invariance
}
```

### Variance in Trait Objects

```rust
// PITFALL: Trait objects are invariant
fn accept_display(d: &dyn std::fmt::Display) { /* ... */ }

fn provide_string() -> &String {
    &String::from("hello")
}

// Cannot coerce &String to &dyn Display via subtyping:
// accept_display(provide_string()); // ERROR

// WORKAROUND: Explicit coercion
accept_display(&*provide_string()); // OK
```

### Lifetime Unsoundness from Variance Mistakes

```rust
// CRITICAL PITFALL: Storing short-lived data in long-lived container
use std::cell::RefCell;

thread_local! {
    static STORAGE: RefCell<Option<&'static str>> = RefCell::new(None);
}

fn store_temporary() {
    let temp = String::from("temporary");
    // STORAGE.with(|s| *s.borrow_mut() = Some(&temp)); // ERROR: cannot infer lifetime
    // Good: Compiler prevents this unsoundness
}
```

### Tools to Detect Variance Issues (Miri)

```bash
# Install Miri
rustup +nightly component add miri

# Run with Miri to detect undefined behavior
cargo +nightly miri test

# Miri catches:
# - Use-after-free from lifetime errors
# - Invalid variance assumptions
# - Unsafe code violating variance rules
```

**Example Miri Test**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_variance_safety() {
        let mut s: &'static str = "static";
        let temp = String::from("temp");
        // Attempt to violate variance:
        // s = &temp; // ERROR: compiler prevents this
    }
}
```

---

## 8. Generic Type Variance

### Variance in Vec<T> (Covariant in T)

```rust
// Vec<T> is covariant in T:
fn upcast_vec<'a, 'b>(v: Vec<&'a str>) -> Vec<&'b str>
where
    'a: 'b,
{
    v // OK: Vec<&'a str> <: Vec<&'b str>
}
```

### Variance in &T and &mut T

```
Type               Variance in T    Variance in Lifetime
─────────────────────────────────────────────────────────
&'a T              Covariant        Covariant
&'a mut T          Invariant        Covariant
*const T           Covariant        N/A
*mut T             Invariant        N/A
```

### Variance in Function Pointers

```rust
// fn(T) -> U
// - Contravariant in T
// - Covariant in U

type StringParser = fn(&str) -> String;
type StaticParser = fn(&'static str) -> String;

// Contravariance: Can pass more general where specific expected
let parser: StringParser = |s| s.to_string();
let specific: StaticParser = parser; // OK: fn(&str) <: fn(&'static str)
```

**ASCII Diagram: Function Pointer Variance**
```
Input Position (Contravariant):     Output Position (Covariant):
──────────────────────────────       ─────────────────────────────
&'static str  ───►  &str             String  ───►  String
     │                │                 │              │
fn(&str)     ───►  fn(&'static)    fn()->String  <── fn()->String
  (supertype)       (subtype)         (identity)
```

### Custom Type Variance Annotations

```rust
use std::marker::PhantomData;

// Covariant wrapper:
struct CovariantBox<T> {
    value: *const T,
    _marker: PhantomData<T>,
}

// Invariant wrapper:
struct InvariantBox<T> {
    value: *mut T,
    _marker: PhantomData<Cell<T>>,
}

// Contravariant wrapper:
struct ContravariantBox<T> {
    callback: *const (),
    _marker: PhantomData<fn(T)>,
}
```

### Checking Variance with Compiler Errors

```rust
// Force compiler to reveal variance:
struct Test<'a, T> {
    r: &'a T,
}

fn check_variance<'a, 'b, T>(x: Test<'a, &'a str>) -> Test<'b, &'b str>
where
    'a: 'b,
{
    x // If this compiles, Test is covariant in both 'a and T
}
```

---

## 9. Framework-Level Variance

### Variance in Trait Objects and Async

```rust
use std::future::Future;

// Trait objects are invariant:
trait AsyncHandler {
    fn handle(&self) -> Box<dyn Future<Output = ()>>;
}

// HRTB needed for flexible async bounds:
fn spawn_handler<F, Fut>(f: F)
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()> + 'static,
    F: 'static,
{
    // Can spawn f on runtime
}
```

### Variance in Higher-Order Functions

```rust
// Map function with variance:
fn map<'a, T, U, F>(items: &'a [T], f: F) -> Vec<U>
where
    F: for<'b> Fn(&'b T) -> U, // HRTB: works for any borrow lifetime
{
    items.iter().map(f).collect()
}
```

### Variance in Callback Systems

```rust
// Callback registry with proper variance:
struct Registry<T> {
    callbacks: Vec<Box<dyn Fn(&T)>>, // Invariant in T via trait object
}

impl<T> Registry<T> {
    fn register<F>(&mut self, f: F)
    where
        F: Fn(&T) + 'static,
    {
        self.callbacks.push(Box::new(f));
    }

    fn trigger(&self, value: &T) {
        for cb in &self.callbacks {
            cb(value);
        }
    }
}
```

### Design Patterns that Respect Variance

**Pattern 1: Builder with Immutable References**
```rust
struct QueryBuilder<'a> {
    filters: Vec<&'a str>,
}

impl<'a> QueryBuilder<'a> {
    fn add_filter(mut self, filter: &'a str) -> Self {
        self.filters.push(filter);
        self // Covariance allows flexible lifetimes
    }
}
```

**Pattern 2: Witness Types for Invariance**
```rust
struct Invariant<T>(PhantomData<fn(T) -> T>); // Invariant in T

struct Container<T> {
    data: Vec<T>,
    _invariant: Invariant<T>, // Enforces exact type match
}
```

### Breaking Variance and Unsafe Implications

```rust
// UNSAFE: Transmuting to bypass variance
use std::mem;

fn evil_upcast<'a>(s: &'a str) -> &'static str {
    unsafe {
        mem::transmute::<&'a str, &'static str>(s) // UNDEFINED BEHAVIOR
    }
}

// Calling this creates dangling reference:
// let temp = String::from("temp");
// let static_ref = evil_upcast(&temp);
// drop(temp);
// println!("{}", static_ref); // USE-AFTER-FREE
```

---

## 10. AI Agent Checklist for Lifetime Issues

### Detecting Invalid Lifetime Subtyping

```rust
// RED FLAG 1: Extending lifetimes
fn suspicious<'a>(s: &'a str) -> &'static str {
    s // ERROR: cannot coerce
}

// RED FLAG 2: Storing in static
static mut STORAGE: Option<&str> = None;
fn store(s: &str) {
    // unsafe { STORAGE = Some(s); } // ERROR: lifetime mismatch
}

// RED FLAG 3: Type variance bypass
fn bypass<T>(t: T) -> T {
    unsafe { std::mem::transmute(t) } // DANGER: may violate variance
}
```

### Verifying Variance-Respecting Code

**Checklist**:
- ✅ Immutable references (`&T`) only used in covariant positions
- ✅ Mutable references (`&mut T`) treated as invariant
- ✅ Interior mutability types (`Cell`, `RefCell`) never coerced
- ✅ PhantomData used correctly for unused type parameters
- ✅ Drop implementations don't access shortened lifetimes
- ✅ Transmute never used to bypass lifetime checks

### Patterns That Seem Right But Violate Variance

```rust
// PATTERN 1: Storing mutable reference in collection
fn broken_collect<'a>(items: Vec<&'a mut String>) -> Vec<&'static mut String> {
    // items // ERROR: cannot coerce (invariance)
    unimplemented!()
}

// PATTERN 2: Returning mutable reference to local
fn broken_return() -> &'static mut i32 {
    let mut x = 42;
    // &mut x // ERROR: cannot return reference to local
    unimplemented!()
}

// PATTERN 3: Cell with lifetime parameter
fn broken_cell<'a>(c: Cell<&'a str>) -> Cell<&'static str> {
    // c // ERROR: Cell is invariant
    unimplemented!()
}
```

### How to Refactor Code with Variance Issues

**Issue**: Mutable reference prevents covariance
```rust
// BEFORE: Too restrictive
struct Container<'a> {
    data: &'a mut String,
}

// AFTER: Split read and write capabilities
struct Container<'a> {
    data: &'a String, // Covariant (read-only)
}

struct ContainerMut<'a> {
    data: &'a mut String, // Invariant (write)
}
```

**Issue**: Unnecessary lifetime coupling
```rust
// BEFORE: Lifetimes unnecessarily coupled
fn process<'a>(s: &'a str, t: &'a str) -> &'a str {
    if s.len() > t.len() { s } else { t }
}

// AFTER: Independent lifetimes with bound
fn process<'a, 'b>(s: &'a str, t: &'b str) -> &'a str
where
    'b: 'a, // Only constrain when needed
{
    s // Return value's lifetime clear
}
```

### Testing Code That Heavily Uses Lifetimes

```rust
#[cfg(test)]
mod variance_tests {
    use super::*;

    // Test 1: Verify covariance
    #[test]
    fn test_lifetime_shortening() {
        fn shorten<'a, 'b>(x: &'a str) -> &'b str
        where
            'a: 'b,
        {
            x
        }

        let s: &'static str = "hello";
        let _: &str = shorten(s); // Should compile
    }

    // Test 2: Verify invariance rejection
    #[test]
    fn test_invariance() {
        use std::cell::Cell;

        fn cannot_shorten<'a, 'b>(c: Cell<&'a str>) -> Cell<&'b str>
        where
            'a: 'b,
        {
            // c // Should NOT compile
            todo!()
        }
    }

    // Test 3: HRTB satisfaction
    #[test]
    fn test_hrtb() {
        fn identity(s: &str) -> &str { s }

        fn accept_hrtb<F>(f: F)
        where
            F: for<'a> Fn(&'a str) -> &'a str,
        {
        }

        accept_hrtb(identity); // Should compile
    }
}
```

---

## Conclusion: Variance Mastery

Variance is not optional knowledge—it's essential for writing sound Rust code. AI agents must internalize these rules:

1. **Default to covariance** for immutable types
2. **Require invariance** for mutable types and interior mutability
3. **Use contravariance** for function arguments
4. **Never bypass variance** with unsafe code unless proven sound
5. **Test variance assumptions** with compile-time checks

**Final ASCII Summary:**
```
Variance Quick Reference:
═════════════════════════════════════════════════════════
Type                 Variance        Reason
─────────────────────────────────────────────────────────
&'a T                Covariant       Safe to shorten lifetime
&'a mut T            Invariant       Prevents aliasing
*const T             Covariant       Read-only pointer
*mut T               Invariant       Write capability
Box<T>               Covariant       Owned, like T
Vec<T>               Covariant       Container of T
Cell<T>              Invariant       Interior mutability
fn(T) -> U           T: Contra       Function input
                     U: Covariant    Function output
PhantomData<T>       Matches T       Explicit variance
═════════════════════════════════════════════════════════
```

Master variance, and you master Rust's type system soundness guarantees.
