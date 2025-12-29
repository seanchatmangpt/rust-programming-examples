# Complete Case Study: libgit2 Integration

## Overview: Two Approaches to FFI

The repository contains two libgit2 integrations that demonstrate the spectrum from raw unsafe FFI to safe, idiomatic Rust APIs:

1. **libgit2-rs**: Raw FFI bindings exposing C functions directly
2. **libgit2-rs-safe**: Safe wrapper providing a Rust-native API

Comparing these projects reveals the principles, patterns, and trade-offs in FFI design. We'll also compare with simpler unsafe examples (`ascii`, `ref-with-flag`, `gap-buffer`) to see how complexity scales.

## The Raw FFI Bindings: libgit2-rs

### Declaring C Functions

The `raw.rs` module declares the C API:

```rust
#[link(name = "git2")]
extern {
    pub fn git_libgit2_init() -> c_int;
    pub fn git_libgit2_shutdown() -> c_int;
    pub fn giterr_last() -> *const git_error;

    pub fn git_repository_open(out: *mut *mut git_repository,
                               path: *const c_char) -> c_int;
    pub fn git_repository_free(repo: *mut git_repository);

    pub fn git_commit_lookup(out: *mut *mut git_commit,
                             repo: *mut git_repository,
                             id: *const git_oid) -> c_int;

    pub fn git_commit_author(commit: *const git_commit) -> *const git_signature;
    pub fn git_commit_message(commit: *const git_commit) -> *const c_char;
    pub fn git_commit_free(commit: *mut git_commit);
}
```

Key observations:

1. **The `extern` block**: Declares foreign functions
2. **The `#[link]` attribute**: Links against libgit2
3. **Raw pointer types**: C doesn't have references, only pointers
4. **Out parameters**: `out: *mut *mut T` is how C returns pointers
5. **C types**: `c_int`, `c_char` from `std::os::raw`

### Opaque Types

C structs that Rust shouldn't inspect are declared as opaque:

```rust
#[repr(C)] pub struct git_repository { _private: [u8; 0] }
#[repr(C)] pub struct git_commit { _private: [u8; 0] }
```

The `_private` field:
- Is zero-sized (doesn't take space)
- Prevents construction in Rust
- Allows pointers to the type
- Hides the internal structure

This is safer than `enum { }` (which would be uninhabited) because it allows null pointers.

### Transparent Types

Some C structs must be read or constructed in Rust:

```rust
#[repr(C)]
pub struct git_error {
    pub message: *const c_char,
    pub klass: c_int
}

#[repr(C)]
pub struct git_signature {
    pub name: *const c_char,
    pub email: *const c_char,
    pub when: git_time
}
```

The `#[repr(C)]` attribute ensures field order and alignment match C. Without it, Rust could reorder fields, breaking FFI.

### Using Raw Bindings

The `main.rs` shows how to use raw FFI:

```rust
unsafe {
    check("initializing library", raw::git_libgit2_init());

    let mut repo = ptr::null_mut();
    check("opening repository",
          raw::git_repository_open(&mut repo, path.as_ptr()));

    let c_name = b"HEAD\0".as_ptr() as *const c_char;
    let oid = {
        let mut oid = mem::MaybeUninit::uninit();
        check("looking up HEAD",
              raw::git_reference_name_to_id(oid.as_mut_ptr(), repo, c_name));
        oid.assume_init()
    };

    let mut commit = ptr::null_mut();
    check("looking up commit",
          raw::git_commit_lookup(&mut commit, repo, &oid));

    show_commit(commit);

    raw::git_commit_free(commit);
    raw::git_repository_free(repo);
    check("shutting down library", raw::git_libgit2_shutdown());
}
```

This code is entirely unsafe and requires manual verification of:
- Initialization order (init before use)
- Null pointer checking
- Memory management (manual free)
- String encoding (null-terminated)
- Error handling (check return codes)

The burden is entirely on the programmer.

## The Safe Wrapper: libgit2-rs-safe

Now we build a safe API over these raw bindings.

### Error Handling Across C Boundaries

C libraries use integer return codes. Rust uses `Result`. We bridge this gap:

```rust
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

impl error::Error for Error { }

pub type Result<T> = result::Result<T, Error>;
```

The `check` helper converts C return codes to Results:

```rust
fn check(code: c_int) -> Result<c_int> {
    if code >= 0 {
        return Ok(code);
    }

    unsafe {
        let error = raw::giterr_last();

        // SAFETY: libgit2 ensures that (*error).message is always non-null
        // and null-terminated, so this call is safe.
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

Now callers use `?` for error propagation instead of manual checking:

```rust
pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
    // ...
    unsafe {
        check(raw::git_repository_open(&mut repo, path.as_ptr()))?;
    }
    Ok(Repository { raw: repo })
}
```

### Resource Lifecycle Management

The safe wrapper uses RAII for automatic resource management:

```rust
pub struct Repository {
    // INVARIANT: This must always be a pointer to a live `git_repository`
    // structure. No other `Repository` may point to it.
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

The invariants are documented clearly. The `Drop` implementation enforces cleanup automatically:

```rust
{
    let repo = Repository::open(&path)?;
    // Use repo...
}  // Automatically freed here
```

No manual `free` calls. No possibility of forgetting to clean up. No double-free bugs.

### Lifetime Relationships

Resources often depend on each other. A commit depends on its repository:

```rust
pub struct Commit<'repo> {
    // INVARIANT: This must always be a pointer to a usable `git_commit`
    raw: *mut raw::git_commit,
    _marker: PhantomData<&'repo Repository>
}
```

The lifetime parameter `'repo` creates a borrow relationship:

```rust
impl Repository {
    pub fn find_commit(&self, oid: &Oid) -> Result<Commit> {
        let mut commit = ptr::null_mut();
        unsafe {
            check(raw::git_commit_lookup(&mut commit, self.raw, &oid.raw))?;
        }
        Ok(Commit { raw: commit, _marker: PhantomData })
    }
}
```

The returned `Commit` borrows from `self` (the repository). The borrow checker prevents use-after-free:

```rust
let commit = {
    let repo = Repository::open(&path)?;
    repo.find_commit(&oid)?
};  // Error! commit can't outlive repo
```

The C library would crash at runtime if we used the commit after freeing the repository. The Rust wrapper prevents this at compile time.

### Borrowed Data: Signatures

Some C data is owned by other objects. The signature is owned by the commit:

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
}
```

The signature doesn't have a `Drop` implementation—it doesn't own the data. The commit owns it. When the commit is freed, all signatures become invalid, enforced by lifetimes:

```rust
let author = commit.author();
drop(commit);
// author.name();  // Error! author can't outlive commit
```

### Null Pointer Handling

C uses null pointers to indicate absence. Rust uses `Option`:

```rust
unsafe fn char_ptr_to_str<T>(_owner: &T, ptr: *const c_char) -> Option<&str> {
    if ptr.is_null() {
        return None;
    } else {
        CStr::from_ptr(ptr).to_str().ok()
    }
}

impl<'repo> Commit<'repo> {
    pub fn message(&self) -> Option<&str> {
        unsafe {
            let message = raw::git_commit_message(self.raw);
            char_ptr_to_str(self, message)
        }
    }
}
```

This converts null → `None`, valid pointer → `Some(&str)`. Callers pattern match:

```rust
match commit.message() {
    Some(msg) => println!("{}", msg),
    None => println!("(no message)"),
}
```

No null pointer dereferences possible.

### Type Safety

Create distinct types for C types to prevent misuse:

```rust
pub struct Oid {
    pub raw: raw::git_oid
}
```

An `Oid` is not just bytes—it's a specific Git object identifier. You can't accidentally pass a random byte array where an `Oid` is expected.

### Initialization Management

libgit2 must be initialized before use and shut down at exit:

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

extern fn shutdown() {
    unsafe {
        if let Err(e) = check(raw::git_libgit2_shutdown()) {
            eprintln!("shutting down libgit2 failed: {}", e);
            std::process::abort();
        }
    }
}
```

`std::sync::Once` ensures initialization happens exactly once, even with multiple threads. The `atexit` registration ensures shutdown on program exit. Users don't think about this—it just works.

## Comparing Complexity Levels

### Level 1: Simple Invariants (ascii)

The `ascii` project has one simple invariant: bytes must be valid ASCII.

```rust
pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Ascii {
    Ascii(bytes)
}
```

The unsafe function trusts the caller. The safe API encapsulates this:

```rust
pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError> {
    if bytes.iter().any(|&byte| !byte.is_ascii()) {
        return Err(NotAsciiError(bytes));
    }
    Ok(Ascii(bytes))
}
```

**Lesson**: Even simple unsafe code needs clear contracts.

### Level 2: Pointer Manipulation (ref-with-flag)

`ref-with-flag` adds complexity: bit manipulation and PhantomData.

```rust
pub struct RefWithFlag<'a, T> {
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>
}
```

The invariants are more complex:
- T must be 2-byte aligned
- The pointer must remain valid for lifetime `'a`
- The lowest bit stores the flag

**Lesson**: Complex invariants need runtime checks where possible and clear documentation everywhere.

### Level 3: Manual Memory Management (gap-buffer)

`gap-buffer` manages uninitialized memory:

```rust
pub struct GapBuffer<T> {
    storage: Vec<T>,
    gap: Range<usize>  // Uninitialized region
}
```

Invariants:
- `storage.len()` is always 0
- Elements before `gap.start` are initialized
- Elements in `gap` are uninitialized
- Elements after `gap.end` are initialized

**Lesson**: Track initialization state carefully. Use `ptr::read`, `ptr::write`, and `drop_in_place` correctly.

### Level 4: Full FFI Integration (libgit2-rs-safe)

libgit2 integration combines all previous complexity:
- Error handling across language boundaries
- Resource lifecycle management
- Lifetime relationships between types
- Null pointer handling
- Type safety wrappers
- Thread-safe initialization

**Lesson**: FFI demands systematic thinking about every aspect of safety.

## Lessons for Building Safe FFI Wrappers

From these examples, we derive principles for FFI design:

### 1. Minimize Unsafe Surface Area

Keep unsafe code in a dedicated module. Expose only safe APIs:

```rust
mod raw {
    // All the unsafe FFI declarations
}

pub mod git {
    // All safe wrappers
}
```

Users import from `git`, never from `raw`.

### 2. Use Types to Enforce Invariants

Create wrapper types that make invalid states unrepresentable:

```rust
pub struct Repository { raw: *mut raw::git_repository }
```

No public constructor from raw pointers. The only way to get a `Repository` is through `Repository::open`, which ensures it's valid.

### 3. Leverage RAII for Resource Management

Implement `Drop` for every resource type:

```rust
impl Drop for Repository {
    fn drop(&mut self) {
        unsafe { raw::git_repository_free(self.raw); }
    }
}
```

Resources are freed automatically. Leaks and double-frees become impossible.

### 4. Express Relationships with Lifetimes

Use lifetime parameters to encode ownership:

```rust
pub struct Commit<'repo> { /* ... */ }
```

The compiler enforces that commits don't outlive repositories.

### 5. Convert at Boundaries

Convert between C and Rust types immediately at the FFI boundary:

```rust
let path = path_to_cstring(path.as_ref())?;  // Rust Path → C string
unsafe {
    check(raw::git_repository_open(&mut repo, path.as_ptr()))?;
}
Ok(Repository { raw: repo })  // C pointer → Rust type
```

Don't let C types leak into your public API.

### 6. Handle Errors Idiomatically

Convert C error codes to `Result`:

```rust
fn check(code: c_int) -> Result<c_int> { /* ... */ }
```

Users use `?` for error propagation, just like native Rust.

### 7. Document Safety Invariants

Every unsafe operation needs a SAFETY comment:

```rust
unsafe {
    // SAFETY: libgit2 ensures (*error).message is non-null and
    // null-terminated after an error
    CStr::from_ptr((*error).message)
}
```

### 8. Test Thoroughly

Test error cases, resource cleanup, and lifetime constraints:

```rust
#[test]
fn test_commit_borrows_repo() {
    let repo = Repository::open(".").unwrap();
    let oid = repo.reference_name_to_id("HEAD").unwrap();
    let commit = repo.find_commit(&oid).unwrap();
    // commit borrows repo
}
```

## Conclusion

The libgit2 integration demonstrates FFI best practices at scale:
- Raw bindings expose the C API faithfully
- Safe wrappers encapsulate all unsafe operations
- Types and lifetimes prevent misuse
- RAII ensures proper resource management
- Error handling is idiomatic Rust

By comparing `ascii` (simple), `ref-with-flag` (moderate), `gap-buffer` (complex), and `libgit2-rs-safe` (comprehensive), we see how unsafe code principles scale. The foundation is always the same: clear invariants, careful documentation, defensive programming, and encapsulation behind safe APIs.

Building safe FFI wrappers is challenging but rewarding. Done well, they provide the safety of Rust with the power of C libraries—the best of both worlds.
