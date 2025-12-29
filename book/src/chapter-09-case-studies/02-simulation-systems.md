# Building a Simulation System

Simulation systems present unique architectural challenges: they model complex domains with intricate state transitions, hierarchical relationships, and often require clear separation between simulation logic and infrastructure. The fern simulation system (`fern_sim`) demonstrates how Rust's module system and type design enable maintainable, extensible simulations.

## Domain Modeling with Rust Types

The foundation of any simulation is its domain model—the types that represent the entities being simulated. The fern_sim project models biological entities with a clear type hierarchy:

```rust
// From fern_sim/src/plant_structures/mod.rs
pub struct Fern {
    pub name: String,
    pub stems: Vec<Stem>,
}

pub struct Stem {
    pub furled: bool,
    pub length: f32,
}

pub enum FernType {
    Fiddlehead,
    Maidenhair,
    BostonFern,
}
```

**Type Design Decisions**:

**Public Fields vs. Encapsulation**: The `pub` fields in `Fern` and `Stem` represent a deliberate trade-off. For simulation code where components frequently interact, public fields reduce boilerplate while maintaining clarity. The module boundary provides encapsulation—users of the `fern_sim` crate see a clean API, but internal structures remain accessible within the module.

**Enums for Classification**: `FernType` uses an enum rather than strings or numeric constants. This provides exhaustive matching—the compiler ensures all fern types are handled—and prevents invalid states. You can't accidentally create a `FernType::Oak`.

**Composition Over Inheritance**: The `Fern` struct contains a `Vec<Stem>` rather than inheriting from a `Plant` base class. This composition pattern is idiomatic in Rust, which lacks inheritance. It's more flexible: you can have multiple vectors of different types, and ownership semantics remain clear.

## Hierarchical Module Structure for Complex Domains

As simulations grow, code organization becomes critical. The fern_sim project uses Rust's module system to create a clean, navigable structure:

```
fern_sim/
├── src/
│   ├── lib.rs                  # Public API definition
│   ├── simulation.rs           # Core simulation logic
│   ├── spores.rs              # Spore propagation
│   ├── plant_structures/      # Nested module
│   │   ├── mod.rs             # Plant structure definitions
│   │   ├── leaves.rs
│   │   ├── roots.rs
│   │   ├── stems.rs
│   │   └── stems/             # Sub-nested module
│   │       └── ...
│   └── net.rs                 # Networking utilities
```

**Root Module Design** (lib.rs):

```rust
//! Simulate the growth of ferns, from the level of
//! individual cells on up.

pub mod plant_structures;
pub mod simulation;
pub mod spores;

pub use plant_structures::Fern;
pub use simulation::Terrarium;

pub mod net;
pub use net::connect;
```

This structure demonstrates several architectural patterns:

**Public Module Declarations**: `pub mod plant_structures` exposes the entire module to external users. This makes the module's contents available as `fern_sim::plant_structures::Fern`.

**Selective Re-exports**: `pub use plant_structures::Fern` brings `Fern` to the crate root, allowing users to write `fern_sim::Fern` instead of the longer path. This is **API design via re-exports**—you control the public interface independent of internal organization.

**Module Documentation**: The `//!` doc comment at the top describes the entire crate. Module-level documentation is crucial for simulations, where understanding the overall purpose helps navigate complex code.

## Module Organization Philosophy

The fern_sim structure embodies a clear organizational philosophy:

**1. Grouping by Domain Concept**: `plant_structures` contains everything related to plant anatomy. `simulation` contains the simulation engine. This **domain-driven organization** makes code discoverable—if you want to modify how ferns grow, you know to look in `simulation.rs`.

**2. Privacy by Default**: Inside `plant_structures/`, submodules like `leaves.rs` and `roots.rs` are not `pub mod` by default—they're private to `plant_structures`. The `mod.rs` file controls what's exposed:

```rust
// plant_structures/mod.rs
mod leaves;
mod stems;

pub use self::leaves::*;
pub use self::stems::Stem;

pub struct Fern {
    pub stems: Vec<Stem>,
    // leaves are internal
}
```

This **façade pattern** hides implementation details. Users get a clean `Fern` API without needing to understand the internal leaf/stem/root organization.

**3. Flat When Possible, Nested When Necessary**: The top-level modules (`simulation.rs`, `spores.rs`) are flat files. Only `plant_structures` is nested, because it genuinely has multiple related concepts. This avoids over-engineering—don't create module hierarchies until complexity demands it.

## Managing Complex State Evolution

Simulations evolve state over time. The `Terrarium` type demonstrates state management patterns:

```rust
pub struct Terrarium {
    ferns: Vec<Fern>
}

impl Terrarium {
    pub fn new() -> Terrarium {
        Terrarium { ferns: vec![] }
    }

    pub fn load(filename: &str) -> Terrarium {
        File::open(filename).unwrap();
        Terrarium {
            ferns: vec![
                Fern::new(FernType::Fiddlehead)
            ]
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

**State Management Patterns**:

**Encapsulated Mutation**: The `ferns` field is private. External code can't directly modify the fern vector—they must use methods like `apply_sunlight`. This **encapsulation** ensures invariants are maintained. If ferns must always be in a certain state when sunlight is applied, the method enforces it.

**Borrowing for Access**: `fern(&self, index)` returns `&Fern`, a shared reference. This prevents mutation while still allowing inspection. If you need mutable access, you'd add `fern_mut(&mut self, index) -> &mut Fern`.

**Iterator Chains for Transformation**: `apply_sunlight` uses nested loops with mutable references:

```rust
for f in &mut self.ferns {      // Mutable borrow of each fern
    for s in &mut f.stems {     // Mutable borrow of each stem
        s.furled = false;       // Mutation allowed
    }
}
```

This pattern is safe because Rust's borrow checker ensures no other code accesses the ferns during this iteration. The mutation is **lexically scoped**—once the loop ends, the mutable borrow is released.

**Batch Updates**: Simulations often need to update all entities simultaneously. The for-loop pattern ensures atomic updates per entity but could be extended:

```rust
pub fn apply_sunlight_parallel(&mut self, time: Duration) {
    use rayon::prelude::*;
    self.ferns.par_iter_mut().for_each(|f| {
        f.stems.iter_mut().for_each(|s| {
            s.furled = false;
        });
    });
}
```

Using `rayon`, parallel iteration is trivial. The type system ensures thread safety—`Fern` must be `Send` to use `par_iter_mut`, guaranteeing safe concurrent access.

## Case Study: Complete Walkthrough

Let's trace a complete simulation cycle:

**1. Initialization**:
```rust
let mut terrarium = Terrarium::new();
// State: terrarium.ferns = []
```

**2. Loading State**:
```rust
let mut terrarium = Terrarium::load("garden.tm");
// State: terrarium.ferns = [Fern { stems: [Stem { furled: true }] }]
```

The `load` method deserializes saved state. In the real implementation, this would use `serde` for proper serialization:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Fern {
    pub name: String,
    pub stems: Vec<Stem>,
}

pub fn load(filename: &str) -> Result<Terrarium, Box<dyn Error>> {
    let file = File::open(filename)?;
    let ferns: Vec<Fern> = serde_json::from_reader(file)?;
    Ok(Terrarium { ferns })
}
```

**3. Simulation Step**:
```rust
terrarium.apply_sunlight(Duration::from_secs(3600));
// State: All stems.furled = false
```

**4. Observation**:
```rust
let fern = terrarium.fern(0);
println!("Fern has {} stems", fern.stems.len());
// Immutable access while state is stable
```

**5. Next Cycle**:
```rust
terrarium.apply_sunlight(Duration::from_secs(3600));
// Simulation continues...
```

This cycle demonstrates **separation of mutation and observation**. Mutation happens through `&mut self` methods, observation through `&self` methods. The borrow checker prevents you from observing during mutation, avoiding race conditions.

## Module Organization Lessons

The fern_sim architecture teaches several lessons:

**Lesson 1: Start Flat, Refactor to Hierarchy**: Begin with flat modules (`simulation.rs`). When a module grows complex, extract submodules. The `plant_structures` module likely started as a single file and was split when it grew.

**Lesson 2: Re-exports Define Your API**: The public API (what users see) is independent of internal organization. Use `pub use` aggressively to create an ergonomic interface:

```rust
// Internal: fern_sim::plant_structures::stems::Stem
// Public:  fern_sim::Stem (via pub use)
```

**Lesson 3: Privacy Prevents Misuse**: Keep fields and modules private unless there's a reason to expose them. It's easier to make something public later than to revoke public access (that's a breaking change).

**Lesson 4: Module Structure Mirrors Domain Structure**: Simulation systems benefit from modules that match domain concepts. `plant_structures` contains plant anatomy, `simulation` contains the simulation engine, `spores` contains propagation logic. This **isomorphism** between code and domain makes the codebase intuitive.

## Testing Simulation Systems

Simulations require specialized testing strategies:

**Property-Based Testing** for state invariants:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult};

    fn invariant_all_stems_valid(terrarium: &Terrarium) -> bool {
        terrarium.ferns.iter().all(|f| {
            f.stems.iter().all(|s| {
                s.length >= 0.0 && s.length <= 100.0
            })
        })
    }

    #[test]
    fn test_sunlight_preserves_invariants() {
        quickcheck(quickcheck as fn(u64) -> TestResult);

        fn quickcheck(seed: u64) -> TestResult {
            let mut terrarium = Terrarium::new();
            terrarium.ferns.push(random_fern(seed));

            terrarium.apply_sunlight(Duration::from_secs(1000));

            TestResult::from_bool(invariant_all_stems_valid(&terrarium))
        }
    }
}
```

**Snapshot Testing** for complex state:

```rust
#[test]
fn test_simulation_cycle() {
    let mut terrarium = Terrarium::load("fixtures/initial.tm");
    terrarium.apply_sunlight(Duration::from_secs(3600));

    let snapshot = serde_json::to_string_pretty(&terrarium).unwrap();
    insta::assert_snapshot!(snapshot);
}
```

The `insta` crate captures the terrarium state. On subsequent runs, it verifies the state matches the snapshot, catching unintended behavior changes.

**Deterministic Randomness** for reproducibility:

```rust
pub struct Terrarium {
    ferns: Vec<Fern>,
    rng: StdRng,  // Seeded RNG for reproducibility
}

impl Terrarium {
    pub fn new_with_seed(seed: u64) -> Terrarium {
        Terrarium {
            ferns: vec![],
            rng: StdRng::seed_from_u64(seed),
        }
    }
}
```

Seeded RNGs make simulations deterministic, enabling reproducible tests even for stochastic processes.

## Extending the Simulation

The module structure enables clean extensions:

**Adding New Entity Types**:
```rust
// plant_structures/fungi.rs
pub struct Mushroom {
    pub cap_diameter: f32,
    pub spore_count: u64,
}

// simulation.rs
pub struct Terrarium {
    ferns: Vec<Fern>,
    mushrooms: Vec<Mushroom>,  // New entity type
}
```

**Adding New Behaviors**:
```rust
// simulation.rs
impl Terrarium {
    pub fn apply_rain(&mut self, amount: f32) {
        for f in &mut self.ferns {
            f.hydration += amount;
        }
    }
}
```

The modular structure isolates changes. Adding mushrooms doesn't require modifying fern code. Adding rain behavior is localized to the simulation module.

## Cross-References to Earlier Patterns

The fern_sim architecture builds on foundational concepts:

- **Chapter 3 (Modules)**: The hierarchical module structure demonstrates Rust's module system at scale. Privacy controls, re-exports, and nested modules all appear.
- **Chapter 4 (Ownership)**: The `apply_sunlight` method shows mutable borrowing in nested iteration. The borrow checker ensures safety.
- **Chapter 5 (Error Handling)**: The `load` method (in the improved version) uses `Result` for I/O errors, demonstrating error propagation.
- **Chapter 7 (Collections)**: `Vec<Fern>` and `Vec<Stem>` show how owned collections enable dynamic simulation growth.

The fern_sim case study reveals how Rust's module system supports complex simulations. By organizing code around domain concepts, using privacy to enforce invariants, and leveraging ownership for safe state mutation, simulations become both maintainable and correct. The architecture scales from simple demonstrations to production systems with thousands of entities and complex interactions.

Next, we'll examine text processing architectures, where memory efficiency and performance become paramount concerns.
