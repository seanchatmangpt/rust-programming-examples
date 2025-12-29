# 17. PUBLIC FACADE MODULE

*A library's public interface—polished, safe, idiomatic Rust—conceals the unsafe machinery churning beneath.*

...within a **SAFE WRAPPER AROUND C LIBRARY (22)**, when you have isolated unsafe C bindings in a **RAW BINDINGS MODULE (16)** but now need to present a safe, ergonomic API to users...

◆ ◆ ◆

**How do you build a safe, idiomatic Rust API on top of unsafe foreign function interfaces without exposing implementation details?**

Raw C bindings are necessary but hostile. They require manual memory management, explicit error checking, raw pointers, and null-terminated strings. If you expose these directly, every user of your library must write unsafe code, defeating Rust's safety guarantees. But if you hide the bindings too deeply, you create artificial barriers that prevent legitimate low-level access when needed.

The challenge is to provide both safety and usability. Users want to call `Repository::open(path)`, not `git_repository_open(&mut ptr, cstring.as_ptr())`. They expect Rust's `Result<T, E>` for errors, not C's integer return codes. They want the compiler to prevent use-after-free bugs, not runtime crashes from dangling pointers.

You must also manage resource lifetimes. C functions like `git_commit_free()` must be called exactly once per allocation, but Rust users expect automatic cleanup through `Drop`. References must not outlive the objects they point to, but C pointers have no such guarantees.

**Therefore:**

**Create a public facade module that wraps unsafe operations in safe abstractions. Use newtype wrappers for C pointers, implement `Drop` for cleanup, convert C errors to `Result`, and use lifetime parameters to enforce borrowing rules. Keep the raw module private.**

```rust
mod raw;  // Private—users never see this

use std::error;
use std::fmt;
use std::result;

// Safe error type wrapping C error details
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

// Safe wrapper for git_repository pointer
pub struct Repository {
    raw: *mut raw::git_repository
}

use std::path::Path;
use std::ptr;
use std::ffi::CString;

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
        let path = CString::new(path.as_ref().to_str().unwrap())?;
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

// Safe wrapper with lifetime tied to Repository
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
        Ok(Commit { raw: commit, _marker: PhantomData })
    }
}

impl<'repo> Drop for Commit<'repo> {
    fn drop(&mut self) {
        unsafe {
            raw::git_commit_free(self.raw);
        }
    }
}

// Convert C error codes to Rust Result
use std::os::raw::c_int;
use std::ffi::CStr;

fn check(code: c_int) -> Result<c_int> {
    if code >= 0 {
        return Ok(code);
    }
    unsafe {
        let error = raw::giterr_last();
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

*The facade module is a translator: users speak idiomatic Rust, the module speaks unsafe C, and the boundary between them is precisely defined and thoroughly checked.*

◆ ◆ ◆

This pattern concentrates all unsafe code in a controlled layer. Users interact only with safe types like `Repository` and `Commit`, never with raw pointers. The type system enforces safety: `Commit<'repo>` cannot outlive its `Repository`, and `Drop` implementations prevent resource leaks.

The facade module becomes your library's true interface. The raw module is an implementation detail, visible only to maintainers who understand the safety invariants. This separation makes auditing easier: unsafe code is concentrated, safe code is abundant, and the boundary is explicit.

Use **TYPE ALIAS FOR CUSTOM RESULT (9)** to create a convenient `Result<T>` for your error type, **PHANTOM DATA FOR LIFETIME TRACKING (25)** to enforce borrowing relationships across FFI boundaries, and **DROP FOR CLEANUP (5)** to manage C resources automatically.
