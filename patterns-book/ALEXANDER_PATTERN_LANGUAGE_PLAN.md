# A Pattern Language for Rust Programming
## Following Christopher Alexander's Methodology

---

## Introduction: The Timeless Way of Programming

This book presents a **pattern language** for Rust programming, following Christopher Alexander's methodology from *A Pattern Language* (1977). Just as Alexander created a language for architecture that connects regions to towns to buildings to rooms to construction details, this language connects project structure to module organization to type definitions to function implementations to expression idioms.

Each pattern in this language has a **number** and a **concrete name**. Patterns are not abstract principles but specific, recognizable solutions that appear repeatedly in well-crafted Rust code. They form a **directed graph**: larger patterns provide context for smaller ones, smaller patterns complete the details of larger ones.

You can use this language to build programs, just as Alexander's language builds buildings. Start with patterns at the Project scale (#1-8), work down through Architecture (#9-20), Types (#21-32), Functions (#33-42), and finish with Expressions (#43-50).

---

## The Five Scales of Rust Programming

### PROJECT SCALE (The Whole Program)
Like Alexander's "regions" - the largest organizing principle
- What kind of crate is this?
- How does it interact with the outside world?
- What is its fundamental purpose?

### ARCHITECTURE SCALE (Module Organization)
Like Alexander's "towns" - major divisions and connections
- How is code organized into modules?
- What is public, what is private?
- How do modules relate to each other?

### TYPE SCALE (Data Structures)
Like Alexander's "buildings" - substantial, named abstractions
- What are the core data types?
- How do types compose?
- What traits define behavior?

### FUNCTION SCALE (Behavior Implementation)
Like Alexander's "rooms" - enclosed, purposeful spaces
- How is behavior organized?
- What do functions take and return?
- How are errors handled?

### EXPRESSION SCALE (Code Idioms)
Like Alexander's "construction" - the actual materials
- What code constructs are used?
- How are common operations expressed?
- What are the smallest reusable patterns?

---

## The Complete Pattern Language: 50 Patterns

### PROJECT SCALE (Patterns 1-8)

**1. BINARY WITH MAIN FUNCTION**
A standalone executable program with a main() entry point

**2. LIBRARY CRATE WITH PUBLIC API**
A reusable library exposing types and functions for other crates

**3. BINARY AND LIBRARY TOGETHER**
A crate with both src/main.rs and src/lib.rs, sharing code

**4. TESTS DIRECTORY BESIDE SOURCE**
Integration tests in tests/ treating the crate as external

**5. EXAMPLES DIRECTORY FOR USAGE**
Example programs in examples/ showing how to use the library

**6. ASYNC RUNTIME WITH ATTRIBUTE MAIN**
A program using async/await with #[tokio::main] or #[actix_web::main]

**7. UNSAFE FFI WRAPPER CRATE**
A library providing Rust bindings to a C library

**8. SAFE WRAPPER AROUND UNSAFE**
A safe Rust API built on top of unsafe FFI bindings

### ARCHITECTURE SCALE (Patterns 9-20)

**9. MODULE TREE IN LIB FILE**
Module declarations in src/lib.rs with mod statements

**10. SUBMODULE IN SEPARATE FILE**
A module defined in its own .rs file (mod foo; → foo.rs)

**11. NESTED SUBMODULES IN DIRECTORY**
A module with children (foo/mod.rs, foo/bar.rs, foo/baz.rs)

**12. PRIVATE MODULE PUBLIC REEXPORT**
Internal module organization hidden behind pub use

**13. FEATURE-BASED MODULE GROUPS**
Modules organized by feature (plant_structures/, simulation.rs)

**14. FLAT MODULE ALL IN LIB**
Simple library with all code in src/lib.rs, no submodules

**15. TEST MODULE WITH USE SUPER STAR**
#[cfg(test)] mod tests with use super::* for unit tests

**16. RAW BINDINGS MODULE**
A private raw module containing unsafe FFI declarations

**17. PUBLIC FACADE MODULE**
A clean public API module hiding internal complexity

**18. CRATE ROOT REEXPORTING CORE**
pub use bringing key types to crate:: namespace

**19. BUILD SCRIPT FOR C DEPENDENCIES**
build.rs compiling and linking external C libraries

**20. CONDITIONAL COMPILATION BY OS**
#[cfg(unix)] and #[cfg(windows)] for platform differences

### TYPE SCALE (Patterns 21-32)

**21. STRUCT WITH VEC FIELDS**
A struct containing Vec<T> for dynamic collections

**22. STRUCT WITH TWO VECS FOR QUEUE**
Two vectors working together (older: Vec<T>, younger: Vec<T>)

**23. ENUM WITH EMPTY AND NONEMPTY**
Enum variants for Empty and NonEmpty(Box<Node>) cases

**24. GENERIC TYPE WITH PARAMETER T**
A struct or enum with <T> for any type

**25. TRAIT BOUND ON IMPL BLOCK**
impl<T: Clone> only when T satisfies constraints

**26. NEWTYPE WRAPPING RAW POINTER**
struct Repository { raw: *mut git_repository }

**27. PHANTOMDATA FOR LIFETIME**
PhantomData<&'repo> to tie struct to lifetime without storing reference

**28. DERIVE DEBUG FOR TESTING**
#[derive(Debug)] for automatic fmt::Debug implementation

**29. DERIVE DESERIALIZE FOR FORMS**
#[derive(Deserialize)] for serde parsing of web forms

**30. CUSTOM ERROR STRUCT WITH DISPLAY**
Error type implementing fmt::Display and std::error::Error

**31. TYPE ALIAS FOR RESULT**
pub type Result<T> = std::result::Result<T, Error>

**32. UNIT STRUCT FOR MARKER**
struct MyMarker; with no fields for zero-size types

### FUNCTION SCALE (Patterns 33-42)

**33. METHOD TAKING SELF BY REFERENCE**
pub fn method(&self) for reading data

**34. METHOD TAKING SELF BY MUT REFERENCE**
pub fn method(&mut self) for modifying data

**35. METHOD CONSUMING SELF**
pub fn method(self) for moving ownership

**36. CONSTRUCTOR FUNCTION NAMED NEW**
pub fn new() -> Self for creating instances

**37. BUILDER METHOD RETURNING SELF**
fn with_param(mut self, val: T) -> Self for chaining

**38. FUNCTION RETURNING RESULT**
fn operation() -> Result<T, E> for fallible operations

**39. ASYNC FUNCTION WITH AWAIT**
async fn handler() awaiting other async operations

**40. UNSAFE FUNCTION WITH SAFETY COMMENT**
unsafe fn operation() with // SAFETY: explanation

**41. GENERIC FUNCTION WITH WHERE CLAUSE**
fn function<T>(param: T) where T: Trait + Other

**42. FUNCTION TAKING ASREF PATH**
fn open<P: AsRef<Path>>(path: P) for flexible path parameters

### EXPRESSION SCALE (Patterns 43-50)

**43. MATCH ON RESULT WITH QUESTION MARK**
Using ? operator to propagate errors early

**44. IF LET FOR OPTION UNWRAPPING**
if let Some(val) = option to handle one case

**45. WHILE LET FOR ITERATION**
while let Some(item) = iterator.next() for manual iteration

**46. FOR LOOP OVER BORROWED REFERENCE**
for item in &collection iterating without consuming

**47. ASSERT MACRO IN FUNCTION BODY**
assert!(condition) to check invariants

**48. MEM SWAP FOR MOVING VALUES**
std::mem::swap(&mut a, &mut b) to exchange ownership

**49. CLONE TO EXTEND LIFETIME**
.clone() when you need owned data instead of borrowed

**50. TEST FUNCTION WITH ATTRIBUTE**
#[test] fn test_case() for unit testing

---

## The Pattern Graph: How Patterns Connect

This directed graph shows which patterns reference which. An arrow from A → B means "pattern A references pattern B in its solution."

```
PROJECT SCALE
├─ 1. BINARY WITH MAIN FUNCTION
│  └─→ 33. METHOD TAKING SELF BY REFERENCE
│  └─→ 43. MATCH ON RESULT WITH QUESTION MARK
│  └─→ 46. FOR LOOP OVER BORROWED REFERENCE
│
├─ 2. LIBRARY CRATE WITH PUBLIC API
│  └─→ 9. MODULE TREE IN LIB FILE
│  └─→ 18. CRATE ROOT REEXPORTING CORE
│  └─→ 28. DERIVE DEBUG FOR TESTING
│
├─ 3. BINARY AND LIBRARY TOGETHER
│  └─→ 1. BINARY WITH MAIN FUNCTION
│  └─→ 2. LIBRARY CRATE WITH PUBLIC API
│  └─→ 14. FLAT MODULE ALL IN LIB
│
├─ 4. TESTS DIRECTORY BESIDE SOURCE
│  └─→ 50. TEST FUNCTION WITH ATTRIBUTE
│  └─→ 47. ASSERT MACRO IN FUNCTION BODY
│
├─ 6. ASYNC RUNTIME WITH ATTRIBUTE MAIN
│  └─→ 39. ASYNC FUNCTION WITH AWAIT
│  └─→ 1. BINARY WITH MAIN FUNCTION
│
├─ 7. UNSAFE FFI WRAPPER CRATE
│  └─→ 16. RAW BINDINGS MODULE
│  └─→ 40. UNSAFE FUNCTION WITH SAFETY COMMENT
│  └─→ 26. NEWTYPE WRAPPING RAW POINTER
│
└─ 8. SAFE WRAPPER AROUND UNSAFE
   └─→ 7. UNSAFE FFI WRAPPER CRATE
   └─→ 17. PUBLIC FACADE MODULE
   └─→ 30. CUSTOM ERROR STRUCT WITH DISPLAY

ARCHITECTURE SCALE
├─ 9. MODULE TREE IN LIB FILE
│  └─→ 10. SUBMODULE IN SEPARATE FILE
│  └─→ 18. CRATE ROOT REEXPORTING CORE
│
├─ 11. NESTED SUBMODULES IN DIRECTORY
│  └─→ 10. SUBMODULE IN SEPARATE FILE
│  └─→ 13. FEATURE-BASED MODULE GROUPS
│
├─ 12. PRIVATE MODULE PUBLIC REEXPORT
│  └─→ 18. CRATE ROOT REEXPORTING CORE
│  └─→ 17. PUBLIC FACADE MODULE
│
├─ 15. TEST MODULE WITH USE SUPER STAR
│  └─→ 50. TEST FUNCTION WITH ATTRIBUTE
│  └─→ 47. ASSERT MACRO IN FUNCTION BODY
│
└─ 20. CONDITIONAL COMPILATION BY OS
   └─→ 42. FUNCTION TAKING ASREF PATH

TYPE SCALE
├─ 21. STRUCT WITH VEC FIELDS
│  └─→ 24. GENERIC TYPE WITH PARAMETER T
│  └─→ 33. METHOD TAKING SELF BY REFERENCE
│
├─ 22. STRUCT WITH TWO VECS FOR QUEUE
│  └─→ 21. STRUCT WITH VEC FIELDS
│  └─→ 34. METHOD TAKING SELF BY MUT REFERENCE
│  └─→ 48. MEM SWAP FOR MOVING VALUES
│
├─ 23. ENUM WITH EMPTY AND NONEMPTY
│  └─→ 24. GENERIC TYPE WITH PARAMETER T
│  └─→ 43. MATCH ON RESULT WITH QUESTION MARK
│
├─ 25. TRAIT BOUND ON IMPL BLOCK
│  └─→ 24. GENERIC TYPE WITH PARAMETER T
│  └─→ 41. GENERIC FUNCTION WITH WHERE CLAUSE
│
├─ 26. NEWTYPE WRAPPING RAW POINTER
│  └─→ 27. PHANTOMDATA FOR LIFETIME
│  └─→ 40. UNSAFE FUNCTION WITH SAFETY COMMENT
│
├─ 30. CUSTOM ERROR STRUCT WITH DISPLAY
│  └─→ 31. TYPE ALIAS FOR RESULT
│  └─→ 28. DERIVE DEBUG FOR TESTING
│
└─ 31. TYPE ALIAS FOR RESULT
   └─→ 38. FUNCTION RETURNING RESULT

FUNCTION SCALE
├─ 34. METHOD TAKING SELF BY MUT REFERENCE
│  └─→ 21. STRUCT WITH VEC FIELDS
│  └─→ 48. MEM SWAP FOR MOVING VALUES
│
├─ 35. METHOD CONSUMING SELF
│  └─→ 37. BUILDER METHOD RETURNING SELF
│
├─ 38. FUNCTION RETURNING RESULT
│  └─→ 31. TYPE ALIAS FOR RESULT
│  └─→ 43. MATCH ON RESULT WITH QUESTION MARK
│
├─ 40. UNSAFE FUNCTION WITH SAFETY COMMENT
│  └─→ 47. ASSERT MACRO IN FUNCTION BODY
│
└─ 42. FUNCTION TAKING ASREF PATH
   └─→ 38. FUNCTION RETURNING RESULT

EXPRESSION SCALE
├─ 43. MATCH ON RESULT WITH QUESTION MARK
│  └─→ 38. FUNCTION RETURNING RESULT
│
├─ 46. FOR LOOP OVER BORROWED REFERENCE
│  └─→ 33. METHOD TAKING SELF BY REFERENCE
│
└─ 48. MEM SWAP FOR MOVING VALUES
   └─→ 34. METHOD TAKING SELF BY MUT REFERENCE
```

---

## Book Structure by Scale

### Part One: PROJECT SCALE (Chapters 1-8)
**Opening**: "In the beginning, there is the decision: what kind of thing are we building?"

- Chapter 1: Binary with Main Function (#1)
- Chapter 2: Library Crate with Public API (#2)
- Chapter 3: Binary and Library Together (#3)
- Chapter 4: Tests Directory Beside Source (#4)
- Chapter 5: Examples Directory for Usage (#5)
- Chapter 6: Async Runtime with Attribute Main (#6)
- Chapter 7: Unsafe FFI Wrapper Crate (#7)
- Chapter 8: Safe Wrapper Around Unsafe (#8)

### Part Two: ARCHITECTURE SCALE (Chapters 9-20)
**Opening**: "Once the project exists, we must organize its internal structure..."

- Chapter 9: Module Tree in Lib File (#9)
- Chapter 10: Submodule in Separate File (#10)
- Chapter 11: Nested Submodules in Directory (#11)
- Chapter 12: Private Module Public Reexport (#12)
- Chapter 13: Feature-Based Module Groups (#13)
- Chapter 14: Flat Module All in Lib (#14)
- Chapter 15: Test Module with Use Super Star (#15)
- Chapter 16: Raw Bindings Module (#16)
- Chapter 17: Public Facade Module (#17)
- Chapter 18: Crate Root Reexporting Core (#18)
- Chapter 19: Build Script for C Dependencies (#19)
- Chapter 20: Conditional Compilation by OS (#20)

### Part Three: TYPE SCALE (Chapters 21-32)
**Opening**: "With modules in place, we define the types that give the program its shape..."

- Chapter 21: Struct with Vec Fields (#21)
- Chapter 22: Struct with Two Vecs for Queue (#22)
- Chapter 23: Enum with Empty and NonEmpty (#23)
- Chapter 24: Generic Type with Parameter T (#24)
- Chapter 25: Trait Bound on Impl Block (#25)
- Chapter 26: Newtype Wrapping Raw Pointer (#26)
- Chapter 27: PhantomData for Lifetime (#27)
- Chapter 28: Derive Debug for Testing (#28)
- Chapter 29: Derive Deserialize for Forms (#29)
- Chapter 30: Custom Error Struct with Display (#30)
- Chapter 31: Type Alias for Result (#31)
- Chapter 32: Unit Struct for Marker (#32)

### Part Four: FUNCTION SCALE (Chapters 33-42)
**Opening**: "Types are static; functions bring them to life..."

- Chapter 33: Method Taking Self by Reference (#33)
- Chapter 34: Method Taking Self by Mut Reference (#34)
- Chapter 35: Method Consuming Self (#35)
- Chapter 36: Constructor Function Named New (#36)
- Chapter 37: Builder Method Returning Self (#37)
- Chapter 38: Function Returning Result (#38)
- Chapter 39: Async Function with Await (#39)
- Chapter 40: Unsafe Function with Safety Comment (#40)
- Chapter 41: Generic Function with Where Clause (#41)
- Chapter 42: Function Taking AsRef Path (#42)

### Part Five: EXPRESSION SCALE (Chapters 43-50)
**Opening**: "Finally, we write the actual code, the smallest patterns that complete the work..."

- Chapter 43: Match on Result with Question Mark (#43)
- Chapter 44: If Let for Option Unwrapping (#44)
- Chapter 45: While Let for Iteration (#45)
- Chapter 46: For Loop over Borrowed Reference (#46)
- Chapter 47: Assert Macro in Function Body (#47)
- Chapter 48: Mem Swap for Moving Values (#48)
- Chapter 49: Clone to Extend Lifetime (#49)
- Chapter 50: Test Function with Attribute (#50)

---

## Sample Pattern in Alexander's Format

Here is one complete pattern written exactly as Alexander would write it, following the format from *A Pattern Language*:

---

## 22. STRUCT WITH TWO VECS FOR QUEUE

*[Illustration: A diagram showing a Queue struct with two Vec fields labeled "older" and "younger", with arrows showing items flowing from younger to older when reversed]*

...within a **LIBRARY CRATE WITH PUBLIC API (2)** or **BINARY AND LIBRARY TOGETHER (3)**, when you need a **STRUCT WITH VEC FIELDS (21)** to implement a first-in-first-out data structure efficiently...

◆ ◆ ◆

**A queue needs to support fast push at one end and fast pop at the other end. A single Vec can only do fast operations at one end.**

When you build a queue with a single vector, you face a dilemma: either push is fast and pop is slow (requiring shifting all elements), or pop is fast and push is slow. Neither solution is acceptable for a real queue.

You could use a linked list, but Rust's Vec is more efficient for most operations. The challenge is that Vec only has efficient push/pop at one end (the back). To pop from the front requires `remove(0)`, which shifts all remaining elements—an O(n) operation.

Consider the actual usage pattern of a queue: elements arrive at the back, wait in line, and leave from the front. Most of the time, elements are either very young (just arrived) or very old (about to leave). They spend very little time in the middle state.

This suggests a solution: keep two vectors, one for "young" elements (newly pushed) and one for "old" elements (waiting to be popped). Both vectors can use fast operations—push on younger, pop on older. Only when older runs empty do we need to move elements from younger to older, reversing them to maintain order.

The cost of the reversal is amortized: each element gets reversed exactly once in its lifetime, so the average cost per operation remains constant.

**Therefore:**

**Make a struct with two Vec<T> fields: `older` for elements that will be popped soon, and `younger` for recently pushed elements. Push onto younger. Pop from older. When older is empty, swap it with younger and reverse.**

```rust
pub struct Queue<T> {
    older: Vec<T>,   // older elements, eldest last
    younger: Vec<T>  // younger elements, youngest last
}

impl<T> Queue<T> {
    pub fn push(&mut self, t: T) {
        self.younger.push(t);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }
            // Move younger to older, reversing order
            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }
        self.older.pop()
    }
}
```

*[Diagram: A sequence showing the queue state over time:
- Initial: older=[5,4,3], younger=[]
- After push(10): older=[5,4,3], younger=[10]
- After push(11): older=[5,4,3], younger=[10,11]
- After pop(): returns 3, older=[5,4], younger=[10,11]
- After pop(): returns 4, older=[5], younger=[10,11]
- After pop(): returns 5, older=[], younger=[10,11]
- After pop(): returns 10, older=[11], younger=[] (swap happened!)
]*

The invariants are clear: elements in `older` are stored in reverse order (so the next element to pop is at the end). Elements in `younger` are stored in push order (so they can be efficiently moved to older by reversing).

◆ ◆ ◆

Give the queue a **CONSTRUCTOR FUNCTION NAMED NEW (36)** that creates empty vectors. Add an `is_empty()` **METHOD TAKING SELF BY REFERENCE (33)** to check if both vectors are empty. When the queue is destroyed, use **METHOD CONSUMING SELF (35)** to return both vectors with `split()`. Inside `pop()`, use **MEM SWAP FOR MOVING VALUES (48)** to efficiently exchange the vectors. Make it **GENERIC TYPE WITH PARAMETER T (24)** so it works for any type.

---

## How to Use This Language

Like Alexander's language, this is not a cookbook. It is a **generative system**. You compose patterns to create programs, selecting the ones that apply to your situation.

### Starting a New Project

1. Choose a PROJECT SCALE pattern (#1-8). Will this be a binary? A library? Both?
2. Add ARCHITECTURE patterns (#9-20) as needed. Simple projects use #14 (flat). Complex projects use #9-13.
3. Define core types with TYPE SCALE patterns (#21-32). What data structures do you need?
4. Implement behavior with FUNCTION SCALE patterns (#33-42). How do operations work?
5. Fill in details with EXPRESSION SCALE patterns (#43-50). What idioms make the code clear?

### Example: Building a CLI Tool

```
Start with: #1 BINARY WITH MAIN FUNCTION
Add: #14 FLAT MODULE ALL IN LIB (if simple)
Define: #21 STRUCT WITH VEC FIELDS (for collected arguments)
Implement: #36 CONSTRUCTOR FUNCTION NAMED NEW
        #38 FUNCTION RETURNING RESULT (for I/O operations)
Use: #43 MATCH ON RESULT WITH QUESTION MARK
     #46 FOR LOOP OVER BORROWED REFERENCE
     #47 ASSERT MACRO IN FUNCTION BODY
```

### Example: Building an Async Web Service

```
Start with: #6 ASYNC RUNTIME WITH ATTRIBUTE MAIN
Add: #12 PRIVATE MODULE PUBLIC REEXPORT (for route handlers)
     #18 CRATE ROOT REEXPORTING CORE (for clean API)
Define: #29 DERIVE DESERIALIZE FOR FORMS (for request params)
        #30 CUSTOM ERROR STRUCT WITH DISPLAY (for HTTP errors)
Implement: #39 ASYNC FUNCTION WITH AWAIT (for handlers)
           #38 FUNCTION RETURNING RESULT
Use: #43 MATCH ON RESULT WITH QUESTION MARK
```

### Example: Building an FFI Wrapper

```
Start with: #7 UNSAFE FFI WRAPPER CRATE
Add: #16 RAW BINDINGS MODULE (for extern declarations)
     #8 SAFE WRAPPER AROUND UNSAFE (for public API)
     #17 PUBLIC FACADE MODULE
Define: #26 NEWTYPE WRAPPING RAW POINTER
        #27 PHANTOMDATA FOR LIFETIME
        #30 CUSTOM ERROR STRUCT WITH DISPLAY
Implement: #40 UNSAFE FUNCTION WITH SAFETY COMMENT
           #42 FUNCTION TAKING ASREF PATH
Use: #43 MATCH ON RESULT WITH QUESTION MARK
     #47 ASSERT MACRO IN FUNCTION BODY (for invariants)
```

The patterns form a **language** because they work together. Each pattern resolves smaller-scale forces by referencing smaller patterns, which reference still smaller patterns, until you reach the expression level where actual code is written.

## Appendix: Pattern Cross-Reference Table

| Pattern | Scale | References These Patterns | Referenced By |
|---------|-------|--------------------------|---------------|
| 1. Binary with Main | Project | 33, 43, 46 | 3, 6 |
| 2. Library Crate | Project | 9, 18, 28 | 3, 8 |
| 3. Binary and Library | Project | 1, 2, 14 | — |
| 4. Tests Directory | Project | 50, 47 | — |
| 6. Async Runtime | Project | 39, 1 | — |
| 7. Unsafe FFI Wrapper | Project | 16, 40, 26 | 8 |
| 8. Safe Wrapper | Project | 7, 17, 30 | — |
| 9. Module Tree | Architecture | 10, 18 | 2 |
| 10. Submodule Separate | Architecture | — | 9, 11 |
| 11. Nested Submodules | Architecture | 10, 13 | — |
| 12. Private Public Reexport | Architecture | 18, 17 | — |
| 15. Test Module | Architecture | 50, 47 | — |
| 16. Raw Bindings | Architecture | — | 7 |
| 17. Public Facade | Architecture | — | 8, 12 |
| 18. Crate Root Reexport | Architecture | — | 2, 9, 12 |
| 21. Struct with Vec | Type | 24, 33 | 22, 34 |
| 22. Two Vecs Queue | Type | 21, 34, 48 | — |
| 23. Enum Empty NonEmpty | Type | 24, 43 | — |
| 24. Generic Type | Type | — | 21, 23, 25 |
| 25. Trait Bound Impl | Type | 24, 41 | — |
| 26. Newtype Raw Pointer | Type | 27, 40 | 7 |
| 27. PhantomData Lifetime | Type | — | 26 |
| 28. Derive Debug | Type | — | 2, 30 |
| 30. Custom Error | Type | 31, 28 | 8 |
| 31. Type Alias Result | Type | — | 30, 38 |
| 33. Method &self | Function | — | 1, 21, 46 |
| 34. Method &mut self | Function | 21, 48 | 22 |
| 35. Method self | Function | 37 | — |
| 36. Constructor new | Function | — | 22 |
| 38. Function Result | Function | 31, 43 | 42 |
| 39. Async Function | Function | — | 6 |
| 40. Unsafe Function | Function | 47 | 7, 26 |
| 43. Question Mark | Expression | 38 | 1, 23, 38 |
| 46. For Borrowed | Expression | 33 | 1 |
| 47. Assert Macro | Expression | — | 4, 15, 40 |
| 48. Mem Swap | Expression | — | 22, 34 |
| 50. Test Function | Expression | — | 4, 15 |

---

## Conclusion: The Quality Without a Name

Alexander's patterns aim for "the quality without a name"—that sense of rightness when things fit together naturally. In Rust, this quality emerges when:

- **Ownership is clear** - you can see who owns what
- **Lifetimes are minimal** - the compiler infers most relationships
- **Types guide usage** - invalid states are unrepresentable
- **Errors are explicit** - failures are part of the type signature
- **Unsafe is contained** - dangerous code is isolated and documented

These 50 patterns, when composed together, create programs with this quality. They are not rules to follow mechanically, but **living patterns** that you adapt to your situation.

Use them as a language. Speak them. Combine them. Make them your own.

---

*This pattern language is derived from the 24 projects in the Programming Rust code examples repository, including gcd, queue, binary-tree, fern_sim, actix-gcd, libgit2-rs-safe, and many others. Each pattern appears multiple times in the actual code, making them true patterns rather than theoretical constructs.*
