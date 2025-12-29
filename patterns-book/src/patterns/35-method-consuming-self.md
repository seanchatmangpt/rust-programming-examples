# 35. METHOD CONSUMING SELF

*[Illustration: A Queue struct being transformed into its component parts, with ownership flowing from the original object through the method into separate return values, the original object ceasing to exist as a new form emerges]*

...within any **STRUCT WITH METHODS (20)**, when an operation fundamentally transforms the object into something else, or extracts owned data that cannot coexist with the original structure...

◆ ◆ ◆

**Some operations don't just modify an object—they consume it, transforming it into something fundamentally different or extracting its components. Trying to preserve the original object after such operations leads to invalid states or inefficient cloning.**

Consider a queue's `split()` operation that breaks it apart into its two internal vectors. After splitting, the queue no longer exists—it has been decomposed into its parts. What should happen to the original queue?

If you use `&self`, you can only return *references* to the internal vectors, but that doesn't give ownership:

```rust
// ❌ Wrong: returns references, doesn't give ownership
fn split(&self) -> (&Vec<char>, &Vec<char>) {
    (&self.older, &self.younger)
}
// Caller gets references but can't own the vectors
// Queue still exists but is now useless (empty references don't help)
```

If you use `&mut self`, you could clear the vectors and return them, but that requires cloning or leaving the queue in a weird empty state:

```rust
// ❌ Wrong: awkward mutation
fn split(&mut self) -> (Vec<char>, Vec<char>) {
    let older = std::mem::take(&mut self.older);
    let younger = std::mem::take(&mut self.younger);
    (older, younger)
}
// Leaves self with empty vectors - confusing state
// Queue still exists but is semantically "dead"
```

The fundamental issue is that **the operation destroys the queue**. After splitting, there is no queue anymore, just two separate vectors. The original structure has been transformed. Trying to preserve it leads to either limiting the operation (only returning references) or creating a zombie object (the queue exists but is empty/invalid).

The solution is to take ownership with `self`. The method *consumes* the original value, gaining the right to dismantle it completely. The caller gives up the original object in exchange for the transformed result.

A method taking `self` can:
- Move out of fields freely (no need to replace them)
- Dismantle the entire structure
- Return owned data from fields
- Transform into a completely different type
- Never worry about leaving the original in a valid state (it no longer exists)

**Therefore:**

**For methods that transform the object into something fundamentally different, or that extract owned components, declare the receiver as `self`. This takes ownership, allowing the method to consume the value and return its transformation. The original object ceases to exist.**

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}

impl<T> Queue<T> {
    /// Consume the queue, returning its two internal vectors.
    /// Takes self by value - the queue is destroyed.
    pub fn split(self) -> (Vec<T>, Vec<T>) {
        (self.older, self.younger)
    }
}

// Usage: consumes the queue
let mut q = Queue::new();
q.push('P');
q.push('D');
assert_eq!(q.pop(), Some('P'));
q.push('X');

// split() consumes q - transfers ownership
let (older, younger) = q.split();

// q is now GONE - moved into the function, dismantled
// println!("{:?}", q);  // ERROR: q was moved

// But we have the owned vectors
assert_eq!(older, vec!['D']);
assert_eq!(younger, vec!['X']);

// This is a one-way transformation: queue → vectors
```

*[Diagram: Ownership flow in consuming methods:

```
Before split():
  q owns Queue {
    older: Vec ['D'],
    younger: Vec ['X']
  }

Calling q.split():
  Ownership of Queue moves into split()
  ↓
  split() has exclusive ownership
  ↓
  Can dismantle: moves older, moves younger
  ↓
  Returns (Vec ['D'], Vec ['X'])
  ↓
  Queue is deallocated (no fields remain)

After split():
  q: MOVED (cannot use)
  older: Vec ['D']  (new owner)
  younger: Vec ['X']  (new owner)
```

Contrast with other receivers:

```
&self    → Borrows → Returns → Original still exists
&mut self → Borrows exclusively → Modifies → Returns → Original still exists
self     → Takes ownership → Transforms → Original DESTROYED → New value returned
```
]*

Taking `self` is a **transfer of responsibility**. The caller says "I'm done with this object—transform it however you need." The method has complete freedom to dismantle, recombine, or transform without worrying about invariants of the original type (since that type is being destroyed).

This pattern is common for:
- **Destructuring**: `split()`, `into_parts()`, `into_inner()`
- **Consuming transformations**: `into_iter()` (Vec → IntoIter), `into_string()` (Vec<u8> → String)
- **Builder finalization**: `builder.build()` (Builder → FinalType)
- **Resource cleanup**: `connection.close()` (Connection → Result<(), Error>)

The standard library uses this extensively. `Vec::into_iter()` consumes the vector to create an owned iterator. `String::into_bytes()` consumes the String to return the underlying `Vec<u8>`. These operations give the caller full ownership of the internals.

◆ ◆ ◆

Use this when the operation is semantically a **transformation** or **destructuring**, not a modification. Contrast with **METHOD TAKING SELF BY MUT REFERENCE (34)** for operations that modify but preserve the object. Name these methods with `into_*` or `to_*` prefixes to signal the consumption (convention: `into_` consumes, `to_` clones then converts). After calling a consuming method, the original binding is unusable—this is enforced by the borrow checker. If you need to keep using the original, the caller must clone before calling: `let parts = queue.clone().split();`. Consuming methods should never panic after taking ownership—either complete the transformation or return a Result.
