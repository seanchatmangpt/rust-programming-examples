# Real-World Case Study: Fern Simulator

The `fern_sim` project is a pedagogical masterpiece. While ostensibly about simulating fern growth, its real purpose is teaching module organization, API design, and architectural thinking. This case study dissects every design decision, revealing patterns applicable to projects of any scale.

## Project Overview and Architecture

### The Facade: lib.rs

Every journey through `fern_sim` begins at `src/lib.rs`, the public face of the crate:

```rust
//! Simulate the growth of ferns, from the level of
//! individual cells on up.

#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

pub mod plant_structures;
pub mod simulation;
pub mod spores;

pub use plant_structures::Fern;
pub use simulation::Terrarium;

pub mod net;
pub use net::connect;
```

This 16-line file makes critical architectural decisions:

**Decision 1: Module Visibility Strategy**
- `plant_structures`, `simulation`, `spores`, `net` are all public modules
- Users can navigate into them: `use fern_sim::plant_structures::roots::Root;`
- This signals: "These are documented, stable subsystems you can explore"

**Decision 2: Convenience Re-Exports**
- `Fern` and `Terrarium` are promoted to the crate root
- Users write `use fern_sim::Fern;` not `use fern_sim::plant_structures::Fern;`
- This signals: "These are the core types. Most code will need them."

**Decision 3: Function Re-Export**
- `connect` is promoted from `net` module
- Networking is common enough to warrant top-level access
- But `net` module remains public for advanced users

The pattern is: **common things at the top, details in modules**. This is the Facade pattern in action.

## Module Hierarchy: Structure Mirrors Domain

```
fern_sim/
├── lib.rs                      # Public API surface
├── simulation.rs               # Orchestration layer
├── spores.rs                   # Reproduction subsystem
├── net.rs                      # External interface
└── plant_structures/           # Domain model
    ├── mod.rs                  # Public types
    ├── roots.rs                # Root system
    ├── leaves.rs               # Photosynthesis
    └── stems/                  # Vascular structure
        ├── mod.rs
        ├── xylem.rs           # Water transport
        └── phloem.rs          # Nutrient transport
```

This structure is not accidental. It mirrors biological reality:

- **Ferns** (the organism) have **plant structures**
- **Plant structures** include **roots**, **leaves**, and **stems**
- **Stems** contain **xylem** and **phloem** (vascular tissue)

A botanist unfamiliar with Rust could navigate this codebase. The module tree is a domain model diagram in executable form.

## The Orchestrator: simulation.rs

The `simulation` module is the application layer—it orchestrates domain objects:

```rust
//! Overall simulation control.
//!
//! The simulation algorithm is complex and has a lot of tweakable parameters.

use std::fs::File;
use std::time::Duration;
use crate::plant_structures::{Fern, FernType};

/// The simulated universe.
pub struct Terrarium {
    ferns: Vec<Fern>
}
```

### Design Decision: Ownership and Encapsulation

The `ferns` field is private. Users cannot:
- Access `ferns` directly
- Add or remove ferns except through `Terrarium` methods
- Iterate over ferns without going through the API

This is intentional encapsulation. The implementation uses `Vec<Fern>`, but that's not a contract. Future versions could use:
```rust
ferns: HashMap<FernId, Fern>        // For sparse collections
ferns: BTreeMap<Timestamp, Fern>    // For temporal queries
ferns: Arc<RwLock<Vec<Fern>>>       // For thread-safety
```

As long as `Terrarium`'s public methods work, external code is unaffected.

### API Design: Controlled Access

```rust
impl Terrarium {
    pub fn new() -> Terrarium {
        Terrarium { ferns: vec![] }
    }

    pub fn load(filename: &str) -> Terrarium {
        File::open(filename).unwrap();  // check that the file is there
        Terrarium {
            ferns: vec![Fern::new(FernType::Fiddlehead)]
        }
    }

    pub fn fern(&self, index: usize) -> &Fern {
        &self.ferns[index]
    }

    pub fn apply_sunlight(&mut self, time: Duration) {
        for f in &mut self.ferns {
            for s in &mut f.stems {
                s.furled = false;
            }
        }
    }
}
```

**Pattern: Constructors**
- `new()` creates an empty terrarium (the default case)
- `load()` creates from file (common initialization)
- No public constructor taking `Vec<Fern>` directly (preserves encapsulation)

**Pattern: Read-Only Access**
- `fern(&self, index: usize) -> &Fern` returns a reference, not ownership
- Users can inspect ferns but not remove them
- Mutation happens only through `Terrarium` methods like `apply_sunlight()`

**Pattern: Batch Operations**
- `apply_sunlight()` affects all ferns at once
- The API encourages thinking about collections, not individual elements
- This scales better than `apply_sunlight_to_fern(index, time)`

### Pragmatic Simplification

Notice this comment:
```rust
pub fn load(filename: &str) -> Terrarium {
    // This implementation is, like everything else in here, completely bogus
    File::open(filename).unwrap();  // check that the file is there
    Terrarium {
        ferns: vec![Fern::new(FernType::Fiddlehead)]
    }
}
```

The implementation is intentionally simplified. Real file parsing would involve:
- Error handling (Result types, not unwrap)
- Actual parsing logic (serde, custom deserializer)
- Validation (schema checking)

But this is **teaching code**. The lesson is about API design and module organization, not file formats. The honest comment acknowledges the simplification without apologizing for it.

## The Domain Model: plant_structures/mod.rs

The heart of the domain lives in `plant_structures/mod.rs`:

```rust
//! Higher-level biological structures.
//!
//! We always simulate a sample of all chemical interactions at the cellular
//! level, but simulating everything that way is just too computationally
//! expensive.  Therefore we keep higher-level data structures representing
//! each fern's roots, leaves, and so on.

pub mod roots;
pub mod stems;
pub mod leaves;

pub use self::leaves::Leaf;
pub use self::roots::Root;

use self::roots::RootSet;
use self::stems::StemSet;
```

### Pattern: Selective Re-Exports

Notice the asymmetry:
- `Leaf` and `Root` are re-exported: `pub use self::leaves::Leaf;`
- `RootSet` and `StemSet` are private imports: `use self::roots::RootSet;`

Why? Because `Leaf` and `Root` are core domain concepts users might reference:
```rust
use fern_sim::plant_structures::{Leaf, Root};
```

But `RootSet` and `StemSet` are internal collections, defined as:
```rust
// In roots.rs
pub type RootSet = Vec<Root>;

// In stems.rs
pub type StemSet = Vec<Stem>;
```

These are type aliases for convenience, not architectural types. Users don't need to know about them. The `Fern` struct uses them internally:

```rust
pub struct Fern {
    pub roots: RootSet,
    pub stems: StemSet
}
```

Wait—the fields are public! Doesn't that expose `RootSet`?

Yes, within the `plant_structures` module. But `RootSet` is not re-exported at the crate root. External users see:
```rust
pub struct Fern {
    pub roots: Vec<Root>,  // Type alias expands
    pub stems: Vec<Stem>,
}
```

The type alias is documentation for internal code. It doesn't create a new type boundary.

### Domain Logic: Methods, Not Getters

```rust
impl Fern {
    pub fn new(_type: FernType) -> Fern {
        Fern {
            roots: vec![],
            stems: vec![stems::Stem { furled: true }]
        }
    }

    pub fn is_furled(&self) -> bool {
        !self.is_fully_unfurled()
    }

    pub fn is_fully_unfurled(&self) -> bool {
        self.stems.iter().all(|s| !s.furled)
    }
}
```

**Anti-Pattern Avoided**: No `get_stems()` method returning `&Vec<Stem>`

**Pattern Used**: Domain queries with clear semantics
- `is_furled()` answers a domain question
- `is_fully_unfurled()` is the canonical implementation
- `is_furled()` is defined in terms of `is_fully_unfurled()` (single source of truth)

This is object-oriented thinking in Rust: objects know how to answer questions about their own state.

### Documentation as Domain Knowledge

```rust
/// Create and return a [`VascularPath`] which represents the path of
/// nutrients from the given [`Root`][r] to the given [`Leaf`](leaves::Leaf).
///
/// [r]: roots::Root
pub fn trace_path(leaf: &leaves::Leaf, root: &roots::Root) -> VascularPath {
    VascularPath { from: leaf.x, to: root.x }
}
```

The doc comment:
- Links to types using markdown syntax: `[`VascularPath`]`
- Uses reference-style links: `[Root][r]` with `[r]: roots::Root`
- Shows the full path: `leaves::Leaf` (helps users find types)

Run `cargo doc`, and these become clickable hyperlinks in the HTML documentation. The documentation is as much a part of the API as the function signatures.

## Encapsulation Layers: spores.rs

The `spores` module demonstrates advanced encapsulation:

```rust
#![allow(dead_code, unused_variables)]

//! Fern reproduction.

use cells::{Cell, Gene};

pub struct Spore {
    size: f64  // Private field!
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

pub struct Sporangium;

mod cells {
    //! The simulation of biological cells, which is as low-level as we go.

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

Four visibility levels in one file:

### Level 1: Public Types with Private Fields

```rust
pub struct Spore {
    size: f64  // Private
}
```

Users can:
- Receive a `Spore` from `produce_spore()`
- Pass `Spore` to functions
- Store `Spore` in collections

Users cannot:
- Construct a `Spore` directly
- Read or modify `size`
- Know anything about the internal representation

This is the **opaque handle** pattern. `Spore` is a capability token.

### Level 2: Factory Functions

```rust
pub fn produce_spore(factory: &mut Sporangium) -> Spore {
    Spore { size: 1.0 }
}
```

The only way to create a `Spore`. This enforces invariants:
- Spores can only come from sporangia (the factory)
- Size is determined by the simulation, not user input
- Future logic (genetics, environment) can be added here without breaking users

### Level 3: Internal API (pub(crate))

```rust
pub(crate) fn genes(spore: &Spore) -> Vec<Gene> {
    todo!()
}
```

Visible within `fern_sim` crate but not to external users. This allows:
- `simulation.rs` to extract genes for reproduction logic
- `plant_structures` to use genetic information for growth

But keeps genetics out of the public API. External users see spores as opaque; internal code sees the genetics layer.

### Level 4: Private Implementation

```rust
fn recombine(parent: &mut Cell) {
    todo!()
}
```

Private to `spores.rs`. Even other modules in `fern_sim` cannot call this. It's an internal helper for spore production.

### Level 5: Private Submodules

```rust
mod cells {
    pub struct Cell { x: f64, y: f64 }
    pub struct Gene;
}
```

The `cells` module is private to `spores.rs`. Its types are `pub` within the module (so `spores.rs` can use them), but the module itself is inaccessible:

```rust
use fern_sim::spores::cells::Cell;  // ERROR: cells is private
```

This creates architectural boundaries. Cellular biology is an implementation detail of spore production, not a public abstraction.

## Integration Testing: Validating the API

The test suite in `tests/unfurl.rs` validates the public API:

```rust
#[test]
fn test_fiddlehead_unfurling() {
    let mut world = Terrarium::load("tests/unfurl_files/fiddlehead.tm");
    assert!(world.fern(0).is_furled());
    let one_hour = Duration::from_secs(60 * 60);
    world.apply_sunlight(one_hour);
    assert!(world.fern(0).is_fully_unfurled());
}
```

This test is in `tests/`, not `src/`, meaning it **only** has access to the public API. It cannot:
- Access private fields
- Call internal functions
- Use `pub(crate)` items

This is integration testing in the truest sense—testing as an external user would use the library.

### What the Test Validates

**Narrative Flow**:
1. Load a terrarium from a file
2. Assert the fern is furled (initial state)
3. Apply sunlight for one hour
4. Assert the fern is now unfurled (state change)

**API Usability**:
- `Terrarium::load()` works with file paths
- `fern(index)` provides access to individual ferns
- `is_furled()` and `is_fully_unfurled()` are discoverable state queries
- `apply_sunlight()` has an intuitive signature

**Domain Correctness**:
- Ferns start furled (matches biological reality)
- Sunlight causes unfurling (domain behavior)

If this test compiles and passes, the API is usable and correct from an external perspective.

## Architectural Lessons

### Lesson 1: Module Tree Mirrors Domain

```
Biological Model          Code Structure
─────────────────         ───────────────
Fern                      plant_structures/Fern
├─ Roots                  └─ roots/Root
├─ Stems                  └─ stems/Stem
│  ├─ Xylem                  ├─ xylem.rs
│  └─ Phloem                 └─ phloem.rs
└─ Leaves                 └─ leaves/Leaf
```

The code structure documents the domain model. A new developer (or a botanist!) can navigate the codebase by understanding the domain.

### Lesson 2: Visibility Encodes Intent

- **`pub`**: "Stable API, external users depend on this"
- **`pub(crate)`**: "Internal infrastructure, shared across modules"
- **Private fields**: "Invariants to maintain, don't touch"
- **Private modules**: "Implementation detail, subject to change"

Every visibility choice is a statement of intent.

### Lesson 3: Re-Exports Create Convenience, Not Necessity

```rust
pub use plant_structures::Fern;  // Convenient
```

Users can write:
```rust
use fern_sim::Fern;  // Short
```

But could also write:
```rust
use fern_sim::plant_structures::Fern;  // Explicit
```

Both work. Re-exports optimize for the common case without restricting the explicit case.

### Lesson 4: Encapsulation Enables Evolution

Because `Terrarium.ferns` is private, the implementation can change:

```rust
// Version 1.0
pub struct Terrarium {
    ferns: Vec<Fern>,
}

// Version 2.0 (internal change only)
pub struct Terrarium {
    ferns: HashMap<FernId, Fern>,
    next_id: FernId,
}
```

As long as `fern(index)` still works (maybe reimplemented as `ferns.get(&index)`), external code is unaffected. This is API stability through encapsulation.

### Lesson 5: Simplification for Pedagogy is Valid

The `load()` function doesn't actually parse files. The `cells` module has stub implementations. This is **intentional teaching code**. The lessons are:
- Module organization patterns
- API surface design
- Encapsulation techniques

Not file parsing or cellular biology. The honest comments (`// completely bogus`) acknowledge this without diminishing the teaching value.

## Scaling `fern_sim`: Evolution Path

While `fern_sim` is ~300 lines, consider how it would scale:

### Stage 1: Current (Small Library)

```
fern_sim/  (~300 lines)
├── lib.rs
├── simulation.rs
├── spores.rs
└── plant_structures/
```

### Stage 2: Medium Library (~3,000 lines)

```
fern_sim/
├── lib.rs
├── core/
│   ├── organisms/  (ferns, mosses, lichens)
│   └── environment/
├── simulation/
│   ├── physics.rs
│   ├── growth.rs
│   └── genetics/
├── storage/
│   ├── snapshot.rs
│   └── formats/
└── network/
    └── sync.rs
```

The structure remains hierarchical. New modules group into architectural layers (`core`, `simulation`, `storage`).

### Stage 3: Large Library (~30,000 lines)

```
fern_sim/
├── lib.rs               # Just re-exports from api/
├── api/                 # Public facade
│   ├── mod.rs
│   └── prelude.rs
├── domain/              # Pure domain logic
├── engine/              # Simulation algorithms
├── persistence/         # Storage layer
└── protocols/           # Network protocols
```

At this scale, `lib.rs` becomes almost empty:
```rust
pub use api::*;
```

All architectural logic moves into submodules. The root is just a thin facade.

## Conclusion: Module Organization as Design Language

`fern_sim` is not about simulating ferns. It's about:

- **Expressing domain structure** in module hierarchies
- **Managing API surface** through re-exports and visibility
- **Enabling evolution** through encapsulation
- **Documenting architecture** in compilable form

The module system is not an afterthought—it's the primary design language for architecture. A well-organized module tree:
- Makes the codebase navigable to newcomers
- Documents architectural intent for maintainers
- Enforces boundaries the compiler can check
- Enables refactoring without breaking users

Study `fern_sim` not for its fern biology, but for its architectural discipline. The patterns shown here—facades, selective re-exports, visibility layers, opaque handles—scale from toy examples to production systems of any size.

The best architecture is one that makes correct usage easy and incorrect usage difficult. Rust's module system makes this expressible, enforceable, and evolutionary.
