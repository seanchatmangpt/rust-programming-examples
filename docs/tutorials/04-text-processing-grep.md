# Tutorial 4: Text Processing - Building a Grep Clone

## Introduction

In this tutorial, we'll build a simplified version of the Unix `grep` command-line tool. This program searches for text patterns in files and demonstrates file reading, line-by-line processing, pattern matching, and handling both files and standard input.

## What You'll Learn

- Reading files line by line efficiently
- Working with `BufReader` for buffered I/O
- Pattern matching and string searching
- Handling standard input vs file input
- The `Box<dyn Error>` error handling pattern
- Iterator patterns for processing data
- Generic functions with trait bounds

## Prerequisites

- Completed Tutorials 1, 2, and 3
- Understanding of Result types and error handling
- Basic familiarity with grep or text search tools

## The Program Overview

Our grep clone searches for a pattern in files or standard input and prints matching lines.

Example usage:
```bash
# Search in a file
cargo run "pattern" file.txt

# Search in multiple files
cargo run "error" log1.txt log2.txt

# Search from standard input
echo "hello world" | cargo run "world"
cat file.txt | cargo run "search term"
```

## Step 1: Explore the Project

Navigate to the grep directory:

```bash
cd ~/rust-programming-examples/grep
cat src/main.rs
```

The complete program:

```rust
// grep - Search stdin or some files for lines matching a given string.

use std::error::Error;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;

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

fn main() {
    let result = grep_main();
    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
```

Let's break this down step by step!

## Step 2: Understanding the Imports

```rust
use std::error::Error;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
```

### What Each Import Does

**`std::error::Error`** - The Error trait:
```rust
// Used for Box<dyn Error> - any error type
```

**`std::io::{self, BufReader}`** - I/O module and buffered reader:
```rust
// self - refers to std::io itself
// BufReader - efficient buffered reading
```

**`std::io::prelude::*`** - Common I/O traits:
```rust
// Imports BufRead, Read, Write traits
// Needed for .lines() method
```

**`std::fs::File`** - File type:
```rust
// Used to open files
```

**`std::path::PathBuf`** - Owned path type:
```rust
// Like String for paths
```

### Python Comparison

**Rust**:
```rust
use std::io::{self, BufReader};
use std::fs::File;
use std::path::PathBuf;
```

**Python**:
```python
import sys
import io
from pathlib import Path
```

## Step 3: The Core grep Function

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
```

### Understanding Generic Parameters

```rust
fn grep<R>(target: &str, reader: R) -> io::Result<()>
    where R: BufRead
```

Breaking this down:
- **`<R>`**: Generic type parameter
- **`reader: R`**: The reader parameter has type R
- **`where R: BufRead`**: R must implement the BufRead trait

This means the function accepts any type that can read buffered data: files, stdin, strings in memory, etc.

### The BufRead Trait

`BufRead` is a trait for buffered reading that provides:
- `.lines()` - Iterator over lines
- `.read_line()` - Read single line
- `.read_until()` - Read until delimiter

### Python Comparison

**Rust** (generic):
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
```

**Python** (duck typing):
```python
def grep(pattern: str, file_obj):
    for line in file_obj:
        line = line.rstrip('\n')
        if pattern in line:
            print(line)
```

Rust uses explicit trait bounds. Python uses duck typing (if it has `.readline()`, it works).

## Step 4: Reading Lines with Iterator Pattern

```rust
for line_result in reader.lines() {
    let line = line_result?;
    if line.contains(target) {
        println!("{}", line);
    }
}
```

### Understanding .lines()

**`reader.lines()`** returns an iterator of `Result<String, io::Error>`:

```rust
// Each iteration:
// line_result is Result<String, io::Error>

for line_result in reader.lines() {
    // Unwrap the Result, or return error if it failed
    let line = line_result?;
    // Now line is a String
}
```

### Why Result<String>?

Reading lines can fail:
- File might be deleted mid-read
- Permissions might change
- Disk errors
- Invalid UTF-8

That's why each line is wrapped in `Result`.

### Python Comparison

**Rust**:
```rust
for line_result in reader.lines() {
    let line = line_result?;  // Handle potential error
    println!("{}", line);
}
```

**Python**:
```python
for line in file:
    line = line.rstrip('\n')  # Remove newline
    print(line)
# Errors raise exceptions automatically
```

## Step 5: Pattern Matching with contains()

```rust
if line.contains(target) {
    println!("{}", line);
}
```

### The contains() Method

**`line.contains(target)`** checks if substring exists:

```rust
let line = "hello world";
line.contains("world")  // true
line.contains("rust")   // false
```

This is simple substring search, not regex (yet).

### Python Comparison

**Rust**:
```rust
if line.contains(target) {
    println!("{}", line);
}
```

**Python**:
```python
if pattern in line:
    print(line)
```

**Python with regex**:
```python
import re

if re.search(pattern, line):
    print(line)
```

For regex in Rust, you'd use the `regex` crate (external dependency).

## Step 6: Understanding Box<dyn Error>

```rust
fn grep_main() -> Result<(), Box<dyn Error>> {
    // ...
}
```

### What is Box<dyn Error>?

**`Box<dyn Error>`** means "a box containing any type that implements Error trait":

- **`Box<T>`**: Heap-allocated value
- **`dyn Error`**: Any type implementing Error trait
- **`Box<dyn Error>`**: Pointer to any error type

### Why Use This?

It allows returning different error types:

```rust
fn grep_main() -> Result<(), Box<dyn Error>> {
    // Can return io::Error
    let file = File::open("test.txt")?;

    // Can return ParseIntError
    let num: i32 = "42".parse()?;

    // Can return custom string error
    Err("custom error")?;

    Ok(())
}
```

All these different errors are converted to `Box<dyn Error>`.

### Python Comparison

**Rust**:
```rust
fn grep_main() -> Result<(), Box<dyn Error>> {
    File::open("test.txt")?;  // io::Error
    "42".parse::<i32>()?;      // ParseIntError
    Ok(())
}
```

**Python**:
```python
def grep_main():
    open("test.txt")  # IOError
    int("42")         # ValueError
    # All exceptions handled the same way
```

Python's exception system naturally handles different error types. Rust uses `Box<dyn Error>` for flexibility.

## Step 7: Parsing Command-Line Arguments

```rust
let mut args = std::env::args().skip(1);
let target = match args.next() {
    Some(s) => s,
    None => Err("usage: grep PATTERN FILE...")?
};
let files: Vec<PathBuf> = args.map(PathBuf::from).collect();
```

### Getting the Pattern

**`args.next()`** gets the first argument:

```rust
let target = match args.next() {
    Some(s) => s,         // Got an argument
    None => Err("...")?   // No argument, return error
};
```

### Creating an Error from a String

```rust
Err("usage: grep PATTERN FILE...")?
```

This works because:
1. `"usage..."` is a `&str`
2. The `?` operator converts it to `Box<dyn Error>`
3. The error is returned from the function

### Collecting Remaining Arguments

```rust
let files: Vec<PathBuf> = args.map(PathBuf::from).collect();
```

Breaking this down:
- **`args`**: Iterator over remaining arguments
- **`.map(PathBuf::from)`**: Convert each String to PathBuf
- **`.collect()`**: Collect into Vec<PathBuf>

### Python Comparison

**Rust**:
```rust
let mut args = std::env::args().skip(1);
let target = match args.next() {
    Some(s) => s,
    None => Err("usage: grep PATTERN FILE...")?
};
let files: Vec<PathBuf> = args.map(PathBuf::from).collect();
```

**Python**:
```python
import sys
from pathlib import Path

args = sys.argv[1:]
if not args:
    raise ValueError("usage: grep PATTERN FILE...")

target = args[0]
files = [Path(f) for f in args[1:]]
```

## Step 8: Handling stdin vs Files

```rust
if files.is_empty() {
    let stdin = io::stdin();
    grep(&target, stdin.lock())?;
} else {
    for file in files {
        let f = File::open(file)?;
        grep(&target, BufReader::new(f))?;
    }
}
```

### Reading from stdin

```rust
let stdin = io::stdin();
grep(&target, stdin.lock())?;
```

**`io::stdin()`** gets a handle to standard input.

**`.lock()`** locks stdin for exclusive access and returns a type that implements `BufRead`.

### Reading from Files

```rust
for file in files {
    let f = File::open(file)?;
    grep(&target, BufReader::new(f))?;
}
```

**`File::open(file)?`** opens the file, returns error if it doesn't exist.

**`BufReader::new(f)`** wraps the file in a buffered reader for efficiency.

### Why BufReader?

Without buffering:
```rust
// Reads byte by byte - very slow!
File::open("large.txt")?
```

With buffering:
```rust
// Reads in chunks - much faster!
BufReader::new(File::open("large.txt")?)
```

### Python Comparison

**Rust**:
```rust
if files.is_empty() {
    let stdin = io::stdin();
    grep(&target, stdin.lock())?;
} else {
    for file in files {
        let f = File::open(file)?;
        grep(&target, BufReader::new(f))?;
    }
}
```

**Python**:
```python
import sys

if not files:
    grep(target, sys.stdin)
else:
    for file in files:
        with open(file) as f:
            grep(target, f)
```

Python's `open()` is automatically buffered. Rust requires explicit `BufReader`.

## Step 9: The main Function

```rust
fn main() {
    let result = grep_main();
    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
```

### Separating Logic from Error Handling

This pattern is common in Rust:
- `main()` doesn't return Result (by convention)
- `grep_main()` returns Result with actual logic
- `main()` handles any errors and sets exit code

### Python Comparison

**Rust**:
```rust
fn main() {
    if let Err(err) = grep_main() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
```

**Python**:
```python
def main():
    try:
        grep_main()
    except Exception as err:
        print(err, file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
```

## Step 10: Hands-On Exercises

### Exercise 1: Search in a File

Create a test file:

```bash
cd ~/rust-programming-examples/grep

cat > test.txt << 'EOF'
The quick brown fox
jumps over the lazy dog
Rust is a systems programming language
that runs blazingly fast
EOF

# Search for "rust" (case-sensitive)
cargo run "Rust" test.txt
```

Output:
```
Rust is a systems programming language
```

### Exercise 2: Search in Multiple Files

Create more test files:

```bash
echo "Error: file not found" > log1.txt
echo "Warning: deprecated function" > log2.txt
echo "Error: connection timeout" > log3.txt

# Search for "Error" in all files
cargo run "Error" log1.txt log2.txt log3.txt
```

Output:
```
Error: file not found
Error: connection timeout
```

### Exercise 3: Use with Pipes (stdin)

```bash
# Search output from echo
echo "hello world" | cargo run "world"

# Search output from cat
cat test.txt | cargo run "fox"

# Search output from ls
ls -la | cargo run ".txt"
```

### Exercise 4: Search with No Matches

```bash
cargo run "notfound" test.txt
# No output (no matches)

echo $?
# Exit code 0 (success, just no matches)
```

### Exercise 5: Test Error Handling

```bash
# No arguments
cargo run
# Output: usage: grep PATTERN FILE...

# File doesn't exist
cargo run "pattern" nonexistent.txt
# Output: No such file or directory (os error 2)
```

## Step 11: Enhancing the Program

Let's add line numbers to the output. Modify the `grep` function:

```rust
fn grep<R>(target: &str, reader: R) -> io::Result<()>
    where R: BufRead
{
    for (line_number, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        if line.contains(target) {
            println!("{}:{}", line_number + 1, line);
        }
    }
    Ok(())
}
```

### What Changed?

**`.enumerate()`** adds line numbers to the iterator:

```rust
for (line_number, line_result) in reader.lines().enumerate() {
    // line_number starts at 0
    // Print with line_number + 1 to start at 1
    println!("{}:{}", line_number + 1, line);
}
```

Rebuild and test:

```bash
cargo run "Error" log1.txt log2.txt log3.txt
```

Output with line numbers:
```
1:Error: file not found
1:Error: connection timeout
```

### Python Comparison

**Rust**:
```rust
for (line_number, line_result) in reader.lines().enumerate() {
    let line = line_result?;
    println!("{}:{}", line_number + 1, line);
}
```

**Python**:
```python
for line_number, line in enumerate(file, start=1):
    print(f"{line_number}:{line}")
```

## Step 12: Adding Case-Insensitive Search

Let's add a case-insensitive option. Modify the grep function:

```rust
fn grep<R>(target: &str, reader: R, case_insensitive: bool) -> io::Result<()>
    where R: BufRead
{
    let target_lower = target.to_lowercase();

    for line_result in reader.lines() {
        let line = line_result?;

        let matches = if case_insensitive {
            line.to_lowercase().contains(&target_lower)
        } else {
            line.contains(target)
        };

        if matches {
            println!("{}", line);
        }
    }
    Ok(())
}
```

### Understanding to_lowercase()

```rust
let target_lower = target.to_lowercase();
line.to_lowercase().contains(&target_lower)
```

**`to_lowercase()`** converts to lowercase:
```rust
"Hello".to_lowercase()  // "hello"
"RUST".to_lowercase()   // "rust"
```

This enables case-insensitive comparison.

### Python Comparison

**Rust**:
```rust
if case_insensitive {
    line.to_lowercase().contains(&target.to_lowercase())
} else {
    line.contains(target)
}
```

**Python**:
```python
if case_insensitive:
    if pattern.lower() in line.lower():
        print(line)
else:
    if pattern in line:
        print(line)
```

## Common Mistakes and Solutions

### Mistake 1: Forgetting to Handle Result in Iterator

```rust
for line in reader.lines() {  // ERROR: line is Result<String>, not String
    if line.contains(target) {  // Can't call contains on Result
        println!("{}", line);
    }
}
```

**Solution**: Unwrap the Result:
```rust
for line_result in reader.lines() {
    let line = line_result?;  // Now line is String
    if line.contains(target) {
        println!("{}", line);
    }
}
```

### Mistake 2: Not Using BufReader

```rust
let f = File::open(file)?;
grep(&target, f)?;  // ERROR: File doesn't implement BufRead
```

**Solution**: Wrap in BufReader:
```rust
let f = File::open(file)?;
grep(&target, BufReader::new(f))?;  // OK
```

### Mistake 3: Mixing Error Types

```rust
fn grep_main() -> io::Result<()> {  // Only accepts io::Error
    Err("custom error")?;  // ERROR: &str is not io::Error
    Ok(())
}
```

**Solution**: Use Box<dyn Error>:
```rust
fn grep_main() -> Result<(), Box<dyn Error>> {
    Err("custom error")?;  // OK, converted to Box<dyn Error>
    Ok(())
}
```

## Key Concepts Summary

### Buffered I/O
- `BufReader` - Efficient line-by-line reading
- `BufRead` trait - Interface for buffered reading
- `.lines()` - Iterator over lines

### Generic Functions
- Generic type parameters with trait bounds
- `where R: BufRead` - Constraint on generic type
- Works with any type implementing the trait

### Error Handling
- `Box<dyn Error>` - Accept any error type
- `?` operator - Convert and propagate errors
- Separate error handling from business logic

### Iterators
- `.lines()` - Iterate over lines
- `.enumerate()` - Add index to iteration
- `.map()` - Transform elements
- `.collect()` - Gather into collection

## Comparison Table: Python vs Rust Text Processing

| Operation | Python | Rust |
|-----------|--------|------|
| Open file | `open(path)` | `File::open(path)?` |
| Buffered read | `open(path)` (automatic) | `BufReader::new(file)` |
| Read lines | `for line in file:` | `for line in reader.lines()` |
| Pattern search | `if pattern in line:` | `if line.contains(pattern)` |
| Regex | `re.search(pattern, line)` | `regex` crate |
| stdin | `sys.stdin` | `io::stdin()` |
| Case-insensitive | `line.lower()` | `line.to_lowercase()` |
| Enumerate lines | `enumerate(file)` | `lines().enumerate()` |

## Advanced Exercises

### Exercise 6: Add File Names to Output

When searching multiple files, show which file each match came from:

```rust
fn grep<R>(target: &str, reader: R, filename: Option<&str>) -> io::Result<()>
    where R: BufRead
{
    for line_result in reader.lines() {
        let line = line_result?;
        if line.contains(target) {
            match filename {
                Some(name) => println!("{}:{}", name, line),
                None => println!("{}", line),
            }
        }
    }
    Ok(())
}
```

### Exercise 7: Count Matches

Instead of printing lines, count how many match:

```rust
fn count_matches<R>(target: &str, reader: R) -> io::Result<usize>
    where R: BufRead
{
    let mut count = 0;
    for line_result in reader.lines() {
        let line = line_result?;
        if line.contains(target) {
            count += 1;
        }
    }
    Ok(count)
}
```

### Exercise 8: Use the regex Crate

Add regex support (requires editing Cargo.toml):

```toml
[dependencies]
regex = "1"
```

```rust
use regex::Regex;

fn grep_regex<R>(pattern: &str, reader: R) -> io::Result<()>
    where R: BufRead
{
    let re = Regex::new(pattern).unwrap();
    for line_result in reader.lines() {
        let line = line_result?;
        if re.is_match(&line) {
            println!("{}", line);
        }
    }
    Ok(())
}
```

## Next Steps

You've now completed the core tutorials! You've learned:
- Command-line argument parsing
- Number types and conversion
- File I/O and paths
- Buffered reading and text processing
- Error handling patterns

### Where to Go From Here

1. **Build more complex programs**: Combine what you've learned
2. **Learn the `clap` crate**: Better command-line parsing
3. **Explore the `regex` crate**: Powerful pattern matching
4. **Study error handling**: `thiserror` and `anyhow` crates
5. **Read "The Rust Book"**: Comprehensive language guide

## Further Reading

- [std::io documentation](https://doc.rust-lang.org/std/io/)
- [BufRead trait](https://doc.rust-lang.org/std/io/trait.BufRead.html)
- [Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)

## Final Project: Enhanced Grep

Combine all the enhancements:
- Line numbers
- Case-insensitive search
- File names in output
- Match counting
- Regular expressions

This will give you a fully-featured grep clone!

Congratulations on completing the tutorial series! You now have a solid foundation in Rust programming and are ready to build your own projects.
