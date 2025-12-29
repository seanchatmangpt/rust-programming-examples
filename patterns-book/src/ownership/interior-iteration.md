# Interior Iteration

**Also Known As:** Internal Iterator, Iterator State Machine, Lending Iterator

## Context

You've built a collection—a tree, graph, or custom container. Users need to traverse it. In garbage-collected languages, you'd expose the internal structure or maintain parent pointers. In Rust, ownership prevents bidirectional links, and exposing internals violates encapsulation. You need a way to traverse the structure without revealing implementation details or violating ownership rules.

## Problem

How do you provide iteration over a recursive or complex data structure when Rust's ownership rules prevent parent pointers, and the standard external iterator pattern requires capturing traversal state independently of the structure?

**The deeper challenge:** Tree traversal requires remembering "where you are" and "where to go next," but the tree owns all its nodes. How do you represent position without shared ownership?

## Forces

- **Ownership boundaries**: Tree owns all nodes; iterator must borrow, not own
- **Lifetime safety**: Iterator references cannot outlive the tree
- **Traversal state**: Must remember visited/unvisited nodes without modifying the tree
- **Encapsulation**: Don't expose internal structure (parent pointers, node internals)
- **Ergonomics**: Should work with `for` loops and iterator adapters
- **Efficiency**: Avoid cloning the entire tree or collecting to a `Vec`
- **Lazy evaluation**: Should compute next item on demand, not all upfront

## Solution

**Create a separate iterator type that holds a stack (or other state structure) of borrowed references to tree nodes. Implement `Iterator` to traverse using this state, borrowing from the tree without taking ownership.**

The complete pattern from binary tree:

```rust
// The state of an in-order traversal of a `BinaryTree`.
struct TreeIter<'a, T> {
    // A stack of references to tree nodes. Since we use `Vec`'s
    // `push` and `pop` methods, the top of the stack is the end of the
    // vector.
    //
    // The node the iterator will visit next is at the top of the stack,
    // with those ancestors still unvisited below it. If the stack is empty,
    // the iteration is over.
    unvisited: Vec<&'a TreeNode<T>>
}
```

**Key insight:** The iterator owns a `Vec` of _references_ to nodes. These references borrow from the tree (`'a` lifetime), so the tree must outlive the iterator.

**Bootstrapping the iteration:**

```rust
impl<'a, T: 'a> TreeIter<'a, T> {
    fn push_left_edge(&mut self, mut tree: &'a BinaryTree<T>) {
        while let NonEmpty(ref node) = *tree {
            self.unvisited.push(node);
            tree = &node.left;
        }
    }
}

impl<T> BinaryTree<T> {
    fn iter(&self) -> TreeIter<T> {
        let mut iter = TreeIter { unvisited: Vec::new() };
        iter.push_left_edge(self);
        iter
    }
}
```

The `push_left_edge` helper walks down the left spine of the tree, pushing references onto the stack. This sets up the initial state: the leftmost node (first in-order element) is at the top of the stack.

**The iteration logic:**

```rust
impl<'a, T> Iterator for TreeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        // Find the node this iteration must produce,
        // or finish the iteration.
        let node = self.unvisited.pop()?;

        // After `node`, the next thing we produce must be the leftmost
        // child in `node`'s right subtree, so push the path from here
        // down.
        self.push_left_edge(&node.right);

        // Produce a reference to this node's value.
        Some(&node.element)
    }
}
```

**How it works:**

1. Pop the top node from the stack (the next in-order element)
2. If that node has a right subtree, push its left edge onto the stack
3. Return a reference to the element

This implements in-order traversal (left, root, right) using explicit stack management instead of recursion.

**Making it work with `for` loops:**

```rust
impl<'a, T: 'a> IntoIterator for &'a BinaryTree<T> {
    type Item = &'a T;
    type IntoIter = TreeIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
```

Now users can write:

```rust
for element in &tree {
    println!("{}", element);
}
```

The `for` loop desugars to:

```rust
let mut iter = (&tree).into_iter();
while let Some(element) = iter.next() {
    println!("{}", element);
}
```

**Usage examples from the tests:**

```rust
let mut tree = BinaryTree::Empty;
tree.add("jaeger");
tree.add("robot");
tree.add("droid");
tree.add("mecha");

// Iterate with for loop
let mut v = Vec::new();
for kind in &tree {
    v.push(*kind);
}
assert_eq!(v, ["droid", "jaeger", "mecha", "robot"]);

// Iterator adapters work
assert_eq!(
    tree.iter()
        .map(|name| format!("mega-{}", name))
        .collect::<Vec<_>>(),
    vec!["mega-droid", "mega-jaeger", "mega-mecha", "mega-robot"]
);

// Manual stepping
let mut iterator = (&tree).into_iter();
assert_eq!(iterator.next(), Some(&"droid"));
assert_eq!(iterator.next(), Some(&"jaeger"));
assert_eq!(iterator.next(), Some(&"mecha"));
assert_eq!(iterator.next(), Some(&"robot"));
assert_eq!(iterator.next(), None);
```

**The lifetime contract:**

```rust
fn iteration_lifetime_example() {
    let tree = /* build tree */;
    let mut iter = tree.iter();  // iter borrows from tree

    let element = iter.next();   // element: Option<&T>

    // tree must live as long as iter and element
    drop(tree);  // ERROR: tree borrowed by iter
}
```

The compiler enforces that the tree outlives both the iterator and any references it produces. This prevents use-after-free bugs automatically.

**Contrast with external collection:**

You could implement iteration by collecting to a `Vec`:

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

But this:
- Requires `T: Clone` (not all types are cloneable)
- Allocates a `Vec` (memory overhead)
- Computes entire traversal upfront (not lazy)
- Doesn't work with iterator adapters (returns `Vec`, not iterator)

The `TreeIter` pattern avoids all these issues.

## Resulting Context

**Benefits:**
- **Lazy evaluation**: Computes next element on demand; can short-circuit (e.g., `find`)
- **No cloning required**: Produces references, not owned values
- **Iterator adapter compatibility**: Works with `map`, `filter`, `take`, `collect`, etc.
- **Memory efficiency**: Stack size proportional to tree depth, not total nodes
- **Type safety**: Lifetime system prevents use-after-free
- **Encapsulation**: Tree internals remain private; only references exposed

**Liabilities:**
- **Lifetime complexity**: `TreeIter<'a, T>` has explicit lifetime; beginners find this confusing
- **Borrowing restrictions**: Cannot mutate tree while iterator exists
- **Stack overhead**: For very deep trees, stack can grow large (but still O(depth), not O(nodes))
- **Implementation complexity**: Manual state management is more complex than recursive `walk()`
- **Single iteration**: Cannot have multiple simultaneous mutable iterators (borrow checker prevents this)

**What's now enabled:**
- Idiomatic Rust iteration with `for` loops
- Lazy, short-circuiting traversals (`find`, `any`, `all`)
- Composable iterator pipelines (`filter`, `map`, `take`)
- Memory-efficient traversal of large trees
- Zero-copy iteration (no cloning)

**What's prevented:**
- Concurrent mutation during iteration (borrow checker enforces single `&mut` or many `&`)
- Use-after-free (lifetimes prevent outliving the tree)
- Parent pointer exposure (tree structure remains encapsulated)

## Related Patterns

- [Recursive Ownership](./recursive-ownership.md) - Trees use `Box` for recursive structure; iterators borrow from this structure
- [Generic Container](./generic-container.md) - `TreeIter<'a, T>` is generic; works for any `T`
- Streaming Iterator - Alternative pattern when items cannot be borrowed (lending iterator)
- Visitor Pattern - Alternative: tree calls visitor methods instead of producing iterator

## Known Uses

**In this codebase:**
- `/home/user/rust-programming-examples/binary-tree/src/lib.rs` - `TreeIter` for in-order traversal

**In the standard library:**

**Vec iterator:**
```rust
pub struct Iter<'a, T> {
    ptr: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>,
}
```
Uses raw pointers but same concept: state independent of container.

**HashMap iterator:**
```rust
pub struct Iter<'a, K, V> {
    // Internal state for hash table traversal
}
```
Iterates buckets, maintains position without owning keys/values.

**In the ecosystem:**

**petgraph graph iteration:**
```rust
for node in graph.node_indices() {
    // Iterate node indices
}

for edge in graph.edges(node) {
    // Iterate edges from a node
}
```

**serde_json Value iteration:**
```rust
if let Value::Array(arr) = value {
    for item in arr {
        // Iterate JSON array elements
    }
}
```

**syn AST traversal:**
```rust
for item in &syntax_tree.items {
    // Iterate syntax tree items
}
```

**The state management principle:**

Interior iteration separates _container ownership_ from _traversal state_. The tree owns the nodes; the iterator owns the traversal position. This separation is critical in Rust:

- Tree: owns data, enforces structure invariants
- Iterator: owns position, implements traversal logic

Neither violates the other's invariants. The iterator borrows but doesn't mutate (for immutable iteration). The tree's structure remains intact.

**Comparison with other languages:**

**Java (external structure exposure):**
```java
class TreeNode {
    public TreeNode left;   // Exposed!
    public TreeNode right;  // Exposed!
}

// Manual traversal
while (node != null) {
    node = node.left;
}
```

**Python (internal iteration with callbacks):**
```python
def walk(tree, callback):
    if tree.left:
        walk(tree.left, callback)
    callback(tree.value)
    if tree.right:
        walk(tree.right, callback)

# Use with lambda
walk(tree, lambda x: print(x))
```

**Rust (iterator protocol):**
```rust
for value in &tree {
    println!("{}", value);
}

// Or with adapters
tree.iter()
    .filter(|x| x.len() > 5)
    .map(|x| x.to_uppercase())
    .collect::<Vec<_>>()
```

Rust's approach:
- Encapsulates structure (like Java should but doesn't)
- Enables lazy evaluation (like Python, but composable)
- Composes with ecosystem (standard `Iterator` trait)
- Guarantees safety (lifetimes prevent use-after-free)

**The deeper pattern:**

Interior iteration is a specific instance of a general principle: **separate mutable state from immutable structure.**

The tree is immutable during iteration. The iterator holds all the mutable state (the stack of unvisited nodes). This is why Rust can enforce safety: the immutable borrow of the tree prevents modification, while the iterator's owned state can change freely.

This pattern appears beyond trees:
- Iterating a `HashMap` while it's not being modified
- Traversing a graph without parent pointers
- Streaming through a file without loading it all into memory

**Alexander's pattern language:**

"The pattern is an instruction, which shows how this spatial configuration can be used, over and over again, to resolve the given system of forces."

The forces: ownership, borrowing, lazy evaluation, encapsulation.
The configuration: separate iterator type with borrowed references.
The instruction: implement `Iterator`, use lifetimes to bind iterator to container.

This pattern appears in every Rust collection because the forces are universal. Rust's ownership model makes bidirectional pointers difficult; internal state separated from structure resolves this. You'll use this pattern whenever you build a custom collection, and you'll recognize it in every standard library container you study.
