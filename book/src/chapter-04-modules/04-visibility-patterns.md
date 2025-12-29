# Visibility Patterns

Rust's visibility system is more sophisticated than simple "public" or "private". It provides fine-grained control over which parts of your code are accessible at different scopes. Understanding these patterns is essential for building maintainable, evolvable APIs.

## The Visibility Spectrum

Rust offers five levels of visibility, from most restrictive to most permissive:

```rust
// 1. Private (default) - visible only in current module
struct InternalDetail;
fn helper_function() {}

// 2. pub(super) - visible in parent module
pub(super) struct ParentCanSee;

// 3. pub(crate) - visible within the entire crate
pub(crate) struct CrateInternal;

// 4. pub(in path) - visible in specific ancestor module
pub(in crate::outer) struct SpecificScope;

// 5. pub - visible to all users
pub struct PublicApi;
```

Each level serves a distinct architectural purpose.

## Private by Default: The Principle of Least Privilege

Rust's default is private, which aligns with the principle of least privilege: grant access only where needed. In `fern_sim/src/spores.rs`:

```rust
mod cells {
    //! The simulation of biological cells
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

The `cells` module is private to `spores.rs`. Even though `Cell` and `Gene` are marked `pub` within `cells`, they're only accessible to other code in `spores.rs`. External modules cannot see them:

```rust
use fern_sim::spores::cells::Cell;  // ERROR: cells is private
```

This creates a hard boundary: cellular biology is an implementation detail of spore production, not a public concept.

### Private Fields for Invariant Preservation

In the same file:

```rust
pub struct Spore {
    size: f64  // Private field
}

pub fn produce_spore(factory: &mut Sporangium) -> Spore {
    Spore { size: 1.0 }
}
```

The `Spore` type is public, but its `size` field is private. Users can receive a `Spore` from `produce_spore()`, but cannot:
- Construct a `Spore` directly (no public constructor)
- Read the `size` (field is private)
- Modify the `size` (field is private)

This enforces invariants: spore size is determined by the simulation, not user code. If the spore creation logic changes (e.g., size depends on genetics), external code doesn't break—it never had access in the first place.

## pub(crate): Internal APIs

The `pub(crate)` visibility is for code that multiple modules within your crate need, but external users should not access.

### Use Case 1: Shared Utilities

```rust
// In utils/math.rs
pub(crate) fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

The `fibonacci` function is accessible to any module in the crate:

```rust
// In simulation.rs
use crate::utils::math::fibonacci;

pub fn simulate_spiral_growth() {
    let count = fibonacci(12);
    // ...
}
```

But external users cannot call it:

```rust
use fern_sim::utils::math::fibonacci;  // ERROR: utils is private
```

### Use Case 2: Internal Traits

In `spores.rs`:

```rust
pub(crate) fn genes(spore: &Spore) -> Vec<Gene> {
    todo!()
}
```

The `genes()` function is public within the crate but hidden from external users. This allows other modules in `fern_sim` to extract genetic information from spores for internal simulation purposes, while keeping genetics out of the public API.

External users see:

```rust
pub struct Spore;
pub fn produce_spore(factory: &mut Sporangium) -> Spore;
```

Internal modules see:

```rust
pub struct Spore;
pub fn produce_spore(factory: &mut Sporangium) -> Spore;
pub(crate) fn genes(spore: &Spore) -> Vec<Gene>;  // Extra capability
```

### Use Case 3: Test Infrastructure

```rust
// In testing/mod.rs
#[cfg(test)]
pub(crate) mod fixtures {
    pub(crate) fn sample_terrarium() -> Terrarium {
        Terrarium::load("tests/fixtures/test.tm")
    }
}
```

Test utility functions are `pub(crate)` so all test modules can share them, but they don't pollute the public API.

## pub(super): Parent-Visible Items

The `pub(super)` visibility makes items accessible to the parent module, useful for tightly-coupled submodules.

### Example: Plant Structures Hierarchy

```rust
// In plant_structures/stems/mod.rs
mod xylem {
    pub(super) struct Vessel {
        diameter: f64,
    }

    pub(super) fn transport_rate(v: &Vessel) -> f64 {
        v.diameter * 3.14
    }
}

mod phloem {
    pub(super) struct SieveCell {
        length: f64,
    }
}

// In stems/mod.rs itself
pub struct Stem {
    vessels: Vec<xylem::Vessel>,     // OK: stems is parent of xylem
    cells: Vec<phloem::SieveCell>,   // OK: stems is parent of phloem
    pub furled: bool,
}

impl Stem {
    fn vascular_capacity(&self) -> f64 {
        self.vessels.iter()
            .map(|v| xylem::transport_rate(v))  // OK: can use pub(super) fn
            .sum()
    }
}
```

The `xylem` and `phloem` modules are private, but their contents are visible to the parent `stems` module via `pub(super)`. The hierarchy is:

```
plant_structures/
└── stems/
    ├── mod.rs       ← Can see pub(super) items
    ├── xylem.rs     ← Defines pub(super) Vessel
    └── phloem.rs    ← Defines pub(super) SieveCell
```

Modules outside `stems/` (like `plant_structures/roots.rs`) cannot access `xylem::Vessel`:

```rust
// In plant_structures/roots.rs
use super::stems::xylem::Vessel;  // ERROR: xylem is private
```

### When to Use pub(super)

Use `pub(super)` for:
- **Tightly-coupled submodules** that the parent orchestrates
- **Internal abstractions** the parent uses but submodules don't share
- **Builder patterns** where submodules construct pieces the parent assembles

Avoid `pub(super)` when:
- Multiple modules at the same level need access (use `pub(crate)` instead)
- The item might become public later (start with `pub(crate)` for flexibility)

## pub(in path): Scoped Visibility

The `pub(in path)` syntax makes items visible only within a specific ancestor module. This is rarely needed but powerful for complex hierarchies.

```rust
// In plant_structures/stems/xylem.rs
pub(in crate::plant_structures) struct VascularPath {
    // Visible to anything in plant_structures/, including:
    // - plant_structures/mod.rs
    // - plant_structures/roots.rs
    // - plant_structures/leaves.rs
    // But NOT visible to simulation.rs or other top-level modules
}
```

This creates a boundary at the `plant_structures` level. All plant structure code can see `VascularPath`, but simulation code cannot.

### Practical Use: Domain Boundaries

```rust
pub mod domain {
    pub(in crate::domain) struct InternalEvent {
        // Visible to all domain modules
    }

    pub mod users {
        pub fn create_user() {
            let evt = super::InternalEvent { /* ... */ };
            // OK: users is inside domain
        }
    }

    pub mod orders {
        pub fn place_order() {
            let evt = super::InternalEvent { /* ... */ };
            // OK: orders is inside domain
        }
    }
}

pub mod infrastructure {
    use crate::domain::InternalEvent;  // ERROR: not visible outside domain
}
```

## Sealed Traits: Preventing External Implementation

Sometimes you want users to consume a trait but not implement it. This is the "sealed trait" pattern.

### The Pattern

```rust
// In src/sealed.rs or embedded in lib.rs
mod sealed {
    pub trait Sealed {}
}

pub trait GrowthStage: sealed::Sealed {
    fn duration(&self) -> Duration;
    fn next_stage(&self) -> Option<Box<dyn GrowthStage>>;
}

// Only crate code can implement the sealed trait
impl sealed::Sealed for Seedling {}
impl GrowthStage for Seedling {
    fn duration(&self) -> Duration { Duration::from_days(7) }
    fn next_stage(&self) -> Option<Box<dyn GrowthStage>> {
        Some(Box::new(Fiddlehead))
    }
}

impl sealed::Sealed for Fiddlehead {}
impl GrowthStage for Fiddlehead {
    fn duration(&self) -> Duration { Duration::from_days(14) }
    fn next_stage(&self) -> Option<Box<dyn GrowthStage>> { None }
}
```

External users can write code that accepts `&dyn GrowthStage`:

```rust
fn simulate_growth(stage: &dyn GrowthStage) {
    println!("Stage duration: {:?}", stage.duration());
}
```

But they cannot implement `GrowthStage` on their own types:

```rust
struct CustomStage;

impl GrowthStage for CustomStage {  // ERROR: trait sealed::Sealed is private
    // ...
}
```

### Why Seal Traits?

Seal traits to:
- **Add methods in minor versions**: If users can't implement the trait, adding methods is non-breaking
- **Maintain invariants**: Ensure all implementors satisfy assumptions (e.g., all stages form a valid lifecycle)
- **Control trait objects**: Ensure `Box<dyn Trait>` only contains known types

Don't seal traits when:
- The trait is explicitly for extension (like `serde::Serialize`)
- It's a core abstraction users should implement (like iterator adapters)

## Encapsulation Patterns in Practice

### Pattern 1: Opaque Handles

```rust
pub struct Terrarium {
    ferns: Vec<Fern>,  // Private field
}

impl Terrarium {
    pub fn fern(&self, index: usize) -> &Fern {
        &self.ferns[index]  // Controlled access
    }
}
```

Users cannot:
- Directly access `ferns` (field is private)
- Add or remove ferns without going through `Terrarium` methods
- Mutate ferns except through `Terrarium`'s API

The `Terrarium` type is an opaque handle to the fern collection.

### Pattern 2: Builder with Private Fields

```rust
pub struct FernBuilder {
    fern_type: FernType,
    initial_roots: usize,
}

impl FernBuilder {
    pub fn new(fern_type: FernType) -> Self {
        FernBuilder {
            fern_type,
            initial_roots: 5,  // Default
        }
    }

    pub fn with_roots(mut self, count: usize) -> Self {
        self.initial_roots = count;
        self
    }

    pub fn build(self) -> Fern {
        Fern {
            roots: vec![Root::default(); self.initial_roots],
            stems: vec![],
        }
    }
}
```

The builder fields are public within the module (for the builder methods) but private to external users. Only the `build()` method constructs the final `Fern`.

### Pattern 3: Newtype for Strong Typing

```rust
pub struct Temperature(f64);  // Field is private by default

impl Temperature {
    pub fn from_celsius(c: f64) -> Self {
        Temperature(c)
    }

    pub fn from_fahrenheit(f: f64) -> Self {
        Temperature((f - 32.0) * 5.0 / 9.0)
    }

    pub fn as_celsius(&self) -> f64 {
        self.0
    }
}
```

Users cannot construct `Temperature` directly or access the inner `f64`. They must use the constructors, which enforce valid units.

## Visibility Checklist for API Design

When deciding on visibility, ask:

1. **Will external users ever need this?**
   - No → Keep private
   - Maybe → Use `pub(crate)` to start
   - Yes → Make `pub`

2. **Do multiple internal modules need this?**
   - No → Keep private to current module
   - Yes, within one parent → Use `pub(super)`
   - Yes, across crate → Use `pub(crate)`

3. **Is this a trait users will implement?**
   - No → Consider sealing it
   - Yes, now → Make `pub`
   - Yes, but not yet → Keep `pub(crate)`, upgrade later

4. **Does this type have invariants to maintain?**
   - Yes → Make fields private, expose methods
   - No → Public fields are acceptable (but rare in libraries)

5. **Is this likely to change?**
   - Yes → Minimize visibility to reduce breakage surface
   - No → Visibility can be more permissive

## Visibility and Documentation

Rust's documentation tools respect visibility. When you run `cargo doc`, only `pub` items appear in the generated API reference. This makes visibility a documentation tool:

```rust
/// Public function, appears in docs
pub fn simulate() {
    internal_helper();  // Called but not documented
}

/// Private helper, does not appear in docs
fn internal_helper() {
    // Implementation
}
```

Use `#[doc(hidden)]` for items that are technically public (for technical reasons like macros) but shouldn't appear in documentation:

```rust
#[doc(hidden)]
pub fn __internal_macro_helper() {
    // Used by macro expansion, but not part of the API
}
```

## Conclusion: Visibility as Architectural Tool

Visibility is not just access control—it's architectural expression. Each visibility level documents intent:

- **Private**: "This is an internal detail, subject to change."
- **`pub(super)`**: "This is part of a tightly-coupled subsystem."
- **`pub(crate)`**: "This is internal infrastructure, used across modules."
- **`pub`**: "This is a stable contract with external users."

Deliberate use of visibility makes codebases navigable, APIs stable, and refactoring safe. The compiler enforces these boundaries, catching violations at build time rather than runtime. This is architectural discipline that costs nothing at runtime but saves immeasurable maintenance burden over the lifetime of the code.
