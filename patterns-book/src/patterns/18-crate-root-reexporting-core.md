# 18. CRATE ROOT REEXPORTING CORE

*The library's front door: a carefully curated collection of types and functions, selected from the rooms within and presented at the threshold for easy access.*

...within a **MULTI-FILE LIBRARY (13)** or **MODULES FOR ORGANIZATION (8)**, when your crate has grown to multiple modules but users shouldn't need to memorize the internal hierarchy...

◆ ◆ ◆

**How do you give users convenient access to your library's main types without forcing them to navigate a deep module tree?**

As libraries grow, modules proliferate. You organize code by concern: `plant_structures` for data types, `simulation` for algorithms, `net` for networking. This internal organization makes maintenance easier—related code lives together, and modules form clear boundaries.

But this organization creates friction for users. Should they write `use fern_sim::plant_structures::Fern` or `use fern_sim::Fern`? If they need both `Terrarium` and `connect`, must they remember that one lives in `simulation` while the other hides in `net`? Deep import paths force users to understand your implementation details.

The tension is between internal organization and external ergonomics. You want logical module structure for maintainability, but you also want users to access core types without thinking about your file layout. Your module tree serves you; the crate root should serve your users.

**Therefore:**

**In your crate root (`lib.rs`), declare modules with `pub mod`, then re-export the most important types with `pub use`. Users can import from the crate root for convenience or from submodules for specificity.**

```rust
//! Simulate the growth of ferns, from the level of
//! individual cells on up.

#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

// Declare modules (creates namespaces)
pub mod plant_structures;
pub mod simulation;
pub mod spores;
pub mod net;

// Re-export core types to crate root
pub use plant_structures::Fern;
pub use simulation::Terrarium;
pub use net::connect;
```

Now users can write either:

```rust
// Convenient—common types at crate root
use fern_sim::{Fern, Terrarium, connect};

// Explicit—full path when disambiguation needed
use fern_sim::plant_structures::Fern;
use fern_sim::simulation::Terrarium;
```

*The crate root is a curated gateway. Not everything is re-exported—only the types most users need most often. Power users can still reach into submodules for specialized types.*

◆ ◆ ◆

This pattern creates two APIs: a convenience API at the crate root for common usage, and a complete API in submodules for advanced needs. Most users import from `crate::`, enjoying short paths and minimal cognitive load. Advanced users navigate to specific modules when they need less common types or want to avoid name collisions.

The re-export list is a statement of intent. It says: "These are the primary types of this library. Start here." It guides newcomers to the most important abstractions without overwhelming them with implementation details.

Choose carefully what to re-export. Too few re-exports and users must constantly dig into modules. Too many and you pollute the crate root namespace, creating ambiguity and name collisions. Re-export types that appear in most programs using your library; keep specialized types in their modules.

Use **DESCRIPTIVE MODULE NAMES (26)** to make submodule navigation intuitive, **DOC COMMENT AT CRATE ROOT (27)** to explain the high-level organization, and **MODULES FOR ORGANIZATION (8)** to maintain clean internal boundaries.
