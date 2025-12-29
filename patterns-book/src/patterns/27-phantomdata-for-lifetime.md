# 27. PHANTOMDATA FOR LIFETIME

*A ghost field that enforces borrowing rules without occupying memory, like a legal covenant that restricts property use without physical presence.*

...within a **NEWTYPE WRAPPING RAW POINTER** (26), when you need to ensure a borrowed object cannot outlive its owner but don't actually store a reference...

◆ ◆ ◆

**How can you prevent a borrowed object from outliving its source when you only have raw pointers?**

Consider a Git commit borrowed from a repository. The commit's C pointer is only valid while the repository exists. If you store just the commit pointer, Rust cannot prevent this danger:

```rust
let commit = {
    let repo = Repository::open(".")?;
    repo.find_commit(&oid)?
}; // repo is dropped
commit.message(); // DANGER: using freed memory
```

You cannot store an actual `&'repo Repository` reference in Commit—the C API doesn't need it, and storing it wastes memory. But without any connection to the repository's lifetime, Rust allows the repository to be freed while commits still point into it.

PhantomData solves this paradox. It's a zero-sized type that exists only at compile time. By adding `PhantomData<&'repo Repository>` to the Commit struct, you tell Rust's borrow checker "this Commit behaves as if it borrows the Repository" without actually storing a reference. The compiler enforces lifetime rules, preventing use-after-free, but no runtime memory is consumed.

This pattern appears in any FFI wrapper where borrowed objects must not outlive their parents: database queries tied to connections, UI widgets tied to windows, slices of buffers.

**Therefore:**

**Add a PhantomData field with the lifetime you need to enforce, making the struct act as if it borrows without actually storing a reference.**

```rust
use std::marker::PhantomData;

pub struct Commit<'repo> {
    // This must always be a pointer to a usable `git_commit` structure.
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
```

*The PhantomData acts like an invisible tether—it takes no space, but the compiler treats it as if the commit holds a reference to the repository, preventing the repository from being dropped too early.*

◆ ◆ ◆

This pattern builds on **NEWTYPE WRAPPING RAW POINTER** (26) and enables safe borrowing hierarchies in **GENERIC STRUCT WITH TYPE PARAMETER** (18).
