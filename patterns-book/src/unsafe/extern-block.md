# Extern Block Pattern

## Context

You need to declare foreign functions from a C library or system API so they can be called from Rust code. These functions use a different calling convention (Application Binary Interface - ABI) than Rust, typically the C ABI.

The `libgit2-rs` bindings demonstrate this: declaring dozens of libgit2 C functions using `extern` blocks, specifying the library to link against, and matching C function signatures precisely.

## Problem

**How do you correctly declare external C functions with appropriate types, calling conventions, and linkage so they can be safely called from Rust?**

Calling foreign functions requires:
- Matching the exact C function signature (types, parameter order, return type)
- Using the correct calling convention (usually "C")
- Linking to the correct library (static or dynamic)
- Using C-compatible types (not Rust types like String or Vec)
- Handling platform differences (Windows vs Unix)

Incorrect declarations lead to:
- Segmentation faults
- Stack corruption
- Wrong values passed/returned
- Link-time errors

The challenge is to accurately represent C interfaces in Rust syntax.

## Forces

- **Correctness**: Function signatures must match C exactly
- **Type safety**: Use Rust's type system where possible
- **Linkage**: Must correctly specify library name and type
- **Portability**: Handle platform-specific variations
- **Maintainability**: Keep declarations organized and documented
- **ABI compatibility**: Ensure calling conventions match

These forces conflict: maximum type safety wants Rust types, but C compatibility demands C types.

## Solution

**Use `extern` blocks to declare foreign functions with C-compatible types, specify the library with `#[link]`, use the "C" calling convention (default), and organize all declarations in a dedicated module.**

Follow this pattern:

1. **Dedicated module**: Put all extern declarations in one module (e.g., `raw.rs`)
2. **Link attribute**: Specify the C library to link with `#[link(name = "...")]`
3. **Extern block**: Declare functions inside `extern { }` or `extern "C" { }`
4. **C types**: Use `std::os::raw` types (`c_int`, `c_char`, etc.)
5. **Function signatures**: Match C signatures exactly
6. **Naming**: Allow non-Rust naming with `#![allow(non_camel_case_types)]`
7. **Documentation**: Comment each function's purpose

### Example from libgit2-rs

Declaring external C functions:

```rust
// raw.rs
#![allow(non_camel_case_types)]  // C uses different naming

use std::os::raw::{c_int, c_char, c_uchar};

// Link to the libgit2 C library
// The linker will look for libgit2.so, libgit2.dylib, or git2.dll
#[link(name = "git2")]
extern {
    // Library initialization
    // C: int git_libgit2_init(void);
    pub fn git_libgit2_init() -> c_int;

    // C: int git_libgit2_shutdown(void);
    pub fn git_libgit2_shutdown() -> c_int;

    // Error handling
    // C: const git_error *giterr_last(void);
    pub fn giterr_last() -> *const git_error;

    // Repository operations
    // C: int git_repository_open(git_repository **out, const char *path);
    pub fn git_repository_open(
        out: *mut *mut git_repository,
        path: *const c_char
    ) -> c_int;

    // C: void git_repository_free(git_repository *repo);
    pub fn git_repository_free(repo: *mut git_repository);

    // Reference operations
    // C: int git_reference_name_to_id(
    //        git_oid *out,
    //        git_repository *repo,
    //        const char *reference
    //    );
    pub fn git_reference_name_to_id(
        out: *mut git_oid,
        repo: *mut git_repository,
        reference: *const c_char
    ) -> c_int;

    // Commit operations
    // C: int git_commit_lookup(
    //        git_commit **out,
    //        git_repository *repo,
    //        const git_oid *id
    //    );
    pub fn git_commit_lookup(
        out: *mut *mut git_commit,
        repo: *mut git_repository,
        id: *const git_oid
    ) -> c_int;

    // C: const git_signature *git_commit_author(const git_commit *commit);
    pub fn git_commit_author(
        commit: *const git_commit
    ) -> *const git_signature;

    // C: const char *git_commit_message(const git_commit *commit);
    pub fn git_commit_message(commit: *const git_commit) -> *const c_char;

    // C: void git_commit_free(git_commit *commit);
    pub fn git_commit_free(commit: *mut git_commit);
}

// Opaque types (never see their internals in Rust)
pub enum git_repository {}
pub enum git_commit {}

// Struct with C layout
#[repr(C)]
pub struct git_error {
    pub message: *const c_char,
    pub klass: c_int
}

#[repr(C)]
pub struct git_oid {
    pub id: [c_uchar; 20]  // 20-byte SHA-1
}

#[repr(C)]
pub struct git_signature {
    pub name: *const c_char,
    pub email: *const c_char,
    pub when: git_time
}

pub type git_time_t = i64;

#[repr(C)]
pub struct git_time {
    pub time: git_time_t,
    pub offset: c_int
}
```

Key features:
- **#[link(name = "git2")]**: Links to libgit2
- **extern block**: All C functions in one place
- **C types**: `c_int` instead of `i32`, `c_char` instead of `i8`
- **Pointer types**: `*const` and `*mut` for C pointers
- **Opaque types**: Empty enums for types we never construct
- **repr(C)**: Structs use C memory layout

### Platform-Specific Extern Blocks

```rust
// Unix-specific functions
#[cfg(unix)]
#[link(name = "pthread")]
extern {
    pub fn pthread_create(
        thread: *mut pthread_t,
        attr: *const pthread_attr_t,
        start_routine: extern "C" fn(*mut c_void) -> *mut c_void,
        arg: *mut c_void
    ) -> c_int;
}

// Windows-specific functions
#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {  // Use Windows calling convention
    pub fn CreateThread(
        lpThreadAttributes: LPSECURITY_ATTRIBUTES,
        dwStackSize: SIZE_T,
        lpStartAddress: LPTHREAD_START_ROUTINE,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: LPDWORD
    ) -> HANDLE;
}
```

### Calling Conventions

```rust
// Default: C calling convention
extern "C" {
    fn c_function();
}

// Explicit (same as default)
extern {
    fn c_function();
}

// Windows system calling convention (stdcall)
extern "system" {
    fn win_api_function();
}

// Rust calling convention (for Rust -> Rust FFI)
extern "Rust" {
    fn rust_function();
}
```

## Resulting Context

### Benefits

- **Type checking**: Rust verifies argument types at call sites
- **Organization**: All FFI in one place for easy auditing
- **Linkage**: Automatic library linking by Rust toolchain
- **Documentation**: Function signatures serve as documentation
- **Safety boundary**: Clear distinction between safe and unsafe code

### Liabilities

- **All unsafe**: Every call requires `unsafe` block
- **Manual maintenance**: Must update if C API changes
- **No verification**: Compiler can't verify signatures match C
- **Platform complexity**: May need multiple versions for different OSes
- **Linkage issues**: Library must be available at link time

### Declaration Patterns

#### Pattern 1: Simple Function

```rust
// C: int foo(void);
extern {
    pub fn foo() -> c_int;
}
```

#### Pattern 2: Function with Parameters

```rust
// C: int bar(int x, const char *s);
extern {
    pub fn bar(x: c_int, s: *const c_char) -> c_int;
}
```

#### Pattern 3: Out-Parameter

```rust
// C: int create_thing(thing_t **out);
extern {
    pub fn create_thing(out: *mut *mut Thing) -> c_int;
}
```

#### Pattern 4: Callback Function

```rust
// C: typedef void (*callback_t)(int);
pub type Callback = extern "C" fn(c_int);

// C: void register_callback(callback_t cb);
extern {
    pub fn register_callback(cb: Callback);
}
```

#### Pattern 5: Variadic Function (Rare)

```rust
// C: int printf(const char *format, ...);
extern {
    pub fn printf(format: *const c_char, ...) -> c_int;
}

// Note: Calling variadic functions from Rust is unsafe and limited
// Usually better to create a wrapper in C
```

## Related Patterns

- **FFI Bindings**: Extern blocks are the core of FFI bindings
- **Safe Wrapper**: Builds on extern blocks to create safe APIs
- **C String Handling**: Necessary when calling functions with string parameters

## Known Uses

- **libgit2-rs**: Declares libgit2 C API
- **libc**: Declares standard C library functions
- **winapi**: Windows API declarations
- **openssl-sys**: OpenSSL function declarations
- **libsqlite3-sys**: SQLite3 function declarations
- Every **-sys** crate on crates.io

## Implementation Notes

### C Type Mappings Reference

```rust
use std::os::raw::*;

// Integer types
c_char      // char (may be signed or unsigned, platform-dependent)
c_schar     // signed char
c_uchar     // unsigned char
c_short     // short
c_ushort    // unsigned short
c_int       // int
c_uint      // unsigned int
c_long      // long
c_ulong     // unsigned long
c_longlong  // long long
c_ulonglong // unsigned long long

// Floating point
// f32         // float (use directly)
// f64         // double (use directly)

// Other
c_void      // void (use as *mut c_void or *const c_void)
// usize       // size_t (use directly)
// isize       // ssize_t (use directly)
```

### Struct Layout

```rust
// Must match C struct layout exactly
#[repr(C)]
pub struct Point {
    pub x: c_int,
    pub y: c_int,
}

// Packed struct (no padding)
#[repr(C, packed)]
pub struct PackedData {
    pub flag: c_char,
    pub value: c_int,
}

// Struct with specific alignment
#[repr(C, align(16))]
pub struct AlignedData {
    pub data: [u8; 16],
}
```

### Link Attribute Variations

```rust
// Dynamic library (default)
#[link(name = "mylib")]
extern { }

// Static library
#[link(name = "mylib", kind = "static")]
extern { }

// Framework (macOS)
#[link(name = "CoreFoundation", kind = "framework")]
extern { }

// Platform-specific
#[cfg_attr(target_os = "macos", link(name = "git2", kind = "dylib"))]
#[cfg_attr(target_os = "linux", link(name = "git2"))]
#[cfg_attr(target_os = "windows", link(name = "git2", kind = "static"))]
extern { }
```

### Function Pointers

```rust
// C: typedef int (*comparator_t)(const void *, const void *);
pub type Comparator = extern "C" fn(*const c_void, *const c_void) -> c_int;

// C: void qsort(void *base, size_t nmemb, size_t size, comparator_t compar);
extern {
    pub fn qsort(
        base: *mut c_void,
        nmemb: usize,
        size: usize,
        compar: Comparator
    );
}
```

### Using bindgen for Automation

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

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

// In your Rust code:
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

## Testing Extern Blocks

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_basic_call() {
        unsafe {
            let result = git_libgit2_init();
            assert!(result >= 0, "Initialization failed");

            let result = git_libgit2_shutdown();
            assert!(result >= 0, "Shutdown failed");
        }
    }

    #[test]
    fn test_type_sizes() {
        use std::mem::size_of;

        // Verify struct sizes match expectations
        assert_eq!(size_of::<git_oid>(), 20);
    }
}
```

## Common Pitfalls

### Pitfall 1: Wrong Type

```rust
// ❌ WRONG: Using Rust types instead of C types
extern {
    pub fn c_function(x: i32) -> i32;  // Should be c_int!
}

// ✅ RIGHT: Use C types
extern {
    pub fn c_function(x: c_int) -> c_int;
}
```

### Pitfall 2: Missing repr(C)

```rust
// ❌ WRONG: Rust can reorder fields
pub struct Point {
    x: c_int,
    y: c_int,
}

extern {
    pub fn process_point(p: *const Point);
}

// ✅ RIGHT: Use C layout
#[repr(C)]
pub struct Point {
    x: c_int,
    y: c_int,
}
```

### Pitfall 3: Wrong Link Name

```rust
// ❌ WRONG: Library file is libfoo.so, but link name includes "lib"
#[link(name = "libfoo")]  // Should be "foo"!
extern { }

// ✅ RIGHT: Use library base name
#[link(name = "foo")]
extern { }
```

### Pitfall 4: Wrong Calling Convention

```rust
// ❌ WRONG: Windows API function with C convention
#[cfg(windows)]
extern "C" {  // Should be "system"!
    pub fn CreateFileW(...) -> HANDLE;
}

// ✅ RIGHT: Use correct calling convention
#[cfg(windows)]
extern "system" {
    pub fn CreateFileW(...) -> HANDLE;
}
```

### Pitfall 5: Forgetting to Mark Unsafe

```rust
// This is implicitly unsafe, but you must mark calls as unsafe:
unsafe {
    git_libgit2_init();  // Must be in unsafe block
}
```

## Documentation Best Practices

```rust
/// Initialize the libgit2 library.
///
/// This must be called before using any other libgit2 functions.
/// It is safe to call this multiple times; it's reference counted.
///
/// # Safety
///
/// This function is unsafe because it may modify global state.
/// It should typically be called once at program startup.
///
/// # Returns
///
/// The number of active initialization calls, or a negative error code.
///
/// # C Documentation
///
/// See: https://libgit2.org/libgit2/#HEAD/group/libgit2/git_libgit2_init
pub fn git_libgit2_init() -> c_int;
```

## Further Reading

- Rust Reference - "External blocks" section
- *The Rustonomicon* - "FFI" chapter
- `bindgen` documentation - Automated binding generation
- Blog: "The Rust FFI Guide" by Jake Goulding
