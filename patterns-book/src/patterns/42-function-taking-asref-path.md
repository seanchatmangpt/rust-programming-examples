# 42. FUNCTION TAKING ASREF PATH

*A function accepting file paths receives both owned PathBufs and borrowed &Paths through a single generic parameter*

...within a **GENERIC FUNCTION WITH WHERE CLAUSE (41)**, when you need to work with filesystem paths that might be owned strings, borrowed strings, PathBufs, or &Path references...

◆ ◆ ◆

**How do you write a function that accepts file paths in any form without requiring callers to convert their data?**

Functions working with filesystem paths face a proliferation of input types. A path might arrive as a `String`, `&str`, `PathBuf`, `&Path`, `OsString`, or even `&OsStr`. Writing separate functions for each type creates maintenance burden and API confusion. Forcing callers to convert everything to `PathBuf::from(...)` before calling your function shifts the burden outward and clutters call sites.

The standard library's `AsRef<Path>` trait solves this elegantly. Any type that can be viewed as a path reference implements it—including all the types listed above. A single generic parameter bounded by `AsRef<Path>` accepts any path-like input.

But simply writing `fn copy<P: AsRef<Path>>(source: P)` still requires the where clause pattern for readability. Inside the function, you must explicitly call `.as_ref()` to convert the generic parameter to a `&Path` before using path methods. This small conversion unlocks all path manipulation operations while keeping the external API flexible.

**Therefore:**

**Declare functions accepting paths with a generic parameter bounded by AsRef<Path>, then call as_ref() once at the function's start to convert to &Path.**

```rust
fn copy_into<P, Q>(source: P, destination: Q) -> io::Result<()>
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let src = source.as_ref();
    let dst = destination.as_ref();

    match src.file_name() {
        None => {
            Err(io::Error::new(io::ErrorKind::Other,
                              format!("can't copy nameless directory: {}",
                                     src.display())))
        }
        Some(src_name) => {
            let md = src.metadata()?;
            copy_to(src, &md.file_type(), &dst.join(src_name))
        }
    }
}

fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Other,
                       format!("can't copy symbolic link: {}",
                              src.as_ref().display())))
}
```

*Generic parameters P and Q at the signature, as_ref() call converting to concrete &Path at function start*

◆ ◆ ◆

This pattern appears throughout the standard library's filesystem APIs. Combine with **GENERIC FUNCTION WITH WHERE CLAUSE (41)** for readability, use **MATCH ON RESULT WITH QUESTION MARK (43)** for error handling, and **STRING SLICE PARAMETER (35)** for the same flexibility with text instead of paths.
