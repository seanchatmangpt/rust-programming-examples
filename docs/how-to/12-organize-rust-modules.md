# How to Organize Rust Modules

## Overview

This guide shows you how to structure a Rust project with modules, using the fern simulation project as a concrete example. You'll learn how to organize code into logical units, control visibility, and design clean public APIs.

## Prerequisites

- Basic Rust syntax knowledge
- Understanding of project structure (src/ directory)
- Familiarity with `pub` keyword

## Why Modules?

Modules help you:
- **Organize code** into logical units
- **Control visibility** with public/private boundaries
- **Manage namespaces** to avoid naming conflicts
- **Design APIs** by choosing what to expose
- **Enable separate compilation** (faster builds)

This is similar to Python's package system, but with compile-time enforcement.

## Project Structure Overview

The fern_sim project has this structure:

```
fern_sim/
├── Cargo.toml
└── src/
    ├── lib.rs                          # Crate root
    ├── spores.rs                       # Single-file module
    ├── simulation.rs                   # Single-file module
    ├── net.rs                          # Single-file module
    └── plant_structures/               # Directory module
        ├── mod.rs                      # Module root
        ├── roots.rs                    # Submodule
        ├── leaves.rs                   # Submodule
        └── stems/                      # Nested module
            ├── mod.rs                  # Stems module root
            ├── xylem.rs                # Submodule
            └── phloem.rs               # Submodule
```

## Step 1: The Crate Root (`lib.rs`)

Every library starts with `lib.rs`, the crate root:

```rust
//! Simulate the growth of ferns, from the level of
//! individual cells on up.

// Declare which modules exist
pub mod plant_structures;
pub mod simulation;
pub mod spores;
pub mod net;

// Re-export important types for convenience
pub use plant_structures::Fern;
pub use simulation::Terrarium;
pub use net::connect;
```

Key points:
- `pub mod name;` declares and makes a module public
- `mod name;` declares a private module
- `pub use` re-exports items for easier access

**Result:** Users can write:
```rust
use fern_sim::Fern;              // Instead of fern_sim::plant_structures::Fern
use fern_sim::Terrarium;         // Instead of fern_sim::simulation::Terrarium
let session = fern_sim::connect(); // Instead of fern_sim::net::connect()
```

## Step 2: Single-File Modules

For simple modules, create a single `.rs` file:

```rust
// src/net.rs
pub struct Session;

pub fn connect() -> Session {
    Session
}

impl Session {
    /// Upload all local terrariums to the online gallery.
    pub fn upload_all(&mut self) {
        unimplemented!();
    }
}
```

To use this module:
```rust
// In lib.rs
pub mod net;
pub use net::connect;  // Re-export for convenience

// Users can now:
use fern_sim::connect;
let session = connect();
```

## Step 3: Directory Modules with `mod.rs`

For complex modules with submodules, create a directory with `mod.rs`:

```rust
// src/plant_structures/mod.rs
//! Higher-level biological structures.

// Declare submodules (files in this directory)
pub mod roots;
pub mod stems;
pub mod leaves;

// Re-export commonly used types
pub use self::leaves::Leaf;
pub use self::roots::Root;

// Import for internal use (not re-exported)
use self::roots::RootSet;
use self::stems::StemSet;

// Types defined in this module
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

    pub fn is_fully_unfurled(&self) -> bool {
        self.stems.iter().all(|s| !s.furled)
    }
}
```

The submodule files:

```rust
// src/plant_structures/roots.rs
pub struct Root {
    pub x: bool
}

pub type RootSet = Vec<Root>;
```

```rust
// src/plant_structures/leaves.rs
pub struct Leaf {
    pub x: bool
}
```

## Step 4: Nested Modules

Modules can contain modules:

```rust
// src/plant_structures/stems.rs
//! Stems hold the weight of the plant.

// Declare submodules
pub mod xylem;
pub mod phloem;

pub struct Stem {
    pub furled: bool
}

pub type StemSet = Vec<Stem>;
```

For this to work, create:
- `src/plant_structures/stems/xylem.rs`
- `src/plant_structures/stems/phloem.rs`

**Alternative:** You could create `stems/mod.rs` instead of `stems.rs` to hold the module content, but `stems.rs` is more concise when you have module code at this level.

## Step 5: Understanding Visibility

Rust has fine-grained visibility control:

```rust
// src/spores.rs
use cells::{Cell, Gene};

/// Public: Available to all users of the crate
pub struct Spore {
    size: f64  // Private field: only this module can access
}

/// Public function
pub fn produce_spore(factory: &mut Sporangium) -> Spore {
    Spore { size: 1.0 }
}

/// Crate-visible: Only code in fern_sim can call this
pub(crate) fn genes(spore: &Spore) -> Vec<Gene> {
    todo!()
}

/// Private: Only this module can call this
fn recombine(parent: &mut Cell) {
    todo!()
}

pub struct Sporangium;

// Private submodule: Only spores.rs can use this
mod cells {
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

Visibility levels:
- `pub` - Public to everyone
- `pub(crate)` - Public within the crate only
- `pub(super)` - Public to parent module only
- `pub(in path)` - Public to specific ancestor module
- (no `pub`) - Private to current module

## Step 6: Re-exports for API Design

Choose what users see by re-exporting strategically:

```rust
// src/lib.rs
pub mod plant_structures;  // Expose entire module

// Re-export just the important types
pub use plant_structures::Fern;
pub use plant_structures::Leaf;

// Don't re-export internal types like RootSet, StemSet

// Re-export from nested modules
pub use net::connect;
```

This creates a clean API:

```rust
// ✅ Clean API - users see:
use fern_sim::{Fern, Leaf, connect};

// ❌ Without re-exports, users would need:
use fern_sim::plant_structures::Fern;
use fern_sim::plant_structures::leaves::Leaf;
use fern_sim::net::connect;
```

## Step 7: Module Paths

Reference items within your crate:

```rust
// Absolute path from crate root
use crate::plant_structures::Fern;

// Relative path from current module
use self::leaves::Leaf;        // Child module
use super::simulation::run;    // Parent's sibling

// From current module
use leaves::Leaf;              // If leaves is declared in current module
```

**When to use each:**
- `crate::` - Unambiguous, always clear where item is from
- `self::` - Explicit that it's in current module
- `super::` - Navigate up the tree
- Direct name - For immediate children only

## Step 8: Inline Modules

For very small submodules, declare them inline:

```rust
// src/spores.rs
pub struct Spore {
    size: f64
}

// Inline module - not in a separate file
mod cells {
    pub struct Cell {
        x: f64,
        y: f64
    }

    impl Cell {
        pub fn distance_from_origin(&self) -> f64 {
            f64::hypot(self.x, self.y)
        }
    }
}

// Use items from inline module
use cells::Cell;
```

Use inline modules for:
- Private helper code
- Test modules (`#[cfg(test)] mod tests { ... }`)
- Very small, tightly coupled code

## Complete Module Organization Example

Here's how it all fits together:

```rust
// src/lib.rs
//! Public API for fern_sim

pub mod plant_structures;  // Complex module (directory)
pub mod simulation;        // Simple module (file)
pub mod spores;           // Module with inline submodule
pub mod net;              // Simple module

// Re-exports for convenience
pub use plant_structures::Fern;
pub use simulation::Terrarium;
pub use net::connect;
```

```rust
// src/plant_structures/mod.rs
pub mod roots;
pub mod stems;
pub mod leaves;

// Re-export public API
pub use self::leaves::Leaf;
pub use self::roots::Root;

// Private imports for internal use
use self::roots::RootSet;
use self::stems::StemSet;

pub struct Fern {
    pub roots: RootSet,
    pub stems: StemSet
}
```

```rust
// src/plant_structures/stems.rs
pub mod xylem;
pub mod phloem;

pub struct Stem {
    pub furled: bool
}

pub type StemSet = Vec<Stem>;
```

**Usage:**
```rust
// External user of the crate
use fern_sim::Fern;
use fern_sim::plant_structures::FernType;

let fern = Fern::new(FernType::Fiddlehead);
```

## Comparison to Python Packages

| Feature | Python | Rust |
|---------|--------|------|
| File is module | `foo.py` | `foo.rs` |
| Directory as module | `foo/__init__.py` | `foo/mod.rs` or `foo.rs` + `foo/` |
| Import syntax | `import foo` | `use foo;` (after `mod foo;`) |
| Declare modules | Implicit (files exist) | Explicit (`mod foo;`) |
| Re-exports | `from .foo import Bar` in `__init__.py` | `pub use foo::Bar;` |
| Visibility | Everything public by default | Everything private by default |
| Relative imports | `from . import sibling` | `use super::sibling;` |
| Namespace access | `foo.bar.baz` | `foo::bar::baz` |
| Private prefix | `_private` by convention | No `pub` (enforced) |

**Python example:**
```python
# fern_sim/__init__.py
from .plant_structures import Fern
from .simulation import Terrarium
from .net import connect

# fern_sim/plant_structures/__init__.py
from .roots import Root
from .leaves import Leaf
from .stems import Stem

class Fern:
    def __init__(self, fern_type):
        self.roots = []
        self.stems = []
```

**Rust equivalent:**
```rust
// src/lib.rs
pub mod plant_structures;
pub mod simulation;
pub mod net;

pub use plant_structures::Fern;
pub use simulation::Terrarium;
pub use net::connect;

// src/plant_structures/mod.rs
pub mod roots;
pub mod leaves;
pub mod stems;

pub use roots::Root;
pub use leaves::Leaf;
pub use stems::Stem;

pub struct Fern {
    pub roots: Vec<Root>,
    pub stems: Vec<Stem>
}
```

Key differences:
- Python: Everything is public unless prefixed with `_`
- Rust: Everything is private unless marked `pub`
- Python: Imports are runtime, can fail at runtime
- Rust: Modules are compile-time, checked by compiler
- Python: Circular imports can cause issues
- Rust: Circular dependencies prevented at compile time

## Patterns and Best Practices

### Pattern 1: Facade Pattern

Create a simple top-level API that hides complexity:

```rust
// src/lib.rs - Simple API
pub use internals::perform_complex_operation;

mod internals {
    pub fn perform_complex_operation() {
        complex_step_1();
        complex_step_2();
    }

    fn complex_step_1() { /* ... */ }
    fn complex_step_2() { /* ... */ }
}
```

### Pattern 2: Prelude Pattern

Provide a prelude module with commonly used imports:

```rust
// src/prelude.rs
pub use crate::Fern;
pub use crate::Terrarium;
pub use crate::plant_structures::{Leaf, Root};

// Users can then:
use fern_sim::prelude::*;
```

### Pattern 3: Flat Crate Structure

Re-export everything at the root for simple APIs:

```rust
// src/lib.rs
mod internal_module;

pub use internal_module::{TypeA, TypeB, TypeC};

// Users see a flat API:
use my_crate::{TypeA, TypeB, TypeC};
```

### Pattern 4: Hierarchical Structure

Keep modules organized by domain:

```rust
// src/lib.rs
pub mod database;
pub mod api;
pub mod models;
pub mod utils;

// Users navigate the hierarchy:
use my_app::database::connect;
use my_app::models::User;
use my_app::api::routes;
```

## Common Module Pitfalls

### Pitfall 1: Forgetting `mod` Declaration

```rust
// ❌ WRONG: File exists but not declared
// src/lib.rs
// (nothing)

// src/utils.rs
pub fn helper() {}

// Error: Can't find `utils`
use crate::utils::helper;

// ✅ FIX: Declare the module
// src/lib.rs
mod utils;  // or pub mod utils;
```

### Pitfall 2: Circular Dependencies

```rust
// ❌ WRONG: Circular module dependencies
// src/a.rs
use crate::b::TypeB;
pub struct TypeA { b: TypeB }

// src/b.rs
use crate::a::TypeA;
pub struct TypeB { a: TypeA }

// Error: Cycle detected

// ✅ FIX: Use a third module for shared types
// src/types.rs
pub struct TypeA { b_data: BData }
pub struct TypeB { a_data: AData }
```

### Pitfall 3: Exposing Implementation Details

```rust
// ❌ WRONG: Leaking internal structure
pub mod internals {
    pub struct InternalType;
}

// Users can depend on internals:
use your_crate::internals::InternalType;

// ✅ BETTER: Keep internals private
mod internals {
    pub(crate) struct InternalType;
}

// Only expose what users need
pub struct PublicApi;
```

### Pitfall 4: Deep Nesting

```rust
// ❌ WRONG: Too deeply nested
use my_crate::level1::level2::level3::level4::UsefulType;

// ✅ BETTER: Flatten with re-exports
// src/lib.rs
pub use level1::level2::level3::level4::UsefulType;

// Now:
use my_crate::UsefulType;
```

## Testing Module Organization

Test your module boundaries:

```rust
// src/plant_structures/mod.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_fern() {
        let fern = Fern::new(FernType::Fiddlehead);
        assert!(!fern.is_fully_unfurled());
    }

    #[test]
    fn can_access_public_types() {
        let _leaf = Leaf { x: true };
        let _root = Root { x: true };
    }
}

// Integration tests (tests/ directory)
// tests/integration_test.rs
use fern_sim::{Fern, connect};

#[test]
fn full_workflow() {
    let fern = Fern::new(fern_sim::plant_structures::FernType::Fiddlehead);
    let session = connect();
    // Test public API as users would use it
}
```

## Module Documentation

Document your module structure:

```rust
//! The `plant_structures` module contains biological models.
//!
//! This module provides types for representing ferns at different
//! levels of abstraction:
//!
//! - [`Fern`] - The top-level plant structure
//! - [`Root`] - Underground structures
//! - [`Leaf`] - Photosynthetic structures
//! - [`Stem`] - Support structures
//!
//! # Examples
//!
//! ```
//! use fern_sim::plant_structures::{Fern, FernType};
//!
//! let fern = Fern::new(FernType::Fiddlehead);
//! assert!(fern.is_furled());
//! ```

pub mod roots;
pub mod stems;
pub mod leaves;

pub use self::leaves::Leaf;
pub use self::roots::Root;
```

Use:
- `//!` for module-level docs
- `///` for item-level docs
- Examples, links, and sections

## Organizing Large Projects

For large projects, consider:

1. **Workspace structure**: Multiple related crates
2. **Feature flags**: Optional modules
3. **Separate bins**: Multiple binaries in one project

```toml
# Cargo.toml for workspace
[workspace]
members = [
    "fern_sim_core",
    "fern_sim_cli",
    "fern_sim_gui"
]

# Individual crate with features
[package]
name = "fern_sim"

[features]
default = ["basic"]
basic = []
advanced = ["complex_physics"]
visualization = ["graphics"]
```

## Next Steps

- Learn about [workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) for multi-crate projects
- Study [API guidelines](https://rust-lang.github.io/api-guidelines/) for public APIs
- Read about [feature flags](https://doc.rust-lang.org/cargo/reference/features.html) for optional functionality

## Related Examples

- `/home/user/rust-programming-examples/fern_sim/src/lib.rs` - Crate root with re-exports
- `/home/user/rust-programming-examples/fern_sim/src/plant_structures/mod.rs` - Directory module
- `/home/user/rust-programming-examples/fern_sim/src/spores.rs` - Module with inline submodule
- `/home/user/rust-programming-examples/fern_sim/src/plant_structures/stems.rs` - Nested module structure
