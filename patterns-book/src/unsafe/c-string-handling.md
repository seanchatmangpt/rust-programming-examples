# C String Handling Pattern

## Context

You are interfacing with C code that uses null-terminated strings (`char *`), but your Rust code uses UTF-8 string slices (`&str`) or owned strings (`String`). You need to convert between these representations safely and correctly.

The `libgit2-rs` examples demonstrate this: converting Rust paths to C strings for opening repositories, and converting C string pointers from commit messages back to Rust strings.

## Problem

**How do you safely convert between Rust strings (UTF-8, length-prefixed) and C strings (null-terminated, potentially non-UTF-8) without causing memory corruption or undefined behavior?**

C and Rust have fundamentally different string representations:

| Aspect | Rust `&str` / `String` | C `char *` |
|--------|----------------------|-----------|
| **Encoding** | UTF-8 (validated) | Usually ASCII or locale-specific |
| **Termination** | Length-prefixed | Null-terminated (`\0`) |
| **Null bytes** | Allowed internally | Terminates string |
| **Memory** | Owned or borrowed | Usually heap-allocated, manual free |
| **Safety** | Memory-safe | Can point anywhere |

Incorrect handling causes:
- Buffer overruns (missing null terminator)
- Invalid UTF-8 (C string isn't valid UTF-8)
- Null bytes in middle of string
- Use-after-free (C string pointer outlives its allocation)
- Memory leaks (forgetting to free C string)

## Forces

- **Correctness**: Must not create invalid strings in either direction
- **Safety**: Must prevent buffer overruns and use-after-free
- **Performance**: Avoid unnecessary copies when possible
- **Encoding**: Handle non-UTF-8 C strings gracefully
- **Null bytes**: Rust strings can contain `\0`, C strings can't
- **Lifetimes**: Ensure pointers remain valid

These forces conflict: maximum safety requires validation and copying, but maximum performance wants zero-copy conversion.

## Solution

**Use `CString` for Rust-to-C conversion and `CStr` for C-to-Rust conversion, validate UTF-8 where necessary, and carefully manage lifetimes to prevent use-after-free.**

Follow this pattern:

1. **Rust → C**: Use `CString::new()` to create owned null-terminated string
2. **C → Rust (borrow)**: Use `CStr::from_ptr()` to borrow C string
3. **C → Rust (owned)**: Use `.to_string_lossy()` or `.to_str()` to convert
4. **Handle errors**: Check for embedded nulls (Rust→C) and invalid UTF-8 (C→Rust)
5. **Lifetime safety**: Ensure C string outlives the `CStr` borrow
6. **Free C strings**: Use `CString::from_raw()` when C code gives ownership

### Example from libgit2-rs

Converting between Rust and C strings:

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

// ========================================
// RUST → C: Creating C strings from Rust
// ========================================

fn main() {
    let path = std::env::args().skip(1).next()
        .expect("usage: git-toy PATH");

    // Convert Rust String to C string
    // This allocates a new buffer with null terminator
    let c_path = CString::new(path)
        .expect("path contains null characters");

    unsafe {
        // c_path.as_ptr() returns *const c_char
        // Valid as long as c_path is alive
        raw::git_repository_open(&mut repo, c_path.as_ptr());
    }
    // c_path dropped here - C string is freed
}

// Platform-specific path conversion
#[cfg(unix)]
fn path_to_cstring(path: &Path) -> Result<CString, NulError> {
    use std::os::unix::ffi::OsStrExt;
    // On Unix, paths are bytes (not necessarily UTF-8)
    CString::new(path.as_os_str().as_bytes())
}

#[cfg(windows)]
fn path_to_cstring(path: &Path) -> Result<CString, NulError> {
    // On Windows, convert to UTF-8 (may fail for non-UTF-8 paths)
    match path.to_str() {
        Some(s) => CString::new(s),
        None => Err(/* custom error */)
    }
}

// ========================================
// C → RUST: Converting C strings to Rust
// ========================================

unsafe fn show_commit(commit: *const raw::git_commit) {
    let author = raw::git_commit_author(commit);

    // SAFETY: libgit2 guarantees name is non-null and null-terminated
    let name = CStr::from_ptr((*author).name).to_string_lossy();
    let email = CStr::from_ptr((*author).email).to_string_lossy();
    println!("{} <{}>\n", name, email);

    // Get commit message
    let message = raw::git_commit_message(commit);
    // SAFETY: libgit2 guarantees message is non-null and null-terminated
    println!("{}", CStr::from_ptr(message).to_string_lossy());
}

// Helper: Convert C string pointer to &str with proper lifetime
unsafe fn char_ptr_to_str<T>(_owner: &T, ptr: *const c_char) -> Option<&str> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: Caller guarantees ptr is valid null-terminated string
        // that lives as long as _owner
        CStr::from_ptr(ptr)
            .to_str()   // Returns Err if not valid UTF-8
            .ok()        // Convert to Option
    }
}

// Usage example:
pub fn message(&self) -> Option<&str> {
    unsafe {
        let message = raw::git_commit_message(self.raw);
        // Lifetime of returned &str is tied to self
        char_ptr_to_str(self, message)
    }
}
```

### Conversion Methods Compared

```rust
use std::ffi::CStr;

unsafe {
    let c_str_ptr: *const c_char = /* from C */;
    let c_str = CStr::from_ptr(c_str_ptr);

    // Method 1: to_str() - Returns Result, validates UTF-8
    match c_str.to_str() {
        Ok(s) => println!("Valid UTF-8: {}", s),
        Err(e) => println!("Invalid UTF-8: {:?}", e),
    }

    // Method 2: to_string_lossy() - Always succeeds, replaces invalid UTF-8
    let s = c_str.to_string_lossy();
    println!("String (lossy): {}", s);  // Invalid bytes become �

    // Method 3: to_bytes() - Get raw bytes (includes null terminator)
    let bytes_with_null = c_str.to_bytes_with_nul();

    // Method 4: to_bytes() - Get raw bytes (without null terminator)
    let bytes = c_str.to_bytes();
}
```

### Creating CString Safely

```rust
use std::ffi::CString;

// ✅ GOOD: Check for embedded nulls
fn to_c_string(s: &str) -> Result<CString, NulError> {
    CString::new(s)  // Returns Err if s contains \0
}

// ❌ BAD: Using expect() - panics on null bytes
fn to_c_string_unsafe(s: &str) -> CString {
    CString::new(s).expect("string contained null bytes")
}

// ✅ GOOD: Handle error properly
fn open_repo(path: &str) -> Result<Repo, Error> {
    let c_path = CString::new(path)
        .map_err(|_| Error::InvalidPath)?;

    unsafe {
        // Use c_path...
    }
    Ok(repo)
}

// Creating from bytes:
let bytes = vec![b'h', b'e', b'l', b'l', b'o'];
let c_string = CString::new(bytes).unwrap();
```

## Resulting Context

### Benefits

- **Safety**: `CString` and `CStr` prevent buffer overruns
- **Validation**: Checks for embedded nulls and invalid UTF-8
- **Lifetime tracking**: Compiler ensures borrowed C strings outlive references
- **Ergonomic**: Easy conversion with `to_str()`, `to_string_lossy()`
- **Flexibility**: Can handle both valid and invalid UTF-8

### Liabilities

- **Allocation**: `CString::new()` always allocates
- **Null check overhead**: Runtime check for embedded nulls
- **UTF-8 validation**: May need to handle invalid UTF-8
- **Complexity**: Multiple types to understand (`CString`, `CStr`, `&str`, `String`)
- **Error handling**: Must handle conversion failures

### Comparison Table

| Operation | Type | Allocation | Null Check | UTF-8 Check | Lifetime |
|-----------|------|------------|------------|-------------|----------|
| `CString::new()` | Owned | Yes | Yes (returns Err) | No | 'static (owned) |
| `CStr::from_ptr()` | Borrowed | No | No (unsafe) | No | Tied to pointer |
| `.to_str()` | Borrowed | No | No | Yes (returns Err) | Tied to CStr |
| `.to_string_lossy()` | Owned (Cow) | Maybe | No | Yes (replaces invalid) | 'static if owned |
| `.to_owned()` | Owned | Yes | No | No | 'static (owned) |

## Related Patterns

- **FFI Bindings**: C strings are fundamental to C FFI
- **Safe Wrapper**: Safe wrappers handle C string conversion internally
- **Extern Block**: Declares functions taking `*const c_char`

## Known Uses

- **libgit2-rs**: Converts paths and messages between Rust and C
- **std::env::args()**: On Unix, converts C argv to Rust strings
- **std::fs**: Converts paths to C strings for system calls
- **libc**: Many functions take or return C strings
- **All FFI code**: Universal pattern for C interop

## Implementation Notes

### Rust → C Patterns

```rust
use std::ffi::CString;

// Pattern 1: Temporary C string (lives until end of scope)
let c_string = CString::new("hello").unwrap();
unsafe {
    c_function(c_string.as_ptr());
}
// c_string dropped here - safe because c_function doesn't store pointer

// Pattern 2: C function stores pointer - NEED TO KEEP ALIVE
let c_string = CString::new("hello").unwrap();
unsafe {
    c_register_callback(c_string.as_ptr());
}
std::mem::forget(c_string);  // DON'T drop it!
// Later: CString::from_raw(ptr) to reclaim and drop

// Pattern 3: C function takes ownership
let c_string = CString::new("hello").unwrap();
unsafe {
    c_take_ownership(c_string.into_raw());
}
// C code will free the string - don't drop in Rust

// Pattern 4: Inline C string literal
unsafe {
    c_function(b"hello\0".as_ptr() as *const c_char);
}
// Works for string literals, but ensure null terminator!
```

### C → Rust Patterns

```rust
use std::ffi::CStr;

// Pattern 1: Borrow C string (short-lived)
unsafe {
    let c_str = CStr::from_ptr(c_string_ptr);
    let rust_str = c_str.to_str().unwrap();
    println!("{}", rust_str);
}
// rust_str borrows from c_str, both gone at end of block

// Pattern 2: Convert to owned String
unsafe {
    let c_str = CStr::from_ptr(c_string_ptr);
    let owned = c_str.to_string_lossy().into_owned();
    // owned is String, can outlive c_str
}

// Pattern 3: Handle potential null pointer
unsafe {
    if c_string_ptr.is_null() {
        None
    } else {
        Some(CStr::from_ptr(c_string_ptr).to_str().ok())
    }
}

// Pattern 4: Take ownership from C (C code will NOT free)
unsafe {
    let c_string = CString::from_raw(c_string_ptr);
    let rust_string = c_string.into_string().unwrap();
}
// Rust will free when rust_string drops
```

### Error Handling

```rust
use std::ffi::{CString, NulError};

// Handle embedded nulls
fn to_c_string(s: &str) -> Result<CString, NulError> {
    CString::new(s)  // Err if s contains \0
}

// Handle invalid UTF-8
unsafe fn from_c_string(ptr: *const c_char) -> Result<String, Utf8Error> {
    CStr::from_ptr(ptr).to_str()?.to_owned()  // Err if not UTF-8
}

// Handle both with custom error type
#[derive(Debug)]
enum ConversionError {
    NullByte(NulError),
    InvalidUtf8(Utf8Error),
}

impl From<NulError> for ConversionError {
    fn from(e: NulError) -> Self {
        ConversionError::NullByte(e)
    }
}

impl From<Utf8Error> for ConversionError {
    fn from(e: Utf8Error) -> Self {
        ConversionError::InvalidUtf8(e)
    }
}
```

### Lossy Conversion

```rust
use std::borrow::Cow;

unsafe {
    let c_str = CStr::from_ptr(c_string_ptr);

    // Returns Cow<'_, str>
    let s: Cow<str> = c_str.to_string_lossy();

    match s {
        Cow::Borrowed(s) => {
            // Valid UTF-8, no allocation
            println!("Valid: {}", s);
        }
        Cow::Owned(s) => {
            // Invalid UTF-8 was replaced, allocated new String
            println!("Replaced: {}", s);
        }
    }
}
```

### Working with OsStr (Platform Paths)

```rust
use std::ffi::{CString, OsStr};
use std::os::unix::ffi::OsStrExt;  // Unix only

#[cfg(unix)]
fn os_str_to_c_string(s: &OsStr) -> Result<CString, NulError> {
    // On Unix, OsStr is bytes (may not be UTF-8)
    CString::new(s.as_bytes())
}

#[cfg(windows)]
fn os_str_to_c_string(s: &OsStr) -> Result<CString, NulError> {
    // On Windows, must convert to UTF-8 or use wide strings
    match s.to_str() {
        Some(s) => CString::new(s),
        None => Err(/* error */),
    }
}
```

## Testing C String Conversions

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CString, CStr};

    #[test]
    fn test_round_trip() {
        let original = "hello, world!";
        let c_string = CString::new(original).unwrap();

        unsafe {
            let back = CStr::from_ptr(c_string.as_ptr())
                .to_str()
                .unwrap();
            assert_eq!(original, back);
        }
    }

    #[test]
    fn test_embedded_null() {
        let s = "hello\0world";
        let result = CString::new(s);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_utf8() {
        let bytes = b"hello\xFF\xFEworld\0";  // Invalid UTF-8
        let c_str = unsafe { CStr::from_ptr(bytes.as_ptr() as *const c_char) };

        // to_str() fails
        assert!(c_str.to_str().is_err());

        // to_string_lossy() succeeds with replacement
        let s = c_str.to_string_lossy();
        assert!(s.contains('�'));  // Replacement character
    }

    #[test]
    fn test_lifetime_safety() {
        let c_string = CString::new("hello").unwrap();
        let c_str = unsafe { CStr::from_ptr(c_string.as_ptr()) };

        // This would be a use-after-free if we dropped c_string:
        // drop(c_string);
        // let s = c_str.to_str();  // INVALID!

        // Correct: c_str lifetime tied to c_string
        let s = c_str.to_str().unwrap();
        assert_eq!(s, "hello");
    }
}
```

## Common Pitfalls

### Pitfall 1: Forgetting Null Terminator

```rust
// ❌ WRONG: Not null-terminated
let bytes = b"hello";
unsafe {
    c_function(bytes.as_ptr() as *const c_char);
}  // C code will read past end!

// ✅ RIGHT: Use CString or literal with \0
let c_string = CString::new("hello").unwrap();
unsafe {
    c_function(c_string.as_ptr());
}

// Or for literals:
unsafe {
    c_function(b"hello\0".as_ptr() as *const c_char);
}
```

### Pitfall 2: Use-After-Free

```rust
// ❌ WRONG: c_ptr outlives c_string
let c_ptr = {
    let c_string = CString::new("hello").unwrap();
    c_string.as_ptr()
};  // c_string dropped here
unsafe {
    c_function(c_ptr);  // DANGLING POINTER!
}

// ✅ RIGHT: Keep CString alive
let c_string = CString::new("hello").unwrap();
unsafe {
    c_function(c_string.as_ptr());
}
// c_string dropped after use
```

### Pitfall 3: Missing Null Check

```rust
// ❌ WRONG: Assuming C string is non-null
unsafe {
    let c_str = CStr::from_ptr(c_ptr);  // Crashes if null!
}

// ✅ RIGHT: Check for null
unsafe {
    if c_ptr.is_null() {
        return None;
    }
    let c_str = CStr::from_ptr(c_ptr);
}
```

### Pitfall 4: Embedded Null Bytes

```rust
// ❌ WRONG: Not checking for embedded nulls
let s = format!("hello{}world", '\0');
let c_string = CString::new(s).unwrap();  // PANICS!

// ✅ RIGHT: Handle error
match CString::new(s) {
    Ok(c_string) => { /* use it */ },
    Err(e) => eprintln!("String contains null byte at position {}", e.nul_position()),
}
```

## Further Reading

- Rust std documentation: `std::ffi::CString` and `std::ffi::CStr`
- *The Rustonomicon* - "FFI" chapter
- Blog: "Rust FFI: Sending strings to the outside world" by Jake Goulding
- Rust Reference - "The C String type"
