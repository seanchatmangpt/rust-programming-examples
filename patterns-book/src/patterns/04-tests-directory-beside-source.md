# 4. TESTS DIRECTORY BESIDE SOURCE

*Integration tests that verify your code works as external users would experience it, kept separate from the implementation they test.*

...within a **RUST PROJECT STRUCTURE**, when you need to verify that your library's public API works correctly from an external perspective...

◆ ◆ ◆

**Unit tests live close to the code they test, but they see everything—private functions, internal state, implementation details. How do you test what users actually experience when they use your library?**

There are two kinds of tests, and they serve different purposes. Unit tests, marked with `#[test]` and placed inside your source files, validate internal logic. They can access private functions and inspect internal state. This is valuable—you need to test the implementation.

But users of your library don't see the implementation. They see only the public API. They import your crate and call its public functions. If your tests only verify internal details, you might miss integration problems—the ways components fail to work together, or how the public API falls short of real-world usage.

Integration tests solve this by treating your code as a black box. Tests in the `tests/` directory compile as separate crates that depend on your library. They can only access what's marked `pub`. They can't peek inside private fields or call internal helpers. This limitation is their strength.

Look at `fern_sim`'s integration test. It imports the library (`use fern_sim::Terrarium`), creates a terrarium from a file, and tests behavior over time. It doesn't care about internal simulation details. It tests what matters to users: "Can I load a terrarium and observe ferns unfurling in sunlight?"

The physical separation reinforces the conceptual separation. Source code in `src/`, tests in `tests/`. When you navigate the project structure, you immediately see two perspectives: the implementation and its verification from the outside.

**Therefore:**

**Create a `tests/` directory beside `src/`. Place integration tests there as separate `.rs` files. Each file compiles as its own test crate, accessing only your library's public API.**

```rust
// tests/unfurl.rs - Fiddleheads unfurl in sunlight

use fern_sim::Terrarium;
use std::time::Duration;

#[test]
fn test_fiddlehead_unfurling() {
    let mut world = Terrarium::load("tests/unfurl_files/fiddlehead.tm");
    assert!(world.fern(0).is_furled());

    let one_hour = Duration::from_secs(60 * 60);
    world.apply_sunlight(one_hour);

    assert!(world.fern(0).is_fully_unfurled());
}
```

*Project structure showing the separation:*
```
fern_sim/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Implementation
│   ├── simulation.rs   # Private modules
│   └── spores.rs
└── tests/
    ├── unfurl.rs       # Integration test
    └── unfurl_files/   # Test data
        └── fiddlehead.tm
```

*The test imports `fern_sim` just like an external user would. It has no special access to internals.*

◆ ◆ ◆

Integration tests naturally need **EXAMPLES DIRECTORY FOR USAGE** (5) to show users the same patterns your tests verify. Complex tests may benefit from **TEST HELPER MODULE** to share setup code. As behavior grows complex, tests help you discover which functionality should be split into **SEPARATE MODULE FILES**.
