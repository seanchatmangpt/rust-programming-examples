# 12. PRIVATE MODULE PUBLIC REEXPORT

*A module hierarchy where internal organization is hidden through pub use statements at higher levels, allowing the implementation structure to change while the public API remains stable*

...within a [NESTED SUBMODULES IN DIRECTORY](#11), when you need to hide internal organization while exposing key types...

◆ ◆ ◆

**How do you provide a clean, stable public API while maintaining the freedom to reorganize internal module structure?**

External users shouldn't care about your internal organization. If they import `fern_sim::plant_structures::leaves::Leaf`, they're coupled to your directory structure—if you later refactor `leaves` into a subdirectory or rename it, all their code breaks. The module tree serves *your* organizational needs, not theirs.

But Rust's module system is based on file paths. By default, users must traverse your entire hierarchy to reach deeply nested types. This creates brittle coupling: your internal refactoring becomes their breaking change.

Re-exporting with `pub use` breaks this coupling. The parent module can selectively expose child types at a higher level, creating a public API boundary independent of the implementation structure. The `fern_sim` crate demonstrates this: `pub use plant_structures::Fern` at the crate root means users write `use fern_sim::Fern`, not `use fern_sim::plant_structures::Fern`. The internal location is now an implementation detail.

**Therefore:**

**In parent modules (mod.rs or lib.rs), use `pub use` to re-export commonly used types from child modules, creating a stable public API that hides internal organization.**

```rust
// In lib.rs - crate root
pub mod plant_structures;  // Module is public
pub mod simulation;
pub mod spores;

// Re-export key types at the crate root
pub use plant_structures::Fern;
pub use simulation::Terrarium;

// In plant_structures/mod.rs
pub mod roots;
pub mod stems;
pub mod leaves;

// Re-export at module level
pub use self::leaves::Leaf;
pub use self::roots::Root;

// Private use - internal only
use self::roots::RootSet;
use self::stems::StemSet;
```

*The diagram shows external code importing from fern_sim::Fern while the actual definition lives in plant_structures/mod.rs, with a dotted arrow representing the pub use re-export creating the API shortcut.*

◆ ◆ ◆

This pattern requires [NESTED SUBMODULES IN DIRECTORY](#11) and enables [STABLE PUBLIC API](#api-versioning). It complements [VISIBILITY BOUNDARIES](#crate-pub).
