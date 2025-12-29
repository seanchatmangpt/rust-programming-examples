# 50. TEST FUNCTION WITH ATTRIBUTE

*A module of production code stands complete, its functions carefully crafted. But how do you know they work correctly? How do you ensure they continue working as the code evolves?*

...within a **SOFTWARE VERIFICATION** context, when you need to validate that code behaves as expected...

◆ ◆ ◆

**How do you write executable specifications that verify code correctness without cluttering production binaries?**

Code without tests is code that only works by accident. Every function embodies assumptions about its inputs, outputs, and behavior. These assumptions live in the programmer's mind, fragile and easily forgotten. As code evolves, assumptions change, and what once worked may silently break.

Traditional testing requires separate test files, complex harness setup, and disconnection between test and implementation. Tests become an afterthought, skipped under time pressure, falling out of sync with the code they verify.

Rust integrates testing directly into the language. The `#[test]` attribute marks a function as a test that cargo will discover and run. These test functions live alongside the code they verify—often in the same file, in a nested `tests` module. This proximity makes tests easy to write, easy to maintain, and impossible to ignore.

The queue implementation demonstrates this pattern perfectly. Each method has a dedicated test function immediately following it: `test_push_pop` validates pushing and popping, `test_is_empty` validates emptiness checking, `test_split` validates splitting the queue. When someone reads the code, the tests serve as executable documentation, showing exactly how the API should be used.

Test functions use assertion macros—`assert!`, `assert_eq!`, `assert_ne!`—to verify expected behavior. When assertions fail, tests fail, and cargo reports exactly which test broke and why. In development, tests run with every change. In CI/CD, tests gate deployments.

**Therefore:**

**Mark verification functions with `#[test]` and use assertions to check expected behavior, creating executable specifications that live with your code.**

```rust
// From queue/src/lib.rs - test functions verify each method

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

#[test]
fn test_is_empty() {
    let mut q = Queue { older: Vec::new(), younger: Vec::new() };

    assert!(q.is_empty());
    q.push('☉');
    assert!(!q.is_empty());
    q.pop();
    assert!(q.is_empty());
}

#[test]
fn test_split() {
    let mut q = Queue { older: Vec::new(), younger: Vec::new() };

    q.push('P');
    q.push('D');
    assert_eq!(q.pop(), Some('P'));
    q.push('X');

    let (older, younger) = q.split();
    assert_eq!(older, vec!['D']);
    assert_eq!(younger, vec!['X']);
}

// From binary-tree/src/lib.rs - tests build structures and verify

#[test]
fn test_add_method_1() {
    let planets = vec!["Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Uranus"];
    let mut tree = BinaryTree::Empty;
    for planet in planets {
        tree.add(planet);
    }

    assert_eq!(tree.walk(),
               vec!["Jupiter", "Mars", "Mercury", "Saturn", "Uranus", "Venus"]);
}
```

*The test function stands guard over correctness: silent when the code works as intended, crying out immediately when behavior diverges from expectation—a tireless sentinel that never forgets to check.*

◆ ◆ ◆

Organize tests in a `#[cfg(test)] mod tests` block to exclude from release builds; use **ASSERT MACRO IN FUNCTION BODY** (47) for invariants; write **INTEGRATION TESTS** in the `tests/` directory for cross-module verification; use `#[should_panic]` for functions expected to panic under certain conditions.
