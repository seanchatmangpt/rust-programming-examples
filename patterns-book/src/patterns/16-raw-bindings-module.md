# 16. RAW BINDINGS MODULE

*A developer stares at C header files, preparing to bridge two worlds—the unsafe realm of foreign functions and the safety guarantees of Rust.*

...within a **SAFE WRAPPER AROUND C LIBRARY (22)**, when you need to call foreign C functions but must not pollute your safe API with unsafe details...

◆ ◆ ◆

**How do you declare C functions in Rust without forcing every user of your library to write unsafe code?**

When interfacing with C libraries, Rust requires you to declare foreign functions in `extern` blocks. These declarations are inherently unsafe—they involve raw pointers, null-terminated strings, manual memory management, and all the hazards that Rust was designed to prevent. If you scatter these declarations throughout your codebase, every module becomes contaminated with unsafe code.

The tension is real: you must declare these functions somewhere, but you want to minimize the surface area of unsafe code. If foreign function declarations appear in public modules alongside safe abstractions, users of your library might accidentally call the unsafe functions directly, bypassing your carefully constructed safety guarantees.

Furthermore, C naming conventions differ from Rust's. C uses snake_case for types (`git_repository`, `git_commit`), while Rust prefers `PascalCase`. C structs often have prefixes to simulate namespacing. These naming mismatches create friction and trigger Rust's linting warnings unless explicitly silenced.

**Therefore:**

**Isolate all foreign function declarations in a private `raw` module at the bottom of your module hierarchy. Use `#[allow(non_camel_case_types)]` to silence warnings about C-style naming. Mark the module private so only your safe wrapper code can access it.**

```rust
// In src/raw.rs (or as `mod raw` in your main module)
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

    pub fn git_commit_lookup(out: *mut *mut git_commit,
                             repo: *mut git_repository,
                             id: *const git_oid) -> c_int;
    pub fn git_commit_free(commit: *mut git_commit);
}

// Opaque types matching C's incomplete types
#[repr(C)] pub struct git_repository { _private: [u8; 0] }
#[repr(C)] pub struct git_commit { _private: [u8; 0] }

// C structs with known layout
#[repr(C)]
pub struct git_error {
    pub message: *const c_char,
    pub klass: c_int
}

#[repr(C)]
pub struct git_oid {
    pub id: [c_uchar; 20]
}
```

*The raw module forms a firewall: unsafe C declarations on one side, safe Rust abstractions on the other. Only your wrapper code crosses this boundary.*

◆ ◆ ◆

This pattern creates a clear separation of concerns. The raw module contains all the unsafe machinery—function declarations, raw pointer types, C-compatible struct layouts—while the rest of your codebase builds safe abstractions on top. When reading your code, the presence of `raw::` prefixes signals "here be dragons" and marks the precise locations where safety invariants must be carefully maintained.

Use **OPAQUE TYPE WITH ZERO-SIZED ARRAY (23)** for C types whose layout you don't control, **REPR C FOR FFI STRUCTS (24)** for structs you pass across the FFI boundary, and **PUBLIC FACADE MODULE (17)** to build the safe API that users actually interact with.
