# Tutorial: Building a Binary Tree with Recursive Data Structures

## Introduction

In this tutorial, you'll learn how to implement a binary tree in Rust - one of the most fundamental recursive data structures. Binary trees present unique challenges in Rust due to ownership rules, but they're also a perfect example of how Rust's type system helps you write correct code.

By the end of this tutorial, you'll understand:
- How to create recursive data structures in Rust
- Why and how to use `Box<T>` for heap allocation
- How to use enums for tree nodes vs empty leaves
- Tree traversal algorithms in Rust
- The ownership challenges unique to tree structures
- How Rust trees differ from Python implementations

## Prerequisites

Before starting this tutorial, you should:
- Have completed the generics tutorial
- Understand Rust's ownership and borrowing
- Be familiar with Option<T>
- Know basic tree concepts (nodes, leaves, children)

## What You'll Build

You'll create a generic binary tree that supports:
- **Insertion** - Add elements maintaining sorted order
- **In-order traversal** - Visit all elements in sorted order
- **Iteration** - Implement the Iterator trait
- **Generic types** - Work with any ordered type

## Step 1: Understanding Binary Trees

### What is a Binary Tree?

A binary tree is a hierarchical data structure where each node has at most two children (left and right). A **binary search tree** (BST) maintains the invariant:
- All values in the left subtree < node value
- All values in the right subtree > node value

```
        "Saturn"
       /        \
    "Mars"      "Uranus"
    /    \         \
"Jupiter" "Mercury" "Venus"
```

### Python vs Rust Implementation Challenges

**Python implementation:**

```python
class TreeNode:
    def __init__(self, value):
        self.value = value
        self.left = None   # Can be None or another TreeNode
        self.right = None  # Can be None or another TreeNode

# Easy to create cycles and memory leaks!
root = TreeNode("Saturn")
root.left = TreeNode("Mars")
root.left.left = root  # Oops! Circular reference
```

**Rust's Challenges:**

1. **No null pointers** - Must use Option<T>
2. **Size must be known** - Recursive types need indirection
3. **Single ownership** - Can't have multiple owners of same node
4. **No cycles** - Ownership prevents circular references (a good thing!)

## Step 2: Designing the Tree Structure

### Using an Enum for Tree Nodes

Our tree uses an enum to represent either an empty tree or a non-empty tree:

```rust
enum BinaryTree<T> {
    Empty,
    NonEmpty(Box<TreeNode<T>>),
}

struct TreeNode<T> {
    element: T,
    left: BinaryTree<T>,
    right: BinaryTree<T>,
}
```

**Why an Enum?**

This design makes illegal states unrepresentable:
- A tree is either Empty or NonEmpty
- You can't have a node without specifying its children
- Pattern matching forces you to handle both cases

**Alternative Design (not used here):**

```rust
// This would require Option everywhere
struct TreeNode<T> {
    element: T,
    left: Option<Box<TreeNode<T>>>,
    right: Option<Box<TreeNode<T>>>,
}
```

Our enum design is cleaner and more idiomatic.

## Step 3: Understanding Box<T>

### Why Box?

Rust needs to know the size of every type at compile time. But our tree is recursive:

```rust
// This doesn't compile!
struct TreeNode<T> {
    element: T,
    left: TreeNode<T>,   // ERROR: infinite size!
    right: TreeNode<T>,
}
```

The compiler can't determine the size because `TreeNode` contains itself.

**Solution: Box<T>**

`Box<T>` is a smart pointer that allocates data on the heap:

```rust
enum BinaryTree<T> {
    Empty,                    // Size: 0 bytes (no heap allocation)
    NonEmpty(Box<TreeNode<T>>),  // Size: 1 word (pointer to heap)
}
```

Now the size is known:
- `Empty` variant: 1 byte for discriminant
- `NonEmpty` variant: 1 word (8 bytes on 64-bit systems)

### Memory Layout

```
Stack:                 Heap:
┌─────────────┐       ┌──────────────────┐
│ BinaryTree  │       │ TreeNode         │
│   NonEmpty ─┼──────>│   element: "A"   │
└─────────────┘       │   left: Empty    │
                      │   right: Empty   │
                      └──────────────────┘
```

**Box Properties:**

1. **Heap allocation** - Data stored on heap, pointer on stack
2. **Ownership** - Box owns its data
3. **Automatic cleanup** - When Box is dropped, heap memory is freed
4. **Deref coercion** - Can use like a reference

### Verifying Size

```rust
#[test]
fn binary_tree_size() {
    use std::mem::size_of;

    let word = size_of::<usize>();
    assert_eq!(size_of::<BinaryTree<String>>(), word);
    type Triple = (&'static str, BinaryTree<&'static str>, BinaryTree<&'static str>);
    assert_eq!(size_of::<Triple>(), 4 * word);
}
```

## Step 4: Creating Tree Nodes

### Manual Construction

Let's build a tree manually to understand the structure:

```rust
use BinaryTree::*;

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
    left: jupiter_tree,
    right: mercury_tree,
}));
```

**Step-by-step breakdown:**

1. `TreeNode { ... }` - Create a TreeNode struct
2. `Box::new(...)` - Allocate it on the heap
3. `NonEmpty(...)` - Wrap in the enum variant
4. Assignment moves ownership into the tree

### The Pattern of Ownership Transfer

```rust
let jupiter_tree = NonEmpty(Box::new(...));
let mars_tree = NonEmpty(Box::new(TreeNode {
    element: "Mars",
    left: jupiter_tree,  // jupiter_tree is MOVED here
    right: mercury_tree, // mercury_tree is MOVED here
}));

// jupiter_tree is no longer accessible!
// This prevents dangling pointers and use-after-free
```

## Step 5: Implementing Tree Traversal

### In-Order Walk

In-order traversal visits nodes in sorted order: left → root → right

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

**Breaking Down the Code:**

1. **Pattern matching**: `match *self` - Dereference self to match on the enum
2. **Empty case**: Return an empty vector
3. **NonEmpty case**:
   - `ref boxed` - Borrow the Box (don't move it)
   - Recursively walk left subtree
   - Add current element (cloned)
   - Extend with right subtree

**Why Clone?**

The `T: Clone` bound is necessary because we're creating a new Vec with copies of the elements. Without Clone, we'd need to move elements out of the tree, destroying it.

### Testing the Walk Method

```rust
#[test]
fn build_binary_tree() {
    use BinaryTree::*;

    // Build the tree manually
    let tree = NonEmpty(Box::new(TreeNode {
        element: "Saturn",
        left: NonEmpty(Box::new(TreeNode {
            element: "Mars",
            left: NonEmpty(Box::new(TreeNode {
                element: "Jupiter",
                left: Empty,
                right: Empty,
            })),
            right: NonEmpty(Box::new(TreeNode {
                element: "Mercury",
                left: Empty,
                right: Empty,
            })),
        })),
        right: NonEmpty(Box::new(TreeNode {
            element: "Uranus",
            left: Empty,
            right: NonEmpty(Box::new(TreeNode {
                element: "Venus",
                left: Empty,
                right: Empty,
            })),
        })),
    }));

    assert_eq!(tree.walk(),
               vec!["Jupiter", "Mars", "Mercury", "Saturn", "Uranus", "Venus"]);
}
```

## Step 6: Implementing Insert

### The Add Method

Add a value to the tree, maintaining BST property:

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

**Understanding the Implementation:**

1. **Trait bound `T: Ord`**: We need to compare values
2. **Empty case**: Replace Empty with a new leaf node
3. **NonEmpty case**:
   - `ref mut node` - Get a mutable reference to the Box
   - Compare value with current element
   - Recursively add to left or right subtree

**Key Rust Concept: Replacing Self**

```rust
*self = BinaryTree::NonEmpty(...)
```

This is powerful! We're replacing an Empty variant with a NonEmpty variant. This is safe because:
- The old value (Empty) is dropped
- The new value is moved into place
- No memory leaks or dangling pointers

### Testing Add

```rust
#[test]
fn test_add_method() {
    let planets = vec!["Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Uranus"];
    let mut tree = BinaryTree::Empty;

    for planet in planets {
        tree.add(planet);
    }

    assert_eq!(tree.walk(),
               vec!["Jupiter", "Mars", "Mercury", "Saturn", "Uranus", "Venus"]);
}
```

## Step 7: Implementing Iterator

### The Iterator State

An iterator needs to track which nodes to visit next:

```rust
struct TreeIter<'a, T> {
    // A stack of references to tree nodes. The top of the stack is the next
    // node to visit. Ancestors are below it.
    unvisited: Vec<&'a TreeNode<T>>
}
```

**Understanding Lifetimes:**

- `'a` - Lifetime parameter
- `&'a TreeNode<T>` - References that live as long as 'a
- The iterator borrows from the tree, so it can't outlive the tree

### Helper Method: Push Left Edge

```rust
impl<'a, T: 'a> TreeIter<'a, T> {
    fn push_left_edge(&mut self, mut tree: &'a BinaryTree<T>) {
        while let NonEmpty(ref node) = *tree {
            self.unvisited.push(node);
            tree = &node.left;
        }
    }
}
```

**What This Does:**

Starting from a tree, push all left-edge nodes onto the stack:

```
        D
       / \
      B   F
     / \
    A   C

Pushing left edge from D: [D, B, A]
```

### Creating the Iterator

```rust
impl<T> BinaryTree<T> {
    fn iter(&self) -> TreeIter<T> {
        let mut iter = TreeIter { unvisited: Vec::new() };
        iter.push_left_edge(self);
        iter
    }
}
```

### Implementing the Iterator Trait

```rust
impl<'a, T> Iterator for TreeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        // Find the node this iteration must produce,
        // or finish the iteration.
        let node = self.unvisited.pop()?;

        // After `node`, the next thing we produce must be the leftmost
        // child in `node`'s right subtree.
        self.push_left_edge(&node.right);

        // Produce a reference to this node's value.
        Some(&node.element)
    }
}
```

**The Algorithm:**

1. Pop the top node from the stack (next to visit)
2. Push its right subtree's left edge
3. Return reference to the node's element

**The `?` Operator:**

```rust
let node = self.unvisited.pop()?;
```

This is shorthand for:
```rust
let node = match self.unvisited.pop() {
    Some(n) => n,
    None => return None,
};
```

### Implementing IntoIterator

```rust
impl<'a, T: 'a> IntoIterator for &'a BinaryTree<T> {
    type Item = &'a T;
    type IntoIter = TreeIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
```

Now you can use `for` loops:

```rust
let mut tree = BinaryTree::Empty;
tree.add("jaeger");
tree.add("robot");
tree.add("droid");
tree.add("mecha");

for kind in &tree {
    println!("{}", kind);
}
// Prints: droid, jaeger, mecha, robot
```

## Step 8: Complete Implementation

Here's the full binary tree implementation:

```rust
#![allow(dead_code)]

use BinaryTree::*;

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

// Iterator implementation
struct TreeIter<'a, T> {
    unvisited: Vec<&'a TreeNode<T>>
}

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

impl<'a, T: 'a> IntoIterator for &'a BinaryTree<T> {
    type Item = &'a T;
    type IntoIter = TreeIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for TreeIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        let node = self.unvisited.pop()?;
        self.push_left_edge(&node.right);
        Some(&node.element)
    }
}
```

## Step 9: Advanced Usage

### Using Iterator Adapters

```rust
let mut tree = BinaryTree::Empty;
tree.add("jaeger");
tree.add("robot");
tree.add("droid");
tree.add("mecha");

// Map over elements
let mega_names: Vec<String> = tree.iter()
    .map(|name| format!("mega-{}", name))
    .collect();

assert_eq!(mega_names,
           vec!["mega-droid", "mega-jaeger", "mega-mecha", "mega-robot"]);
```

### Manual Iteration

```rust
let mut iterator = (&tree).into_iter();
assert_eq!(iterator.next(), Some(&"droid"));
assert_eq!(iterator.next(), Some(&"jaeger"));
assert_eq!(iterator.next(), Some(&"mecha"));
assert_eq!(iterator.next(), Some(&"robot"));
assert_eq!(iterator.next(), None);
```

## Key Rust Concepts for Trees

### 1. Box<T> for Heap Allocation

```rust
// Stack: 8 bytes (pointer)
let tree = NonEmpty(Box::new(TreeNode { ... }));

// Heap: sizeof(TreeNode<T>)
```

### 2. Enum for Type States

```rust
enum BinaryTree<T> {
    Empty,           // No data
    NonEmpty(Box<TreeNode<T>>),  // Has data
}
```

This makes impossible states impossible!

### 3. Pattern Matching

```rust
match *self {
    Empty => { /* handle empty */ }
    NonEmpty(ref mut node) => { /* handle non-empty */ }
}
```

### 4. Recursive Ownership

```rust
struct TreeNode<T> {
    element: T,       // Owns the element
    left: BinaryTree<T>,   // Owns the left subtree
    right: BinaryTree<T>,  // Owns the right subtree
}
```

When a TreeNode is dropped, all its children are automatically dropped too!

### 5. Lifetimes in Iterators

```rust
struct TreeIter<'a, T> {
    unvisited: Vec<&'a TreeNode<T>>  // Borrows from tree
}
```

The `'a` ensures references don't outlive the tree.

## Comparison: Rust vs Python

### Python Binary Tree

```python
class TreeNode:
    def __init__(self, value):
        self.value = value
        self.left = None
        self.right = None

class BinaryTree:
    def __init__(self):
        self.root = None

    def add(self, value):
        if self.root is None:
            self.root = TreeNode(value)
        else:
            self._add_recursive(self.root, value)

    def _add_recursive(self, node, value):
        if value <= node.value:
            if node.left is None:
                node.left = TreeNode(value)
            else:
                self._add_recursive(node.left, value)
        else:
            if node.right is None:
                node.right = TreeNode(value)
            else:
                self._add_recursive(node.right, value)
```

### Key Differences

| Feature | Rust | Python |
|---------|------|--------|
| Null handling | Enum variants (Empty/NonEmpty) | None checks |
| Memory management | Automatic (ownership) | Garbage collection |
| Type safety | Compile-time | Runtime |
| Recursion limits | Stack size | sys.setrecursionlimit() |
| Memory leaks | Impossible (no cycles) | Possible with cycles |
| Performance | Zero-cost abstraction | Interpreter overhead |

## Common Pitfalls and Solutions

### Pitfall 1: Infinite Size

```rust
// WRONG - infinite size
struct TreeNode<T> {
    element: T,
    left: TreeNode<T>,  // ERROR!
    right: TreeNode<T>,
}

// CORRECT - use Box
struct TreeNode<T> {
    element: T,
    left: BinaryTree<T>,  // where BinaryTree uses Box
    right: BinaryTree<T>,
}
```

### Pitfall 2: Moving Out of Borrowed Content

```rust
impl<T> BinaryTree<T> {
    fn walk(self) -> Vec<T> {  // Consumes tree
        match self {
            Empty => vec![],
            NonEmpty(boxed) => {
                let mut result = boxed.left.walk();
                result.push(boxed.element);  // OK - we own it
                result.extend(boxed.right.walk());
                result
            }
        }
    }
}
```

Use `self` (consumes) vs `&self` (borrows) appropriately.

### Pitfall 3: Forgetting Trait Bounds

```rust
// WRONG - T might not be comparable
impl<T> BinaryTree<T> {
    fn add(&mut self, value: T) {
        if value <= node.element {  // ERROR: T doesn't implement Ord
            // ...
        }
    }
}

// CORRECT
impl<T: Ord> BinaryTree<T> {
    fn add(&mut self, value: T) {
        // ...
    }
}
```

## Performance Considerations

### Time Complexity

| Operation | Average | Worst Case |
|-----------|---------|------------|
| Insert | O(log n) | O(n) |
| Search | O(log n) | O(n) |
| Traversal | O(n) | O(n) |

Worst case occurs with unbalanced trees (e.g., sorted input).

### Space Complexity

- **Memory per node**: `size_of::<T>() + 2 * size_of::<usize>()` (two pointers)
- **Stack depth**: O(height) for recursive operations
- **No overhead**: Box has zero runtime cost beyond heap allocation

### Optimization: Balancing

For production use, consider self-balancing trees:
- AVL trees
- Red-Black trees
- B-trees (used in std::collections::BTreeMap)

## Exercises

### Exercise 1: Implement Contains

Check if a value exists in the tree:

```rust
impl<T: Ord> BinaryTree<T> {
    pub fn contains(&self, value: &T) -> bool {
        // Your implementation here
        todo!()
    }
}
```

### Exercise 2: Implement Height

Calculate the tree's height:

```rust
impl<T> BinaryTree<T> {
    pub fn height(&self) -> usize {
        // Your implementation here
        // Hint: max(left.height(), right.height()) + 1
        todo!()
    }
}
```

### Exercise 3: Pre-order and Post-order Traversal

Implement different traversal orders:

```rust
impl<T: Clone> BinaryTree<T> {
    pub fn preorder(&self) -> Vec<T> {
        // Root → Left → Right
        todo!()
    }

    pub fn postorder(&self) -> Vec<T> {
        // Left → Right → Root
        todo!()
    }
}
```

### Exercise 4: Implement Remove

Remove a value from the tree:

```rust
impl<T: Ord> BinaryTree<T> {
    pub fn remove(&mut self, value: &T) -> Option<T> {
        // This is challenging! Consider the three cases:
        // 1. Node is a leaf
        // 2. Node has one child
        // 3. Node has two children (use successor)
        todo!()
    }
}
```

## Next Steps

Now that you understand recursive data structures with Box, you're ready to:

1. **Learn about Rc and RefCell**: For shared ownership trees
2. **Study the Gap Buffer**: A different data structure for text editing
3. **Explore arena allocation**: Alternative to Box for graphs
4. **Implement graph structures**: Using Vec or HashMap

## Summary

Congratulations! You've built a fully functional binary search tree in Rust. You've learned:

- How to use Box<T> for recursive data structures
- Why enums are perfect for tree nodes
- How ownership prevents common tree bugs (cycles, dangling pointers)
- How to implement tree traversal recursively and iteratively
- How to create custom iterators with lifetimes
- The differences between Rust and Python tree implementations

This knowledge is essential for understanding Rust's approach to complex data structures and will help you design safe, efficient data structures in your own projects.

## Resources

- [Rust Book - Box<T>](https://doc.rust-lang.org/book/ch15-01-box.html)
- [Rust Book - Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [Learning Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)
- [Rust Collections - BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html)
