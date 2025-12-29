# 8. SAFE WRAPPER AROUND UNSAFE

*Rust types that encapsulate unsafe operations, exposing a safe API through ownership, lifetimes, and Drop implementations*

...within a **UNSAFE FFI WRAPPER CRATE**, when raw C bindings are available but you want Rust users to work safely without writing `unsafe` blocks...

◆ ◆ ◆

**The essential problem: The raw FFI layer makes C functions available but offers no protection—null pointers, use-after-free, and data races are all possible. Rust programs need the functionality without sacrificing memory safety. How do you bridge unsafe C into safe Rust?**

Simply exposing the raw bindings forces every caller to write `unsafe` code, scattering safety concerns throughout the codebase. This defeats Rust's core value proposition. Worse, it pushes the burden of understanding C's memory model onto every user—they must know when to free resources, which pointers might be null, which functions require initialization first.

The libgit2-rs-safe project demonstrates the solution. It wraps raw pointers in Rust structs that own their resources. A `Repository` struct contains a `*mut git_repository` but makes it private, ensuring only the wrapper accesses it. The wrapper's `Drop` implementation calls the C cleanup function when the Rust value goes out of scope—automatic resource management through RAII.

For borrowed data like commit messages, the wrapper uses lifetime parameters. A `Commit<'repo>` can't outlive the `Repository` it came from because the lifetime parameter expresses this dependency in the type system. The borrow checker enforces that you can't free the repository while commits are still alive.

Error handling translates C return codes into Rust `Result` types. The `check()` helper function examines integer return codes, fetching detailed error messages from C and converting them into Rust error types. Now error handling uses `?` and pattern matching instead of manual checks.

**Therefore:**

**Create public Rust types that privately hold raw pointers from the FFI layer. Implement Drop to automatically free C resources. Use PhantomData and lifetime parameters to encode borrowing relationships. Convert C return codes to Result types. Hide all `unsafe` blocks inside these wrapper implementations, exposing only safe public methods.**

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
}

impl Drop for Repository {
    fn drop(&mut self) {
        unsafe {
            raw::git_repository_free(self.raw);
        }
    }
}

pub struct Commit<'repo> {
    raw: *mut raw::git_commit,
    _marker: PhantomData<&'repo Repository>
}

impl<'repo> Commit<'repo> {
    pub fn message(&self) -> Option<&str> {
        unsafe {
            let message = raw::git_commit_message(self.raw);
            char_ptr_to_str(self, message)
        }
    }
}

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

*The safe wrapper acts as a protective membrane, allowing safe Rust code to call through to unsafe C functions while maintaining all of Rust's safety invariants*

◆ ◆ ◆

With the safe wrapper complete, users can work entirely in safe Rust, using **RESOURCE CLEANUP** through Drop, **ERROR PROPAGATION** through Result, and normal ownership patterns—never writing an `unsafe` block while still accessing C library functionality.
