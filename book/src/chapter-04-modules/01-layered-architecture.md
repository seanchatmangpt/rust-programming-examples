# Layered Architecture with Modules

Module organization in Rust is not just about code organization—it's about expressing architectural intent. Unlike file-based module systems where the directory structure dictates the module tree, Rust requires explicit declaration of every module and its visibility. This design choice forces architects to think deliberately about boundaries, dependencies, and the public API surface.

## Module Hierarchy as Architectural Blueprint

The fundamental principle of module architecture is that **the module tree should mirror your domain structure, not your implementation details**. Consider the `fern_sim` project:

```
fern_sim/
├── lib.rs              # Public API surface
├── simulation.rs       # Top-level simulation control
├── spores.rs          # Reproduction mechanics
├── net.rs             # Network integration
└── plant_structures/  # Hierarchical submodules
    ├── mod.rs         # Plant components
    ├── roots.rs
    ├── leaves.rs
    └── stems/         # Nested modules
        ├── mod.rs
        ├── xylem.rs
        └── phloem.rs
```

This structure is intentional: ferns have plant structures, plant structures have stems, and stems have vascular tissue (xylem and phloem). The code structure reflects biological reality, making the codebase navigable by domain experts who may not be Rust experts.

### Declaration vs. File Location

In `plant_structures/mod.rs`, modules are declared explicitly:

```rust
// in plant_structures/mod.rs
pub mod roots;
pub mod stems;
pub mod leaves;

pub use self::leaves::Leaf;
pub use self::roots::Root;

use self::roots::RootSet;
use self::stems::StemSet;
```

Each `pub mod` declaration creates a public submodule. The compiler then looks for either:
- `plant_structures/roots.rs` (a file)
- `plant_structures/roots/mod.rs` (a directory with submodules)

This decoupling between module tree and file layout means you can refactor file organization without changing the public API. The module tree is the contract; files are implementation.

## Architectural Layers and Dependencies

A well-structured module hierarchy enforces architectural layers. In `fern_sim`, we see three distinct layers:

### Layer 1: Public API Surface (`lib.rs`)

The top-level `lib.rs` acts as a facade, presenting a curated interface:

```rust
pub mod plant_structures;
pub mod simulation;
pub mod spores;

pub use plant_structures::Fern;
pub use simulation::Terrarium;

pub mod net;
pub use net::connect;
```

This layer makes architectural decisions visible:
- **Re-exported types** (`Fern`, `Terrarium`) are considered central to the API
- **Re-exported functions** (`connect`) are common entry points
- **Public modules** (`plant_structures`, `simulation`) are available but require qualification

Users writing `use fern_sim::Fern;` get the convenience of a top-level import, while `use fern_sim::plant_structures::stems::Stem;` requires intentional navigation into implementation details.

### Layer 2: Domain Logic (`simulation.rs`, `plant_structures/`)

The middle layer contains the domain model and business logic:

```rust
// in simulation.rs
use crate::plant_structures::{Fern, FernType};

pub struct Terrarium {
    ferns: Vec<Fern>
}

impl Terrarium {
    pub fn apply_sunlight(&mut self, time: Duration) {
        for f in &mut self.ferns {
            for s in &mut f.stems {
                s.furled = false;
            }
        }
    }
}
```

Notice the dependency direction: `simulation` depends on `plant_structures`, not the reverse. This is enforced by the module system—circular dependencies between modules cause compilation errors.

### Layer 3: Implementation Details (`spores.rs`, internal modules)

The innermost layer contains implementation details that may change:

```rust
// in spores.rs
mod cells {
    //! The simulation of biological cells
    pub struct Cell {
        x: f64,
        y: f64
    }
}
```

The `cells` module is private to `spores.rs`. No other module can access it, creating a hard boundary. This is architectural enforcement: the cell simulation is an implementation detail of spore production, not a public concept.

## Visibility as Architectural Contract

Rust's visibility modifiers create compile-time enforced boundaries:

```rust
pub struct Spore {
    size: f64  // Private field
}

pub fn produce_spore(factory: &mut Sporangium) -> Spore {
    Spore { size: 1.0 }
}

pub(crate) fn genes(spore: &Spore) -> Vec<Gene> {
    todo!()
}

fn recombine(parent: &mut Cell) {
    todo!()
}
```

Four levels of visibility:
1. **`pub struct Spore`** with private fields: Public type, but users can't construct or inspect it directly
2. **`pub fn produce_spore`**: The only way to create a `Spore`—a factory pattern enforced at compile time
3. **`pub(crate) fn genes`**: Accessible within `fern_sim` crate, but not to external users
4. **`fn recombine`**: Private to the `spores` module only

This visibility ladder creates concentric circles of access, with the public API as the outermost ring and private implementation details at the core.

## Domain-Driven Design with Modules

The `fern_sim` structure exemplifies domain-driven design (DDD) principles:

### Ubiquitous Language

The module names use domain vocabulary: `Terrarium`, `Fern`, `Sporangium`, `VascularPath`. A botanist reading the code would recognize these terms. The module structure documents the domain model in compilable form.

### Bounded Contexts

Each module represents a bounded context:
- `simulation` owns the lifecycle and orchestration
- `plant_structures` owns the biological hierarchy
- `spores` owns reproduction mechanics
- `net` owns network communication

These contexts have defined interfaces (their public APIs) and internal implementations that can evolve independently.

### Aggregates and Ownership

In DDD, aggregates have clear ownership boundaries. Rust's ownership system makes this explicit:

```rust
pub struct Terrarium {
    ferns: Vec<Fern>  // Terrarium owns its ferns
}

impl Terrarium {
    pub fn fern(&self, index: usize) -> &Fern {
        &self.ferns[index]  // Borrow, don't transfer ownership
    }
}
```

The `Terrarium` is an aggregate root. External code cannot add or remove ferns directly—all mutations flow through `Terrarium`'s methods. This isn't just good practice; it's enforced by the type system.

## Scaling Patterns: From Flat to Hierarchical

As projects grow, module architecture must evolve. Consider three stages:

### Stage 1: Single-File Binary (e.g., `grep`)

```rust
// grep/src/main.rs
fn grep<R>(target: &str, reader: R) -> io::Result<()>
    where R: BufRead
{ /* ... */ }

fn grep_main() -> Result<(), Box<dyn Error>> { /* ... */ }

fn main() { /* ... */ }
```

All code in one file. Functions are the organizational unit. This works for utilities under 500 lines.

### Stage 2: Library with Modules (e.g., `fern_sim`)

```
fern_sim/
├── lib.rs              # Public facade
├── simulation.rs       # Feature module
├── spores.rs          # Feature module
└── plant_structures/  # Nested modules
```

Multiple files, explicit module declarations. Feature modules group related functionality. This scales to several thousand lines.

### Stage 3: Workspace with Sub-Crates

For very large projects (not shown in this repository), split into multiple crates:

```
workspace/
├── Cargo.toml          # Workspace manifest
├── core/              # Core domain logic
├── api/               # Public API layer
├── storage/           # Persistence
└── network/           # Networking
```

Each directory is a separate crate with its own `Cargo.toml`. The workspace coordinates compilation and dependencies.

## Anti-Patterns and Code Smells

### Anti-Pattern 1: Implementation Leakage

```rust
// BAD: Exposing internal structure
pub mod database {
    pub struct InternalRow { /* ... */ }
    pub fn raw_query() -> Vec<InternalRow> { /* ... */ }
}
```

External users now depend on `InternalRow`, an internal concept. If the database implementation changes, the entire public API breaks.

**Better:**
```rust
pub mod database {
    struct InternalRow { /* ... */ }  // Private
    pub struct Record { /* ... */ }    // Public wrapper

    pub fn query() -> Vec<Record> {
        raw_query().into_iter()
            .map(|row| Record::from(row))
            .collect()
    }

    fn raw_query() -> Vec<InternalRow> { /* ... */ }
}
```

### Anti-Pattern 2: Module Spaghetti

```rust
// BAD: Bidirectional dependencies
// simulation.rs
use crate::plant_structures::Fern;

// plant_structures/mod.rs
use crate::simulation::Terrarium;  // Circular!
```

Rust prevents this at compile time with error messages about circular dependencies. The fix is architectural: identify which module should depend on which, and introduce abstractions (traits) if bidirectional communication is needed.

### Anti-Pattern 3: God Module

```rust
// BAD: Everything in one module
pub mod core {
    pub struct Fern { /* ... */ }
    pub struct Terrarium { /* ... */ }
    pub fn simulate() { /* ... */ }
    pub fn render() { /* ... */ }
    pub fn save() { /* ... */ }
    pub fn load() { /* ... */ }
    // ... 2000 more lines
}
```

The `core` module has too many responsibilities. Split by concern: `domain`, `simulation`, `persistence`, `rendering`.

## Practical Guidelines for Architects

When designing module hierarchies:

1. **Start with domain nouns**: Modules should represent domain concepts (e.g., `orders`, `customers`, `inventory`), not technical layers (e.g., `models`, `controllers`, `views`)

2. **Dependencies flow inward**: High-level modules depend on low-level modules. The domain model should not depend on infrastructure.

3. **Public types at boundaries**: Types that cross module boundaries should be in public modules with stable APIs.

4. **Re-exports for convenience, not necessity**: Re-export commonly-used types at the crate root, but keep the canonical definition in a specific module.

5. **Test module organization**: If navigating the module tree feels awkward, the architecture probably needs revision.

The module system is not just an organizational tool—it's a design language that expresses architectural intent in executable, compiler-checked form. A well-designed module hierarchy makes the codebase self-documenting and guides developers toward correct usage patterns.
