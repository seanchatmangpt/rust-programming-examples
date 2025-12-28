# Tutorial 3: File Operations - Building a Copy Utility

## Introduction

In this tutorial, we'll explore file I/O in Rust by examining a file copy utility. This program demonstrates how to work with files, directories, symbolic links, and error handling in a real-world scenario.

## What You'll Learn

- File and directory operations with `std::fs`
- Working with paths using `std::path`
- The `Result` type for error handling
- Recursive directory copying
- Platform-specific code (Unix vs Windows)
- Pattern matching and control flow

## Prerequisites

- Completed Tutorials 1 and 2
- Understanding of basic Rust syntax
- Familiarity with file operations from Python or another language

## The Program Overview

The copy utility mimics the behavior of the Unix `cp` command. It can:
- Copy a single file to a destination
- Copy a file into a directory
- Copy multiple files into a directory
- Recursively copy directories
- Handle symbolic links (on Unix)

Example usage:
```bash
# Copy file to new location
cargo run source.txt destination.txt

# Copy file into directory
cargo run source.txt my_directory/

# Copy multiple files into directory
cargo run file1.txt file2.txt file3.txt target_dir/
```

## Step 1: Explore the Project Structure

Navigate to the copy directory:

```bash
cd ~/rust-programming-examples/copy
ls -la
```

You'll see:
```
.
├── Cargo.toml
└── src/
    └── main.rs
```

## Step 2: Understanding the Imports

Let's examine the imports at the top of `src/main.rs`:

```rust
use std::fs;
use std::io;
use std::path::Path;
```

### What These Modules Do

**`std::fs`** - File system operations:
- Creating, reading, deleting files and directories
- Reading metadata (size, permissions, file type)
- Copying files

**`std::io`** - I/O traits and error types:
- The `Result` type for error handling
- Error types like `io::Error`
- Traits for reading and writing

**`std::path::Path`** - Path manipulation:
- Working with file paths
- Extracting file names, extensions
- Joining paths

### Python Comparison

**Rust**:
```rust
use std::fs;
use std::io;
use std::path::Path;
```

**Python**:
```python
import os
import shutil
from pathlib import Path
```

Rust's standard library separates concerns more explicitly:
- `std::fs` ≈ `os` + `shutil`
- `std::path::Path` ≈ `pathlib.Path`
- `std::io` ≈ built-in I/O + `io` module

## Step 3: Understanding the Result Type

The copy program uses `io::Result<()>` for error handling. Let's understand this crucial concept.

### What is Result?

`Result<T, E>` is an enum with two variants:
```rust
enum Result<T, E> {
    Ok(T),      // Success with value T
    Err(E),     // Error with error value E
}
```

For I/O operations, we use `io::Result<T>` which is shorthand for `Result<T, io::Error>`.

### Example Function

```rust
fn copy_dir_to(src: &Path, dst: &Path) -> io::Result<()> {
    // ... do work ...
    Ok(())  // Return Ok if successful
}
```

Breaking this down:
- `-> io::Result<()>` means returns `Ok(())` on success or `Err(io::Error)` on failure
- `()` is the "unit type" - like `None` or `void`, means "no meaningful value"
- We return `Ok(())` to indicate success without a value

### Python Comparison

**Rust**:
```rust
fn read_file(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

// Using it:
match read_file(path) {
    Ok(contents) => println!("{}", contents),
    Err(e) => eprintln!("Error: {}", e),
}
```

**Python**:
```python
def read_file(path: str) -> str:
    return open(path).read()  # Raises exception on error

# Using it:
try:
    contents = read_file(path)
    print(contents)
except IOError as e:
    print(f"Error: {e}", file=sys.stderr)
```

**Key Difference**: Rust makes errors explicit in the type signature. Python uses exceptions.

## Step 4: The Question Mark Operator

The `?` operator is crucial for error handling:

```rust
fn copy_dir_to(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.is_dir() {
        fs::create_dir(dst)?;  // The ? operator
    }
    Ok(())
}
```

### What ? Does

The `?` operator:
1. If the result is `Ok(value)`, unwraps and returns `value`
2. If the result is `Err(e)`, returns `Err(e)` from the current function

**Without ?**:
```rust
match fs::create_dir(dst) {
    Ok(_) => { /* continue */ },
    Err(e) => return Err(e),  // Propagate error
}
```

**With ?**:
```rust
fs::create_dir(dst)?;  // Much cleaner!
```

### Python Comparison

**Rust**:
```rust
fn process_file(path: &Path) -> io::Result<()> {
    let contents = fs::read_to_string(path)?;
    let metadata = fs::metadata(path)?;
    Ok(())
}
```

**Python**:
```python
def process_file(path: str):
    contents = open(path).read()  # Exception propagates automatically
    metadata = os.stat(path)      # Exception propagates automatically
```

Python's exceptions propagate automatically. Rust's `?` makes propagation explicit.

## Step 5: Working with Paths

Let's examine path handling:

```rust
fn copy_dir_to(src: &Path, dst: &Path) -> io::Result<()> {
    for entry_result in src.read_dir()? {
        let entry = entry_result?;
        let file_type = entry.file_type()?;
        copy_to(&entry.path(), &file_type, &dst.join(entry.file_name()))?;
    }
    Ok(())
}
```

### Path Operations

**`src.read_dir()?`** - List directory contents:
```rust
for entry_result in src.read_dir()? {
    // entry_result is Result<DirEntry, io::Error>
}
```

**`entry.path()`** - Get full path of entry:
```rust
let path = entry.path();  // Returns PathBuf
```

**`dst.join(entry.file_name())`** - Join paths:
```rust
// If dst is "/home/user" and file_name is "test.txt"
let full_path = dst.join(entry.file_name());
// Result: "/home/user/test.txt"
```

### Python Comparison

**Rust**:
```rust
use std::path::Path;

let src = Path::new("/home/user");
for entry_result in src.read_dir()? {
    let entry = entry_result?;
    let path = entry.path();
    println!("{}", path.display());
}
```

**Python (pathlib)**:
```python
from pathlib import Path

src = Path("/home/user")
for entry in src.iterdir():
    print(entry)
```

**Python (os)**:
```python
import os

src = "/home/user"
for entry in os.listdir(src):
    full_path = os.path.join(src, entry)
    print(full_path)
```

## Step 6: Understanding copy_dir_to

Let's break down the recursive directory copying function:

```rust
fn copy_dir_to(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.is_dir() {
        fs::create_dir(dst)?;
    }

    for entry_result in src.read_dir()? {
        let entry = entry_result?;
        let file_type = entry.file_type()?;
        copy_to(&entry.path(), &file_type, &dst.join(entry.file_name()))?;
    }

    Ok(())
}
```

### Step-by-Step Walkthrough

**Step 1**: Check if destination exists, create if not:
```rust
if !dst.is_dir() {
    fs::create_dir(dst)?;
}
```

**Step 2**: Iterate over source directory contents:
```rust
for entry_result in src.read_dir()? {
    let entry = entry_result?;  // Handle potential error
    // ...
}
```

**Step 3**: Get file type for each entry:
```rust
let file_type = entry.file_type()?;
```

This returns whether it's a file, directory, or symlink.

**Step 4**: Recursively copy each entry:
```rust
copy_to(&entry.path(), &file_type, &dst.join(entry.file_name()))?;
```

### Python Comparison

**Rust** (manual recursion):
```rust
fn copy_dir_to(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.is_dir() {
        fs::create_dir(dst)?;
    }
    for entry_result in src.read_dir()? {
        let entry = entry_result?;
        let file_type = entry.file_type()?;
        copy_to(&entry.path(), &file_type, &dst.join(entry.file_name()))?;
    }
    Ok(())
}
```

**Python** (shutil does it for you):
```python
import shutil

shutil.copytree(src, dst)
```

Rust's standard library is more low-level. This gives you control but requires more code.

## Step 7: The copy_to Dispatcher Function

This function handles different file types:

```rust
fn copy_to(src: &Path, src_type: &fs::FileType, dst: &Path) -> io::Result<()> {
    if src_type.is_file() {
        fs::copy(src, dst)?;
    } else if src_type.is_dir() {
        copy_dir_to(src, dst)?;
    } else if src_type.is_symlink() {
        let target = src.read_link()?;
        symlink(target, dst)?;
    } else {
        return Err(io::Error::new(io::ErrorKind::Other,
                                  format!("don't know how to copy: {}",
                                          src.display())));
    }
    Ok(())
}
```

### Understanding the Logic

**File copying**:
```rust
if src_type.is_file() {
    fs::copy(src, dst)?;
}
```

**Directory copying** (recursive call):
```rust
else if src_type.is_dir() {
    copy_dir_to(src, dst)?;
}
```

**Symlink copying**:
```rust
else if src_type.is_symlink() {
    let target = src.read_link()?;  // Read where symlink points
    symlink(target, dst)?;           // Create new symlink
}
```

**Error for unknown types**:
```rust
else {
    return Err(io::Error::new(
        io::ErrorKind::Other,
        format!("don't know how to copy: {}", src.display())
    ));
}
```

### Creating Custom Errors

```rust
io::Error::new(io::ErrorKind::Other, "message")
```

This creates a new I/O error with:
- **ErrorKind**: Category of error (Other, NotFound, PermissionDenied, etc.)
- **Message**: Description of what went wrong

### Python Comparison

**Rust**:
```rust
if src_type.is_file() {
    fs::copy(src, dst)?;
} else if src_type.is_dir() {
    copy_dir_to(src, dst)?;
}
```

**Python**:
```python
if os.path.isfile(src):
    shutil.copy2(src, dst)
elif os.path.isdir(src):
    shutil.copytree(src, dst)
```

## Step 8: Platform-Specific Code

The symlink handling is platform-specific:

```rust
#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(not(unix))]
fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, _dst: Q) -> std::io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Other,
                       format!("can't copy symbolic link: {}",
                               src.as_ref().display())))
}
```

### Understanding Conditional Compilation

**`#[cfg(unix)]`** - Only compile on Unix systems (Linux, macOS):
```rust
#[cfg(unix)]
use std::os::unix::fs::symlink;  // Real symlink function
```

**`#[cfg(not(unix))]`** - Only compile on non-Unix (Windows):
```rust
#[cfg(not(unix))]
fn symlink(...) -> io::Result<()> {
    Err(...)  // Stub that returns error
}
```

### Python Comparison

**Rust**:
```rust
#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(not(unix))]
fn symlink(...) -> io::Result<()> {
    Err(...)
}
```

**Python**:
```python
import os

if hasattr(os, 'symlink'):
    os.symlink(src, dst)
else:
    raise OSError("Symbolic links not supported")
```

Rust's conditional compilation happens at build time. Python checks at runtime.

## Step 9: The Main Entry Point

```rust
fn copy_main() -> io::Result<()> {
    let args = std::env::args_os().collect::<Vec<_>>();
    if args.len() < 3 {
        println!("usage: copy FILE... DESTINATION");
    } else if args.len() == 3 {
        dwim_copy(&args[1], &args[2])?;
    } else {
        let dst = Path::new(&args[args.len() - 1]);
        if !dst.is_dir() {
            return Err(io::Error::new(io::ErrorKind::Other,
                                      format!("target '{}' is not a directory",
                                              dst.display())));
        }
        for i in 1 .. args.len() - 1 {
            copy_into(&args[i], dst)?;
        }
    }
    Ok(())
}
```

### Understanding args_os

**`std::env::args_os()`** - Get raw OS arguments:
```rust
let args = std::env::args_os().collect::<Vec<_>>();
```

Unlike `args()` which returns Strings, `args_os()` returns `OsString` which can represent any valid OS string (including non-UTF-8 on Unix).

### The Logic Flow

**Too few arguments**:
```rust
if args.len() < 3 {
    println!("usage: copy FILE... DESTINATION");
}
```

**Single file copy**:
```rust
else if args.len() == 3 {
    dwim_copy(&args[1], &args[2])?;
}
```

**Multiple file copy**:
```rust
else {
    let dst = Path::new(&args[args.len() - 1]);
    for i in 1 .. args.len() - 1 {
        copy_into(&args[i], dst)?;
    }
}
```

### Python Comparison

**Rust**:
```rust
let args = std::env::args_os().collect::<Vec<_>>();
if args.len() < 3 {
    println!("usage: copy FILE... DESTINATION");
}
```

**Python**:
```python
import sys

if len(sys.argv) < 3:
    print("usage: copy FILE... DESTINATION")
```

## Step 10: The dwim_copy Function

DWIM = "Do What I Mean" - smart copying that adapts to the destination:

```rust
fn dwim_copy<P, Q>(source: P, destination: Q) -> io::Result<()>
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let src = source.as_ref();
    let dst = destination.as_ref();

    if dst.is_dir() {
        copy_into(src, dst)
    } else {
        let md = src.metadata()?;
        copy_to(src, &md.file_type(), dst)
    }
}
```

### Understanding Generic Parameters

```rust
fn dwim_copy<P, Q>(source: P, destination: Q) -> io::Result<()>
    where P: AsRef<Path>,
          Q: AsRef<Path>
```

This means:
- `P` and `Q` are generic types
- `P` must implement `AsRef<Path>` (can be converted to `&Path`)
- `Q` must implement `AsRef<Path>`

This allows the function to accept `&str`, `String`, `PathBuf`, `&Path`, etc.

### The AsRef Trait

```rust
let src = source.as_ref();  // Convert to &Path
```

**Examples**:
```rust
dwim_copy("file.txt", "dest.txt");           // &str
dwim_copy(String::from("file.txt"), dst);    // String
dwim_copy(PathBuf::from("file.txt"), dst);   // PathBuf
```

### Python Comparison

**Rust** (generic with trait bounds):
```rust
fn dwim_copy<P, Q>(source: P, destination: Q) -> io::Result<()>
    where P: AsRef<Path>, Q: AsRef<Path>
{
    let src = source.as_ref();
    // ...
}
```

**Python** (duck typing):
```python
def dwim_copy(source, destination):
    # Python doesn't need type constraints
    # Any object with the right methods works
    pass
```

## Step 11: Error Handling in main()

```rust
fn main() {
    use std::io::Write;

    if let Err(err) = copy_main() {
        writeln!(io::stderr(), "error: {}", err).unwrap();
    }
}
```

### Understanding if let

**`if let Err(err) = copy_main()`** - Pattern match on Result:
```rust
// If copy_main() returns Err(err), execute the block
if let Err(err) = copy_main() {
    // Handle error
}
// If it returns Ok(()), do nothing
```

This is shorthand for:
```rust
match copy_main() {
    Err(err) => {
        // Handle error
    }
    Ok(()) => {
        // Do nothing
    }
}
```

### Writing to stderr

```rust
writeln!(io::stderr(), "error: {}", err).unwrap();
```

- **`io::stderr()`**: Get standard error stream
- **`writeln!`**: Write formatted line
- **`.unwrap()`**: Panic if writing fails (rare)

### Python Comparison

**Rust**:
```rust
fn main() {
    if let Err(err) = copy_main() {
        writeln!(io::stderr(), "error: {}", err).unwrap();
    }
}
```

**Python**:
```python
def main():
    try:
        copy_main()
    except Exception as err:
        print(f"error: {err}", file=sys.stderr)

if __name__ == "__main__":
    main()
```

## Step 12: Hands-On Exercises

### Exercise 1: Test Basic Copy

Create a test file and copy it:

```bash
cd ~/rust-programming-examples/copy

# Create test file
echo "Hello, Rust!" > test.txt

# Copy the file
cargo run test.txt test_copy.txt

# Verify
cat test_copy.txt
```

### Exercise 2: Copy into Directory

```bash
# Create a directory
mkdir test_dir

# Copy file into directory
cargo run test.txt test_dir/

# Verify
ls -l test_dir/
cat test_dir/test.txt
```

### Exercise 3: Copy Multiple Files

```bash
# Create more test files
echo "File 1" > file1.txt
echo "File 2" > file2.txt
echo "File 3" > file3.txt

# Copy all into directory
cargo run file1.txt file2.txt file3.txt test_dir/

# Verify
ls -l test_dir/
```

### Exercise 4: Copy a Directory

```bash
# Create source directory with files
mkdir src_dir
echo "Content A" > src_dir/a.txt
echo "Content B" > src_dir/b.txt

# Copy entire directory
cargo run src_dir dst_dir

# Verify
ls -l dst_dir/
cat dst_dir/a.txt
```

### Exercise 5: Test Error Handling

Try invalid operations:

```bash
# No arguments
cargo run

# File doesn't exist
cargo run nonexistent.txt dest.txt

# Multiple files to non-directory
cargo run file1.txt file2.txt file3.txt
```

Observe the error messages!

## Step 13: Adding Debug Output

To understand the program flow better, you can add debug prints. Add this to `copy_to`:

```rust
fn copy_to(src: &Path, src_type: &fs::FileType, dst: &Path) -> io::Result<()> {
    eprintln!("Copying {:?} to {:?}", src, dst);  // Add this line

    if src_type.is_file() {
        fs::copy(src, dst)?;
    }
    // ... rest of function
}
```

Rebuild and run:
```bash
cargo run file1.txt test_dir/
```

You'll see debug output showing what's being copied.

## Common Mistakes and Solutions

### Mistake 1: Forgetting the ? Operator

```rust
fn process() -> io::Result<()> {
    fs::create_dir("/tmp/test");  // Returns Result, not used!
    Ok(())
}
```

**Solution**: Use `?` to propagate errors:
```rust
fn process() -> io::Result<()> {
    fs::create_dir("/tmp/test")?;
    Ok(())
}
```

### Mistake 2: Mixing String Types

```rust
let path: &Path = "/tmp/test";  // ERROR: expected Path, found &str
```

**Solution**: Convert explicitly:
```rust
let path = Path::new("/tmp/test");  // OK
```

### Mistake 3: Not Handling Both Ok and Err

```rust
if let Ok(_) = copy_main() {
    // Only handles Ok case
}
// Err case is silently ignored!
```

**Solution**: Use match or if-let with Err:
```rust
if let Err(e) = copy_main() {
    eprintln!("Error: {}", e);
}
```

## Key Concepts Summary

### File System Operations
- `fs::copy()` - Copy files
- `fs::create_dir()` - Create directories
- `Path::read_dir()` - List directory contents
- `Path::metadata()` - Get file information

### Error Handling
- `Result<T, E>` - Explicit error handling
- `?` operator - Error propagation
- `io::Error::new()` - Create custom errors

### Paths
- `Path::new()` - Create path from string
- `path.join()` - Combine paths
- `path.is_dir()` - Check if directory
- `path.display()` - Format for display

### Traits and Generics
- `AsRef<Path>` - Convert to path reference
- Generic functions with trait bounds
- Platform-specific conditional compilation

## Comparison Table: Python vs Rust File I/O

| Operation | Python | Rust |
|-----------|--------|------|
| Copy file | `shutil.copy(src, dst)` | `fs::copy(src, dst)?` |
| Create dir | `os.mkdir(path)` | `fs::create_dir(path)?` |
| List dir | `os.listdir(path)` | `path.read_dir()?` |
| Check exists | `os.path.exists(path)` | `path.exists()` |
| Check is dir | `os.path.isdir(path)` | `path.is_dir()` |
| Join paths | `os.path.join(a, b)` | `a.join(b)` |
| Read file | `open(path).read()` | `fs::read_to_string(path)?` |
| Error handling | `try/except` | `Result` + `?` |

## Next Steps

In the next tutorial, we'll build a grep utility to learn about:
- Reading files line by line
- Text pattern matching
- Working with standard input
- More advanced error handling with `Box<dyn Error>`

## Further Reading

- [std::fs documentation](https://doc.rust-lang.org/std/fs/)
- [std::path documentation](https://doc.rust-lang.org/std/path/)
- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [The Result type](https://doc.rust-lang.org/std/result/)

Congratulations! You now understand file I/O, error handling with Result, and working with paths in Rust. These skills are essential for building real-world applications.
