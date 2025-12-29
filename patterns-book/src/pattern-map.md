# Pattern Map: The Landscape of Rust Patterns

## Introduction

This pattern map shows the relationships, dependencies, and flows between all patterns in this book. Like Christopher Alexander's pattern languages for architecture, Rust patterns form a coherent language where patterns reference and build upon each other.

Use this map to:
- **Understand dependencies**: Which patterns require knowledge of others
- **Navigate the pattern language**: Follow paths from simple to complex
- **Plan learning**: See prerequisite chains
- **Design systems**: Identify patterns that work together
- **Discover connections**: Find unexpected relationships

---

## The Pattern Landscape (Visual Overview)

```
                    RUST PATTERN LANGUAGE
                            |
        +-------------------+-------------------+
        |                   |                   |
    OWNERSHIP           TYPES &              BEHAVIOR
     SYSTEM             TRAITS              PATTERNS
        |                   |                   |
        v                   v                   v
```

### Foundation Layer (Learning Prerequisites)

These patterns form the bedrock of Rust programming. Master these first.

```
┌─────────────────────────────────────────────────────────────────┐
│                    FOUNDATION PATTERNS                          │
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    │
│  │  Ownership   │───▶│  Borrowing   │───▶│  Lifetimes   │    │
│  │  Transfer    │    │  (&T, &mut T)│    │  ('a, 'b)    │    │
│  └──────────────┘    └──────────────┘    └──────────────┘    │
│         │                    │                    │             │
│         └────────────────────┴────────────────────┘             │
│                              ▼                                   │
│                    ┌──────────────────┐                         │
│                    │      RAII        │                         │
│                    │ (Resource Scope) │                         │
│                    └──────────────────┘                         │
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    │
│  │   Struct     │    │     Enum     │    │   Generic    │    │
│  │ Definition   │    │  Definition  │    │    Types     │    │
│  └──────────────┘    └──────────────┘    └──────────────┘    │
│         │                    │                    │             │
│         └────────────────────┴────────────────────┘             │
│                              ▼                                   │
│                    ┌──────────────────┐                         │
│                    │      Trait       │                         │
│                    │ Implementation   │                         │
│                    └──────────────────┘                         │
└─────────────────────────────────────────────────────────────────┘
```

**Key Relationships**:
- **Ownership → Borrowing → Lifetimes**: Linear progression of reference safety
- **Struct/Enum → Generic Types**: Data definition to parameterization
- **All types → Trait Implementation**: Behavior definition

---

## Core Pattern Clusters

### Cluster 1: Ownership & Memory Management

```
                    ┌─────────────────┐
                    │   OWNERSHIP     │
                    │    TRANSFER     │
                    └────────┬────────┘
                             │
            ┌────────────────┼────────────────┐
            ▼                ▼                ▼
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │   Immutable  │ │   Mutable    │ │     Move     │
    │   Borrow     │ │   Borrow     │ │  Semantics   │
    │     (&T)     │ │   (&mut T)   │ │              │
    └──────┬───────┘ └──────┬───────┘ └──────┬───────┘
           │                │                │
           └────────────────┼────────────────┘
                            ▼
                  ┌──────────────────┐
                  │    Lifetimes     │
                  │   'a, 'static    │
                  └────────┬─────────┘
                           │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │     RAII     │ │  Reference   │ │  Interior    │
    │  (Drop)      │ │  Counting    │ │ Mutability   │
    │              │ │  (Rc/Arc)    │ │ (RefCell)    │
    └──────────────┘ └──────────────┘ └──────────────┘
```

**Pattern Flow**:
1. Start with **Ownership Transfer** (default behavior)
2. When multiple readers needed → **Immutable Borrow**
3. When single writer needed → **Mutable Borrow**
4. When borrows complex → Add **Lifetimes**
5. For automatic cleanup → **RAII**
6. For shared ownership → **Rc/Arc**
7. For runtime borrow checking → **RefCell**

**Anti-Pattern Exits**:
- Fighting Borrow Checker → Review borrowing rules
- Excessive Cloning → Use borrowing instead
- Leaking References → Fix lifetime issues

---

### Cluster 2: Type System & Abstractions

```
                  ┌──────────────────┐
                  │  TYPE DEFINITION │
                  │ (Struct, Enum)   │
                  └────────┬─────────┘
                           │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │   Generic    │ │   Newtype    │ │  Type State  │
    │    Types     │ │   Pattern    │ │   Pattern    │
    │   <T, U>     │ │  Wrapper     │ │ Compile-time │
    └──────┬───────┘ └──────┬───────┘ └──────┬───────┘
           │                │                │
           └────────────────┼────────────────┘
                            ▼
                  ┌──────────────────┐
                  │      TRAIT       │
                  │ IMPLEMENTATION   │
                  └────────┬─────────┘
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│  Operator    │   │  From/Into   │   │   Custom     │
│ Overloading  │   │  Conversion  │   │  Behavior    │
│ (Add, Sub)   │   │              │   │              │
└──────┬───────┘   └──────┬───────┘   └──────┬───────┘
       │                  │                  │
       └──────────────────┼──────────────────┘
                          ▼
              ┌──────────────────────┐
              │   Trait Objects      │
              │ (Dynamic Dispatch)   │
              │     dyn Trait        │
              └──────────────────────┘
```

**Pattern Flow**:
1. Define types with **Struct/Enum**
2. Add flexibility with **Generic Types**
3. Add type safety with **Newtype**
4. Encode state with **Type State**
5. Implement behavior with **Traits**
6. Make intuitive with **Operator Overloading**
7. Enable conversions with **From/Into**
8. Allow polymorphism with **Trait Objects**

**Anti-Pattern Exits**:
- Primitive Obsession → Use Newtype
- God Object → Split into focused types
- Trait Soup → Combine related traits

---

### Cluster 3: Error Handling

```
              ┌──────────────────────┐
              │   FALLIBLE           │
              │   OPERATIONS         │
              └──────────┬───────────┘
                         │
        ┌────────────────┼────────────────┐
        ▼                                 ▼
┌──────────────┐                  ┌──────────────┐
│   Option<T>  │                  │  Result<T,E> │
│  Some/None   │                  │   Ok/Err     │
└──────┬───────┘                  └──────┬───────┘
       │                                 │
       │         ┌──────────────────┐    │
       └────────▶│   ? Operator     │◀───┘
                 │  Error Propagate │
                 └────────┬─────────┘
                          │
            ┌─────────────┼─────────────┐
            ▼             ▼             ▼
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │   Custom     │ │    Error     │ │    Error     │
    │    Error     │ │ Conversion   │ │   Context    │
    │    Types     │ │  (From)      │ │   (Debug)    │
    └──────────────┘ └──────────────┘ └──────────────┘
                          │
                          ▼
                ┌──────────────────┐
                │  Error Type      │
                │  Hierarchy       │
                └──────────────────┘
```

**Pattern Flow**:
1. Use **Option** for nullable values
2. Use **Result** for fallible operations
3. Propagate with **? Operator**
4. Create **Custom Error Types** for domain
5. Enable conversion with **Error From**
6. Add debugging with **Error Context**
7. Organize with **Error Type Hierarchy**

**Anti-Pattern Exits**:
- Unwrap Abuse → Use Result and ?
- Swallowing Errors → Log and propagate
- Generic Errors → Add context

---

### Cluster 4: Collections & Iteration

```
              ┌──────────────────────┐
              │    COLLECTIONS       │
              │  Vec, HashMap, etc.  │
              └──────────┬───────────┘
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│  Vec<T>      │ │ HashMap<K,V> │ │  BTreeMap    │
│ Sequential   │ │   Lookup     │ │   Ordered    │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │                │                │
       └────────────────┼────────────────┘
                        ▼
              ┌──────────────────┐
              │     ITERATOR     │
              │      TRAIT       │
              └────────┬─────────┘
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│  Iterator    │ │    Custom    │ │     Lazy     │
│  Adapters    │ │   Iterator   │ │  Evaluation  │
│ (map, filter)│ │              │ │              │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │                │                │
       └────────────────┼────────────────┘
                        ▼
              ┌──────────────────┐
              │  collect()       │
              │  Materialize     │
              └──────────────────┘
```

**Pattern Flow**:
1. Choose collection: **Vec/HashMap/BTreeMap**
2. Process with **Iterator Adapters**
3. Implement **Custom Iterator** if needed
4. Exploit **Lazy Evaluation**
5. **Collect** results when needed

**Anti-Pattern Exits**:
- Eager Evaluation → Use lazy iterators
- Index Overuse → Use iterator methods
- Collect Too Early → Keep lazy

---

### Cluster 5: Async & Concurrency

```
                  ┌──────────────────┐
                  │  ASYNC CONTEXT   │
                  └────────┬─────────┘
                           │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │    Async     │ │    Future    │ │   Async      │
    │  Functions   │ │    Trait     │ │  Runtime     │
    │   async fn   │ │              │ │ tokio/async  │
    └──────┬───────┘ └──────┬───────┘ └──────┬───────┘
           │                │                │
           └────────────────┼────────────────┘
                            ▼
                  ┌──────────────────┐
                  │   CONCURRENT     │
                  │   OPERATIONS     │
                  │  (spawn/join)    │
                  └────────┬─────────┘
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│    Spawn     │   │    Actor     │   │   Custom     │
│  Blocking    │   │    Model     │   │  Executor    │
│ CPU-bound    │   │ Message Pass │   │  (Advanced)  │
└──────────────┘   └──────────────┘   └──────────────┘
```

**Pattern Flow**:
1. Define **Async Functions**
2. Understand **Future Trait**
3. Choose **Async Runtime**
4. Spawn **Concurrent Operations**
5. Handle blocking with **Spawn Blocking**
6. For complex state → **Actor Model**
7. Advanced: **Custom Executor**

**Anti-Pattern Exits**:
- Blocking in Async → Use spawn_blocking
- Over-Threading → Use async tasks
- Missing Send/Sync → Fix bounds

---

### Cluster 6: Macros & Metaprogramming

```
              ┌──────────────────────┐
              │   MACRO CONTEXT      │
              └──────────┬───────────┘
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ Macro Rules  │ │  Fragment    │ │   Recursive  │
│ macro_rules! │ │  Specifiers  │ │    Macro     │
│              │ │ (expr, ty)   │ │   (Nesting)  │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │                │                │
       └────────────────┼────────────────┘
                        ▼
              ┌──────────────────┐
              │  Macro Hygiene   │
              │  ($crate::)      │
              └────────┬─────────┘
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│     DSL      │ │ Procedural   │ │   Derive     │
│Construction  │ │   Macros     │ │   Macros     │
│  (json!)     │ │  (Advanced)  │ │  #[derive]   │
└──────────────┘ └──────────────┘ └──────────────┘
```

**Pattern Flow**:
1. Start with **Macro Rules**
2. Choose **Fragment Specifiers**
3. Add **Recursion** for nesting
4. Ensure **Hygiene**
5. Build **DSL** if appropriate
6. Advanced: **Procedural Macros**

**Anti-Pattern Exits**:
- Over-Engineering → Use functions
- tt Overuse → Use specific specifiers
- Missing $crate → Add absolute paths

---

### Cluster 7: Unsafe & FFI

```
              ┌──────────────────────┐
              │    UNSAFE NEED       │
              └──────────┬───────────┘
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│    Unsafe    │ │     Raw      │ │     FFI      │
│    Block     │ │   Pointers   │ │  Boundary    │
│   unsafe{}   │ │  *const T    │ │   extern     │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │                │                │
       └────────────────┼────────────────┘
                        ▼
              ┌──────────────────┐
              │     SAFETY       │
              │ Documentation    │
              │  // SAFETY:      │
              └────────┬─────────┘
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│     Safe     │ │  Minimal     │ │   Testing    │
│   Wrapper    │ │   Unsafe     │ │  (Miri)      │
│  Public API  │ │    Scope     │ │              │
└──────────────┘ └──────────────┘ └──────────────┘
```

**Pattern Flow**:
1. Identify **Unsafe Need**
2. Write **Unsafe Block**
3. Use **Raw Pointers** if needed
4. Define **FFI Boundary** for C
5. Document with **SAFETY**
6. Encapsulate with **Safe Wrapper**
7. **Minimize Scope**
8. **Test thoroughly**

**Anti-Pattern Exits**:
- Unsafe Everywhere → Minimize scope
- Missing Documentation → Add SAFETY
- Unsafe Public API → Add safe wrapper

---

## Horizontal Relationships (Cross-Cluster)

### Web Development Path

```
Async Functions ─────┐
                     ▼
Future Trait ────────┼──────▶ Request/Response Pattern
                     │                │
Concurrent Ops ──────┘                ▼
                             Route Handling
                                      │
                                      ▼
Error Handling ───────────────▶ Middleware Pattern
                                      │
Module Organization ──────────────────┘
                                      │
                                      ▼
                              Web Service Architecture
```

**Projects**: echo-server → http-get → actix-gcd

---

### CLI Development Path

```
Module Organization ─────┐
                         ▼
Error Handling ──────────┼──────▶ CLI Structure
                         │              │
Iterator Adapters ───────┘              ▼
                                Argument Parsing
                                        │
File I/O ────────────────────────────────┤
                                        │
Result & ? ──────────────────────────────┘
                                        │
                                        ▼
                                  CLI Application
```

**Projects**: grep, copy

---

### Type-Safe Library Path

```
Struct Definition ───────┐
                         ▼
Generic Types ───────────┼──────▶ Trait Implementation
                         │                │
From/Into Conversion ────┘                ▼
                                  Operator Overloading
                                          │
Iterator Implementation ──────────────────┤
                                          │
Public API Design ────────────────────────┘
                                          │
                                          ▼
                                  Library Component
```

**Projects**: queue → generic-queue → complex → binary-tree

---

## Vertical Progressions (Skill Levels)

### Beginner Path (Weeks 1-4)

```
Week 1: Foundation
├─ Ownership Transfer
├─ Borrowing
├─ Struct Definition
└─ Option Handling

Week 2: Types & Traits
├─ Enum Definition
├─ Trait Implementation
├─ From/Into
└─ Result Handling

Week 3: Collections & Iteration
├─ Vec & HashMap
├─ Iterator Adapters
├─ ? Operator
└─ Error Handling

Week 4: Organization
├─ Module Organization
├─ Public API Design
└─ Testing
```

**Projects**: gcd → queue → complex → echo-server

---

### Intermediate Path (Weeks 5-8)

```
Week 5: Generics & Advanced Types
├─ Generic Types
├─ Lifetime Annotations
├─ Newtype Pattern
└─ Custom Iterator

Week 6: Async Basics
├─ Async Functions
├─ Async Runtime
├─ Concurrent Operations
└─ Error in Async

Week 7: Macros Introduction
├─ Macro Rules
├─ Fragment Specifiers
├─ Recursive Macro
└─ Hygiene

Week 8: Integration
├─ Web Development Basics
├─ CLI Tool Design
└─ Library API Design
```

**Projects**: generic-queue → binary-tree → cheapo-request → json-macro

---

### Advanced Path (Weeks 9-12)

```
Week 9: Advanced Async
├─ Custom Executors
├─ Actor Model
├─ Spawn Blocking
└─ Performance

Week 10: Unsafe & FFI
├─ Unsafe Blocks
├─ Raw Pointers
├─ FFI Boundary
└─ Safe Wrapper

Week 11: Advanced Macros
├─ DSL Construction
├─ Procedural Macros
├─ Macro Testing
└─ Error Messages

Week 12: Architecture
├─ Large-Scale Organization
├─ Performance Optimization
├─ Production Patterns
└─ Testing Strategies
```

**Projects**: spawn-blocking → block-on → libgit2-rs → libgit2-rs-safe

---

## Pattern Dependencies (Prerequisite Graph)

```
                    [Ownership Transfer]
                            │
                            ▼
                      [Borrowing]
                            │
                ┌───────────┼───────────┐
                ▼           ▼           ▼
         [Lifetimes]    [RAII]    [Struct Def]
                                        │
                                        ▼
                                 [Generic Types]
                                        │
                            ┌───────────┼───────────┐
                            ▼           ▼           ▼
                    [Trait Impl]  [From/Into] [Iterator]
                            │           │           │
                            └───────────┼───────────┘
                                        ▼
                                [Public API Design]
```

**Dependency Rules**:
- Must learn **Ownership** before **Borrowing**
- Must learn **Borrowing** before **Lifetimes**
- Must learn **Struct/Enum** before **Generic Types**
- Must learn **Generic Types** before **Trait Implementation**
- Should learn **Traits** before **Operator Overloading**

---

## Pattern Composition (Common Combinations)

### Composition 1: Type-Safe Wrapper

```
[Struct Definition]
       +
[Newtype Pattern]
       +
[From/Into Conversion]
       +
[Trait Implementation]
       ↓
Type-Safe Wrapper API
```

**Example**: Wrapping primitive types for domain safety

---

### Composition 2: Async Service

```
[Async Functions]
       +
[Error Handling]
       +
[Concurrent Operations]
       +
[Actor Model]
       ↓
Web Service
```

**Example**: actix-gcd web application

---

### Composition 3: Safe FFI

```
[FFI Boundary]
       +
[Unsafe Blocks]
       +
[SAFETY Documentation]
       +
[Safe Wrapper]
       +
[Error Conversion]
       ↓
Safe C Library Binding
```

**Example**: libgit2-rs → libgit2-rs-safe

---

### Composition 4: Complete DSL

```
[Macro Rules]
       +
[Fragment Specifiers]
       +
[Recursive Macro]
       +
[Macro Hygiene]
       +
[From/Into Conversion]
       ↓
Domain-Specific Language
```

**Example**: json! macro

---

## Navigation Guide

### "I Want To Learn..."

**Ownership & Memory**:
```
Start → Ownership Transfer → Borrowing → Lifetimes → RAII
```

**Type System**:
```
Start → Struct/Enum → Generic Types → Traits → Operator Overloading
```

**Error Handling**:
```
Start → Option → Result → ? Operator → Custom Errors
```

**Async Programming**:
```
Start → Async Functions → Runtime → Concurrent Ops → Advanced
```

**Macros**:
```
Start → Macro Rules → Specifiers → Recursion → Hygiene → DSL
```

**Unsafe Code**:
```
Start → Unsafe Blocks → FFI → Safe Wrapper → Testing
```

---

### "I'm Building..."

**A Library**:
```
Module Org → Struct/Enum → Generics → Traits → API Design → Testing
```

**A Web Service**:
```
Async → Error Handling → Routing → Middleware → Actor Model
```

**A CLI Tool**:
```
Struct → Error Handling → Iterator → File I/O → Argument Parsing
```

**A DSL**:
```
Macro Rules → Specifiers → Recursion → Hygiene → Testing
```

**C Bindings**:
```
FFI → Unsafe → SAFETY Docs → Safe Wrapper → Error Conversion
```

---

## Pattern Language Principles

### Christopher Alexander's Principles Applied to Rust

1. **Each pattern resolves a force**: Every Rust pattern solves a specific tension (safety vs. performance, flexibility vs. simplicity)

2. **Patterns reference other patterns**: Like buildings reference doors and windows, Rust patterns reference each other

3. **Patterns form a language**: You can combine patterns to express complex designs

4. **Scale matters**: Small patterns (borrowing) combine into medium patterns (iterators) combine into large patterns (applications)

5. **Context determines choice**: The right pattern depends on your current situation

6. **Quality without a name**: Good Rust code has a certain feel—safe, ergonomic, efficient—that comes from proper pattern use

---

## The Journey Through the Pattern Space

### Beginner's Journey

```
Entry Point: Ownership Transfer
    ↓
Borrowing (most common need)
    ↓
Struct & Enum (building types)
    ↓
Traits (adding behavior)
    ↓
Option & Result (handling absence/errors)
    ↓
Iterator Adapters (processing data)
    ↓
Milestone: Can build basic programs
```

### Intermediate Journey

```
Starting Point: Generic Types
    ↓
Lifetimes (complex references)
    ↓
Custom Iterator (advanced patterns)
    ↓
Async Functions (concurrency)
    ↓
Macro Rules (code generation)
    ↓
Milestone: Can build libraries and services
```

### Advanced Journey

```
Starting Point: Unsafe Blocks
    ↓
FFI Boundaries (C interop)
    ↓
Safe Wrappers (encapsulation)
    ↓
Custom Executors (runtime internals)
    ↓
Procedural Macros (advanced metaprogramming)
    ↓
Milestone: Can build systems-level components
```

---

## Conclusion

This pattern map shows that Rust patterns form a coherent language:

- **Foundation patterns** (ownership, types) support everything else
- **Cluster patterns** (async, macros, unsafe) build on the foundation
- **Composition patterns** combine smaller patterns for complete solutions
- **Learning progressions** guide the journey from beginner to expert

Like Alexander's architectural patterns, these patterns:
- Have prerequisites (dependency graph)
- Combine naturally (composition patterns)
- Scale from small to large (nano to mega)
- Resolve forces (safety/performance/ergonomics)
- Form a complete language for expressing Rust designs

Use this map to:
1. **Find your starting point** based on current knowledge
2. **Navigate to your goal** following pattern sequences
3. **Understand relationships** between patterns
4. **Plan learning** following prerequisite chains
5. **Design systems** combining appropriate patterns

The pattern language is not a rigid prescription but a flexible toolkit. Learn the patterns, understand their relationships, and apply them thoughtfully to your specific context.

---

## Quick Reference: Pattern Locator

| Your Current Situation | Next Pattern to Learn | See Also |
|------------------------|----------------------|----------|
| Just starting Rust | Ownership Transfer | Foundation Layer |
| Understand ownership | Borrowing | Ownership Cluster |
| Can use references | Lifetimes | Ownership Cluster |
| Have basic types | Generic Types | Type System Cluster |
| Need behavior | Trait Implementation | Type System Cluster |
| Handling errors | Result & ? Operator | Error Handling Cluster |
| Processing sequences | Iterator Adapters | Collections Cluster |
| Need concurrency | Async Functions | Async Cluster |
| Building web service | Request/Response | Web Development Path |
| Creating CLI tool | CLI Structure | CLI Development Path |
| Need code generation | Macro Rules | Macro Cluster |
| Interfacing with C | FFI Boundary | Unsafe Cluster |
| Want compile-time DSL | DSL Construction | Macro Cluster |

---

## Pattern Language Version

**Version**: 1.0 (Based on Rust 2021 Edition)

**Coverage**:
- 40+ core patterns
- 7 major clusters
- 12+ pattern sequences
- 15+ anti-patterns
- Complete dependency graph

**Based on Projects**:
- gcd, actix-gcd (CLI & web basics)
- queue, generic-queue (type system)
- complex, interval (traits & operators)
- binary-tree (advanced types & iterators)
- fern_sim (module organization)
- basic-router (closures)
- grep, copy (CLI applications)
- echo-server, http-get (networking)
- cheapo-request, many-requests (async)
- spawn-blocking, block-on (async internals)
- json-macro (macros & DSL)
- ascii, ref-with-flag, gap-buffer (unsafe)
- libgit2-rs, libgit2-rs-safe (FFI)

This map evolves with the Rust language and community practices.
