# Callback Container

## Pattern Name
**Callback Container with Trait Objects**

## Context

You are building a system that needs to store and invoke callbacks or handlers dynamically. Common scenarios include:
- Web frameworks routing URLs to handler functions
- Event systems mapping events to listener callbacks
- Plugin architectures where plugins register handlers
- Command pattern implementations storing executable actions
- GUI frameworks mapping events to handlers

The callbacks may have different concrete types (closures capturing different data), but you need to store them in a uniform collection and invoke them through a common interface.

## Problem

**How do you store a collection of callbacks that have different concrete types but share a common interface, while maintaining flexibility in what data each callback captures?**

You need to:
- Store callbacks with different capture lists in the same collection
- Invoke callbacks through a uniform interface
- Allow callbacks to be closures, function pointers, or any callable type
- Maintain type safety for parameters and return values
- Enable dynamic dispatch when the exact callback type isn't known at compile time

## Forces

- **Static vs Dynamic Dispatch**: Function pointers have zero overhead but limited flexibility; trait objects enable dynamic dispatch
- **Ownership**: Callbacks often capture data; need to decide on ownership model (owned, borrowed, shared)
- **Heterogeneity**: Different callbacks capture different data with different types
- **Lifetime Management**: Closures capturing references need lifetime tracking
- **Performance**: Virtual dispatch has small overhead; boxing requires heap allocation
- **Type Erasure**: Need to store different closure types uniformly while preserving type safety
- **Flexibility vs Simplicity**: Most flexible approach (trait objects) requires understanding advanced features

## Solution

**Store callbacks as boxed trait objects (`Box<dyn Fn>`) in a collection, enabling heterogeneous callable storage with dynamic dispatch.**

### Core Technique

From `/home/user/rust-programming-examples/basic-router/src/lib.rs`:

```rust
use std::collections::HashMap;

struct Request {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Vec<u8>
}

struct Response {
    code: u32,
    headers: HashMap<String, String>,
    body: Vec<u8>
}

// Type alias for boxed callback trait object
type BoxedCallback = Box<dyn Fn(&Request) -> Response>;

struct BasicRouter {
    routes: HashMap<String, BoxedCallback>
}
```

### Key Elements

1. **Trait Object Type**: `Box<dyn Fn(&Request) -> Response>`
   - `dyn Fn(...)`: Trait object for callable with specific signature
   - `Box<...>`: Heap allocation to enable dynamic sizing
   - Allows storing closures with different capture lists

2. **Type Alias**: `type BoxedCallback = Box<dyn Fn(&Request) -> Response>;`
   - Simplifies type signatures
   - Makes intent clear (this is a callback)

3. **Collection**: `HashMap<String, BoxedCallback>`
   - Maps keys (URLs) to callbacks
   - Each callback can have different concrete type

### Adding Callbacks

```rust
impl BasicRouter {
    fn new() -> BasicRouter {
        BasicRouter { routes: HashMap::new() }
    }

    fn add_route<C>(&mut self, url: &str, callback: C)
    where
        C: Fn(&Request) -> Response + 'static
    {
        self.routes.insert(url.to_string(), Box::new(callback));
    }
}
```

**Generic Parameter**: `<C>` makes this method generic over any callable type
**Trait Bound**: `C: Fn(&Request) -> Response + 'static`
- `Fn(&Request) -> Response`: Must be callable with this signature
- `'static`: Must not capture any references (owned data only)

### Invoking Callbacks

```rust
impl BasicRouter {
    fn handle_request(&self, request: &Request) -> Response {
        match self.routes.get(&request.url) {
            None => not_found_response(),
            Some(callback) => callback(request)  // Dynamic dispatch
        }
    }
}

fn not_found_response() -> Response {
    Response {
        code: 404,
        headers: HashMap::new(),
        body: b"<h1>Page not found</h1>".to_vec()
    }
}
```

The `callback(request)` syntax works because `Box<dyn Fn>` implements `Fn` through deref coercion.

### Usage Example

```rust
#[test]
fn test_router() {
    let mut router = BasicRouter::new();

    // Closure without captures
    router.add_route("/", |_| Response {
        code: 200,
        headers: HashMap::new(),
        body: b"<form>".to_vec()
    });

    // Closure capturing context
    let error_code = 500;
    router.add_route("/gcd", move |req| Response {
        code: error_code,  // Captured value
        headers: HashMap::new(),
        body: b"<h1>Internal server error</h1>".to_vec()
    });

    // Function pointer
    router.add_route("/about", get_about_response);

    assert_eq!(router.handle_request(&req("/")).code, 200);
    assert_eq!(router.handle_request(&req("/gcd")).code, 500);
    assert_eq!(router.handle_request(&req("/missing")).code, 404);
}

fn get_about_response(_req: &Request) -> Response {
    Response { code: 200, /* ... */ }
}
```

### Understanding the Fn Trait Hierarchy

Rust has three function traits, forming a hierarchy:

```rust
// 1. FnOnce: Can be called once (may move captured values)
trait FnOnce<Args> {
    type Output;
    fn call_once(self, args: Args) -> Self::Output;
}

// 2. FnMut: Can be called multiple times, may mutate captures
trait FnMut<Args>: FnOnce<Args> {
    fn call_mut(&mut self, args: Args) -> Self::Output;
}

// 3. Fn: Can be called multiple times, immutable access to captures
trait Fn<Args>: FnMut<Args> {
    fn call(&self, args: Args) -> Self::Output;
}
```

For callback storage, we typically use `Fn` because:
- Callbacks need to be invoked multiple times
- Handlers shouldn't modify router state through the callback
- Most restrictive = most flexible for callers

### Variant: Mutable Callbacks

If callbacks need to maintain state:

```rust
type MutableCallback = Box<dyn FnMut(&Request) -> Response>;

struct StatefulRouter {
    routes: HashMap<String, MutableCallback>
}

impl StatefulRouter {
    fn add_route<C>(&mut self, url: &str, callback: C)
    where
        C: FnMut(&Request) -> Response + 'static
    {
        self.routes.insert(url.to_string(), Box::new(callback));
    }

    fn handle_request(&mut self, request: &Request) -> Response {
        match self.routes.get_mut(&request.url) {
            None => not_found_response(),
            Some(callback) => callback(request)  // Mutable call
        }
    }
}

// Example: Counter callback
let mut router = StatefulRouter::new();
let mut count = 0;
router.add_route("/counter", move |_| {
    count += 1;  // Mutates captured state
    Response {
        code: 200,
        body: format!("Count: {}", count).into_bytes(),
        headers: HashMap::new(),
    }
});
```

### Variant: One-Time Callbacks

For callbacks that should only execute once:

```rust
type OnceCallback = Box<dyn FnOnce(&Request) -> Response>;

// Note: Cannot store FnOnce in HashMap directly because calling it
// consumes it. Need wrapper type:
struct OnceRouter {
    routes: HashMap<String, Option<OnceCallback>>
}

impl OnceRouter {
    fn handle_request(&mut self, request: &Request) -> Response {
        match self.routes.get_mut(&request.url) {
            Some(Some(callback)) => {
                // Take the callback out (leaving None)
                let callback = self.routes.get_mut(&request.url)
                    .unwrap()
                    .take()
                    .unwrap();
                callback(request)
            },
            _ => not_found_response()
        }
    }
}
```

## Resulting Context

### Benefits

- **Heterogeneous Storage**: Store closures with different capture types uniformly
- **Dynamic Dispatch**: Call callbacks without knowing concrete type at compile time
- **Type Safety**: Function signature is checked at compile time
- **Flexibility**: Accept closures, function pointers, or any type implementing the function trait
- **Encapsulation**: Callback internals (captured data) are hidden from container

### Liabilities

- **Heap Allocation**: Each callback requires boxing (heap allocation)
- **Indirection Overhead**: Dynamic dispatch involves virtual function call
- **No Clone**: `Box<dyn Fn>` doesn't implement `Clone` (can't copy callbacks)
- **Lifetime Constraints**: `'static` bound prevents capturing references
- **Debugging**: Trait objects are harder to inspect and debug
- **Size**: Each `Box<dyn Fn>` is two pointers (data + vtable)

### Performance Considerations

```rust
// Static dispatch (zero overhead, but same type for all)
fn call_static<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)  // Inlined, no indirection
}

// Dynamic dispatch (small overhead, heterogeneous types)
fn call_dynamic(f: Box<dyn Fn(i32) -> i32>, x: i32) -> i32 {
    f(x)  // Virtual call, cannot inline
}
```

Dynamic dispatch overhead is typically negligible for I/O-bound operations (like web routing) but matters for tight loops.

## Related Patterns

- **Function Traits**: Understanding Fn, FnMut, FnOnce is essential
- **Type Erasure**: Trait objects erase concrete type information
- **Strategy Pattern**: Callbacks implement strategy pattern
- **Command Pattern**: Callbacks can represent commands
- **Observer Pattern**: Callbacks work well for event observers

## Known Uses

- **actix-web**: Web framework uses callbacks for route handlers
- **tokio**: Async runtime accepts callbacks for task spawning
- **serde**: Custom serialization/deserialization callbacks
- **GUI frameworks** (druid, iced): Event handlers stored as callbacks
- **diesel**: Query builder accepts callbacks for complex queries

## Example: Event System

A practical event callback system:

```rust
use std::collections::HashMap;

type EventId = String;
type Listener = Box<dyn Fn(&Event) + 'static>;

struct Event {
    id: EventId,
    data: HashMap<String, String>,
}

struct EventBus {
    listeners: HashMap<EventId, Vec<Listener>>,
}

impl EventBus {
    fn new() -> Self {
        EventBus { listeners: HashMap::new() }
    }

    fn subscribe<F>(&mut self, event_id: &str, listener: F)
    where
        F: Fn(&Event) + 'static
    {
        self.listeners
            .entry(event_id.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(listener));
    }

    fn publish(&self, event: &Event) {
        if let Some(listeners) = self.listeners.get(&event.id) {
            for listener in listeners {
                listener(event);
            }
        }
    }
}

#[test]
fn test_event_bus() {
    let mut bus = EventBus::new();
    let mut received = Vec::new();

    // Subscribe with closure capturing mutable reference
    // (Can't do this! 'static bound prevents it)
    // bus.subscribe("user.login", |event| {
    //     received.push(event.id.clone());  // ERROR: captures reference
    // });

    // Instead: Use channels or shared state
    use std::sync::{Arc, Mutex};
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_clone = Arc::clone(&received);

    bus.subscribe("user.login", move |event| {
        received_clone.lock().unwrap().push(event.id.clone());
    });

    bus.publish(&Event {
        id: "user.login".to_string(),
        data: HashMap::new(),
    });

    assert_eq!(received.lock().unwrap().len(), 1);
}
```

## Guidelines

1. **Choose the Right Fn Trait**:
   - `Fn`: Callback called multiple times, immutable
   - `FnMut`: Callback needs mutable state
   - `FnOnce`: Callback consumes resources, called once

2. **Consider Lifetime Bounds**:
   - `'static`: Most flexible, but prevents reference captures
   - Specific lifetimes: Allow reference captures but complicate types

3. **Type Alias for Clarity**:
   ```rust
   type Handler = Box<dyn Fn(&Request) -> Response>;
   ```

4. **Generic Add, Dynamic Store**:
   ```rust
   fn add<F: Fn() + 'static>(&mut self, f: F) {
       self.items.push(Box::new(f));
   }
   ```

5. **Arc for Shared Callbacks**:
   If callbacks need to be cloned:
   ```rust
   type SharedCallback = Arc<dyn Fn(&Request) -> Response>;
   ```

6. **Document Lifetime Requirements**: Make clear what captures are allowed

## Common Mistakes

### Mistake 1: Trying to Clone Box<dyn Fn>

```rust
// ❌ WRONG: Box<dyn Fn> doesn't implement Clone
let router1 = router.clone();  // ERROR

// ✅ OPTION 1: Use Arc instead
type SharedCallback = Arc<dyn Fn(&Request) -> Response>;

// ✅ OPTION 2: Implement custom Clone with macro
// (Advanced, requires object-safe Clone trait)
```

### Mistake 2: Capturing References Without Lifetime

```rust
// ❌ WRONG: Captures reference, but 'static required
let config = Config::new();
router.add_route("/", |req| {
    use_config(&config);  // ERROR: config doesn't live long enough
    Response { /* ... */ }
});

// ✅ RIGHT: Use Arc to share ownership
let config = Arc::new(Config::new());
let config_clone = Arc::clone(&config);
router.add_route("/", move |req| {
    use_config(&config_clone);
    Response { /* ... */ }
});
```

### Mistake 3: Using Wrong Fn Trait

```rust
// ❌ WRONG: FnMut closure stored as Fn
let mut counter = 0;
router.add_route("/count", |_| {
    counter += 1;  // ERROR: cannot mutate captured variable
    Response { code: 200, /* ... */ }
});

// ✅ RIGHT: Use FnMut for mutable callbacks
type MutableCallback = Box<dyn FnMut(&Request) -> Response>;
```

## Advanced: Send and Sync Bounds

For thread-safe callback storage:

```rust
// Callback that can be sent between threads
type ThreadSafeCallback = Box<dyn Fn(&Request) -> Response + Send + Sync>;

struct ThreadSafeRouter {
    routes: HashMap<String, ThreadSafeCallback>
}

impl ThreadSafeRouter {
    fn add_route<C>(&mut self, url: &str, callback: C)
    where
        C: Fn(&Request) -> Response + Send + Sync + 'static
    {
        self.routes.insert(url.to_string(), Box::new(callback));
    }
}

// Now router can be shared across threads
use std::sync::Arc;
let router = Arc::new(ThreadSafeRouter::new());
```

**Bounds Explanation**:
- `Send`: Callback can be transferred to another thread
- `Sync`: Callback can be shared between threads
- `'static`: Callback doesn't capture references

## Why This Pattern Works in Rust

Rust's trait object system enables type-safe dynamic dispatch:

1. **Type Safety**: Function signatures are checked at compile time
2. **No Null Callbacks**: Type system prevents null pointers
3. **Ownership Clarity**: `Box` clearly indicates ownership
4. **Lifetime Safety**: `'static` bound prevents dangling references
5. **Zero-Cost Abstraction**: Generic `add_route` has no overhead; boxing is explicit

This pattern demonstrates how Rust's type system enables **flexible heterogeneous collections** while maintaining memory safety and type safety guarantees.

## Comparison with Other Approaches

### Function Pointers (fn)

```rust
// Simpler but less flexible
type FnCallback = fn(&Request) -> Response;

// Cannot capture environment (no closures with captures)
router.add_route("/", get_home);  // OK
router.add_route("/", |_| ...);   // OK: empty closure
router.add_route("/", move |_| ...);  // ERROR if captures anything
```

### Generic Storage (Not Possible)

```rust
// ❌ Can't do this: need concrete type for collection
struct Router<F: Fn(&Request) -> Response> {
    routes: HashMap<String, F>  // ERROR: All F must be same type
}
```

### Enum of Callback Types (Verbose)

```rust
enum Callback {
    Function(fn(&Request) -> Response),
    Closure(Box<dyn Fn(&Request) -> Response>),
}

// Verbose but gives you control over dispatch
```

The trait object approach (`Box<dyn Fn>`) provides the best balance of flexibility, type safety, and ergonomics for heterogeneous callback storage.
