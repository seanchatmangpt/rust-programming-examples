# Re-exporting

## Pattern Summary

Use `pub use` declarations to re-export types and functions from nested modules at higher levels in the module hierarchy, creating a clean, stable public API that shields users from internal organization.

---

## Context

You are building a Rust library with a hierarchical module structure. Your implementation details are organized across multiple nested modules for maintainability, but you want to provide users with a simpler, more convenient API. Users shouldn't need to know or care about your internal module organization—they should be able to import the most commonly used items from a predictable location.

You have the freedom to reorganize your internal structure without breaking downstream code.

---

## Problem

**How do you provide a clean, stable public API while maintaining the freedom to reorganize internal implementation details across multiple modules?**

Direct exposure of your module hierarchy creates problems:
- **Fragile API**: Moving a type to a different module breaks all importers
- **Cognitive burden**: Users must learn your internal organization
- **Long import paths**: Deep nesting creates verbose imports
- **Leaky abstraction**: Internal structure becomes part of your public contract
- **Inflexible evolution**: Refactoring requires coordinating with all dependents

---

## Forces

- **Stability**: Public APIs should remain stable across versions
- **Convenience**: Common operations should be easy to access
- **Discoverability**: Users should find what they need without documentation archaeology
- **Flexibility**: Internal reorganization shouldn't break user code
- **Clarity**: The public API should reflect the conceptual model, not implementation details
- **Maintainability**: You want to reorganize code without waiting for major version bumps
- **Gradual disclosure**: Advanced users can access detailed modules; beginners see simplified API

Too many re-exports flatten the namespace and hide useful structure. Too few re-exports force users to navigate deep hierarchies and couple them to implementation details.

---

## Solution

**Selectively re-export key types and functions from nested modules using `pub use` declarations at higher levels in the module tree, creating "convenience layers" that present a simplified view of your API.**

### Structure

```rust
// lib.rs - Root module
pub mod internal_detail;
pub mod subsystem;

// Re-export the most commonly used items
pub use subsystem::ImportantType;
pub use subsystem::nested::FrequentlyUsedFunction;

// Users import from here
// use my_crate::ImportantType;  ✓ Convenient

// Instead of from deep paths
// use my_crate::subsystem::nested::FrequentlyUsedFunction;  ✗ Burdensome
```

### Implementation

From `fern_sim`, a biological simulation library:

**Root module (`lib.rs`)** - Re-exports key types:

```rust
//! Simulate the growth of ferns, from the level of
//! individual cells on up.

// Declare all modules
pub mod plant_structures;
pub mod simulation;
pub mod spores;
pub mod net;

// Re-export the primary types users need
pub use plant_structures::Fern;
pub use simulation::Terrarium;
pub use net::connect;

// Result: Users can write:
//   use fern_sim::{Fern, Terrarium};
//
// Instead of:
//   use fern_sim::plant_structures::Fern;
//   use fern_sim::simulation::Terrarium;
```

**Mid-level module (`plant_structures/mod.rs`)** - Re-exports from nested modules:

```rust
//! Higher-level biological structures.

// Declare submodules
pub mod roots;
pub mod stems;
pub mod leaves;

// Re-export commonly used types
pub use self::leaves::Leaf;
pub use self::roots::Root;

// Keep implementation details private
use self::roots::RootSet;    // Not pub use - internal only
use self::stems::StemSet;

// Result: Users can write:
//   use fern_sim::plant_structures::{Leaf, Root};
//
// Instead of:
//   use fern_sim::plant_structures::leaves::Leaf;
//   use fern_sim::plant_structures::roots::Root;
//
// And RootSet/StemSet remain internal implementation details
```

**Usage patterns** users can choose from:

```rust
// Pattern 1: Top-level convenience (most common use case)
use fern_sim::{Fern, Terrarium};

fn main() {
    let mut terrarium = Terrarium::new();
    // Fern is available directly
}

// Pattern 2: Module-level import (when you need several related items)
use fern_sim::plant_structures::{Fern, Leaf, Root};

fn analyze_plant(fern: &Fern) {
    // All plant structures available with short names
}

// Pattern 3: Deep import (when you need implementation details)
use fern_sim::plant_structures::stems::phloem::Phloem;

fn advanced_analysis(phloem: &Phloem) {
    // Access to internal structures still possible
}

// Pattern 4: Namespace prefix (when names might conflict)
use fern_sim::plant_structures;

fn process() {
    let leaf = plant_structures::Leaf { x: true };
}
```

### Guidelines

1. **Re-export primary API types** at the crate root (`lib.rs`)
   - Core data structures users interact with directly
   - Main entry points and functions
   - Commonly used traits

2. **Re-export related types** at module boundaries
   - Group conceptually related items
   - Balance convenience vs. namespace clutter

3. **Keep implementation details private**
   - Use non-`pub` `use` for internal imports
   - Don't re-export every type—only the public API

4. **Document the canonical path**
   ```rust
   /// A leaf structure.
   ///
   /// Also available as [`fern_sim::plant_structures::Leaf`].
   pub struct Leaf { /* ... */ }
   ```

5. **Use `pub use self::` for clarity in mod.rs**
   ```rust
   // Explicit about re-exporting from submodule
   pub use self::leaves::Leaf;

   // vs. importing from crate root (less clear)
   pub use crate::plant_structures::leaves::Leaf;
   ```

6. **Maintain stability**: Once re-exported, keep it re-exported
   - Removing a re-export is a breaking change
   - Deprecate before removing if necessary

---

## Resulting Context

### Benefits

- **Simplified imports**: Users access common types with minimal path typing
- **API stability**: Internal refactoring doesn't break user code
  - You can move `Leaf` from `plant_structures/leaves.rs` to `plant_structures/leaf_detail/leaf.rs` without affecting users who import `fern_sim::plant_structures::Leaf`
- **Gradual disclosure**: Beginners use top-level re-exports; experts navigate deeper
- **Namespace organization**: You control what's visible at each level
- **Documentation clarity**: rustdoc shows items at multiple levels, with clear "Re-export" labels

### Drawbacks

- **Duplicate visibility**: Items appear in multiple places in documentation
  - *Mitigation*: rustdoc clearly marks re-exports; use doc comments to indicate canonical location
- **Import confusion**: Users may not know which path to use
  - *Mitigation*: Document recommended import style in crate-level docs
- **Namespace pollution**: Over-re-exporting clutters the top level
  - *Mitigation*: Only re-export truly common items (if in doubt, leave it nested)
- **Maintenance overhead**: Re-exports must be kept in sync
  - *Mitigation*: Automated testing and CI checks for missing re-exports

### Invariants Maintained

- Public re-exports are stable across non-breaking versions
- Re-exported items remain accessible at their original paths
- Internal-only items use non-`pub` imports
- Documentation references canonical paths when multiple exist

---

## Related Patterns

- **[Hierarchical Modules](hierarchical-modules.md)**: Re-exporting complements deep module hierarchies by providing shortcuts
- **[Visibility Boundaries](visibility-boundaries.md)**: Re-exports are public; internal imports are `pub(crate)` or private
- **[Separation of Concerns](separation-of-concerns.md)**: Re-exports can unify concerns split across modules

---

## Known Uses

### From `fern_sim`

```rust
// lib.rs
pub use plant_structures::Fern;     // Primary type
pub use simulation::Terrarium;       // Core API

// plant_structures/mod.rs
pub use self::leaves::Leaf;          // Commonly used structure
pub use self::roots::Root;

// Result: Three levels of access
// 1. fern_sim::Fern                 (most convenient)
// 2. fern_sim::plant_structures::Leaf  (grouped by domain)
// 3. fern_sim::plant_structures::leaves::Leaf  (full path, always available)
```

### Rust Standard Library

The standard library uses re-exporting extensively:

```rust
// std::lib.rs (simplified)
pub use core::option::Option::{self, Some, None};
pub use core::result::Result::{self, Ok, Err};

// Result: Users write
use std::option::Option;  // ✓ Clear, at root

// Instead of
use std::core::option::Option;  // ✗ Exposes internal organization
```

### Tokio Runtime

```rust
// tokio/lib.rs
pub use crate::runtime::Runtime;
pub use crate::task::spawn;

// Users write:
use tokio::{Runtime, spawn};  // ✓

// Instead of:
use tokio::runtime::Runtime;  // ✗ More verbose
use tokio::task::spawn;
```

### Serde

```rust
// serde/lib.rs
pub use ser::{Serialize, Serializer};
pub use de::{Deserialize, Deserializer};

// Users write:
use serde::{Serialize, Deserialize};  // ✓ The most common import
```

---

## Examples from Real Projects

### Prelude Pattern

A common variation: Create a `prelude` module with re-exports of the most common items:

```rust
// lib.rs
pub mod prelude {
    pub use crate::Fern;
    pub use crate::Terrarium;
    pub use crate::plant_structures::{Leaf, Root};
}

// Users import with a glob:
use fern_sim::prelude::*;

// All common types now available
```

**When to use preludes**:
- You have 5-10 types users almost always need together
- The crate is used as a framework (like `tokio::prelude`)
- Convention in your domain (game engines often use this)

**When to avoid preludes**:
- Small libraries with few exports
- When glob imports are discouraged in your project
- When name conflicts are likely

### Facade Pattern

Re-export everything from implementation modules to create a facade:

```rust
// lib.rs
mod internal_impl;

// Expose entire internal API through facade
pub use internal_impl::*;

// Allows reorganizing internal_impl without breaking users
```

Use when you want complete control over internal organization but a simple external API.

### Versioned Re-exports

For major refactorings with backwards compatibility:

```rust
// v2 module structure
pub mod new_location {
    pub struct ImportantType { /* ... */ }
}

// v1 compatibility re-export
#[deprecated(since = "2.0.0", note = "Use `new_location::ImportantType` instead")]
pub use new_location::ImportantType as OldName;
```

---

## Anti-Patterns to Avoid

### Over-re-exporting

```rust
// ❌ BAD: Re-exporting everything clutters the namespace
pub use implementation::detail::*;
pub use module_a::*;
pub use module_b::*;

// Result: Namespace pollution, unclear what's actually public
```

**Fix**: Re-export only common, primary API types.

### Under-re-exporting

```rust
// ❌ BAD: No re-exports forces users through deep paths
pub mod deeply {
    pub mod nested {
        pub mod module {
            pub struct ImportantType;
        }
    }
}

// Users must write:
// use crate::deeply::nested::module::ImportantType;
```

**Fix**: Re-export `ImportantType` at a higher level if it's commonly used.

### Inconsistent Re-exporting

```rust
// ❌ BAD: Some types re-exported, others not, with no clear reason
pub use module_a::TypeA;  // Re-exported
// pub use module_b::TypeB;  // Not re-exported (equally important)
```

**Fix**: Establish clear criteria (e.g., "re-export all primary data structures").

---

## Decision Criteria

**Use re-exporting when**:
- You have a hierarchical module structure with commonly used types buried deep
- You want to decouple your public API from internal organization
- You're building a library (not just a binary)
- You expect to reorganize internal modules over time

**Avoid re-exporting when**:
- Your crate is small with a flat structure (< 5 modules)
- All types are equally important and specialized
- Users need to understand module organization (rare)
- You're writing internal application code, not a library

---

## Checklist

When adding re-exports, verify:

- [ ] Re-exported items are documented at their original location
- [ ] rustdoc shows clear "Re-export" labels
- [ ] Users know the canonical path (document in module-level docs)
- [ ] Commonly used types are re-exported at the crate root
- [ ] Related types are re-exported together at module boundaries
- [ ] Implementation details remain private (no accidental re-exports)
- [ ] Re-exports are tested (imports in integration tests or examples)

---

## Further Reading

- *Programming Rust*, Chapter 8: Modules and Crates
- [Rust API Guidelines: Flexibility (C-REEXPORT)](https://rust-lang.github.io/api-guidelines/flexibility.html#public-dependencies-of-a-stable-crate-are-stable-c-stable)
- [Rust Book: Controlling Visibility with `pub`](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#exposing-paths-with-the-pub-keyword)
