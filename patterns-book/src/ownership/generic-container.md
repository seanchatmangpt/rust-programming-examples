# Generic Container

**Also Known As:** Type Parameterization, Polymorphic Container, Monomorphization

## Context

You've built a concrete data structure—perhaps a `Queue` that works with `char`, or a tree that stores `i32`. The implementation is correct, tested, and you understand exactly how ownership flows through each method. Now you face a new requirement: the same data structure must work with multiple types. You could copy-paste the code and replace `char` with `String`, but that path leads to maintenance nightmares.

## Problem

How do you transform a concrete container into a reusable abstraction that works with any type, while preserving Rust's zero-cost abstraction guarantee and maintaining the ownership semantics you've already mastered?

**The deeper question:** How do you add type parameters without changing the fundamental behavior of your data structure or introducing runtime overhead?

## Forces

- **Code reuse**: One implementation should work for all types
- **Type safety**: A `Queue<char>` must not accept `String` values
- **Zero-cost abstraction**: Generic code should compile to the same machine code as concrete types
- **Ownership preservation**: Move, borrow, and lifetime semantics must remain correct
- **Trait bounds**: Some operations require specific type capabilities (Clone, Ord, etc.)
- **Ergonomics**: Type inference should work; users shouldn't constantly annotate types
- **Compile times**: Monomorphization creates code for each instantiated type

## Solution

**Replace the concrete type with a type parameter `T`, add `impl<T>` to method blocks, and let Rust's monomorphization generate specialized versions at compile time.**

The evolution from concrete to generic is surgical and systematic:

**Before (concrete):**
```rust
pub struct Queue {
    older: Vec<char>,
    younger: Vec<char>
}

impl Queue {
    pub fn new() -> Queue {
        Queue { older: Vec::new(), younger: Vec::new() }
    }

    pub fn push(&mut self, c: char) {
        self.younger.push(c);
    }
}
```

**After (generic):**
```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { older: Vec::new(), younger: Vec::new() }
    }

    pub fn push(&mut self, t: T) {
        self.younger.push(t);
    }
}
```

**What changed:**

1. **Struct declaration**: `Queue` → `Queue<T>` (introduces type parameter)
2. **Field types**: `Vec<char>` → `Vec<T>` (parameterizes storage)
3. **Impl block**: `impl Queue` → `impl<T> Queue<T>` (scopes type parameter)
4. **Return types**: `Queue` → `Self` (type alias for current type)
5. **Parameter names**: `c: char` → `t: T` (semantic clarity)

**The complete transformation:**

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { older: Vec::new(), younger: Vec::new() }
    }

    pub fn push(&mut self, t: T) {
        self.younger.push(t);
    }

    pub fn is_empty(&self) -> bool {
        self.older.is_empty() && self.younger.is_empty()
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.older.is_empty() {
            use std::mem::swap;

            if self.younger.is_empty() {
                return None;
            }

            // Bring the elements in younger over to older, and put them in
            // the promised order.
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        // Now older is guaranteed to have something. Vec's pop method
        // already returns an Option, so we're set.
        self.older.pop()
    }

    pub fn split(self) -> (Vec<T>, Vec<T>) {
        (self.older, self.younger)
    }
}
```

**Crucially, the algorithm doesn't change.** The ownership flow is identical:
- `push` borrows mutably (`&mut self`)
- `pop` borrows mutably and returns ownership of a `T` (via `Option<T>`)
- `split` consumes the queue (takes `self`) and transfers ownership of the vectors

**Type inference in action:**

```rust
let mut q = Queue::new();
q.push("CAD");   // Compiler infers Queue<&'static str>
q.push("BTC");   // Confirmed: still &str

let mut r = Queue::new();
r.push(0.74);    // Compiler infers Queue<f64>
r.push(13764.0); // Confirmed: still f64
```

The compiler determines `T` from first use. No explicit `Queue::<&str>::new()` required (though you can write it).

**When trait bounds are needed:**

Not all operations work on all types. The binary tree's `add` method requires ordering:

```rust
impl<T: Ord> BinaryTree<T> {
    fn add(&mut self, value: T) {
        // Can use <= because T: Ord
        if value <= node.element {
            node.left.add(value);
        }
    }
}
```

The bound `T: Ord` restricts `add` to types that can be ordered. This is a _method-level_ constraint; the `BinaryTree<T>` type itself has no bounds.

## Resulting Context

**Benefits:**
- **True reusability**: `Queue<char>`, `Queue<String>`, `Queue<MyType>` all work
- **Type safety**: Cannot mix types; `Queue<i32>` rejects `String` at compile time
- **Zero runtime cost**: Monomorphization generates specialized code per type, identical to hand-written concrete versions
- **Inference ergonomics**: Most of the time, type parameters are invisible to users
- **Standard library compatibility**: Works with traits like `Iterator`, `Debug`, `Clone`

**Liabilities:**
- **Compile time increase**: Each instantiation (`Queue<char>`, `Queue<String>`) is compiled separately
- **Code bloat potential**: Many instantiations increase binary size (though often optimized away)
- **Error message complexity**: Compiler errors mention `T` and trait bounds, requiring more careful reading
- **Learning curve**: Understanding `impl<T>` vs `impl<T: Trait>` vs `impl Queue<SpecificType>` takes time
- **Trait bound proliferation**: Complex types accumulate bounds like `T: Clone + Debug + Default + Ord`

**What's now enabled:**
- Your container works in generic contexts (other generic functions, trait implementations)
- You can add trait-bounded methods incrementally (`impl<T: Clone>`, `impl<T: Ord>`)
- The structure can be published as a library crate
- Users can specialize it for their own types without forking your code

## Related Patterns

- [Concrete Container](./concrete-container.md) - Start here before generalizing
- [Builder Methods](./builder-methods.md) - `split(self)` works identically with generics
- [Recursive Ownership](./recursive-ownership.md) - `BinaryTree<T>` shows generic recursion with `Box`
- Interior Iteration - Generic iterators require lifetime parameters

## Known Uses

**In this codebase:**
- `/home/user/rust-programming-examples/generic-queue/src/lib.rs` - Direct evolution from concrete Queue
- `/home/user/rust-programming-examples/binary-tree/src/lib.rs` - Generic tree with trait-bounded methods

**In the standard library:**
- `Vec<T>` - The archetypal generic container
- `HashMap<K, V>` - Multi-parameter generics
- `BTreeMap<K, V>` - Generic with trait bounds (`K: Ord`)
- `Option<T>`, `Result<T, E>` - Generic enums

**In the wider ecosystem:**
- `serde::Serialize` - Trait for generic serialization
- `tokio::sync::Mutex<T>` - Async-aware generic synchronization
- `Arc<T>`, `Rc<T>` - Generic reference counting

**The monomorphization trade-off:**

Rust's approach differs from languages with runtime generics (Java, C#) or erasure (Go, older Java). Each instantiation produces specialized machine code:

```rust
let q1: Queue<u8> = Queue::new();   // Generates Queue_u8 code
let q2: Queue<u64> = Queue::new();  // Generates Queue_u64 code
```

The compiler literally creates two versions of the code, optimized for each type. This is why Rust generics have zero runtime cost—but it's also why Rust binaries can be larger and compilation slower.

**Alexander's insight:**

"The pattern describes the core of the solution in such a way that you can use this solution a million times over, without ever doing it the same way twice."

Generic containers embody this perfectly. The _pattern_ (`impl<T>` with type parameters) is constant, but each instantiation—`Queue<char>`, `Queue<Request>`, `Queue<Future>`—is a unique, optimized implementation. The abstraction carries no cost; it's as if you hand-wrote each version.

This is Rust's central bargain: compile-time complexity for runtime performance. The pattern you see in `Queue<T>` scales to the most complex systems—async runtimes, database drivers, web frameworks—all built on this foundation of zero-cost generic abstraction.
