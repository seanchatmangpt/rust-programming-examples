# Systems Architecture Patterns in Rust: 2026 Edition

[Introduction](00-introduction.md)

---

# Part I: Core Architectural Foundations

- [Chapter 1: Architecture as Applied Type Theory](./chapter-01-architecture.md)
  - The architectural mindset and 2022-2026 evolution
  - Type system as first-class architectural tool

- [Chapter 2: Ownership-Based Architecture](./chapter-02-ownership/00-overview.md)
  - [2.1 Ownership as Architectural Constraint](./chapter-02-ownership/01-ownership-as-constraint.md)
  - [2.2 Designing with Move Semantics](./chapter-02-ownership/02-move-semantics.md)
  - [2.3 Borrowing as Architectural Interface](./chapter-02-ownership/03-borrowing-as-interface.md)
  - [2.4 Lifetimes in Large Systems](./chapter-02-ownership/04-lifetimes-in-systems.md)
  - [2.5 Case Study: Queue-Based Systems](./chapter-02-ownership/05-case-study-queues.md)

- [Chapter 3: Trait-Based Design for Flexibility](./chapter-03-traits/00-overview.md)
  - [3.1 Traits as Architectural Interfaces](./chapter-03-traits/01-traits-as-interfaces.md)
  - [3.2 Composition Patterns with Traits](./chapter-03-traits/02-composition-patterns.md)
  - [3.3 Polymorphism Strategies](./chapter-03-traits/03-polymorphism-strategies.md)
  - [3.4 Error Handling as Trait Architecture](./chapter-03-traits/04-error-handling-traits.md)
  - [3.5 Case Studies: Operator Overloading](./chapter-03-traits/05-case-studies-operators.md)

- [Chapter 4: Module Organization and API Design](./chapter-04-modules/00-overview.md)
  - [4.1 Layered Architecture with Modules](./chapter-04-modules/01-layered-architecture.md)
  - [4.2 API Surface Management](./chapter-04-modules/02-api-surface-design.md)
  - [4.3 Large Crate Architecture](./chapter-04-modules/03-large-crate-architecture.md)
  - [4.4 Visibility Patterns](./chapter-04-modules/04-visibility-patterns.md)
  - [4.5 Case Study: Fern Simulator](./chapter-04-modules/05-case-study-fern-sim.md)

---

# Part II: Core Concerns

- [Chapter 5: Error Handling as Architectural Foundation](./chapter-05-errors/00-overview.md)
  - [5.1 Result Types as Control Flow](./chapter-05-errors/01-result-types-control-flow.md)
  - [5.2 Custom Error Types](./chapter-05-errors/02-custom-error-types.md)
  - [5.3 Error Handling Patterns](./chapter-05-errors/03-error-handling-patterns.md)
  - [5.4 Architectural Implications](./chapter-05-errors/04-architectural-implications.md)
  - [5.5 Case Study: HTTP Client Architecture](./chapter-05-errors/05-case-study-http-client.md)

- [Chapter 6: Async/Concurrency Architecture](./chapter-06-async/00-overview.md)
  - [6.1 Async as Architectural Primitive](./chapter-06-async/01-async-as-primitive.md)
  - [6.2 Designing Async Systems](./chapter-06-async/02-designing-async-systems.md)
  - [6.3 Async Trait Patterns](./chapter-06-async/03-async-traits.md)
  - [6.4 Concurrent System Design](./chapter-06-async/04-concurrent-design.md)
  - [6.5 Real-World Async Architectures](./chapter-06-async/05-real-world-async.md)

- [Chapter 7: Type-Driven Architecture](./chapter-07-types/00-overview.md)
  - [7.1 The Newtype Pattern](./chapter-07-types/01-newtype-pattern.md)
  - [7.2 Phantom Types and Compile-Time Constraints](./chapter-07-types/02-phantom-types.md)
  - [7.3 Generic Programming for Flexibility](./chapter-07-types/03-generic-programming.md)
  - [7.4 Type-Driven Testing](./chapter-07-types/04-type-driven-testing.md)
  - [7.5 Case Study: ASCII and Custom Types](./chapter-07-types/05-case-studies-ascii.md)

---

# Part III: Advanced Techniques

- [Chapter 8: FFI and Unsafe Boundaries](./chapter-08-unsafe/00-overview.md)
  - [8.1 FFI as Architectural Boundary](./chapter-08-unsafe/01-ffi-as-boundary.md)
  - [8.2 Building Safe Abstractions Over Unsafe Code](./chapter-08-unsafe/02-safe-abstractions.md)
  - [8.3 Unsafe Code Patterns](./chapter-08-unsafe/03-unsafe-code-patterns.md)
  - [8.4 Pointer Arithmetic and Memory Manipulation](./chapter-08-unsafe/04-pointer-arithmetic.md)
  - [8.5 Case Study: libgit2 Integration](./chapter-08-unsafe/05-case-study-libgit2.md)

- [Chapter 9: Real-World Architectures: Complete Case Studies](./chapter-09-case-studies/00-overview.md)
  - [9.1 Designing a Web Service](./chapter-09-case-studies/01-web-service-architecture.md)
  - [9.2 Building a Simulation System](./chapter-09-case-studies/02-simulation-systems.md)
  - [9.3 Text Processing Architecture](./chapter-09-case-studies/03-text-processing.md)
  - [9.4 Networking and Concurrency](./chapter-09-case-studies/04-networking-concurrency.md)
  - [9.5 Integration Across Systems](./chapter-09-case-studies/05-integration-across-layers.md)

---

# Part IV: Mastery and Future Directions

- [Chapter 10: Advanced Patterns and Performance](./chapter-10-advanced/00-overview.md)
  - [10.1 Advanced Generic Patterns](./chapter-10-advanced/01-advanced-generics.md)
  - [10.2 Performance-Critical Architecture](./chapter-10-advanced/02-performance-critical.md)
  - [10.3 Advanced Trait Patterns](./chapter-10-advanced/03-advanced-traits.md)
  - [10.4 Design Trade-Offs](./chapter-10-advanced/04-design-trade-offs.md)
  - [10.5 Macro-Driven Architecture](./chapter-10-advanced/05-macro-driven.md)

- [Chapter 11: 2026 Best Practices and Future Directions](./chapter-11-future/00-overview.md)
  - [11.1 Modern Rust Ecosystem (2026)](./chapter-11-future/01-modern-ecosystem.md)
  - [11.2 Evolution of Architecture Patterns](./chapter-11-future/02-pattern-evolution.md)
  - [11.3 Building Resilient Systems](./chapter-11-future/03-resilient-systems.md)
  - [11.4 Team and Organization](./chapter-11-future/04-team-organization.md)
  - [11.5 Looking Ahead](./chapter-11-future/05-looking-ahead.md)

---

# Appendices

- [Appendix A: 24 Projects Quick Reference](./appendix-a-projects.md)
- [Appendix B: Pattern Catalog](./appendix-b-patterns.md)
- [Appendix C: 2022-2026 Ecosystem Evolution](./appendix-c-evolution.md)
