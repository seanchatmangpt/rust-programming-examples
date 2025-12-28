# FFI & Macros API Reference

This reference provides complete API documentation for Foreign Function Interface (FFI) bindings and procedural macros.

## libgit2-rs - Raw FFI Bindings

Unsafe, low-level bindings to the libgit2 C library demonstrating raw FFI.

### External Functions

All functions are declared in an `extern` block and link to the `git2` library.

**Link Declaration:**
```rust
#[link(name = "git2")]
extern { /* ... */ }
```

---

#### Library Initialization

##### `git_libgit2_init`

Initializes the libgit2 library.

**Signature:**
```rust
pub fn git_libgit2_init() -> c_int
```

**Returns:**
- `c_int`: Number of initializations (≥0 on success, <0 on error)

**Safety:**
- Must be called before using any other libgit2 functions
- Thread-safe (uses internal reference counting)

---

##### `git_libgit2_shutdown`

Shuts down the libgit2 library.

**Signature:**
```rust
pub fn git_libgit2_shutdown() -> c_int
```

**Returns:**
- `c_int`: Number of remaining initializations (≥0 on success, <0 on error)

**Safety:**
- Decrements reference count
- Last shutdown releases resources

---

#### Error Handling

##### `giterr_last`

Returns the last error that occurred in the current thread.

**Signature:**
```rust
pub fn giterr_last() -> *const git_error
```

**Returns:**
- `*const git_error`: Pointer to error struct, or null if no error

**Safety:**
- Returned pointer is valid until next libgit2 call
- Must not be freed by caller
- May return null

---

#### Repository Operations

##### `git_repository_open`

Opens a Git repository.

**Signature:**
```rust
pub fn git_repository_open(
    out: *mut *mut git_repository,
    path: *const c_char
) -> c_int
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `out` | `*mut *mut git_repository` | Receives repository handle |
| `path` | `*const c_char` | Null-terminated path string |

**Returns:**
- `0`: Success
- `<0`: Error code

**Safety:**
- `out` must be valid pointer
- `path` must be null-terminated UTF-8
- On success, `*out` contains owned repository handle

---

##### `git_repository_free`

Frees a repository handle.

**Signature:**
```rust
pub fn git_repository_free(repo: *mut git_repository)
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `repo` | `*mut git_repository` | Repository handle to free |

**Safety:**
- `repo` must be a valid repository handle
- Must not be used after this call
- Null pointer is allowed (no-op)

---

#### Reference Operations

##### `git_reference_name_to_id`

Converts a reference name to an object ID.

**Signature:**
```rust
pub fn git_reference_name_to_id(
    out: *mut git_oid,
    repo: *mut git_repository,
    reference: *const c_char
) -> c_int
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `out` | `*mut git_oid` | Receives object ID |
| `repo` | `*mut git_repository` | Repository handle |
| `reference` | `*const c_char` | Reference name (e.g., "HEAD") |

**Returns:**
- `0`: Success
- `<0`: Error code

**Example:**
```rust
let mut oid = std::mem::MaybeUninit::uninit();
let name = CString::new("HEAD")?;
unsafe {
    let code = git_reference_name_to_id(
        oid.as_mut_ptr(),
        repo,
        name.as_ptr()
    );
    if code == 0 {
        let oid = oid.assume_init();
        // Use oid
    }
}
```

---

#### Commit Operations

##### `git_commit_lookup`

Looks up a commit by object ID.

**Signature:**
```rust
pub fn git_commit_lookup(
    out: *mut *mut git_commit,
    repo: *mut git_repository,
    id: *const git_oid
) -> c_int
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `out` | `*mut *mut git_commit` | Receives commit handle |
| `repo` | `*mut git_repository` | Repository handle |
| `id` | `*const git_oid` | Object ID of commit |

**Returns:**
- `0`: Success
- `<0`: Error code

---

##### `git_commit_author`

Gets the author signature of a commit.

**Signature:**
```rust
pub fn git_commit_author(commit: *const git_commit) -> *const git_signature
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `commit` | `*const git_commit` | Commit handle |

**Returns:**
- `*const git_signature`: Pointer to signature (never null)

**Safety:**
- Returned pointer valid while commit handle is valid
- Must not be freed

---

##### `git_commit_message`

Gets the commit message.

**Signature:**
```rust
pub fn git_commit_message(commit: *const git_commit) -> *const c_char
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `commit` | `*const git_commit` | Commit handle |

**Returns:**
- `*const c_char`: Null-terminated message string

**Safety:**
- Returned pointer valid while commit handle is valid
- UTF-8 encoding not guaranteed

---

##### `git_commit_free`

Frees a commit handle.

**Signature:**
```rust
pub fn git_commit_free(commit: *mut git_commit)
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `commit` | `*mut git_commit` | Commit handle to free |

**Safety:**
- Null pointer allowed

---

### C Types

#### Opaque Types

##### `git_repository`

**Definition:**
```rust
#[repr(C)]
pub struct git_repository {
    _private: [u8; 0]
}
```

**Purpose:**
- Opaque handle to Git repository
- Size unknown, never instantiated in Rust
- Only used through pointers

---

##### `git_commit`

**Definition:**
```rust
#[repr(C)]
pub struct git_commit {
    _private: [u8; 0]
}
```

**Purpose:**
- Opaque handle to Git commit

---

#### Data Structures

##### `git_error`

**Definition:**
```rust
#[repr(C)]
pub struct git_error {
    pub message: *const c_char,
    pub klass: c_int
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `message` | `*const c_char` | Error message (null-terminated) |
| `klass` | `c_int` | Error category |

**Layout:**
- `#[repr(C)]` ensures C-compatible memory layout

---

##### `git_oid`

Object identifier (SHA-1 hash).

**Definition:**
```rust
pub const GIT_OID_RAWSZ: usize = 20;

#[repr(C)]
pub struct git_oid {
    pub id: [c_uchar; GIT_OID_RAWSZ]
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `id` | `[c_uchar; 20]` | 20-byte SHA-1 hash |

---

##### `git_signature`

Author or committer signature.

**Definition:**
```rust
#[repr(C)]
pub struct git_signature {
    pub name: *const c_char,
    pub email: *const c_char,
    pub when: git_time
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `name` | `*const c_char` | Author/committer name |
| `email` | `*const c_char` | Email address |
| `when` | `git_time` | Timestamp |

---

##### `git_time`

**Definition:**
```rust
pub type git_time_t = i64;

#[repr(C)]
pub struct git_time {
    pub time: git_time_t,     // Unix timestamp
    pub offset: c_int         // Timezone offset in minutes
}
```

---

## libgit2-rs-safe - Safe Wrapper

Safe, idiomatic Rust wrapper around libgit2 raw bindings.

### Error Handling

#### `Error`

Safe error type wrapping libgit2 errors.

**Definition:**
```rust
#[derive(Debug)]
pub struct Error {
    code: i32,
    message: String,
    class: i32
}
```

**Trait Implementations:**
- `Debug`
- `Display`
- `std::error::Error`

**Display Format:**
```rust
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.message.fmt(f)
    }
}
```

---

#### `Result<T>`

Type alias for operations that can fail.

**Definition:**
```rust
pub type Result<T> = result::Result<T, Error>
```

---

#### `check`

Converts libgit2 return codes to `Result`.

**Signature:**
```rust
fn check(code: c_int) -> Result<c_int>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `code` | `c_int` | Return code from libgit2 function |

**Returns:**
- `Ok(code)`: If `code >= 0`
- `Err(Error)`: If `code < 0`, with error details from `giterr_last()`

**Safety:**
- Calls unsafe `giterr_last()`
- Converts C string to Rust `String`

---

### Repository

#### `Repository`

Safe wrapper for Git repository.

**Definition:**
```rust
pub struct Repository {
    raw: *mut raw::git_repository
}
```

**Invariants:**
- `raw` always points to a valid repository
- No other `Repository` shares the same pointer
- Automatically freed on drop

---

##### `open`

Opens a Git repository at the given path.

**Signature:**
```rust
pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | `P: AsRef<Path>` | Path to repository |

**Returns:**
- `Ok(Repository)`: Successfully opened
- `Err(Error)`: Path invalid or not a repository

**Example:**
```rust
let repo = Repository::open("/path/to/repo")?;
```

**Initialization:**
- Automatically calls `ensure_initialized()`
- Registers shutdown handler

---

##### `reference_name_to_id`

Resolves a reference name to an object ID.

**Signature:**
```rust
pub fn reference_name_to_id(&self, name: &str) -> Result<Oid>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | `&str` | Reference name (e.g., "HEAD", "refs/heads/main") |

**Returns:**
- `Ok(Oid)`: Object ID of the reference
- `Err(Error)`: Reference not found

**Example:**
```rust
let oid = repo.reference_name_to_id("HEAD")?;
```

---

##### `find_commit`

Looks up a commit by object ID.

**Signature:**
```rust
pub fn find_commit(&self, oid: &Oid) -> Result<Commit>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `oid` | `&Oid` | Object ID |

**Returns:**
- `Ok(Commit)`: Commit object
- `Err(Error)`: Not found or not a commit

**Example:**
```rust
let oid = repo.reference_name_to_id("HEAD")?;
let commit = repo.find_commit(&oid)?;
```

---

#### `Drop` for `Repository`

**Signature:**
```rust
impl Drop for Repository {
    fn drop(&mut self)
}
```

**Behavior:**
- Calls `raw::git_repository_free(self.raw)`
- Automatically releases resources

---

### Commit

#### `Commit<'repo>`

Safe wrapper for a Git commit.

**Definition:**
```rust
pub struct Commit<'repo> {
    raw: *mut raw::git_commit,
    _marker: PhantomData<&'repo Repository>
}
```

**Lifetime:**
- `'repo`: Lifetime of the repository
- Ensures commit doesn't outlive repository

---

##### `author`

Gets the commit's author signature.

**Signature:**
```rust
pub fn author(&self) -> Signature
```

**Returns:**
- `Signature`: Author information

**Example:**
```rust
let author = commit.author();
println!("Author: {}", author.name().unwrap_or("unknown"));
```

---

##### `message`

Gets the commit message.

**Signature:**
```rust
pub fn message(&self) -> Option<&str>
```

**Returns:**
- `Some(&str)`: UTF-8 commit message
- `None`: Message is not valid UTF-8

**Example:**
```rust
if let Some(msg) = commit.message() {
    println!("Message: {}", msg);
}
```

---

#### `Drop` for `Commit<'repo>`

**Signature:**
```rust
impl<'repo> Drop for Commit<'repo> {
    fn drop(&mut self)
}
```

**Behavior:**
- Calls `raw::git_commit_free(self.raw)`

---

### Signature

#### `Signature<'text>`

Author or committer signature.

**Definition:**
```rust
pub struct Signature<'text> {
    raw: *const raw::git_signature,
    _marker: PhantomData<&'text str>
}
```

**Lifetime:**
- `'text`: Lifetime of the source (commit)
- Ensures signature doesn't outlive commit

---

##### `name`

Gets the author's name.

**Signature:**
```rust
pub fn name(&self) -> Option<&str>
```

**Returns:**
- `Some(&str)`: UTF-8 name
- `None`: Name is not valid UTF-8

---

##### `email`

Gets the author's email.

**Signature:**
```rust
pub fn email(&self) -> Option<&str>
```

**Returns:**
- `Some(&str)`: UTF-8 email
- `None`: Email is not valid UTF-8

---

### Object ID

#### `Oid`

Safe wrapper for Git object IDs.

**Definition:**
```rust
pub struct Oid {
    pub raw: raw::git_oid
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `raw` | `raw::git_oid` | Underlying C structure |

---

### Utility Functions

#### `ensure_initialized`

Ensures libgit2 is initialized exactly once.

**Signature:**
```rust
fn ensure_initialized()
```

**Behavior:**
- Uses `std::sync::Once` for thread-safe initialization
- Calls `git_libgit2_init()` on first call
- Registers `shutdown` to be called at program exit

---

#### `shutdown`

Cleanup function registered with `atexit`.

**Signature:**
```rust
extern fn shutdown()
```

**Behavior:**
- Calls `git_libgit2_shutdown()`
- Aborts on error

---

#### `path_to_cstring`

Converts a Rust path to a C string.

**Unix Signature:**
```rust
#[cfg(unix)]
fn path_to_cstring(path: &Path) -> Result<CString>
```

**Windows Signature:**
```rust
#[cfg(windows)]
fn path_to_cstring(path: &Path) -> Result<CString>
```

**Platform Differences:**
- Unix: Uses raw bytes from `OsStr`
- Windows: Requires UTF-8 conversion

---

#### `char_ptr_to_str`

Safely borrows a string from a C pointer.

**Signature:**
```rust
unsafe fn char_ptr_to_str<T>(_owner: &T, ptr: *const c_char) -> Option<&str>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `_owner` | `&T` | Lifetime anchor |
| `ptr` | `*const c_char` | C string pointer |

**Returns:**
- `Some(&str)`: Valid UTF-8 string
- `None`: Null pointer or invalid UTF-8

**Safety:**
- `ptr` must be null or point to null-terminated string
- String must be valid for lifetime of `_owner`

---

## json! - JSON Construction Macro

Declarative macro for building JSON values with Rust-like syntax.

### Macro Definition

**Signature:**
```rust
#[macro_export]
macro_rules! json { /* ... */ }
```

### Syntax Patterns

#### Null

**Pattern:**
```rust
json!(null)
```

**Expands To:**
```rust
Json::Null
```

---

#### Array

**Pattern:**
```rust
json!([ $( $element:tt ),* ])
```

**Example:**
```rust
json!([1, 2, 3])
```

**Expands To:**
```rust
Json::Array(vec![
    json!(1),
    json!(2),
    json!(3)
])
```

**Nested:**
```rust
json!([
    {"name": "Alice"},
    {"name": "Bob"}
])
```

---

#### Object

**Pattern:**
```rust
json!({ $( $key:tt : $value:tt ),* })
```

**Example:**
```rust
json!({
    "name": "Alice",
    "age": 30,
    "active": true
})
```

**Expands To:**
```rust
{
    let mut fields = Box::new(HashMap::new());
    fields.insert("name".to_string(), json!("Alice"));
    fields.insert("age".to_string(), json!(30));
    fields.insert("active".to_string(), json!(true));
    Json::Object(fields)
}
```

---

#### Literal Value

**Pattern:**
```rust
json!($other:tt)
```

**Example:**
```rust
json!(42)
json!("hello")
json!(true)
```

**Expands To:**
```rust
Json::from(42)
Json::from("hello")
Json::from(true)
```

---

### Advanced Usage

#### Rust Expressions

**Interpolation:**
```rust
let width = 4.0;
json!({
    "width": width,
    "height": (width * 9.0 / 4.0)
})
```

**Computed Keys:**
```rust
const HELLO: &str = "hello";
json!({
    "en": HELLO,
    HELLO: "bonjour!"
})
```

---

#### Nested Structures

**Complex Example:**
```rust
let students = json!([
    {
        "name": "Jim Blandy",
        "class_of": 1926,
        "major": "Tibetan throat singing"
    },
    {
        "name": "Jason Orendorff",
        "class_of": 1702,
        "major": "Knots"
    }
]);
```

---

### Macro Hygiene

**Variable Capture:**
```rust
// User's variable doesn't conflict with macro's internal `fields`
let fields = "W.C. Fields";
let role = json!({
    "name": "Larson E. Whipsnade",
    "actor": fields  // Uses user's `fields`
});
```

**Rust automatically renames macro variables to avoid conflicts.**

---

### Dependencies

**Re-exported Types:**
```rust
pub use std::collections::HashMap;
pub use std::boxed::Box;
pub use std::string::ToString;
```

**Purpose:**
- Make types available to macro expansion
- Avoid requiring users to import them

---

## fern_sim - Module System Example

Demonstration of Rust's module system with a fern growth simulator.

### Public API

**Crate Root (`lib.rs`):**
```rust
pub mod plant_structures;
pub mod simulation;
pub mod spores;
pub mod net;

pub use plant_structures::Fern;
pub use simulation::Terrarium;
pub use net::connect;
```

---

### Module Structure

```
fern_sim/
├── lib.rs
├── net.rs
├── simulation.rs
├── spores.rs
└── plant_structures/
    ├── mod.rs
    ├── leaves.rs
    ├── roots.rs
    └── stems/
        ├── mod.rs
        ├── phloem.rs
        └── xylem.rs
```

---

### plant_structures

#### `Fern`

Main plant type.

**Definition:**
```rust
pub struct Fern {
    pub roots: RootSet,
    pub stems: StemSet
}
```

**Type Aliases:**
```rust
pub type RootSet = Vec<Root>;
pub type StemSet = Vec<Stem>;
```

---

##### `new`

Creates a new fern.

**Signature:**
```rust
pub fn new(_type: FernType) -> Fern
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `_type` | `FernType` | Fern species |

**Returns:**
- `Fern`: New fern instance

---

##### `is_furled` / `is_fully_unfurled`

**Signatures:**
```rust
pub fn is_furled(&self) -> bool
pub fn is_fully_unfurled(&self) -> bool
```

**Returns:**
- `bool`: Fern's unfurl state

---

#### `FernType`

**Definition:**
```rust
pub enum FernType {
    Fiddlehead
}
```

---

#### `VascularPath`

**Definition:**
```rust
#[doc(alias = "route")]
pub struct VascularPath {
    pub from: bool,
    pub to: bool,
}
```

**Doc Alias:**
- `#[doc(alias = "route")]` makes it searchable as "route"

---

#### `trace_path`

Creates a path from leaf to root.

**Signature:**
```rust
pub fn trace_path(leaf: &Leaf, root: &Root) -> VascularPath
```

---

### simulation

#### `Terrarium`

The simulated universe.

**Definition:**
```rust
pub struct Terrarium {
    ferns: Vec<Fern>
}
```

---

##### `new`

**Signature:**
```rust
pub fn new() -> Terrarium
```

**Returns:**
- `Terrarium`: Empty terrarium

---

##### `load`

**Signature:**
```rust
pub fn load(filename: &str) -> Terrarium
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `filename` | `&str` | Path to `.tm` file |

**Returns:**
- `Terrarium`: Loaded terrarium

---

##### `fern`

**Signature:**
```rust
pub fn fern(&self, index: usize) -> &Fern
```

**Returns:**
- `&Fern`: Reference to fern at index

---

##### `apply_sunlight`

**Signature:**
```rust
pub fn apply_sunlight(&mut self, time: Duration)
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `time` | `Duration` | Simulation time |

**Example:**
```rust
use fern_sim::Terrarium;
use std::time::Duration;

let mut tm = Terrarium::new();
tm.apply_sunlight(Duration::from_secs(60));
```

---

### spores

#### `Spore`

**Definition:**
```rust
pub struct Spore {
    size: f64
}
```

---

#### `produce_spore`

**Signature:**
```rust
pub fn produce_spore(factory: &mut Sporangium) -> Spore
```

---

#### `genes`

**Signature:**
```rust
pub(crate) fn genes(spore: &Spore) -> Vec<Gene>
```

**Visibility:**
- `pub(crate)`: Visible within crate, not exported

---

### Module Visibility

| Visibility | Keyword | Scope |
|-----------|---------|-------|
| Private | (none) | Current module only |
| Crate | `pub(crate)` | Entire crate |
| Public | `pub` | External crates |
| Super | `pub(super)` | Parent module |
| In path | `pub(in path)` | Specific ancestor |

---

## FFI Safety Patterns

### Opaque Types

**Pattern:**
```rust
#[repr(C)]
pub struct OpaqueType {
    _private: [u8; 0]
}
```

**Use:** Types never instantiated, only used through pointers

---

### RAII Wrappers

**Pattern:**
```rust
pub struct Wrapper {
    raw: *mut RawType
}

impl Drop for Wrapper {
    fn drop(&mut self) {
        unsafe { free_raw(self.raw) }
    }
}
```

**Use:** Automatic resource management

---

### Lifetime Binding

**Pattern:**
```rust
pub struct Borrowed<'a> {
    raw: *const RawType,
    _marker: PhantomData<&'a Owner>
}
```

**Use:** Ensure borrowed data doesn't outlive owner

---

## Macro Patterns

### Declarative Macros

**Token Tree Matching:**
```rust
macro_rules! name {
    ($pattern:tt) => { /* expansion */ };
}
```

**Repetition:**
```rust
$( $item:tt ),*  // Zero or more, comma-separated
$( $item:tt ),+  // One or more, comma-separated
```

**Designators:**
- `tt` - Token tree (any single token or {...})
- `expr` - Expression
- `ident` - Identifier
- `ty` - Type

---

### Hygiene

**Automatic:** Macro variables don't clash with user variables
**Export:** `#[macro_export]` makes macro available to other crates

---

## Best Practices

### FFI
1. Use `#[repr(C)]` for C-compatible structs
2. Wrap unsafe functions in safe interfaces
3. Use RAII for resource management
4. Document safety requirements
5. Check error codes immediately

### Macros
1. Use token trees (`tt`) for flexibility
2. Test macro hygiene
3. Provide examples in documentation
4. Consider procedural macros for complex logic
5. Use `#[macro_export]` for public macros
