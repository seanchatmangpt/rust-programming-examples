# API Surface Management

The public API of a crate is a contract with users. Once published, breaking changes require semver major version bumps and force downstream users to update their code. Rust's module system provides powerful tools for managing this contract deliberately, allowing library authors to evolve internal implementations while maintaining API stability.

## The Facade Pattern: lib.rs as API Gateway

The `lib.rs` file in a Rust crate serves as the API gateway—the single point of entry for external consumers. Its primary role is to curate what the crate exposes, creating a clear distinction between public API and internal implementation.

### Strategic Re-Exports

In `fern_sim/src/lib.rs`, we see careful selection of what to re-export:

```rust
pub mod plant_structures;
pub mod simulation;
pub mod spores;

pub use plant_structures::Fern;
pub use simulation::Terrarium;

pub mod net;
pub use net::connect;
```

Three patterns are at play:

**Pattern 1: Module Without Re-Export**
```rust
pub mod spores;
```
The `spores` module is public, but no types are promoted to the crate root. Users must write:
```rust
use fern_sim::spores::Spore;
```
This signals: "Spores exist, but they're not central to typical usage."

**Pattern 2: Type Re-Export**
```rust
pub use plant_structures::Fern;
```
The `Fern` type is promoted, allowing:
```rust
use fern_sim::Fern;
```
This signals: "Fern is a core concept. Most users will need it."

**Pattern 3: Function Re-Export**
```rust
pub use net::connect;
```
The `connect` function is promoted from its module. Users can write:
```rust
use fern_sim::connect;
```
instead of:
```rust
use fern_sim::net::connect;
```

These choices are **architectural decisions**. Re-exporting creates API surface; every re-export is a promise to maintain compatibility.

## The Prelude Pattern: Convenience Without Commitment

Many Rust standard library crates use a `prelude` module for commonly-imported items. While `fern_sim` doesn't include one, let's explore how it would look:

```rust
// In src/lib.rs
pub mod prelude {
    pub use crate::plant_structures::Fern;
    pub use crate::simulation::Terrarium;
    pub use crate::spores::{Spore, Sporangium};
}
```

Users then write:
```rust
use fern_sim::prelude::*;

let fern = Fern::new(FernType::Fiddlehead);
let mut terrarium = Terrarium::new();
```

### Prelude Design Guidelines

A prelude should contain:
- **Common types** users interact with directly (`Fern`, `Terrarium`)
- **Essential traits** needed for basic usage
- **Frequently-used functions** that appear in most code

A prelude should **not** contain:
- **Everything** in the crate (defeats the purpose)
- **Rarely-used types** (pollutes the namespace)
- **Types with common names** that might conflict (e.g., `Error`, `Result`)

The standard library's `std::prelude` contains only the most fundamental items: `Option`, `Result`, `Vec`, `String`, iterator traits, and a few others. This conservative approach prevents namespace pollution while maximizing convenience.

## Sealed Traits: Controlled Extension

Sometimes you want to provide a trait for users to consume but prevent them from implementing it. This is called a "sealed trait" pattern. While not present in `fern_sim`, it's a critical API design tool.

### The Problem: Uncontrolled Implementation

Consider a trait for plant growth stages:

```rust
// BAD: Users can implement this
pub trait GrowthStage {
    fn duration(&self) -> Duration;
    fn next_stage(&self) -> Option<Box<dyn GrowthStage>>;
}
```

If users implement `GrowthStage` on their own types, they become part of your API contract. You cannot add new methods without breaking their code.

### The Solution: The Sealed Trait Pattern

```rust
// In src/lib.rs or src/sealed.rs
mod sealed {
    pub trait Sealed {}
}

pub trait GrowthStage: sealed::Sealed {
    fn duration(&self) -> Duration;
    fn next_stage(&self) -> Option<Box<dyn GrowthStage>>;
}

// Only internal types can implement GrowthStage
impl sealed::Sealed for Seedling {}
impl GrowthStage for Seedling {
    fn duration(&self) -> Duration { Duration::from_days(7) }
    fn next_stage(&self) -> Option<Box<dyn GrowthStage>> {
        Some(Box::new(Fiddlehead))
    }
}
```

The `sealed::Sealed` trait is in a private module. External crates cannot implement it, and therefore cannot implement `GrowthStage`. This allows the crate to:
- Add new methods to `GrowthStage` in minor versions (non-breaking)
- Control which types are valid `GrowthStage` implementations
- Make assumptions about behavior that hold for all implementors

### When to Seal Traits

Seal traits when:
- The trait represents an **internal abstraction** (e.g., internal iterators)
- You plan to **add methods** in the future
- **Correctness depends on invariants** that external implementations might violate

Don't seal traits when:
- The trait is explicitly **for extension** (e.g., `Serialize`, `Clone`)
- It represents a **stable interface** unlikely to change
- **Third-party types** legitimately should implement it

## API Stability Through Module Design

Module organization directly impacts API stability. Consider two approaches to organizing plant structures:

### Approach 1: Flat and Exposed

```rust
// BAD for stability
pub mod roots {
    pub struct Root { pub x: bool, pub y: bool }
    pub type RootSet = Vec<Root>;
}

pub mod stems {
    pub struct Stem { pub furled: bool }
    pub type StemSet = Vec<Stem>;
}

pub struct Fern {
    pub roots: roots::RootSet,
    pub stems: stems::StemSet,
}
```

Every detail is public. Users can:
- Construct `Root` and `Stem` directly
- Inspect and mutate all fields
- Create their own `RootSet` and `StemSet`

This is maximally flexible but provides **zero** API stability. Any change to fields, types, or structure is a breaking change.

### Approach 2: Encapsulated and Controlled

```rust
// GOOD for stability
pub mod plant_structures {
    pub use self::roots::Root;
    pub use self::leaves::Leaf;

    use self::roots::RootSet;  // Not re-exported!
    use self::stems::StemSet;  // Not re-exported!

    pub struct Fern {
        pub(crate) roots: RootSet,  // Visible only within crate
        pub(crate) stems: StemSet,
    }

    impl Fern {
        pub fn root_count(&self) -> usize { self.roots.len() }
        pub fn stem_count(&self) -> usize { self.stems.len() }
    }

    mod roots { /* ... */ }
    mod stems { /* ... */ }
    mod leaves { /* ... */ }
}
```

Users can access `Root` and `Leaf` by name, but:
- Cannot construct `Fern` with arbitrary roots/stems
- Cannot see or mutate `Fern`'s internal structure
- Use methods (`root_count()`, `stem_count()`) instead of direct field access

Now the implementation can change:
```rust
// Internal change, no API break
struct Fern {
    roots: HashMap<RootId, Root>,  // Changed from Vec
    stems: BTreeSet<Stem>,         // Changed from Vec
}
```

As long as `root_count()` and `stem_count()` still work, external code is unaffected.

## Evolutionary API Design

APIs evolve over time. Rust provides tools to manage this evolution gracefully.

### Deprecation Warnings

```rust
#[deprecated(since = "1.2.0", note = "use `Fern::new` instead")]
pub fn create_fern() -> Fern {
    Fern::new(FernType::Fiddlehead)
}
```

The compiler warns users at compile time, giving them advance notice to migrate before the function is removed in a future major version.

### Re-Exports for Compatibility

When refactoring internal organization, re-exports maintain old paths:

```rust
// v1.0: Originally here
pub mod old_location {
    pub struct ImportantType;
}

// v1.5: Refactored, but compatibility maintained
pub mod old_location {
    #[deprecated(note = "use `new_location::ImportantType`")]
    pub use crate::new_location::ImportantType;
}

pub mod new_location {
    pub struct ImportantType;
}
```

Users still writing `use mycrate::old_location::ImportantType` get a warning but no error. They can migrate at their convenience.

### Type Aliases for Flexibility

```rust
pub type RootSet = Vec<Root>;
```

This looks like a concrete type to users, but it's an alias. The implementation can change:
```rust
pub type RootSet = std::collections::HashSet<Root>;
```

Users still write `RootSet`, but the internal representation evolves. **Caveat**: Type aliases don't provide true encapsulation—if `RootSet` is `pub`, users can see through the alias. For true abstraction, use newtype wrappers:

```rust
pub struct RootSet(Vec<Root>);

impl RootSet {
    pub fn len(&self) -> usize { self.0.len() }
    pub fn iter(&self) -> impl Iterator<Item = &Root> {
        self.0.iter()
    }
}
```

Now `RootSet` is opaque. The internal `Vec` can become a `HashSet` without breaking users.

## Case Study: Real-World API Surface from `grep`

The `grep` project is a single-file binary, but it demonstrates API surface thinking even within one file:

```rust
// Internal: Generic over BufRead
fn grep<R>(target: &str, reader: R) -> io::Result<()>
    where R: BufRead
{ /* ... */ }

// Public: Entry point with concrete types
fn grep_main() -> Result<(), Box<dyn Error>> {
    // Handles args, files, calls grep()
}

fn main() {
    let result = grep_main();
    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
```

Even though everything is in one file, the structure shows:
- **`grep<R>`**: Internal implementation, generic and reusable
- **`grep_main()`**: Boundary between CLI parsing and core logic
- **`main()`**: Thin wrapper for error handling

If `grep` were refactored into a library, `grep<R>` would become `pub fn grep<R>` in `lib.rs`, while `grep_main()` and `main()` would move to `main.rs`. The API surface is already designed, just not yet extracted.

## Best Practices Summary

When designing API surfaces:

1. **Re-export deliberately**: Every re-export is a compatibility promise.

2. **Use preludes sparingly**: Only the most common items; avoid polluting the namespace.

3. **Seal traits that aren't extension points**: Control implementation and enable future additions.

4. **Hide internal types**: Use `pub(crate)` for internal APIs, `pub` only for external contracts.

5. **Provide methods over field access**: Methods can evolve; direct field access cannot.

6. **Use deprecation warnings**: Give users time to migrate before breaking changes.

7. **Type aliases for documentation**: But newtype wrappers for true abstraction.

8. **Test the external perspective**: Write integration tests using only the public API. If it's awkward, the API needs work.

The module system is more than organization—it's the mechanism for defining, enforcing, and evolving the contract between your crate and its users. A well-designed API surface enables rapid internal iteration while maintaining stability for dependents.
