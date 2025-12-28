# How to Wrap FFI Safely

## Overview

This guide shows you how to create safe, idiomatic Rust wrappers around raw FFI bindings. We'll transform unsafe C bindings into a safe Rust API that prevents common errors at compile time.

## Prerequisites

- Understanding of raw FFI bindings (see [How to Create FFI Bindings](09-create-ffi-bindings.md))
- Familiarity with Rust ownership and lifetimes
- Knowledge of RAII (Resource Acquisition Is Initialization)

## Why Wrap FFI?

Raw FFI is unsafe and error-prone. By wrapping it in safe Rust types, you:
- Prevent memory leaks through automatic cleanup
- Make null pointer errors impossible
- Convert C error codes to Rust's `Result` type
- Enforce lifetime relationships at compile time
- Create an idiomatic API that feels natural to Rust users

## Design Principles

1. **Make illegal states unrepresentable**: Use types to prevent misuse
2. **Zero-cost abstractions**: No runtime overhead for safety
3. **RAII for resources**: Use `Drop` to guarantee cleanup
4. **Ergonomic API**: Hide complexity, expose functionality

## Step 1: Create a Custom Error Type

Convert C error codes into proper Rust errors:

```rust
use std::error;
use std::fmt;
use std::result;

#[derive(Debug)]
pub struct Error {
    code: i32,
    message: String,
    class: i32
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        // Display just the message, hiding internal details
        self.message.fmt(f)
    }
}

impl error::Error for Error { }

// Convenient type alias
pub type Result<T> = result::Result<T, Error>;
```

This implements the standard `Error` trait, allowing it to work with `?`, `Result`, and error handling libraries.

## Step 2: Create a Helper to Check C Error Codes

Wrap C error checking in a single function:

```rust
use std::os::raw::c_int;
use std::ffi::CStr;

fn check(code: c_int) -> Result<c_int> {
    if code >= 0 {
        return Ok(code);
    }

    unsafe {
        let error = raw::giterr_last();

        // libgit2 guarantees message is non-null and null-terminated
        let message = CStr::from_ptr((*error).message)
            .to_string_lossy()
            .into_owned();

        Err(Error {
            code: code as i32,
            message,
            class: (*error).klass as i32
        })
    }
}
```

Now instead of panicking, errors are returned as `Result`:

```rust
// Before (raw FFI): panics on error
check_raw("opening repo", raw::git_repository_open(&mut repo, path));

// After (wrapped): returns Result
raw::git_repository_open(&mut repo, path).and_then(check)?;
```

## Step 3: Wrap Resources with RAII

Use Rust structs with `Drop` to manage C resources automatically:

```rust
/// A Git repository
pub struct Repository {
    // This must always point to a valid git_repository.
    // No other Repository may point to the same resource.
    raw: *mut raw::git_repository
}

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
        ensure_initialized();  // Initialize library once

        let path = path_to_cstring(path.as_ref())?;
        let mut repo = ptr::null_mut();

        unsafe {
            check(raw::git_repository_open(&mut repo, path.as_ptr()))?;
        }

        Ok(Repository { raw: repo })
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        unsafe {
            raw::git_repository_free(self.raw);
        }
    }
}
```

Key benefits:
- **No manual cleanup**: `Drop` automatically frees the resource
- **No double-free**: Rust's ownership prevents multiple `Repository` instances for the same pointer
- **No use-after-free**: Borrow checker ensures references die before the `Repository`

**Usage comparison:**

```rust
// ❌ Raw FFI: Manual cleanup required
unsafe {
    let mut repo = ptr::null_mut();
    raw::git_repository_open(&mut repo, path.as_ptr());
    // ... use repo ...
    raw::git_repository_free(repo);  // Easy to forget!
}

// ✅ Safe wrapper: Automatic cleanup
{
    let repo = Repository::open("/path/to/repo")?;
    // ... use repo ...
}  // Automatically freed here
```

## Step 4: Initialize C Libraries Once

Many C libraries require initialization. Use `std::sync::Once` for thread-safe initialization:

```rust
fn ensure_initialized() {
    static ONCE: std::sync::Once = std::sync::Once::new();

    ONCE.call_once(|| {
        unsafe {
            check(raw::git_libgit2_init())
                .expect("initializing libgit2 failed");

            // Register cleanup at program exit
            assert_eq!(libc::atexit(shutdown), 0);
        }
    });
}

extern "C" fn shutdown() {
    unsafe {
        if let Err(e) = check(raw::git_libgit2_shutdown()) {
            eprintln!("shutting down libgit2 failed: {}", e);
            std::process::abort();
        }
    }
}
```

This ensures:
- Library is initialized exactly once, even with multiple threads
- Shutdown happens automatically at program exit
- Users don't need to worry about initialization

## Step 5: Use Lifetimes for Borrowed Resources

When returning references to data owned by C, use lifetime parameters to enforce safety:

```rust
use std::marker::PhantomData;

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
        Ok(Commit {
            raw: commit,
            _marker: PhantomData  // Ties lifetime to Repository
        })
    }
}

impl<'repo> Drop for Commit<'repo> {
    fn drop(&mut self) {
        unsafe {
            raw::git_commit_free(self.raw);
        }
    }
}
```

The `'repo` lifetime parameter means:
- A `Commit` can't outlive its `Repository`
- Compiler prevents use-after-free at compile time
- Zero runtime cost

**The compiler prevents errors:**

```rust
let commit = {
    let repo = Repository::open("/path")?;
    repo.find_commit(&oid)?
};  // ❌ Compile error: `repo` doesn't live long enough

// ✅ Correct: Keep repository alive
let repo = Repository::open("/path")?;
let commit = repo.find_commit(&oid)?;
// Use commit...
```

## Step 6: Return Borrowed Data with Lifetimes

For data that's owned by C but borrowed by Rust, use lifetimes carefully:

```rust
pub struct Signature<'text> {
    raw: *const raw::git_signature,
    _marker: PhantomData<&'text str>
}

impl<'repo> Commit<'repo> {
    pub fn author(&self) -> Signature {
        unsafe {
            Signature {
                raw: raw::git_commit_author(self.raw),
                _marker: PhantomData
            }
        }
    }

    pub fn message(&self) -> Option<&str> {
        unsafe {
            let message = raw::git_commit_message(self.raw);
            char_ptr_to_str(self, message)
        }
    }
}

impl<'text> Signature<'text> {
    pub fn name(&self) -> Option<&str> {
        unsafe {
            char_ptr_to_str(self, (*self.raw).name)
        }
    }

    pub fn email(&self) -> Option<&str> {
        unsafe {
            char_ptr_to_str(self, (*self.raw).email)
        }
    }
}

/// Convert C string to &str with proper lifetime
unsafe fn char_ptr_to_str<T>(_owner: &T, ptr: *const c_char) -> Option<&str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}
```

The `_owner` parameter ties the returned `&str` lifetime to the owner's lifetime, preventing dangling references.

## Step 7: Handle Platform Differences

Convert between Rust's `Path` type and C strings correctly on each platform:

```rust
use std::ffi::CString;
use std::path::Path;

#[cfg(unix)]
fn path_to_cstring(path: &Path) -> Result<CString> {
    use std::os::unix::ffi::OsStrExt;
    Ok(CString::new(path.as_os_str().as_bytes())?)
}

#[cfg(windows)]
fn path_to_cstring(path: &Path) -> Result<CString> {
    // Windows: Convert to UTF-8
    match path.to_str() {
        Some(s) => Ok(CString::new(s)?),
        None => {
            let message = format!("Couldn't convert path '{}' to UTF-8",
                                  path.display());
            Err(message.into())
        }
    }
}

// Support converting String to Error
impl From<String> for Error {
    fn from(message: String) -> Error {
        Error { code: -1, message, class: 0 }
    }
}

// Support CString creation errors
impl From<std::ffi::NulError> for Error {
    fn from(e: std::ffi::NulError) -> Error {
        Error { code: -1, message: e.to_string(), class: 0 }
    }
}
```

## Step 8: Create Ergonomic Value Types

Wrap C types in Rust types that are safe to copy and use:

```rust
/// Git object identifier (SHA-1 hash)
pub struct Oid {
    pub raw: raw::git_oid
}

impl Repository {
    pub fn reference_name_to_id(&self, name: &str) -> Result<Oid> {
        let name = CString::new(name)?;

        unsafe {
            let oid = {
                let mut oid = std::mem::MaybeUninit::uninit();
                check(raw::git_reference_name_to_id(
                    oid.as_mut_ptr(),
                    self.raw,
                    name.as_ptr() as *const c_char
                ))?;
                oid.assume_init()
            };

            Ok(Oid { raw: oid })
        }
    }
}
```

## Complete Example: Using the Safe API

Compare the raw FFI to the safe wrapper:

**Raw FFI (unsafe everywhere):**
```rust
unsafe {
    check(raw::git_libgit2_init());

    let path = CString::new("/path/to/repo").unwrap();
    let mut repo = ptr::null_mut();
    check(raw::git_repository_open(&mut repo, path.as_ptr()));

    let c_name = b"HEAD\0".as_ptr() as *const c_char;
    let mut oid = mem::MaybeUninit::uninit();
    check(raw::git_reference_name_to_id(
        oid.as_mut_ptr(), repo, c_name
    ));
    let oid = oid.assume_init();

    let mut commit = ptr::null_mut();
    check(raw::git_commit_lookup(&mut commit, repo, &oid));

    let author = raw::git_commit_author(commit);
    let name = CStr::from_ptr((*author).name).to_string_lossy();
    let email = CStr::from_ptr((*author).email).to_string_lossy();
    println!("{} <{}>", name, email);

    let message = raw::git_commit_message(commit);
    println!("{}", CStr::from_ptr(message).to_string_lossy());

    raw::git_commit_free(commit);
    raw::git_repository_free(repo);
    check(raw::git_libgit2_shutdown());
}
```

**Safe wrapper (clean and safe):**
```rust
fn main() -> Result<()> {
    let path = std::env::args_os().skip(1).next()
        .expect("usage: git-toy PATH");

    let repo = Repository::open(&path)?;
    let commit_oid = repo.reference_name_to_id("HEAD")?;
    let commit = repo.find_commit(&commit_oid)?;

    let author = commit.author();
    println!("{} <{}>\n",
             author.name().unwrap_or("(none)"),
             author.email().unwrap_or("(none)"));

    println!("{}", commit.message().unwrap_or("(none)"));

    Ok(())
}  // Everything cleaned up automatically
```

Key improvements:
- No `unsafe` blocks in user code
- No manual memory management
- Errors handled with `?` operator
- Readable, idiomatic Rust
- Compile-time prevention of common bugs

## Safety Guarantees

The safe wrapper provides these compile-time guarantees:

1. **No memory leaks**: RAII ensures cleanup
2. **No use-after-free**: Lifetimes prevent outliving owners
3. **No null pointer dereferences**: Types are never null
4. **No double-free**: Ownership prevents multiple destructors
5. **No data races**: Type system enforces thread safety

## Design Patterns for Safe Wrappers

### Pattern 1: Newtype Wrappers

Wrap raw pointers in structs to add behavior:

```rust
pub struct Repository {
    raw: *mut raw::git_repository
}

// Only Repository can create this pointer
// Users can't access raw field directly
```

### Pattern 2: PhantomData for Lifetimes

Tie borrowed data to owner's lifetime:

```rust
pub struct Commit<'repo> {
    raw: *mut raw::git_commit,
    _marker: PhantomData<&'repo Repository>
}
```

### Pattern 3: Builder Pattern

For complex initialization:

```rust
pub struct RepositoryBuilder {
    path: PathBuf,
    bare: bool,
    // ... other options
}

impl RepositoryBuilder {
    pub fn bare(mut self, bare: bool) -> Self {
        self.bare = bare;
        self
    }

    pub fn open(self) -> Result<Repository> {
        // Use self.path, self.bare, etc.
        todo!()
    }
}
```

### Pattern 4: Type States

Use the type system to enforce state machines:

```rust
pub struct Connection<State> {
    raw: *mut raw::connection,
    _state: PhantomData<State>
}

pub struct Disconnected;
pub struct Connected;

impl Connection<Disconnected> {
    pub fn connect(self) -> Result<Connection<Connected>> {
        // ... connect logic
        Ok(Connection { raw: self.raw, _state: PhantomData })
    }
}

impl Connection<Connected> {
    pub fn send(&self, data: &[u8]) -> Result<()> {
        // Can only call send when connected
        todo!()
    }
}
```

## Testing Safe Wrappers

Test both success and failure cases:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_valid_repo() {
        let repo = Repository::open("/path/to/repo");
        assert!(repo.is_ok());
    }

    #[test]
    fn open_invalid_repo() {
        let repo = Repository::open("/nonexistent");
        assert!(repo.is_err());
    }

    #[test]
    fn commit_lifetime_enforced() {
        // This would fail to compile:
        // let commit = {
        //     let repo = Repository::open("/path")?;
        //     repo.find_commit(&oid)?
        // };  // repo dropped here, but commit still exists
    }
}
```

## Performance Considerations

Safe wrappers should be zero-cost:

```rust
// ✅ Zero cost: just wraps a pointer
pub struct Repository {
    raw: *mut raw::git_repository
}

// ✅ Zero cost: PhantomData is zero-sized
pub struct Commit<'repo> {
    raw: *mut raw::git_commit,
    _marker: PhantomData<&'repo Repository>
}

// ❌ Not zero-cost: stores extra data
pub struct Repository {
    raw: *mut raw::git_repository,
    path: PathBuf,  // Extra heap allocation
}
```

Check assembly with `cargo asm` to verify zero overhead.

## Common Pitfalls

### Pitfall 1: Breaking Lifetime Invariants

```rust
// ❌ WRONG: Returning raw pointer breaks lifetime tracking
impl Repository {
    pub fn raw(&self) -> *mut raw::git_repository {
        self.raw  // Users could use this after Repository is dropped
    }
}

// ✅ BETTER: Keep pointer private, or use clear unsafe API
impl Repository {
    /// Returns raw pointer. Caller must ensure Repository outlives usage.
    pub unsafe fn as_raw(&self) -> *mut raw::git_repository {
        self.raw
    }
}
```

### Pitfall 2: Cloning Resource Handles

```rust
// ❌ WRONG: Cloning would double-free
#[derive(Clone)]
pub struct Repository {
    raw: *mut raw::git_repository
}

// ✅ BETTER: Don't implement Clone, or implement it correctly
impl Clone for Repository {
    fn clone(&self) -> Self {
        // Must call C API to increment ref count or make copy
        unsafe {
            raw::git_repository_clone(self.raw)
        }
    }
}
```

### Pitfall 3: Leaking Unsafe into API

```rust
// ❌ WRONG: Forcing users to use unsafe
pub fn commit_message(&self) -> *const c_char {
    unsafe { raw::git_commit_message(self.raw) }
}

// ✅ BETTER: Provide safe API
pub fn message(&self) -> Option<&str> {
    unsafe {
        let ptr = raw::git_commit_message(self.raw);
        if ptr.is_null() {
            None
        } else {
            CStr::from_ptr(ptr).to_str().ok()
        }
    }
}
```

## Next Steps

- Learn about [async wrappers for blocking C APIs](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
- Study [real-world FFI wrappers](https://github.com/rust-lang/git2-rs) for inspiration
- Read [The Rustonomicon](https://doc.rust-lang.org/nomicon/) for advanced unsafe techniques

## Related Examples

- `/home/user/rust-programming-examples/libgit2-rs-safe/src/git/mod.rs` - Safe wrapper implementation
- `/home/user/rust-programming-examples/libgit2-rs-safe/src/main.rs` - Using the safe API
- `/home/user/rust-programming-examples/libgit2-rs-safe/src/git/raw.rs` - Raw FFI declarations
