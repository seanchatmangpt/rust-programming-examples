# Sample Patterns in Alexander's Format

This document contains several complete patterns written in Christopher Alexander's exact format, demonstrating how the pattern language works across different scales.

---

## 1. BINARY WITH MAIN FUNCTION

*[Illustration: A simple Rust project structure showing src/main.rs at the root, with Cargo.toml alongside it, and a target/ directory for compilation output]*

...this is the most fundamental pattern in Rust programming. It is the starting point when you want to create a **standalone executable program** that runs on its own, taking input and producing output...

◆ ◆ ◆

**Programs need an entry point—a place where execution begins when the operating system loads them.**

In the Unix tradition, every program begins at a function called `main()`. Without this entry point, the operating system has nowhere to transfer control when the program starts. The C language established this convention decades ago, and Rust follows it.

But Rust's main function is different from C's. It doesn't have to return an integer status code—it can return `()` (unit type) for simple programs, or it can return `Result<(), E>` for programs that might fail during initialization. This flexibility lets you write clean, expressive code without the boilerplate of C's `return 0;`.

The main function is also where command-line arguments arrive (through `std::env::args()`), where you configure logging and runtime settings, and where you set up the initial state of your program before entering the main loop or processing logic.

A binary crate in Rust is defined by the presence of `src/main.rs`. This file must contain exactly one `fn main()` function—no more, no less. The compiler looks for this file and this function when building an executable.

**Therefore:**

**Create a file `src/main.rs` with a function `fn main()` as the entry point. Let the main function set up the program's initial state, collect arguments if needed, and call into the core logic. Keep main short and focused on setup.**

```rust
use std::env;

fn main() {
    let mut numbers = Vec::new();

    for arg in env::args().skip(1) {
        numbers.push(u64::from_str(&arg)
                     .expect("error parsing argument"));
    }

    if numbers.len() == 0 {
        eprintln!("Usage: gcd NUMBER ...");
        std::process::exit(1);
    }

    let mut d = numbers[0];
    for m in &numbers[1..] {
        d = gcd(d, *m);
    }

    println!("The greatest common divisor of {:?} is {}", numbers, d);
}

fn gcd(mut n: u64, mut m: u64) -> u64 {
    // ... actual algorithm lives in a separate function
}
```

*[Diagram: A flow chart showing:
1. OS loads program → main() starts
2. main() collects args → parses into data
3. main() calls core logic (gcd function)
4. main() outputs result
5. main() exits → OS reclaims resources
]*

The main function is thin—it's a **coordinator**, not a worker. The actual logic (the `gcd` function) lives separately, where it can be tested and reused.

Notice that main uses a **FOR LOOP OVER BORROWED REFERENCE (46)** to iterate through arguments without consuming them. It uses `expect()` for errors during startup (where failing fast is acceptable), and it uses `eprintln!` to write errors to stderr, not stdout.

◆ ◆ ◆

When your program might fail during initialization, make main return `Result<(), Box<dyn std::error::Error>>` and use **MATCH ON RESULT WITH QUESTION MARK (43)** to propagate errors cleanly. If you have both a binary and library, use **BINARY AND LIBRARY TOGETHER (3)**. Inside main, use **FOR LOOP OVER BORROWED REFERENCE (46)** to process arguments. Put the core logic in separate functions that you can test independently.

---

## 12. PRIVATE MODULE PUBLIC REEXPORT

*[Illustration: A crate structure showing internal modules `plant_structures`, `simulation`, `spores` hidden behind a single `pub use` statement in lib.rs, with external users seeing only `Fern` and `Terrarium` at the crate root]*

...within a **LIBRARY CRATE WITH PUBLIC API (2)**, after you have created **SUBMODULE IN SEPARATE FILE (10)** or **NESTED SUBMODULES IN DIRECTORY (11)** to organize your internal implementation...

◆ ◆ ◆

**Library users should see a clean, simple API with the most important types at the crate root. But internally, you need to organize code into logical modules. These two needs conflict.**

When you build a library with multiple modules, users would normally have to write:

```rust
use fern_sim::plant_structures::Fern;
use fern_sim::simulation::Terrarium;
```

This exposes your internal organization. If you later refactor and move `Fern` to a different module, all users' code breaks. The internal structure has leaked into the public API.

But there's a deeper problem: users don't care about your internal organization. They want the most common types to be easy to access. If every use requires three or four path segments, the library feels cumbersome and bureaucratic.

You could put everything in lib.rs, using **FLAT MODULE ALL IN LIB (14)**, but then you lose the organizational benefits of modules. Large files become hard to navigate, and related code gets separated by hundreds of lines.

The solution is to **separate the internal organization from the public API**. Internally, use modules to organize code logically. But publicly, re-export the most important types at the crate root, hiding the module structure.

**Therefore:**

**Declare your modules as `pub mod` or private `mod` in lib.rs, then use `pub use` to bring the important types to the crate root. Users import from the crate root; the internal module structure stays hidden.**

```rust
// src/lib.rs
pub mod plant_structures;
pub mod simulation;
pub mod spores;

// Reexport core types at the crate root
pub use plant_structures::Fern;
pub use simulation::Terrarium;

// spores stays internal - users don't need it directly
```

Now users can write:

```rust
use fern_sim::{Fern, Terrarium};
```

The internal organization (that `Fern` lives in `plant_structures`) is hidden. You can refactor freely without breaking user code.

*[Diagram: Two views of the same crate:

Internal view (developer):
- fern_sim/
  - plant_structures/
    - Fern
  - simulation/
    - Terrarium
  - spores/
    - (internal details)

External view (user):
- fern_sim::Fern ✓ (reexported)
- fern_sim::Terrarium ✓ (reexported)
- fern_sim::plant_structures::Fern ✓ (still accessible)
- fern_sim::spores::... ✗ (not public)
]*

◆ ◆ ◆

In your lib.rs, use **CRATE ROOT REEXPORTING CORE (18)** to bring key types to the top level. Keep module declarations together at the top of the file, followed by reexports. If you have many types, consider using **PUBLIC FACADE MODULE (17)** as an intermediary. Mark some modules as `pub` (users can still access them if needed) and others as private (internal only). The most frequently used types should be shortest to import.

---

## 26. NEWTYPE WRAPPING RAW POINTER

*[Illustration: A Rust struct containing a single field `raw: *mut git_repository`, with a clear boundary between safe Rust code on the outside and unsafe C FFI on the inside]*

...when building an **UNSAFE FFI WRAPPER CRATE (7)** or **SAFE WRAPPER AROUND UNSAFE (8)**, after you have created a **RAW BINDINGS MODULE (16)** with extern declarations...

◆ ◆ ◆

**FFI code requires raw pointers to C structures, but raw pointers are unsafe and can't participate in Rust's ownership system. We need a way to bring C resources into Rust's ownership model.**

When calling C libraries from Rust, you get back raw pointers: `*mut git_repository`, `*const git_commit`, etc. These pointers are handles to C data structures allocated by C code. But raw pointers in Rust are:

1. **Not owned** - Rust doesn't know when to free them
2. **Not lifetime-tracked** - references can outlive the data
3. **Not Send/Sync** - can't be used safely across threads
4. **Not null-safe** - might be null at any time

If you use raw pointers directly throughout your code, every use is an `unsafe` block. The unsafe code spreads like a virus, making your entire library dangerous.

The key insight is that while the *pointer* is unsafe, the *handle to the resource* can be safe. A C `git_repository*` represents ownership of a repository handle—there should be exactly one owner, and when that owner is done, the repository should be freed.

This is exactly what Rust's ownership system provides! We need to wrap the raw pointer in a Rust type that enforces ownership and implements Drop to clean up.

**Therefore:**

**Create a struct containing a single raw pointer field. Make the field private. This struct represents ownership of the C resource. Implement Drop to call the C cleanup function. Never expose the raw pointer directly—all access goes through safe methods.**

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

*[Diagram: The lifecycle of a Repository:

1. User calls `Repository::open(path)`
2. Unsafe block: C library allocates git_repository
3. Raw pointer wrapped in Repository struct
4. User works with safe Repository methods
5. Repository goes out of scope
6. Drop impl called: unsafe block frees C resource
7. No memory leak—Rust's ownership ensured cleanup
]*

The invariants are **documented in comments**: "This must always be a pointer to a live `git_repository` structure." The struct enforces this invariant—only the constructor can create a Repository, and it ensures the pointer is valid. Drop ensures cleanup happens.

The raw pointer field is **private**—users can never access it, so they can't violate the invariant. All public methods take `&self` or `&mut self`, which means Rust's borrowing rules ensure the pointer is still valid when methods use it.

◆ ◆ ◆

If the C resource is tied to the lifetime of another resource, add **PHANTOMDATA FOR LIFETIME (27)** to track the relationship. Use **UNSAFE FUNCTION WITH SAFETY COMMENT (40)** for the constructor that wraps the raw pointer, documenting why it's safe. Return errors with **FUNCTION RETURNING RESULT (38)** if the C library can fail. Make constructor use **FUNCTION TAKING ASREF PATH (42)** for flexible path parameters. The Drop implementation should never panic—use a separate shutdown function if cleanup can fail.

---

## 43. MATCH ON RESULT WITH QUESTION MARK

*[Illustration: A function with multiple fallible operations, showing error flow short-circuiting up the call stack with ? operators, compared to a tower of nested match statements]*

...within any **FUNCTION RETURNING RESULT (38)**, when you need to call other fallible operations and want to propagate errors upward...

◆ ◆ ◆

**Operations can fail, and when they do, you usually want to return the error to your caller rather than handle it locally. But checking each Result with match creates deeply nested code.**

The naive way to handle errors is to match on every Result:

```rust
fn fetch_user_data(user_id: u64) -> Result<UserData, Error> {
    match open_database() {
        Ok(db) => {
            match db.find_user(user_id) {
                Ok(user) => {
                    match load_profile(user) {
                        Ok(profile) => Ok(UserData { profile }),
                        Err(e) => Err(e.into()),
                    }
                }
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => Err(e.into()),
    }
}
```

This is the "pyramid of doom"—each operation adds another level of nesting. The signal (what the function does) is buried in noise (error handling boilerplate). Worse, the error handling is **identical** at every level: just convert and return.

Some languages use exceptions to avoid this, but exceptions are invisible control flow. You can't see from a function's signature whether it might throw. Rust's Result type makes errors visible and type-checked, which is better—but only if the syntax doesn't punish you for it.

The pattern is clear: if an operation returns `Err(e)`, we want to immediately return `Err(e.into())` from the current function. This is **early return on error**, a pattern so common it deserves dedicated syntax.

**Therefore:**

**After any expression that returns Result, add a question mark: `operation()?`. If the Result is Ok(value), the expression evaluates to value. If it's Err(e), the function immediately returns Err(e.into()).**

```rust
fn fetch_user_data(user_id: u64) -> Result<UserData, Error> {
    let db = open_database()?;
    let user = db.find_user(user_id)?;
    let profile = load_profile(user)?;
    Ok(UserData { profile })
}
```

The happy path reads **linearly**, like synchronous code. The error paths are **implicit but visible** (you can see every `?`). The function signature **declares** that it can fail.

*[Diagram: Two flow charts side by side:

Without ?:
  open_database() → match Ok/Err
    → if Ok: find_user() → match Ok/Err
      → if Ok: load_profile() → match Ok/Err
        → if Ok: return Ok
        → if Err: return Err
      → if Err: return Err
    → if Err: return Err

With ?:
  open_database()? → (returns early on Err)
  find_user()? → (returns early on Err)
  load_profile()? → (returns early on Err)
  return Ok
]*

The `?` operator also converts errors using `From::from()`, so if `open_database()` returns `Result<Db, DbError>` but your function returns `Result<UserData, Error>`, the conversion happens automatically (if you've implemented `From<DbError> for Error`).

◆ ◆ ◆

Use this in any **FUNCTION RETURNING RESULT (38)**. The function's error type must be compatible with the errors being propagated (either the same type, or implement From for conversion). In a **BINARY WITH MAIN FUNCTION (1)**, you can make main return `Result<(), Box<dyn std::error::Error>>` to use ? for startup errors. For Option types, ? works too, returning None early. If you need to add context to errors, use `.map_err(|e| ...)` before the ?.

---

## 48. MEM SWAP FOR MOVING VALUES

*[Illustration: Two boxes labeled 'older' and 'younger' with arrows showing their contents exchanging places in a single atomic operation, without any intermediate copying]*

...when you have a **METHOD TAKING SELF BY MUT REFERENCE (34)** or **STRUCT WITH VEC FIELDS (21)**, and you need to exchange the contents of two variables without copying...

◆ ◆ ◆

**Sometimes you need to move a value out of a location and move another value in, but Rust doesn't allow leaving variables uninitialized. You can't move out without replacing.**

Consider a queue implemented with two vectors:

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}
```

When `older` runs empty, you need to move `younger`'s contents into `older`, reversing them. The naive approach doesn't work:

```rust
// ❌ This doesn't compile
pub fn pop(&mut self) -> Option<T> {
    if self.older.is_empty() {
        self.older = self.younger;  // ERROR: can't move out of borrowed content
        self.older.reverse();
    }
    self.older.pop()
}
```

You can't move `self.younger` because `self` is borrowed (`&mut self`). Moving would leave `self.younger` uninitialized, which Rust forbids.

You could clone: `self.older = self.younger.clone()`, but that's expensive—it copies every element. For a queue with a million items, that's a million unnecessary allocations.

You could use `mem::replace`: `self.older = mem::replace(&mut self.younger, Vec::new())`, but that still allocates a new empty Vec.

The fundamental insight is that you don't need to copy or allocate—you just need to **exchange the two values**. What was in `older` should go into `younger`, and vice versa. Both locations stay initialized; their contents just swap places.

**Therefore:**

**Use `std::mem::swap(&mut a, &mut b)` to exchange two values in place, without copying or allocation. Both values must be mutable references.**

```rust
pub fn pop(&mut self) -> Option<T> {
    if self.older.is_empty() {
        if self.younger.is_empty() {
            return None;
        }

        use std::mem::swap;
        swap(&mut self.older, &mut self.younger);
        self.older.reverse();
    }
    self.older.pop()
}
```

*[Diagram: The memory state during swap:

Before:
  older → Vec [capacity: 10, len: 0, data: 0x1234]
  younger → Vec [capacity: 8, len: 5, data: 0x5678]

After swap (single pointer swap):
  older → Vec [capacity: 8, len: 5, data: 0x5678]
  younger → Vec [capacity: 10, len: 0, data: 0x1234]

No allocations. No copying. Just pointer exchanges.
]*

The swap is **O(1)** regardless of how large the vectors are, because it only swaps the Vec control structures (pointer, capacity, length), not the actual data. It's safe because both variables remain initialized—they just have each other's values now.

◆ ◆ ◆

Use this in **STRUCT WITH TWO VECS FOR QUEUE (22)** to efficiently move data between buffers. Also useful when implementing **METHOD CONSUMING SELF (35)** and you need to move fields out of self—swap with a default value, then consume the swapped-out value. Import with `use std::mem::swap;` at the top of the function (narrow scope). Never swap with uninitialized memory—both sides must be valid. For Option values, use `Option::take()` instead, which is a specialized swap with None.

---

## How These Patterns Work Together

Notice how patterns at different scales reference each other:

- **Pattern 1 (BINARY WITH MAIN FUNCTION)** references Pattern 43 (MATCH ON RESULT WITH QUESTION MARK) and Pattern 46 (FOR LOOP OVER BORROWED REFERENCE)

- **Pattern 12 (PRIVATE MODULE PUBLIC REEXPORT)** references Pattern 18 (CRATE ROOT REEXPORTING CORE) and Pattern 10 (SUBMODULE IN SEPARATE FILE)

- **Pattern 26 (NEWTYPE WRAPPING RAW POINTER)** references Pattern 27 (PHANTOMDATA FOR LIFETIME) and Pattern 40 (UNSAFE FUNCTION WITH SAFETY COMMENT)

- **Pattern 43 (MATCH ON RESULT WITH QUESTION MARK)** references Pattern 38 (FUNCTION RETURNING RESULT)

- **Pattern 48 (MEM SWAP FOR MOVING VALUES)** is referenced by Pattern 22 (STRUCT WITH TWO VECS FOR QUEUE)

This forms Alexander's **generative sequence**: start with large-scale patterns (what kind of crate?), add architectural patterns (how is it organized?), define types (what data structures?), implement functions (what operations?), and finish with expressions (what code?).

Each pattern is **complete in itself** but **incomplete without the others**. They form a language.
