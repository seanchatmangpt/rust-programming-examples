# Function Traits

## Pattern Name
**Function Traits for Callback Flexibility**

## Context

You are designing an API that accepts callable values—closures, function pointers, or types implementing function-like behavior. Your API might:
- Accept callbacks for event handling
- Take transformation functions for data processing
- Receive predicates for filtering
- Use custom logic injected by users
- Implement higher-order functions (map, filter, fold)

Rust provides three function traits (`Fn`, `FnMut`, `FnOnce`) that represent different calling conventions and capture semantics. Understanding when to use each trait is essential for flexible, ergonomic APIs.

## Problem

**How do you design functions and types that accept callables in a way that maximizes flexibility while maintaining clear semantics about mutability and ownership?**

You need to:
- Accept the widest possible range of callable types (closures, functions, methods)
- Clearly communicate whether callbacks can be called once, many times, or need mutation
- Enable closures to capture environment by value, reference, or mutable reference
- Avoid over-constraining API users (don't require `Fn` if `FnOnce` suffices)
- Maintain type safety and prevent use-after-move errors

## Forces

- **Generality vs Specificity**: `FnOnce` is most general (accepts all callables), `Fn` is most specific
- **Calling Convention**: Some operations naturally call callbacks once (consume), others call repeatedly
- **Mutability**: Callbacks may need to mutate captured state
- **Ownership**: Closures may need to move captured values out
- **Ergonomics**: Users expect natural syntax (`|x| x + 1` should "just work")
- **Trait Hierarchy**: `Fn` is a subtrait of `FnMut`, which is a subtrait of `FnOnce`
- **Type Inference**: Rust infers which trait based on closure body

## Solution

**Choose the least restrictive function trait that matches your calling pattern: use `FnOnce` if the callback is called once, `FnMut` if it's called multiple times and may mutate state, and `Fn` if it's called multiple times without mutation.**

### The Function Trait Hierarchy

From the Rust standard library (conceptual):

```rust
// Most general: can be called once, may consume captures
trait FnOnce<Args> {
    type Output;
    fn call_once(self, args: Args) -> Self::Output;
}

// Can be called multiple times, may mutate captures
// Automatically implements FnOnce
trait FnMut<Args>: FnOnce<Args> {
    fn call_mut(&mut self, args: Args) -> Self::Output;
}

// Can be called multiple times, immutable access to captures
// Automatically implements FnMut and FnOnce
trait Fn<Args>: FnMut<Args> {
    fn call(&self, args: Args) -> Self::Output;
}
```

**Hierarchy**: `Fn` ⊆ `FnMut` ⊆ `FnOnce`

Any `Fn` can be used where `FnMut` is required.
Any `FnMut` can be used where `FnOnce` is required.

### When to Use Each Trait

#### Use Fn: Immutable, Repeated Calls

From `/home/user/rust-programming-examples/basic-router/src/lib.rs`:

```rust
type BoxedCallback = Box<dyn Fn(&Request) -> Response>;

struct BasicRouter {
    routes: HashMap<String, BoxedCallback>
}

impl BasicRouter {
    fn add_route<C>(&mut self, url: &str, callback: C)
    where
        C: Fn(&Request) -> Response + 'static
    {
        self.routes.insert(url.to_string(), Box::new(callback));
    }

    fn handle_request(&self, request: &Request) -> Response {
        match self.routes.get(&request.url) {
            Some(callback) => callback(request),  // Called with &self
            None => not_found_response(),
        }
    }
}
```

**Characteristics of `Fn`**:
- Takes `&self` (immutable reference to closure)
- Can be called many times
- Cannot mutate captured variables
- Closures that don't capture, or capture by immutable reference, implement `Fn`

**Examples**:
```rust
// Pure function - implements Fn
let add_one = |x| x + 1;

// Captures by immutable reference - implements Fn
let base = 10;
let add_base = |x| x + base;

// Can call multiple times
println!("{}", add_one(5));  // 6
println!("{}", add_one(5));  // 6 (can call again)
```

#### Use FnMut: Mutable, Repeated Calls

When callbacks need to maintain state across calls:

```rust
fn process_items<F>(items: &[i32], mut callback: F)
where
    F: FnMut(i32)
{
    for &item in items {
        callback(item);  // Called with &mut self
    }
}

// Usage: closure that mutates captured state
let mut sum = 0;
process_items(&[1, 2, 3], |x| {
    sum += x;  // Mutates captured variable
});
assert_eq!(sum, 6);
```

**Characteristics of `FnMut`**:
- Takes `&mut self` (mutable reference to closure)
- Can be called many times
- CAN mutate captured variables
- Closures that capture by mutable reference implement `FnMut` (but not `Fn`)

**Examples**:
```rust
// Mutates captured state - implements FnMut but NOT Fn
let mut count = 0;
let mut increment = || {
    count += 1;
    count
};

println!("{}", increment());  // 1
println!("{}", increment());  // 2

// This won't work with Fn bound
// fn requires_fn<F: Fn()>(f: F) { f(); }
// requires_fn(increment);  // ERROR: increment is FnMut, not Fn
```

#### Use FnOnce: Consumes Resources, Called Once

When callbacks move captured values out or are called only once:

```rust
fn call_once<F>(callback: F)
where
    F: FnOnce() -> String
{
    let result = callback();  // Consumes callback
    println!("Result: {}", result);
    // callback();  // ERROR: callback was moved
}

// Usage: closure that moves out captured value
let text = String::from("Hello");
call_once(|| {
    text  // Moves text out of closure
});
// text is no longer accessible here
```

**Characteristics of `FnOnce`**:
- Takes `self` (consumes the closure)
- Can be called only once
- Can move out of captured variables
- All closures implement `FnOnce` (most general trait)

**Examples**:
```rust
// Moves captured value - implements FnOnce only
let resource = vec![1, 2, 3];
let consume = || {
    drop(resource);  // Moves resource
};

consume();      // OK: called once
// consume();   // ERROR: consume was moved when called
```

### Pattern: Accept Most General Trait

```rust
// ✅ GOOD: Most flexible
fn call_with_five<F>(f: F) -> i32
where
    F: FnOnce(i32) -> i32  // Accepts Fn, FnMut, or FnOnce
{
    f(5)
}

// Can use with any closure type
call_with_five(|x| x + 1);              // Fn
call_with_five(|x| { let mut y = x; y += 1; y });  // FnMut internally
call_with_five(|x| { let s = x.to_string(); s.len() as i32 });  // FnOnce
```

### Pattern: Multiple Calls Require FnMut or Fn

```rust
// Calls callback multiple times
fn apply_twice<F>(mut f: F, x: i32) -> i32
where
    F: FnMut(i32) -> i32  // Must be FnMut or Fn (not FnOnce)
{
    let temp = f(x);
    f(temp)
}

let double = |x| x * 2;
assert_eq!(apply_twice(double, 5), 20);  // double(double(5))
```

### Pattern: Storing Callbacks

From the `basic-router` example, when storing callbacks for later invocation:

```rust
struct EventHandler<F>
where
    F: Fn(&Event),
{
    callback: F,
}

impl<F> EventHandler<F>
where
    F: Fn(&Event),
{
    fn new(callback: F) -> Self {
        EventHandler { callback }
    }

    fn trigger(&self, event: &Event) {
        (self.callback)(event);  // Call with &self
    }
}

// Usage
let handler = EventHandler::new(|event| {
    println!("Event: {:?}", event);
});
handler.trigger(&event1);
handler.trigger(&event2);  // Can trigger multiple times
```

### Inference: Rust Determines Which Trait

Rust automatically infers the most specific function trait based on closure body:

```rust
// Rust infers Fn (no captures, or immutable captures)
let f1 = |x| x + 1;

// Rust infers FnMut (mutates captures)
let mut sum = 0;
let f2 = |x| sum += x;

// Rust infers FnOnce (moves captures)
let vec = vec![1, 2, 3];
let f3 = || drop(vec);
```

You don't explicitly specify which trait; the compiler figures it out.

## Resulting Context

### Benefits

- **Maximum Flexibility**: Accept widest range of closures by using most general trait
- **Clear Semantics**: Trait choice communicates calling pattern and mutability
- **Type Safety**: Compiler prevents calling FnOnce twice
- **Ergonomics**: Users write natural closure syntax; compiler infers trait
- **Zero-Cost Abstraction**: No runtime overhead for static dispatch
- **Composability**: Function traits work seamlessly with generic code

### Liabilities

- **Conceptual Complexity**: Three function traits can confuse beginners
- **Error Messages**: Trait mismatches produce verbose error messages
- **Lifetime Complexity**: Closures capturing references introduce lifetimes
- **No Partial Application**: Can't easily create curried functions
- **Trait Objects**: Dynamic dispatch requires choosing one trait (`Box<dyn Fn>`)

### Common Errors

```rust
// ERROR: Can't call FnOnce twice
fn call_twice<F: FnOnce()>(f: F) {
    f();
    f();  // ERROR: use of moved value `f`
}

// ERROR: Can't mutate with Fn
fn with_fn<F: Fn()>(mut f: F) {
    // If f captures mut state, this won't compile
}

let mut x = 0;
with_fn(|| x += 1);  // ERROR: x is mutated, so closure is FnMut, not Fn
```

## Related Patterns

- **Callback Container**: Stores function trait objects for dynamic dispatch
- **Iterator Adapters**: Methods like `map`, `filter` use function traits
- **Strategy Pattern**: Function traits enable strategy pattern
- **Template Method**: Pass behavior via closures
- **Higher-Order Functions**: Functions taking or returning closures

## Known Uses

- **Iterator methods**: `map`, `filter`, `fold` accept `FnMut` closures
  ```rust
  vec.iter().map(|x| x * 2)  // FnMut closure
  ```

- **Thread spawning**: `std::thread::spawn` requires `FnOnce + Send`
  ```rust
  std::thread::spawn(|| { /* moved data */ })
  ```

- **Option::map**: Takes `FnOnce` (called at most once)
  ```rust
  Some(5).map(|x| x * 2)  // FnOnce closure
  ```

- **Web frameworks**: Route handlers are `Fn(&Request) -> Response`

- **Event systems**: Event listeners are typically `Fn(&Event)`

## Example: Practical Usage

### Map-Reduce Pattern

```rust
fn map_reduce<T, U, M, R>(
    items: &[T],
    map_fn: M,
    reduce_fn: R,
    initial: U,
) -> U
where
    T: Clone,
    U: Clone,
    M: Fn(&T) -> U,          // Map called many times, immutable
    R: FnMut(U, U) -> U,     // Reduce accumulates state
{
    let mapped: Vec<U> = items.iter().map(map_fn).collect();
    mapped.into_iter().fold(initial, reduce_fn)
}

// Usage
let numbers = vec![1, 2, 3, 4, 5];
let sum_of_squares = map_reduce(
    &numbers,
    |x| x * x,              // Map: square each number (Fn)
    |acc, x| acc + x,       // Reduce: sum (FnMut)
    0,
);
assert_eq!(sum_of_squares, 55);  // 1 + 4 + 9 + 16 + 25
```

### Callback Registration

```rust
struct Button {
    on_click: Option<Box<dyn Fn()>>,
}

impl Button {
    fn new() -> Self {
        Button { on_click: None }
    }

    fn set_on_click<F>(&mut self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.on_click = Some(Box::new(callback));
    }

    fn click(&self) {
        if let Some(ref callback) = self.on_click {
            callback();
        }
    }
}

let mut button = Button::new();
let mut click_count = 0;

// Can't capture mutable reference with Fn bound
// button.set_on_click(|| click_count += 1);  // ERROR

// Solution: Use Arc<Mutex> for shared mutable state
use std::sync::{Arc, Mutex};
let counter = Arc::new(Mutex::new(0));
let counter_clone = Arc::clone(&counter);

button.set_on_click(move || {
    *counter_clone.lock().unwrap() += 1;
});

button.click();
button.click();
assert_eq!(*counter.lock().unwrap(), 2);
```

## Guidelines

### Choosing the Right Trait

1. **Single Call? Use FnOnce**
   ```rust
   fn once<F: FnOnce()>(f: F) { f(); }
   ```

2. **Multiple Calls, Immutable? Use Fn**
   ```rust
   fn many<F: Fn()>(f: F) { f(); f(); }
   ```

3. **Multiple Calls, Mutable? Use FnMut**
   ```rust
   fn many_mut<F: FnMut()>(mut f: F) { f(); f(); }
   ```

4. **Storing Callback? Usually Fn**
   ```rust
   struct Handler<F: Fn()> { callback: F }
   ```

5. **Maximum Flexibility? Use FnOnce**
   ```rust
   fn flexible<F: FnOnce()>(f: F) { f(); }  // Accepts all
   ```

### API Design Checklist

- [ ] Will callback be called once or multiple times?
- [ ] Does callback need to mutate captured state?
- [ ] Is callback stored for later or called immediately?
- [ ] Should callback be thread-safe? (add `Send`/`Sync` bounds)
- [ ] Does callback capture references? (add lifetime parameters)

### Common Bounds

```rust
// Immutable, repeatable, thread-safe
F: Fn() + Send + Sync + 'static

// Mutable, repeatable, owned
F: FnMut() + 'static

// Single-use, thread-safe
F: FnOnce() + Send + 'static
```

## Advanced: Return Type Impl Trait

Modern Rust allows returning closures without boxing:

```rust
// Before: Had to box return value
fn make_adder_boxed(x: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |y| x + y)
}

// After: impl Trait (no boxing, zero cost)
fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}

let add_five = make_adder(5);
assert_eq!(add_five(10), 15);
```

## Common Mistakes

### Mistake 1: Using Fn When FnMut Needed

```rust
// ❌ WRONG: Tries to mutate with Fn bound
fn process<F: Fn(i32)>(items: &[i32], f: F) {
    for &item in items {
        f(item);
    }
}

let mut sum = 0;
process(&[1, 2, 3], |x| sum += x);  // ERROR: closure is FnMut, not Fn

// ✅ RIGHT: Use FnMut
fn process<F: FnMut(i32)>(items: &[i32], mut f: F) {
    for &item in items {
        f(item);
    }
}
```

### Mistake 2: Over-Constraining with Fn

```rust
// ❌ UNNECESSARILY RESTRICTIVE: Only calls once, but requires Fn
fn call_once<F: Fn()>(f: F) {
    f();  // Only called once, FnOnce would suffice
}

// ✅ BETTER: Use FnOnce for maximum flexibility
fn call_once<F: FnOnce()>(f: F) {
    f();
}
```

### Mistake 3: Forgetting mut for FnMut

```rust
// ❌ WRONG: FnMut closure not marked mut
fn apply_twice<F: FnMut()>(f: F) {
    f();  // ERROR: cannot borrow as mutable
    f();
}

// ✅ RIGHT: Mark parameter as mut
fn apply_twice<F: FnMut()>(mut f: F) {
    f();
    f();
}
```

## Why This Pattern Works in Rust

Rust's function traits enable powerful abstractions while maintaining safety:

1. **Type Safety**: Compiler enforces calling conventions (once vs multiple)
2. **Zero-Cost**: Generic closures are inlined (no allocation or indirection)
3. **Ownership Tracking**: Prevents use-after-move for `FnOnce`
4. **Mutability Control**: Type system enforces mutability requirements
5. **Trait Hierarchy**: Subtyping allows flexible APIs

This pattern demonstrates Rust's philosophy of **fearless concurrency and memory safety** through the type system: you can write generic, reusable code with closures while the compiler ensures safety at compile time.

## Comparison Table

| Trait | Calls | Mutates? | Moves? | Use Case |
|-------|-------|----------|--------|----------|
| `Fn` | Multiple | No | No | Event handlers, pure transformations |
| `FnMut` | Multiple | Yes | No | Stateful operations, accumulators |
| `FnOnce` | Once | Yes | Yes | Resource cleanup, thread spawning |

### Closure Examples

```rust
let x = 5;
let y = &x;
let mut z = 10;
let w = String::from("hello");

// Fn: Immutable borrows only
let f1 = || println!("{}", x);     // Copy
let f2 = || println!("{}", y);     // Immutable borrow

// FnMut: Mutable borrows
let f3 = || z += 1;                // Mutable borrow

// FnOnce: Moves values
let f4 = || drop(w);               // Moves w
```

Understanding when to use each function trait is essential for designing flexible, ergonomic Rust APIs that leverage closures effectively.
