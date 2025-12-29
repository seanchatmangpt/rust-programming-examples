# 15. TEST MODULE WITH USE SUPER STAR

*A test module using #[cfg(test)] and use super::* to write tests that access the parent module's private items, living alongside the code it tests*

...within a [FLAT MODULE ALL IN LIB](#14) or any module file, when you need to test both public and private items...

◆ ◆ ◆

**How do you write comprehensive tests that can access private implementation details without making everything public?**

Tests need to verify invariants that cross public API boundaries. A `Queue` should maintain FIFO ordering even after internal reorganization between `older` and `younger` vectors. These vectors are private—external users shouldn't depend on them—but tests must verify they're managed correctly.

If you put tests in `tests/` (integration tests), they can only access public APIs. To test private items, you'd need to make them `pub`, breaking encapsulation. If you put tests in separate files within `src/`, they become sibling modules, still unable to access privates.

The solution: `#[cfg(test)] mod tests` creates a *child module* that only compiles during testing. Because it's a child, it can access parent private items through `use super::*`. The wildcard import (`*`) brings everything from the parent into scope—public and private—making tests read naturally without qualification.

The `queue` crate demonstrates this perfectly. Tests access `Queue` internals (`older`, `younger` vectors) to verify state transitions after `push` and `pop`, but external users never see these details.

**Therefore:**

**Create a child module annotated with #[cfg(test)], import everything from the parent with use super::*, and write tests that can freely access both public and private items.**

```rust
// In queue/src/lib.rs

pub struct Queue {
    older: Vec<char>,   // Private implementation detail
    younger: Vec<char>  // Private implementation detail
}

impl Queue {
    pub fn push(&mut self, c: char) {
        self.younger.push(c);
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }
            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }
        self.older.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;  // Brings Queue and all private items into scope

    #[test]
    fn test_push_pop() {
        let mut q = Queue { older: Vec::new(), younger: Vec::new() };

        q.push('0');
        q.push('1');
        assert_eq!(q.pop(), Some('0'));

        q.push('∞');
        assert_eq!(q.pop(), Some('1'));
        assert_eq!(q.pop(), Some('∞'));
        assert_eq!(q.pop(), None);
    }
}
```

*The diagram shows a parent module with public and private items, with a nested #[cfg(test)] mod tests box inside it, connected by an arrow labeled "use super::*" showing access flowing from parent to child.*

◆ ◆ ◆

This pattern complements [FLAT MODULE ALL IN LIB](#14) and contrasts with [INTEGRATION TESTS IN TESTS DIR](#integration). It enables [WHITE BOX TESTING](#testing-internals) while maintaining [ENCAPSULATION](#privacy).
