# Common Rust Patterns and Idioms

This repository demonstrates several fundamental Rust patterns. Understanding these patterns is key to thinking in Rust, not just translating Python code.

## The Newtype Pattern

**Example**: `/home/user/rust-programming-examples/ascii/src/lib.rs`

```rust
pub struct Ascii(Vec<u8>);
```

This is a **newtype** - a wrapper around an existing type that creates a distinct type in the type system.

### Why Newtypes?

```rust
impl Ascii {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError> {
        if bytes.iter().any(|&byte| !byte.is_ascii()) {
            return Err(NotAsciiError(bytes));
        }
        Ok(Ascii(bytes))
    }
}
```

The `Ascii` type enforces an **invariant**: the wrapped bytes are always valid ASCII. Once you have an `Ascii`, you know it's valid - no need to re-check.

### Safe, Zero-Cost Conversion

```rust
impl From<Ascii> for String {
    fn from(ascii: Ascii) -> String {
        unsafe { String::from_utf8_unchecked(ascii.0) }
    }
}
```

Because we maintain the invariant (ASCII is valid UTF-8), we can use `unsafe` conversion without checking. **The type system proves it's safe.**

### Python Doesn't Have This

In Python, you might write:

```python
class Ascii:
    def __init__(self, bytes):
        if not all(b < 128 for b in bytes):
            raise ValueError("Not ASCII")
        self.bytes = bytes
```

But nothing prevents:
```python
ascii_str = Ascii(b"hello")
ascii_str.bytes[0] = 255  # Whoops, no longer ASCII!
```

**Rust's newtype is immutable by default** and encapsulates the data. The invariant is enforced, not just hoped for.

### When to Use Newtypes

- **Enforcing constraints**: Positive numbers, non-empty strings, validated email addresses
- **Preventing mixing units**: `Meters(f64)` vs. `Feet(f64)`
- **Domain modeling**: `UserId(u64)` vs. `PostId(u64)` - can't accidentally use one as the other
- **Zero-cost abstraction**: No runtime overhead, just compile-time safety

## The RAII Pattern

**Example**: `/home/user/rust-programming-examples/libgit2-rs-safe/src/git/mod.rs`

RAII = Resource Acquisition Is Initialization. Better name: **Scope-Based Resource Management**.

### Wrapping C Resources

```rust
pub struct Repository {
    raw: *mut raw::git_repository
}

impl Drop for Repository {
    fn drop(&mut self) {
        unsafe {
            raw::git_repository_free(self.raw);
        }
    }
}
```

When a `Repository` is dropped, it automatically frees the underlying C resource. **You can't forget to free it.**

### Lifetime-Bound Resources

```rust
pub struct Commit<'repo> {
    raw: *mut raw::git_commit,
    _marker: PhantomData<&'repo Repository>
}
```

The `'repo` lifetime means: "This `Commit` can't outlive the `Repository` it came from."

```rust
let commit = {
    let repo = Repository::open(".")?;
    repo.find_commit(&oid)?
}; // ERROR: `repo` dropped here, but `commit` references it
```

The compiler prevents use-after-free **at compile time**.

### Python Comparison

Python has context managers:

```python
with open("file.txt") as f:
    data = f.read()
# File automatically closed
```

But they require explicit `with` blocks. In Rust, RAII is automatic - every value has a lifetime, and cleanup happens automatically at the end of that lifetime.

You can't accidentally:
- Forget to close a file
- Free memory twice
- Use a resource after freeing it

### Initialization and Cleanup

```rust
fn ensure_initialized() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        unsafe {
            check(raw::git_libgit2_init())
                .expect("initializing libgit2 failed");
            assert_eq!(libc::atexit(shutdown), 0);
        }
    });
}
```

Initialization happens exactly once, no matter how many times you call this function. Cleanup is registered with `atexit`. **RAII scales from single objects to entire libraries.**

## Builder Pattern

While not explicitly in the repo, the builder pattern is common in Rust. Here's the idea:

```rust
pub struct ServerConfig {
    host: String,
    port: u16,
    timeout: Duration,
    retries: usize,
}

pub struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<Duration>,
    retries: Option<usize>,
}

impl ServerConfigBuilder {
    pub fn new() -> Self {
        ServerConfigBuilder {
            host: None,
            port: None,
            timeout: None,
            retries: None,
        }
    }

    pub fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn build(self) -> Result<ServerConfig, &'static str> {
        Ok(ServerConfig {
            host: self.host.ok_or("host is required")?,
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
            retries: self.retries.unwrap_or(3),
        })
    }
}
```

Usage:
```rust
let config = ServerConfigBuilder::new()
    .host("localhost".to_string())
    .port(3000)
    .build()?;
```

### Why Builders in Rust?

Rust doesn't have:
- Optional function arguments
- Keyword arguments
- Default parameter values

The builder pattern provides:
- **Fluent API**: Method chaining
- **Validation**: `build()` can fail if required fields are missing
- **Defaults**: Some fields can have default values
- **Compile-time checks**: Can use typestate pattern to make required fields enforce at compile time

### Python Comparison

Python uses keyword arguments:

```python
config = ServerConfig(
    host="localhost",
    port=3000,
    timeout=30,  # optional
    retries=3    # optional
)
```

This is more concise but less safe - typos in argument names fail at runtime, and there's no compile-time check that you provided all required arguments.

## Type State Pattern

This takes builders further - encoding state in the type system:

```rust
struct Locked;
struct Unlocked;

struct Door<State> {
    _state: PhantomData<State>
}

impl Door<Locked> {
    fn unlock(self) -> Door<Unlocked> {
        Door { _state: PhantomData }
    }
}

impl Door<Unlocked> {
    fn open(&self) { /* ... */ }

    fn lock(self) -> Door<Locked> {
        Door { _state: PhantomData }
    }
}
```

You can only `open()` an unlocked door. **The compiler enforces the state machine.**

```rust
let door = Door::<Locked>::new();
door.open();  // ERROR: method `open` not found for `Door<Locked>`

let door = door.unlock();
door.open();  // OK!
```

**In Python**, you'd use runtime checks:

```python
class Door:
    def __init__(self):
        self.locked = True

    def open(self):
        if self.locked:
            raise RuntimeError("Door is locked")
        # ...
```

Rust catches the error at compile time, before the code even runs.

## Interior Mutability

**Example**: `/home/user/rust-programming-examples/ref-with-flag/src/lib.rs`

```rust
pub struct RefWithFlag<'a, T> {
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>
}
```

This packs a reference and a boolean into a single word by using the low bit of the pointer (pointers are aligned, so the low bit is always 0).

### Why PhantomData?

```rust
behaves_like: PhantomData<&'a T>
```

This field:
- **Occupies zero bytes** (doesn't affect layout)
- Tells the compiler to treat `RefWithFlag<'a, T>` as if it contains `&'a T`
- Ensures correct variance and drop check behavior

Without it, the compiler wouldn't know that `RefWithFlag<'a, T>` should follow the same lifetime rules as `&'a T`.

### Unsafe Pointer Manipulation

```rust
pub fn get_ref(&self) -> &'a T {
    unsafe {
        let ptr = (self.ptr_and_bit & !1) as *const T;
        &*ptr
    }
}

pub fn get_flag(&self) -> bool {
    self.ptr_and_bit & 1 != 0
}
```

**Bit manipulation** to extract pointer and flag:
- Mask off low bit (`& !1`) to get pointer
- Check low bit (`& 1`) to get flag

This is **unsafe** because we're converting `usize` to a pointer, but it's safe if we maintain the invariant that the pointer is valid.

### Memory Optimization

Standard layout:
```
&T     = 8 bytes
bool   = 1 byte
Padding= 7 bytes (alignment)
Total  = 16 bytes
```

With bit packing:
```
RefWithFlag<T> = 8 bytes
```

**50% memory savings** for this pattern. Useful in space-constrained structures like tree nodes.

### Python Comparison

Python doesn't let you do this. Everything is boxed, with overhead:

```python
class RefWithFlag:
    def __init__(self, ref, flag):
        self.ref = ref      # 8 bytes pointer
        self.flag = flag    # 28 bytes for a Python bool object!
```

Every Python object has a reference count, type pointer, and other metadata. Rust lets you work at the bit level when needed.

## Module Privacy Pattern

From `fern_sim`:

```rust
// In spores.rs
pub fn produce_spore(factory: &mut Sporangium) -> Spore {
    Spore { size: 1.0 }
}

pub(crate) fn genes(spore: &Spore) -> Vec<Gene> {
    todo!()
}

fn recombine(parent: &mut Cell) {
    todo!()
}

mod cells {
    pub struct Cell { /* ... */ }
}
```

Three levels of visibility:
1. `pub`: Visible to everyone
2. `pub(crate)`: Visible within the crate only
3. (no modifier): Visible within the module only
4. `pub(in path)`: Visible within a specific path

**And** private submodules (`mod cells`) that aren't even visible to the parent module's siblings.

This lets you create **layered APIs**:
- Public API for users
- Semi-public API for internal crates
- Private implementation details

### Python's Weak Privacy

Python has conventions:
- `_private`: "Don't use this" (but you can)
- `__very_private`: Name mangling (but you can still access it)

Nothing is truly private. In Rust, privacy is **enforced**. You cannot access a private field, period.

## Iterator Pattern

From `gap_buffer`:

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

impl<'a, T: 'a> IntoIterator for &'a GapBuffer<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        Iter { buffer: self, pos: 0 }
    }
}
```

This enables:
```rust
for element in &gap_buffer {
    println!("{:?}", element);
}
```

### Zero-Cost Abstraction

The `for` loop compiles to exactly the same code as:

```rust
let mut pos = 0;
while pos < gap_buffer.len() {
    let element = gap_buffer.get(pos).unwrap();
    println!("{:?}", element);
    pos += 1;
}
```

**No allocation, no indirection, no overhead.**

### Python's Iterator Protocol

Python has `__iter__` and `__next__`:

```python
class Iter:
    def __iter__(self):
        return self

    def __next__(self):
        # ...
```

Similar concept, but:
- Python iterators are objects (heap allocation)
- Method calls are dynamic dispatch (vtable lookup)
- No compile-time optimization

Rust iterators can be completely inlined and optimized away.

## Comparison Table

| Pattern | Rust | Python | Rust Advantage |
|---------|------|--------|----------------|
| **Newtype** | Type-level invariants | Runtime validation | Compile-time enforcement |
| **RAII** | Automatic via `Drop` | Manual with `with` blocks | Can't forget cleanup |
| **Builder** | Method chaining | Keyword args | Validation in `build()` |
| **Type State** | Compile-time state machine | Runtime checks | Impossible states unrepresentable |
| **Iterator** | Zero-cost trait | Protocol with overhead | Optimizes to loops |
| **Privacy** | Enforced by compiler | Convention only | True encapsulation |

## When to Use Each Pattern

### Newtype
- You have invariants to maintain (validated data)
- You want to prevent mixing semantically different values
- You want zero-cost type safety

### RAII
- Any time you have resources (files, sockets, locks, memory)
- C interop (wrap raw pointers)
- Guaranteed cleanup is critical

### Builder
- Structs with many optional fields
- Complex initialization logic
- You want fluent APIs

### Type State
- Complex initialization sequences
- State machines
- API misuse should be impossible

### Interior Mutability
- Shared ownership with mutation (`RefCell`, `Mutex`)
- Graph structures
- Caches and memoization

## The Meta-Pattern: Types as Documentation

In Python:
```python
def process(data, timeout=None, retries=3):
    # What type is data? What happens if timeout is negative?
    # Can retries be zero? Who knows!
    pass
```

In Rust:
```rust
fn process(data: &[u8], timeout: Option<Duration>, retries: NonZeroU32)
```

The types tell you:
- `data` is borrowed bytes (won't be modified, no allocation)
- `timeout` is optional but when present, is a positive duration
- `retries` is at least 1 (can't be zero)

**Types are checked documentation that can't get out of sync with the code.**

## Conclusion

Rust patterns aren't just translations of Python patterns. They leverage the type system and ownership model to:

1. **Prevent bugs at compile time** (newtype, type state)
2. **Guarantee cleanup** (RAII)
3. **Enable zero-cost abstractions** (iterators)
4. **Make invalid states unrepresentable** (type state)
5. **Encapsulate unsafe code** (newtype with unsafe internals)

For Python developers, the shift is from "check at runtime" to "prove at compile time." This requires more upfront thought but pays dividends in reliability and performance.

The patterns in this repository aren't academic exercises - they're battle-tested idioms used throughout the Rust ecosystem. Master them, and you'll be thinking in Rust, not just writing Python with semicolons.
