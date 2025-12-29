# 23. ENUM WITH EMPTY AND NONEMPTY

*A box that is either completely empty or contains something—never half-full*

...within a **[RECURSIVE DATA STRUCTURE](./18-recursive-data-structure.md)**, when you need to represent the absence of data as a distinct state, not as a null pointer...

◆ ◆ ◆

**How do you represent a structure that can be either empty or contain data, without using null pointers?**

Recursive structures like trees and linked lists need a way to terminate. In C, you'd use NULL. In Java, you'd use null references. But Rust has no null—the billion-dollar mistake is prevented by design. You must explicitly represent the presence or absence of a value.

You could use `Option<Box<TreeNode<T>>>`, and many Rust programs do. But this creates indirection even for the empty case—you're wrapping nothing in an Option wrapper. An enum with explicit Empty and NonEmpty variants makes the two states symmetric and clear. Empty is just Empty, not Some(nothing).

The enum also makes pattern matching natural. When you match on the tree, you handle Empty and NonEmpty as equal cases. There's no special nil checking, no defensive null guards. The type system enforces that you must handle both cases. This is especially elegant in recursive algorithms where the base case is Empty.

**Therefore:**

**Define an enum with two variants: Empty (no data) and NonEmpty(Box<Node>) (contains data). Use pattern matching to handle both cases explicitly.**

```rust
// An ordered collection of `T`s.
enum BinaryTree<T> {
    Empty,
    NonEmpty(Box<TreeNode<T>>),
}

// A part of a BinaryTree.
struct TreeNode<T> {
    element: T,
    left: BinaryTree<T>,
    right: BinaryTree<T>,
}

impl<T: Ord> BinaryTree<T> {
    fn add(&mut self, value: T) {
        match *self {
            BinaryTree::Empty => {
                *self = BinaryTree::NonEmpty(Box::new(TreeNode {
                    element: value,
                    left: BinaryTree::Empty,
                    right: BinaryTree::Empty,
                }))
            }
            BinaryTree::NonEmpty(ref mut node) => {
                if value <= node.element {
                    node.left.add(value);
                } else {
                    node.right.add(value);
                }
            }
        }
    }
}
```

*The enum clearly distinguishes between an empty tree and a tree with at least one node, making the base case explicit*

◆ ◆ ◆

This pattern works with **[BOX FOR INDIRECTION](./17-box-for-indirection.md)** to break cycles in recursive types, and with **[PATTERN MATCHING ON ENUM](./12-pattern-matching-on-enum.md)** to handle both cases exhaustively.
