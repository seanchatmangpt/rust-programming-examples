# 13. FEATURE-BASED MODULE GROUPS

*Modules organized by feature domain or conceptual cohesion rather than technical layers, where each module group represents a complete slice of functionality*

...within a [HIERARCHICAL MODULE TREE](#10), when you need to organize a growing codebase by problem domain...

◆ ◆ ◆

**Should you organize modules by technical role (models, handlers, services) or by feature domain (plants, simulation, network)?**

Layer-based organization (MVC-style) scatters each feature across multiple files. To add a new plant structure, you touch `models/plants.rs`, `services/plant_service.rs`, and `handlers/plant_handlers.rs`. Understanding one feature requires jumping between distant files. Teams working on different features constantly collide in the same files.

Feature-based organization groups all code for a feature together. The `fern_sim` crate demonstrates this clearly: `plant_structures`, `simulation`, `spores`, and `net` are *domains*, not layers. Each module is cohesive—everything about plants lives in `plant_structures`, everything about simulation lives in `simulation`.

This pattern mirrors how humans think about problems. Biologists don't think "first I'll design all the data structures, then all the behaviors, then all the I/O"—they think "roots have structure and behavior, stems have structure and behavior." Feature-based organization respects this mental model.

When features are modules, changing one feature touches one subtree. Parallel development becomes easier—different developers own different feature modules. The module boundary becomes a natural team boundary.

**Therefore:**

**Organize top-level modules by feature domain or problem area, not by technical layer. Each module should contain all code related to its feature: types, implementations, tests, and internal helpers.**

```rust
// Feature-based organization (fern_sim)
pub mod plant_structures;  // All plant biology
pub mod simulation;        // All simulation logic
pub mod spores;           // All reproduction logic
pub mod net;              // All networking

// NOT layer-based:
// pub mod models;     // All data structures
// pub mod services;   // All business logic
// pub mod handlers;   // All API endpoints

// In plant_structures/ directory:
// - roots.rs, stems.rs, leaves.rs (feature components)
// - mod.rs defines Fern (feature coordinator)
```

*The diagram shows three columns: plant_structures (containing Fern, Leaf, Root, Stem in one box), simulation (containing Terrarium, run in one box), and spores (containing SporeType, reproduce in one box), emphasizing vertical feature slices rather than horizontal technical layers.*

◆ ◆ ◆

This pattern uses [NESTED SUBMODULES IN DIRECTORY](#11) and works with [SEPARATION OF CONCERNS](#separation). It enables [PARALLEL DEVELOPMENT](#teams) and [FEATURE FLAGS](#conditional).
