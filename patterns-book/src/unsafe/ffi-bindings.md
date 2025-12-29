# FFI Bindings Pattern

## Context

You need to call functions from a C library in your Rust program. The library provides functionality that would be difficult or impossible to reimplement in Rust, or you need to integrate with an existing C ecosystem.

The `libgit2-rs` example demonstrates this: creating raw Rust bindings to libgit2, a C library for Git operations, exposing the C API directly to Rust code.

## Problem

**How do you create Rust bindings to C library functions while maintaining type safety and establishing clear boundaries between safe and unsafe code?**

C libraries use different conventions than Rust:
- Manual memory management with malloc/free
- Null-terminated strings instead of UTF-8 slices
- Error codes instead of Result types
- Raw pointers everywhere
- No lifetime tracking

Simply calling C functions from Rust is unsafe and error-prone. You need a systematic way to declare foreign functions and translate between C and Rust conventions.

## Forces

- **Safety**: C functions are inherently unsafe from Rust's perspective
- **Compatibility**: Must match C's ABI, calling conventions, and data layouts
- **Type correctness**: Rust and C types must correspond correctly
- **Performance**: FFI calls should have minimal overhead
- **Maintainability**: Bindings should be easy to update when C library changes
- **Usability**: Eventually want safe Rust wrappers, but raw bindings come first

These forces conflict: maximally safe code requires extensive wrapping, but raw bindings should be minimal and direct.

## Solution

**Create a raw FFI bindings module that declares C functions and types using `extern` blocks, repr(C) structs, and appropriate C-compatible types, keeping all FFI in one isolated module.**

Follow this pattern:

1. **Separate module**: Put all FFI declarations in a dedicated module (often `raw.rs`)
2. **Link to library**: Use `#[link]` attribute to specify C library
3. **Extern block**: Declare C functions in `extern` blocks
4. **C types**: Use `std::os::raw` types (`c_int`, `c_char`, etc.)
5. **repr(C)**: Mark structs with `#[repr(C)]` to match C layout
6. **Opaque types**: Use empty enums for opaque C types
7. **Raw pointers**: Use `*const T` and `*mut T` for C pointers
8. **All unsafe**: Document that all FFI functions are unsafe

### Example from libgit2-rs

Raw bindings to the libgit2 C library:

```rust
// raw.rs - Raw FFI bindings to libgit2
#![allow(non_camel_case_types)]  // C naming conventions differ

use std::os::raw::{c_int, c_char, c_uchar};

// Link against the libgit2 C library
#[link(name = "git2")]
extern {
    // Library initialization and shutdown
    pub fn git_libgit2_init() -> c_int;
    pub fn git_libgit2_shutdown() -> c_int;

    // Error handling
    pub fn giterr_last() -> *const git_error;

    // Repository operations
    pub fn git_repository_open(
        out: *mut *mut git_repository,
        path: *const c_char
    ) -> c_int;
    pub fn git_repository_free(repo: *mut git_repository);

    // Reference operations
    pub fn git_reference_name_to_id(
        out: *mut git_oid,
        repo: *mut git_repository,
        reference: *const c_char
    ) -> c_int;

    // Commit operations
    pub fn git_commit_lookup(
        out: *mut *mut git_commit,
        repo: *mut git_repository,
        id: *const git_oid
    ) -> c_int;
    pub fn git_commit_author(commit: *const git_commit) -> *const git_signature;
    pub fn git_commit_message(commit: *const git_commit) -> *const c_char;
    pub fn git_commit_free(commit: *mut git_commit);
}

// Opaque types - we never see inside these in C
// Use empty enums (zero-size, can't be instantiated)
pub enum git_repository {}
pub enum git_commit {}

// Error type - matches C struct layout
#[repr(C)]
pub struct git_error {
    pub message: *const c_char,
    pub klass: c_int
}

// Object ID (20-byte SHA-1 hash)
pub const GIT_OID_RAWSZ: usize = 20;

#[repr(C)]
pub struct git_oid {
    pub id: [c_uchar; GIT_OID_RAWSZ]
}

// Time representation
pub type git_time_t = i64;

#[repr(C)]
pub struct git_time {
    pub time: git_time_t,
    pub offset: c_int
}

// Signature (author/committer information)
#[repr(C)]
pub struct git_signature {
    pub name: *const c_char,
    pub email: *const c_char,
    pub when: git_time
}
```

Key features:
- **#[link]**: Tells linker to link against libgit2.so (or .dylib, .dll)
- **extern block**: All C functions declared in one place
- **C types**: Uses `c_int`, `c_char` instead of Rust's i32, u8
- **Opaque types**: `git_repository` and `git_commit` are empty enums (can't construct, only use pointers)
- **repr(C)**: All structs use C layout
- **Raw pointers**: All pointers are `*const` or `*mut`

### Using the Raw Bindings

```rust
// main.rs - Using raw FFI (all unsafe)
mod raw;

use std::ffi::{CStr, CString};
use std::os::raw::c_int;
use std::ptr;
use std::mem;

fn check(activity: &'static str, status: c_int) -> c_int {
    if status < 0 {
        unsafe {
            let error = &*raw::giterr_last();
            println!("error while {}: {} ({})",
                     activity,
                     CStr::from_ptr(error.message).to_string_lossy(),
                     error.klass);
            std::process::exit(1);
        }
    }
    status
}

fn main() {
    let path = std::env::args().skip(1).next()
        .expect("usage: git-toy PATH");
    let path = CString::new(path)
        .expect("path contains null characters");

    unsafe {
        // Initialize library
        check("initializing library", raw::git_libgit2_init());

        // Open repository
        let mut repo = ptr::null_mut();
        check("opening repository",
              raw::git_repository_open(&mut repo, path.as_ptr()));

        // Look up HEAD reference
        let c_name = b"HEAD\0".as_ptr() as *const c_char;
        let oid = {
            let mut oid = mem::MaybeUninit::uninit();
            check("looking up HEAD",
                  raw::git_reference_name_to_id(
                      oid.as_mut_ptr(), repo, c_name
                  ));
            oid.assume_init()
        };

        // Look up commit
        let mut commit = ptr::null_mut();
        check("looking up commit",
              raw::git_commit_lookup(&mut commit, repo, &oid));

        // Get commit author
        let author = raw::git_commit_author(commit);
        let name = CStr::from_ptr((*author).name).to_string_lossy();
        let email = CStr::from_ptr((*author).email).to_string_lossy();
        println!("{} <{}>\n", name, email);

        // Get commit message
        let message = raw::git_commit_message(commit);
        println!("{}", CStr::from_ptr(message).to_string_lossy());

        // Clean up (critical - C doesn't have RAII!)
        raw::git_commit_free(commit);
        raw::git_repository_free(repo);
        check("shutting down library", raw::git_libgit2_shutdown());
    }
}
```

## Resulting Context

### Benefits

- **Direct access**: Can call any C function from the library
- **Minimal overhead**: FFI calls are fast (just a function call)
- **Clear boundary**: All unsafe FFI isolated in one module
- **Foundation**: Raw bindings are basis for safe wrappers
- **Explicit**: Every C interaction is clearly marked unsafe

### Liabilities

- **All unsafe**: Every FFI call requires an unsafe block
- **Manual memory management**: Must call free functions correctly
- **Error-prone**: Easy to leak memory, dereference null, etc.
- **Maintenance burden**: Changes to C API require updating bindings
- **No type safety**: Compiler can't verify C function signatures
- **Poor ergonomics**: C strings, out-parameters, error codes are awkward

### Essential FFI Patterns

#### Pattern 1: Opaque Types

```rust
// For C types you never see inside (opaque pointers):
pub enum git_repository {}  // Can't construct, only use *mut git_repository

// NOT this:
pub struct git_repository { _private: [u8; 0] }  // Older pattern, less clear
```

#### Pattern 2: Out-Parameters

C functions often return values through pointer parameters:

```rust
extern {
    // C: int foo_create(foo_t **out);
    pub fn foo_create(out: *mut *mut Foo) -> c_int;
}

// Usage:
let mut foo = ptr::null_mut();
let status = foo_create(&mut foo);
// foo now points to allocated Foo (or null if failed)
```

#### Pattern 3: Const vs Mut

Match C's const-correctness:

```rust
extern {
    // C: const char* get_name(const thing_t *thing);
    pub fn get_name(thing: *const Thing) -> *const c_char;

    // C: void set_name(thing_t *thing, const char *name);
    pub fn set_name(thing: *mut Thing, name: *const c_char);
}
```

## Related Patterns

- **Safe Wrapper**: Builds safe Rust API on top of raw FFI
- **C String Handling**: Converting between C and Rust strings
- **Resource Cleanup**: Using Drop to call C free functions
- **Extern Block**: The mechanism for declaring foreign functions

## Known Uses

- **libgit2-rs**: Bindings to libgit2 (Git library)
- **libgit2-sys**: Official low-level libgit2 bindings (uses bindgen)
- **libc**: Bindings to C standard library
- **openssl-sys**: OpenSSL bindings
- **libsqlite3-sys**: SQLite bindings
- **All *-sys crates**: Convention for raw FFI bindings

## Implementation Notes

### Using bindgen

For large C libraries, use `bindgen` to auto-generate bindings:

```rust
// build.rs
fn main() {
    println!("cargo:rustc-link-lib=git2");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}
```

### C Type Mappings

```rust
use std::os::raw::*;

// C type          -> Rust type
// char            -> c_char (i8 or u8, platform-dependent)
// short           -> c_short
// int             -> c_int
// long            -> c_long
// long long       -> c_longlong
// unsigned char   -> c_uchar
// unsigned int    -> c_uint
// float           -> f32
// double          -> f64
// void*           -> *mut c_void
// size_t          -> usize
// ssize_t         -> isize
```

### Platform-Specific Bindings

```rust
#[cfg(unix)]
extern {
    pub fn unix_specific_function() -> c_int;
}

#[cfg(windows)]
extern {
    pub fn windows_specific_function() -> c_int;
}

// Or use different link names:
#[cfg_attr(target_os = "macos", link(name = "git2", kind = "dylib"))]
#[cfg_attr(target_os = "linux", link(name = "git2", kind = "dylib"))]
#[cfg_attr(target_os = "windows", link(name = "git2", kind = "static"))]
extern {
    // ...
}
```

### Function Pointer Types

```rust
// C: typedef int (*callback_t)(void *data);
pub type CallbackFn = extern "C" fn(data: *mut c_void) -> c_int;

extern {
    // C: void register_callback(callback_t cb);
    pub fn register_callback(cb: CallbackFn);
}
```

### Variadic Functions

```rust
// C: int printf(const char *format, ...);
extern {
    // Variadic args must be ... (but can't actually be called safely from Rust)
    pub fn printf(format: *const c_char, ...) -> c_int;
}

// Better: create a wrapper that uses va_list, or avoid variadic functions
```

## Testing FFI Bindings

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_init_shutdown() {
        unsafe {
            let status = git_libgit2_init();
            assert!(status >= 0, "init failed");

            let status = git_libgit2_shutdown();
            assert!(status >= 0, "shutdown failed");
        }
    }

    #[test]
    fn test_type_sizes() {
        use std::mem::size_of;

        // Verify our type definitions match C
        assert_eq!(size_of::<git_oid>(), GIT_OID_RAWSZ);

        // git_error should be two pointers plus an int
        assert!(size_of::<git_error>() >= size_of::<usize>() * 2);
    }
}
```

## Common Pitfalls

### Pitfall 1: Wrong Link Name

```rust
// ❌ WRONG: Library filename is libfoo.so, but link name is wrong
#[link(name = "libfoo")]  // Should be "foo", not "libfoo"
extern { }

// ✅ RIGHT:
#[link(name = "foo")]
extern { }
```

### Pitfall 2: Forgetting repr(C)

```rust
// ❌ WRONG: Rust can reorder fields
struct Point {
    x: c_int,
    y: c_int,
}

// ✅ RIGHT: Force C layout
#[repr(C)]
struct Point {
    x: c_int,
    y: c_int,
}
```

### Pitfall 3: Using Rust Strings

```rust
// ❌ WRONG: Passing Rust &str to C
extern {
    pub fn c_function(s: &str);  // This doesn't work!
}

// ✅ RIGHT: Use C strings
extern {
    pub fn c_function(s: *const c_char);
}
```

### Pitfall 4: Not Handling Null

```rust
unsafe {
    let ptr = c_function();
    // ❌ WRONG: Dereferencing without null check
    let value = *ptr;

    // ✅ RIGHT: Check for null
    if ptr.is_null() {
        panic!("C function returned null");
    }
    let value = *ptr;
}
```

## Further Reading

- *The Rustonomicon* - "FFI" chapter
- Rust Reference - "External blocks" section
- The `bindgen` book - Automated binding generation
- Blog: "How to call C from Rust" by Michael Gattozzi
