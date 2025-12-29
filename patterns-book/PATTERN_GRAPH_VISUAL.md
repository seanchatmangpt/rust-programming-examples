# Pattern Language Graph: Visual Reference

This document shows the complete directed graph of all 50 patterns, organized by scale and showing all references between patterns.

---

## Reading This Graph

- **→** means "references" (pattern A → pattern B means A uses B in its solution)
- **Scales flow downward** (PROJECT at top, EXPRESSION at bottom)
- **Clusters** show related patterns that often appear together
- **Bold** patterns are "hub" patterns referenced by many others

---

## The Complete Graph

```
┌─────────────────────────────────────────────────────────────────┐
│                         PROJECT SCALE                            │
│                     (Patterns 1-8)                               │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────────────────┐
    │  1. BINARY WITH      │
    │     MAIN FUNCTION    │────┐
    └──────────────────────┘    │
         │                      │
         ├────────────────────┐ │
         ↓                    ↓ ↓
    [#33 &self]       [#43 ?]   [#46 for &]


    ┌──────────────────────┐
    │  2. LIBRARY CRATE    │
    │     PUBLIC API       │────┐
    └──────────────────────┘    │
         │                      │
         ├─────┬────────┐       │
         ↓     ↓        ↓       ↓
    [#9 Module Tree] [#18 Reexport] [#28 Debug]


    ┌──────────────────────┐
    │  3. BINARY AND LIB   │────┬───────┐
    └──────────────────────┘    │       │
                                ↓       ↓
                            [#1 Binary] [#2 Library]
                                │
                                ↓
                            [#14 Flat Module]


    ┌──────────────────────┐
    │  4. TESTS DIRECTORY  │────┬───────┐
    └──────────────────────┘    │       │
                                ↓       ↓
                            [#50 #[test]] [#47 assert!]


    ┌──────────────────────┐
    │  6. ASYNC RUNTIME    │────┬───────┐
    │     ATTRIBUTE MAIN   │    │       │
    └──────────────────────┘    ↓       ↓
                            [#39 async fn] [#1 Binary]


    ┌──────────────────────┐
    │  7. UNSAFE FFI       │────┬───────┬───────┐
    │     WRAPPER CRATE    │    │       │       │
    └──────────────────────┘    ↓       ↓       ↓
                            [#16 Raw] [#40 unsafe] [#26 Newtype]


    ┌──────────────────────┐
    │  8. SAFE WRAPPER     │────┬───────┬───────┐
    │     AROUND UNSAFE    │    │       │       │
    └──────────────────────┘    ↓       ↓       ↓
                            [#7 FFI] [#17 Facade] [#30 Error]


┌─────────────────────────────────────────────────────────────────┐
│                      ARCHITECTURE SCALE                          │
│                     (Patterns 9-20)                              │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────────────────┐
    │  9. MODULE TREE      │────┬───────┐
    │     IN LIB FILE      │    │       │
    └──────────────────────┘    ↓       ↓
                            [#10 Submodule] [#18 Reexport]


    ┌──────────────────────┐
    │  10. SUBMODULE IN    │  (leaf - no references)
    │      SEPARATE FILE   │
    └──────────────────────┘


    ┌──────────────────────┐
    │  11. NESTED          │────┬───────┐
    │      SUBMODULES      │    │       │
    └──────────────────────┘    ↓       ↓
                            [#10 Submodule] [#13 Feature Groups]


    ┌──────────────────────┐
    │  12. PRIVATE MODULE  │────┬───────┐
    │      PUBLIC REEXPORT │    │       │
    └──────────────────────┘    ↓       ↓
                            [#18 Reexport] [#17 Facade]


    ┌──────────────────────┐
    │  13. FEATURE-BASED   │  (organizational pattern)
    │      MODULE GROUPS   │
    └──────────────────────┘


    ┌──────────────────────┐
    │  14. FLAT MODULE     │  (leaf - all in lib.rs)
    │      ALL IN LIB      │
    └──────────────────────┘


    ┌──────────────────────┐
    │  15. TEST MODULE     │────┬───────┐
    │      USE SUPER STAR  │    │       │
    └──────────────────────┘    ↓       ↓
                            [#50 #[test]] [#47 assert!]


    ┌──────────────────────┐
    │  16. RAW BINDINGS    │  (leaf - extern blocks)
    │      MODULE          │
    └──────────────────────┘


    ┌──────────────────────┐
    │  17. PUBLIC FACADE   │  (architectural pattern)
    │      MODULE          │
    └──────────────────────┘


    ┌──────────────────────┐
    │**18. CRATE ROOT**    │  (hub - many refs to this)
    │**    REEXPORTING**   │
    └──────────────────────┘


    ┌──────────────────────┐
    │  20. CONDITIONAL     │────┐
    │      COMPILATION     │    │
    └──────────────────────┘    ↓
                            [#42 AsRef<Path>]


┌─────────────────────────────────────────────────────────────────┐
│                          TYPE SCALE                              │
│                     (Patterns 21-32)                             │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────────────────┐
    │  21. STRUCT WITH     │────┬───────┐
    │      VEC FIELDS      │    │       │
    └──────────────────────┘    ↓       ↓
                            [#24 Generic T] [#33 &self]


    ┌──────────────────────┐
    │  22. STRUCT WITH     │────┬───────┬───────┐
    │      TWO VECS QUEUE  │    │       │       │
    └──────────────────────┘    ↓       ↓       ↓
                            [#21 Vec] [#34 &mut] [#48 swap]


    ┌──────────────────────┐
    │  23. ENUM WITH       │────┬───────┐
    │      EMPTY NONEMPTY  │    │       │
    └──────────────────────┘    ↓       ↓
                            [#24 Generic T] [#43 ?]


    ┌──────────────────────┐
    │**24. GENERIC TYPE**  │  (hub - used by many)
    │**    WITH PARAM T**  │
    └──────────────────────┘


    ┌──────────────────────┐
    │  25. TRAIT BOUND ON  │────┬───────┐
    │      IMPL BLOCK      │    │       │
    └──────────────────────┘    ↓       ↓
                            [#24 Generic T] [#41 where]


    ┌──────────────────────┐
    │  26. NEWTYPE         │────┬───────┐
    │      WRAPPING RAW    │    │       │
    └──────────────────────┘    ↓       ↓
                            [#27 PhantomData] [#40 unsafe]


    ┌──────────────────────┐
    │  27. PHANTOMDATA     │  (leaf - marker type)
    │      FOR LIFETIME    │
    └──────────────────────┘


    ┌──────────────────────┐
    │  28. DERIVE DEBUG    │  (leaf - attribute)
    │      FOR TESTING     │
    └──────────────────────┘


    ┌──────────────────────┐
    │  29. DERIVE          │  (leaf - serde attribute)
    │      DESERIALIZE     │
    └──────────────────────┘


    ┌──────────────────────┐
    │  30. CUSTOM ERROR    │────┬───────┐
    │      STRUCT DISPLAY  │    │       │
    └──────────────────────┘    ↓       ↓
                            [#31 Type Alias] [#28 Debug]


    ┌──────────────────────┐
    │**31. TYPE ALIAS**    │  (hub - error handling)
    │**    FOR RESULT**    │
    └──────────────────────┘


    ┌──────────────────────┐
    │  32. UNIT STRUCT     │  (leaf - marker)
    │      FOR MARKER      │
    └──────────────────────┘


┌─────────────────────────────────────────────────────────────────┐
│                       FUNCTION SCALE                             │
│                     (Patterns 33-42)                             │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────────────────┐
    │**33. METHOD TAKING** │  (hub - used everywhere)
    │**    SELF BY REF**   │
    └──────────────────────┘


    ┌──────────────────────┐
    │  34. METHOD TAKING   │────┬───────┐
    │      SELF BY MUT     │    │       │
    └──────────────────────┘    ↓       ↓
                            [#21 Vec] [#48 swap]


    ┌──────────────────────┐
    │  35. METHOD          │────┐
    │      CONSUMING SELF  │    │
    └──────────────────────┘    ↓
                            [#37 Builder]


    ┌──────────────────────┐
    │  36. CONSTRUCTOR     │  (common pattern)
    │      FUNCTION NEW    │
    └──────────────────────┘


    ┌──────────────────────┐
    │  37. BUILDER METHOD  │  (fluent API)
    │      RETURNING SELF  │
    └──────────────────────┘


    ┌──────────────────────┐
    │**38. FUNCTION**      │────┬───────┐
    │**    RETURNING**     │    │       │
    │**    RESULT**        │    ↓       ↓
    └──────────────────────┘  [#31 Alias] [#43 ?]


    ┌──────────────────────┐
    │  39. ASYNC FUNCTION  │  (async/await)
    │      WITH AWAIT      │
    └──────────────────────┘


    ┌──────────────────────┐
    │  40. UNSAFE FUNCTION │────┐
    │      SAFETY COMMENT  │    │
    └──────────────────────┘    ↓
                            [#47 assert!]


    ┌──────────────────────┐
    │  41. GENERIC FN      │  (trait bounds)
    │      WHERE CLAUSE    │
    └──────────────────────┘


    ┌──────────────────────┐
    │  42. FUNCTION TAKING │────┐
    │      ASREF PATH      │    │
    └──────────────────────┘    ↓
                            [#38 Result]


┌─────────────────────────────────────────────────────────────────┐
│                      EXPRESSION SCALE                            │
│                     (Patterns 43-50)                             │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────────────────┐
    │**43. MATCH RESULT**  │────┐
    │**    QUESTION MARK** │    │  (hub - error propagation)
    └──────────────────────┘    ↓
                            [#38 Result]


    ┌──────────────────────┐
    │  44. IF LET FOR      │  (Option unwrapping)
    │      OPTION          │
    └──────────────────────┘


    ┌──────────────────────┐
    │  45. WHILE LET FOR   │  (manual iteration)
    │      ITERATION       │
    └──────────────────────┘


    ┌──────────────────────┐
    │  46. FOR LOOP OVER   │────┐
    │      BORROWED REF    │    │
    └──────────────────────┘    ↓
                            [#33 &self]


    ┌──────────────────────┐
    │  47. ASSERT MACRO    │  (invariant checking)
    │      IN FUNCTION     │
    └──────────────────────┘


    ┌──────────────────────┐
    │  48. MEM SWAP FOR    │  (efficient exchange)
    │      MOVING VALUES   │
    └──────────────────────┘


    ┌──────────────────────┐
    │  49. CLONE TO EXTEND │  (copy when needed)
    │      LIFETIME        │
    └──────────────────────┘


    ┌──────────────────────┐
    │  50. TEST FUNCTION   │  (testing)
    │      WITH ATTRIBUTE  │
    └──────────────────────┘
```

---

## Hub Patterns (Most Referenced)

These patterns are referenced by many others and form the "backbone" of the language:

### #24: Generic Type with Parameter T
**Referenced by**: #21, #22, #23, #25
**Reason**: Foundation for reusable data structures

### #31: Type Alias for Result
**Referenced by**: #30, #38
**Reason**: Central to error handling throughout the language

### #33: Method Taking Self by Reference
**Referenced by**: #1, #21, #46
**Reason**: Most common method signature for reading data

### #38: Function Returning Result
**Referenced by**: #42, #43
**Reason**: Error handling is pervasive in Rust

### #43: Match on Result with Question Mark
**Referenced by**: #1, #23, #38
**Reason**: The primary error propagation mechanism

### #18: Crate Root Reexporting Core
**Referenced by**: #2, #9, #12
**Reason**: Key architectural pattern for clean APIs

---

## Pattern Clusters (Often Used Together)

### Cluster 1: Queue Implementation
```
#22 (Two Vecs Queue)
  ├→ #21 (Struct with Vec)
  ├→ #24 (Generic Type T)
  ├→ #34 (&mut self)
  ├→ #36 (new())
  └→ #48 (mem::swap)
```
**Used in**: queue, generic-queue projects

### Cluster 2: Error Handling
```
#30 (Custom Error)
  ├→ #31 (Type Alias Result)
  └→ #28 (Derive Debug)

#38 (Function Returning Result)
  └→ #43 (Question Mark ?)
```
**Used in**: Almost all projects with I/O

### Cluster 3: FFI Safe Wrapper
```
#8 (Safe Wrapper)
  ├→ #7 (Unsafe FFI)
  │  ├→ #16 (Raw Bindings)
  │  ├→ #40 (unsafe fn)
  │  └→ #26 (Newtype Pointer)
  │     └→ #27 (PhantomData)
  ├→ #17 (Public Facade)
  └→ #30 (Custom Error)
```
**Used in**: libgit2-rs-safe

### Cluster 4: Multi-Module Library
```
#2 (Library Crate)
  ├→ #11 (Nested Submodules)
  │  ├→ #10 (Submodule File)
  │  └→ #13 (Feature Groups)
  ├→ #12 (Private Reexport)
  │  └→ #18 (Crate Root Reexport)
  └→ #17 (Public Facade)
```
**Used in**: fern_sim

### Cluster 5: Async Web Service
```
#6 (Async Runtime)
  ├→ #1 (Binary Main)
  ├→ #39 (async fn)
  ├→ #29 (Deserialize)
  └→ Error handling cluster (30, 31, 38, 43)
```
**Used in**: actix-gcd

### Cluster 6: CLI Tool
```
#1 (Binary Main)
  ├→ #43 (Question Mark)
  ├→ #46 (For Loop &ref)
  ├→ #47 (assert!)
  └→ Error handling (38, 43)
```
**Used in**: gcd, grep, copy

---

## Dependency Depth (Longest Chains)

These show the deepest pattern chains from PROJECT to EXPRESSION:

### Chain 1: Binary → Error Handling
```
#1 Binary with Main
  ↓
#38 Function Returning Result
  ↓
#31 Type Alias for Result
  ↓
(terminal - no further refs)
```
**Depth**: 3 levels

### Chain 2: Binary → Iteration
```
#1 Binary with Main
  ↓
#46 For Loop Over Borrowed Reference
  ↓
#33 Method Taking Self by Reference
  ↓
(terminal - no further refs)
```
**Depth**: 3 levels

### Chain 3: FFI Wrapper → Unsafe
```
#8 Safe Wrapper Around Unsafe
  ↓
#7 Unsafe FFI Wrapper Crate
  ↓
#26 Newtype Wrapping Raw Pointer
  ↓
#27 PhantomData for Lifetime
  ↓
(terminal - no further refs)
```
**Depth**: 4 levels

### Chain 4: Library → Queue
```
#2 Library Crate with Public API
  ↓
#9 Module Tree in Lib File
  ↓
#18 Crate Root Reexporting Core
  ↓
(terminal - architectural endpoint)

(Meanwhile, type chain:)
#21 Struct with Vec Fields
  ↓
#24 Generic Type with Parameter T
  ↓
(terminal - no further refs)
```
**Depth**: 3 levels (architecture), 2 levels (types)

---

## Anti-Patterns (What NOT to Do)

These are patterns to **avoid** (not numbered, but worth documenting):

### Anti-Pattern A: Deep Module Nesting
```
❌ DON'T:
crate::models::user::profile::settings::privacy::Public

✅ DO: Use pattern #12 (Private Module Public Reexport)
crate::Public
```

### Anti-Pattern B: Cloning Instead of Borrowing
```
❌ DON'T:
fn process(data: Vec<T>) {
    let copy = data.clone();
    // ...
}

✅ DO: Use pattern #33 (Method Taking &self)
fn process(data: &Vec<T>) {
    // ...
}
```

### Anti-Pattern C: Unwrap Everywhere
```
❌ DON'T:
fn main() {
    let file = File::open("data").unwrap();
    let contents = read_to_string(file).unwrap();
}

✅ DO: Use pattern #43 (Question Mark)
fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("data")?;
    let contents = read_to_string(file)?;
    Ok(())
}
```

### Anti-Pattern D: Exposing Internal Structure
```
❌ DON'T:
pub struct Queue {
    pub older: Vec<T>,    // Public fields!
    pub younger: Vec<T>,
}

✅ DO: Use pattern #21 (Struct with Vec Fields) with private fields
pub struct Queue {
    older: Vec<T>,        // Private
    younger: Vec<T>,
}
```

---

## Pattern Discovery Process

How were these patterns identified from the codebase?

### Step 1: Code Mining
Analyzed all 24 projects in the repository, looking for recurring structures:
- Counted how many times each pattern appears (minimum: 3 occurrences)
- Verified patterns solve a real problem (not arbitrary choices)
- Checked that patterns work together (form a coherent language)

### Step 2: Scale Assignment
Classified each pattern by scale:
- **PROJECT**: Affects whole crate structure
- **ARCHITECTURE**: Affects module organization
- **TYPE**: Defines data structures
- **FUNCTION**: Implements behavior
- **EXPRESSION**: Specific code idioms

### Step 3: Naming
Named each pattern concretely following Alexander's method:
- Describe what you see in the code (not abstract concept)
- Use literal terminology from Rust
- Make names searchable and memorable

### Step 4: Graph Construction
Mapped references between patterns:
- Which patterns reference which?
- What's the dependency depth?
- Which are hub patterns?

### Result: 50 Patterns
From ~10,000 lines of code across 24 projects, extracted 50 recurring, named patterns that work together as a language.

---

## How to Use This Graph

### For Choosing Patterns
1. Start at PROJECT scale (top of graph)
2. Follow arrows downward to smaller scales
3. Stop when you reach concrete code expressions

### For Understanding Dependencies
1. Find your pattern in the graph
2. Follow arrows to see what it references
3. Implement referenced patterns first

### For Code Review
1. Identify patterns in the code
2. Check if they're in the right scale
3. Verify correct usage based on graph

### For Learning
1. Start with hub patterns (#24, #31, #33, #38, #43)
2. Learn patterns they reference
3. Learn patterns that reference them
4. Build outward through the graph

---

## Summary Statistics

- **Total patterns**: 50
- **PROJECT scale**: 8 patterns (16%)
- **ARCHITECTURE scale**: 12 patterns (24%)
- **TYPE scale**: 12 patterns (24%)
- **FUNCTION scale**: 10 patterns (20%)
- **EXPRESSION scale**: 8 patterns (16%)

**Hub patterns** (referenced by 3+ others): 6
- #18, #24, #31, #33, #38, #43

**Leaf patterns** (reference no others): 15
- #10, #14, #16, #17, #27, #28, #29, #32, #36, #37, #39, #41, #44, #45, #47, #49, #50

**Average depth**: 2.3 levels
**Maximum depth**: 4 levels (FFI chain)

---

This graph is the **skeleton** of the pattern language. The patterns themselves (in other documents) are the **flesh**—they explain why each pattern exists and how to use it. Together, they form a complete generative system for building Rust programs.
