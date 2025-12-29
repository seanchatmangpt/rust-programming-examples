# 45. WHILE LET FOR ITERATION

*A loop that continues as long as a pattern matches, extracting values from Options or enum variants until None appears*

...within an **ITERATOR (26)** or **OPTION-RETURNING METHOD (22)**, when you need to repeatedly extract values from a pattern until it no longer matches...

◆ ◆ ◆

**How do you write a loop that continues until an Option returns None or an enum variant changes?**

Many iteration patterns don't fit cleanly into `for` loops. Iterators with complex state might require manual calls to `next()`, returning `Option<T>` that eventually becomes `None`. Tree traversal might descend through `NonEmpty` nodes until reaching `Empty`. Writing these with `loop` and `break` requires explicit unwrapping: `loop { match iter.next() { Some(v) => process(v), None => break } }`.

The nesting and explicit break obscure the loop's essential nature—continue while a pattern matches. Each iteration requires matching, extracting the value, processing it, and breaking on the alternate case. The structure is repetitive and error-prone; forgetting the `break` creates an infinite loop.

`while let` collapses this ceremony into a single construct. It combines pattern matching with loop continuation: `while let Some(v) = iter.next() { process(v) }`. The loop continues as long as the pattern matches. When the pattern fails (when `next()` returns `None`), the loop terminates naturally. No explicit break, no nested match.

This pattern appears in tree traversal, where nodes are visited by descending through matching variants. A binary tree might traverse left edges with `while let NonEmpty(ref node) = *tree { visit(node); tree = &node.left; }`. Each iteration extracts the node, processes it, and advances—continuing until reaching `Empty`.

**Therefore:**

**Use while let to loop as long as a pattern matches, automatically terminating when the pattern fails.**

```rust
impl<'a, T: 'a> TreeIter<'a, T> {
    fn push_left_edge(&mut self, mut tree: &'a BinaryTree<T>) {
        while let NonEmpty(ref node) = *tree {
            self.unvisited.push(node);
            tree = &node.left;
        }
    }
}

#[test]
fn external_iterator() {
    let mut tree = BinaryTree::Empty;
    tree.add("jaeger");
    tree.add("robot");
    tree.add("droid");

    let mut v = Vec::new();
    let mut state = tree.into_iter();
    while let Some(kind) = state.next() {
        v.push(*kind);
    }
    assert_eq!(v, ["droid", "jaeger", "mecha", "robot"]);
}
```

*Loop continues while NonEmpty nodes exist, or while next() returns Some*

◆ ◆ ◆

Prefer `for` loops with standard iterators when possible—they're clearer and harder to misuse. Use `while let` when manual control over iteration is required, when the condition involves pattern matching beyond simple boolean tests, or when implementing iterators themselves. Combine with **IF LET FOR OPTION UNWRAPPING (44)** for one-time checks, **ITERATOR (26)** for standard iteration, and **MATCH ON RESULT WITH QUESTION MARK (43)** when loops need error propagation.
