# 32. UNIT STRUCT FOR MARKER

*[Illustration: A type system diagram showing a zero-sized struct with no fields, yet carrying semantic meaning through its name and trait implementations, occupying no runtime memory but enabling compile-time distinctions]*

...when you need to distinguish types at compile time without storing runtime data, or when implementing **TRAIT BOUNDS ON GENERIC TYPES (24)** where the type itself is the information...

◆ ◆ ◆

**Sometimes you need a type that exists only for its name—to distinguish one thing from another at compile time—but the type carries no data at runtime. Creating a struct with dummy fields wastes memory and obscures intent.**

Consider a state machine represented in the type system. A database connection might be `Connected` or `Disconnected`, but these states don't need to store data—the state *is* the type:

```rust
// ❌ Wastes memory
struct Connected { _marker: u8 }
struct Disconnected { _marker: u8 }
```

Each instance carries a useless byte just to exist. When you have thousands of connections, you're wasting kilobytes.

Or consider marker types for phantom data, where you need to track a type relationship without storing values of that type. For example, a buffer that's "owned" vs "borrowed" doesn't need to store anything about the ownership—the type parameter conveys that information:

```rust
// We need types that mean "owned" and "borrowed" but store nothing
```

Rust's type system is structural for most types but nominal for structs—two structs with identical fields are different types if they have different names. This means you can create distinct types even with no fields.

A unit struct is written `struct Name;` with no body. It has size zero—the compiler knows this and optimizes it away completely. An instance of a unit struct is written `Name` (not `Name {}`), like a constant. In fact, the type and its sole value have the same name.

Unit structs are commonly used as:
1. **Marker types** for phantom data (tracking types not stored in values)
2. **Type-level flags** for type state patterns
3. **Singleton types** implementing traits
4. **Error variants** in enums where no data is needed

**Therefore:**

**Define a struct with no fields using `struct Name;` syntax. Use it wherever you need a distinct type with no runtime data. The type's *name* carries the semantic meaning, and its *trait implementations* define its behavior.**

```rust
// Zero-sized marker types
struct Owned;
struct Borrowed;

// Use as phantom data to track ownership semantics
use std::marker::PhantomData;

struct Buffer<Ownership> {
    data: Vec<u8>,
    _ownership: PhantomData<Ownership>,
}

impl Buffer<Owned> {
    pub fn new(data: Vec<u8>) -> Self {
        Buffer {
            data,
            _ownership: PhantomData,
        }
    }

    // Only owned buffers can be consumed
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }
}

impl Buffer<Borrowed> {
    // Only borrowed buffers can be created from references
    pub fn from_slice(slice: &[u8]) -> Self {
        Buffer {
            data: slice.to_vec(),
            _ownership: PhantomData,
        }
    }
}

// State machine with unit types
struct Locked;
struct Unlocked;

struct State<S> {
    _state: PhantomData<S>,
}

impl State<Locked> {
    fn unlock(self) -> State<Unlocked> {
        State { _state: PhantomData }
    }
}

impl State<Unlocked> {
    fn lock(self) -> State<Locked> {
        State { _state: PhantomData }
    }
}
```

*[Diagram: Memory layout comparison:

Regular struct with data:
```
struct Person { name: String, age: u32 }
Size: 24 + 4 = 28 bytes (+ padding)
```

Unit struct:
```
struct Marker;
Size: 0 bytes
```

In memory:
```
Vec<Marker> with capacity 1000:
  - Allocates space for capacity (0 * 1000 = 0 bytes)
  - Only stores length and capacity metadata
  - No heap allocation needed!
```
]*

The compiler recognizes unit structs as zero-sized types (ZSTs) and elides them entirely from the runtime representation. A `Vec<Marker>` allocates no heap memory for elements—only the control structure. Pattern matching on a unit struct has no runtime cost.

Unit structs are **nominally typed**—`struct Foo;` and `struct Bar;` are different types even though both have no fields. This is exactly what we want for markers: the *name* is the distinction.

They also implement `Copy` automatically if you derive it, since there's nothing to copy. In fact, copying a unit struct is a no-op—the value already exists everywhere.

◆ ◆ ◆

Use this pattern with **PHANTOMDATA FOR LIFETIME (27)** when you need to track type parameters that aren't stored in fields. Combine with **TYPE STATE PATTERN (25)** to build compile-time state machines. If you need to store data, use a regular struct; if you only need a distinct type, use a unit struct. Derive `Debug`, `Clone`, `Copy` for convenience. Never add dummy fields just to "give the struct something"—that defeats the purpose of zero-sized types.
