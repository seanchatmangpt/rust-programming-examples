# 44. IF LET FOR OPTION UNWRAPPING

*A conditional that matches a single pattern, extracting the wrapped value only when it matches, ignoring all other cases*

...within a **RESULT-RETURNING FUNCTION (23)** or **OPTION-RETURNING METHOD (22)**, when you need to handle just the success case of an Option or Result without writing a full match statement...

◆ ◆ ◆

**How do you extract a value from an Option or Result when you only care about one variant?**

Pattern matching with `match` provides exhaustive handling of all possibilities—`Some` and `None` for `Option`, `Ok` and `Err` for `Result`. But many situations demand action only for the success case. Writing a full match statement when you only care about one branch creates visual noise: `match result { Some(value) => { /* ten lines */ }, None => {} }`. The empty branch advertises that you're doing nothing with it.

The problem intensifies when success handling spans many lines. A large block indented under a match arm, followed by `None => {}`, signals "this empty branch is important" when it's actually meaningless. Readers must verify that the empty branch is intentionally empty, not a forgotten implementation.

`if let` solves this by inverting the structure. It matches a single pattern and executes a block only if the pattern matches. No else branch means "I don't care about other cases"—the absence communicates intent. The code becomes `if let Some(value) = result { /* ten lines */ }`, making it clear that only the success case matters.

This pattern shines in `main` functions handling errors. Rather than propagating every error with `?`, the program can check for an error at the very end: `if let Err(err) = result { /* handle error */ }`. This keeps the error handling localized and visible.

**Therefore:**

**Use if let to match a single pattern from an Option or Result, executing code only when that specific variant matches.**

```rust
fn main() {
    let result = grep_main();
    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn copy_into<P, Q>(source: P, destination: Q) -> io::Result<()>
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let src = source.as_ref();
    let dst = destination.as_ref();

    match src.file_name() {
        None => {
            return Err(io::Error::new(io::ErrorKind::Other,
                                      format!("can't copy nameless directory: {}",
                                              src.display())));
        }
        Some(src_name) => {
            let md = src.metadata()?;
            copy_to(src, &md.file_type(), &dst.join(src_name))?;
        }
    }
    Ok(())
}
```

*The if let Err(err) pattern in main, extracting error only if present*

◆ ◆ ◆

Use `if let` when only one variant matters and requires handling. When you need both branches, use full `match`. When propagating errors through many calls, use **MATCH ON RESULT WITH QUESTION MARK (43)**. For repeated option checking in a loop, use **WHILE LET FOR ITERATION (45)** instead.
