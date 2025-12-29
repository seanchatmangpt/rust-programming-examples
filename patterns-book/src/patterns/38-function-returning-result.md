# 38. FUNCTION RETURNING RESULT

*A bridge inspector checking every cable and strut, returning either a certificate of safety or a detailed report of failures—never leaving the question unanswered*

...within a **PUBLIC FUNCTION (35)** or **PRIVATE HELPER FUNCTION**, when the operation might fail and the caller must decide how to handle that failure...

◆ ◆ ◆

**How do you write functions that can fail, without panicking the program or hiding errors from the caller?**

Many operations can fail: files might not exist, network connections might drop, parsing might encounter invalid input, calculations might overflow. In languages with exceptions, these errors jump up the call stack unpredictably. In languages with error codes, functions return magic values (-1, null, false) that callers must remember to check.

Rust makes failure explicit through the type system. Functions that can fail return `Result<T, E>`: either `Ok(value)` or `Err(error)`. This appears in the function signature, visible to every caller. You cannot ignore a Result—the compiler warns if you don't use it. You cannot forget to check for errors—attempting to use a Result as if it were the success value produces a type error.

The `?` operator propagates errors automatically, making error handling concise. Write `let value = operation()?;` and errors flow upward to the caller without ceremony. This creates a clean separation: functions at the leaves return Results, functions in the middle propagate them with `?`, and functions at the top (main, request handlers) decide how to present errors to users.

The pattern scales from simple I/O operations to complex multi-step workflows, maintaining the same structure at every level.

**Therefore:**

**Return `Result<T, E>` from any function that can fail, using the `?` operator to propagate errors and letting callers decide how to handle failure.**

```rust
// From grep/src/main.rs
use std::error::Error;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;

fn grep<R>(target: &str, reader: R) -> io::Result<()>
    where R: BufRead
{
    for line_result in reader.lines() {
        let line = line_result?;  // Propagate errors
        if line.contains(target) {
            println!("{}", line);
        }
    }
    Ok(())
}

fn grep_main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let target = match args.next() {
        Some(s) => s,
        None => Err("usage: grep PATTERN FILE...")?  // Convert to error
    };
    let files: Vec<PathBuf> = args.map(PathBuf::from).collect();

    if files.is_empty() {
        let stdin = io::stdin();
        grep(&target, stdin.lock())?;  // Propagate I/O errors
    } else {
        for file in files {
            let f = File::open(file)?;  // Propagate file errors
            grep(&target, BufReader::new(f))?;
        }
    }

    Ok(())
}

fn main() {
    let result = grep_main();
    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
```

*The function holds two paths forward—success and failure—returning which path it took and letting the caller choose the response*

◆ ◆ ◆

This connects to **QUESTION MARK OPERATOR (41)** for error propagation, **MATCH ON RESULT (28)** for handling specific error cases, and supports **PUBLIC FUNCTION (35)** by making failure modes explicit in the API contract.
