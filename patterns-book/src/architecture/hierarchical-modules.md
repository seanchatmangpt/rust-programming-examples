# Hierarchical Modules

## Pattern Summary

Organize code in nested module trees that mirror the logical structure of your domain, using Rust's file system-based module hierarchy to enforce architectural boundaries and manage complexity.

---

## Context

You are building a medium to large Rust application with multiple conceptual layers or subsystems. The codebase has grown beyond a single file, and you need to organize related functionality together. You want to maintain clear boundaries between different parts of the system while making the overall structure intuitive to navigate.

Your domain has natural hierarchical relationships—for example, biological structures (plants have stems, stems have xylem and phloem), UI components (windows contain panels, panels contain widgets), or networking layers (application layer contains HTTP, HTTP contains headers and bodies).

---

## Problem

**How do you organize a complex codebase into manageable, logically-grouped modules that prevent tangled dependencies while remaining easy to navigate and understand?**

Flat module structures quickly become overwhelming. As your application grows, you face:
- **Namespace pollution**: Too many items in the top-level scope
- **Lost context**: Related functionality scattered across unrelated files
- **Dependency chaos**: Unclear relationships between modules
- **Cognitive overload**: Developers can't build mental models of the system

---

## Forces

- **Modularity**: Code should be broken into focused, single-responsibility units
- **Discoverability**: Developers should find related code intuitively
- **Encapsulation**: Implementation details should be hidden from unrelated code
- **Flexibility**: The structure should accommodate growth without major refactoring
- **Simplicity**: The module hierarchy shouldn't be deeper than necessary
- **Compilation performance**: Deep nesting can increase compilation times
- **Maintainability**: The structure should make it easy to change one part without affecting others

Too flat a structure leads to massive files and unclear relationships. Too deep a hierarchy creates navigation overhead and excessive indirection.

---

## Solution

**Organize your code into a tree of nested modules where each level represents a meaningful conceptual grouping, using Rust's file system mapping to make the structure explicit.**

### Structure

```
project/
└── src/
    ├── lib.rs                    # Root module, declares top-level modules
    ├── module_a/
    │   ├── mod.rs                # Module root, declares submodules
    │   ├── submodule_1.rs        # Focused submodule
    │   ├── submodule_2.rs
    │   └── submodule_3/
    │       ├── mod.rs            # Nested submodule root
    │       ├── detail_a.rs
    │       └── detail_b.rs
    └── module_b/
        ├── mod.rs
        └── implementation.rs
```

### Implementation

From `fern_sim`, a biological simulation with hierarchical structure:

**Root module (`lib.rs`)** - Declares top-level modules:

```rust
//! Simulate the growth of ferns, from the level of
//! individual cells on up.

pub mod plant_structures;
pub mod simulation;
pub mod spores;

// Re-export key types for convenience
pub use plant_structures::Fern;
pub use simulation::Terrarium;
```

**First-level module (`plant_structures/mod.rs`)** - Organizes major subsystems:

```rust
//! Higher-level biological structures.
//!
//! We always simulate a sample of all chemical interactions at the cellular
//! level, but simulating everything that way is just too computationally
//! expensive. Therefore we keep higher-level data structures representing
//! each fern's roots, leaves, and so on.

pub mod roots;
pub mod stems;
pub mod leaves;

// Re-export commonly used types
pub use self::leaves::Leaf;
pub use self::roots::Root;

// Private use - only for this module
use self::roots::RootSet;
use self::stems::StemSet;

pub enum FernType {
    Fiddlehead
}

pub struct Fern {
    pub roots: RootSet,
    pub stems: StemSet
}

impl Fern {
    pub fn new(_type: FernType) -> Fern {
        Fern {
            roots: vec![],
            stems: vec![stems::Stem { furled: true }]
        }
    }
}
```

**Second-level module (`plant_structures/stems.rs`)** - Contains further specialization:

```rust
//! Stems hold the weight of the plant and are largely responsible for its
//! shape. Parameters on `Stem` (not `Leaf`) are responsible for pinnation,
//! the feathery leaf structure that's the most immediately recognizable
//! property of ferns.

pub mod xylem;
pub mod phloem;

pub struct Stem {
    pub furled: bool
}

pub type StemSet = Vec<Stem>;
```

**Third-level module (`plant_structures/stems/phloem.rs`)** - Leaf-level implementation:

```rust
//! Structures for distributing the products of photosynthesis.

/// Tissue for translocating sucrose and other photosynthesis products.
pub struct Phloem {
    pub flow_rate: f32,
}
```

### Usage

External code imports from the logical hierarchy:

```rust
use fern_sim::Terrarium;                    // Top-level re-export
use fern_sim::plant_structures::Fern;       // Direct module path
use fern_sim::plant_structures::leaves::Leaf;  // Nested path

fn main() {
    let mut terrarium = Terrarium::new();
    let fern = Fern::new(FernType::Fiddlehead);
}
```

Internal code uses relative imports:

```rust
// In plant_structures/mod.rs
use self::stems::StemSet;         // Submodule in same parent
use crate::simulation::Terrarium; // From crate root
```

### Guidelines

1. **One concept per module**: Each module should represent a single conceptual grouping
2. **Depth reflects abstraction**: Deeper modules contain more specific implementation details
3. **Limit depth to 3-4 levels**: Beyond this, consider if you're over-organizing
4. **Use mod.rs for aggregation**: Place shared types and re-exports in `mod.rs`, implementations in sibling files
5. **Mirror domain structure**: If your domain has natural hierarchies (organism → organ → tissue → cell), reflect this in modules
6. **Keep siblings cohesive**: Modules at the same level should be roughly equivalent in scope and abstraction

---

## Resulting Context

### Benefits

- **Clear architecture**: The file system reveals the logical structure at a glance
- **Namespace management**: Each module provides a scope, preventing name collisions
- **Encapsulation boundaries**: Module privacy enforces architectural decisions
- **Gradual disclosure**: Developers navigate from general (top) to specific (deep)
- **Independent development**: Teams can work on different branches of the tree
- **Testability**: Each module can be tested in isolation

### Drawbacks

- **Deep paths**: Importing from nested modules requires longer paths
  - *Mitigation*: Use re-exports at higher levels for commonly used items
- **File navigation**: Deep hierarchies require more clicking/navigation
  - *Mitigation*: Good IDE support (jump to definition, search by symbol)
- **Initial overhead**: Setting up the structure takes planning
  - *Mitigation*: Start simple, refactor as patterns emerge
- **Compilation time**: Very deep trees can slow incremental compilation
  - *Mitigation*: Keep depth reasonable (3-4 levels maximum)

### Invariants Maintained

- Module boundaries align with conceptual boundaries
- No circular dependencies between modules (enforced by Rust)
- Public API surface is controlled through visibility modifiers
- Changes to implementation details don't require changes to importers (if re-exports used)

---

## Related Patterns

- **[Re-exporting](re-exporting.md)**: Complement this pattern by selectively re-exporting items to simplify imports
- **[Visibility Boundaries](visibility-boundaries.md)**: Use `pub(crate)` and `pub(super)` to fine-tune encapsulation within hierarchies
- **[Separation of Concerns](separation-of-concerns.md)**: Different modules represent different concerns
- **[Library-Binary Split](library-binary-split.md)**: The library often uses hierarchical modules, while binaries remain flat

---

## Known Uses

### From `fern_sim`

```
fern_sim/src/
├── lib.rs
├── plant_structures/
│   ├── mod.rs              # Re-exports Leaf, Root; defines Fern
│   ├── leaves.rs           # Leaf structure
│   ├── roots.rs            # Root structure
│   └── stems/
│       ├── mod.rs          # Defines Stem, StemSet
│       ├── phloem.rs       # Phloem tissue
│       └── xylem.rs        # Xylem tissue
├── simulation.rs           # Terrarium simulation
└── spores.rs               # Reproduction logic
```

**Why it works**: The hierarchy mirrors biology (plant → structures → stems → tissue types), making it intuitive for domain experts and developers alike.

### Rust Standard Library

```
std::
├── collections/
│   ├── hash_map.rs
│   ├── vec_deque.rs
│   └── btree/
│       ├── map.rs
│       └── set.rs
├── io/
│   ├── mod.rs
│   ├── buffered.rs
│   └── stdio.rs
└── net/
    ├── tcp.rs
    └── udp.rs
```

### Actix-web Framework

```
actix_web/
├── app/
├── middleware/
├── handler/
└── dev/
    ├── mod.rs
    └── extensions.rs
```

---

## Examples from Real Projects

### Compiler Structure (rustc-style)

```
compiler/
├── frontend/
│   ├── parser/
│   │   ├── lexer.rs
│   │   └── syntax.rs
│   └── ast/
├── middle/
│   ├── ty/
│   └── mir/
└── backend/
    ├── codegen/
    └── link/
```

### Web Application

```
web_app/
├── api/
│   ├── users/
│   ├── posts/
│   └── auth/
├── db/
│   ├── models/
│   └── migrations/
└── services/
    ├── email/
    └── storage/
```

---

## Variations

### Flat Sibling Modules

When subsystems are independent and equally important:

```
src/
├── lib.rs
├── users.rs
├── orders.rs
└── inventory.rs
```

Use when there's no clear hierarchy—just separate concerns.

### Feature-Based Organization

Group by feature instead of technical layer:

```
src/
├── user_management/
│   ├── models.rs
│   ├── handlers.rs
│   └── db.rs
└── order_processing/
    ├── models.rs
    ├── handlers.rs
    └── db.rs
```

Use when features evolve independently.

---

## Decision Criteria

**Use Hierarchical Modules when**:
- Your domain has natural hierarchical relationships
- You have more than 5-6 top-level modules
- Different subsystems have different visibility requirements
- You want to enforce architectural boundaries through the module system

**Consider alternatives when**:
- Your application is small (< 1000 lines)
- All modules are truly peers with no hierarchical relationship
- You're building a simple library with a narrow focus

---

## Further Reading

- *Programming Rust* (Blandy, Orendorff, Tindall), Chapter 8: Modules and Packages
- Rust Book: [Managing Growing Projects with Packages, Crates, and Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [Rust API Guidelines: Module Organization](https://rust-lang.github.io/api-guidelines/naming.html)
