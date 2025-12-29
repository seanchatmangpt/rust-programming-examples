# FFI Deep Dive & C Interoperability Internals

Foreign Function Interface (FFI) represents one of Rust's most powerful yet dangerous capabilities. When done correctly, FFI enables seamless integration with decades of existing C libraries while maintaining Rust's safety guarantees at the boundary. When done incorrectly, it becomes a vector for undefined behavior, memory corruption, and security vulnerabilities.

## 1. FFI Fundamentals

### Extern Blocks and Function Declarations

The `extern` block declares foreign functions available from dynamically or statically linked libraries. Every function declared in an `extern` block is inherently `unsafe` to call, as Rust cannot verify the C code's correctness.

```rust
use std::os::raw::{c_int, c_char};

#[link(name = "git2")]
extern {
    pub fn git_libgit2_init() -> c_int;
    pub fn git_repository_open(
        out: *mut *mut git_repository,
        path: *const c_char
    ) -> c_int;
    pub fn git_repository_free(repo: *mut git_repository);
}
```

**Critical Details:**

1. **`#[link(name = "git2")]`**: Instructs the linker to link against `libgit2.so` (Linux), `libgit2.dylib` (macOS), or `git2.dll` (Windows)
2. **Function signatures must exactly match C**: Incorrect signatures lead to undefined behavior, not compile errors
3. **All calls require `unsafe`**: The compiler cannot verify C code maintains Rust's invariants

### Calling Conventions

Different platforms and compilers use different calling conventions that determine how arguments are passed and stack frames are managed:

```rust
// Default C calling convention (platform-dependent)
extern "C" fn callback(data: *mut c_void) -> c_int {
    // Function body
    0
}

// Explicit calling conventions
extern "cdecl" fn cdecl_func() {}      // C default on many platforms
extern "stdcall" fn stdcall_func() {}  // Windows API standard
extern "fastcall" fn fastcall_func() {} // Register-based args
extern "system" fn system_func() {}    // Platform's system ABI
```

**When to specify:**
- Use `extern "C"` for maximum portability
- Use `extern "system"` for Windows API functions
- Incorrect convention causes stack corruption (hard to debug)

### ABI Compatibility and Platform Specifics

The Application Binary Interface (ABI) defines low-level details like struct layout, calling conventions, and name mangling. Rust's ABI is **unstable** and **incompatible** with C unless explicitly specified.

```rust
// INCOMPATIBLE with C - Rust's internal layout
pub struct RustStruct {
    field1: i32,
    field2: u64,
}

// COMPATIBLE with C - guaranteed layout
#[repr(C)]
pub struct CCompatibleStruct {
    field1: i32,
    field2: u64,
}
```

**Platform-specific considerations from libgit2-rs-safe:**

```rust
#[cfg(unix)]
fn path_to_cstring(path: &Path) -> Result<CString> {
    use std::os::unix::ffi::OsStrExt;
    Ok(CString::new(path.as_os_str().as_bytes())?)
}

#[cfg(windows)]
fn path_to_cstring(path: &Path) -> Result<CString> {
    // Windows paths may contain non-UTF-8 characters
    match path.to_str() {
        Some(s) => Ok(CString::new(s)?),
        None => Err("Path not UTF-8".into()),
    }
}
```

### Unsafe FFI Calls and Preconditions

Every FFI call has preconditions that must be maintained by the caller. The `SAFETY` comment documents these invariants:

```rust
fn check(activity: &'static str, status: c_int) -> c_int {
    if status < 0 {
        unsafe {
            // SAFETY: libgit2 guarantees giterr_last() returns a valid pointer
            // to a git_error struct with a non-null, null-terminated message.
            // This pointer is valid until the next libgit2 call.
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
```

**Key preconditions:**
- Pointer validity (not null, properly aligned, points to valid memory)
- Lifetime constraints (pointer valid for duration of access)
- Thread safety (function is reentrant/thread-safe)
- Initialization requirements (library initialized before use)

### Return Value Handling

C functions typically return error codes or null pointers. Rust should convert these to idiomatic `Result` types:

```rust
use std::os::raw::c_int;
use std::ffi::CStr;

fn check(code: c_int) -> Result<c_int> {
    if code >= 0 {
        return Ok(code);
    }

    unsafe {
        let error = raw::giterr_last();

        // SAFETY: libgit2 ensures (*error).message is always non-null
        // and null-terminated
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

## 2. Type Mapping: Rust to C

### Primitive Type Equivalences

Never assume Rust primitives match C types directly. Use `std::os::raw` for guaranteed compatibility:

```rust
use std::os::raw::{
    c_char,    // char in C (usually i8 or u8)
    c_int,     // int in C (usually i32, but not guaranteed)
    c_uint,    // unsigned int
    c_long,    // long (i32 on Windows, i64 on Unix 64-bit)
    c_uchar,   // unsigned char
    c_void,    // void (opaque type)
};

// WRONG - assumes platform details
fn bad_binding(x: i32) -> i32 { /* ... */ }

// CORRECT - uses guaranteed C types
fn good_binding(x: c_int) -> c_int { /* ... */ }
```

**Size guarantees:**

| Rust Type | C Equivalent | Guaranteed Size | Notes |
|-----------|--------------|-----------------|-------|
| `i8` / `u8` | `int8_t` / `uint8_t` | 1 byte | Safe to use directly |
| `i16` / `u16` | `int16_t` / `uint16_t` | 2 bytes | Safe to use directly |
| `i32` / `u32` | `int32_t` / `uint32_t` | 4 bytes | Safe to use directly |
| `i64` / `u64` | `int64_t` / `uint64_t` | 8 bytes | Safe to use directly |
| `c_int` | `int` | **Platform-dependent** | Use for C `int` |
| `c_long` | `long` | **Platform-dependent** | Use for C `long` |
| `usize` | `size_t` | **Platform-dependent** | Match pointer size |

### Pointer Equivalences and Differences

Rust pointers have strict aliasing rules that C pointers do not:

```rust
// Immutable pointer: *const T
// - Can create multiple *const T to same data
// - Cannot mutate through *const T (even if T is not const)
let x: i32 = 42;
let ptr: *const i32 = &x;

// Mutable pointer: *mut T
// - Only one *mut T should exist to same data (aliasing UB)
// - Can mutate through *mut T
let mut y: i32 = 10;
let ptr_mut: *mut i32 = &mut y;

unsafe {
    *ptr_mut = 20;  // OK - mutation through *mut
    // let val = *ptr;  // OK - read through *const
}
```

**Critical difference from C:**
- C allows arbitrary pointer aliasing
- Rust assumes `*mut T` pointers don't alias (for optimization)
- Violating this assumption is undefined behavior

### String Representation

C strings are null-terminated byte arrays. Rust strings are UTF-8 with explicit length:

```rust
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

// Rust → C: CString owns the data
let path = std::env::args().skip(1).next()
    .expect("usage: program PATH");
let path_c = CString::new(path)
    .expect("path contains null bytes");

unsafe {
    // path_c.as_ptr() returns *const c_char valid while path_c lives
    some_c_function(path_c.as_ptr());
}
// path_c dropped here - C must not retain pointer!

// C → Rust: CStr borrows the data
unsafe {
    let c_message: *const c_char = git_commit_message(commit);

    // SAFETY: libgit2 ensures c_message is null-terminated and
    // valid for the lifetime of `commit`
    let message: &str = CStr::from_ptr(c_message)
        .to_str()
        .expect("message not UTF-8");
}
```

**Common pitfalls:**

1. **Interior nulls**: `CString::new("hello\0world")` fails - C strings can't contain embedded nulls
2. **Dangling pointers**: C code retaining `as_ptr()` after `CString` drops
3. **Encoding mismatch**: C strings may not be UTF-8 (use `to_string_lossy()`)

### Array Handling

C arrays decay to pointers. Rust arrays have known size:

```rust
// Rust fixed-size array
let rust_array: [u8; 20] = [0; 20];

// C expects pointer + length
extern "C" {
    fn process_array(data: *const u8, len: usize);
}

unsafe {
    process_array(rust_array.as_ptr(), rust_array.len());
}

// From libgit2-rs: git_oid is exactly 20 bytes
pub const GIT_OID_RAWSZ: usize = 20;

#[repr(C)]
pub struct git_oid {
    pub id: [c_uchar; GIT_OID_RAWSZ]
}
```

**Slices vs pointers:**
```rust
// Rust slice: fat pointer (pointer + length)
fn process_slice(data: &[u8]) {
    // data.len() and data.as_ptr() available
}

// C array: thin pointer only
extern "C" {
    fn process_c_array(data: *const u8, len: usize);
}
```

### Struct and Union Mapping

Without `#[repr(C)]`, Rust reorders fields for optimization:

```rust
// Rust layout: compiler may reorder fields
struct RustLayout {
    a: u8,   // Compiler might place at offset 4
    b: u32,  // Compiler might place at offset 0
    c: u8,   // Compiler might place at offset 5
}
// Size: 6 bytes (optimized), but layout unpredictable

// C-compatible layout: fields in declaration order
#[repr(C)]
struct CLayout {
    a: u8,   // Offset 0
    // 3 bytes padding
    b: u32,  // Offset 4 (aligned to 4 bytes)
    c: u8,   // Offset 8
    // 3 bytes padding
}
// Size: 12 bytes (with padding for alignment)
```

**Real-world example from libgit2-rs:**

```rust
#[repr(C)]
pub struct git_signature {
    pub name: *const c_char,
    pub email: *const c_char,
    pub when: git_time
}

#[repr(C)]
pub struct git_time {
    pub time: git_time_t,
    pub offset: c_int
}
```

### Using libc Crate for C Type Definitions

The `libc` crate provides portable type definitions:

```rust
use libc::{size_t, ssize_t, c_void, pthread_t, FILE};

extern "C" {
    fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t;
    fn atexit(callback: extern "C" fn()) -> c_int;
}

// From libgit2-rs-safe: using libc::atexit for cleanup
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

extern "C" fn shutdown() {
    unsafe {
        if let Err(e) = check(raw::git_libgit2_shutdown()) {
            eprintln!("shutting down libgit2 failed: {}", e);
            std::process::abort();
        }
    }
}
```

## 3. Pointer Safety in FFI

### Null Pointer Checks Before Dereferencing

Always check for null before dereferencing C pointers:

```rust
unsafe fn char_ptr_to_str<T>(_owner: &T, ptr: *const c_char) -> Option<&str> {
    if ptr.is_null() {
        return None;
    }

    // SAFETY: Caller guarantees non-null ptr is valid, null-terminated
    CStr::from_ptr(ptr).to_str().ok()
}
```

**Why this matters:**
- C functions often return null on error
- Dereferencing null is undefined behavior
- UB can lead to security vulnerabilities (CVE-level)

### Lifetime Requirements for C Pointers

C pointers have no lifetime tracking. Rust wrappers must encode lifetime relationships:

```rust
pub struct Signature<'text> {
    raw: *const raw::git_signature,
    _marker: PhantomData<&'text str>
}

impl<'text> Signature<'text> {
    pub fn name(&self) -> Option<&str> {
        unsafe {
            // SAFETY: Signature borrows from commit, so this pointer
            // is valid for 'text lifetime
            char_ptr_to_str(self, (*self.raw).name)
        }
    }
}
```

**PhantomData enforces:**
- `Signature` cannot outlive the data it references
- Compiler tracks lifetime 'text automatically
- No runtime cost (zero-sized type)

### Memory Ownership Across FFI Boundary

Critical question: **Who owns the memory?**

```rust
// Pattern 1: C owns memory, Rust borrows
pub struct Commit<'repo> {
    raw: *mut raw::git_commit,
    _marker: PhantomData<&'repo Repository>
}

impl<'repo> Commit<'repo> {
    // Returns borrowed data - C still owns it
    pub fn author(&self) -> Signature {
        unsafe {
            Signature {
                raw: raw::git_commit_author(self.raw),
                _marker: PhantomData
            }
        }
    }
}

// Pattern 2: Rust owns memory, must free via C function
impl<'repo> Drop for Commit<'repo> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: self.raw is a valid git_commit pointer
            // that we own, and we must free it via git_commit_free
            raw::git_commit_free(self.raw);
        }
    }
}
```

### Dangling Pointer Prevention

The borrow checker prevents most dangling pointers, but FFI can bypass it:

```rust
// DANGEROUS: C function might retain pointer
let path = CString::new("/tmp/repo")?;
unsafe {
    git_repository_open(&mut repo, path.as_ptr());
}
// path dropped here - if C retained the pointer, it's now dangling!

// SAFE: Ensure C copies the data
let path = CString::new("/tmp/repo")?;
unsafe {
    // libgit2 copies path internally, doesn't retain pointer
    git_repository_open(&mut repo, path.as_ptr());
}
// Safe to drop path
```

**Verification strategy:**
1. Read C library documentation
2. Check if function copies or retains pointers
3. If retains, keep Rust owner alive (move into struct, use Box::leak, etc.)

### Using Box to Transfer Ownership

`Box` allocates on heap and can transfer ownership to C:

```rust
#[repr(C)]
struct CallbackData {
    counter: i32,
    name: *const c_char,
}

extern "C" fn callback(data: *mut c_void) -> c_int {
    unsafe {
        let data = &mut *(data as *mut CallbackData);
        data.counter += 1;
        println!("Callback called {} times", data.counter);
        0
    }
}

fn register_callback() {
    let data = Box::new(CallbackData {
        counter: 0,
        name: std::ptr::null(),
    });

    let data_ptr = Box::into_raw(data);  // Transfer ownership to C

    unsafe {
        register_c_callback(callback, data_ptr as *mut c_void);
    }
}

// Later, in cleanup callback:
extern "C" fn cleanup(data: *mut c_void) {
    unsafe {
        // SAFETY: Take back ownership and drop
        let _data = Box::from_raw(data as *mut CallbackData);
    }
}
```

## 4. String Handling Across FFI

### CStr and CString Wrapper Types

```rust
use std::ffi::{CStr, CString};

// CString: Owned, heap-allocated, null-terminated
let owned = CString::new("hello").unwrap();
let ptr: *const c_char = owned.as_ptr();  // Valid while owned lives

// CStr: Borrowed, null-terminated
let borrowed: &CStr = &owned;
let str_slice: &str = borrowed.to_str().unwrap();
```

### Null Terminator Requirements

C strings **require** null terminator. Rust strings **forbid** interior nulls:

```rust
// Valid Rust string, invalid C string
let rust_str = "hello\0world";
let result = CString::new(rust_str);
assert!(result.is_err());  // NulError: interior null byte

// Remove nulls before converting
let sanitized = rust_str.replace('\0', "");
let c_string = CString::new(sanitized).unwrap();
```

### Encoding Assumptions

C strings have **no encoding**. They're byte arrays. Rust strings are **UTF-8**.

```rust
// Safe conversion (validates UTF-8)
let c_str: &CStr = unsafe { CStr::from_ptr(c_ptr) };
match c_str.to_str() {
    Ok(s) => println!("Valid UTF-8: {}", s),
    Err(_) => println!("Not UTF-8"),
}

// Lossy conversion (replaces invalid UTF-8 with �)
let string = c_str.to_string_lossy();
println!("Lossy: {}", string);  // May contain �

// From libgit2-rs-safe:
let message = CStr::from_ptr((*error).message)
    .to_string_lossy()
    .into_owned();
```

### C String to Rust String Conversions

```rust
// Owned conversion (allocates new String)
let owned: String = c_str.to_string_lossy().into_owned();

// Borrowed conversion (zero-copy if valid UTF-8)
let borrowed: &str = c_str.to_str()?;

// Manual conversion (maximum control)
let bytes: &[u8] = c_str.to_bytes();
let string = std::str::from_utf8(bytes)?;
```

### Performance Implications

**CString creation**: Allocates and copies
**CStr::from_ptr**: Zero-cost (just wraps pointer)
**to_str()**: Validates UTF-8 (O(n) scan)
**to_string_lossy()**: May allocate replacement String

**Optimization:**
```rust
// SLOW: Validates UTF-8 every call
fn get_name(&self) -> Option<&str> {
    unsafe {
        let ptr = self.get_name_ptr();
        if ptr.is_null() { return None; }
        CStr::from_ptr(ptr).to_str().ok()
    }
}

// FAST: Cache validated result
pub struct CachedName {
    ptr: *const c_char,
    cached: Option<String>,
}

impl CachedName {
    fn get(&mut self) -> Option<&str> {
        if self.cached.is_none() && !self.ptr.is_null() {
            unsafe {
                self.cached = Some(
                    CStr::from_ptr(self.ptr).to_string_lossy().into_owned()
                );
            }
        }
        self.cached.as_deref()
    }
}
```

## 5. Memory Layout & repr(C)

### C Struct Layout Rules

C compilers insert padding to satisfy alignment requirements:

```rust
// Example C struct
struct Example {
    char a;      // 1 byte at offset 0
    // 3 bytes padding
    int b;       // 4 bytes at offset 4
    char c;      // 1 byte at offset 8
    // 3 bytes padding
};  // Total size: 12 bytes

// Rust equivalent
#[repr(C)]
struct Example {
    a: u8,
    b: i32,
    c: u8,
}
assert_eq!(std::mem::size_of::<Example>(), 12);
```

### Rust Struct Layout Differences

Without `#[repr(C)]`, Rust optimizes layout:

```rust
struct Optimized {
    a: u8,   // Might be at offset 8
    b: i32,  // Might be at offset 0
    c: u8,   // Might be at offset 4
}
// Rust might pack this to 6 bytes with clever ordering
assert_eq!(std::mem::size_of::<Optimized>(), 8);  // Or 6, or 12!
```

### Field Ordering and Padding Implications

Order matters for cache performance and size:

```rust
// BAD: 24 bytes due to padding
#[repr(C)]
struct BadOrder {
    a: u8,    // 1 byte
    // 7 bytes padding
    b: u64,   // 8 bytes
    c: u8,    // 1 byte
    // 7 bytes padding
}

// GOOD: 16 bytes (optimal packing)
#[repr(C)]
struct GoodOrder {
    b: u64,   // 8 bytes
    a: u8,    // 1 byte
    c: u8,    // 1 byte
    // 6 bytes padding
}
```

### Union Handling in FFI

Rust unions require `Copy` types and manual initialization tracking:

```rust
#[repr(C)]
union CUnion {
    int_val: c_int,
    float_val: f32,
    ptr_val: *mut c_void,
}

// C API might use tag for discriminant
#[repr(C)]
struct TaggedUnion {
    tag: c_int,
    data: CUnion,
}

impl TaggedUnion {
    fn get_int(&self) -> Option<c_int> {
        if self.tag == TAG_INT {
            Some(unsafe { self.data.int_val })
        } else {
            None
        }
    }
}
```

### Size Mismatches Between Rust and C

Always verify sizes match:

```rust
#[repr(C)]
struct GitOid {
    id: [u8; 20],
}

// At compile time
const _: () = assert!(std::mem::size_of::<GitOid>() == 20);

// Or use build.rs for runtime verification
#[cfg(test)]
mod ffi_tests {
    use super::*;

    #[test]
    fn verify_layout() {
        assert_eq!(
            std::mem::size_of::<git_signature>(),
            std::mem::size_of::<usize>() * 2 + 16
        );
    }
}
```

## 6. Callbacks & Function Pointers

### Function Pointers as C Callbacks

Only `extern "C" fn` can be passed to C:

```rust
// Correct: extern "C" fn
extern "C" fn my_callback(data: *mut c_void) -> c_int {
    println!("Callback invoked");
    0
}

// Wrong: regular fn (different ABI)
fn wrong_callback(data: *mut c_void) -> c_int {
    0
}

extern "C" {
    fn register_callback(cb: extern "C" fn(*mut c_void) -> c_int);
}

unsafe {
    register_callback(my_callback);  // OK
    // register_callback(wrong_callback);  // Compilation error
}
```

### Extern fn Types and Their Properties

```rust
type Callback = extern "C" fn(*mut c_void) -> c_int;

// Properties:
// 1. Can be null (use Option<Callback>)
// 2. Cannot capture environment (no closures)
// 3. Must have explicit lifetime if returns references
// 4. Panic across FFI is undefined behavior

extern "C" fn safe_callback(data: *mut c_void) -> c_int {
    // Catch panics to prevent unwinding into C
    let result = std::panic::catch_unwind(|| {
        // Callback logic here
        0
    });

    match result {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Panic in callback!");
            -1  // Return error code to C
        }
    }
}
```

### Closures vs Function Pointers

Closures **cannot** be passed to C directly:

```rust
let captured = 42;

// WRONG: Closure captures environment
let closure = |data: *mut c_void| -> c_int {
    println!("Captured: {}", captured);
    0
};
// Cannot pass closure to C function

// WORKAROUND: Use void* to pass data
extern "C" fn trampoline(data: *mut c_void) -> c_int {
    unsafe {
        let captured = &*(data as *const i32);
        println!("Captured: {}", captured);
        0
    }
}

let captured = Box::new(42);
unsafe {
    register_callback_with_data(
        trampoline,
        Box::into_raw(captured) as *mut c_void
    );
}
```

### Lifetime Requirements in Callbacks

```rust
// Callback returns reference - needs lifetime bound
extern "C" fn get_name<'a>(
    ctx: *mut c_void
) -> *const c_char {
    unsafe {
        let ctx = &*(ctx as *const Context);
        ctx.name.as_ptr()
    }
}

// Context must outlive all callback invocations
struct Context {
    name: CString,
}
```

### State Passing in Callbacks

The `void*` pattern for passing state:

```rust
struct CallbackContext {
    counter: i32,
    results: Vec<String>,
}

extern "C" fn process_item(
    item: *const c_char,
    user_data: *mut c_void
) -> c_int {
    unsafe {
        let ctx = &mut *(user_data as *mut CallbackContext);
        ctx.counter += 1;

        if !item.is_null() {
            let item_str = CStr::from_ptr(item).to_string_lossy();
            ctx.results.push(item_str.into_owned());
        }

        0  // Continue iteration
    }
}

fn iterate_with_callback() {
    let mut ctx = CallbackContext {
        counter: 0,
        results: Vec::new(),
    };

    unsafe {
        c_iterate_items(
            process_item,
            &mut ctx as *mut _ as *mut c_void
        );
    }

    println!("Processed {} items", ctx.counter);
}
```

## 7. Ownership & Cleanup

### Who Owns Memory Returned from C

**Rule 1: Check documentation**
**Rule 2: If unclear, assume C owns it (don't free)**

```rust
// libgit2 owns this - don't free
let author: *const git_signature = git_commit_author(commit);

// Caller owns this - must free
let repo: *mut git_repository = /* allocated by git_repository_open */;
// Must later call git_repository_free(repo)
```

### Allocation in Rust, Deallocation in C (Danger)

**Never** allocate in Rust and free in C (or vice versa) - different allocators!

```rust
// WRONG - undefined behavior
let data = Box::new([0u8; 100]);
let ptr = Box::into_raw(data);
unsafe {
    c_function_that_calls_free(ptr as *mut c_void);  // UB!
}

// CORRECT - use C allocator
unsafe {
    let ptr = libc::malloc(100) as *mut u8;
    c_function_that_calls_free(ptr as *mut c_void);  // OK
}
```

### Allocation in C, Deallocation in Rust

Same principle applies:

```rust
// WRONG
let ptr = c_function_that_allocates();
let _boxed = unsafe { Box::from_raw(ptr) };  // UB!

// CORRECT
let ptr = c_function_that_allocates();
unsafe {
    c_function_that_frees(ptr);  // Use C's free
}
```

### Resource Cleanup Patterns

Use RAII with Drop:

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

// Automatic cleanup when Repository goes out of scope
fn example() -> Result<()> {
    let repo = Repository::open("path")?;
    // Use repo...
}  // repo automatically freed here
```

### Custom Allocators for FFI

When you need matching allocators:

```rust
use std::alloc::{alloc, dealloc, Layout};

#[no_mangle]
pub extern "C" fn rust_alloc(size: usize) -> *mut c_void {
    if size == 0 {
        return std::ptr::null_mut();
    }

    unsafe {
        let layout = Layout::from_size_align_unchecked(size, 1);
        alloc(layout) as *mut c_void
    }
}

#[no_mangle]
pub extern "C" fn rust_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        // Must know size - store it alongside allocation
        // or use a different strategy
        dealloc(ptr as *mut u8, Layout::from_size_align_unchecked(1, 1));
    }
}
```

## 8. Data Race Prevention in FFI

### Send/Sync Boundaries at FFI

Raw pointers are neither `Send` nor `Sync` by default. Wrapper types must explicitly implement these:

```rust
// WRONG - raw pointer is not Send
pub struct UnsafeWrapper {
    ptr: *mut git_repository,  // Not Send or Sync
}

// CORRECT - explicit Send/Sync with safety comments
pub struct Repository {
    raw: *mut raw::git_repository
}

// SAFETY: libgit2 repository handles are thread-safe
// (per libgit2 documentation, multiple threads can safely
// access different repository handles)
unsafe impl Send for Repository {}

// SAFETY: libgit2 requires external synchronization for
// concurrent access to the same repository handle
// (so we do NOT implement Sync)
// unsafe impl Sync for Repository {}
```

### Thread-Safety Across C Libraries

Not all C libraries are thread-safe:

```rust
// Non-thread-safe C library
static LIBRARY_LOCK: Mutex<()> = Mutex::new(());

pub fn call_non_threadsafe_function() {
    let _guard = LIBRARY_LOCK.lock().unwrap();
    unsafe {
        non_threadsafe_c_function();
    }
}
```

### Mutex Requirements for Shared State

```rust
use std::sync::Mutex;

pub struct ThreadSafeWrapper {
    inner: Mutex<*mut raw::git_repository>,
}

impl ThreadSafeWrapper {
    pub fn do_operation(&self) -> Result<()> {
        let repo = self.inner.lock().unwrap();
        unsafe {
            check(raw::git_repository_operation(*repo))?;
        }
        Ok(())
    }
}

unsafe impl Send for ThreadSafeWrapper {}
unsafe impl Sync for ThreadSafeWrapper {}
```

### Race Conditions in C Code

Even if Rust is race-free, C code might have races:

```rust
// C library has internal static state
static mut GLOBAL_COUNTER: c_int = 0;

extern "C" {
    fn increment_counter() -> c_int;  // Modifies GLOBAL_COUNTER
}

// Must synchronize access
static C_LIBRARY_MUTEX: Mutex<()> = Mutex::new(());

fn safe_increment() -> c_int {
    let _guard = C_LIBRARY_MUTEX.lock().unwrap();
    unsafe { increment_counter() }
}
```

### Static FFI State and Synchronization

```rust
fn ensure_initialized() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        unsafe {
            check(raw::git_libgit2_init())
                .expect("initializing libgit2 failed");
        }
    });
}
```

## 9. Error Handling Across FFI

### Interpreting C Error Codes

C functions typically return:
- **0 or positive**: Success (sometimes return value)
- **Negative**: Error code
- **NULL pointer**: Error

```rust
fn check(code: c_int) -> Result<c_int> {
    if code >= 0 {
        return Ok(code);
    }

    // Get detailed error from C library
    unsafe {
        let error = raw::giterr_last();
        let message = CStr::from_ptr((*error).message)
            .to_string_lossy()
            .into_owned();

        Err(Error { code: code as i32, message, class: (*error).klass as i32 })
    }
}
```

### Exception-like Behavior in C

Some C libraries use setjmp/longjmp for error handling. **This is incompatible with Rust unwinding:**

```rust
// DANGER: C library uses longjmp
extern "C" {
    fn c_function_that_longjmps();
}

// This is undefined behavior if c_function_that_longjmps actually jumps
fn dangerous() {
    let _guard = SomeDropGuard::new();
    unsafe {
        c_function_that_longjmps();  // UB if it jumps - _guard not dropped!
    }
}

// SOLUTION: Document and avoid, or use separate process
```

### Returning Result from FFI Functions

Convert C errors to Rust Result:

```rust
impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
        ensure_initialized();

        let path = path_to_cstring(path.as_ref())?;
        let mut repo = ptr::null_mut();

        unsafe {
            check(raw::git_repository_open(&mut repo, path.as_ptr()))?;
        }

        Ok(Repository { raw: repo })
    }
}
```

### Propagating Errors Back to C

When Rust code is called from C:

```rust
#[no_mangle]
pub extern "C" fn rust_process_data(
    data: *const u8,
    len: usize,
    out_result: *mut *mut ProcessedData,
    out_error: *mut *mut c_char
) -> c_int {
    // Catch panics
    let result = std::panic::catch_unwind(|| {
        if data.is_null() || out_result.is_null() {
            return -1;  // Invalid argument
        }

        let slice = unsafe { std::slice::from_raw_parts(data, len) };

        match process_data_internal(slice) {
            Ok(processed) => {
                unsafe {
                    *out_result = Box::into_raw(Box::new(processed));
                }
                0  // Success
            }
            Err(e) => {
                if !out_error.is_null() {
                    let error_msg = CString::new(e.to_string()).unwrap();
                    unsafe {
                        *out_error = error_msg.into_raw();
                    }
                }
                -2  // Processing error
            }
        }
    });

    result.unwrap_or(-3)  // Panic error code
}
```

### Panic Safety at FFI Boundary

**Never** panic across FFI boundary:

```rust
extern "C" fn callback(data: *mut c_void) -> c_int {
    let result = std::panic::catch_unwind(|| {
        // Code that might panic
        do_work(data)
    });

    match result {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Panic caught in FFI callback");
            -1  // Error code
        }
    }
}
```

## 10. Advanced FFI Patterns

### Opaque Pointers (void* Pattern)

From libgit2-rs, opaque types prevent direct access:

```rust
// Version 1: Zero-sized struct (can't be instantiated)
#[repr(C)]
pub struct git_repository {
    _private: [u8; 0]
}

// Version 2: Empty enum (more idiomatic)
pub enum git_repository {}
pub enum git_commit {}

// Both prevent: let x = git_repository { ... };
// Can only work with pointers: *mut git_repository
```

### Vtable-based Inheritance in C

C "objects" with function pointers:

```rust
#[repr(C)]
struct CVtable {
    destroy: extern "C" fn(*mut c_void),
    process: extern "C" fn(*mut c_void, *const u8, usize) -> c_int,
}

#[repr(C)]
struct CObject {
    vtable: *const CVtable,
    data: *mut c_void,
}

impl CObject {
    fn destroy(&self) {
        unsafe {
            ((*self.vtable).destroy)(self.data);
        }
    }

    fn process(&self, input: &[u8]) -> c_int {
        unsafe {
            ((*self.vtable).process)(self.data, input.as_ptr(), input.len())
        }
    }
}
```

### C++ Class Interop

Rust can call C++ through `extern "C"` wrappers or `cxx` crate:

```rust
// Manual C wrapper approach
// In C++ wrapper:
// extern "C" MyClass* MyClass_new() { return new MyClass(); }
// extern "C" void MyClass_delete(MyClass* ptr) { delete ptr; }
// extern "C" int MyClass_method(MyClass* ptr, int arg) { return ptr->method(arg); }

extern "C" {
    fn MyClass_new() -> *mut c_void;
    fn MyClass_delete(ptr: *mut c_void);
    fn MyClass_method(ptr: *mut c_void, arg: c_int) -> c_int;
}

pub struct MyClass {
    ptr: *mut c_void,
}

impl MyClass {
    pub fn new() -> Self {
        MyClass {
            ptr: unsafe { MyClass_new() }
        }
    }

    pub fn method(&self, arg: i32) -> i32 {
        unsafe { MyClass_method(self.ptr, arg as c_int) as i32 }
    }
}

impl Drop for MyClass {
    fn drop(&mut self) {
        unsafe { MyClass_delete(self.ptr); }
    }
}
```

### Custom Drop for C Resources

Always pair acquire with release:

```rust
pub struct Commit<'repo> {
    raw: *mut raw::git_commit,
    _marker: PhantomData<&'repo Repository>
}

impl<'repo> Drop for Commit<'repo> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: self.raw is a valid git_commit pointer
            // allocated by git_commit_lookup
            raw::git_commit_free(self.raw);
        }
    }
}
```

### FFI-based Wrapper Libraries

Complete example from libgit2-rs-safe:

```rust
pub struct Repository {
    raw: *mut raw::git_repository
}

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
        ensure_initialized();
        let path = path_to_cstring(path.as_ref())?;
        let mut repo = ptr::null_mut();

        unsafe {
            check(raw::git_repository_open(&mut repo, path.as_ptr()))?;
        }

        Ok(Repository { raw: repo })
    }

    pub fn find_commit(&self, oid: &Oid) -> Result<Commit> {
        let mut commit = ptr::null_mut();

        unsafe {
            check(raw::git_commit_lookup(&mut commit, self.raw, &oid.raw))?;
        }

        Ok(Commit { raw: commit, _marker: PhantomData })
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

## 11. Testing & Verification

### Property-based Testing Across FFI

Use `proptest` to verify invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn ffi_string_roundtrip(s in "\\PC*") {
        // Property: Any valid Rust string (no nulls) can roundtrip
        if !s.contains('\0') {
            let c_string = CString::new(s.clone()).unwrap();
            let recovered = unsafe {
                CStr::from_ptr(c_string.as_ptr())
            }.to_str().unwrap();
            prop_assert_eq!(&s, recovered);
        }
    }
}
```

### Fuzzing C-Rust Boundary

Use `cargo-fuzz` to find edge cases:

```rust
// fuzz/fuzz_targets/ffi_parse.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes through FFI
    let _ = std::panic::catch_unwind(|| {
        unsafe {
            parse_ffi_data(data.as_ptr(), data.len());
        }
    });
});
```

### Memory Sanitizers (ASAN, MSAN)

Build with sanitizers to detect bugs:

```bash
# AddressSanitizer - detects memory errors
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test

# MemorySanitizer - detects uninitialized reads
RUSTFLAGS="-Z sanitizer=memory" cargo +nightly test

# ThreadSanitizer - detects data races
RUSTFLAGS="-Z sanitizer=thread" cargo +nightly test
```

### Valgrind for C Memory Issues

```bash
cargo build
valgrind --leak-check=full --show-leak-kinds=all \
    ./target/debug/libgit2-rs /path/to/repo

# Look for:
# - "definitely lost" (memory leaks)
# - "invalid read/write" (use-after-free, bounds violations)
# - "conditional jump depends on uninitialised value"
```

### CI Testing with Multiple Platforms

```yaml
# .github/workflows/ffi-test.yml
name: FFI Tests

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v2

      - name: Install libgit2
        run: |
          if [ "$RUNNER_OS" == "Linux" ]; then
            sudo apt-get install -y libgit2-dev
          elif [ "$RUNNER_OS" == "macOS" ]; then
            brew install libgit2
          fi
        shell: bash

      - name: Run tests
        run: cargo test --verbose

      - name: Run with sanitizer (Linux only)
        if: runner.os == 'Linux'
        run: |
          RUSTFLAGS="-Z sanitizer=address" cargo +nightly test
```

## 12. Performance & Optimization

### FFI Call Overhead

Each FFI call has overhead:
- **Function call**: ~10-20 CPU cycles
- **Context switch**: None (same process)
- **Data marshalling**: Depends on conversions

Measure:

```rust
use std::time::Instant;

fn benchmark_ffi_calls() {
    let iterations = 1_000_000;

    let start = Instant::now();
    for _ in 0..iterations {
        unsafe {
            lightweight_c_function();
        }
    }
    let elapsed = start.elapsed();

    println!("FFI call overhead: {:?} per call", elapsed / iterations);
}
```

### Batching FFI Calls

```rust
// SLOW: Many FFI calls
for item in items {
    unsafe {
        process_one_item(item);
    }
}

// FAST: Batch FFI call
unsafe {
    process_items_batch(items.as_ptr(), items.len());
}
```

### Buffer Strategies to Minimize Calls

```rust
// SLOW: One char at a time
fn read_slow(fd: c_int) -> String {
    let mut result = String::new();
    loop {
        let mut ch = 0u8;
        unsafe {
            if libc::read(fd, &mut ch as *mut u8 as *mut c_void, 1) != 1 {
                break;
            }
        }
        result.push(ch as char);
    }
    result
}

// FAST: Buffered reads
fn read_fast(fd: c_int) -> String {
    let mut result = Vec::new();
    let mut buffer = [0u8; 4096];

    loop {
        let n = unsafe {
            libc::read(fd, buffer.as_mut_ptr() as *mut c_void, buffer.len())
        };

        if n <= 0 { break; }
        result.extend_from_slice(&buffer[..n as usize]);
    }

    String::from_utf8_lossy(&result).into_owned()
}
```

### Inline Hints for FFI Functions

```rust
// Prevents inlining (default for extern functions)
extern "C" {
    fn heavyweight_function();
}

// Wrapper can be inlined
#[inline]
pub fn call_heavyweight() {
    unsafe { heavyweight_function(); }
}
```

### When to Use FFI vs Native Rust

**Use FFI when:**
- Mature C library exists (OpenSSL, libgit2, SQLite)
- Performance-critical C code (optimized over decades)
- Platform integration required (Windows API, POSIX)
- Ecosystem compatibility needed

**Avoid FFI when:**
- Pure Rust alternative exists and is mature
- Safety is paramount (crypto primitives - use `ring` not OpenSSL)
- Cross-platform support needed (Rust handles this better)
- Development velocity matters (FFI is slower to iterate)

## 13. Security in FFI

### Input Validation Before C Calls

**Never trust C code to validate:**

```rust
pub fn open_repository(path: &str) -> Result<Repository> {
    // VALIDATE FIRST
    if path.len() > 4096 {
        return Err("Path too long".into());
    }

    if path.contains('\0') {
        return Err("Path contains null byte".into());
    }

    // Canonicalize to prevent traversal
    let canonical = std::fs::canonicalize(path)?;

    // NOW safe to pass to C
    let path_c = CString::new(canonical.to_str().unwrap())?;

    unsafe {
        // C code receives validated input
        check(raw::git_repository_open(&mut repo, path_c.as_ptr()))?;
    }

    Ok(Repository { raw: repo })
}
```

### Buffer Overflow Prevention

```rust
// VULNERABLE: No bounds check
fn vulnerable_copy(src: &[u8]) {
    let mut dest = [0u8; 100];
    unsafe {
        std::ptr::copy_nonoverlapping(
            src.as_ptr(),
            dest.as_mut_ptr(),
            src.len()  // DANGER: src.len() might be > 100
        );
    }
}

// SAFE: Explicit bounds check
fn safe_copy(src: &[u8]) -> Result<[u8; 100]> {
    if src.len() > 100 {
        return Err("Source too large".into());
    }

    let mut dest = [0u8; 100];
    dest[..src.len()].copy_from_slice(src);
    Ok(dest)
}
```

### Integer Overflow in C APIs

```rust
// C API expects size as uint32_t
extern "C" {
    fn allocate_buffer(size: u32) -> *mut u8;
}

fn safe_allocate(size: usize) -> Result<*mut u8> {
    // Check for overflow when converting usize -> u32
    let size_u32 = u32::try_from(size)
        .map_err(|_| "Size too large for C API")?;

    Ok(unsafe { allocate_buffer(size_u32) })
}
```

### Heap Exhaustion Attacks

```rust
// VULNERABLE: Attacker controls allocation size
fn vulnerable(untrusted_size: usize) {
    unsafe {
        let ptr = libc::malloc(untrusted_size);
        // OOM if untrusted_size is huge
    }
}

// SAFE: Limit maximum allocation
const MAX_ALLOCATION: usize = 100 * 1024 * 1024;  // 100MB

fn safe_allocate(requested_size: usize) -> Result<*mut u8> {
    if requested_size > MAX_ALLOCATION {
        return Err("Allocation too large".into());
    }

    unsafe {
        let ptr = libc::malloc(requested_size);
        if ptr.is_null() {
            return Err("Allocation failed".into());
        }
        Ok(ptr as *mut u8)
    }
}
```

### Secure Cleanup of Secrets

```rust
use zeroize::Zeroize;

pub struct SecretKey {
    data: Vec<u8>,
}

impl SecretKey {
    pub fn new(key: Vec<u8>) -> Self {
        SecretKey { data: key }
    }

    pub fn use_with_c_api(&self) {
        unsafe {
            c_crypto_function(self.data.as_ptr(), self.data.len());
        }
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        // Zero memory before deallocation
        self.data.zeroize();
    }
}
```

## 14. Real-World Examples

### libgit2 Integration Patterns

The complete pattern from libgit2-rs-safe demonstrates best practices:

1. **Opaque types** for C structures
2. **Lifetime tracking** with PhantomData
3. **RAII cleanup** with Drop
4. **Error conversion** from C codes to Result
5. **Safe initialization** with Once
6. **Platform-specific** path handling

```rust
// Complete safe wrapper pattern
pub struct Repository {
    raw: *mut raw::git_repository
}

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
        ensure_initialized();
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
        unsafe { raw::git_repository_free(self.raw); }
    }
}
```

### Analyzing Unsafe FFI Code

When reviewing FFI code, verify:

1. **Null checks** before dereferencing
2. **Lifetime relationships** are sound
3. **Memory ownership** is clear
4. **Error handling** is complete
5. **Thread safety** is documented
6. **Platform differences** are handled
7. **Resource cleanup** is guaranteed
8. **Panic safety** at boundaries

## Conclusion

FFI is Rust's bridge to the vast ecosystem of C libraries, but it requires rigorous discipline. The key principles:

1. **Minimize unsafe code** - keep it isolated in small, well-documented functions
2. **Validate at boundaries** - never trust C code to validate inputs
3. **Encode invariants in types** - use lifetimes, PhantomData, and RAII
4. **Test extensively** - use sanitizers, fuzzers, and property-based tests
5. **Document thoroughly** - every unsafe block needs a SAFETY comment

The libgit2-rs examples in this repository demonstrate the progression from raw, unsafe FFI bindings to safe, idiomatic Rust wrappers. Study both to understand the full spectrum of FFI patterns.

When in doubt, favor safety over performance, and remember: undefined behavior in Rust is not a compile error—it's a time bomb waiting to explode in production.
