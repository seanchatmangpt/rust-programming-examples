# 9. MODULE TREE IN LIB FILE

*The library root file that declares the module hierarchy and establishes the public API surface*

...within a **HIERARCHICAL MODULES** architecture, when you have organized code into separate files but need to connect them into a coherent library...

◆ ◆ ◆

**The organizational question: You've separated concerns into focused modules, each in its own file, but Rust doesn't automatically discover them. How do you declare the structure of your library and control what external code can access?**

Rust's module system requires explicit declaration. Unlike languages that use file-system scanning, Rust needs you to state which files are modules. This is intentional—it gives you precise control over visibility and API boundaries. But it means the library root (`lib.rs` or `main.rs`) must serve as the architectural manifest, declaring the module tree.

The fern_sim project shows this pattern clearly. The `lib.rs` file contains module declarations (`pub mod plant_structures;`, `pub mod simulation;`) that pull in code from separate files. It uses `pub use` statements to re-export important types, creating a convenient API surface. It sets crate-level attributes like `#![warn(rust_2018_idioms)]` that apply to the entire library.

This file is unusually important—it's the entry point for understanding the codebase. A developer reading `lib.rs` sees the complete module structure at a glance: what major subsystems exist, which types are publicly exported, what the library's concerns are. It's simultaneously a table of contents, an index, and an architectural diagram.

The pattern also establishes privacy boundaries. Modules not declared here remain private implementation details. Types not re-exported require longer import paths. This gives library authors control over API stability—you can refactor internal module structure without breaking external code, as long as re-exports remain consistent.

**Therefore:**

**Use your `lib.rs` (or `main.rs`) as the architectural root: declare all top-level modules with `pub mod`, re-export key types with `pub use` to simplify imports, set crate-level attributes and documentation. Keep implementation details in the separate module files, using the root file only for structure declaration.**

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

*The lib.rs file acts as a gateway, channeling external access through declared paths while keeping the internal module organization flexible*

◆ ◆ ◆

This declaration structure enables **SUBMODULE IN SEPARATE FILE**, **RE-EXPORTING** for API convenience, and **VISIBILITY BOUNDARIES** to control access. Each module file can now focus on its specific concern, trusting that the root file connects them properly.
