# 30. CUSTOM ERROR STRUCT WITH DISPLAY

*An error type that carries context and explains itself when examined, like a detailed incident report rather than just an alarm code.*

...within any **MODULE BOUNDARY** or **LIBRARY API**, when you need to report errors that carry more information than a string but present clearly to users...

◆ ◆ ◆

**How can you create errors that carry structured information yet display as readable messages?**

When wrapping a C library or building a complex system, errors have structure. A Git error carries an error code, a message, and an error class. If you return just a String, you lose this information—you cannot programmatically inspect the error code, and you must format everything immediately, even if the error is just propagated upward.

Rust's error handling works through two traits: `Display` for human-readable messages, and `std::error::Error` for the error trait bound. By creating a struct to hold error data, you preserve all information. By implementing Display, you control exactly how errors appear to users. The struct can store codes, causes, backtraces—anything you need—while Display presents a clean message.

This pattern enables the `?` operator to work with your errors, allows downstream code to inspect error details programmatically, and gives you a single place to control error formatting. FFI boundaries especially need this—C error codes and messages must be captured immediately, before the C library's error state changes.

**Therefore:**

**Create a struct to hold error information, derive Debug for programmer inspection, implement Display for user messages, and implement std::error::Error to make it work with ?.**

```rust
use std::error;
use std::fmt;

#[derive(Debug)]
pub struct Error {
    code: i32,
    message: String,
    class: i32
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Displaying an `Error` simply displays the message from libgit2.
        self.message.fmt(f)
    }
}

impl error::Error for Error { }

pub type Result<T> = std::result::Result<T, Error>;

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

*The error struct is like a detailed incident report—it contains all the technical data internally (code, class) but presents a clear human message when displayed, while still allowing programmatic inspection if needed.*

◆ ◆ ◆

This pattern works with **RESULT TYPE FOR FALLIBLE FUNCTIONS** (8) and **NEWTYPE WRAPPING RAW POINTER** (26), especially when converting C errors to Rust. Implement `From` to enable automatic error conversions with `?`.
