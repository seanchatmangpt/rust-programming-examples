# Building Safe Abstractions Over Unsafe Code

## The Purpose of Unsafe

A common misconception is that `unsafe` code is "bad" code. In reality, unsafe code is the foundation upon which safe abstractions are built. Every Rust program uses unsafe code—it's just hidden inside the standard library. `Vec`, `String`, `HashMap`, and other fundamental types all use unsafe internally to provide safe, high-level APIs.

The goal when writing unsafe code is not to avoid it entirely, but to encapsulate it properly. Unsafe operations should be:

1. **Localized**: Kept in small, focused functions
2. **Documented**: With clear safety invariants
3. **Encapsulated**: Behind safe APIs that users cannot misuse
4. **Auditable**: Easy to review and verify

The examples in this repository demonstrate these principles at different scales, from the simple `ascii` wrapper to the comprehensive `libgit2-rs-safe` library.

## Invariant Maintenance and SAFETY Comments

Every unsafe operation depends on invariants—conditions that must remain true for the code to be correct. Documenting these invariants is not optional; it's essential for correctness and maintainability.

### The SAFETY Comment Pattern

The convention in Rust is to precede unsafe code with a `// SAFETY:` comment explaining why the operation is safe. This serves three purposes:

1. **Documents assumptions**: Future readers (including yourself) understand what must be true
2. **Aids review**: Reviewers can verify the invariants hold
3. **Prevents misuse**: If code changes violate invariants, the comment flags the issue

From the `ascii` project:

```rust
/// Construct an `Ascii` value from `bytes`, without checking
/// whether `bytes` actually contains well-formed ASCII.
///
/// # Safety
///
/// The caller must ensure that `bytes` contains only ASCII
/// characters: bytes no greater than 0x7f. Otherwise, the effect is
/// undefined.
pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Ascii {
    Ascii(bytes)
}
```

The safety contract is clear: the caller must ensure all bytes are valid ASCII. Later, when converting to a String:

```rust
impl From<Ascii> for String {
    fn from(ascii: Ascii) -> String {
        // If this module has no bugs, this is safe, because
        // well-formed ASCII text is also well-formed UTF-8.
        unsafe { String::from_utf8_unchecked(ascii.0) }
    }
}
```

The SAFETY comment references the module's invariant. If the `Ascii` type maintains its invariant (only valid ASCII), then converting to UTF-8 is safe because ASCII is a subset of UTF-8.

### Documenting Complex Invariants

More complex unsafe code requires more detailed documentation. From `gap-buffer`:

```rust
/// Return a pointer to the `index`'th element of the underlying storage,
/// regardless of the gap.
///
/// Safety: `index` must be a valid index into `self.storage`.
unsafe fn space(&self, index: usize) -> *const T {
    self.storage.as_ptr().offset(index as isize)
}
```

And when using it:

```rust
pub fn get(&self, index: usize) -> Option<&T> {
    let raw = self.index_to_raw(index);
    if raw < self.capacity() {
        unsafe {
            // We just checked `raw` against self.capacity(),
            // and index_to_raw skips the gap, so this is safe.
            Some(&*self.space(raw))
        }
    } else {
        None
    }
}
```

The unsafe block includes a comment explaining exactly why the invariants hold. We checked the bounds, and `index_to_raw` ensures we skip the gap, so dereferencing the pointer is safe.

## RAII: Resource Acquisition Is Initialization

RAII is one of Rust's most powerful patterns for managing resources across unsafe boundaries. The idea is simple: acquire a resource in a type's constructor and release it in the destructor. This ensures resources are always cleaned up, even when errors occur.

### Automatic Resource Management

In `libgit2-rs-safe`, repositories are managed via RAII:

```rust
pub struct Repository {
    // This must always be a pointer to a live `git_repository` structure.
    // No other `Repository` may point to it.
    raw: *mut raw::git_repository
}

impl Drop for Repository {
    fn drop(&mut self) {
        unsafe {
            raw::git_repository_free(self.raw);
        }
    }
}
```

The invariants are stated clearly: `raw` must always point to a valid repository, and no other `Repository` can point to the same repository (ensuring exclusive ownership). The `Drop` implementation guarantees cleanup:

```rust
let repo = Repository::open(&path)?;
// Use repo...
// Repository automatically freed when it goes out of scope
```

Compare this to the manual management in `libgit2-rs`:

```rust
unsafe {
    let mut repo = ptr::null_mut();
    raw::git_repository_open(&mut repo, path.as_ptr());

    // ... use repo ...

    raw::git_repository_free(repo);  // Must remember to call this!
}
```

If an error occurs between opening and freeing, we leak the repository. If we forget to call `free`, we leak. If we call `free` twice, we have undefined behavior. RAII eliminates all these problems.

### Nested Resource Lifetimes

RAII becomes more powerful when resources depend on each other. In libgit2, a commit must not outlive the repository it came from. The safe wrapper encodes this with lifetimes:

```rust
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
```

The lifetime parameter `'repo` ties the commit to its repository. You cannot drop the repository while commits from it exist:

```rust
let repo = Repository::open(&path)?;
let commit = repo.find_commit(&oid)?;
// drop(repo);  // Error! Commit borrows repo
```

This is enforced at compile time. The C library would crash at runtime if we freed the repository before the commit. The safe wrapper prevents this misuse entirely.

## Safe Wrappers Over C Libraries

Building a safe wrapper requires systematic thinking about safety at every level of the API.

### Type Safety

Replace raw C types with Rust types that enforce invariants:

```rust
// C API uses raw bytes
#[repr(C)]
pub struct git_oid {
    pub id: [c_uchar; GIT_OID_RAWSZ]
}

// Rust wrapper creates a named type
pub struct Oid {
    pub raw: raw::git_oid
}
```

Now an `Oid` is distinct from any other byte array. You can't accidentally pass the wrong type to a function expecting an `Oid`.

### Null Pointer Safety

C APIs often return null to indicate absence. Rust uses `Option`:

```rust
/// Try to borrow a `&str` from `ptr`, given that `ptr` may be null or
/// refer to ill-formed UTF-8.
///
/// Safety: if `ptr` is non-null, it must point to a null-terminated C
/// string that is safe to access for at least as long as the lifetime of
/// `_owner`.
unsafe fn char_ptr_to_str<T>(_owner: &T, ptr: *const c_char) -> Option<&str> {
    if ptr.is_null() {
        return None;
    } else {
        CStr::from_ptr(ptr).to_str().ok()
    }
}
```

This helper converts C's null-or-pointer convention to Rust's `Option<&str>`. Callers use pattern matching instead of manual null checks:

```rust
pub fn message(&self) -> Option<&str> {
    unsafe {
        let message = raw::git_commit_message(self.raw);
        char_ptr_to_str(self, message)
    }
}

// Safe usage
match commit.message() {
    Some(msg) => println!("{}", msg),
    None => println!("(no message)"),
}
```

### Error Handling

C libraries use integer return codes. Rust uses `Result`:

```rust
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

This converts libgit2's error codes into idiomatic Rust errors. Callers use `?` to propagate:

```rust
pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
    // ...
    unsafe {
        check(raw::git_repository_open(&mut repo, path.as_ptr()))?;
    }
    Ok(Repository { raw: repo })
}
```

## Ownership Models for C Resources

Designing ownership models for C resources requires deciding: who owns what, and when is it freed?

### Exclusive Ownership

The `Repository` type owns its C repository pointer exclusively:

```rust
pub struct Repository {
    raw: *mut raw::git_repository
}
```

No `Clone` implementation exists—only one `Repository` can point to a given C repository. When dropped, the C resource is freed. This is the simplest model.

### Borrowed Ownership

The `Signature` type doesn't own its data; it borrows from the commit:

```rust
pub struct Signature<'text> {
    raw: *const raw::git_signature,
    _marker: PhantomData<&'text str>
}
```

The lifetime `'text` indicates this is borrowed data. We don't free it in `Drop` because the commit owns it:

```rust
impl<'repo> Commit<'repo> {
    pub fn author(&self) -> Signature {
        unsafe {
            Signature {
                raw: raw::git_commit_author(self.raw),
                _marker: PhantomData
            }
        }
    }
}
```

The signature cannot outlive the commit that produced it, enforced by the borrow checker.

### Shared Ownership

Some C resources use reference counting. For these, you might use `Arc` to share ownership:

```rust
pub struct SharedResource {
    inner: Arc<RawResource>
}

struct RawResource {
    ptr: *mut c_void
}

impl Drop for RawResource {
    fn drop(&mut self) {
        unsafe { c_release(self.ptr); }
    }
}
```

Multiple `SharedResource` instances can exist. The C resource is freed when the last one is dropped.

## Putting It Together

Building safe abstractions over unsafe code is about:

1. **Identifying invariants**: What must remain true?
2. **Enforcing them**: Use types, lifetimes, and RAII
3. **Documenting assumptions**: SAFETY comments for unsafe code
4. **Testing thoroughly**: Verify invariants hold in all cases

The result is unsafe code that is auditable, maintainable, and safe to use. Users of your API benefit from Rust's safety guarantees without needing to understand the underlying unsafe operations.
