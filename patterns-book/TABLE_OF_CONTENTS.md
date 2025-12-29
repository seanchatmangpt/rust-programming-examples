# A Pattern Language for Rust Programming
## Complete Table of Contents

**Following Christopher Alexander's *A Pattern Language* (1977)**

---

## Front Matter

### Preface: The Timeless Way of Programming
- Why pattern languages for code
- How this differs from typical programming books
- How to use this book
- Acknowledgments to Christopher Alexander

### Introduction: What is a Pattern Language?
- Alexander's methodology explained
- The five scales of Rust programming
- How patterns reference each other
- The generative sequence
- The quality without a name

### How to Use This Book
- Starting a new project (work top-down)
- Solving a specific problem (work bottom-up)
- Code review checklist
- Learning path for different skill levels

### Map of the Language
- Complete pattern graph
- Hub patterns and key connections
- Pattern clusters and combinations
- Quick reference by pattern number

---

## PART ONE: PROJECT SCALE
### "In the beginning, there is the decision: what kind of thing are we building?"

**Patterns 1-8: The Whole Program**

#### 1. BINARY WITH MAIN FUNCTION
*A standalone executable program with a main() entry point*
- The fundamental executable pattern
- Entry point for command-line tools
- Examples: gcd, grep, copy, http-get
- References: #33, #43, #46

#### 2. LIBRARY CRATE WITH PUBLIC API
*A reusable library exposing types and functions for other crates*
- Pure library design
- Public API considerations
- Examples: queue, complex, binary-tree
- References: #9, #18, #28

#### 3. BINARY AND LIBRARY TOGETHER
*A crate with both src/main.rs and src/lib.rs, sharing code*
- Combining executable and library
- Testable core logic
- Examples: Most projects in repository
- References: #1, #2, #14

#### 4. TESTS DIRECTORY BESIDE SOURCE
*Integration tests in tests/ treating the crate as external*
- Testing as external consumer
- Integration vs unit tests
- Examples: fern_sim/tests/
- References: #50, #47

#### 5. EXAMPLES DIRECTORY FOR USAGE
*Example programs in examples/ showing how to use the library*
- Demonstrating library usage
- Executable documentation
- Cargo examples feature

#### 6. ASYNC RUNTIME WITH ATTRIBUTE MAIN
*A program using async/await with #[tokio::main] or #[actix_web::main]*
- Async executables
- Runtime selection
- Examples: actix-gcd, cheapo-request
- References: #39, #1

#### 7. UNSAFE FFI WRAPPER CRATE
*A library providing Rust bindings to a C library*
- Low-level FFI
- Raw C bindings
- Examples: libgit2-rs
- References: #16, #40, #26

#### 8. SAFE WRAPPER AROUND UNSAFE
*A safe Rust API built on top of unsafe FFI bindings*
- Safety abstractions
- Wrapping unsafe code
- Examples: libgit2-rs-safe
- References: #7, #17, #30

---

## PART TWO: ARCHITECTURE SCALE
### "Once the project exists, we must organize its internal structure..."

**Patterns 9-20: Module Organization**

#### 9. MODULE TREE IN LIB FILE
*Module declarations in src/lib.rs with mod statements*
- Central module organization
- Module visibility control
- References: #10, #18

#### 10. SUBMODULE IN SEPARATE FILE
*A module defined in its own .rs file (mod foo; → foo.rs)*
- Code organization across files
- Module file structure
- Referenced by: #9, #11

#### 11. NESTED SUBMODULES IN DIRECTORY
*A module with children (foo/mod.rs, foo/bar.rs, foo/baz.rs)*
- Deep hierarchies
- Complex organization
- Examples: fern_sim/plant_structures/
- References: #10, #13

#### 12. PRIVATE MODULE PUBLIC REEXPORT
*Internal module organization hidden behind pub use*
- API vs implementation separation
- Flexibility to refactor
- References: #18, #17

#### 13. FEATURE-BASED MODULE GROUPS
*Modules organized by feature (plant_structures/, simulation.rs)*
- Domain-driven organization
- Feature boundaries
- Examples: fern_sim structure

#### 14. FLAT MODULE ALL IN LIB
*Simple library with all code in src/lib.rs, no submodules*
- Simplest organization
- Single-file libraries
- Examples: queue (early version)

#### 15. TEST MODULE WITH USE SUPER STAR
*#[cfg(test)] mod tests with use super::* for unit tests*
- Unit test organization
- Test visibility
- Examples: All projects with tests
- References: #50, #47

#### 16. RAW BINDINGS MODULE
*A private raw module containing unsafe FFI declarations*
- FFI declarations isolation
- Unsafe code boundaries
- Examples: libgit2-rs/src/raw.rs

#### 17. PUBLIC FACADE MODULE
*A clean public API module hiding internal complexity*
- API simplification
- Internal complexity hiding
- Referenced by: #8, #12

#### 18. CRATE ROOT REEXPORTING CORE
*pub use bringing key types to crate:: namespace*
- Convenient imports
- API ergonomics
- Examples: fern_sim lib.rs
- Referenced by: #2, #9, #12

#### 19. BUILD SCRIPT FOR C DEPENDENCIES
*build.rs compiling and linking external C libraries*
- FFI compilation
- System library linking
- Examples: libgit2-rs/build.rs

#### 20. CONDITIONAL COMPILATION BY OS
*#[cfg(unix)] and #[cfg(windows)] for platform differences*
- Cross-platform code
- Platform-specific implementations
- Examples: libgit2-rs-safe path handling
- References: #42

---

## PART THREE: TYPE SCALE
### "With modules in place, we define the types that give the program its shape..."

**Patterns 21-32: Data Structures**

#### 21. STRUCT WITH VEC FIELDS
*A struct containing Vec<T> for dynamic collections*
- Collection-based types
- Owned dynamic data
- Examples: Queue struct
- References: #24, #33

#### 22. STRUCT WITH TWO VECS FOR QUEUE
*Two vectors working together (older: Vec<T>, younger: Vec<T>)*
- Amortized algorithms
- Dual-buffer patterns
- Examples: queue implementation
- References: #21, #34, #48

#### 23. ENUM WITH EMPTY AND NONEMPTY
*Enum variants for Empty and NonEmpty(Box<Node>) cases*
- Recursive data structures
- Optional complex data
- Examples: BinaryTree enum
- References: #24, #43

#### 24. GENERIC TYPE WITH PARAMETER T
*A struct or enum with <T> for any type*
- Parametric polymorphism
- Reusable containers
- Examples: Queue<T>, BinaryTree<T>
- Referenced by: #21, #22, #23, #25

#### 25. TRAIT BOUND ON IMPL BLOCK
*impl<T: Clone> only when T satisfies constraints*
- Conditional implementations
- Capability-based methods
- Examples: BinaryTree walk() for Clone types
- References: #24, #41

#### 26. NEWTYPE WRAPPING RAW POINTER
*struct Repository { raw: *mut git_repository }*
- FFI resource ownership
- Safe wrappers for unsafe pointers
- Examples: Repository, Commit in libgit2-rs-safe
- References: #27, #40

#### 27. PHANTOMDATA FOR LIFETIME
*PhantomData<&'repo> to tie struct to lifetime without storing reference*
- Lifetime tracking without storage
- Lifetime relationships
- Examples: Commit<'repo> in libgit2-rs-safe
- Referenced by: #26

#### 28. DERIVE DEBUG FOR TESTING
*#[derive(Debug)] for automatic fmt::Debug implementation*
- Debugging support
- Test output
- Examples: Used in most types
- Referenced by: #2, #30

#### 29. DERIVE DESERIALIZE FOR FORMS
*#[derive(Deserialize)] for serde parsing of web forms*
- Automatic parsing
- Web form handling
- Examples: GcdParameters in actix-gcd
- Referenced by: #6 (async web)

#### 30. CUSTOM ERROR STRUCT WITH DISPLAY
*Error type implementing fmt::Display and std::error::Error*
- Domain-specific errors
- Error context
- Examples: Error type in libgit2-rs-safe
- References: #31, #28

#### 31. TYPE ALIAS FOR RESULT
*pub type Result<T> = std::result::Result<T, Error>*
- Convenient error handling
- Consistent error types
- Examples: Used with #30
- Referenced by: #30, #38

#### 32. UNIT STRUCT FOR MARKER
*struct MyMarker; with no fields for zero-size types*
- Type-level markers
- Zero-cost abstractions
- Examples: Marker types in advanced patterns

---

## PART FOUR: FUNCTION SCALE
### "Types are static; functions bring them to life..."

**Patterns 33-42: Behavior Implementation**

#### 33. METHOD TAKING SELF BY REFERENCE
*pub fn method(&self) for reading data*
- Immutable access
- No ownership transfer
- Examples: is_empty(), walk()
- Referenced by: #1, #21, #46

#### 34. METHOD TAKING SELF BY MUT REFERENCE
*pub fn method(&mut self) for modifying data*
- Mutable access
- Modification in place
- Examples: push(), pop(), add()
- References: #21, #48

#### 35. METHOD CONSUMING SELF
*pub fn method(self) for moving ownership*
- Ownership transfer
- Destructuring
- Examples: split() in Queue
- References: #37

#### 36. CONSTRUCTOR FUNCTION NAMED NEW
*pub fn new() -> Self for creating instances*
- Instance creation
- Rust idiom for constructors
- Examples: Queue::new()
- Referenced by: #22 (queue pattern)

#### 37. BUILDER METHOD RETURNING SELF
*fn with_param(mut self, val: T) -> Self for chaining*
- Fluent interfaces
- Method chaining
- Examples: Request builders
- Referenced by: #35

#### 38. FUNCTION RETURNING RESULT
*fn operation() -> Result<T, E> for fallible operations*
- Error propagation
- Fallible operations
- Examples: Repository::open()
- References: #31, #43

#### 39. ASYNC FUNCTION WITH AWAIT
*async fn handler() awaiting other async operations*
- Asynchronous operations
- Non-blocking I/O
- Examples: actix-gcd handlers
- Referenced by: #6

#### 40. UNSAFE FUNCTION WITH SAFETY COMMENT
*unsafe fn operation() with // SAFETY: explanation*
- Unsafe boundaries
- Safety documentation
- Examples: FFI wrappers
- References: #47

#### 41. GENERIC FUNCTION WITH WHERE CLAUSE
*fn function<T>(param: T) where T: Trait + Other*
- Complex trait bounds
- Multiple constraints
- Examples: Generic algorithms
- Referenced by: #25

#### 42. FUNCTION TAKING ASREF PATH
*fn open<P: AsRef<Path>>(path: P) for flexible path parameters*
- Flexible arguments
- Path handling
- Examples: Repository::open()
- References: #38

---

## PART FIVE: EXPRESSION SCALE
### "Finally, we write the actual code, the smallest patterns that complete the work..."

**Patterns 43-50: Code Idioms**

#### 43. MATCH ON RESULT WITH QUESTION MARK
*Using ? operator to propagate errors early*
- Error propagation
- Early returns
- Examples: Used throughout I/O code
- References: #38

#### 44. IF LET FOR OPTION UNWRAPPING
*if let Some(val) = option to handle one case*
- Partial pattern matching
- Option handling
- Examples: Conditional processing

#### 45. WHILE LET FOR ITERATION
*while let Some(item) = iterator.next() for manual iteration*
- Manual iteration control
- Iterator patterns
- Examples: Custom iteration logic

#### 46. FOR LOOP OVER BORROWED REFERENCE
*for item in &collection iterating without consuming*
- Non-consuming iteration
- Immutable traversal
- Examples: Processing arguments
- References: #33

#### 47. ASSERT MACRO IN FUNCTION BODY
*assert!(condition) to check invariants*
- Precondition checking
- Invariant enforcement
- Examples: gcd function
- Referenced by: #4, #15, #40

#### 48. MEM SWAP FOR MOVING VALUES
*std::mem::swap(&mut a, &mut b) to exchange ownership*
- Efficient exchanges
- No-copy moves
- Examples: Queue pop() implementation
- Referenced by: #22, #34

#### 49. CLONE TO EXTEND LIFETIME
*.clone() when you need owned data instead of borrowed*
- Ownership extension
- Explicit copying
- Examples: Collecting iterator results

#### 50. TEST FUNCTION WITH ATTRIBUTE
*#[test] fn test_case() for unit testing*
- Unit testing
- Test organization
- Examples: All test modules
- Referenced by: #4, #15

---

## Back Matter

### Appendix A: Pattern Cross-Reference
- Complete pattern dependency graph
- Hub patterns analysis
- Pattern clusters and combinations
- Reference table

### Appendix B: Pattern Selection Guide
- Decision trees by project type
- "If you're building X, use patterns Y..."
- Common combinations
- Quick reference

### Appendix C: Anti-Patterns
- What not to do
- Common mistakes
- Better alternatives using patterns

### Appendix D: Code Examples Index
- Which repository projects demonstrate which patterns
- Line number references
- Complete working examples

### Appendix E: Extending the Language
- How to identify new patterns
- Pattern naming guidelines
- Contributing to the language
- Pattern validation criteria

### Glossary
- Rust terminology
- Alexander's terminology
- Pattern language concepts

### Bibliography
- Christopher Alexander's works
- Rust language references
- Related pattern languages
- Recommended reading

### Index
- Pattern names alphabetically
- Rust concepts
- Code examples
- Projects

---

## Book Statistics

- **Total pages**: ~400 (estimated)
- **Total patterns**: 50
- **Code examples**: 100+ (from 24 projects)
- **Diagrams**: ~75 (one per pattern plus overview diagrams)

## Pattern Distribution by Scale

| Scale | Patterns | Pages | Percentage |
|-------|----------|-------|------------|
| Project | 8 | ~60 | 16% |
| Architecture | 12 | ~90 | 24% |
| Type | 12 | ~90 | 24% |
| Function | 10 | ~75 | 20% |
| Expression | 8 | ~60 | 16% |

## Reading Time Estimates

- **Cover to cover**: 20-25 hours
- **Skim all patterns**: 4-6 hours
- **Quick reference**: 30 minutes to find specific patterns
- **Deep study of one pattern**: 20-30 minutes

## How to Read This Book

### Linear Reading (Traditional)
1. Start at PROJECT SCALE (Part One)
2. Read through each scale in order
3. End at EXPRESSION SCALE (Part Five)
4. Appendices as reference

**Best for**: Learning systematically, complete understanding

### Pattern-Driven Reading (Alexander's Way)
1. Start with a problem ("I need to build X")
2. Find relevant pattern in selection guide
3. Read that pattern
4. Follow references to related patterns
5. Jump between scales as needed

**Best for**: Solving specific problems, practical application

### Hub-First Reading (Efficient)
1. Read hub patterns first (#24, #31, #33, #38, #43, #18)
2. Then read patterns they reference
3. Then read patterns that reference them
4. Fill in remaining patterns

**Best for**: Understanding core concepts quickly

### Example-Driven Reading (Hands-On)
1. Pick a project from repository (e.g., queue)
2. Read the code
3. Identify patterns (use index)
4. Read those pattern descriptions
5. Try another project

**Best for**: Learning by doing, code-first learners

---

## Format Conventions

Throughout the book:

- **Pattern numbers** are in bold: **#43**
- *Pattern names* in italic when referenced: *Match on Result with Question Mark*
- `Code examples` in monospace: `Result<T, E>`
- **Important terms** in bold on first use
- [Diagram placeholders] in brackets
- ◆ ◆ ◆ marks pattern sections

Each pattern follows the same structure:
```
## NUMBER. PATTERN NAME

[Illustration]

...context paragraph...

◆ ◆ ◆

**Problem statement**

Explanation body

**Therefore: Solution**

Code example

[Diagram]

◆ ◆ ◆

References to completing patterns
```

---

## Companion Materials

### Online Resources
- Interactive pattern graph
- Searchable pattern database
- Code example playground
- Community pattern submissions

### Repository
- `/home/user/rust-programming-examples`
- All 24 source projects
- Tests and examples
- Pattern mapping

### Study Guides
- Pattern flashcards
- Quiz by scale
- Code reading exercises
- Project building challenges

---

This table of contents represents the complete structure of **A Pattern Language for Rust Programming**, a 400-page book applying Christopher Alexander's methodology to software development in Rust. Each pattern is concrete, named, numbered, and connected to others in a directed graph that forms a generative language for building programs.
