# 20. CONDITIONAL COMPILATION BY OS

*A single codebase spans many platforms—Unix, Windows, macOS—each with different system calls, different conventions, different capabilities. The code must adapt, selecting the right implementation at compile time.*

...within a **PORTABLE LIBRARY (28)** or **CROSS-PLATFORM COMMAND-LINE TOOL (29)**, when your code must handle platform-specific differences in system APIs, file paths, or functionality...

◆ ◆ ◆

**How do you write code that works correctly on multiple operating systems when those systems provide different APIs and different semantics for the same operations?**

Operating systems are not uniform. Unix systems provide `symlink()` for creating symbolic links; Windows historically did not. File paths on Unix are arbitrary byte sequences; on Windows they're meant to be Unicode. System APIs differ in signature, behavior, and availability.

If you write only for one platform, your code won't compile elsewhere. If you try to write generic code that works everywhere, you'll find that no such code exists—eventually you must call OS-specific functions. Runtime checks (`if cfg!(unix)`) can select behavior, but they include dead code for other platforms, bloating binaries and causing maintenance hazards.

The tension is between code reuse and platform specificity. You want to share logic where platforms agree, but substitute implementations where they diverge. You want compile-time selection so dead code is eliminated, but you also want a clear structure that's easy to maintain.

**Therefore:**

**Use `#[cfg(unix)]`, `#[cfg(windows)]`, and `#[cfg(not(unix))]` attributes to conditionally compile different implementations. Write platform-specific functions with the same signature, so calling code doesn't need to change.**

```rust
use std::path::Path;
use std::ffi::CString;
use std::io;

// Unix-specific implementation using OsStrExt
#[cfg(unix)]
fn path_to_cstring(path: &Path) -> io::Result<CString> {
    use std::os::unix::ffi::OsStrExt;
    Ok(CString::new(path.as_os_str().as_bytes())?)
}

// Windows-specific implementation requiring UTF-8
#[cfg(windows)]
fn path_to_cstring(path: &Path) -> io::Result<CString> {
    match path.to_str() {
        Some(s) => Ok(CString::new(s)?),
        None => {
            let message = format!("Couldn't convert path '{}' to UTF-8",
                                  path.display());
            Err(io::Error::new(io::ErrorKind::Other, message))
        }
    }
}
```

For functionality that exists on only some platforms:

```rust
// Full implementation on Unix
#[cfg(unix)]
use std::os::unix::fs::symlink;

// Stub implementation on other platforms
#[cfg(not(unix))]
fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, _dst: Q)
    -> io::Result<()>
{
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("can't copy symbolic link: {}", src.as_ref().display())
    ))
}

// Calling code works on all platforms
fn copy_symlink(src: &Path, dst: &Path) -> io::Result<()> {
    let target = src.read_link()?;
    symlink(target, dst)?;  // Works on Unix, returns error elsewhere
    Ok(())
}
```

*Conditional compilation is a scalpel: it cuts away the parts that don't apply to this platform, leaving only the code that can actually run.*

◆ ◆ ◆

This pattern makes cross-platform code maintainable. Each platform's implementation is self-contained, visible in the source, and compiled only when relevant. The compiler eliminates dead code, so Windows builds don't carry Unix system call logic, and vice versa.

Common `cfg` attributes include:

- `#[cfg(unix)]` — Linux, macOS, BSDs, etc.
- `#[cfg(windows)]` — Windows
- `#[cfg(target_os = "macos")]` — Specifically macOS
- `#[cfg(target_os = "linux")]` — Specifically Linux
- `#[cfg(not(unix))]` — Everything except Unix
- `#[cfg(any(unix, target_os = "wasi"))]` — Multiple conditions

You can also combine conditions:

```rust
#[cfg(all(unix, not(target_os = "macos")))]
fn unix_except_macos() {
    // Linux, BSD, etc., but not macOS
}
```

The pattern works best when platform differences are localized to specific functions. Write a platform-agnostic interface, then provide multiple implementations. This keeps platform-specific code isolated, making it easy to test and maintain.

For more complex cases, consider platform-specific modules:

```rust
#[cfg(unix)]
mod platform {
    pub fn get_user_home() -> PathBuf { /* Unix impl */ }
}

#[cfg(windows)]
mod platform {
    pub fn get_user_home() -> PathBuf { /* Windows impl */ }
}

// Common code just uses platform::get_user_home()
```

Be careful with `cfg` attributes. They create multiple code paths that must all be tested. If you only develop on Linux, your Windows code might rot. Use continuous integration to test all platforms regularly.

Use **BUILD SCRIPT FOR C DEPENDENCIES (19)** when platform differences affect linking or compilation, **TYPE ALIAS FOR ERGONOMIC APIS (9)** to hide platform-specific type differences, and **ERROR TYPE WITH CONTEXT (10)** to provide helpful messages when platform-specific operations fail.
