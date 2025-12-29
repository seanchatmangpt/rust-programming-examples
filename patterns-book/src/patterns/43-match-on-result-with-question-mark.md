# 43. MATCH ON RESULT WITH QUESTION MARK

*A single question mark after a fallible operation, causing immediate return from the enclosing function if an error occurs*

...within a **RESULT-RETURNING FUNCTION (23)** or **MAIN FUNCTION RETURNING RESULT (24)**, when you need to handle errors from multiple operations without nesting match statements...

◆ ◆ ◆

**How do you propagate errors from fallible operations without drowning your logic in explicit error handling?**

Functions performing I/O or parsing inevitably call operations returning `Result<T, E>`. Each call site requires a decision: handle the error locally or propagate it to the caller. Handling every error with `match` creates deeply nested code that obscures the happy path.

Consider reading lines from a file. The traditional approach requires explicit matching: `match reader.lines().next() { Some(line_result) => match line_result { Ok(line) => process(line), Err(e) => return Err(e) }, None => ... }`. Each operation adds another layer of nesting. Three or four fallible calls create an unreadable pyramid.

The `?` operator collapses this ceremony to a single character. Placed after any expression returning `Result`, it either unwraps the success value for immediate use or returns the error from the enclosing function. The operator handles error propagation invisibly, letting the main logic flow naturally.

But the `?` operator imposes a requirement: the enclosing function must return `Result` (or another type implementing `Try`). This creates beneficial pressure to organize code into clear layers—functions returning `Result` for fallible operations, calling each other with `?`, with only the top level converting errors to exit codes or user-facing messages.

**Therefore:**

**Place a question mark after any Result-returning expression to unwrap success values or propagate errors immediately to the caller.**

```rust
fn grep<R>(target: &str, reader: R) -> io::Result<()>
    where R: BufRead
{
    for line_result in reader.lines() {
        let line = line_result?;
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
        None => Err("usage: grep PATTERN FILE...")?
    };
    let files: Vec<PathBuf> = args.map(PathBuf::from).collect();

    if files.is_empty() {
        let stdin = io::stdin();
        grep(&target, stdin.lock())?;
    } else {
        for file in files {
            let f = File::open(file)?;
            grep(&target, BufReader::new(f))?;
        }
    }

    Ok(())
}
```

*Question marks appearing after File::open, stdin.lock(), grep calls—any operation that might fail*

◆ ◆ ◆

The `?` operator works with `Option<T>` in functions returning `Option`, unwrapping `Some` or returning `None`. Use with **RESULT-RETURNING FUNCTION (23)** for the function signature, **IF LET FOR OPTION UNWRAPPING (44)** when you need to handle errors locally, and **MAIN FUNCTION RETURNING RESULT (24)** to propagate errors all the way to program exit.
