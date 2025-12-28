# How to Create FFI Bindings to C Libraries

## Overview

This guide shows you how to create Foreign Function Interface (FFI) bindings to call C libraries from Rust. We'll use the libgit2 bindings as a concrete example.

## Prerequisites

- A C library installed on your system (we use libgit2)
- Basic understanding of C types and memory management
- Rust toolchain installed

## Why FFI Bindings?

FFI allows Rust to call functions from C libraries, giving you access to mature C ecosystems while writing Rust code. This is similar to Python's `ctypes` or `cffi`, but with compile-time type checking.

## Step 1: Set Up Your Project

Create a new Rust project and add a `build.rs` file to configure linking:

```rust
// build.rs
fn main() {
    println!(r"cargo:rustc-link-search=native=/path/to/libgit2/build");
}
```

This tells the linker where to find the C library at build time. For system libraries, you might use `pkg-config` instead.

## Step 2: Declare Opaque Types

C structs that you won't access directly should be declared as opaque types:

```rust
// Using empty enums (preferred for safety)
pub enum git_repository {}
pub enum git_commit {}

// Or using zero-sized structs with private field
#[repr(C)]
pub struct git_repository { _private: [u8; 0] }
```

The `#[repr(C)]` attribute ensures Rust uses C's memory layout. Opaque types prevent direct access to internal fields you shouldn't touch.

## Step 3: Declare C Structs You Need to Access

For structs where you need to read fields, declare the full layout:

```rust
use std::os::raw::{c_int, c_char, c_uchar};

#[repr(C)]
pub struct git_error {
    pub message: *const c_char,
    pub klass: c_int
}

#[repr(C)]
pub struct git_oid {
    pub id: [c_uchar; 20]  // GIT_OID_RAWSZ = 20
}

#[repr(C)]
pub struct git_signature {
    pub name: *const c_char,
    pub email: *const c_char,
    pub when: git_time
}
```

Key points:
- Use `std::os::raw` types (`c_int`, `c_char`, etc.) for portability
- Match C struct field order exactly
- Use `*const c_char` for C strings
- Array sizes must match C definitions

## Step 4: Declare C Functions with `extern`

Use an `extern` block to declare C functions you want to call:

```rust
#[link(name = "git2")]
extern {
    pub fn git_libgit2_init() -> c_int;
    pub fn git_libgit2_shutdown() -> c_int;
    pub fn giterr_last() -> *const git_error;

    pub fn git_repository_open(
        out: *mut *mut git_repository,
        path: *const c_char
    ) -> c_int;

    pub fn git_repository_free(repo: *mut git_repository);

    pub fn git_commit_lookup(
        out: *mut *mut git_commit,
        repo: *mut git_repository,
        id: *const git_oid
    ) -> c_int;

    pub fn git_commit_author(commit: *const git_commit) -> *const git_signature;
    pub fn git_commit_message(commit: *const git_commit) -> *const c_char;
    pub fn git_commit_free(commit: *mut git_commit);
}
```

The `#[link(name = "git2")]` tells Rust to link against `libgit2.so` (or `libgit2.dylib`/`git2.dll`).

## Step 5: Handle C Types in Rust

### Converting Rust Strings to C Strings

```rust
use std::ffi::CString;

let path = std::env::args().skip(1).next().expect("usage: program PATH");
let path = CString::new(path).expect("path contains null characters");

// Pass to C function
unsafe {
    raw::git_repository_open(&mut repo, path.as_ptr());
}
```

`CString` allocates a null-terminated string that C can use. It panics if the string contains interior null bytes.

### Converting C Strings to Rust Strings

```rust
use std::ffi::CStr;

unsafe {
    let error = &*raw::giterr_last();
    let message = CStr::from_ptr(error.message).to_string_lossy();
    println!("Error: {}", message);
}
```

`CStr::from_ptr()` creates a borrowed reference from a raw pointer. Use `to_string_lossy()` to handle invalid UTF-8.

### Using Out-Parameters

C functions often use out-parameters (pointers to return values):

```rust
use std::ptr;
use std::mem::MaybeUninit;

unsafe {
    // For pointers: initialize to null
    let mut repo = ptr::null_mut();
    check(raw::git_repository_open(&mut repo, path.as_ptr()));

    // For values: use MaybeUninit
    let oid = {
        let mut oid = MaybeUninit::uninit();
        check(raw::git_reference_name_to_id(
            oid.as_mut_ptr(), repo, c_name
        ));
        oid.assume_init()
    };
}
```

## Step 6: Handle C Error Codes

C libraries typically return integers for status codes. Wrap this in a helper:

```rust
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

// Usage
unsafe {
    check("opening repository",
          raw::git_repository_open(&mut repo, path.as_ptr()));
}
```

## Complete Example: Using the FFI

Here's a complete program that uses these bindings:

```rust
mod raw;  // The FFI declarations above

use std::ffi::{CStr, CString};
use std::ptr;

fn main() {
    let path = std::env::args().skip(1).next()
        .expect("usage: git-toy PATH");
    let path = CString::new(path).expect("path contains null");

    unsafe {
        check("initializing", raw::git_libgit2_init());

        let mut repo = ptr::null_mut();
        check("opening repository",
              raw::git_repository_open(&mut repo, path.as_ptr()));

        // Look up HEAD commit
        let c_name = b"HEAD\0".as_ptr() as *const std::os::raw::c_char;
        let oid = {
            let mut oid = std::mem::MaybeUninit::uninit();
            check("looking up HEAD",
                  raw::git_reference_name_to_id(
                      oid.as_mut_ptr(), repo, c_name
                  ));
            oid.assume_init()
        };

        let mut commit = ptr::null_mut();
        check("looking up commit",
              raw::git_commit_lookup(&mut commit, repo, &oid));

        // Display commit info
        let author = raw::git_commit_author(commit);
        let name = CStr::from_ptr((*author).name).to_string_lossy();
        let email = CStr::from_ptr((*author).email).to_string_lossy();
        println!("{} <{}>", name, email);

        let message = raw::git_commit_message(commit);
        println!("{}", CStr::from_ptr(message).to_string_lossy());

        // Clean up
        raw::git_commit_free(commit);
        raw::git_repository_free(repo);
        check("shutdown", raw::git_libgit2_shutdown());
    }
}
```

## Safety Considerations

### Why Everything is `unsafe`

All FFI calls are inherently unsafe because:
1. The compiler can't verify C's memory safety
2. C can violate Rust's aliasing rules
3. Raw pointers may be null or dangling
4. C functions may have undocumented preconditions

### Best Practices

1. **Minimize unsafe boundaries**: Create small, well-documented unsafe functions
2. **Validate pointers**: Check for null before dereferencing
3. **Match C's ownership model**: Free resources you allocate
4. **Document assumptions**: Comment on preconditions C requires
5. **Use raw types correctly**:
   - `*const T` for read-only pointers
   - `*mut T` for mutable pointers
   - Never create multiple `&mut` references to the same data

### Common Pitfalls

```rust
// ❌ WRONG: Creating references from raw pointers without validation
let error = &*raw::giterr_last();  // What if it returns null?

// ✅ BETTER: Check for null first
let error_ptr = raw::giterr_last();
if error_ptr.is_null() {
    panic!("No error info available");
}
let error = &*error_ptr;
```

## Comparison to Python's ctypes

If you're familiar with Python, here's how Rust FFI compares:

| Aspect | Python ctypes | Rust FFI |
|--------|--------------|----------|
| Type safety | Runtime | Compile-time |
| Pointer handling | Automatic | Manual but type-safe |
| String conversion | Automatic | Explicit (CString/CStr) |
| Error handling | Exceptions | Return codes or panics |
| Memory safety | GC-protected | Manual but compiler-checked |
| Performance | Overhead per call | Zero overhead |

**Python ctypes example:**
```python
from ctypes import *
libgit2 = CDLL("libgit2.so")

# Declare function signature
libgit2.git_libgit2_init.restype = c_int
libgit2.git_repository_open.argtypes = [POINTER(c_void_p), c_char_p]
libgit2.git_repository_open.restype = c_int

# Call
libgit2.git_libgit2_init()
repo = c_void_p()
path = b"/path/to/repo"
result = libgit2.git_repository_open(byref(repo), path)
```

**Rust equivalent:**
```rust
// Type-checked at compile time
extern {
    pub fn git_libgit2_init() -> c_int;
    pub fn git_repository_open(
        out: *mut *mut git_repository,
        path: *const c_char
    ) -> c_int;
}

// Usage
unsafe {
    raw::git_libgit2_init();
    let mut repo = ptr::null_mut();
    let path = CString::new("/path/to/repo").unwrap();
    raw::git_repository_open(&mut repo, path.as_ptr());
}
```

Key differences:
- Rust checks types at compile time; Python checks at runtime
- Rust requires explicit `unsafe` blocks
- Rust has better IDE support and refactoring safety
- Python is more concise but easier to get wrong

## Using Bindgen for Automatic Generation

For large C libraries, manually writing bindings is tedious. Use [`bindgen`](https://rust-lang.github.io/rust-bindgen/) to auto-generate them:

```toml
# Cargo.toml
[build-dependencies]
bindgen = "0.69"
```

```rust
// build.rs
fn main() {
    println!("cargo:rustc-link-lib=git2");

    let bindings = bindgen::Builder::default()
        .header("/usr/include/git2.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
```

Then include the generated bindings:

```rust
// In your lib.rs or raw.rs
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

## Next Steps

- See [How to Wrap FFI Safely](10-wrap-ffi-safely.md) to create safe, idiomatic Rust APIs on top of these raw bindings
- Learn about [advanced bindgen features](https://rust-lang.github.io/rust-bindgen/) for complex C APIs
- Read about [Rust's memory model](https://doc.rust-lang.org/nomicon/) in The Rustonomicon

## Related Examples

- `/home/user/rust-programming-examples/libgit2-rs/src/raw.rs` - Raw FFI declarations
- `/home/user/rust-programming-examples/libgit2-rs/src/main.rs` - Using raw FFI
- `/home/user/rust-programming-examples/libgit2-rs/build.rs` - Build configuration
