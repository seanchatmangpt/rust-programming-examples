# Team and Organization Patterns

## Learning Objectives

By the end of this chapter, you will:
- Implement effective code review practices for Rust
- Design testing strategies that scale with team size
- Create documentation that serves as architecture
- Establish knowledge transfer processes
- Organize distributed Rust teams effectively

## Introduction

As Rust adoption has grown (2021-2026), organizational patterns have emerged for teams building production systems. This chapter explores how teams work effectively with Rust, from code review to knowledge sharing, drawing lessons from companies running Rust at scale and applying them to the architectural patterns in this repository.

## Code Review Practices for Rust

### The Rust Review Checklist

Code review in Rust is different from other languages due to the type system's guarantees. Focus areas shift from "is this memory-safe?" to higher-level concerns.

#### Level 1: Automatic Checks (CI Enforced)

```yaml
# .github/workflows/pr.yml
name: Pull Request

on: [pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      # 1. Format check (non-negotiable)
      - run: cargo fmt --all -- --check

      # 2. Clippy lints (catch common mistakes)
      - run: cargo clippy --all-targets --all-features -- -D warnings

      # 3. Tests must pass
      - run: cargo test --all-features

      # 4. Documentation builds
      - run: cargo doc --no-deps --all-features

      # 5. Security audit
      - run: cargo install cargo-audit && cargo audit

      # 6. Semver compatibility (for libraries)
      - run: cargo install cargo-semver-checks && cargo semver-checks
```

**If CI passes, reviewers skip these mechanical checks entirely.**

#### Level 2: Human Review Focus Areas

```rust
// Review focus: API design, not syntax
// From binary-tree project review

// ❌ COMMENT: API allows invalid state
pub struct BinaryTree<T> {
    pub value: T,  // Public fields - can create invalid tree
    pub left: Option<Box<BinaryTree<T>>>,
    pub right: Option<Box<BinaryTree<T>>>,
}

// ✅ APPROVED: Encapsulated, maintains invariants
pub struct BinaryTree<T> {
    value: T,
    left: Option<Box<BinaryTree<T>>>,
    right: Option<Box<BinaryTree<T>>>,
}

impl<T: Ord> BinaryTree<T> {
    pub fn insert(&mut self, value: T) {
        // Maintains BST invariant
    }
}
```

**Review checklist for humans**:

1. **API Design**
   - [ ] Functions have clear, single responsibilities
   - [ ] Types prevent invalid states
   - [ ] Naming follows Rust conventions
   - [ ] Public API is minimal (least privilege principle)

2. **Error Handling**
   - [ ] Errors use `Result`, not panics (except for unrecoverable)
   - [ ] Error types are descriptive (not just `String`)
   - [ ] `unwrap()` justified in comments (e.g., "invariant guaranteed")
   - [ ] `?` operator used consistently

3. **Performance Considerations**
   - [ ] Unnecessary allocations avoided
   - [ ] Clones justified (can't borrow?)
   - [ ] Appropriate data structures (Vec vs. HashMap vs. BTreeMap)
   - [ ] Hot paths identified and optimized

4. **Safety (Unsafe Code)**
   - [ ] `unsafe` blocks minimized
   - [ ] SAFETY comments explain invariants
   - [ ] Unsafe code isolated in dedicated functions
   - [ ] Tests verify safety invariants

5. **Testing**
   - [ ] Happy path tested
   - [ ] Edge cases covered
   - [ ] Error cases tested
   - [ ] Public API has doc tests

6. **Documentation**
   - [ ] Public items have doc comments
   - [ ] Examples in doc comments
   - [ ] Panics documented
   - [ ] Safety requirements documented (if unsafe)

### Review Comments: Effective Communication

**Poor comment**:
```rust
// This is wrong.
fn process(data: &[u8]) -> Vec<u8> {
    data.to_vec()  // Unnecessary clone
}
```

**Effective comment**:
```rust
// Suggestion: Avoid cloning if possible
//
// Can we return &[u8] instead of Vec<u8>? Cloning large
// slices impacts performance. If ownership transfer is necessary,
// consider taking `data: Vec<u8>` as parameter.
//
// Context: This function is called in hot path (10k+ times/sec).
fn process(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}
```

**Components of effective review**:
1. **Tag**: "Suggestion", "Question", "Blocker", "Nit"
2. **Observation**: What you noticed
3. **Rationale**: Why it matters
4. **Alternative**: Proposed solution
5. **Context**: Additional information

### Pairing Reviews with Architecture

From `gap-buffer` example—unsafe code requires rigorous review:

```rust
// Author provides context in PR description:
//
// ## Changes
// Optimized `insert()` to use unsafe pointer manipulation.
//
// ## Safety Rationale
// 1. `gap_start < gap_end` verified before unsafe block
// 2. `storage.len() >= gap_end` maintained as invariant
// 3. Property tests verify no corruption after 10k operations
//
// ## Performance Impact
// Before: 200ns per insert
// After:   50ns per insert
// 4× speedup critical for editor responsiveness.

impl<T> GapBuffer<T> {
    pub fn insert(&mut self, value: T) {
        assert!(self.gap_start < self.gap_end);

        // SAFETY: gap_start < gap_end verified above.
        // storage has capacity >= gap_end (invariant).
        unsafe {
            *self.storage.get_unchecked_mut(self.gap_start) = value;
        }
        self.gap_start += 1;
    }
}
```

**Reviewer response**:
```
Approved with suggestions:

1. ✅ Safety rationale is clear
2. ✅ Performance benefit justified
3. ⚠️ Suggestion: Add `#[cfg(debug_assertions)]` variant that checks
   invariants in debug builds:

   ```rust
   #[cfg(debug_assertions)]
   {
       assert!(self.gap_start < self.gap_end);
       assert!(self.storage.len() >= self.gap_end);
   }
   ```

4. ℹ️ Note: Consider MIRI testing in CI for unsafe code validation.
```

## Testing Strategies at Scale

### The Testing Pyramid for Rust

```
           E2E Tests (10%)
         ┌───────────────┐
         │   Expensive   │
         │   Slow setup  │
         └───────────────┘

      Integration Tests (20%)
    ┌────────────────────────┐
    │  Test module boundaries │
    │  Real dependencies      │
    └────────────────────────┘

         Unit Tests (70%)
   ┌──────────────────────────────┐
   │  Fast, isolated, exhaustive  │
   │  Mock external dependencies  │
   └──────────────────────────────┘
```

### Unit Testing Pattern

From `queue` project:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue_is_empty() {
        let queue: Queue<i32> = Queue::new();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }

    #[test]
    fn push_increases_len() {
        let mut queue = Queue::new();
        queue.push(1);
        assert_eq!(queue.len(), 1);
        queue.push(2);
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn pop_decreases_len() {
        let mut queue = Queue::new();
        queue.push(1);
        queue.push(2);

        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn pop_empty_returns_none() {
        let mut queue: Queue<i32> = Queue::new();
        assert_eq!(queue.pop(), None);
    }

    // Property-based test (2026 standard)
    #[cfg(test)]
    mod properties {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn push_then_pop_preserves_order(
                values in prop::collection::vec(0..100i32, 0..1000)
            ) {
                let mut queue = Queue::new();
                for &v in &values {
                    queue.push(v);
                }

                let mut results = Vec::new();
                while let Some(v) = queue.pop() {
                    results.push(v);
                }

                assert_eq!(results, values);
            }

            #[test]
            fn len_matches_pushes_minus_pops(
                pushes in prop::collection::vec(0..100i32, 0..100),
                num_pops in 0..100usize,
            ) {
                let mut queue = Queue::new();
                for v in pushes.iter() {
                    queue.push(*v);
                }

                let pops = num_pops.min(queue.len());
                for _ in 0..pops {
                    queue.pop();
                }

                assert_eq!(queue.len(), pushes.len() - pops);
            }
        }
    }
}
```

### Integration Testing Pattern

From `actix-gcd` project:

```rust
// tests/http_api.rs
use actix_web::{test, App};
use actix_gcd::{create_app, AppState};

mod common;  // Shared test utilities

#[actix_web::test]
async fn test_gcd_endpoint_success() {
    let app = test::init_service(create_app()).await;

    let req = test::TestRequest::get()
        .uri("/gcd?n=48&m=18")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: GcdResponse = test::read_body_json(resp).await;
    assert_eq!(body.result, 6);
}

#[actix_web::test]
async fn test_gcd_missing_params() {
    let app = test::init_service(create_app()).await;

    let req = test::TestRequest::get()
        .uri("/gcd?n=48")  // Missing m parameter
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 400);  // Bad Request
}

// Load test (2026 practice: test performance in CI)
#[actix_web::test]
#[ignore]  // Only run with --ignored flag
async fn test_gcd_under_load() {
    let app = test::init_service(create_app()).await;

    let start = Instant::now();
    let mut futures = Vec::new();

    for i in 0..1000 {
        let req = test::TestRequest::get()
            .uri(&format!("/gcd?n={}&m={}", i * 2, i * 3))
            .to_request();

        futures.push(test::call_service(&app, req.clone()));
    }

    let results = futures::future::join_all(futures).await;

    let elapsed = start.elapsed();
    let success_count = results.iter().filter(|r| r.status().is_success()).count();

    assert_eq!(success_count, 1000);
    assert!(
        elapsed < Duration::from_secs(5),
        "Load test took too long: {:?}",
        elapsed
    );
}
```

### Test Organization at Scale

For a repository with 24 projects:

```
workspace/
├── Cargo.toml                 # Workspace root
├── members/
│   ├── gcd/
│   │   ├── src/
│   │   │   └── lib.rs        # Unit tests inline
│   │   └── tests/
│   │       └── integration.rs
│   └── actix-gcd/
│       ├── src/
│       ├── tests/
│       │   ├── common/       # Shared test utilities
│       │   │   └── mod.rs
│       │   ├── http_tests.rs
│       │   └── load_tests.rs
│       └── benches/
│           └── bench.rs
└── tests/                     # Workspace-level tests
    └── cross_crate_tests.rs
```

**Workspace test commands**:
```bash
# Run all tests
cargo test --workspace

# Run specific crate's tests
cargo test -p actix-gcd

# Run integration tests only
cargo test --test '*'

# Run benchmarks
cargo bench --workspace
```

## Documentation as Architecture

In Rust, documentation is executable code. Treat it as first-class architecture artifact.

### Documentation Layers

#### Layer 1: Inline Comments

```rust
// From gap-buffer
impl<T> GapBuffer<T> {
    /// Inserts a value at the current cursor position.
    ///
    /// This is an O(1) operation as long as the gap is at the cursor.
    /// If the gap needs to be moved, it becomes O(n) where n is the
    /// distance to move the gap.
    ///
    /// # Examples
    ///
    /// ```
    /// use gap_buffer::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::new();
    /// buffer.insert('H');
    /// buffer.insert('i');
    /// assert_eq!(buffer.to_string(), "Hi");
    /// ```
    pub fn insert(&mut self, value: T) {
        // Implementation
    }
}
```

#### Layer 2: Module Documentation

```rust
//! # Binary Tree Module
//!
//! This module provides a simple binary search tree implementation
//! for educational purposes.
//!
//! ## Design Decisions
//!
//! - Uses `Box` for heap allocation (simplicity over performance)
//! - Does not self-balance (teaching insertion/traversal basics)
//! - Provides in-order iterator (demonstrates Iterator trait)
//!
//! ## Performance Characteristics
//!
//! | Operation | Average | Worst Case |
//! |-----------|---------|------------|
//! | Insert    | O(log n)| O(n)       |
//! | Search    | O(log n)| O(n)       |
//! | Traversal | O(n)    | O(n)       |
//!
//! ## Example
//!
//! ```
//! use binary_tree::BinaryTree;
//!
//! let mut tree = BinaryTree::new(10);
//! tree.insert(5);
//! tree.insert(15);
//!
//! let values: Vec<_> = tree.iter().copied().collect();
//! assert_eq!(values, vec![5, 10, 15]);
//! ```

pub struct BinaryTree<T> {
    // ...
}
```

#### Layer 3: Architecture Decision Records (ADRs)

```markdown
<!-- docs/adr/0001-gap-buffer-unsafe.md -->
# ADR 0001: Use Unsafe Pointer Operations in Gap Buffer

## Status
Accepted (2026-01-15)

## Context
Gap buffer insert operations are on the critical path for text editing.
Initial safe implementation averaged 200ns per character, causing
perceptible lag during fast typing.

## Decision
Use unsafe pointer operations for gap manipulation, isolated in
dedicated methods with comprehensive safety documentation.

## Consequences

### Positive
- Insert latency reduced to 50ns (4× improvement)
- User experience: no perceptible lag during typing

### Negative
- Requires careful code review for unsafe blocks
- Must maintain safety invariants (gap_start < gap_end, etc.)
- Added complexity in testing (property tests, MIRI integration)

### Mitigations
- Extensive property-based testing (10k+ operations)
- Debug assertions check invariants in debug builds
- MIRI testing in CI catches undefined behavior
- Safety rationale documented in SAFETY comments
```

### Documentation Maintenance

**2026 best practice**: Documentation tests in CI prevent drift.

```rust
/// Computes GCD using Euclid's algorithm.
///
/// # Examples
///
/// ```
/// use gcd::gcd;
///
/// assert_eq!(gcd(48, 18), 6);
/// assert_eq!(gcd(0, 5), 5);  // Edge case
/// ```
///
/// # Panics
///
/// Never panics. Handles zero inputs gracefully.
pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    // If example is wrong, `cargo test` fails
}
```

## Knowledge Transfer in Teams

### Onboarding New Rust Developers

**Week 1: Foundations**
- Read chapters 1-6 of Programming Rust
- Complete rustlings exercises
- Review `gcd`, `queue`, `complex` projects (simple examples)

**Week 2: Intermediate Concepts**
- Read chapters 11-15 (traits, iterators, closures)
- Review `binary-tree`, `grep`, `actix-gcd` (intermediate patterns)
- Pair program on small feature

**Week 3: Advanced Topics**
- Read chapters 18-22 (concurrency, async, unsafe)
- Review `spawn-blocking`, `gap-buffer`, `json-macro` (advanced patterns)
- Own a small feature end-to-end

**Week 4: Production Readiness**
- Read this chapter (Chapter 11)
- Participate in code review
- Deploy a service to staging

### Pair Programming Patterns

**Driver-Navigator for Rust**:
- **Navigator** focuses on design, ownership, lifetimes
- **Driver** focuses on syntax, compiler errors
- Switch every 25 minutes (Pomodoro)

**Effective for**:
- Lifetime puzzle resolution
- Unsafe code review
- Architecture decisions

### Internal Tech Talks

**Monthly topics** (2026 pattern at Rust shops):
1. "Trait Deep Dive: When to use associated types vs. generics"
2. "Async Patterns: Circuit breakers and timeouts"
3. "Performance Optimization: Profiling and benchmarking"
4. "Production War Stories: Debugging hard Rust bugs"

**Format**:
- 30 minutes presentation
- 15 minutes Q&A
- Code examples from production
- Recorded for asynchronous viewing

## Rust for Distributed Teams

### Asynchronous Collaboration

**2026 tools**:
- **GitHub Discussions**: Design proposals, RFCs
- **Async code review**: Detailed written feedback (not synchronous calls)
- **Documentation-first**: Write design doc before code
- **Recorded demos**: Loom videos showing feature in action

### Decision-Making Process

**RFC (Request for Comments) Process**:

```markdown
<!-- rfcs/0042-circuit-breaker.md -->
# RFC 0042: Circuit Breaker for HTTP Client

## Summary
Add circuit breaker pattern to http-get client to prevent
cascading failures.

## Motivation
Production incidents show HTTP client amplifies failures in
downstream services, causing cascades.

## Design
(See Chapter 11.3 Circuit Breaker implementation)

## Alternatives Considered
1. Simple retry with exponential backoff (insufficient)
2. Client-side rate limiting (doesn't prevent cascades)

## Open Questions
- Failure threshold: 5 or 10 failures?
- Timeout duration: 30s or 60s?

## Decision
(Filled in after discussion)
```

**Process**:
1. Author creates RFC PR
2. Team reviews asynchronously (3-5 days)
3. Discussion in PR comments
4. Author updates RFC based on feedback
5. Decision: Approve, Reject, or Needs Changes
6. Implement if approved

## Summary

Effective teams building Rust systems:
- **Code review**: Focus on design, not syntax (CI checks syntax)
- **Testing**: Unit tests (70%), integration tests (20%), E2E (10%)
- **Documentation**: Executable, maintained, architecture-preserving
- **Knowledge transfer**: Structured onboarding, pair programming, tech talks
- **Distributed work**: Async collaboration, RFC process, documentation-first

The patterns in this repository—from simple `gcd` to complex `json-macro`—serve as teaching tools for teams. Use them for onboarding, reference in code reviews, and examples in design discussions.

## Further Reading

- Chapter 11.5: Looking ahead to future team practices
- Chapter 11.3: Resilient systems require team coordination
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
