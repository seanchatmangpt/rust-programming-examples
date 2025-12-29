# A Pattern Language for Rust Programming

**Following Christopher Alexander's Methodology**

---

## What Is This?

This is a **pattern language** for Rust programming, modeled after Christopher Alexander's *A Pattern Language: Towns, Buildings, Construction* (1977). It contains **50 concrete, numbered patterns** that work together as a generative system for building Rust programs.

Unlike typical programming books that teach concepts linearly, this language lets you **compose patterns** to create programs. Each pattern solves a specific problem and references other patterns that complete it.

## Why Alexander's Method?

Christopher Alexander discovered that great architecture emerges not from abstract principles but from **concrete, named patterns** that connect in a directed graph. His patterns have names like "Six-Foot Balcony" and "Light on Two Sides of Every Room"—not abstract concepts like "Natural Lighting Principle."

We apply the same method to Rust:
- **Not** "Error Handling Pattern" → **Yes** "Match on Result with Question Mark" (#43)
- **Not** "Container Pattern" → **Yes** "Struct with Two Vecs for Queue" (#22)
- **Not** "API Design" → **Yes** "Private Module Public Reexport" (#12)

Each pattern is **concrete** and **recognizable** in actual code.

## The Five Scales

Patterns exist at five scales, from largest to smallest:

### 1. PROJECT SCALE (Patterns 1-8)
**The whole program—what kind of thing are we building?**
- Binary with Main Function (#1)
- Library Crate with Public API (#2)
- Async Runtime with Attribute Main (#6)
- Safe Wrapper Around Unsafe (#8)
- [5 more patterns...]

### 2. ARCHITECTURE SCALE (Patterns 9-20)
**Module organization—how is code structured?**
- Module Tree in Lib File (#9)
- Nested Submodules in Directory (#11)
- Private Module Public Reexport (#12)
- Test Module with Use Super Star (#15)
- [8 more patterns...]

### 3. TYPE SCALE (Patterns 21-32)
**Data structures—what are the core types?**
- Struct with Vec Fields (#21)
- Struct with Two Vecs for Queue (#22)
- Enum with Empty and NonEmpty (#23)
- Generic Type with Parameter T (#24)
- Newtype Wrapping Raw Pointer (#26)
- Custom Error Struct with Display (#30)
- [6 more patterns...]

### 4. FUNCTION SCALE (Patterns 33-42)
**Behavior—how do functions work?**
- Method Taking Self by Reference (#33)
- Method Taking Self by Mut Reference (#34)
- Constructor Function Named New (#36)
- Function Returning Result (#38)
- Async Function with Await (#39)
- Unsafe Function with Safety Comment (#40)
- [4 more patterns...]

### 5. EXPRESSION SCALE (Patterns 43-50)
**Code idioms—what constructs are used?**
- Match on Result with Question Mark (#43)
- If Let for Option Unwrapping (#44)
- For Loop Over Borrowed Reference (#46)
- Mem Swap for Moving Values (#48)
- [4 more patterns...]

## Documents in This Collection

### 📘 Start Here
**[README.md](README.md)** (this file)
- Overview of the pattern language
- How to use it
- Quick start guide

### 📖 Core Documents
**[ALEXANDER_PATTERN_LANGUAGE_PLAN.md](ALEXANDER_PATTERN_LANGUAGE_PLAN.md)**
- Complete list of all 50 patterns
- The pattern graph (how patterns connect)
- Book structure outline
- One complete example pattern (#22: Struct with Two Vecs for Queue)

**[SAMPLE_PATTERNS_ALEXANDER_FORMAT.md](SAMPLE_PATTERNS_ALEXANDER_FORMAT.md)**
- Five complete patterns in Alexander's exact format
- Demonstrates patterns at different scales
- Shows how patterns reference each other

**[PATTERN_SELECTION_GUIDE.md](PATTERN_SELECTION_GUIDE.md)**
- Decision trees for choosing patterns
- Common pattern combinations
- Quick reference by pattern number
- "If you're building X, use patterns Y, Z..."

### 🔍 How to Navigate

1. **New to the pattern language?**
   - Read this README first
   - Read one complete pattern in SAMPLE_PATTERNS
   - Try the decision trees in PATTERN_SELECTION_GUIDE

2. **Starting a new project?**
   - Go to PATTERN_SELECTION_GUIDE
   - Follow the "What are you building?" decision tree
   - Note the pattern numbers
   - Look them up in ALEXANDER_PATTERN_LANGUAGE_PLAN

3. **Want to understand the methodology?**
   - Read the introduction in ALEXANDER_PATTERN_LANGUAGE_PLAN
   - Study the sample patterns in SAMPLE_PATTERNS
   - See how patterns form a directed graph

4. **Looking for a specific pattern?**
   - Check the Quick Reference in PATTERN_SELECTION_GUIDE
   - Use the pattern numbers to navigate

## Quick Start Examples

### Example 1: Building a CLI Tool (like grep or gcd)

**Patterns you'll use:**
```
#1  Binary with Main Function
#38 Function Returning Result
#42 Function Taking AsRef Path
#43 Match on Result with Question Mark
#46 For Loop Over Borrowed Reference
#47 Assert Macro in Function Body
```

**See it in action:**
- `/home/user/rust-programming-examples/gcd/src/main.rs`
- `/home/user/rust-programming-examples/grep/src/main.rs`

### Example 2: Building a Data Structure Library (like queue)

**Patterns you'll use:**
```
#2  Library Crate with Public API
#21 Struct with Vec Fields
#24 Generic Type with Parameter T
#28 Derive Debug for Testing
#33 Method Taking Self by Reference
#34 Method Taking Self by Mut Reference
#36 Constructor Function Named New
#15 Test Module with Use Super Star
```

**See it in action:**
- `/home/user/rust-programming-examples/queue/src/lib.rs`
- `/home/user/rust-programming-examples/generic-queue/src/lib.rs`

### Example 3: Building an Async Web Service (like actix-gcd)

**Patterns you'll use:**
```
#6  Async Runtime with Attribute Main
#29 Derive Deserialize for Forms
#39 Async Function with Await
#30 Custom Error Struct with Display
#31 Type Alias for Result
#43 Match on Result with Question Mark
```

**See it in action:**
- `/home/user/rust-programming-examples/actix-gcd/src/main.rs`

### Example 4: Building an FFI Safe Wrapper (like libgit2-rs-safe)

**Patterns you'll use:**
```
#8  Safe Wrapper Around Unsafe
#16 Raw Bindings Module
#17 Public Facade Module
#26 Newtype Wrapping Raw Pointer
#27 PhantomData for Lifetime
#40 Unsafe Function with Safety Comment
#30 Custom Error Struct with Display
```

**See it in action:**
- `/home/user/rust-programming-examples/libgit2-rs-safe/src/git/mod.rs`

## The Pattern Graph (Simplified)

Patterns connect to each other. Larger patterns reference smaller ones:

```
PROJECT SCALE
  ↓
  references
  ↓
ARCHITECTURE SCALE
  ↓
  references
  ↓
TYPE SCALE
  ↓
  references
  ↓
FUNCTION SCALE
  ↓
  references
  ↓
EXPRESSION SCALE
```

For example:
- **#1 (Binary with Main)** references **#43 (Question Mark)** and **#46 (For Loop)**
- **#22 (Two Vecs Queue)** references **#24 (Generic Type)** and **#48 (Mem Swap)**
- **#38 (Function Returning Result)** references **#31 (Type Alias Result)** and **#43 (Question Mark)**

See the complete graph in ALEXANDER_PATTERN_LANGUAGE_PLAN.md.

## How to Use This Language

### The Alexander Method

1. **Start with a problem** ("I need to build a CLI tool")
2. **Choose a large-scale pattern** (#1: Binary with Main Function)
3. **That pattern references smaller patterns** (#43, #46, #47)
4. **Follow the references down** until you reach code expressions
5. **Compose the patterns** to build your program

This is a **generative system**—not a cookbook of recipes, but a language you speak by combining patterns.

### Working Top-Down

Start at the PROJECT scale and work down:

```
1. What kind of crate? → Choose pattern #1-8
2. How organize modules? → Choose patterns #9-20
3. What data structures? → Choose patterns #21-32
4. What functions? → Choose patterns #33-42
5. What code idioms? → Choose patterns #43-50
```

### Working Bottom-Up

Or start with a specific problem and work up:

```
"I need to propagate errors..." → Pattern #43 (Question Mark)
  ↑ requires
Pattern #38 (Function Returning Result)
  ↑ appears in
Pattern #1 (Binary with Main) or #2 (Library Crate)
```

Both directions work—that's the power of a pattern language.

## Real Code Examples

Every pattern is derived from actual code in this repository:

| Pattern | Examples in Repository |
|---------|----------------------|
| #1 Binary with Main | gcd, grep, copy, http-get, echo-server |
| #2 Library Crate | queue, complex, interval, binary-tree |
| #3 Binary and Library | Most projects have both |
| #6 Async Runtime | actix-gcd, cheapo-request, many-requests |
| #8 Safe Wrapper | libgit2-rs-safe (wraps libgit2-rs) |
| #11 Nested Submodules | fern_sim (plant_structures/) |
| #22 Two Vecs Queue | queue, generic-queue |
| #23 Enum Empty/NonEmpty | binary-tree (BinaryTree enum) |
| #26 Newtype Pointer | libgit2-rs-safe (Repository, Commit) |
| #43 Question Mark | Used in 20+ projects |

You can read the actual implementations to see patterns in context.

## Pattern Format (Alexander's Template)

Each complete pattern follows this structure:

```
## NUMBER. PATTERN NAME

[Illustration description]

...context paragraph linking to larger patterns...

◆ ◆ ◆

**Problem statement in bold**

Body text explaining:
- Why this problem exists
- What forces are at play
- Why other solutions don't work
- The fundamental insight

**Therefore: Solution in bold**

Code example showing the pattern

[Diagram description]

◆ ◆ ◆

References to smaller patterns that complete this one
```

See examples in SAMPLE_PATTERNS_ALEXANDER_FORMAT.md.

## What Makes This Different?

### Compared to Programming Books
- **Not linear**: You don't read cover-to-cover
- **Compositional**: Combine patterns like Lego bricks
- **Concrete**: Names describe actual code, not abstractions
- **Interconnected**: Patterns reference each other in a graph

### Compared to Design Patterns (Gang of Four)
- **More granular**: 50 patterns vs 23
- **More concrete**: "Struct with Two Vecs for Queue" vs "Strategy Pattern"
- **Language-specific**: For Rust, not generic OOP
- **Scale-aware**: From project structure down to expressions

### Compared to Best Practices Lists
- **Generative**: Build programs by composing patterns
- **Contextual**: Each pattern explains when to use it
- **Interconnected**: Patterns work together as a language
- **Proven**: Derived from real code examples

## The Source Material

These patterns are extracted from 24 real Rust projects in this repository:

- **Basic**: gcd, queue, complex, interval
- **Advanced**: binary-tree, generic-queue, fern_sim
- **Async**: cheapo-request, many-requests, spawn-blocking, block-on
- **Web**: actix-gcd, http-get
- **Unsafe/FFI**: ascii, ref-with-flag, gap-buffer, libgit2-rs, libgit2-rs-safe
- **Tools**: grep, copy, echo-server
- **Macros**: json-macro

Each pattern appears multiple times in the actual code, making them true patterns (not theoretical constructs).

## Learning Path

### For Beginners
1. Read this README
2. Read Pattern #1 (Binary with Main) in SAMPLE_PATTERNS
3. Read Pattern #43 (Question Mark) in SAMPLE_PATTERNS
4. Look at `/home/user/rust-programming-examples/gcd/src/main.rs`
5. Try building something using patterns #1, #43, #46, #47

### For Intermediate Rustaceans
1. Skim the complete pattern list in ALEXANDER_PATTERN_LANGUAGE_PLAN
2. Pick a project type from PATTERN_SELECTION_GUIDE
3. Follow the decision tree to find relevant patterns
4. Read those patterns in detail
5. Study the corresponding code examples

### For Advanced Users
1. Read the full pattern graph in ALEXANDER_PATTERN_LANGUAGE_PLAN
2. Study how patterns compose in SAMPLE_PATTERNS
3. Use the patterns as a code review checklist
4. Contribute new patterns you discover

## The Quality Without a Name

Alexander's patterns aim for "the quality without a name"—that sense of rightness when things fit together naturally. In Rust, this emerges when:

- **Ownership is clear** - you can see who owns what
- **Lifetimes are minimal** - the compiler infers most relationships
- **Types guide usage** - invalid states are unrepresentable
- **Errors are explicit** - failures are part of the type signature
- **Unsafe is contained** - dangerous code is isolated and documented

When you compose these 50 patterns, your programs will have this quality.

## Contributing

This pattern language is **alive**. As Rust evolves and new patterns emerge, they can be added:

### Pattern Criteria
To be included, a pattern must:
1. ✅ Appear in multiple real programs (not theoretical)
2. ✅ Solve a recurring problem (not a one-off trick)
3. ✅ Have a concrete, descriptive name (not abstract)
4. ✅ Reference other patterns (fits in the graph)
5. ✅ Be at a consistent scale (PROJECT/ARCH/TYPE/FUNC/EXPR)

### Potential New Patterns
As you work with Rust, you may discover patterns not yet in this language. Candidates might include:

- **Cfg-gated modules** (compile-time platform selection)
- **Feature flags in Cargo** (optional dependencies)
- **Trait object with Box** (dynamic dispatch)
- **Iterator with adapters** (map/filter/collect chain)
- **State machine with enums** (type-safe state transitions)

If you find patterns that meet the criteria, document them using Alexander's format.

## References

### Christopher Alexander's Original Work
- *A Pattern Language* (1977) - The 253 architecture patterns
- *The Timeless Way of Building* (1979) - The philosophy
- *The Nature of Order* (2002-2004) - The deeper theory

### Rust Resources
- This repository: `/home/user/rust-programming-examples`
- Programming Rust book (O'Reilly)
- The Rust Programming Language (rust-lang.org)

### Related Pattern Languages
- *Design Patterns* (Gang of Four, 1994) - OOP patterns
- *Pattern-Oriented Software Architecture* (Buschmann et al.)
- *Refactoring* (Fowler) - Code transformation patterns

## File Listing

```
patterns-book/
├── README.md (this file)
├── ALEXANDER_PATTERN_LANGUAGE_PLAN.md
│   └── Complete list of 50 patterns
│       Pattern graph
│       Book structure
│       One complete example
│
├── SAMPLE_PATTERNS_ALEXANDER_FORMAT.md
│   └── Five complete patterns:
│       #1 Binary with Main Function
│       #12 Private Module Public Reexport
│       #26 Newtype Wrapping Raw Pointer
│       #43 Match on Result with Question Mark
│       #48 Mem Swap for Moving Values
│
└── PATTERN_SELECTION_GUIDE.md
    └── Decision trees
        Common combinations
        Quick reference
        Navigation guide
```

## Next Steps

1. **Read a complete pattern**: Go to SAMPLE_PATTERNS and read Pattern #1
2. **Try the decision tree**: Go to PATTERN_SELECTION_GUIDE and answer "What are you building?"
3. **Look at real code**: Open one of the example projects and find the patterns
4. **Build something**: Start a project using patterns from the language

Remember: **patterns are a language**. You learn a language by speaking it, not by memorizing grammar. Start composing patterns to build programs, and the language will become natural.

---

*"This language, like a seed, is a genetic thing... it is a gate, through which you pass when you begin to build."*
— Christopher Alexander, *A Pattern Language*
