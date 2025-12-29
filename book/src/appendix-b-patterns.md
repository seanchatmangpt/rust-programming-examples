# Appendix B: Pattern Catalog

Quick reference to all architectural patterns discussed in this book.

## Patterns by Category

### Ownership & Resource Management
- Single-Owner Pattern
- Move Semantics Pattern
- Borrowing Pattern
- RAII Pattern
- Lifetime Boundary Pattern

### Type System
- Newtype Pattern
- Type-State Pattern
- Phantom Type Pattern
- Generic Parameter Pattern
- Associated Type Pattern

### Traits & Polymorphism
- Trait-Based Interface Pattern
- Static Dispatch Pattern
- Dynamic Dispatch Pattern
- Blanket Implementation Pattern
- Sealed Trait Pattern

### Modules & Organization
- Layered Architecture Pattern
- Module Hierarchy Pattern
- Visibility Control Pattern
- Prelude Module Pattern
- API Surface Pattern

### Error Handling
- Result Type Pattern
- Error Enum Pattern
- Error Conversion Pattern
- Error Context Pattern
- Error Recovery Pattern

### Async & Concurrency
- Async Function Pattern
- Future Composition Pattern
- Message Passing Pattern
- Shared State Pattern
- Backpressure Pattern

### Unsafe & FFI
- SAFETY Comment Pattern
- Invariant Maintenance Pattern
- Safe Wrapper Pattern
- Raw Pointer Pattern
- Pointer Arithmetic Pattern

## Pattern Relationships

Patterns often build on each other. For example:
- Type-State Pattern depends on Type System
- Safe Wrapper Pattern depends on SAFETY Comment Pattern
- Error Conversion Pattern depends on Result Type Pattern

Refer to each chapter for detailed examples and relationships.
