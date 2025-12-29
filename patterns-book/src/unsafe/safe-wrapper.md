# Safe Wrapper Pattern

## Context

You have raw FFI bindings to a C library that expose unsafe functions and manual memory management. You want to provide a safe, idiomatic Rust API that prevents common errors like use-after-free, null pointer dereference, and resource leaks.

The `libgit2-rs-safe` example demonstrates this: wrapping the raw `libgit2` bindings with safe types that automatically manage resources and provide Rust-style error handling.

## Problem

**How do you wrap unsafe FFI bindings in a safe Rust API that prevents misuse, manages resources automatically, and provides idiomatic Rust patterns while maintaining the same performance as the raw C API?**

Raw FFI bindings are powerful but dangerous:
- Must manually call free functions
- Null pointers cause crashes
- Use-after-free is easy
- Error codes instead of Result types
- No lifetime tracking
- Can pass invalid arguments

The challenge is to create a safe wrapper that makes incorrect usage impossible (or at least difficult) while preserving zero-cost abstraction.

## Forces

- **Safety**: Users shouldn't need unsafe blocks to use your API
- **Ergonomics**: API should feel like idiomatic Rust
- **Performance**: Wrapping should add no runtime cost
- **Completeness**: All C functionality should be accessible
- **Correctness**: Wrapper must enforce C library's invariants
- **Maintenance**: Changes to C library should require minimal wrapper changes

These forces conflict: perfect safety requires runtime checks, but zero overhead demands compile-time guarantees.

## Solution

**Create safe Rust types that wrap raw C pointers, use the type system to enforce invariants, implement Drop for automatic cleanup, and convert C error codes to Result types.**

Follow this pattern:

1. **Opaque wrapper types**: Wrap C pointers in Rust structs
2. **Private fields**: Keep C pointers private to prevent misuse
3. **Drop implementation**: Free C resources automatically
4. **Lifetime parameters**: Use PhantomData to track borrowing relationships
5. **Result types**: Convert C error codes to Result<T, Error>
6. **Safe constructors**: Validate arguments before calling C
7. **Safe methods**: Provide safe Rust API over unsafe FFI

### Example from libgit2-rs-safe

Building a safe wrapper over raw libgit2 bindings:

```rust
mod raw;  // Raw FFI bindings

use std::error;
use std::fmt;
use std::result;
use std::path::Path;
use std::ptr;
use std::marker::PhantomData;

// Custom error type for this library
#[derive(Debug)]
pub struct Error {
    code: i32,
    message: String,
    class: i32
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        self.message.fmt(f)
    }
}

impl error::Error for Error {}

pub type Result<T> = result::Result<T, Error>;

// Helper: Convert C error codes to Result
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

// Safe wrapper around git_repository
pub struct Repository {
    // INVARIANT: This must always point to a live git_repository.
    // INVARIANT: No other Repository points to the same git_repository.
    raw: *mut raw::git_repository
}

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
        ensure_initialized();

        let path = path_to_cstring(path.as_ref())?;
        let mut repo = ptr::null_mut();

        unsafe {
            // Call C function, check error code
            check(raw::git_repository_open(&mut repo, path.as_ptr()))?;
        }

        // Return safe wrapper
        Ok(Repository { raw: repo })
    }

    pub fn reference_name_to_id(&self, name: &str) -> Result<Oid> {
        let name = CString::new(name)?;
        unsafe {
            let oid = {
                let mut oid = mem::MaybeUninit::uninit();
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

    pub fn find_commit(&self, oid: &Oid) -> Result<Commit> {
        let mut commit = ptr::null_mut();
        unsafe {
            check(raw::git_commit_lookup(&mut commit, self.raw, &oid.raw))?;
        }
        Ok(Commit {
            raw: commit,
            _marker: PhantomData  // Ties commit lifetime to repository
        })
    }
}

// Automatic cleanup when Repository is dropped
impl Drop for Repository {
    fn drop(&mut self) {
        unsafe {
            raw::git_repository_free(self.raw);
        }
    }
}

// Ensure library is initialized exactly once
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

extern fn shutdown() {
    unsafe {
        if let Err(e) = check(raw::git_libgit2_shutdown()) {
            eprintln!("shutting down libgit2 failed: {}", e);
            std::process::abort();
        }
    }
}

// Object ID wrapper
pub struct Oid {
    pub raw: raw::git_oid
}

// Commit wrapper - lifetime tied to Repository
pub struct Commit<'repo> {
    // INVARIANT: Points to valid git_commit for lifetime of Repository
    raw: *mut raw::git_commit,
    _marker: PhantomData<&'repo Repository>
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

impl<'repo> Drop for Commit<'repo> {
    fn drop(&mut self) {
        unsafe {
            raw::git_commit_free(self.raw);
        }
    }
}

// Signature wrapper - lifetime tied to owning Commit
pub struct Signature<'text> {
    raw: *const raw::git_signature,
    _marker: PhantomData<&'text str>
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

// Helper: Convert C string to &str with proper lifetime
use std::os::raw::c_char;

unsafe fn char_ptr_to_str<T>(_owner: &T, ptr: *const c_char) -> Option<&str> {
    if ptr.is_null() {
        None
    } else {
        CStr::from_ptr(ptr).to_str().ok()
    }
}

// Path to CString conversion
use std::ffi::CString;

#[cfg(unix)]
fn path_to_cstring(path: &Path) -> Result<CString> {
    use std::os::unix::ffi::OsStrExt;
    Ok(CString::new(path.as_os_str().as_bytes())?)
}

#[cfg(windows)]
fn path_to_cstring(path: &Path) -> Result<CString> {
    match path.to_str() {
        Some(s) => Ok(CString::new(s)?),
        None => {
            let message = format!("Couldn't convert path '{}' to UTF-8",
                                  path.display());
            Err(message.into())
        }
    }
}

// Error conversions
impl From<String> for Error {
    fn from(message: String) -> Error {
        Error { code: -1, message, class: 0 }
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(e: std::ffi::NulError) -> Error {
        Error { code: -1, message: e.to_string(), class: 0 }
    }
}
```

### Safe API Usage

Compare the raw API to the safe wrapper:

```rust
// ❌ Raw API - all unsafe, manual cleanup, error codes
unsafe {
    let mut repo = ptr::null_mut();
    if git_repository_open(&mut repo, path.as_ptr()) < 0 {
        // Handle error manually
    }
    // ... use repo ...
    git_repository_free(repo);  // Must remember to free!
}

// ✅ Safe wrapper - no unsafe, automatic cleanup, Result type
let repo = Repository::open(&path)?;
// ... use repo ...
// Automatically freed when repo goes out of scope
```

## Resulting Context

### Benefits

- **Memory safe**: Drop ensures resources are freed
- **Type safe**: Lifetimes prevent use-after-free
- **Ergonomic**: Idiomatic Rust error handling
- **Zero cost**: No runtime overhead over raw FFI
- **Maintainable**: Changes to wrapper don't affect users
- **Auditable**: Unsafe code isolated to wrapper implementation

### Liabilities

- **Incomplete**: May not expose all C functionality immediately
- **Complexity**: Wrapper adds additional code to maintain
- **Dual maintenance**: Must update wrapper when C library changes
- **Over-abstraction**: May hide useful C features for "safety"
- **Learning curve**: Users must learn wrapper API, not just C API

### Design Principles

#### Principle 1: Ownership Follows RAII

```rust
pub struct Resource {
    raw: *mut CResource,
}

impl Drop for Resource {
    fn drop(&mut self) {
        unsafe {
            c_free_resource(self.raw);
        }
    }
}
```

#### Principle 2: Lifetimes Prevent Dangling

```rust
// Child cannot outlive parent
pub struct Child<'parent> {
    raw: *mut CChild,
    _marker: PhantomData<&'parent Parent>
}

impl Parent {
    pub fn get_child(&self) -> Child {
        // Child's lifetime tied to &self
        Child { raw: /* ... */, _marker: PhantomData }
    }
}
```

#### Principle 3: Result Types for Errors

```rust
pub fn operation() -> Result<Value, Error> {
    unsafe {
        let code = c_operation();
        if code < 0 {
            Err(get_last_error())
        } else {
            Ok(Value { /* ... */ })
        }
    }
}
```

## Related Patterns

- **FFI Bindings**: Raw bindings are the foundation for safe wrappers
- **Resource Cleanup**: Drop ensures C resources are freed
- **C String Handling**: Convert between C and Rust strings safely
- **Safety Invariants**: Wrapper maintains invariants the C library requires

## Known Uses

- **libgit2-rs-safe**: Safe wrapper around libgit2 FFI
- **git2-rs**: Production libgit2 wrapper (similar pattern)
- **rusqlite**: Safe wrapper around SQLite
- **openssl**: Safe wrapper around OpenSSL
- **libc**: Some safe wrappers in std (File, etc.)
- **winapi**: Windows API wrappers

## Implementation Notes

### Initialization Patterns

```rust
// One-time initialization
fn ensure_initialized() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        unsafe {
            c_library_init();
        }
    });
}

// Or use lazy_static
use lazy_static::lazy_static;

lazy_static! {
    static ref INITIALIZED: () = {
        unsafe { c_library_init(); }
    };
}
```

### Builder Pattern for Complex Constructors

```rust
pub struct ConfigBuilder {
    // Fields for configuration
}

impl ConfigBuilder {
    pub fn new() -> Self { /* ... */ }

    pub fn option1(mut self, value: T) -> Self {
        self.option1 = value;
        self
    }

    pub fn build(self) -> Result<Config> {
        // Validate and call C
        unsafe {
            let cfg = c_create_config(/* ... */)?;
            Ok(Config { raw: cfg })
        }
    }
}
```

### Callback Handling

```rust
// Store Rust closure as C callback
pub fn set_callback<F>(callback: F)
where
    F: FnMut(&Event) + 'static
{
    // Box the closure
    let boxed = Box::new(callback);
    let ptr = Box::into_raw(boxed) as *mut c_void;

    unsafe {
        c_set_callback(Some(trampoline::<F>), ptr);
    }
}

// C-compatible trampoline
extern "C" fn trampoline<F>(data: *mut c_void, event: *const CEvent)
where
    F: FnMut(&Event)
{
    unsafe {
        let closure = &mut *(data as *mut F);
        let event = Event::from_raw(&*event);
        closure(&event);
    }
}
```

## Testing Safe Wrappers

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_usage() {
        let repo = Repository::open("test/repo").unwrap();
        let oid = repo.reference_name_to_id("HEAD").unwrap();
        let commit = repo.find_commit(&oid).unwrap();

        assert!(commit.message().is_some());
    }

    #[test]
    fn test_error_handling() {
        let result = Repository::open("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_lifetime_enforcement() {
        let repo = Repository::open("test/repo").unwrap();
        let commit = repo.find_commit(&oid).unwrap();
        // This won't compile - commit can't outlive repo:
        // drop(repo);
        // commit.message();
    }
}
```

## Common Pitfalls

### Pitfall 1: Forgetting Drop

```rust
// ❌ WRONG: Resource leak
pub struct Resource {
    raw: *mut CResource,
}
// Missing Drop impl - memory leaked!

// ✅ RIGHT:
impl Drop for Resource {
    fn drop(&mut self) {
        unsafe { c_free_resource(self.raw); }
    }
}
```

### Pitfall 2: Wrong Lifetime

```rust
// ❌ WRONG: Child can outlive parent
pub struct Child {
    raw: *mut CChild,
}

impl Parent {
    pub fn get_child(&self) -> Child {
        // Dangling pointer when parent is dropped!
    }
}

// ✅ RIGHT: Tie child lifetime to parent
pub struct Child<'parent> {
    raw: *mut CChild,
    _marker: PhantomData<&'parent Parent>
}
```

### Pitfall 3: Exposing Raw Pointers

```rust
// ❌ WRONG: Users can create invalid states
pub struct Resource {
    pub raw: *mut CResource,  // Public!
}

// ✅ RIGHT: Keep internals private
pub struct Resource {
    raw: *mut CResource,  // Private
}
```

### Pitfall 4: Double Free

```rust
// ❌ WRONG: Both instances free same resource
impl Clone for Resource {
    fn clone(&self) -> Self {
        Resource { raw: self.raw }  // Shared pointer!
    }
}

// ✅ RIGHT: Either don't implement Clone, or use Rc:
use std::rc::Rc;

pub struct Resource {
    raw: Rc<ResourceInner>,
}

struct ResourceInner {
    raw: *mut CResource,
}

impl Drop for ResourceInner {
    fn drop(&mut self) {
        unsafe { c_free_resource(self.raw); }
    }
}
```

## Further Reading

- *The Rustonomicon* - "Working with FFI"
- Rust API Guidelines - FFI section
- Blog: "Rust FFI: Sending strings to the outside world" by Jake Goulding
- crates.io: Search for "*-sys" crates to see raw bindings
