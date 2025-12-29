# 49. CLONE TO EXTEND LIFETIME

*A function borrows a value, processes it, but needs to return something that outlives the borrow—a transformed copy that can survive after the original goes away.*

...within a **OWNERSHIP TRANSFORMATION** context, when borrowed data must become owned data...

◆ ◆ ◆

**How do you create owned data from a borrowed reference when the borrow cannot be extended?**

Rust's borrow checker ensures references never outlive the data they point to. This safety guarantee creates a challenge: when a function borrows data and needs to return something derived from it, the return value cannot contain the original reference—it would outlive the borrow.

Consider the binary tree's walk method, which performs an in-order traversal and returns a vector of all elements. The method borrows `&self`, so it has only temporary access to the tree's nodes. Each element inside the tree is owned by a `TreeNode`, and those nodes remain in the tree. But the returned `Vec<T>` must outlive the method call—callers expect to keep the vector after the tree reference expires.

The method cannot return references to the tree's elements because those references would become invalid when the borrow ends. It cannot move elements out of the tree because that would leave the tree in an invalid state, violating the shared borrow.

The only solution is to copy each element, creating new owned instances that can safely outlive the tree. This is exactly what the `Clone` trait provides: the ability to create a new owned value from a reference.

There is a trade-off: cloning has cost. For small types like integers or short strings, the cost is negligible. For large structures, it may be significant. But when you need owned data and have only a borrow, cloning is the correct, safe choice—Rust makes the cost explicit so you can make informed decisions.

**Therefore:**

**Call `.clone()` on a reference to create an owned copy when the borrowed value cannot be returned directly but must outlive the borrow.**

```rust
// From binary-tree/src/lib.rs - clone elements during tree walk
impl<T: Clone> BinaryTree<T> {
    fn walk(&self) -> Vec<T> {
        match *self {
            BinaryTree::Empty => vec![],
            BinaryTree::NonEmpty(ref boxed) => {
                let mut result = boxed.left.walk();
                result.push(boxed.element.clone());  // Clone to extend lifetime
                result.extend(boxed.right.walk());
                result
            }
        }
    }
}

// Usage: returned vector owns its elements
let tree = /* ... */;
let elements = tree.walk();  // elements owns cloned data
// tree can be dropped here, elements still valid

// From spawn-blocking - clone Arc to share across threads
let inner = inner.clone();  // Increment reference count
tokio::spawn(async move {
    // inner now owned by this task, original still valid
});

// From complex numbers - clone for arithmetic
impl<T: Clone> Mul for Complex<T> {
    fn mul(self, rhs: Self) -> Self {
        Complex {
            re: self.re.clone() * rhs.re.clone()
                - self.im.clone() * rhs.im.clone(),
            im: self.re * rhs.im + self.im * rhs.re
        }
    }
}
```

*The clone operation reaches into borrowed memory, reads the shape and substance of the value there, and constructs a fresh, independent copy in newly owned space—the borrow remains temporary, but the clone lives on.*

◆ ◆ ◆

Prefer **REFERENCE RETURN** when possible to avoid cloning; use **ARC SHARED OWNERSHIP** (44) for thread-safe sharing without cloning the underlying data; use **COPY TYPES** for automatic copying without explicit clone calls; combine with **TO_OWNED CONVERSION** for string and slice types.
