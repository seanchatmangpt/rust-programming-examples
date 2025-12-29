# Pattern Selection Guide
## How to Navigate the Pattern Language

This guide helps you choose which patterns to use when building Rust programs. Like Christopher Alexander's diagrams in *A Pattern Language*, it shows the decision paths through the language.

---

## The Central Question: What Are You Building?

Start here. Every program begins by answering this question.

```
                    WHAT ARE YOU BUILDING?
                            ↓
        ┌───────────────────┴───────────────────┐
        ↓                                       ↓
   Executable Program                    Reusable Library
        ↓                                       ↓
   [Go to Section A]                    [Go to Section B]
```

---

## Section A: Building an Executable Program

### A1. What kind of executable?

```
EXECUTABLE PROGRAM
    ↓
    ├─→ Command-line tool (grep, gcd, copy)
    │   └─→ Pattern #1: BINARY WITH MAIN FUNCTION
    │       └─→ Does it also have reusable logic?
    │           ├─→ Yes → Pattern #3: BINARY AND LIBRARY TOGETHER
    │           └─→ No → Stay with #1
    │
    ├─→ Web service (actix-gcd)
    │   └─→ Pattern #6: ASYNC RUNTIME WITH ATTRIBUTE MAIN
    │       └─→ Pattern #1: BINARY WITH MAIN FUNCTION
    │
    ├─→ Network daemon (echo-server)
    │   └─→ Is it async?
    │       ├─→ Yes → Pattern #6: ASYNC RUNTIME WITH ATTRIBUTE MAIN
    │       └─→ No → Pattern #1: BINARY WITH MAIN FUNCTION
    │
    └─→ FFI demonstration (libgit2-rs example)
        └─→ Pattern #1: BINARY WITH MAIN FUNCTION
            └─→ Pattern #7: UNSAFE FFI WRAPPER CRATE
```

### A2. How complex is the executable?

```
BINARY COMPLEXITY
    ↓
    ├─→ Very simple (< 100 lines, single file)
    │   └─→ Pattern #14: FLAT MODULE ALL IN LIB
    │       └─→ Put everything in main.rs
    │
    ├─→ Moderate (100-500 lines, a few modules)
    │   └─→ Pattern #3: BINARY AND LIBRARY TOGETHER
    │       └─→ Pattern #9: MODULE TREE IN LIB FILE
    │       └─→ Pattern #10: SUBMODULE IN SEPARATE FILE
    │
    └─→ Complex (> 500 lines, many modules)
        └─→ Pattern #3: BINARY AND LIBRARY TOGETHER
            └─→ Pattern #11: NESTED SUBMODULES IN DIRECTORY
            └─→ Pattern #12: PRIVATE MODULE PUBLIC REEXPORT
```

### A3. What does the program do?

```
PROGRAM PURPOSE
    ↓
    ├─→ Processes files/text
    │   └─→ Pattern #38: FUNCTION RETURNING RESULT
    │       └─→ Pattern #42: FUNCTION TAKING ASREF PATH
    │       └─→ Pattern #43: MATCH ON RESULT WITH QUESTION MARK
    │
    ├─→ Makes network requests
    │   └─→ Pattern #39: ASYNC FUNCTION WITH AWAIT (if async)
    │       └─→ Pattern #38: FUNCTION RETURNING RESULT
    │       └─→ Pattern #30: CUSTOM ERROR STRUCT WITH DISPLAY
    │
    ├─→ Serves web requests
    │   └─→ Pattern #6: ASYNC RUNTIME WITH ATTRIBUTE MAIN
    │       └─→ Pattern #29: DERIVE DESERIALIZE FOR FORMS
    │       └─→ Pattern #39: ASYNC FUNCTION WITH AWAIT
    │
    └─→ Calls C libraries
        └─→ Pattern #7: UNSAFE FFI WRAPPER CRATE
            └─→ Pattern #26: NEWTYPE WRAPPING RAW POINTER
            └─→ Pattern #40: UNSAFE FUNCTION WITH SAFETY COMMENT
```

---

## Section B: Building a Reusable Library

### B1. What kind of library?

```
REUSABLE LIBRARY
    ↓
    ├─→ Data structure library (queue, binary-tree)
    │   └─→ Pattern #2: LIBRARY CRATE WITH PUBLIC API
    │       └─→ Go to Section C: DEFINING TYPES
    │
    ├─→ Algorithm library (complex number math)
    │   └─→ Pattern #2: LIBRARY CRATE WITH PUBLIC API
    │       └─→ Pattern #24: GENERIC TYPE WITH PARAMETER T
    │       └─→ Pattern #25: TRAIT BOUND ON IMPL BLOCK
    │
    ├─→ Safe wrapper around unsafe code
    │   └─→ Pattern #8: SAFE WRAPPER AROUND UNSAFE
    │       └─→ Pattern #16: RAW BINDINGS MODULE
    │       └─→ Pattern #17: PUBLIC FACADE MODULE
    │
    └─→ Domain logic library (fern_sim)
        └─→ Pattern #2: LIBRARY CRATE WITH PUBLIC API
            └─→ Pattern #11: NESTED SUBMODULES IN DIRECTORY
            └─→ Pattern #13: FEATURE-BASED MODULE GROUPS
```

### B2. How should modules be organized?

```
MODULE ORGANIZATION
    ↓
    ├─→ Small library (< 500 lines)
    │   └─→ Pattern #14: FLAT MODULE ALL IN LIB
    │
    ├─→ Medium library (500-2000 lines)
    │   └─→ Pattern #9: MODULE TREE IN LIB FILE
    │       └─→ Pattern #10: SUBMODULE IN SEPARATE FILE
    │       └─→ Pattern #18: CRATE ROOT REEXPORTING CORE
    │
    └─→ Large library (> 2000 lines)
        └─→ Pattern #11: NESTED SUBMODULES IN DIRECTORY
            └─→ Pattern #12: PRIVATE MODULE PUBLIC REEXPORT
            └─→ Pattern #13: FEATURE-BASED MODULE GROUPS
            └─→ Pattern #17: PUBLIC FACADE MODULE
```

### B3. What should the public API look like?

```
PUBLIC API DESIGN
    ↓
    ├─→ Expose internal structure
    │   └─→ Pattern #9: MODULE TREE IN LIB FILE
    │       └─→ Users write: use mycrate::module::Type;
    │
    ├─→ Hide internal structure
    │   └─→ Pattern #12: PRIVATE MODULE PUBLIC REEXPORT
    │       └─→ Pattern #18: CRATE ROOT REEXPORTING CORE
    │       └─→ Users write: use mycrate::Type;
    │
    └─→ Hybrid (most common)
        └─→ Pattern #12: PRIVATE MODULE PUBLIC REEXPORT
            └─→ Re-export common types at root
            └─→ Keep modules public for advanced users
```

---

## Section C: Defining Types

### C1. What kind of data are you storing?

```
DATA STORAGE
    ↓
    ├─→ Collection of same-type items
    │   └─→ Pattern #21: STRUCT WITH VEC FIELDS
    │       └─→ Pattern #24: GENERIC TYPE WITH PARAMETER T
    │
    ├─→ Multiple collections working together
    │   └─→ Pattern #22: STRUCT WITH TWO VECS FOR QUEUE
    │       └─→ Pattern #24: GENERIC TYPE WITH PARAMETER T
    │       └─→ Pattern #48: MEM SWAP FOR MOVING VALUES
    │
    ├─→ Recursive data (tree, list)
    │   └─→ Pattern #23: ENUM WITH EMPTY AND NONEMPTY
    │       └─→ Pattern #24: GENERIC TYPE WITH PARAMETER T
    │
    ├─→ Variants/alternatives
    │   └─→ Use enum (not a numbered pattern)
    │       └─→ Pattern #43: MATCH ON RESULT WITH QUESTION MARK
    │
    └─→ C pointer/resource
        └─→ Pattern #26: NEWTYPE WRAPPING RAW POINTER
            └─→ Pattern #27: PHANTOMDATA FOR LIFETIME
```

### C2. Does the type need trait implementations?

```
TRAIT IMPLEMENTATIONS
    ↓
    ├─→ Need debugging output
    │   └─→ Pattern #28: DERIVE DEBUG FOR TESTING
    │
    ├─→ Need to parse from external data
    │   └─→ Pattern #29: DERIVE DESERIALIZE FOR FORMS
    │
    ├─→ Type represents an error
    │   └─→ Pattern #30: CUSTOM ERROR STRUCT WITH DISPLAY
    │       └─→ Pattern #31: TYPE ALIAS FOR RESULT
    │
    ├─→ Operations only for specific types
    │   └─→ Pattern #25: TRAIT BOUND ON IMPL BLOCK
    │
    └─→ Generic operations
        └─→ Pattern #24: GENERIC TYPE WITH PARAMETER T
            └─→ Pattern #41: GENERIC FUNCTION WITH WHERE CLAUSE
```

### C3. What are the ownership characteristics?

```
OWNERSHIP DESIGN
    ↓
    ├─→ Type owns its data
    │   └─→ Pattern #21: STRUCT WITH VEC FIELDS
    │       └─→ Pattern #35: METHOD CONSUMING SELF (for split)
    │
    ├─→ Type borrows data (has lifetime)
    │   └─→ Pattern #27: PHANTOMDATA FOR LIFETIME
    │       └─→ Lifetime parameters on struct
    │
    ├─→ Type manages unsafe resource
    │   └─→ Pattern #26: NEWTYPE WRAPPING RAW POINTER
    │       └─→ Implement Drop to clean up
    │
    └─→ Type is just a marker
        └─→ Pattern #32: UNIT STRUCT FOR MARKER
```

---

## Section D: Implementing Functions

### D1. What does the function take?

```
FUNCTION PARAMETERS
    ↓
    ├─→ Reads from struct without modifying
    │   └─→ Pattern #33: METHOD TAKING SELF BY REFERENCE
    │       └─→ fn method(&self) -> T
    │
    ├─→ Modifies struct
    │   └─→ Pattern #34: METHOD TAKING SELF BY MUT REFERENCE
    │       └─→ fn method(&mut self)
    │
    ├─→ Consumes struct (takes ownership)
    │   └─→ Pattern #35: METHOD CONSUMING SELF
    │       └─→ fn method(self) -> T
    │       └─→ Often returns something else (split, into_inner)
    │
    ├─→ Creates new instance
    │   └─→ Pattern #36: CONSTRUCTOR FUNCTION NAMED NEW
    │       └─→ fn new() -> Self
    │
    ├─→ Builds up instance step by step
    │   └─→ Pattern #37: BUILDER METHOD RETURNING SELF
    │       └─→ fn with_x(mut self, x: X) -> Self
    │
    └─→ Flexible path parameter
        └─→ Pattern #42: FUNCTION TAKING ASREF PATH
            └─→ fn open<P: AsRef<Path>>(path: P)
```

### D2. What does the function return?

```
RETURN VALUES
    ↓
    ├─→ Can fail
    │   └─→ Pattern #38: FUNCTION RETURNING RESULT
    │       └─→ Pattern #31: TYPE ALIAS FOR RESULT
    │       └─→ Pattern #43: MATCH ON RESULT WITH QUESTION MARK
    │
    ├─→ Might not have a value
    │   └─→ Return Option<T>
    │       └─→ Pattern #44: IF LET FOR OPTION UNWRAPPING
    │
    ├─→ Async operation
    │   └─→ Pattern #39: ASYNC FUNCTION WITH AWAIT
    │       └─→ async fn operation() -> Result<T>
    │
    └─→ Always succeeds
        └─→ Return T directly
```

### D3. Is the function safe or unsafe?

```
SAFETY LEVEL
    ↓
    ├─→ Safe (most functions)
    │   └─→ Use safe Rust only
    │
    └─→ Unsafe (FFI, pointer manipulation)
        └─→ Pattern #40: UNSAFE FUNCTION WITH SAFETY COMMENT
            └─→ Document why it's safe to call
            └─→ Document caller's obligations
```

---

## Section E: Writing Expressions

### E1. How do you handle errors?

```
ERROR HANDLING
    ↓
    ├─→ Propagate to caller
    │   └─→ Pattern #43: MATCH ON RESULT WITH QUESTION MARK
    │       └─→ operation()?
    │
    ├─→ Handle one case, ignore others
    │   └─→ Pattern #44: IF LET FOR OPTION UNWRAPPING
    │       └─→ if let Some(x) = option { ... }
    │
    ├─→ Crash if condition fails
    │   └─→ Pattern #47: ASSERT MACRO IN FUNCTION BODY
    │       └─→ assert!(condition);
    │
    └─→ Handle all cases
        └─→ match expression (not a numbered pattern)
```

### E2. How do you iterate?

```
ITERATION
    ↓
    ├─→ Over collection without consuming
    │   └─→ Pattern #46: FOR LOOP OVER BORROWED REFERENCE
    │       └─→ for item in &collection { ... }
    │
    ├─→ Over collection, consuming it
    │   └─→ for item in collection { ... }
    │
    ├─→ Manual iteration with control
    │   └─→ Pattern #45: WHILE LET FOR ITERATION
    │       └─→ while let Some(item) = iter.next() { ... }
    │
    └─→ With iterator adapters
        └─→ collection.iter().map(...).filter(...).collect()
```

### E3. How do you move values?

```
MOVING VALUES
    ↓
    ├─→ Exchange two values
    │   └─→ Pattern #48: MEM SWAP FOR MOVING VALUES
    │       └─→ std::mem::swap(&mut a, &mut b)
    │
    ├─→ Need owned value but only have reference
    │   └─→ Pattern #49: CLONE TO EXTEND LIFETIME
    │       └─→ let owned = borrowed.clone()
    │
    └─→ Move out of one location into another
        └─→ Use mem::replace or mem::take
```

### E4. How do you test?

```
TESTING
    ↓
    ├─→ Unit test in same file
    │   └─→ Pattern #15: TEST MODULE WITH USE SUPER STAR
    │       └─→ Pattern #50: TEST FUNCTION WITH ATTRIBUTE
    │       └─→ Pattern #47: ASSERT MACRO IN FUNCTION BODY
    │
    ├─→ Integration test
    │   └─→ Pattern #4: TESTS DIRECTORY BESIDE SOURCE
    │       └─→ Pattern #50: TEST FUNCTION WITH ATTRIBUTE
    │
    └─→ Example program
        └─→ Pattern #5: EXAMPLES DIRECTORY FOR USAGE
```

---

## Common Pattern Combinations

These are frequent combinations that appear together:

### Combination 1: Basic CLI Tool
```
#1 BINARY WITH MAIN FUNCTION
  + #38 FUNCTION RETURNING RESULT
  + #42 FUNCTION TAKING ASREF PATH
  + #43 MATCH ON RESULT WITH QUESTION MARK
  + #46 FOR LOOP OVER BORROWED REFERENCE
  + #47 ASSERT MACRO IN FUNCTION BODY
```
**Example**: gcd, grep, copy

### Combination 2: Data Structure Library
```
#2 LIBRARY CRATE WITH PUBLIC API
  + #21 STRUCT WITH VEC FIELDS
  + #24 GENERIC TYPE WITH PARAMETER T
  + #28 DERIVE DEBUG FOR TESTING
  + #33 METHOD TAKING SELF BY REFERENCE
  + #34 METHOD TAKING SELF BY MUT REFERENCE
  + #36 CONSTRUCTOR FUNCTION NAMED NEW
  + #15 TEST MODULE WITH USE SUPER STAR
```
**Example**: queue, generic-queue, binary-tree

### Combination 3: Async Web Service
```
#6 ASYNC RUNTIME WITH ATTRIBUTE MAIN
  + #29 DERIVE DESERIALIZE FOR FORMS
  + #39 ASYNC FUNCTION WITH AWAIT
  + #30 CUSTOM ERROR STRUCT WITH DISPLAY
  + #31 TYPE ALIAS FOR RESULT
  + #43 MATCH ON RESULT WITH QUESTION MARK
```
**Example**: actix-gcd

### Combination 4: FFI Safe Wrapper
```
#8 SAFE WRAPPER AROUND UNSAFE
  + #16 RAW BINDINGS MODULE
  + #17 PUBLIC FACADE MODULE
  + #26 NEWTYPE WRAPPING RAW POINTER
  + #27 PHANTOMDATA FOR LIFETIME
  + #40 UNSAFE FUNCTION WITH SAFETY COMMENT
  + #30 CUSTOM ERROR STRUCT WITH DISPLAY
  + #42 FUNCTION TAKING ASREF PATH
```
**Example**: libgit2-rs-safe

### Combination 5: Large Multi-Module Library
```
#2 LIBRARY CRATE WITH PUBLIC API
  + #11 NESTED SUBMODULES IN DIRECTORY
  + #12 PRIVATE MODULE PUBLIC REEXPORT
  + #13 FEATURE-BASED MODULE GROUPS
  + #18 CRATE ROOT REEXPORTING CORE
  + #17 PUBLIC FACADE MODULE
  + #4 TESTS DIRECTORY BESIDE SOURCE
```
**Example**: fern_sim

### Combination 6: Queue Implementation
```
#22 STRUCT WITH TWO VECS FOR QUEUE
  + #24 GENERIC TYPE WITH PARAMETER T
  + #34 METHOD TAKING SELF BY MUT REFERENCE
  + #36 CONSTRUCTOR FUNCTION NAMED NEW
  + #48 MEM SWAP FOR MOVING VALUES
  + #44 IF LET FOR OPTION UNWRAPPING
  + #15 TEST MODULE WITH USE SUPER STAR
```
**Example**: queue, generic-queue

---

## Quick Reference: Pattern by Number

When you see a pattern number referenced, here's what it means:

**PROJECT (1-8)**
- 1: Binary with Main / 2: Library Crate / 3: Binary+Library
- 4: Tests Directory / 6: Async Runtime / 7: FFI Wrapper / 8: Safe Wrapper

**ARCHITECTURE (9-20)**
- 9: Module Tree / 10: Submodule File / 11: Nested Submodules
- 12: Private Reexport / 13: Feature Groups / 14: Flat Module
- 15: Test Module / 16: Raw Bindings / 17: Public Facade / 18: Crate Root Reexport

**TYPES (21-32)**
- 21: Struct+Vec / 22: Two Vecs Queue / 23: Enum Empty/NonEmpty
- 24: Generic Type / 25: Trait Bound / 26: Newtype Pointer / 27: PhantomData
- 28: Derive Debug / 29: Derive Deserialize / 30: Custom Error / 31: Type Alias Result

**FUNCTIONS (33-42)**
- 33: &self / 34: &mut self / 35: self / 36: new() / 37: Builder
- 38: Result / 39: async fn / 40: unsafe / 42: AsRef<Path>

**EXPRESSIONS (43-50)**
- 43: ? operator / 44: if let / 45: while let / 46: for &ref
- 47: assert! / 48: mem::swap / 49: clone / 50: #[test]

---

## How to Use This Guide

1. **Start at the top**: "What are you building?"
2. **Follow the decision tree** to find relevant patterns
3. **Note the pattern numbers**
4. **Look up full patterns** in the main book
5. **Check common combinations** for your use case
6. **Compose patterns** like a language

Remember: patterns are not rules. They are options. Choose the ones that fit your situation.
