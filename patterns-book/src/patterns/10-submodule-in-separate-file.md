# 10. SUBMODULE IN SEPARATE FILE

*A focused module implementing one cohesive concern, living in its own file within the crate structure*

...within a **MODULE TREE IN LIB FILE**, when a module's implementation has grown beyond a few dozen lines and deserves its own file...

◆ ◆ ◆

**The separation question: As modules grow, keeping them inline in `lib.rs` clutters the root file with implementation details. When code belongs together conceptually but takes too much space, where does it live?**

Rust's file-system mapping makes the answer elegant: a module declared as `pub mod simulation;` in `lib.rs` loads its contents from `simulation.rs` (or `simulation/mod.rs` if it has submodules). This convention creates a direct correspondence between module paths and file paths—`crate::simulation::Terrarium` lives in `src/simulation.rs`.

The fern_sim simulation module demonstrates this. The `simulation.rs` file contains 50+ lines implementing the `Terrarium` type and its methods. This code logically belongs together—it's all about the simulation subsystem—but mixing it into `lib.rs` would obscure the library's structure. The separate file provides breathing room for implementation details, documentation, and tests.

Notice how the file begins with a module-level doc comment (`//!`) explaining its purpose. This comment becomes the module's documentation in `cargo doc`. The file uses `crate::` paths to import from other modules, making dependencies explicit. It can have private helper functions and types that never appear in the public API.

The pattern scales beautifully. As `simulation` grows more complex, you can convert `simulation.rs` into `simulation/mod.rs` and add submodules like `simulation/state.rs` or `simulation/events.rs`. The root `lib.rs` declaration stays the same—`pub mod simulation;`—while the internal organization evolves.

**Therefore:**

**Place each module's implementation in its own file named `module_name.rs` in the `src/` directory. Begin the file with module-level documentation (`//!`). Use `pub` strategically to export only the types and functions that belong in the module's public API. Import from other modules using `crate::` paths to make dependencies clear.**

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

impl Terrarium {
    /// Create a new empty terrarium.
    pub fn new() -> Terrarium {
        Terrarium { ferns: vec![] }
    }

    /// Load a terrarium from a `.tm` file.
    pub fn load(filename: &str) -> Terrarium {
        File::open(filename).unwrap();
        Terrarium {
            ferns: vec![
                Fern::new(FernType::Fiddlehead)
            ]
        }
    }

    /// Let the sun shine in and run the simulation for a given
    /// amount of time.
    pub fn apply_sunlight(&mut self, time: Duration) {
        for f in &mut self.ferns {
            for s in &mut f.stems {
                s.furled = false;
            }
        }
    }
}
```

*Each module file becomes a focused unit of code, comprehensible in isolation while connected to the larger system through explicit imports and the lib.rs declaration*

◆ ◆ ◆

This separation enables further refinement through **NESTED SUBMODULES** (converting the file into a directory with `mod.rs`), **VISIBILITY BOUNDARIES** within the module, and **SEPARATION OF CONCERNS** where each file addresses one aspect of the system. The file becomes a natural unit for code review, testing, and understanding.
