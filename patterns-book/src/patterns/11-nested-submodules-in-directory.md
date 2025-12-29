# 11. NESTED SUBMODULES IN DIRECTORY

*A module directory containing a mod.rs file and several .rs submodule files, with the mod.rs serving as the gateway that declares and aggregates the submodules*

...within a [HIERARCHICAL MODULE TREE](#10), when you need to group several related modules under a common parent...

◆ ◆ ◆

**How do you organize multiple related modules under a single parent module when they share common types and need a clear entry point?**

When modules grow beyond simple siblings, you need a place to put shared types, re-exports, and declarations that bind the submodules together. A single `.rs` file cannot contain both module declarations and serve as the parent's implementation. The compiler needs to know where to find submodules, and developers need to understand what's public from this module family.

The `mod.rs` pattern solves this by treating a directory as a module. The `mod.rs` file becomes the module's root—it declares child modules, defines shared types, controls visibility through re-exports, and serves as the conceptual gateway. This mirrors how `lib.rs` works for a crate, but at the module level.

Consider the `plant_structures` module from the fern simulator: it needs to organize `roots`, `stems`, and `leaves` modules, while also defining the parent `Fern` type that composes them. Without a dedicated parent file, this coordination would scatter across unrelated files.

**Therefore:**

**Create a directory with the module's name, place a `mod.rs` file inside to declare submodules and shared types, and put each submodule in its own `.rs` file within that directory.**

```rust
// In plant_structures/mod.rs
pub mod roots;
pub mod stems;
pub mod leaves;

// Shared types defined at the parent level
pub use self::leaves::Leaf;
pub use self::roots::Root;

use self::roots::RootSet;
use self::stems::StemSet;

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

*The directory structure shows plant_structures/ containing mod.rs (the aggregator), leaves.rs, roots.rs, and stems.rs (the implementations), creating a clear parent-child hierarchy in the file system.*

◆ ◆ ◆

This pattern enables [FEATURE-BASED MODULE GROUPS](#13), [PRIVATE MODULE PUBLIC REEXPORT](#12), and provides the foundation for [DEEPLY NESTED MODULES](#stems-within-structures).
