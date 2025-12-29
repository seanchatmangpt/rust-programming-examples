# 25. TRAIT BOUND ON IMPL BLOCK

*A gate that only opens for those who carry the right credentials*

...within a **[GENERIC TYPE WITH PARAMETER T](./24-generic-type-with-parameter-t.md)**, when certain operations only make sense for types with specific capabilities...

◆ ◆ ◆

**How do you implement methods that require specific capabilities without requiring those capabilities for all methods?**

Your BinaryTree<T> needs two operations: walking the tree (requires Clone to copy elements) and adding elements in sorted order (requires Ord to compare them). But Clone and Ord are different requirements. A tree of non-cloneable elements can still add items. A tree of incomparable elements can't maintain sort order but could still traverse.

You could make the whole type require both: `BinaryTree<T: Clone + Ord>`. But this is too restrictive—you can't even create a tree of non-cloneable items. You could use separate types: `SortedTree<T: Ord>` and `WalkableTree<T: Clone>`, but this duplicates the core structure. The solution is to put trait bounds on impl blocks, not on the type itself.

Multiple impl blocks can coexist for the same type with different bounds. `impl<T: Clone> BinaryTree<T>` adds methods only when T is Clone. `impl<T: Ord> BinaryTree<T>` adds different methods only when T is Ord. The type itself stays generic. This is conditional compilation at the type level—methods appear or disappear based on type capabilities.

**Therefore:**

**Put trait bounds on impl blocks, not on the type definition. Each impl block adds methods that require specific capabilities, while the type remains maximally generic.**

```rust
enum BinaryTree<T> {  // No trait bounds here
    Empty,
    NonEmpty(Box<TreeNode<T>>),
}

// Methods available when T can be cloned
impl<T: Clone> BinaryTree<T> {
    fn walk(&self) -> Vec<T> {
        match *self {
            BinaryTree::Empty => vec![],
            BinaryTree::NonEmpty(ref boxed) => {
                let mut result = boxed.left.walk();
                result.push(boxed.element.clone());  // Requires Clone
                result.extend(boxed.right.walk());
                result
            }
        }
    }
}

// Methods available when T can be compared
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
                if value <= node.element {  // Requires Ord
                    node.left.add(value);
                } else {
                    node.right.add(value);
                }
            }
        }
    }
}
```

*Different impl blocks add methods with different requirements, keeping the type maximally flexible*

◆ ◆ ◆

This pattern enables **[TRAIT-BASED POLYMORPHISM](./14-trait-based-polymorphism.md)** and works with **[MULTIPLE IMPL BLOCKS](./16-multiple-impl-blocks.md)** to organize capabilities by their requirements. Use with **[WHERE CLAUSES FOR COMPLEX BOUNDS](./26-where-clauses-for-complex-bounds.md)** when constraints grow complex.
