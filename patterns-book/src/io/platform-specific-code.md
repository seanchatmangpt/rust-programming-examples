# Platform-Specific Code

## Context

You are writing Rust code that needs to handle differences between operating systems. These differences might be in file system operations (symbolic links, permissions), system APIs (network configuration, process management), path conventions (separators, case sensitivity), or available system calls.

Your code should compile and run correctly on multiple platforms (Linux, macOS, Windows) while providing platform-appropriate behavior where necessary. Some features may only be available on certain platforms, requiring graceful degradation or alternative implementations.

## Problem

**How do you write Rust code that handles platform-specific differences cleanly, compiles only the appropriate code for each target, provides good error messages when features are unavailable, and maintains readability without cluttering the codebase with conditional compilation?**

Writing separate codebases for each platform is unmaintainable. Scattering runtime platform checks throughout code is inefficient and error-prone. Some platform-specific APIs don't exist on others, causing compilation errors. Rust's cross-platform story requires understanding conditional compilation attributes.

## Forces

- **Cross-platform**: Code should work on Linux, macOS, Windows where possible
- **Platform features**: Some OS features (symlinks, signals) aren't universal
- **Compile-time selection**: Only compile code for the target platform
- **Runtime efficiency**: No runtime overhead checking platform unnecessarily
- **Error messages**: Clear feedback when features aren't available
- **Maintainability**: Platform differences should be isolated and clear
- **Testing**: Must test on each target platform
- **Type safety**: Platform-specific types shouldn't leak across boundaries

## Solution

**Use `#[cfg(target_os)]` and `#[cfg(target_family)]` attributes to conditionally compile platform-specific code. Provide platform-specific implementations or stub implementations with clear error messages. Use `std::os::unix` and `std::os::windows` modules for platform-specific APIs.**

### Structure

```rust
// Platform-specific imports
#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(windows)]
use std::os::windows::fs::symlink_file as symlink;

// Stub for unsupported platforms
#[cfg(not(any(unix, windows)))]
fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Symlinks not supported on this platform"
    ))
}
```

### Real Implementation (from copy)

```rust
use std::io;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::symlink;

/// Stub implementation of `symlink` for platforms that don't provide it.
#[cfg(not(unix))]
fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, _dst: Q) -> std::io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Other,
                       format!("can't copy symbolic link: {}",
                               src.as_ref().display())))
}

/// Copy whatever is at `src` to the target path `dst`.
fn copy_to(src: &Path, src_type: &fs::FileType, dst: &Path) -> io::Result<()> {
    if src_type.is_file() {
        fs::copy(src, dst)?;
    } else if src_type.is_dir() {
        copy_dir_to(src, dst)?;
    } else if src_type.is_symlink() {
        let target = src.read_link()?;
        symlink(target, dst)?;  // Platform-specific implementation
    } else {
        return Err(io::Error::new(io::ErrorKind::Other,
                                  format!("don't know how to copy: {}",
                                          src.display())));
    }
    Ok(())
}
```

### Key Elements

1. **#[cfg(unix)]**: Compile only on Unix-like systems (Linux, macOS, BSD)
2. **#[cfg(not(unix))]**: Compile on non-Unix systems (Windows, others)
3. **#[cfg(windows)]**: Compile only on Windows
4. **Platform modules**: `std::os::unix` and `std::os::windows` for OS-specific APIs
5. **Stub implementations**: Provide fallback behavior with clear error messages
6. **Same signature**: Platform-specific implementations must have identical signatures
7. **Error handling**: Use `io::Error` with descriptive messages for unsupported features

### Common cfg Predicates

```rust
#[cfg(unix)]          // Linux, macOS, BSD, etc.
#[cfg(windows)]       // Windows
#[cfg(target_os = "linux")]      // Linux only
#[cfg(target_os = "macos")]      // macOS only
#[cfg(target_os = "windows")]    // Windows only
#[cfg(target_family = "unix")]   // Unix-like systems
#[cfg(target_family = "windows")] // Windows family

#[cfg(target_arch = "x86_64")]   // 64-bit x86
#[cfg(target_arch = "aarch64")]  // 64-bit ARM

#[cfg(target_pointer_width = "64")] // 64-bit pointer size

#[cfg(any(unix, windows))]       // Either Unix or Windows
#[cfg(all(unix, target_arch = "x86_64"))] // Unix AND x86_64
#[cfg(not(windows))]             // Not Windows
```

## Resulting Context

### Benefits

- **Compile-time selection**: Only target platform code is compiled
- **No runtime overhead**: No conditional checks at runtime
- **Type safety**: Compiler ensures platform-specific types are used correctly
- **Clear errors**: Explicit error messages for unsupported features
- **Isolated differences**: Platform-specific code is clearly marked
- **Testable**: Can test platform-specific code on appropriate systems

### Liabilities

- **Testing complexity**: Must test on all target platforms
- **Code duplication**: May duplicate similar logic for different platforms
- **Maintenance**: Must update all platform-specific implementations
- **Documentation**: Need to document platform-specific behavior
- **API leakage**: Easy to accidentally use platform types in cross-platform code

### Performance Characteristics

- **Zero runtime cost**: Platform selection happens at compile time
- **Binary size**: Only includes code for target platform
- **No branching**: No if statements checking OS at runtime

## Variations

### Multiple Platform Implementations

```rust
use std::path::Path;
use std::io;

#[cfg(target_os = "linux")]
fn get_file_size(path: &Path) -> io::Result<u64> {
    use std::os::linux::fs::MetadataExt;
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.st_size())
}

#[cfg(target_os = "macos")]
fn get_file_size(path: &Path) -> io::Result<u64> {
    use std::os::macos::fs::MetadataExt;
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.st_size())
}

#[cfg(target_os = "windows")]
fn get_file_size(path: &Path) -> io::Result<u64> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}
```

### Platform-Specific Types

```rust
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
fn set_executable(path: &Path) -> io::Result<()> {
    use std::fs;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(path: &Path) -> io::Result<()> {
    // Windows doesn't use Unix permissions
    // Executable determined by file extension
    Ok(())
}
```

### Platform-Specific Constants

```rust
#[cfg(unix)]
const PATH_SEPARATOR: char = ':';

#[cfg(windows)]
const PATH_SEPARATOR: char = ';';

#[cfg(unix)]
const LINE_ENDING: &str = "\n";

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";
```

### Runtime Platform Detection (When Necessary)

```rust
// For rare cases where compile-time detection isn't enough
fn get_platform_name() -> &'static str {
    if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else {
        "Unknown"
    }
}

// Use std::env::consts for runtime info
use std::env;

fn print_platform_info() {
    println!("OS: {}", env::consts::OS);
    println!("Family: {}", env::consts::FAMILY);
    println!("Arch: {}", env::consts::ARCH);
}
```

### Feature Flags for Optional Dependencies

```rust
// In Cargo.toml:
// [target.'cfg(unix)'.dependencies]
// libc = "0.2"
//
// [target.'cfg(windows)'.dependencies]
// winapi = { version = "0.3", features = ["..."] }

#[cfg(unix)]
extern crate libc;

#[cfg(windows)]
extern crate winapi;
```

### Graceful Degradation

```rust
use std::io;
use std::path::Path;

pub struct FileInfo {
    pub size: u64,
    pub permissions: Option<u32>,  // None on Windows
    pub owner: Option<u32>,        // None on Windows
}

#[cfg(unix)]
pub fn get_file_info(path: &Path) -> io::Result<FileInfo> {
    use std::os::unix::fs::MetadataExt;

    let metadata = std::fs::metadata(path)?;

    Ok(FileInfo {
        size: metadata.len(),
        permissions: Some(metadata.mode()),
        owner: Some(metadata.uid()),
    })
}

#[cfg(not(unix))]
pub fn get_file_info(path: &Path) -> io::Result<FileInfo> {
    let metadata = std::fs::metadata(path)?;

    Ok(FileInfo {
        size: metadata.len(),
        permissions: None,
        owner: None,
    })
}
```

## Related Patterns

- **Error Propagation**: Used to report platform-specific errors
- **Recursive Directory Walk**: Symlink handling is platform-specific
- **Argument Parsing**: Path separators differ by platform

## Known Uses

### Standard Library Examples

```rust
// std::fs::symlink is platform-specific
#[cfg(unix)]
pub use std::os::unix::fs::symlink;

#[cfg(windows)]
pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q
) -> io::Result<()> {
    // Windows has symlink_file and symlink_dir
    // Must choose based on target type
}

// Process handling
#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
```

### Real Projects

```rust
// Network interface enumeration
#[cfg(unix)]
fn get_network_interfaces() -> io::Result<Vec<String>> {
    use std::process::Command;

    let output = Command::new("ifconfig")
        .output()?;

    parse_ifconfig_output(&output.stdout)
}

#[cfg(windows)]
fn get_network_interfaces() -> io::Result<Vec<String>> {
    use std::process::Command;

    let output = Command::new("ipconfig")
        .output()?;

    parse_ipconfig_output(&output.stdout)
}

// Signal handling
#[cfg(unix)]
fn setup_signal_handlers() -> io::Result<()> {
    use signal_hook::{consts::SIGINT, iterator::Signals};

    let mut signals = Signals::new(&[SIGINT])?;

    std::thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal: {}", sig);
        }
    });

    Ok(())
}

#[cfg(not(unix))]
fn setup_signal_handlers() -> io::Result<()> {
    // Signals don't exist on Windows in the same way
    // Use Ctrl+C handler instead
    ctrlc::set_handler(move || {
        println!("Ctrl+C received");
    }).expect("Error setting Ctrl+C handler");

    Ok(())
}

// File locking
#[cfg(unix)]
fn lock_file(file: &File) -> io::Result<()> {
    use std::os::unix::fs::FileExt;

    // Unix file locking (flock/fcntl)
    // Implementation details...
    Ok(())
}

#[cfg(windows)]
fn lock_file(file: &File) -> io::Result<()> {
    use std::os::windows::fs::FileExt;

    // Windows file locking (LockFile)
    // Implementation details...
    Ok(())
}
```

### Path Handling

```rust
use std::path::{Path, PathBuf};

#[cfg(unix)]
fn expand_tilde(path: &Path) -> PathBuf {
    if path.starts_with("~") {
        if let Some(home) = std::env::var_os("HOME") {
            let mut result = PathBuf::from(home);
            result.push(path.strip_prefix("~").unwrap());
            return result;
        }
    }
    path.to_path_buf()
}

#[cfg(windows)]
fn expand_tilde(path: &Path) -> PathBuf {
    // Windows doesn't use tilde for home
    // Use %USERPROFILE% instead
    path.to_path_buf()
}

// Path normalization
#[cfg(unix)]
const MAIN_SEPARATOR: char = '/';

#[cfg(windows)]
const MAIN_SEPARATOR: char = '\\';

fn normalize_path(path: &str) -> String {
    path.replace(
        if cfg!(windows) { '/' } else { '\\' },
        &MAIN_SEPARATOR.to_string()
    )
}
```

## Implementation Notes

### Testing Platform-Specific Code

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn test_unix_specific() {
        // This test only runs on Unix
        assert!(symlink("target", "link").is_ok());
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_specific() {
        // This test only runs on Windows
        assert!(symlink("target", "link").is_ok());
    }

    #[test]
    fn test_cross_platform() {
        // This test runs on all platforms
        // Use platform-agnostic APIs
        let info = get_file_info(Path::new("test.txt")).unwrap();
        assert!(info.size > 0);
    }
}
```

### Conditional Compilation in Modules

```rust
// src/platform/mod.rs
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use self::unix::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::*;

// src/platform/unix.rs
use std::os::unix::fs::PermissionsExt;

pub fn set_permissions(path: &Path, mode: u32) -> io::Result<()> {
    // Unix implementation
}

// src/platform/windows.rs
pub fn set_permissions(path: &Path, mode: u32) -> io::Result<()> {
    // Windows implementation or no-op
}
```

### Documentation for Platform-Specific Features

```rust
/// Set file permissions.
///
/// # Platform-specific behavior
///
/// - **Unix**: Sets the file mode bits (e.g., 0o755 for rwxr-xr-x)
/// - **Windows**: No effect; Windows uses ACLs instead of Unix permissions
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # fn main() -> std::io::Result<()> {
/// set_permissions(Path::new("script.sh"), 0o755)?;
/// # Ok(())
/// # }
/// ```
#[cfg(unix)]
pub fn set_permissions(path: &Path, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    use std::fs;

    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(mode);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
pub fn set_permissions(path: &Path, mode: u32) -> io::Result<()> {
    // Windows: permissions handled differently
    Ok(())
}
```

### Avoiding Platform Leakage

```rust
// BAD: Platform-specific types leak into public API
#[cfg(unix)]
pub fn get_permissions(path: &Path) -> io::Result<std::os::unix::fs::Permissions> {
    // Platform type in return value
}

// GOOD: Use platform-agnostic representation
pub struct Permissions {
    #[cfg(unix)]
    mode: u32,
    #[cfg(windows)]
    readonly: bool,
}

pub fn get_permissions(path: &Path) -> io::Result<Permissions> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(path)?.permissions();
        Ok(Permissions { mode: perms.mode() })
    }

    #[cfg(windows)]
    {
        let perms = std::fs::metadata(path)?.permissions();
        Ok(Permissions { readonly: perms.readonly() })
    }
}
```

### Common Pitfalls

```rust
// PITFALL 1: Forgetting not() for fallback
#[cfg(unix)]
fn do_something() -> io::Result<()> {
    // Unix implementation
}

// Missing: #[cfg(not(unix))]
// Results in compile error on Windows!

// FIX: Always provide fallback
#[cfg(not(unix))]
fn do_something() -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Other, "Not supported"))
}

// PITFALL 2: Using wrong cfg predicate
#[cfg(target_os = "unix")]  // WRONG: "unix" is target_family, not target_os
fn do_something() { }

// FIX:
#[cfg(unix)]  // Correct shorthand
fn do_something() { }

// PITFALL 3: Runtime check instead of compile-time
fn do_something() {
    if std::env::consts::OS == "linux" {  // Runtime check - inefficient
        // ...
    }
}

// FIX: Use cfg! or #[cfg]
fn do_something() {
    #[cfg(target_os = "linux")]
    {
        // Compile-time check - only compiled on Linux
    }
}
```

## References

- Rust Reference: Conditional Compilation
- "Programming Rust" Chapter 2: A Tour of Rust
- std::os documentation (unix, windows modules)
- The Cargo Book: Platform-specific dependencies
- cfg attribute documentation: https://doc.rust-lang.org/reference/conditional-compilation.html
