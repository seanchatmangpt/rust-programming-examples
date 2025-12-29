# Text Processing Architecture

Text processing demands careful attention to memory efficiency and performance. Whether building text editors, search tools, or parsers, the choice of data structures and algorithms dramatically impacts both speed and memory usage. This section examines two case studies—the gap buffer and grep tool—revealing architectures optimized for different text processing scenarios.

## Stream-Based Processing Design

The grep tool demonstrates stream-based text processing, a pattern optimized for handling large files without loading them entirely into memory:

```rust
fn grep<R>(target: &str, reader: R) -> io::Result<()>
    where R: BufRead
{
    for line_result in reader.lines() {
        let line = line_result?;
        if line.contains(target) {
            println!("{}", line);
        }
    }
    Ok(())
}
```

This deceptively simple function embodies several critical architectural decisions:

**Generic Over Input Source**: The `R: BufRead` constraint makes `grep` work with any buffered input—files, stdin, network sockets, or in-memory buffers. This is **abstraction through traits**, enabling code reuse across vastly different scenarios.

**Iterator-Based Processing**: `reader.lines()` returns an iterator that lazily produces lines. The file isn't loaded into memory—instead, each line is read, processed, and discarded before the next line is read. For a 10GB log file, memory usage remains constant.

**Error Propagation with ?**: The `?` operator propagates I/O errors immediately. If reading a line fails (corrupt data, permission error), the function returns early with the error. This **fail-fast** pattern prevents partial processing of corrupted files.

**Ownership and Allocation**: Each `line` is an owned `String`, allocated for this iteration. After printing, it's dropped, freeing memory. This ownership pattern ensures no memory leaks—the compiler guarantees cleanup.

## The Grep Tool Architecture

The complete grep tool demonstrates how to structure a command-line text processor:

```rust
fn grep_main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let target = match args.next() {
        Some(s) => s,
        None => Err("usage: grep PATTERN FILE...")?
    };
    let files: Vec<PathBuf> = args.map(PathBuf::from).collect();

    if files.is_empty() {
        let stdin = io::stdin();
        grep(&target, stdin.lock())?;
    } else {
        for file in files {
            let f = File::open(file)?;
            grep(&target, BufReader::new(f))?;
        }
    }

    Ok(())
}
```

**Architecture Patterns**:

**Separation of Concerns**: Business logic (`grep`) is separated from CLI handling (`grep_main`). This enables testing `grep` with in-memory data:

```rust
#[test]
fn test_grep() {
    let input = "foo\nbar\nbaz\n";
    let cursor = std::io::Cursor::new(input);
    let mut output = Vec::new();

    grep("bar", cursor, &mut output).unwrap();
    assert_eq!(output, b"bar\n");
}
```

**Polymorphic Input Handling**: The same `grep` function handles both stdin and file input. The abstraction is the `BufRead` trait, not a concrete type. This is **interface-oriented design**.

**Error Aggregation**: The `Box<dyn Error>` return type accepts any error type implementing `Error`. This enables using `?` with both `io::Error` (from file operations) and custom errors (from argument parsing).

**Resource Management**: Files are automatically closed when the `File` goes out of scope. No explicit cleanup needed—RAII (Resource Acquisition Is Initialization) handles it. This prevents file descriptor leaks even in error paths.

## Memory-Efficient Buffers: The Gap Buffer

For text editing, different constraints apply. Insertions and deletions at arbitrary positions must be fast, but streaming isn't possible—users jump around the document. The gap buffer architecture solves this:

```rust
pub struct GapBuffer<T> {
    storage: Vec<T>,
    gap: Range<usize>
}
```

**Conceptual Model**: A gap buffer maintains a "gap" (unused space) at the cursor position. Insertions write into the gap, deletions expand the gap. Moving the cursor moves the gap. This is **spatial locality optimization**—most edits happen near the cursor.

**Visual Representation**:
```
Text:  "Hello World"
Buffer: [H][e][l][l][o][ ][ ][ ][W][o][r][l][d]
                         ^gap^
```

Inserting 'X' at cursor position 5:
```
Buffer: [H][e][l][l][o][X][ ][ ][W][o][r][l][d]
                            ^gap^
```

The gap shrinks. When it's full, reallocate with a larger gap.

## The Gap Buffer Implementation

The core operations reveal sophisticated unsafe code architecture:

**Initialization**:
```rust
impl<T> GapBuffer<T> {
    pub fn new() -> GapBuffer<T> {
        GapBuffer { storage: Vec::new(), gap: 0..0 }
    }

    pub fn capacity(&self) -> usize {
        self.storage.capacity()
    }

    pub fn len(&self) -> usize {
        self.capacity() - self.gap.len()
    }
}
```

**Key Insight**: The `storage` Vec has capacity but zero length. All elements live in Vec's "unused" capacity. This is **subverting Vec's invariants**—we use Vec for allocation but manage elements ourselves. The gap represents uninitialized memory within the buffer.

**Pointer Arithmetic for Access**:
```rust
unsafe fn space(&self, index: usize) -> *const T {
    self.storage.as_ptr().offset(index as isize)
}

unsafe fn space_mut(&mut self, index: usize) -> *mut T {
    self.storage.as_mut_ptr().offset(index as isize)
}
```

These functions provide raw access to storage, bypassing Rust's safety checks. The `unsafe` annotation indicates we're responsible for ensuring:
1. `index` is within bounds
2. The pointer points to initialized memory (or we don't dereference it)
3. No data races occur

**Index Translation**:
```rust
fn index_to_raw(&self, index: usize) -> usize {
    if index < self.gap.start {
        index
    } else {
        index + self.gap.len()
    }
}
```

This function translates logical indices (what the user sees) to physical indices (actual storage locations). Indices before the gap are unchanged; indices after the gap skip over the gap. This is **address translation**, similar to virtual memory.

**Safe Element Access**:
```rust
pub fn get(&self, index: usize) -> Option<&T> {
    let raw = self.index_to_raw(index);
    if raw < self.capacity() {
        unsafe {
            Some(&*self.space(raw))
        }
    } else {
        None
    }
}
```

The public `get` method is safe, but calls unsafe code internally. Safety comes from the bounds check: `raw < self.capacity()` ensures the pointer dereference is valid. This pattern—**unsafe internals, safe interface**—is fundamental to Rust systems programming.

## Moving the Gap: The Critical Operation

Gap movement makes or breaks gap buffer performance:

```rust
pub fn set_position(&mut self, pos: usize) {
    if pos > self.len() {
        panic!("index {} out of range for GapBuffer", pos);
    }

    unsafe {
        let gap = self.gap.clone();
        if pos > gap.start {
            // Move gap right: shift elements after gap to before it
            let distance = pos - gap.start;
            std::ptr::copy(
                self.space(gap.end),
                self.space_mut(gap.start),
                distance
            );
        } else if pos < gap.start {
            // Move gap left: shift elements before gap to after it
            let distance = gap.start - pos;
            std::ptr::copy(
                self.space(pos),
                self.space_mut(gap.end - distance),
                distance
            );
        }

        self.gap = pos .. pos + gap.len();
    }
}
```

**Algorithm Analysis**:

Moving the gap from position A to position B requires copying `|B - A|` elements. This is O(n) worst case (moving from start to end), but O(1) for typical editing (small cursor movements). The **amortized cost** is excellent for text editing workloads.

**Why ptr::copy?**: The `std::ptr::copy` function is like `memmove`—it handles overlapping memory regions correctly. We're moving elements within the same buffer, so overlap is possible. Using `copy_nonoverlapping` would be unsafe here.

**Unsafe Invariants**:
1. `gap.start` and `gap.end` are valid indices within storage capacity
2. Elements outside the gap are initialized
3. Elements inside the gap are uninitialized but not accessed

The compiler can't verify these invariants, so we document them and carefully maintain them in code.

## Insertion and Deletion

**Insertion**:
```rust
pub fn insert(&mut self, elt: T) {
    if self.gap.len() == 0 {
        self.enlarge_gap();
    }

    unsafe {
        let index = self.gap.start;
        std::ptr::write(self.space_mut(index), elt);
    }
    self.gap.start += 1;
}
```

`std::ptr::write` writes to uninitialized memory without dropping the old value (because there is no old value—it's uninitialized). This is the **correct way to initialize** memory in unsafe code.

**Deletion**:
```rust
pub fn remove(&mut self) -> Option<T> {
    if self.gap.end == self.capacity() {
        return None;
    }

    let element = unsafe {
        std::ptr::read(self.space(self.gap.end))
    };
    self.gap.end += 1;
    Some(element)
}
```

`std::ptr::read` reads from memory without dropping—it moves the value out, leaving uninitialized memory behind. This is safe because we immediately expand the gap to include that position, marking it as uninitialized.

## Memory Management and Drop

The `Drop` implementation ensures proper cleanup:

```rust
impl<T> Drop for GapBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0 .. self.gap.start {
                std::ptr::drop_in_place(self.space_mut(i));
            }
            for i in self.gap.end .. self.capacity() {
                std::ptr::drop_in_place(self.space_mut(i));
            }
        }
    }
}
```

This drops all initialized elements (those outside the gap) and leaves gap elements uninitialized. The Vec's own drop then frees the memory. This is **manual memory management**—we're responsible for running destructors.

**Why Two Loops?**: Elements are divided into two contiguous regions: `[0, gap.start)` and `[gap.end, capacity)`. The gap `[gap.start, gap.end)` contains uninitialized memory and must not be dropped.

## Performance Considerations

Gap buffer performance comes from several design choices:

**Cache Locality**: Consecutive elements are consecutive in memory (except across the gap). This maximizes CPU cache efficiency during iteration.

**Minimal Allocations**: Insertions don't allocate until the gap fills. Deletions never allocate. This is **allocation amortization**—occasional large allocations amortized over many operations.

**Incremental Resizing**: When enlarging the gap, double the capacity:

```rust
fn enlarge_gap(&mut self) {
    let mut new_capacity = self.capacity() * 2;
    if new_capacity == 0 {
        new_capacity = 4;
    }
    // ... allocation and copying ...
}
```

Doubling ensures **amortized O(1)** insertions, the same as Vec. The amortization argument: if you do n insertions, you trigger log(n) reallocations, total cost O(n).

**Trade-offs**: Gap buffers excel at localized editing but degrade with random access. Jumping around the document repeatedly moves the gap, triggering copies. For this workload, other structures (piece tables, ropes) might be better.

## Testing Unsafe Code

Gap buffer tests must verify safety invariants:

```rust
#[test]
fn test_gap_buffer_operations() {
    let mut buf = GapBuffer::new();
    buf.insert_iter("Lord of the Rings".chars());
    assert_eq!(buf.len(), 17);

    buf.set_position(12);
    buf.insert_iter("Onion ".chars());

    assert_eq!(buf.get_string(), "Lord of the Onion Rings");
}
```

**What to Test**:
- **Correctness**: Do operations produce the expected results?
- **Memory Safety**: Use `valgrind` or `miri` to detect undefined behavior
- **Edge Cases**: Empty buffer, full gap, moving gap to extremes
- **Drop Behavior**: Ensure elements are properly dropped (test with types that track allocations)

**Using Miri**:
```bash
cargo +nightly miri test
```

Miri executes tests in an interpreter that detects undefined behavior: use-after-free, invalid pointer dereferences, data races. It's invaluable for validating unsafe code.

## Architectural Lessons from Text Processing

These case studies reveal key architectural principles:

**1. Choose Data Structures by Workload**: Grep uses streaming (constant memory), gap buffer uses in-place editing (O(n) memory, fast local edits). The architecture flows from requirements.

**2. Unsafe Code Needs Discipline**: Gap buffer demonstrates the pattern: unsafe internals with safe, well-tested public APIs. Document invariants, test exhaustively, use tools like Miri.

**3. Abstraction Through Traits**: Grep's `BufRead` abstraction enables polymorphism without runtime cost. Traits are Rust's primary abstraction mechanism.

**4. Ownership Eliminates Leaks**: Both case studies benefit from RAII. Files close automatically, buffers are cleaned up properly, no manual resource management needed.

**5. Performance Through Zero-Cost Abstractions**: Gap buffer's iterator is zero-cost—it compiles to the same code as manual indexing. Grep's generic reader compiles to monomorphized code with no virtual dispatch.

## Integration with Binary Tree Example

The gap buffer iterator demonstrates the same pattern as binary tree iteration (from earlier chapters):

```rust
pub struct Iter<'a, T> {
    buffer: &'a GapBuffer<T>,
    pos: usize
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.pos >= self.buffer.len() {
            None
        } else {
            self.pos += 1;
            self.buffer.get(self.pos - 1)
        }
    }
}
```

Like `BinaryTree::iter()`, this iterator:
- Holds a reference to the data structure (lifetime `'a`)
- Maintains traversal state (`pos`)
- Implements `Iterator` trait

This is a **fundamental Rust pattern**: stateful iteration over borrowed data. The lifetime system ensures the buffer can't be modified while iterators exist, preventing invalidation bugs that plague C++ iterators.

## Cross-References to Core Concepts

The text processing architectures build on foundational patterns:

- **Chapter 2 (Ownership)**: Gap buffer's Drop implementation demonstrates custom cleanup. Grep shows how RAII manages file handles.
- **Chapter 4 (Lifetimes)**: The gap buffer iterator's `'a` lifetime connects iterator lifetime to buffer lifetime.
- **Chapter 6 (Traits)**: Grep's `BufRead` trait enables abstraction. Iterator trait standardizes iteration.
- **Chapter 8 (Unsafe)**: Gap buffer extensively uses unsafe for performance, following the "safe interface" pattern.

These case studies demonstrate that performance and safety aren't mutually exclusive. Gap buffer achieves C-like performance with Rust's safety guarantees. Grep handles arbitrarily large files without memory leaks or buffer overflows. The architecture enables both efficiency and correctness.

Next, we'll examine networking and concurrency architectures, where these patterns extend to handle multiple simultaneous connections safely.
