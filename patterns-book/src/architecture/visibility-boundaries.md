# Visibility Boundaries

## Pattern Summary

Use Rust's visibility modifiers (`pub`, `pub(crate)`, `pub(super)`, and private) strategically to create architectural boundaries that enforce design decisions and prevent unwanted coupling between modules.

---

## Context

You are developing a Rust library or application with multiple modules. Some functions and types are part of your public API, others are shared across modules within your crate, and still others are pure implementation details that should never be accessed outside their defining module.

You want the compiler to enforce architectural boundaries—preventing accidental dependencies, making refactoring safer, and clearly communicating intent to other developers.

---

## Problem

**How do you control access to types and functions in a way that enforces your architectural design, prevents unwanted coupling, and makes the intended scope of each item explicit?**

Making everything public (`pub`) creates problems:
- **Accidental dependencies**: Other modules depend on implementation details
- **Fragile code**: Changing internals breaks unrelated code
- **Unclear intent**: No distinction between public API and internal helpers
- **Semver violations**: Changing "internal" items is technically a breaking change if they're `pub`

Making everything private creates problems:
- **Can't share code**: Utility functions must be duplicated or made public
- **Inflexible architecture**: Modules can't collaborate

---

## Forces

- **Encapsulation**: Implementation details should be hidden
- **Modularity**: Related modules should share code without exposing it globally
- **Clarity**: Code should clearly indicate its intended scope
- **Safety**: The compiler should prevent architectural violations
- **Flexibility**: Modules should collaborate without tight coupling
- **API stability**: Public APIs should be stable; internals can change freely
- **Discoverability**: Users should see only relevant items in documentation

Too much visibility leads to fragile coupling. Too little visibility leads to code duplication and rigid architecture.

---

## Solution

**Use the most restrictive visibility modifier that allows necessary access, progressing from private (default) → `pub(super)` → `pub(crate)` → `pub` as needed.**

### Visibility Levels

| Modifier | Scope | Use Case |
|----------|-------|----------|
| (none) | Private to module | Implementation details, helper functions |
| `pub(super)` | Visible to parent module | Shared among sibling modules |
| `pub(crate)` | Visible within crate | Internal API, cross-module utilities |
| `pub` | Public API | Exported to library users |

### Implementation

From `fern_sim`, showing strategic visibility use:

**1. Private by default** (`spores.rs`):

```rust
//! Fern reproduction.

use cells::{Cell, Gene};

/// A cell made by an adult fern. It disperses on the wind as part of
/// the fern life cycle.
pub struct Spore {
    size: f64   // Field is private - implementation detail
}

/// Simulate the production of a spore by meiosis.
pub fn produce_spore(factory: &mut Sporangium) -> Spore {
    Spore { size: 1.0 }
}

/// Extract the genes in a particular spore.
///
/// VISIBILITY: pub(crate) - used by other modules in this crate,
/// but not part of the public API.
pub(crate) fn genes(spore: &Spore) -> Vec<Gene> {
    todo!()
}

/// Mix genes to prepare for meiosis (part of interphase).
///
/// VISIBILITY: private - only used within this module.
fn recombine(parent: &mut Cell) {
    todo!()
}

pub struct Sporangium;

// Nested private module
mod cells {
    //! The simulation of biological cells, which is as low-level as we go.

    // These types are used by parent module (spores.rs) through `use cells::{Cell, Gene};`
    // but not exposed outside this file.

    pub struct Cell {
        x: f64,
        y: f64
    }

    impl Cell {
        pub fn distance_from_origin(&self) -> f64 {
            f64::hypot(self.x, self.y)
        }
    }

    pub struct Gene;
}
```

**Analysis of visibility decisions**:

```rust
pub struct Spore { size: f64 }
//               └─ Private field: users can't construct or mutate directly
//                  Must use `produce_spore()` - enforces invariants

pub fn produce_spore(factory: &mut Sporangium) -> Spore { ... }
//  └─ Public: part of the crate's API for creating spores

pub(crate) fn genes(spore: &Spore) -> Vec<Gene> { ... }
//  └─ Crate-visible: other modules (e.g., simulation.rs) can use this,
//                    but library users cannot - it's an internal detail

fn recombine(parent: &mut Cell) { ... }
// └─ Private: implementation detail of spore production,
//             never needed outside this module

mod cells {
    pub struct Cell { ... }
    //  └─ pub within `cells` mod, but `cells` itself is private,
    //     so Cell is only visible to parent module (spores.rs)
}
```

**2. Cross-module visibility** (`plant_structures/mod.rs`):

```rust
pub mod roots;
pub mod stems;
pub mod leaves;

// Public re-exports - part of library API
pub use self::leaves::Leaf;
pub use self::roots::Root;

// Private use - implementation detail
use self::roots::RootSet;
use self::stems::StemSet;
//      └─ These are `pub` in their respective modules but not re-exported,
//         so they're effectively crate-internal
```

**3. Struct field visibility** (`simulation.rs`):

```rust
/// The simulated universe.
pub struct Terrarium {
    ferns: Vec<Fern>  // Private field - users can't directly mutate
}

impl Terrarium {
    pub fn new() -> Terrarium {
        Terrarium { ferns: vec![] }
    }

    /// Get a reference to a fern inside the simulation.
    pub fn fern(&self, index: usize) -> &Fern {
        &self.ferns[index]  // Controlled access to private field
    }

    pub fn apply_sunlight(&mut self, time: Duration) {
        for f in &mut self.ferns {  // Internal mutation only through public methods
            for s in &mut f.stems {
                s.furled = false;
            }
        }
    }
}
```

### Usage Patterns

**Pattern 1: Public API type with private fields**

```rust
pub struct Config {
    timeout: Duration,      // Private - enforce validation
    max_retries: u32,
}

impl Config {
    pub fn new() -> Self {
        Config {
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        assert!(timeout.as_secs() > 0, "Timeout must be positive");
        self.timeout = timeout;
        self
    }

    pub fn timeout(&self) -> Duration {
        self.timeout  // Getter for private field
    }
}
```

**Pattern 2: Crate-internal utilities**

```rust
// src/utils.rs
pub(crate) fn validate_identifier(s: &str) -> bool {
    s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

pub(crate) struct InternalCache {
    // Used across modules but not public API
}
```

**Pattern 3: Module-local helpers**

```rust
pub fn public_api_function(input: &str) -> Result<String> {
    let normalized = normalize_input(input);  // Private helper
    process(normalized)
}

fn normalize_input(s: &str) -> String {
    // Implementation detail - private by default
    s.trim().to_lowercase()
}
```

**Pattern 4: Parent-scoped items with `pub(super)`**

```rust
// src/network/tcp.rs
pub(super) struct TcpConnectionPool {
    // Visible to network module, not to entire crate
}

// src/network/mod.rs
mod tcp;
mod udp;

use tcp::TcpConnectionPool;  // OK: tcp is child module

// src/other_module.rs
// use crate::network::tcp::TcpConnectionPool;  // ERROR: not visible
```

### Guidelines

1. **Start private, widen as needed**
   - Default: no visibility modifier (private)
   - If needed by sibling: `pub(super)`
   - If needed by other modules: `pub(crate)`
   - If part of API: `pub`

2. **Private fields by default**
   ```rust
   pub struct MyType {
       field: T,  // Private unless you need direct access
   }
   ```
   Provide getters/setters for controlled access

3. **Use `pub(crate)` for internal APIs**
   - Helper functions shared across modules
   - Types used in multiple modules but not public
   - Testing utilities

4. **Document visibility decisions**
   ```rust
   /// Internal helper for parsing.
   ///
   /// VISIBILITY: pub(crate) because both the parser and validator
   /// modules need this, but it's not stable API.
   pub(crate) fn parse_helper(input: &str) -> Option<Node> {
       // ...
   }
   ```

5. **Make intent explicit in documentation**
   ```rust
   /// Public API for creating widgets.
   pub fn create_widget() -> Widget { ... }

   /// Internal function - do not use directly.
   #[doc(hidden)]  // Hide from documentation
   pub fn __internal_widget_hack() { ... }
   ```

---

## Resulting Context

### Benefits

- **Enforced boundaries**: Compiler prevents accidental cross-module dependencies
- **Safer refactoring**: Private code can change without breaking anything
- **Clear contracts**: Visibility indicates intended usage scope
- **Reduced API surface**: Documentation shows only relevant items
- **Semver compliance**: Changing internal items isn't a breaking change
- **Better IDE support**: Autocomplete suggests only accessible items

### Drawbacks

- **Learning curve**: Developers must understand visibility rules
  - *Mitigation*: Clear documentation and consistent patterns
- **Friction in prototyping**: May need to widen visibility during development
  - *Mitigation*: Start wide, narrow during cleanup phase
- **Testing complexity**: Private items harder to test from outside
  - *Mitigation*: Test through public API or use `#[cfg(test)]` visibility

### Invariants Maintained

- Public items (`pub`) are stable and documented
- Crate-internal items (`pub(crate)`) can change without semver bump
- Private items can be refactored or removed at will
- Nested module visibility is strictly controlled by parent

---

## Related Patterns

- **[Hierarchical Modules](hierarchical-modules.md)**: Visibility enforces the hierarchy's boundaries
- **[Re-exporting](re-exporting.md)**: Public re-exports create stable API; non-re-exported items remain internal
- **[Separation of Concerns](separation-of-concerns.md)**: Visibility prevents concerns from leaking across boundaries

---

## Known Uses

### From `fern_sim`

```rust
// Public API (visible to library users)
pub struct Spore { size: f64 }                // Type is public
pub fn produce_spore(...) -> Spore { ... }    // Function is public

// Crate-internal (visible to other modules)
pub(crate) fn genes(spore: &Spore) -> Vec<Gene> { ... }

// Private (only within module)
fn recombine(parent: &mut Cell) { ... }
mod cells { ... }  // Entire module private
```

### Rust Standard Library

```rust
// std::collections::HashMap uses visibility strategically
pub struct HashMap<K, V> {
    base: base::HashMap<K, V>,  // Private field
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self { ... }           // Public API

    pub(crate) fn resize(&mut self) { ... }  // Internal to std

    fn internal_capacity(&self) -> usize { ... }  // Private helper
}
```

### Serde

```rust
// serde_derive uses pub(crate) extensively for internal code generation helpers
pub(crate) fn derive_serialize(input: TokenStream) -> TokenStream {
    // Used by multiple derive modules but not public
}
```

---

## Examples from Real Projects

### Testing Internal Items

```rust
pub struct Parser {
    tokens: Vec<Token>,  // Private
}

impl Parser {
    #[cfg(test)]  // Only compiled in tests
    pub(crate) fn from_tokens(tokens: Vec<Token>) -> Self {
        Parser { tokens }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_internals() {
        let parser = Parser::from_tokens(vec![/* ... */]);
        // Can test with controlled token stream
    }
}
```

### Builder with Staged Visibility

```rust
mod builder {
    pub struct ConfigBuilder {
        timeout: Option<Duration>,
    }

    impl ConfigBuilder {
        pub(crate) fn new() -> Self {  // Crate-internal constructor
            ConfigBuilder { timeout: None }
        }

        pub fn timeout(mut self, timeout: Duration) -> Self {  // Public setter
            self.timeout = Some(timeout);
            self
        }

        pub fn build(self) -> Config {  // Public build
            Config {
                timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
            }
        }
    }

    pub struct Config {
        timeout: Duration,  // Private field
    }
}

// Usage:
use builder::ConfigBuilder;
let config = ConfigBuilder::new().timeout(Duration::from_secs(60)).build();
```

---

## Anti-Patterns to Avoid

### Everything Public

```rust
// ❌ BAD: No encapsulation
pub struct Parser {
    pub tokens: Vec<Token>,
    pub current_index: usize,
    pub errors: Vec<String>,
}
```

**Problem**: Users can violate invariants, refactoring is impossible.

**Fix**: Make fields private, provide controlled access:

```rust
// ✅ GOOD
pub struct Parser {
    tokens: Vec<Token>,
    current_index: usize,
    errors: Vec<String>,
}

impl Parser {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}
```

### Inconsistent Visibility

```rust
// ❌ BAD: Similar items have different visibility without clear reason
pub fn process_a(input: &str) -> Result<String> { ... }
fn process_b(input: &str) -> Result<String> { ... }  // Also needed publicly?
pub(crate) fn process_c(input: &str) -> Result<String> { ... }
```

**Fix**: Establish clear rules and follow them consistently.

### Overusing `pub(crate)`

```rust
// ❌ BAD: Making everything pub(crate) "just in case"
pub(crate) fn rarely_used_helper() { ... }
pub(crate) struct InternalDetail { ... }
```

**Fix**: Start private. Widen only when actually needed by other modules.

---

## Decision Matrix

| Situation | Visibility | Rationale |
|-----------|-----------|-----------|
| Public API type | `pub struct T { private: U }` | Encapsulation |
| Public API function | `pub fn f()` | Part of contract |
| Helper used by siblings | `pub(super) fn h()` | Shared in submodule group |
| Utility used across crate | `pub(crate) fn u()` | Internal API |
| Implementation detail | `fn detail()` | Default private |
| Test helper | `#[cfg(test)] pub(crate) fn test_h()` | Test-only visibility |

---

## Checklist

When deciding visibility:

- [ ] Is this part of the public API? → `pub`
- [ ] Do other modules in this crate need it? → `pub(crate)`
- [ ] Do sibling modules need it? → `pub(super)`
- [ ] Is it only used in this module? → (private)
- [ ] Are struct fields encapsulated? (private unless necessary)
- [ ] Is visibility documented in unclear cases?
- [ ] Will changing this break semver? (only if `pub`)

---

## Further Reading

- *Programming Rust*, Chapter 8: Modules and Visibility
- [Rust Book: Controlling Visibility with `pub`](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#exposing-paths-with-the-pub-keyword)
- [Rust Reference: Visibility and Privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html)
- [Rust API Guidelines: Future-proofing (C-STRUCT-PRIVATE)](https://rust-lang.github.io/api-guidelines/future-proofing.html#structs-have-private-fields-c-struct-private)
