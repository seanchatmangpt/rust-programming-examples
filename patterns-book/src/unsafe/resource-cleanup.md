# Resource Cleanup Pattern

## Context

You are wrapping a C library or managing uninitialized memory that requires manual cleanup. Resources must be freed when no longer needed: C library objects, file handles, network connections, or allocated memory. Forgetting to free resources causes memory leaks; freeing too early causes use-after-free.

The `libgit2-rs-safe` and `gap-buffer` examples demonstrate this: using Rust's `Drop` trait to automatically free C resources and clean up uninitialized memory when wrapper types go out of scope.

## Problem

**How do you ensure that C resources and manually managed memory are always freed, even in the presence of early returns, panics, or complex control flow, without requiring explicit cleanup calls from users?**

C requires manual resource management:
- Call `free()` or library-specific cleanup functions
- Must free exactly once (not zero, not twice)
- Must free even when errors occur
- Easy to forget cleanup in complex code paths

Rust's ownership system can automate this, but you must implement the cleanup correctly. Incorrect cleanup causes:
- Memory leaks (not freeing)
- Use-after-free (freeing too early, then using)
- Double-free (freeing twice)
- Panics during unwinding (Drop panicking during panic)

## Forces

- **Automatic cleanup**: Users shouldn't manually call free functions
- **Safety**: Must free resources exactly once
- **Exception safety**: Must clean up even when panicking
- **Correctness**: Must handle partially initialized state
- **Performance**: Cleanup should be zero-cost
- **Complexity**: Tracking initialization state can be difficult

These forces conflict: simplest cleanup might leak on panic; panic-safe cleanup requires careful state tracking.

## Solution

**Implement the `Drop` trait to automatically clean up resources when the owning type goes out of scope, ensuring cleanup happens exactly once and handles partially initialized states correctly.**

Follow this pattern:

1. **Wrap resources**: Store C pointers or uninitialized memory in Rust structs
2. **Private fields**: Keep resources private to prevent external access
3. **Implement Drop**: Call cleanup functions in `drop()`
4. **Track state**: Maintain invariants about what's initialized
5. **Idempotent cleanup**: Make drop safe to call on partially initialized instances
6. **Don't panic**: Avoid panicking in Drop (or handle carefully)
7. **Test**: Verify resources are freed (use leak detectors, manual inspection)

### Example from libgit2-rs-safe

Automatic cleanup of C library objects:

```rust
mod raw;  // Raw FFI bindings

use std::ptr;
use std::marker::PhantomData;

// Repository wrapper - owns a git_repository pointer
pub struct Repository {
    // INVARIANT: This always points to a valid git_repository.
    // INVARIANT: No other Repository owns this pointer.
    raw: *mut raw::git_repository
}

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
        let path = path_to_cstring(path.as_ref())?;
        let mut repo = ptr::null_mut();

        unsafe {
            check(raw::git_repository_open(&mut repo, path.as_ptr()))?;
        }

        // Ownership transferred to Repository
        Ok(Repository { raw: repo })
    }
}

// Automatic cleanup when Repository is dropped
impl Drop for Repository {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: self.raw is guaranteed to be valid by invariants
            raw::git_repository_free(self.raw);
        }
    }
}

// Commit wrapper - lifetime tied to Repository
pub struct Commit<'repo> {
    raw: *mut raw::git_commit,
    _marker: PhantomData<&'repo Repository>
}

impl Repository {
    pub fn find_commit(&self, oid: &Oid) -> Result<Commit> {
        let mut commit = ptr::null_mut();
        unsafe {
            check(raw::git_commit_lookup(&mut commit, self.raw, &oid.raw))?;
        }
        Ok(Commit { raw: commit, _marker: PhantomData })
    }
}

// Automatic cleanup of commit
impl<'repo> Drop for Commit<'repo> {
    fn drop(&mut self) {
        unsafe {
            raw::git_commit_free(self.raw);
        }
    }
}

// Usage - no manual cleanup needed!
fn main() {
    let repo = Repository::open("my_repo").unwrap();
    let commit = repo.find_commit(&oid).unwrap();
    println!("{}", commit.message().unwrap());
    // Both commit and repo are automatically freed here
}
```

Compare to raw FFI (manual cleanup):

```rust
// ❌ Manual cleanup - easy to forget or get wrong
unsafe {
    let mut repo = ptr::null_mut();
    git_repository_open(&mut repo, path.as_ptr());

    let mut commit = ptr::null_mut();
    git_commit_lookup(&mut commit, repo, &oid);

    // Use commit...

    // Must remember to free in correct order!
    git_commit_free(commit);
    git_repository_free(repo);
}
```

### Example from gap-buffer

Cleaning up partially initialized memory:

```rust
use std::ops::Range;

pub struct GapBuffer<T> {
    storage: Vec<T>,

    // INVARIANT: Elements [0..gap.start) are initialized
    // INVARIANT: Elements [gap.start..gap.end) are UNinitialized
    // INVARIANT: Elements [gap.end..capacity) are initialized
    gap: Range<usize>
}

impl<T> Drop for GapBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            // Drop all initialized elements before gap
            // SAFETY: Invariant guarantees these are initialized
            for i in 0 .. self.gap.start {
                std::ptr::drop_in_place(self.space_mut(i));
            }

            // Skip the gap - these are uninitialized!
            // DON'T call drop_in_place on uninitialized memory

            // Drop all initialized elements after gap
            // SAFETY: Invariant guarantees these are initialized
            for i in self.gap.end .. self.capacity() {
                std::ptr::drop_in_place(self.space_mut(i));
            }
        }

        // Vec's drop handles freeing the allocation
    }
}

// Usage example:
let mut buf = GapBuffer::new();
buf.insert("hello".to_string());
buf.insert("world".to_string());
// Both strings are automatically dropped when buf is dropped
```

Key features:
- **Tracks initialization**: Only drops initialized elements
- **Skips gap**: Doesn't touch uninitialized memory
- **Safe**: Even if GapBuffer is partially filled, Drop works correctly

### Drop Order and Dependencies

Resources are dropped in this order:

```rust
struct Outer {
    field1: Inner1,
    field2: Inner2,
}

impl Drop for Outer {
    fn drop(&mut self) {
        // 1. Drop implementation runs first
        println!("Dropping Outer");
    }
    // 2. Then field2 is dropped
    // 3. Then field1 is dropped
}

// Drop order: Outer::drop() → field2 → field1
```

For dependent resources:

```rust
pub struct Handle {
    // Order matters! inner dropped before library
    inner: *mut CHandle,
    _library: Rc<Library>,  // Keeps library alive
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            c_free_handle(self.inner);
        }
        // _library dropped after this, ensuring library outlives handle
    }
}
```

## Resulting Context

### Benefits

- **Automatic**: Resources freed without explicit calls
- **Safe**: Can't forget to free (type system enforces)
- **Exception safe**: Cleanup happens even during panics
- **Composable**: Nested types clean up recursively
- **Clear ownership**: Drop makes ownership explicit
- **Zero cost**: No runtime overhead

### Liabilities

- **Drop order**: Must be careful with dependencies
- **Panic safety**: Panicking in Drop is dangerous
- **Circular references**: `Rc` cycles prevent dropping
- **Double drop**: Must prevent if using raw pointers
- **Incomplete cleanup**: Partially initialized state is tricky

### Drop Guidelines

#### Guideline 1: Don't Panic in Drop

```rust
// ❌ AVOID: Panicking in Drop
impl Drop for Resource {
    fn drop(&mut self) {
        unsafe {
            let result = c_cleanup(self.ptr);
            assert!(result == 0);  // DON'T panic in Drop!
        }
    }
}

// ✅ BETTER: Log and continue
impl Drop for Resource {
    fn drop(&mut self) {
        unsafe {
            let result = c_cleanup(self.ptr);
            if result != 0 {
                eprintln!("Warning: cleanup failed with code {}", result);
            }
        }
    }
}

// ✅ BEST: Provide explicit cleanup method for fallible cleanup
impl Resource {
    pub fn close(mut self) -> Result<(), Error> {
        unsafe {
            let result = c_cleanup(self.ptr);
            self.ptr = ptr::null_mut();  // Mark as cleaned up
            if result < 0 {
                return Err(Error::from_code(result));
            }
        }
        Ok(())
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = c_cleanup(self.ptr);  // Ignore errors
            }
        }
    }
}
```

#### Guideline 2: Handle Partially Initialized State

```rust
struct TwoResources {
    first: Option<ResourceA>,
    second: Option<ResourceB>,
}

impl TwoResources {
    fn new() -> Result<Self, Error> {
        let first = ResourceA::new()?;

        let second = match ResourceB::new() {
            Ok(r) => r,
            Err(e) => {
                // If second fails, first is dropped automatically
                return Err(e);
            }
        };

        Ok(TwoResources {
            first: Some(first),
            second: Some(second),
        })
    }
}

impl Drop for TwoResources {
    fn drop(&mut self) {
        // Drop in reverse order of initialization
        if let Some(second) = self.second.take() {
            drop(second);
        }
        if let Some(first) = self.first.take() {
            drop(first);
        }
    }
}
```

## Related Patterns

- **Safe Wrapper**: Uses Drop to clean up C resources
- **FFI Bindings**: Raw bindings need manual cleanup; wrappers use Drop
- **Safety Invariants**: Drop maintains invariants about resource state

## Known Uses

- **libgit2-rs-safe::Repository**: Frees git_repository in Drop
- **libgit2-rs-safe::Commit**: Frees git_commit in Drop
- **gap-buffer::GapBuffer**: Drops initialized elements only
- **std::fs::File**: Closes file descriptor in Drop
- **std::net::TcpStream**: Closes socket in Drop
- **std::vec::Vec**: Drops elements and deallocates in Drop
- **std::rc::Rc**: Decrements reference count in Drop
- **std::sync::Mutex**: Unlocks in Drop (MutexGuard)

## Implementation Notes

### Manual Drop (Rare)

```rust
// Sometimes you need to drop early
let resource = Resource::new();
// ... use resource ...
drop(resource);  // Explicit drop
// resource is no longer usable here
```

### Preventing Drop

```rust
use std::mem;

// Transfer ownership to C
let resource = Resource::new();
let raw = resource.into_raw();
mem::forget(resource);  // Don't drop
// C will free raw later

// Or use ManuallyDrop
use std::mem::ManuallyDrop;

let resource = ManuallyDrop::new(Resource::new());
// resource will NOT be dropped automatically
// Must manually: ManuallyDrop::drop(&mut resource)
```

### Drop Flags (Compiler-Generated)

```rust
// Compiler tracks whether fields have been moved
struct Outer {
    inner: Inner,
}

impl Outer {
    fn take_inner(self) -> Inner {
        self.inner  // inner moved out
    }  // Compiler knows not to drop inner, only Outer's Drop runs
}
```

### Conditional Drop

```rust
struct Conditional {
    resource: *mut CResource,
    owned: bool,
}

impl Drop for Conditional {
    fn drop(&mut self) {
        if self.owned && !self.resource.is_null() {
            unsafe {
                c_free(self.resource);
            }
        }
    }
}
```

### Drop and Panic

```rust
// Drop runs even during panic unwinding
struct Logger {
    name: &'static str,
}

impl Drop for Logger {
    fn drop(&mut self) {
        println!("Dropping {}", self.name);
    }
}

fn example() {
    let _a = Logger { name: "A" };
    let _b = Logger { name: "B" };
    panic!("Oh no!");
    // Output:
    // Dropping B
    // Dropping A
    // thread 'main' panicked at 'Oh no!'
}
```

### Testing Drop

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct CountedResource {
        id: usize,
    }

    impl Drop for CountedResource {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_drop_called() {
        DROP_COUNT.store(0, Ordering::SeqCst);

        {
            let _r1 = CountedResource { id: 1 };
            let _r2 = CountedResource { id: 2 };
        }  // Both dropped here

        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_drop_on_panic() {
        DROP_COUNT.store(0, Ordering::SeqCst);

        let result = std::panic::catch_unwind(|| {
            let _r = CountedResource { id: 1 };
            panic!("test");
        });

        assert!(result.is_err());
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1);
    }
}
```

### Complex Cleanup Example

```rust
struct Database {
    connection: Option<*mut CConnection>,
    transaction: Option<*mut CTransaction>,
}

impl Database {
    fn new(url: &str) -> Result<Self, Error> {
        let conn = unsafe { c_connect(url.as_ptr()) };
        if conn.is_null() {
            return Err(Error::Connection);
        }

        Ok(Database {
            connection: Some(conn),
            transaction: None,
        })
    }

    fn begin_transaction(&mut self) -> Result<(), Error> {
        let tx = unsafe {
            c_begin_transaction(self.connection.unwrap())
        };
        if tx.is_null() {
            return Err(Error::Transaction);
        }

        self.transaction = Some(tx);
        Ok(())
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        // Clean up in reverse order of initialization
        if let Some(tx) = self.transaction {
            unsafe {
                let _ = c_rollback_transaction(tx);
                c_free_transaction(tx);
            }
        }

        if let Some(conn) = self.connection {
            unsafe {
                c_disconnect(conn);
            }
        }
    }
}
```

## Common Pitfalls

### Pitfall 1: Wrong Drop Order

```rust
// ❌ WRONG: Drops in wrong order
struct BadOrder {
    library: Library,
    handle: Handle,  // Uses library
}

impl Drop for BadOrder {
    fn drop(&mut self) {
        // handle dropped first, then library
        // But handle might need library during cleanup!
    }
}

// ✅ RIGHT: Correct order
struct GoodOrder {
    handle: Handle,     // Dropped first
    library: Library,   // Dropped second
}
```

### Pitfall 2: Forgetting Null Check

```rust
// ❌ WRONG: Not checking for null
impl Drop for Resource {
    fn drop(&mut self) {
        unsafe {
            c_free(self.ptr);  // Crashes if ptr is null!
        }
    }
}

// ✅ RIGHT: Check for null
impl Drop for Resource {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                c_free(self.ptr);
            }
        }
    }
}
```

### Pitfall 3: Double Drop

```rust
// ❌ WRONG: Can be dropped twice
#[derive(Clone)]
struct Resource {
    ptr: *mut CResource,
}

impl Drop for Resource {
    fn drop(&mut self) {
        unsafe { c_free(self.ptr); }
    }
}

// When cloned, both copies free same pointer!

// ✅ RIGHT: Don't implement Clone, or use Rc
struct Resource {
    ptr: Rc<ResourceInner>,
}

struct ResourceInner {
    ptr: *mut CResource,
}

impl Drop for ResourceInner {
    fn drop(&mut self) {
        unsafe { c_free(self.ptr); }
    }
}
```

### Pitfall 4: Panicking in Drop During Panic

```rust
// ❌ WRONG: Can panic during panic
impl Drop for Resource {
    fn drop(&mut self) {
        self.critical_cleanup().unwrap();  // Panics if fails
    }
}

// If drop runs during panic unwinding, this aborts!

// ✅ RIGHT: Don't panic
impl Drop for Resource {
    fn drop(&mut self) {
        if let Err(e) = self.critical_cleanup() {
            eprintln!("Cleanup failed: {}", e);
            // Continue, don't panic
        }
    }
}
```

### Pitfall 5: Forgetting to Drop Elements

```rust
// ❌ WRONG: Leaking inner elements
struct Container<T> {
    data: Vec<T>,
}

impl<T> Drop for Container<T> {
    fn drop(&mut self) {
        // Forgot to drop elements!
        unsafe {
            dealloc(self.data.as_mut_ptr());
        }
    }
}

// ✅ RIGHT: Drop elements first
impl<T> Drop for Container<T> {
    fn drop(&mut self) {
        for i in 0..self.data.len() {
            unsafe {
                std::ptr::drop_in_place(self.data.as_mut_ptr().add(i));
            }
        }
        // Then deallocate
    }
}

// Or just let Vec's Drop do it:
// (Don't implement Drop at all if Vec already does what you need)
```

## Further Reading

- *The Rustonomicon* - "Drop" chapter
- Rust Reference - "Destructors" section
- Blog: "Learning Rust With Entirely Too Many Linked Lists" - Drop section
- RFC 1857 - Stabilization of drop order
