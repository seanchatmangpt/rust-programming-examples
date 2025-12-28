# Data Structures API Reference

This reference provides complete API documentation for custom data structures implemented in the repository.

## Queue - Character Queue

A first-in, first-out (FIFO) queue of characters with amortized O(1) operations.

### Struct Definition

```rust
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last
    younger: Vec<char>  // younger elements, youngest last
}
```

**Implementation Strategy:**
- Uses two vectors to achieve amortized O(1) push and pop
- Elements are pushed onto `younger`
- Elements are popped from `older` (reversed `younger` when empty)

### Methods

#### `new`

Creates a new empty queue.

**Signature:**
```rust
pub fn new() -> Queue
```

**Returns:**
- `Queue`: Empty queue instance

**Complexity:**
- Time: O(1)
- Space: O(1)

**Example:**
```rust
let mut q = Queue::new();
```

---

#### `push`

Pushes a character onto the back of the queue.

**Signature:**
```rust
pub fn push(&mut self, c: char)
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `c` | `char` | Character to enqueue |

**Complexity:**
- Time: O(1) amortized
- Space: O(1) amortized

**Example:**
```rust
let mut q = Queue::new();
q.push('a');
q.push('b');
```

---

#### `pop`

Removes and returns the character at the front of the queue.

**Signature:**
```rust
pub fn pop(&mut self) -> Option<char>
```

**Returns:**
- `Some(char)`: The front character if the queue is non-empty
- `None`: If the queue is empty

**Complexity:**
- Time: O(1) amortized, O(n) worst case (when reversing)
- Space: O(1)

**Example:**
```rust
let mut q = Queue::new();
q.push('a');
assert_eq!(q.pop(), Some('a'));
assert_eq!(q.pop(), None);
```

---

#### `is_empty`

Checks if the queue is empty.

**Signature:**
```rust
pub fn is_empty(&self) -> bool
```

**Returns:**
- `bool`: `true` if the queue contains no elements

**Complexity:**
- Time: O(1)
- Space: O(1)

**Example:**
```rust
let mut q = Queue::new();
assert!(q.is_empty());
q.push('x');
assert!(!q.is_empty());
```

---

#### `split`

Consumes the queue and returns its internal vectors.

**Signature:**
```rust
pub fn split(self) -> (Vec<char>, Vec<char>)
```

**Returns:**
- `(Vec<char>, Vec<char>)`: Tuple of (older, younger) vectors

**Complexity:**
- Time: O(1) - moves ownership
- Space: O(1)

**Example:**
```rust
let mut q = Queue::new();
q.push('P');
q.push('D');
q.pop();
q.push('X');

let (older, younger) = q.split();
assert_eq!(older, vec!['D']);
assert_eq!(younger, vec!['X']);
```

---

## Queue&lt;T&gt; - Generic Queue

A generic first-in, first-out queue that works with any type `T`.

### Struct Definition

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}
```

**Type Parameters:**
| Parameter | Bounds | Description |
|-----------|--------|-------------|
| `T` | (none) | Type of elements stored in the queue |

### Methods

#### `new`

Creates a new empty generic queue.

**Signature:**
```rust
pub fn new() -> Self
```

**Returns:**
- `Queue<T>`: Empty queue instance

**Example:**
```rust
let mut q = Queue::<String>::new();
let mut nums = Queue::new(); // Type inferred from usage
nums.push(42);
```

---

#### `push`

Pushes an element onto the back of the queue.

**Signature:**
```rust
pub fn push(&mut self, t: T)
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `t` | `T` | Element to enqueue |

**Complexity:**
- Time: O(1) amortized
- Space: O(1) amortized

---

#### `pop`

Removes and returns the element at the front of the queue.

**Signature:**
```rust
pub fn pop(&mut self) -> Option<T>
```

**Returns:**
- `Some(T)`: The front element if the queue is non-empty
- `None`: If the queue is empty

**Complexity:**
- Time: O(1) amortized
- Space: O(1)

**Example:**
```rust
let mut q = Queue::new();
q.push("hello");
q.push("world");
assert_eq!(q.pop(), Some("hello"));
```

---

#### `is_empty`

Checks if the queue is empty.

**Signature:**
```rust
pub fn is_empty(&self) -> bool
```

**Returns:**
- `bool`: `true` if the queue contains no elements

---

#### `split`

Consumes the queue and returns its internal vectors.

**Signature:**
```rust
pub fn split(self) -> (Vec<T>, Vec<T>)
```

**Returns:**
- `(Vec<T>, Vec<T>)`: Tuple of (older, younger) vectors

**Example:**
```rust
let mut q = Queue::new();
q.push('P');
q.push('D');
assert_eq!(q.pop(), Some('P'));
q.push('X');

assert_eq!(q.split(), (vec!['D'], vec!['X']));
```

---

## BinaryTree&lt;T&gt; - Binary Search Tree

An ordered collection of values stored in a binary tree structure.

### Type Definition

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

**Type Parameters:**
| Parameter | Bounds | Description |
|-----------|--------|-------------|
| `T` | Varies by method | Type of elements stored in the tree |

**Memory Layout:**
- `BinaryTree<T>` is one word (size of a pointer)
- Nodes are heap-allocated via `Box`

### Methods

#### `add` (requires `T: Ord`)

Inserts a value into the binary search tree.

**Signature:**
```rust
impl<T: Ord> BinaryTree<T> {
    fn add(&mut self, value: T)
}
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `value` | `T` | Value to insert |

**Trait Bounds:**
- `T: Ord` - Elements must be orderable

**Behavior:**
- Inserts value maintaining BST property
- Values <= node go left, > node go right
- Allows duplicate values

**Complexity:**
- Time: O(h) where h is tree height (O(log n) balanced, O(n) worst)
- Space: O(h) for recursion stack

**Example:**
```rust
let mut tree = BinaryTree::Empty;
tree.add("Mercury");
tree.add("Venus");
tree.add("Mars");
```

---

#### `walk` (requires `T: Clone`)

Performs in-order traversal, collecting elements into a vector.

**Signature:**
```rust
impl<T: Clone> BinaryTree<T> {
    fn walk(&self) -> Vec<T>
}
```

**Returns:**
- `Vec<T>`: Elements in sorted order (for BST)

**Trait Bounds:**
- `T: Clone` - Elements must be cloneable

**Complexity:**
- Time: O(n)
- Space: O(n)

**Example:**
```rust
let mut tree = BinaryTree::Empty;
tree.add("Mars");
tree.add("Jupiter");
tree.add("Saturn");
assert_eq!(tree.walk(), vec!["Jupiter", "Mars", "Saturn"]);
```

---

#### `iter`

Creates an iterator over the tree's elements.

**Signature:**
```rust
fn iter(&self) -> TreeIter<T>
```

**Returns:**
- `TreeIter<'a, T>`: Iterator yielding `&'a T` in sorted order

**Complexity:**
- Time: O(1) to create iterator
- Space: O(h) for iterator state

**Example:**
```rust
let mut tree = BinaryTree::Empty;
tree.add("droid");
tree.add("jaeger");

let elements: Vec<_> = tree.iter()
    .map(|name| format!("mega-{}", name))
    .collect();
```

---

### Iterator Support

#### `IntoIterator` for `&BinaryTree<T>`

**Signature:**
```rust
impl<'a, T: 'a> IntoIterator for &'a BinaryTree<T> {
    type Item = &'a T;
    type IntoIter = TreeIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter
}
```

**Example:**
```rust
for element in &tree {
    println!("{}", element);
}
```

---

### TreeIter - Tree Iterator

Custom iterator that performs in-order traversal.

**Struct Definition:**
```rust
struct TreeIter<'a, T> {
    unvisited: Vec<&'a TreeNode<T>>
}
```

**Iterator Implementation:**
```rust
impl<'a, T> Iterator for TreeIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T>
}
```

**Traversal Strategy:**
- Maintains a stack of unvisited nodes
- Visits nodes in sorted order (in-order traversal)

---

## GapBuffer&lt;T&gt; - Text Editor Buffer

A data structure optimized for text editing, allowing constant-time insertion and deletion at a cursor position.

### Struct Definition

```rust
pub struct GapBuffer<T> {
    storage: Vec<T>,
    gap: Range<usize>  // Range of uninitialized elements
}
```

**Type Parameters:**
| Parameter | Bounds | Description |
|-----------|--------|-------------|
| `T` | (none) | Type of elements stored |

**Implementation Strategy:**
- Maintains a "gap" of uninitialized space at the cursor
- Moving cursor moves the gap
- Insertions fill the gap from the left
- Deletions expand the gap

### Methods

#### `new`

Creates a new empty gap buffer.

**Signature:**
```rust
pub fn new() -> GapBuffer<T>
```

**Returns:**
- `GapBuffer<T>`: Empty buffer instance

**Example:**
```rust
let mut buf = GapBuffer::new();
```

---

#### `capacity`

Returns the total capacity of the buffer.

**Signature:**
```rust
pub fn capacity(&self) -> usize
```

**Returns:**
- `usize`: Number of elements the buffer can hold without reallocation

---

#### `len`

Returns the number of elements currently in the buffer.

**Signature:**
```rust
pub fn len(&self) -> usize
```

**Returns:**
- `usize`: Count of initialized elements (excluding the gap)

**Complexity:**
- Time: O(1)

---

#### `position`

Returns the current cursor position.

**Signature:**
```rust
pub fn position(&self) -> usize
```

**Returns:**
- `usize`: Index where insertions will occur

**Example:**
```rust
let mut buf = GapBuffer::new();
buf.insert('h');
assert_eq!(buf.position(), 1);
```

---

#### `set_position`

Moves the cursor to a new position.

**Signature:**
```rust
pub fn set_position(&mut self, pos: usize)
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `pos` | `usize` | New cursor position (0 to len()) |

**Panics:**
- If `pos > len()`

**Complexity:**
- Time: O(d) where d = distance moved
- Space: O(1)

**Example:**
```rust
let mut buf = GapBuffer::new();
buf.insert_iter("hello".chars());
buf.set_position(0);  // Move to start
```

---

#### `insert`

Inserts an element at the current position.

**Signature:**
```rust
pub fn insert(&mut self, elt: T)
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `elt` | `T` | Element to insert |

**Complexity:**
- Time: O(1) amortized (may trigger reallocation)
- Space: O(1) amortized

**Example:**
```rust
let mut buf = GapBuffer::new();
buf.insert('a');
buf.insert('b');
```

---

#### `insert_iter`

Inserts multiple elements from an iterator.

**Signature:**
```rust
pub fn insert_iter<I>(&mut self, iterable: I)
where
    I: IntoIterator<Item=T>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `iterable` | `I: IntoIterator<Item=T>` | Source of elements |

**Example:**
```rust
let mut buf = GapBuffer::new();
buf.insert_iter("hello".chars());
```

---

#### `remove`

Removes and returns the element just after the cursor.

**Signature:**
```rust
pub fn remove(&mut self) -> Option<T>
```

**Returns:**
- `Some(T)`: The element if cursor is not at end
- `None`: If cursor is at the end

**Complexity:**
- Time: O(1)
- Space: O(1)

**Example:**
```rust
let mut buf = GapBuffer::new();
buf.insert_iter("abc".chars());
buf.set_position(0);
assert_eq!(buf.remove(), Some('a'));
```

---

#### `get`

Returns a reference to the element at the given index.

**Signature:**
```rust
pub fn get(&self, index: usize) -> Option<&T>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `index` | `usize` | Logical index (ignoring gap) |

**Returns:**
- `Some(&T)`: Reference to element if index is valid
- `None`: If index is out of bounds

**Complexity:**
- Time: O(1)

---

### Special Implementation for `GapBuffer<char>`

#### `get_string`

Collects all characters into a `String`.

**Signature:**
```rust
impl GapBuffer<char> {
    pub fn get_string(&self) -> String
}
```

**Returns:**
- `String`: All characters in the buffer

**Example:**
```rust
let mut buf = GapBuffer::new();
buf.insert_iter("Lord of the Rings".chars());
buf.set_position(12);
buf.insert_iter("Onion ".chars());
assert_eq!(buf.get_string(), "Lord of the Onion Rings");
```

---

### Trait Implementations

#### `IntoIterator` for `&GapBuffer<T>`

```rust
impl<'a, T: 'a> IntoIterator for &'a GapBuffer<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
}
```

**Example:**
```rust
for element in &gap_buffer {
    println!("{:?}", element);
}
```

#### `Debug` for `GapBuffer<T: Debug>`

Formats the buffer for debugging, showing only the elements (not the gap).

---

## Complexity Summary

| Data Structure | Operation | Best Case | Average | Worst Case | Space |
|----------------|-----------|-----------|---------|------------|-------|
| Queue | push | O(1) | O(1) | O(n)* | O(1) |
| Queue | pop | O(1) | O(1) | O(n)** | O(1) |
| BinaryTree | add | O(log n) | O(log n) | O(n) | O(log n) |
| BinaryTree | walk | O(n) | O(n) | O(n) | O(n) |
| GapBuffer | insert | O(1) | O(1) | O(n)* | O(1) |
| GapBuffer | remove | O(1) | O(1) | O(1) | O(1) |
| GapBuffer | set_position | O(1) | O(n/2) | O(n) | O(1) |

\* Amortized O(1) - reallocation occurs
\*\* When reversing younger to older
