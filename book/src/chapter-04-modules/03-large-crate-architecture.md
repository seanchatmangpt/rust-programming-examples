# Large Crate Architecture

As Rust projects grow from hundreds to thousands to tens of thousands of lines, architectural discipline becomes critical. The module system provides the structural tools, but success requires deliberate design patterns, clear separation of concerns, and strategies for managing complexity.

## Scaling Challenges in Large Codebases

The `fern_sim` project, at around 300 lines, demonstrates good structure for a small-to-medium library. But real-world projects face challenges that emerge only at scale:

### Challenge 1: Module Interdependencies

As module count grows, dependency graphs become complex. Without discipline, you end up with:

```
simulation → plant_structures → roots
          ↓                    ↗
       spores ─────────────────
```

This web of dependencies makes changes risky—modifying `roots` might break `simulation` through an indirect path via `spores`.

### Challenge 2: Compilation Time

Rust's compilation units are crates, not modules. Every change to a module requires recompiling all modules that depend on it, transitively. A change to a low-level module can trigger a complete rebuild.

### Challenge 3: Cognitive Overload

A flat list of 50 modules in `lib.rs` is overwhelming:

```rust
pub mod accounting;
pub mod analytics;
pub mod authentication;
pub mod authorization;
pub mod billing;
// ... 45 more modules
```

Developers spend time navigating rather than building.

## Hierarchical Module Organization

The solution is hierarchical organization with clear boundaries. Let's evolve `fern_sim` as it scales from a toy example to a realistic simulation engine.

### Stage 1: Small Crate (Current `fern_sim`)

```
fern_sim/
├── lib.rs              # 16 lines
├── simulation.rs       # 53 lines
├── spores.rs          # 48 lines
└── plant_structures/  # 100 lines total
    ├── mod.rs
    ├── roots.rs
    ├── leaves.rs
    └── stems.rs
```

Total: ~300 lines. Everything fits in working memory. Module relationships are simple.

### Stage 2: Medium Crate (Realistic Growth)

```
fern_sim/
├── lib.rs
├── core/              # Core domain model
│   ├── mod.rs
│   ├── organisms/
│   │   ├── mod.rs
│   │   ├── ferns.rs
│   │   ├── mosses.rs
│   │   └── lichens.rs
│   └── environment/
│       ├── mod.rs
│       ├── terrarium.rs
│       └── climate.rs
├── simulation/        # Simulation engines
│   ├── mod.rs
│   ├── physics.rs
│   ├── growth.rs
│   └── reproduction.rs
├── io/                # Input/output
│   ├── mod.rs
│   ├── persistence.rs
│   └── serialization.rs
└── network/           # Network protocols
    ├── mod.rs
    └── sync.rs
```

Total: ~3,000 lines. Modules are grouped by architectural layer: `core` (domain), `simulation` (engine), `io` (infrastructure), `network` (external interface).

### Stage 3: Large Crate (Production Scale)

```
fern_sim/
├── lib.rs
├── domain/            # Domain model (aggregate roots)
│   ├── mod.rs
│   ├── organisms/     # 15+ files
│   ├── genetics/      # 10+ files
│   └── lifecycle/     # 8+ files
├── engine/            # Simulation engine
│   ├── mod.rs
│   ├── physics/       # 12+ files
│   ├── chemistry/     # 10+ files
│   └── time/          # 5+ files
├── storage/           # Persistence layer
│   ├── mod.rs
│   ├── snapshot.rs
│   └── replay.rs
├── rendering/         # Visualization (optional feature)
│   ├── mod.rs
│   └── webgl/         # 20+ files
└── api/               # Public API facade
    ├── mod.rs
    └── prelude.rs
```

Total: ~30,000 lines. At this scale, each top-level module is its own mini-crate with internal organization. The root `lib.rs` imports from `api`, which re-exports selected items from other modules.

## Separation of Concerns: Architectural Layers

Large crates benefit from explicit architectural layers. Each layer has a specific responsibility and dependency rules.

### The Dependency Rule

**Dependencies flow inward**: Outer layers depend on inner layers, never the reverse.

```
┌─────────────────────────────────────┐
│  API Layer (api/)                   │  ← Public interface
│    depends on ↓                     │
├─────────────────────────────────────┤
│  Application Layer (simulation/)    │  ← Use cases
│    depends on ↓                     │
├─────────────────────────────────────┤
│  Domain Layer (core/)               │  ← Business logic
│    depends on nothing               │
└─────────────────────────────────────┘
```

Infrastructure layers (storage, network) depend on the domain but are imported only where needed.

### Example: Layered Error Handling

The domain layer defines core errors:

```rust
// In domain/mod.rs
pub enum DomainError {
    InvalidGrowthRate(f64),
    DeadOrganism,
}
```

The engine layer adds simulation-specific errors:

```rust
// In engine/mod.rs
use crate::domain::DomainError;

pub enum SimulationError {
    Domain(DomainError),
    PhysicsViolation(String),
    TimestepTooLarge,
}

impl From<DomainError> for SimulationError {
    fn from(err: DomainError) -> Self {
        SimulationError::Domain(err)
    }
}
```

The API layer exposes user-facing errors:

```rust
// In api/mod.rs
use crate::engine::SimulationError;

pub enum ApiError {
    Simulation(SimulationError),
    InvalidInput(String),
}
```

Each layer handles errors at its own level of abstraction. See Chapter 5 for detailed error handling patterns.

## Managing Module Interdependencies

### Anti-Pattern: Circular Dependencies

```rust
// BAD: Circular dependency
// In simulation.rs
use crate::plant_structures::Fern;

pub struct Terrarium {
    ferns: Vec<Fern>,
}

// In plant_structures/mod.rs
use crate::simulation::Terrarium;  // ERROR: circular!

pub struct Fern {
    home: Terrarium,  // Fern knows its terrarium?
}
```

Rust forbids this. The compiler error is:

```
error[E0369]: cyclic modules: simulation -> plant_structures -> simulation
```

### Solution 1: Reverse the Dependency

Often one dependency is less essential:

```rust
// GOOD: Only one dependency direction
// simulation.rs owns the relationship
pub struct Terrarium {
    ferns: Vec<Fern>,
}

// plant_structures/mod.rs
pub struct Fern {
    // No reference to Terrarium
}
```

The `Terrarium` knows about `Fern`, but `Fern` doesn't need to know about `Terrarium`.

### Solution 2: Introduce a Trait

If bidirectional knowledge is truly needed, abstract with traits:

```rust
// In core/mod.rs (new module)
pub trait Environment {
    fn temperature(&self) -> f64;
    fn humidity(&self) -> f64;
}

// In plant_structures/mod.rs
use crate::core::Environment;

pub struct Fern {
    pub fn growth_rate(&self, env: &dyn Environment) -> f64 {
        env.temperature() * env.humidity()
    }
}

// In simulation.rs
use crate::core::Environment;
use crate::plant_structures::Fern;

pub struct Terrarium { /* ... */ }

impl Environment for Terrarium {
    fn temperature(&self) -> f64 { 20.0 }
    fn humidity(&self) -> f64 { 0.8 }
}
```

Now `Fern` depends on the `Environment` trait (in `core`), and `Terrarium` implements it. No circular dependency.

### Solution 3: Extract Common Types

Sometimes modules share concepts that belong in a separate module:

```rust
// In types/mod.rs (new module)
pub struct Duration(f64);
pub struct Position { x: f64, y: f64 }

// Both simulation.rs and plant_structures/mod.rs can use types/
// without depending on each other
```

## Internal vs. Public Implementation Details

Large crates accumulate internal helper modules. Use `pub(crate)` to hide them from external users while keeping them accessible internally.

### The `pub(crate)` Strategy

```rust
// In utils/mod.rs (utility module)
pub(crate) fn calculate_fibonacci(n: u32) -> u64 {
    // Complex implementation used across modules
}

// In lib.rs
mod utils;  // Not pub!

pub mod simulation;
pub mod plant_structures;
```

The `utils` module is private to the crate. `simulation` and `plant_structures` can both import from it:

```rust
// In simulation.rs
use crate::utils::calculate_fibonacci;
```

But external users cannot:

```rust
use fern_sim::utils::calculate_fibonacci;  // ERROR: module is private
```

### When to Use `pub(crate)`

Use `pub(crate)` for:
- **Internal utilities** (parsing, formatting, math)
- **Shared helper types** used across modules
- **Test infrastructure** (factories, fixtures)
- **Traits for internal abstraction** (not for external extension)

Keep `pub` for:
- **API types and functions** external users need
- **Extension traits** meant for downstream implementation
- **Public examples** shown in documentation

## Case Study: The `copy` Utility Architecture

The `copy` program, while a single-file binary, shows how internal organization prevents feature creep:

```rust
// copy/src/main.rs structure

// LAYER 1: Core algorithm (platform-agnostic)
fn copy_to(src: &Path, src_type: &fs::FileType, dst: &Path) -> io::Result<()> {
    if src_type.is_file() {
        fs::copy(src, dst)?;
    } else if src_type.is_dir() {
        copy_dir_to(src, dst)?;
    } else if src_type.is_symlink() {
        let target = src.read_link()?;
        symlink(target, dst)?;
    }
    // ...
}

// LAYER 2: Platform-specific adaptations
#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(not(unix))]
fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, _dst: Q) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Other, "symlinks unsupported"))
}

// LAYER 3: User-facing logic
fn dwim_copy<P, Q>(source: P, destination: Q) -> io::Result<()>
    where P: AsRef<Path>, Q: AsRef<Path>
{
    // "Do What I Mean" heuristics
}

// LAYER 4: CLI entry point
fn copy_main() -> io::Result<()> {
    let args = std::env::args_os().collect::<Vec<_>>();
    // Argument parsing and validation
}

fn main() {
    if let Err(err) = copy_main() {
        writeln!(io::stderr(), "error: {}", err).unwrap();
    }
}
```

Even in a single file, the layers are clear:
1. **Core algorithm**: Pure logic, platform-agnostic
2. **Platform shims**: `cfg` conditional compilation
3. **User convenience**: Higher-level wrappers
4. **CLI interface**: Argument parsing

If `copy` were refactored into a library, each layer would become a module:

```
copy_lib/
├── lib.rs
├── core.rs        # copy_to, copy_dir_to
├── platform.rs    # symlink adaptations
├── api.rs         # dwim_copy
└── bin/
    └── copy.rs    # CLI wrapper
```

The structure is already there, just not yet extracted.

## Strategies for Growing Codebases

### Strategy 1: Extract Early, Extract Often

When a module exceeds ~500 lines, split it:

```rust
// Before: simulation.rs (800 lines)
mod simulation {
    // Physics code
    // Growth code
    // Time code
}

// After: simulation/ (3 files × 300 lines)
mod simulation {
    pub mod physics;
    pub mod growth;
    pub mod time;
}
```

### Strategy 2: Introduce Facades as You Grow

When internal modules reach 10+, add an intermediate facade:

```rust
// Before: lib.rs with 15 modules
pub mod accounting;
pub mod analytics;
// ... 13 more

// After: Grouped into domains
pub mod finance {  // Finance domain
    pub mod accounting;
    pub mod billing;
    pub mod invoicing;
}

pub mod insights {  // Insights domain
    pub mod analytics;
    pub mod reporting;
}
```

### Strategy 3: Use Workspaces for Very Large Projects

When a crate exceeds ~50,000 lines, consider splitting into a workspace:

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["core", "engine", "api", "cli"]

# core/Cargo.toml
[package]
name = "fern_sim_core"

# engine/Cargo.toml
[dependencies]
fern_sim_core = { path = "../core" }
```

Now `core`, `engine`, `api`, and `cli` are separate crates compiled independently. Changes to `api` don't trigger recompilation of `core`.

## Conclusion: Architecture as Evolution

Large crate architecture is not a one-time design—it's continuous evolution. The module system supports this evolution through:

- **Hierarchical organization** that groups related concerns
- **Explicit dependency management** that prevents tangled graphs
- **Visibility controls** (`pub`, `pub(crate)`, private) that enforce boundaries
- **Re-exports** that decouple internal structure from public API

Start simple, refactor as you grow, and let the module tree document your architectural decisions. The compiler enforces what documentation cannot: clean boundaries, clear dependencies, and stable APIs.
