# Concrete Container

**Also Known As:** Specific Type First, Monomorphic Container, Hardcoded Type

## Context

You're building a new data structure in Rust. You understand the algorithm conceptually—perhaps a queue, tree, or graph—but you're still learning how ownership, borrowing, and lifetimes work in practice. The temptation exists to immediately reach for generics to make your container "reusable," but you're uncertain about trait bounds, lifetime parameters, and where ownership should live.

## Problem

How do you learn Rust's ownership system while building a non-trivial data structure, without getting overwhelmed by the additional complexity of generic type parameters?

**The core tension:** Generics add cognitive load through type parameters, trait bounds, and lifetime annotations. When you're simultaneously wrestling with ownership rules, the compound complexity can be paralyzing.

## Forces

- **Learning curve**: Ownership is already difficult; adding generics multiplies the mental burden
- **Compile-time feedback**: Concrete types produce clearer error messages with specific types rather than abstract `T`
- **Iteration speed**: Faster compile times and simpler errors enable rapid experimentation
- **Future flexibility**: You know the structure should eventually work with multiple types
- **Production readiness**: Concrete types feel "toy-like" compared to generic libraries
- **Code duplication**: Fear of writing throwaway code that must be rewritten later

## Solution

**Start with a concrete type. Choose a simple, well-understood type like `char`, `i32`, or `String`. Build your entire data structure using this single type. Master the ownership semantics in this constrained context before generalizing.**

The queue example demonstrates this perfectly. The original implementation hardcodes `char`:

```rust
/// A first-in, first-out queue of characters.
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last.
    younger: Vec<char>  // younger elements, youngest last.
}

impl Queue {
    /// Push a character onto the back of a queue.
    pub fn push(&mut self, c: char) {
        self.younger.push(c);
    }

    /// Pop a character off the front of a queue.
    pub fn pop(&mut self) -> Option<char> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }

            // Bring the elements in younger over to older, and put them in
            // the promised order.
            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        self.older.pop()
    }
}
```

**Why this works:**

1. **Clear ownership semantics**: The `Vec<char>` owns the characters. No lifetime questions.
2. **Simple method signatures**: `&mut self` is obvious. No `impl<T> Queue<T>` noise.
3. **Concrete return types**: `Option<char>` is immediately comprehensible.
4. **Focused experimentation**: The two-vector algorithm is challenging enough without generic complexity.

The implementation can explore sophisticated patterns:

```rust
pub fn split(self) -> (Vec<char>, Vec<char>) {
    (self.older, self.younger)
}
```

This demonstrates **consuming self**—transferring ownership of the entire structure. With a concrete type, the implications are crystal clear: after calling `split`, the Queue is gone, moved into the returned tuple.

## Resulting Context

**Benefits:**
- **Accelerated learning**: You understand move semantics, borrowing, and mutable references in a specific context
- **Clear error messages**: When the compiler complains about `Vec<char>`, you see exactly what type is problematic
- **Working prototype**: You have a correct, tested implementation that demonstrates the algorithm
- **Refactoring target**: The concrete version becomes a specification for the generic version
- **Documentation**: The simple version serves as clear documentation of core behavior

**Liabilities:**
- **Limited reusability**: The queue only works with `char`, not `String`, `i32`, or custom types
- **Code duplication**: If you need multiple types, you might be tempted to copy-paste
- **Appears incomplete**: Other developers might view it as a toy or work-in-progress
- **Refactoring cost**: Converting to generic requires modifying all method signatures

**What's now possible:**
- You can confidently explain why `pop` takes `&mut self` (it modifies the queue)
- You understand why `split` takes `self` (it consumes the queue)
- You've internalized that `Vec<char>` automatically cleans up when dropped
- You're ready to tackle [Generic Container](./generic-container.md)

## Related Patterns

- [Generic Container](./generic-container.md) - The natural evolution: parameterize the concrete type
- [Builder Methods](./builder-methods.md) - Techniques like `split(self)` that consume and transform
- Interior Iteration - How concrete types simplify iterator implementation

## Known Uses

**In this codebase:**
- `/home/user/rust-programming-examples/queue/src/lib.rs` - The canonical example of this pattern
- The concrete `Queue` serves as a reference implementation before `generic-queue`

**In the wild:**
- Rust book examples frequently start concrete (e.g., `Vec<i32>` before `Vec<T>`)
- Many Rust tutorials teach ownership with `String` before introducing `Vec<T>`
- Protocol buffer generated code often starts with concrete types for specific message schemas
- Learning-focused codebases intentionally use concrete types for clarity

**Why it matters:**
Christopher Alexander wrote: "Each pattern describes a problem which occurs over and over again in our environment, and then describes the core of the solution to that problem, in such a way that you can use this solution a million times over, without ever doing it the same way twice."

In Rust, this pattern appears whenever someone is learning ownership. The concrete type is not a limitation—it's a pedagogical tool. Once mastered, the pattern naturally evolves into generic code, but the fundamental understanding gained from the concrete version remains invaluable.

The progression from `Queue` (concrete) to `Queue<T>` (generic) mirrors the way expertise develops: master the specific before abstracting to the general.
