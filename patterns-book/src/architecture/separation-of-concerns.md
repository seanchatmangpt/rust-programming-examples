# Separation of Concerns

## Pattern Summary

Split your codebase into distinct modules where each module addresses a separate concern—such as separating data structures from algorithms, simulation logic from domain models, or I/O from computation—enabling independent evolution and testing.

---

## Context

You are building a Rust application or library with multiple responsibilities: managing data structures, implementing algorithms, handling I/O, controlling simulation flow, etc. As the codebase grows, mixing these concerns in the same module leads to tangled dependencies and makes it difficult to understand, test, or modify any single aspect.

You want to organize code so that changes to one concern (e.g., simulation algorithm) don't require changes to another (e.g., data structure representation).

---

## Problem

**How do you organize code so that each module has a single, well-defined responsibility, making the system easier to understand, test, and evolve?**

Mixing concerns creates problems:
- **Tangled dependencies**: Changing data structures requires changing algorithms
- **Hard to test**: Can't test simulation logic without I/O infrastructure
- **Difficult to understand**: Modules do too many things at once
- **Rigid architecture**: Adding new features requires modifying multiple unrelated parts
- **Poor reusability**: Can't reuse data structures without bringing along unrelated logic

---

## Forces

- **Cohesion**: Related functionality should be grouped together
- **Coupling**: Unrelated functionality should be separated
- **Testability**: Each concern should be testable in isolation
- **Comprehensibility**: Developers should understand one concern without understanding others
- **Reusability**: Modules should be reusable in different contexts
- **Changeability**: Modifying one concern shouldn't ripple to others
- **Granularity**: Modules shouldn't be so fine-grained that you need dozens for a simple task

Too much separation leads to fragmentation and indirection. Too little separation leads to monolithic, tangled code.

---

## Solution

**Organize your codebase into modules where each module addresses a single, well-defined concern. Use clear interfaces between modules to minimize coupling.**

### Core Separation Patterns

1. **Data vs. Behavior**: Separate data structures from algorithms that operate on them
2. **Domain vs. Infrastructure**: Separate business logic from I/O, persistence, frameworks
3. **Configuration vs. Execution**: Separate setup/initialization from runtime logic
4. **Public API vs. Implementation**: Separate interface from implementation details

### Implementation

From `fern_sim`, demonstrating multiple separations:

**Separation 1: Simulation Logic vs. Data Structures**

```
fern_sim/src/
├── plant_structures/    ← Data structures (WHAT)
│   ├── mod.rs          # Fern, Leaf, Root, Stem definitions
│   ├── leaves.rs
│   ├── roots.rs
│   └── stems/
└── simulation.rs        ← Algorithms (HOW)
                        # Terrarium, apply_sunlight, simulation control
```

**Data structures module** (`plant_structures/mod.rs`):

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

pub use self::leaves::Leaf;
pub use self::roots::Root;

use self::roots::RootSet;
use self::stems::StemSet;

pub enum FernType {
    Fiddlehead
}

// Pure data structure - no simulation logic
pub struct Fern {
    pub roots: RootSet,
    pub stems: StemSet
}

impl Fern {
    // Constructor - minimal behavior
    pub fn new(_type: FernType) -> Fern {
        Fern {
            roots: vec![],
            stems: vec![stems::Stem { furled: true }]
        }
    }

    // Query methods - no simulation logic
    pub fn is_furled(&self) -> bool {
        !self.is_fully_unfurled()
    }

    pub fn is_fully_unfurled(&self) -> bool {
        self.stems.iter().all(|s| !s.furled)
    }
}
```

**Simulation module** (`simulation.rs`):

```rust
//! Overall simulation control.
//!
//! The simulation algorithm is complex and has a lot of tweakable parameters.

use std::fs::File;
use std::time::Duration;
use crate::plant_structures::{Fern, FernType};

/// The simulated universe.
///
/// CONCERN: Simulation control and orchestration.
/// Does NOT define plant structures - imports them.
pub struct Terrarium {
    ferns: Vec<Fern>  // Uses Fern, doesn't define it
}

impl Terrarium {
    pub fn new() -> Terrarium {
        Terrarium { ferns: vec![] }
    }

    pub fn load(filename: &str) -> Terrarium {
        // I/O concern - loading state from disk
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

    /// Run the simulation for a given amount of time.
    ///
    /// CONCERN: Simulation algorithm (HOW ferns grow).
    /// Operates on Fern data structures but doesn't define them.
    pub fn apply_sunlight(&mut self, time: Duration) {
        for f in &mut self.ferns {
            for s in &mut f.stems {
                s.furled = false;  // Simulation effect
            }
        }
    }
}
```

**Why this separation works**:
- `plant_structures` can evolve independently (add new FernType, change field types)
- `simulation` can change algorithms without touching data structures
- Both can be tested separately
- Fern structures can be reused in other contexts (visualization, serialization)

---

**Separation 2: Core Logic vs. Reproduction Details**

```
fern_sim/src/
├── plant_structures/    ← Core plant data
└── spores.rs            ← Specialized concern: reproduction
```

**Spores module** (`spores.rs`):

```rust
//! Fern reproduction.
//!
//! CONCERN: Biological reproduction (specialized domain logic).
//! Separate from general plant structures.

use cells::{Cell, Gene};

pub struct Spore {
    size: f64
}

/// Simulate the production of a spore by meiosis.
pub fn produce_spore(factory: &mut Sporangium) -> Spore {
    Spore { size: 1.0 }
}

pub(crate) fn genes(spore: &Spore) -> Vec<Gene> {
    todo!()
}

fn recombine(parent: &mut Cell) {
    todo!()
}

// Nested module for low-level concern: cellular biology
mod cells {
    //! The simulation of biological cells, which is as low-level as we go.
    //!
    //! CONCERN: Cell-level biology (lowest abstraction level).

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

**Why this separation works**:
- Reproduction logic doesn't clutter main plant structures
- Cellular details (`cells` module) are hidden from higher levels
- Can test spore production without entire simulation

---

**Separation 3: Hierarchical Concerns**

```
plant_structures/
├── mod.rs        ← High-level concern: whole plant
├── leaves.rs     ← Mid-level concern: leaf structures
├── roots.rs      ← Mid-level concern: root structures
└── stems/
    ├── mod.rs    ← Mid-level concern: stem structures
    ├── phloem.rs ← Low-level concern: nutrient transport
    └── xylem.rs  ← Low-level concern: water transport
```

Each level addresses a concern at a specific abstraction:
- **mod.rs**: Whole plant (Fern)
- **leaves/roots/stems**: Organ systems
- **phloem/xylem**: Tissue-level details

---

### Usage Patterns

**Pattern 1: Data-Algorithm Separation**

```rust
// data.rs - Pure data structures
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

// validation.rs - Business rules
pub fn validate_user(user: &User) -> Result<(), ValidationError> {
    if user.name.is_empty() {
        return Err(ValidationError::EmptyName);
    }
    if !user.email.contains('@') {
        return Err(ValidationError::InvalidEmail);
    }
    Ok(())
}

// persistence.rs - I/O concern
pub fn save_user(user: &User, db: &Database) -> Result<()> {
    db.execute("INSERT INTO users ...", user)?;
    Ok(())
}
```

**Pattern 2: Domain-Infrastructure Separation**

```rust
// domain/models.rs - Business logic (pure)
pub struct Order {
    items: Vec<Item>,
    total: Money,
}

impl Order {
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
        self.total += item.price;
    }

    pub fn validate(&self) -> Result<(), OrderError> {
        // Business rules - no I/O, no infrastructure
        if self.items.is_empty() {
            return Err(OrderError::EmptyOrder);
        }
        Ok(())
    }
}

// infrastructure/database.rs - I/O concern
use crate::domain::models::Order;

pub struct OrderRepository {
    db: Database,
}

impl OrderRepository {
    pub fn save(&self, order: &Order) -> Result<()> {
        // Infrastructure - knows how to persist, not business rules
        self.db.execute("INSERT ...", order)
    }
}
```

**Pattern 3: Configuration-Execution Separation**

```rust
// config.rs - Setup concern
pub struct SimulationConfig {
    pub duration: Duration,
    pub fern_count: usize,
    pub light_intensity: f64,
}

// runtime.rs - Execution concern
use crate::config::SimulationConfig;

pub struct Simulator {
    config: SimulationConfig,
    state: SimulationState,
}

impl Simulator {
    pub fn new(config: SimulationConfig) -> Self {
        Simulator {
            config,
            state: SimulationState::default(),
        }
    }

    pub fn run(&mut self) {
        // Runtime logic uses config but doesn't define it
        for _ in 0..self.config.fern_count {
            self.add_fern();
        }
    }
}
```

### Guidelines

1. **Identify concerns early**
   - Data structures vs. algorithms
   - Pure logic vs. I/O
   - Core domain vs. infrastructure
   - Public API vs. implementation

2. **One concern per module**
   - If a module has "and" in its description, it's probably doing too much
   - Bad: "user validation and database persistence"
   - Good: "user validation" + "user persistence"

3. **Minimize cross-concern coupling**
   - Use clear interfaces between concerns
   - Depend on abstractions, not implementations

4. **Test concerns independently**
   - Data structures: test invariants
   - Algorithms: test with mock data
   - I/O: test with in-memory implementations

5. **Name modules by concern**
   - `simulation` (concern: simulation algorithm)
   - `plant_structures` (concern: domain data)
   - `spores` (concern: reproduction logic)

---

## Resulting Context

### Benefits

- **Independent evolution**: Change one concern without affecting others
  - Refactor simulation algorithm without touching Fern struct
  - Change Fern fields without rewriting apply_sunlight
- **Isolated testing**: Test each concern separately
  - Test Fern creation without running simulation
  - Test simulation logic with mock Ferns
- **Reusability**: Use modules in different contexts
  - Use plant_structures in a visualization tool
  - Use simulation in different environments
- **Comprehensibility**: Understand one piece at a time
  - Learn data structures without simulation algorithm
  - Learn simulation without cellular biology details
- **Parallel development**: Different teams/developers can work on different concerns

### Drawbacks

- **More files**: Separation creates more modules to navigate
  - *Mitigation*: Good naming and documentation
- **Indirection**: May need to import from multiple modules
  - *Mitigation*: Use re-exports for common combinations
- **Over-separation**: Too many tiny modules
  - *Mitigation*: Separate only when concerns are truly independent

### Invariants Maintained

- Each module addresses a single, well-defined concern
- Dependencies flow from high-level concerns to low-level concerns
- Changes to one concern don't require changes to others (except through interfaces)
- Concerns can be tested independently

---

## Related Patterns

- **[Hierarchical Modules](hierarchical-modules.md)**: Different levels of hierarchy often represent different concerns
- **[Visibility Boundaries](visibility-boundaries.md)**: Visibility enforces separation of concerns
- **[Library-Binary Split](library-binary-split.md)**: Ultimate separation—library (logic) from binary (I/O)

---

## Known Uses

### From `fern_sim`

- **Simulation vs. Data**: `simulation.rs` (algorithms) separate from `plant_structures/` (data)
- **Reproduction vs. Growth**: `spores.rs` (reproduction) separate from `simulation.rs` (growth)
- **Tissue-level vs. Organ-level**: `stems/phloem.rs` (tissue) separate from `stems/mod.rs` (organ)

### Rust Standard Library

```
std::
├── io/           ← I/O concern
├── collections/  ← Data structures concern
├── net/          ← Networking concern
├── fs/           ← Filesystem concern
└── sync/         ← Concurrency concern
```

Each module addresses a separate concern.

### Actix-web

```
actix_web/
├── app/          ← Application setup concern
├── handler/      ← Request handling concern
├── middleware/   ← Cross-cutting concerns
└── test/         ← Testing utilities concern
```

### Compiler Architecture (rustc-style)

```
compiler/
├── frontend/     ← Parsing and AST concern
├── middle/       ← Type checking and IR concern
└── backend/      ← Code generation concern
```

Classic three-phase compiler separation.

---

## Examples from Real Projects

### Web Application Layered Architecture

```
src/
├── domain/          ← Business logic concern
│   ├── models.rs
│   └── rules.rs
├── application/     ← Use case orchestration concern
│   └── services.rs
├── infrastructure/  ← I/O and external dependencies concern
│   ├── database.rs
│   └── http_client.rs
└── presentation/    ← HTTP API concern
    └── handlers.rs
```

### Game Engine

```
src/
├── ecs/            ← Entity-component-system concern
├── renderer/       ← Graphics concern
├── physics/        ← Physics simulation concern
├── audio/          ← Sound concern
└── input/          ← Input handling concern
```

Each system can evolve independently.

---

## Anti-Patterns to Avoid

### God Module

```rust
// ❌ BAD: One module does everything
mod app {
    pub struct User { ... }
    pub fn validate_user(...) { ... }
    pub fn save_user_to_db(...) { ... }
    pub fn render_user_profile(...) { ... }
    pub fn send_welcome_email(...) { ... }
}
```

**Problem**: Data, validation, persistence, rendering, and I/O all mixed.

**Fix**: Separate into concerns:

```rust
// ✅ GOOD
mod models {
    pub struct User { ... }
}

mod validation {
    pub fn validate_user(...) { ... }
}

mod persistence {
    pub fn save_user(...) { ... }
}

mod views {
    pub fn render_user_profile(...) { ... }
}

mod notifications {
    pub fn send_welcome_email(...) { ... }
}
```

### Leaky Abstractions

```rust
// ❌ BAD: Simulation knows about database details
pub fn apply_sunlight(&mut self, db: &Database) {
    for f in &mut self.ferns {
        f.grow();
        db.execute("UPDATE ferns ...");  // Simulation shouldn't know about DB!
    }
}
```

**Fix**: Separate persistence concern:

```rust
// ✅ GOOD: Simulation is pure, persistence is separate
pub fn apply_sunlight(&mut self) {
    for f in &mut self.ferns {
        f.grow();
    }
}

// In persistence module:
pub fn save_terrarium(terrarium: &Terrarium, db: &Database) {
    db.execute("UPDATE ...", terrarium);
}
```

---

## Decision Criteria

**Separate into different modules when**:
- Two pieces of code have different reasons to change
- You want to test them independently
- You want to reuse one without the other
- They operate at different abstraction levels
- They have different visibility requirements

**Keep in the same module when**:
- They always change together
- One is a private implementation detail of the other
- Separating would create excessive indirection
- The module is still small and comprehensible

---

## Checklist

When organizing modules by concern:

- [ ] Each module has a single, clear responsibility
- [ ] Module names reflect their concern
- [ ] Data structures are separate from algorithms
- [ ] Core logic is separate from I/O/infrastructure
- [ ] Concerns can be tested independently
- [ ] Changes to one concern don't ripple to others
- [ ] Dependencies flow in one direction (no cycles)

---

## Further Reading

- *Clean Architecture* (Robert C. Martin) - Separation of Concerns principles
- *Domain-Driven Design* (Eric Evans) - Domain vs. Infrastructure separation
- [Rust Book: Separating Modules into Different Files](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html)
- *Programming Rust*, Chapter 8: Modules
