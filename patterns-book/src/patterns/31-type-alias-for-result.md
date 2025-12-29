# 31. TYPE ALIAS FOR RESULT

*[Illustration: A library's public API showing multiple functions all returning the same `Result<T, Error>` pattern, with a single type alias declaration that simplifies every signature]*

...within a **LIBRARY CRATE WITH PUBLIC API (2)**, after you have created **CUSTOM ERROR TYPE (39)** to represent your crate's specific errors...

◆ ◆ ◆

**Every function in your library returns Result with the same error type, forcing users to write `std::result::Result<T, YourError>` repeatedly. The repetition obscures the actual return type and makes refactoring painful.**

When you build a library with consistent error handling, every public function that can fail returns a Result. But the full type signature is verbose:

```rust
pub fn open(path: &Path) -> std::result::Result<Repository, GitError> { }
pub fn commit(&self, msg: &str) -> std::result::Result<Commit, GitError> { }
pub fn push(&self) -> std::result::Result<(), GitError> { }
```

Each signature repeats `GitError`. If you later wrap your error type in another type, or add error context, every function signature must change. Users who have written their own wrapper functions must also update.

The noise-to-signal ratio is poor. When scanning the API, you want to see what each function *does* (returns a Repository, a Commit, nothing), not the error-handling machinery. But the error type takes more characters than the success type in many signatures.

The standard library itself uses this pattern. `std::io` defines `type Result<T> = std::result::Result<T, Error>`, allowing functions to write `io::Result<T>` instead of the full form. This has become idiomatic—users expect crates to define their own `Result` alias.

There's a subtlety: the alias is a *type constructor*, not a concrete type. It takes one type parameter (the success type) and fixes the second parameter (the error type) to your custom error. This preserves the flexibility of Result while eliminating repetition.

**Therefore:**

**In your crate's main module or prelude, define `pub type Result<T> = std::result::Result<T, YourError>` where YourError is your custom error type. Use this alias in all public function signatures. Users can refer to it as `your_crate::Result<T>`.**

```rust
// In lib.rs or error.rs
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    NotFound,
    PermissionDenied,
    InvalidInput(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotFound => write!(f, "resource not found"),
            Error::PermissionDenied => write!(f, "permission denied"),
            Error::InvalidInput(msg) => write!(f, "invalid input: {}", msg),
        }
    }
}

impl StdError for Error {}

// The key pattern: type alias
pub type Result<T> = std::result::Result<T, Error>;

// Now use it everywhere
pub fn open(path: &Path) -> Result<Repository> {
    // Implementation
}

pub fn read_config() -> Result<Config> {
    // Implementation
}

pub fn write_data(&self, data: &[u8]) -> Result<()> {
    // Implementation
}
```

*[Diagram: Two columns showing "Before" and "After":

Before:
```
fn open() -> std::result::Result<T, MyError>
fn read() -> std::result::Result<T, MyError>
fn write() -> std::result::Result<(), MyError>
                    ^^^^^^^^^^^^^^^^^^^^^^^^
                    repeated every time
```

After:
```
type Result<T> = std::result::Result<T, MyError>
                 (defined once)

fn open() -> Result<T>
fn read() -> Result<T>
fn write() -> Result<()>
             ^^^^^^
             concise
```
]*

The alias lives in your crate's namespace, so users import it naturally: `use your_crate::Result;`. Inside your crate, `Result<T>` is unambiguous—it always means your custom Result type. The full `std::result::Result` is still available when needed (for example, if you need to return a different error type internally).

This pattern creates **API consistency**. Every function uses the same error type, making it easy to compose operations with the `?` operator. Users only need to handle one error type, not a mixture of different errors across your API.

◆ ◆ ◆

Define this alias immediately after defining your **CUSTOM ERROR TYPE (39)**. Place it in the same module as the error type, or re-export both together in your crate root using **PRIVATE MODULE PUBLIC REEXPORT (12)**. If your crate has multiple error types for different subsystems, create multiple Result aliases: `type FooResult<T> = Result<T, FooError>`. All public functions should return this alias, not the full `std::result::Result`. When a function cannot fail, return `T` directly, not `Result<T>`.
