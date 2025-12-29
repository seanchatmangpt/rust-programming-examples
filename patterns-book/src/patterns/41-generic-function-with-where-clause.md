# 41. GENERIC FUNCTION WITH WHERE CLAUSE

*A function signature with type parameters listed at the top, their bounds listed below in a separate where clause, creating clean horizontal space*

...within a **TRAIT-BOUNDED GENERIC (38)** or **FUNCTION ACCEPTING MULTIPLE TRAITS (39)**, when you need to specify complex trait bounds that would clutter the function signature...

◆ ◆ ◆

**How do you write generic functions with multiple trait bounds without making the signature unreadable?**

When functions accept generic parameters, you must specify what capabilities those types provide—their trait bounds. The simplest approach puts bounds directly in angle brackets: `fn process<T: Display + Debug + Clone>(item: T)`. But as bounds accumulate, signatures become dense paragraphs that obscure the function's actual purpose.

Consider a file copying function that accepts any path-like type. The signature `fn copy_into<P: AsRef<Path>, Q: AsRef<Path>>(source: P, destination: Q) -> io::Result<()>` puts bounds inline, forcing readers to parse constraints and parameters simultaneously. Add another bound or lifetime, and comprehension degrades rapidly.

The problem intensifies with multiple parameters and complex bounds. A function accepting both a reader implementing `BufRead` and a writer implementing `Write` might need: `fn process<R: BufRead + Seek, W: Write + Debug>(reader: R, writer: W)`. The actual parameter types—`reader: R` and `writer: W`—disappear into the noise.

**Therefore:**

**Declare generic parameters in angle brackets with minimal or no bounds, then specify all trait bounds in a separate where clause below the signature.**

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

fn copy_into<P, Q>(source: P, destination: Q) -> io::Result<()>
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let src = source.as_ref();
    let dst = destination.as_ref();

    match src.file_name() {
        None => Err(io::Error::new(io::ErrorKind::Other, "no filename")),
        Some(src_name) => {
            let md = src.metadata()?;
            copy_to(src, &md.file_type(), &dst.join(src_name))
        }
    }
}
```

*Function signature on one line, where clause indented below, each bound on its own line for complex cases*

◆ ◆ ◆

This separates interface (what parameters the function takes) from contract (what those parameters must be able to do). Use **TRAIT-BOUNDED GENERIC (38)** for the actual constraints, **FUNCTION ACCEPTING MULTIPLE TRAITS (39)** when combining bounds, and **GENERIC STRUCT WITH PHANTOM DATA (40)** when generics appear in types rather than functions.
