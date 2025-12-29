# 36. CONSTRUCTOR FUNCTION NAMED NEW

*A factory floor where identical products roll off an assembly line, each beginning from the same blueprint, each initialized to a known good state*

...within a **STRUCT WITH NAMED FIELDS (15)** or **GENERIC STRUCT (16)**, when you need to create instances with sensible defaults without forcing users to spell out every field value...

◆ ◆ ◆

**How do you create new instances of a type when direct construction would expose implementation details or require verbose initialization?**

In many languages, constructors are special syntax tied to the type system. But Rust has no privileged constructor mechanism. The language treats all functions equally, which means you must establish your own conventions.

Users coming from other languages expect a standard way to create instances. If you force them to use struct literal syntax, you expose every field—even those that should remain private. If you provide multiple constructors with different names, users must memorize your custom vocabulary. If you make construction difficult, users will create invalid or partially-initialized values.

The Rust community has converged on a simple convention: every type that can be constructed from scratch provides a function named `new()`. This function returns `Self`, takes no parameters (or only essential ones), and initializes the type to a sensible default state. The name is short, universal, and carries no semantic baggage. It works with generic types, enums, and structs alike.

This pattern dovetails with **PRIVATE FIELDS (17)**: the constructor can access private fields to ensure valid initialization, while users of the type work only through the public API.

**Therefore:**

**Provide an associated function named `new()` that returns `Self`, constructing the type with sensible defaults or from required parameters.**

```rust
// From queue/src/lib.rs
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last.
    younger: Vec<char>  // younger elements, youngest last.
}

impl Queue {
    pub fn new() -> Queue {
        Queue { older: Vec::new(), younger: Vec::new() }
    }
}

// Usage:
let mut q = Queue::new();
q.push('*');
assert_eq!(q.pop(), Some('*'));
```

*The new() function serves as the gateway to every type—a single, predictable entrance that guarantees valid initialization*

◆ ◆ ◆

This leads naturally to **BUILDER METHOD RETURNING SELF (37)** for types needing incremental configuration, **FUNCTION RETURNING RESULT (38)** when construction can fail, and supports **PRIVATE FIELDS (17)** by providing controlled initialization of internal state.
