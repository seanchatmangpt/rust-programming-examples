# Case Study: The Fern Simulator

The `fern_sim` project is a pedagogical example that demonstrates how to structure a non-trivial Rust library. While it simulates fern growth, the real lesson is in its architecture, module organization, and API design decisions.

## Project Architecture

### Module Hierarchy

The project uses a hierarchical module structure:

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
        ├── xylem.rs
        └── phloem.rs
```

This structure mirrors the conceptual organization of the domain: a fern simulator has plant structures, which have stems, which have vascular tissue. **The module tree reflects the problem domain, not the implementation details.**

### API Surface Design

Look at `/home/user/rust-programming-examples/fern_sim/src/lib.rs`:

```rust
pub mod plant_structures;
pub mod simulation;
pub mod spores;

pub use plant_structures::Fern;
pub use simulation::Terrarium;

pub mod net;
pub use net::connect;
```

Notice the deliberate choices:
- **Selective re-exports**: `Fern` and `Terrarium` are re-exported at the crate root for convenience
- **Module visibility**: Some modules (`plant_structures`, `simulation`) are public but require qualification
- **Function re-exports**: `connect` is promoted to the top level because it's a common entry point

**In Python**, you'd achieve similar organization with `__init__.py` imports, but Python's module system is fundamentally different - it's based on files and directories, not on explicit declarations. Rust gives you more control but requires more upfront design.

## Component Interaction

### The Terrarium Orchestrates the World

`/home/user/rust-programming-examples/fern_sim/src/simulation.rs` shows the central orchestrator:

```rust
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

Key design decisions:

1. **Ownership**: `Terrarium` owns its `Vec<Fern>`. It's responsible for the lifetime of all ferns.
2. **Controlled access**: You get ferns via `fern(&self, index: usize) -> &Fern`, preventing external mutation except through controlled APIs
3. **Batch operations**: `apply_sunlight` affects all ferns at once, showing how Rust encourages thinking about collections

**Contrast with Python**: In Python, you might be tempted to expose `terrarium.ferns` directly as a list. Users could then append, remove, or modify ferns at will. Rust's default is encapsulation - you must explicitly choose to expose internals.

### The Fern Knows Its State

From `/home/user/rust-programming-examples/fern_sim/src/plant_structures/mod.rs`:

```rust
pub struct Fern {
    pub roots: RootSet,
    pub stems: StemSet
}

impl Fern {
    pub fn is_furled(&self) -> bool {
        !self.is_fully_unfurled()
    }

    pub fn is_fully_unfurled(&self) -> bool {
        self.stems.iter().all(|s| !s.furled)
    }
}
```

Notice:
- Fields are public (within the module), but the crate API could restrict this
- State queries are methods, not direct field access
- `is_furled()` is defined in terms of `is_fully_unfurled()` - single source of truth

## Module Visibility and Encapsulation

### Public, Private, and pub(crate)

Look at `/home/user/rust-programming-examples/fern_sim/src/spores.rs`:

```rust
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

mod cells {
    pub struct Cell {
        x: f64,
        y: f64
    }
}
```

Multiple levels of visibility:

1. **`pub struct Spore`**: Public type, but private fields - you can have a `Spore`, but can't create one or inspect it
2. **`pub fn produce_spore`**: Public factory function - the only way to create spores
3. **`pub(crate) fn genes`**: Visible within `fern_sim`, but not to external users
4. **`fn recombine`**: Private to this module only
5. **`mod cells`**: Private submodule - an implementation detail

**This is impossible in Python**. Python has weak encapsulation conventions (`_private`, `__very_private`), but nothing prevents access. Rust enforces these boundaries at compile time.

## Design Decisions and Trade-offs

### Simplicity vs. Realism

The code includes intentional simplifications:

```rust
pub fn load(filename: &str) -> Terrarium {
    // This implementation is, like everything else in here, completely bogus
    File::open(filename).unwrap();  // check that the file is there
    Terrarium {
        ferns: vec![Fern::new(FernType::Fiddlehead)]
    }
}
```

**Why?** Because this is teaching code. The comment acknowledges the simplification. The real lesson is about file I/O patterns, error handling, and API design - not about actual fern biology or `.tm` file formats.

### Type Safety with Newtypes and Type Aliases

```rust
pub type RootSet = Vec<Root>;
pub type StemSet = Vec<Stem>;
```

These are type aliases, not newtypes. They're documentation - making it clear that a `StemSet` is a collection of stems. But they don't provide type safety - you could pass a `Vec<Root>` where a `StemSet` is expected.

**Alternative design**: Make them proper newtypes:

```rust
pub struct RootSet(Vec<Root>);
pub struct StemSet(Vec<Stem>);
```

This would prevent mixing them up, but would require more boilerplate to make them usable. The code chose convenience over maximum safety - a legitimate trade-off for teaching code.

## Testing Strategy

The test in `/home/user/rust-programming-examples/fern_sim/tests/unfurl.rs` demonstrates integration testing:

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

Key points:

1. **Tests live in `tests/`**: These are integration tests that test the public API
2. **File-based fixtures**: The test references an external file (`fiddlehead.tm`)
3. **Behavioral testing**: Testing state changes over time, not just functions
4. **Clear narrative**: The test reads like a story - setup, action, assertion

**In Python**, you'd write similar tests with `pytest` or `unittest`, but Rust's testing framework is built-in and follows strong conventions about where tests live and how they're organized.

## What Python Developers Can Learn

### 1. Explicit Module Design

Python's module system is implicit - create a directory with `__init__.py`, and it's a package. Rust requires you to declare every module and its visibility explicitly. This is more work upfront but makes the architecture visible in the code.

### 2. Compile-Time API Boundaries

```rust
pub use plant_structures::Fern;
```

This line is a contract. External code can access `Fern` at the crate root. Change or remove it, and dependent code won't compile. **In Python**, imports might fail at runtime, possibly in rarely-tested code paths.

### 3. Ownership Clarifies Responsibility

When `Terrarium` owns `Vec<Fern>`, it's clear who's responsible for those ferns' lifecycle. In Python, with shared references everywhere, responsibility is often ambiguous. Who should clean up? Who can modify? Rust makes these questions explicit.

### 4. Documentation as Code

```rust
/// Create and return a [`VascularPath`] which represents the path of
/// nutrients from the given [`Root`][r] to the given [`Leaf`](leaves::Leaf).
///
/// [r]: roots::Root
pub fn trace_path(leaf: &leaves::Leaf, root: &roots::Root) -> VascularPath {
    VascularPath { from: leaf.x, to: root.x }
}
```

Doc comments use markdown and can link to types. Run `cargo doc`, and you get a complete, hyperlinked API reference. The documentation lives with the code and can't get out of sync with function signatures.

### 5. Modules Are Not Files

In Rust, `mod stems` in `plant_structures/mod.rs` can refer to either:
- `plant_structures/stems.rs` (a file)
- `plant_structures/stems/mod.rs` (a directory module)

Both exist in this project! The module tree is explicit in the code, with files as an implementation detail. Python conflates the two concepts.

## Architectural Patterns

### The Facade Pattern

`lib.rs` is a facade - it presents a simple, curated API to users while hiding the complexity of internal module organization:

```rust
pub use plant_structures::Fern;
pub use simulation::Terrarium;
```

Users can write:
```rust
use fern_sim::{Fern, Terrarium};
```

Instead of:
```rust
use fern_sim::plant_structures::Fern;
use fern_sim::simulation::Terrarium;
```

### Hierarchical Composition

Ferns have stems, stems have xylem and phloem. This is modeled both in the type hierarchy and the module hierarchy:

```
Fern { stems: StemSet }
stems/
  ├── xylem.rs
  └── phloem.rs
```

The structure of the code mirrors the structure of the domain.

## Conclusion

`fern_sim` is deceptively simple. It's not about simulating ferns - it's about:

- **Module organization** that scales to real projects
- **API design** that balances convenience and encapsulation
- **Documentation** that becomes part of the developer experience
- **Testing** that validates behavior, not implementation

For Python developers, the key insight is that Rust requires more upfront design. You can't just create files and import them - you must think about visibility, ownership, and API boundaries from the start. This feels constraining at first, but it prevents entire classes of bugs and makes large codebases maintainable.

The module system isn't just an organizational tool - it's a design language that lets you express architectural intent in code.
