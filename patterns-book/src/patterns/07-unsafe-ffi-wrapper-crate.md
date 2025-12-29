# 7. UNSAFE FFI WRAPPER CRATE

*Raw bindings to a C library, exposing foreign functions directly without safety guarantees*

...within a **SYSTEM INTEGRATION** context, when you must call existing C libraries from Rust but no safe bindings exist yet...

◆ ◆ ◆

**The challenge: C libraries offer powerful functionality that Rust programs need, but they operate outside Rust's safety guarantees. Someone must write the unsafe bridge between languages—but doing so carelessly breaks Rust's promise of memory safety.**

Rewriting mature C libraries in Rust is impractical. Libraries like libgit2 represent decades of development, edge case handling, and battle-testing. Yet C and Rust have fundamentally incompatible memory models—C uses manual allocation and raw pointers, Rust uses ownership and lifetimes. The type systems don't match: C's `int*` could be null, immutable, mutable, or garbage; Rust needs to know which.

The libgit2-rs project shows the pattern. In `src/raw.rs`, we find `extern` blocks declaring C functions with their exact signatures: raw pointers, C integers, opaque structure types. These declarations are all marked `unsafe`—every call requires an `unsafe` block because the compiler cannot verify the C code's behavior. The module uses `#[link(name = "git2")]` to tell the linker which library to load.

The structures mirror C's layout exactly, using `#[repr(C)]` to match memory representation. Opaque types like `git_repository` are declared as zero-sized types with private fields—you can't construct them in Rust, only receive pointers from C. This prevents accidentally creating invalid C structures.

**Therefore:**

**Create a dedicated module (often named `raw` or `ffi`) containing raw `extern` blocks that declare C functions and `#[repr(C)]` types. Make all foreign functions `pub unsafe`, requiring callers to explicitly acknowledge they're leaving Rust's safety guarantees. Use opaque types for C structures, never exposing their internal layout.**

```rust
#![allow(non_camel_case_types)]

use std::os::raw::{c_int, c_char, c_uchar};

#[link(name = "git2")]
extern {
    pub fn git_libgit2_init() -> c_int;
    pub fn git_libgit2_shutdown() -> c_int;
    pub fn giterr_last() -> *const git_error;

    pub fn git_repository_open(out: *mut *mut git_repository,
                               path: *const c_char) -> c_int;
    pub fn git_repository_free(repo: *mut git_repository);
}

#[repr(C)] pub struct git_repository { _private: [u8; 0] }

#[repr(C)]
pub struct git_error {
    pub message: *const c_char,
    pub klass: c_int
}

#[repr(C)]
pub struct git_oid {
    pub id: [c_uchar; GIT_OID_RAWSZ]
}
```

*The extern block acts as a boundary marker, clearly delineating where Rust's safety ends and C's manual memory management begins*

◆ ◆ ◆

This raw layer becomes the foundation for **SAFE WRAPPER AROUND UNSAFE**, which encapsulates these dangerous functions in Rust types that enforce safety through the type system. Keep this layer thin and mechanical—the real API design happens in the safe wrapper.
