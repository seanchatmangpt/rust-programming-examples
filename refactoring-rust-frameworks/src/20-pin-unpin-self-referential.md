# Pin, Unpin & Self-Referential Structures

**Target Audience**: AI agents working with async code, futures, and self-referential data structures
**Prerequisites**: Understanding of ownership, lifetimes, and unsafe code
**Scope**: Pin guarantees, Unpin trait, structural pinning, async implications

---

## 1. Why Pin Exists

### The Self-Referential Problem

Rust's ownership model assumes values can be moved freely in memory. However, certain data structures contain internal pointers that become invalid when the structure moves:

```rust
// PROBLEMATIC: Self-referential struct
struct SelfReferential {
    data: String,
    ptr: *const String,  // Points to 'data' field
}

impl SelfReferential {
    fn new(s: String) -> Self {
        let mut obj = SelfReferential {
            data: s,
            ptr: std::ptr::null(),
        };
        obj.ptr = &obj.data as *const String;  // Becomes dangling on move!
        obj
    }
}
```

When this value moves in memory, the pointer becomes dangling because it still points to the old memory location.

### Async State Machines and Pin

The compiler transforms async functions into state machines that can self-reference:

```rust
async fn example() {
    let local = String::from("data");
    do_something(&local).await;  // State machine holds reference to local
}
```

**Pin solves this** by guaranteeing the value won't move in memory.

## 2. Pin Fundamentals

### Pin<P<T>> Structure

```rust
pub struct Pin<P> {
    pointer: P,
}

impl<P: Deref> Pin<P> {
    // Safe: can only create Pin from pointer to Unpin type
    pub fn new(pointer: P) -> Pin<P>
    where
        P::Target: Unpin,
    {
        unsafe { Pin::new_unchecked(pointer) }
    }
}
```

### Creating Pinned Values

**Heap Pinning (Safe):**
```rust
use std::pin::Pin;

let pinned: Pin<Box<String>> = Box::pin(String::from("pinned"));
// Value is now permanently pinned on heap
```

**Stack Pinning (With macro):**
```rust
use std::pin::pin;

fn stack_pin_example() {
    let pinned = pin!(String::from("pinned"));
    // Type: Pin<&mut String>
    // Valid only within this scope
}
```

## 3. Unpin & Opt-Out

### Unpin Auto Trait

`Unpin` is an **auto trait** implemented by default for most types:

```rust
// These automatically implement Unpin:
struct Regular {
    x: i32,
    y: String,
}

// Unpin allows Pin to be safely created and unwrapped:
let boxed = Box::new(Regular { x: 5, y: String::from("test") });
let pinned = Pin::new(boxed);  // OK: Regular is Unpin
let unpinned = Pin::into_inner(pinned);  // OK: can move again
```

### Removing Unpin with PhantomPinned

```rust
use std::marker::PhantomPinned;

struct MustPin {
    data: String,
    ptr: *const String,
    _pin: PhantomPinned,  // Removes Unpin
}
```

A type is `!Unpin` if:
1. It contains `PhantomPinned`
2. It contains a field that is `!Unpin`
3. It explicitly opts out: `impl !Unpin for MyType {}`

## 4. Projection & Structural Pinning

### Structural Pinning Rules

**Structural pinning**: If a parent is pinned, its fields are also pinned.

```rust
use std::pin::Pin;

struct Parent {
    field1: String,      // Unpin
    field2: Vec<u8>,     // Unpin
}

impl Parent {
    // SAFE projection: String is Unpin
    fn project_field1(self: Pin<&mut Self>) -> &mut String {
        &mut self.get_mut().field1
    }
}
```

### Pin::map_unchecked for Field Access

```rust
impl<P: Deref> Pin<P> {
    pub unsafe fn map_unchecked<U, F>(self, func: F) -> Pin<&U>
    where
        F: FnOnce(&P::Target) -> &U,
    {
        // SAFETY: Caller must ensure structural pinning invariants
        let pointer = &*self.pointer;
        let new_pointer = func(pointer);
        Pin::new_unchecked(new_pointer)
    }
}
```

## 5. Self-Referential Structs

### Safe Self-Referential Pattern

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

pub struct SelfRef {
    data: String,
    // Points to data - only valid when pinned
    data_ptr: *const String,
    _pin: PhantomPinned,
}

impl SelfRef {
    pub fn new(s: String) -> Pin<Box<Self>> {
        let mut obj = Box::new(SelfRef {
            data: s,
            data_ptr: std::ptr::null(),
            _pin: PhantomPinned,
        });

        // Initialize pointer after boxing
        let ptr: *const String = &obj.data;
        obj.data_ptr = ptr;

        // Pin guarantees obj won't move
        unsafe { Pin::new_unchecked(obj) }
    }

    pub fn get_data(&self) -> &str {
        &self.data
    }

    pub fn get_via_pointer(&self) -> &str {
        unsafe {
            // SAFETY: Pin guarantees data_ptr remains valid
            &*self.data_ptr
        }
    }
}
```

## 6. Drop with Pin

### Drop Requirements

When implementing `Drop` for pinned types:

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

struct PinnedResource {
    data: Vec<u8>,
    registered: bool,
    _pin: PhantomPinned,
}

impl Drop for PinnedResource {
    fn drop(&mut self) {
        // SAFETY: Drop is called in-place
        if self.registered {
            println!("Unregistering resource at {:p}", self);
        }
        self.data.clear();
    }
}
```

**Critical rule**: Drop must never panic - it could leave the system in an inconsistent state.

## 7. Async/Await & Pin

### Why Futures Require Pin

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// Future trait requires Pin:
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

The compiler generates state machines for async functions that self-reference across `await` points. Pin guarantees these self-references remain valid.

### Pinning in Executors

```rust
use std::future::Future;
use std::pin::Pin;

struct SimpleExecutor;

impl SimpleExecutor {
    fn block_on<F: Future>(future: F) -> F::Output {
        // Must pin future before polling
        let mut future = Box::pin(future);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => {
                    std::thread::yield_now();
                }
            }
        }
    }
}
```

## 8. Common Pinning Patterns

### Intrusive Linked List

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

pub struct IntrusiveNode<T> {
    value: T,
    next: Option<*mut IntrusiveNode<T>>,
    prev: Option<*mut IntrusiveNode<T>>,
    _pin: PhantomPinned,
}

pub struct IntrusiveList<T> {
    head: Option<Pin<Box<IntrusiveNode<T>>>>,
}
```

### Custom Future Pattern

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct MyFuture {
    state: State,
    data: String,
    data_ref: Option<*const String>,
}

enum State {
    NotStarted,
    InProgress,
    Done,
}

impl Future for MyFuture {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<usize> {
        let this = unsafe { self.get_unchecked_mut() };

        match this.state {
            State::NotStarted => {
                // Initialize self-reference
                this.data_ref = Some(&this.data as *const String);
                this.state = State::InProgress;
                Poll::Pending
            }
            State::InProgress => {
                let data_ref = unsafe { &*this.data_ref.unwrap() };
                this.state = State::Done;
                Poll::Ready(data_ref.len())
            }
            State::Done => panic!("Polled after completion"),
        }
    }
}
```

## 9. Pin Safety Rules

### Structural Pinning Invariants

**Rule 1**: If `Pin<&mut T>` exists, `T` will not move until dropped.

**Rule 2**: Structural pinning - pinning a struct pins all `!Unpin` fields.

**Rule 3**: Cannot obtain `&mut T` from `Pin<&mut T>` if `T: !Unpin`.

```rust
// SOUND projection:
struct Sound {
    unpinned: String,      // Unpin
    pinned: PhantomPinned, // !Unpin
}

impl Sound {
    // OK: String is Unpin
    fn project_unpinned(self: Pin<&mut Self>) -> &mut String {
        &mut self.get_mut().unpinned
    }

    // OK: Returns pinned reference
    fn project_pinned(self: Pin<&mut Self>) -> Pin<&PhantomPinned> {
        unsafe { self.map_unchecked(|s| &s.pinned) }
    }
}
```

## 10. Testing Pinned Types

### Asserting Unpin

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn assert_unpin<T: Unpin>() {}

    #[test]
    fn test_unpin_properties() {
        // These should be Unpin
        assert_unpin::<String>();
        assert_unpin::<Vec<u8>>();

        // These should NOT be Unpin
        // assert_unpin::<PhantomPinned>();  // Won't compile
    }
}
```

### Miri Analysis

```bash
# Run tests under Miri to detect undefined behavior
cargo +nightly miri test

# Miri will catch:
# - Invalid pointer dereferences
# - Use-after-free
# - Violations of Pin guarantees
```

## 11. Performance Implications

### Pin Overhead

Pin is a **zero-cost abstraction**. Pin overhead: ~0% at runtime.

Pin is purely a type-system guarantee enforced at compile-time.

### Stack vs Heap Pinning

```rust
fn pinning_performance() {
    // Heap allocation (required for safety)
    let heap_pinned = Box::pin(String::from("heap"));

    // Stack allocation (faster, lifetime-restricted)
    let stack_pinned = std::pin::pin!(String::from("stack"));
}
```

Stack pinning avoids allocation but has lifetime restrictions.

## 12. Compiler Analysis

### Pin Error Messages

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

struct NotUnpin {
    _pin: PhantomPinned,
}

fn causes_error() {
    let value = Box::new(NotUnpin { _pin: PhantomPinned });
    let pinned = Pin::new(value);
    // ERROR: the trait `Unpin` is not implemented
}
```

### Common Pin Confusion

"Pin<&mut T> doesn't let me access fields" - this is by design for `!Unpin` types.

## 13. Alternatives to Pin

### When Pin Is Unnecessary

```rust
// BEFORE (using Pin):
struct WithPin {
    data: String,
    ptr: *const String,
    _pin: PhantomPinned,
}

// AFTER (avoiding self-reference):
struct WithoutPin {
    data: String,
}

impl WithoutPin {
    fn get_ref(&self) -> &str {
        &self.data  // Just borrow normally
    }
}
```

## 14. AI Agent Pin Analysis

### Detecting Unsound Pinning

```rust
// AI agents should flag this:
struct Unsound {
    data: Vec<u8>,
    ptr: *const Vec<u8>,
    _pin: PhantomPinned,
}

impl Unsound {
    // UNSOUND: Exposes &mut to !Unpin type
    fn get_data_mut(self: Pin<&mut Self>) -> &mut Vec<u8> {
        unsafe { &mut self.get_unchecked_mut().data }  // DANGER!
    }
}
```

### Verification Checklist

AI agents analyzing Pin code should verify:

1. ✅ **PhantomPinned usage** - Present for self-referential types
2. ✅ **Projection safety** - Projections return `Pin<&mut Field>` for `!Unpin` fields
3. ✅ **Initialization order** - Self-references initialized after pinning
4. ✅ **Drop implementation** - Panic-safe and doesn't move values
5. ✅ **Future implementation** - Uses `Pin<&mut Self>` correctly

## Summary: Pin Mental Model

```
┌──────────────────────────────────────────┐
│ Pin<P<T>> Guarantees                     │
├──────────────────────────────────────────┤
│                                          │
│  IF T: !Unpin:                           │
│    - Value CANNOT move in memory        │
│    - Address remains stable until drop   │
│    - Enables self-references             │
│                                          │
│  IF T: Unpin:                            │
│    - Pin is transparent wrapper          │
│    - Value CAN be moved                  │
│    - No special guarantees               │
│                                          │
└──────────────────────────────────────────┘
```

**Key Insight**: Pin enables safe self-referential structures by preventing moves through the type system. It's essential for async/await but adds complexity—use only when necessary.
