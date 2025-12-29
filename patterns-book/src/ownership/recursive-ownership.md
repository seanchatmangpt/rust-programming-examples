# Recursive Ownership

**Also Known As:** Boxed Recursion, Heap-Allocated Indirection, Recursive Sum Type

## Context

You're modeling a naturally recursive structure—a tree, linked list, or abstract syntax tree. In languages with garbage collection, you'd write `left: Tree, right: Tree` and let the runtime handle indirection. But Rust requires knowing the size of every type at compile time. A recursive struct without indirection would have infinite size: a tree node contains a tree, which contains a tree, which contains a tree...

## Problem

How do you represent recursive data structures in Rust when the type system demands compile-time-known sizes, but recursion implies potentially infinite nesting?

**The paradox:** Rust needs to know how many bytes to allocate for a `TreeNode`, but if `TreeNode` contains itself, the calculation never terminates.

## Forces

- **Compile-time sizing**: Rust allocates stack space based on type size; infinite recursion means infinite size
- **Heap allocation**: Moving data to the heap provides indirection through a fixed-size pointer
- **Ownership semantics**: Each node must own its children to enable tree destruction
- **Memory efficiency**: Unnecessary indirection wastes space and adds allocation overhead
- **Ergonomics**: Pattern matching should feel natural, not obscured by pointer manipulation
- **Safety**: No null pointers, dangling references, or cycles that leak memory

## Solution

**Use an enum with an `Empty` variant and a `NonEmpty` variant containing `Box<T>`. The `Box` provides a fixed-size pointer to heap-allocated data, breaking the infinite size recursion.**

The canonical pattern:

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
```

**Why this works:**

1. **`BinaryTree<T>` has finite size**: It's an enum with two variants
   - `Empty`: zero bytes (optimized)
   - `NonEmpty(Box<...>)`: size of one pointer (8 bytes on 64-bit)

2. **`Box<TreeNode<T>>` breaks the recursion**: The box is a pointer, not the full TreeNode
   - Pointer size: fixed (8 bytes on 64-bit)
   - Actual TreeNode: allocated on heap when created

3. **Ownership flows naturally**:
   - Each `BinaryTree` owns its `Box`
   - Each `Box` owns its `TreeNode`
   - Each `TreeNode` owns its `left` and `right` subtrees
   - When a tree is dropped, the entire structure is recursively freed

**Construction demonstrates ownership transfer:**

```rust
use self::BinaryTree::*;

let jupiter_tree = NonEmpty(Box::new(TreeNode {
    element: "Jupiter",
    left: Empty,
    right: Empty,
}));

let mercury_tree = NonEmpty(Box::new(TreeNode {
    element: "Mercury",
    left: Empty,
    right: Empty,
}));

let mars_tree = NonEmpty(Box::new(TreeNode {
    element: "Mars",
    left: jupiter_tree,    // Ownership moved here
    right: mercury_tree,   // Ownership moved here
}));
```

After `mars_tree` is created, `jupiter_tree` and `mercury_tree` are no longer accessible—their ownership has been transferred into the `mars_tree` structure. This is exactly what we want: the parent owns the children.

**Pattern matching works elegantly:**

```rust
impl<T: Clone> BinaryTree<T> {
    fn walk(&self) -> Vec<T> {
        match *self {
            BinaryTree::Empty => vec![],
            BinaryTree::NonEmpty(ref boxed) => {
                let mut result = boxed.left.walk();
                result.push(boxed.element.clone());
                result.extend(boxed.right.walk());
                result
            }
        }
    }
}
```

The `ref boxed` pattern borrows the `Box` instead of moving it. This is critical: we're implementing a read-only traversal, not consuming the tree.

**Mutation follows the same recursive pattern:**

```rust
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

Notice `ref mut node`: we're borrowing the `Box` mutably, allowing us to mutate the tree in place without taking ownership.

**Memory layout:**

The two-variant enum is optimized using "null pointer optimization":

```rust
// Size verification from the actual test
assert_eq!(size_of::<BinaryTree<String>>(), size_of::<usize>());
```

A `BinaryTree<String>` is exactly one word (8 bytes on 64-bit)—the size of a single pointer. Rust optimizes the `Empty` variant to use the null pointer value, so there's no discriminant tag overhead.

## Resulting Context

**Benefits:**
- **Finite size**: The type has a known size despite infinite potential depth
- **Ownership clarity**: The tree owns all its nodes; no lifetime parameters needed
- **Automatic cleanup**: Dropping the root recursively frees the entire tree
- **Null-pointer optimization**: Empty variants cost nothing beyond the pointer slot
- **Safe recursion**: No risk of cycles (tree structure enforced by ownership)
- **Pattern matching**: Natural `match` syntax for recursive algorithms

**Liabilities:**
- **Heap allocation**: Every node requires a heap allocation (unlike cache-friendly arrays)
- **Pointer chasing**: Tree traversal follows pointers, less cache-friendly than contiguous storage
- **Indirection overhead**: Accessing a child requires dereferencing a `Box`
- **No parent pointers**: Ownership forbids parent←child bidirectional links (would create cycles)
- **Stack overflow risk**: Deep recursion in methods can overflow the stack

**What's now possible:**
- Trees, linked lists, and other recursive structures
- Safe construction without worrying about cycles or leaks
- Recursive algorithms that mirror the data structure's shape
- Pattern matching that feels like algebraic data types from functional languages

**What remains forbidden:**
- Circular structures (cycles violate single ownership)
- Parent pointers without additional indirection (`Rc`, `Weak`)
- Efficient in-order modification (need iterator protocols)

## Related Patterns

- [Generic Container](./generic-container.md) - BinaryTree is `BinaryTree<T>`, generic from the start
- [Interior Iteration](./interior-iteration.md) - TreeIter manages iteration state without parent pointers
- Reference counting - Use `Rc<RefCell<TreeNode>>` for graphs with cycles
- Arena allocation - Alternative pattern for tree allocation without individual `Box` overhead

## Known Uses

**In this codebase:**
- `/home/user/rust-programming-examples/binary-tree/src/lib.rs` - The definitive example

**In the standard library:**
- Rust doesn't expose recursive structures directly, but internal implementations use this pattern

**In the ecosystem:**
- `syn` crate (Rust syntax trees): Extensive use of `Box<Expr>` for recursive AST nodes
- `serde_json::Value`: Recursive JSON values using `Box`
- Parser combinators: Recursive grammars represented with boxed alternatives
- Compiler implementations: ASTs, IR nodes, all use boxed recursion

**Comparison with alternatives:**

**Linked List (single recursion):**
```rust
enum List<T> {
    Nil,
    Cons(T, Box<List<T>>),
}
```

**Tree (dual recursion):**
```rust
enum Tree<T> {
    Empty,
    Node(Box<TreeNode<T>>),  // TreeNode contains two Tree children
}
```

**Graph with cycles (requires reference counting):**
```rust
struct GraphNode<T> {
    value: T,
    edges: Vec<Rc<RefCell<GraphNode<T>>>>,  // Shared ownership
}
```

**The ownership trade-off:**

This pattern enforces **tree structure** through ownership. You cannot have:
- Cycles (would require shared ownership)
- Parent pointers (would create shared ownership)
- Cross-links between subtrees (would violate single ownership)

These constraints aren't limitations—they're guarantees. A `BinaryTree<T>` is provably a tree. It cannot be a graph with cycles. This makes reasoning about the structure trivial: follow ownership from root to leaves, and you've traversed the entire structure.

**The recursive ownership principle:**

"Each parent owns its children; when the parent is dropped, the children are recursively dropped."

This mirrors how real-world trees work: cutting a branch removes all sub-branches. The ownership tree _is_ the data structure tree. They're not separate concepts requiring synchronization—they're the same thing.

**From Alexander's perspective:**

This pattern resolves the fundamental tension between compile-time guarantees and runtime flexibility. By making the heap allocation explicit (`Box`) and the recursion bounded by ownership rules, Rust gives you both: recursive structures that are safe by construction.

The pattern appears again and again because the problem appears again and again: representing hierarchical data with clear ownership. Each time you apply it—binary trees, expression trees, DOM trees, file system trees—the solution is the same, but the instantiation is unique. This is pattern thinking at its finest: a invariant core applied to infinite variations.
