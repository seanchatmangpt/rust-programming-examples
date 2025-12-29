# 26. NEWTYPE WRAPPING RAW POINTER

*A safe struct containing a dangerous raw pointer, like a glove protecting a hand from fire.*

...within a **FFI BOUNDARY** or **UNSAFE ABSTRACTION**, when you need to hold ownership of a raw pointer from C code while providing safe Rust operations...

◆ ◆ ◆

**How can you safely own and manage a raw pointer from unsafe code or FFI?**

Raw pointers are fundamentally unsafe. They can be null, dangling, or point to invalid memory. Yet when interfacing with C libraries, you must hold onto these pointers—a repository handle, a file descriptor, a connection object. If you expose the raw pointer directly, every user of your code must write unsafe blocks. A single mistake anywhere in the codebase can cause memory corruption.

But Rust's ownership system offers a solution: wrap the raw pointer in a struct. The struct becomes the safe interface—it cannot be copied accidentally, it can implement Drop to clean up resources, and it can provide safe methods that internally use unsafe code. The danger is contained within a well-defined boundary.

This pattern appears everywhere in FFI: database connections, GUI widgets, network sockets. The libgit2 bindings demonstrate it perfectly—a Repository is just a wrapped pointer, but the wrapper ensures the pointer is always valid and properly freed.

**Therefore:**

**Create a struct that contains the raw pointer as a private field, then implement Drop to free the resource and provide only safe public methods.**

```rust
/// A Git repository.
pub struct Repository {
    // This must always be a pointer to a live `git_repository` structure.
    // No other `Repository` may point to it.
    raw: *mut raw::git_repository
}

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
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
```

*The raw pointer lives inside the struct like a dangerous substance in a sealed container—isolated from direct contact, automatically cleaned up when the container is destroyed.*

◆ ◆ ◆

This pattern enables **PHANTOMDATA FOR LIFETIME** (27) when borrowed objects need lifetime tracking, and supports **CUSTOM ERROR STRUCT WITH DISPLAY** (30) for converting C error codes to Rust errors.
