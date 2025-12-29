# Line-Oriented Processing

## Context

You are building a command-line tool that needs to process text files or input streams. The text data has a natural line-by-line structure, such as log files, CSV data, configuration files, or any text format where each line represents a discrete unit of information.

Your tool might need to search, filter, transform, or analyze these lines. The input could come from files, stdin, or any other readable source.

## Problem

**How do you efficiently process text input line by line without loading the entire file into memory, while handling errors gracefully and maintaining abstraction over the input source?**

Reading entire files into memory doesn't scale for large files. Manual line parsing is error-prone. You need a solution that works uniformly across files, stdin, and network streams. Rust's ownership system makes it challenging to iterate over borrowed data without understanding the right abstractions.

## Forces

- **Memory efficiency**: Large files cannot be loaded entirely into memory
- **Abstraction**: Code should work with files, stdin, or any readable source
- **Error handling**: Line reading can fail (I/O errors, encoding issues)
- **Performance**: Buffering is essential for efficient I/O operations
- **Ownership**: Iterator must not outlive the underlying reader
- **Encoding**: Must handle UTF-8 properly and detect invalid sequences
- **Zero-copy**: Avoid unnecessary string allocations where possible

## Solution

**Use the `BufRead` trait with its `lines()` iterator to process text streams line by line.** Abstract over the input source by accepting any type that implements `BufRead`, use buffered readers for efficiency, and propagate errors with the `?` operator.

### Structure

```rust
use std::io::{self, BufRead};

fn process_lines<R>(reader: R) -> io::Result<()>
    where R: BufRead
{
    for line_result in reader.lines() {
        let line = line_result?;
        // Process the line
        process_line(&line)?;
    }
    Ok(())
}
```

### Real Implementation (from grep)

```rust
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;

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

// Use with stdin:
let stdin = io::stdin();
grep(&target, stdin.lock())?;

// Use with file:
let f = File::open(file)?;
grep(&target, BufReader::new(f))?;
```

### Key Elements

1. **Generic over BufRead**: The function accepts `R: BufRead`, making it work with any buffered reader
2. **lines() iterator**: Returns an iterator over `Result<String, io::Error>`
3. **Error propagation**: Each line is a `Result` that must be unwrapped with `?`
4. **Stdin locking**: `stdin.lock()` provides a `BufRead` implementation with proper locking
5. **File buffering**: Wrap `File` in `BufReader` for efficient reading

### Why BufRead?

The `BufRead` trait provides:
- `lines()`: Iterator over lines as `Result<String>`
- `read_line()`: Read a single line into an existing buffer (zero-allocation)
- `read_until()`: Read until a delimiter byte
- Internal buffering for efficient system calls

```rust
// Alternative: zero-allocation reading
fn grep_zero_alloc<R>(target: &str, reader: R) -> io::Result<()>
    where R: BufRead
{
    let mut line = String::new();
    let mut reader = reader;

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break; // EOF
        }

        if line.contains(target) {
            print!("{}", line); // line includes newline
        }
    }
    Ok(())
}
```

## Resulting Context

### Benefits

- **Memory efficient**: Only one line in memory at a time
- **Source agnostic**: Works with files, stdin, network streams, in-memory buffers
- **Composable**: Can chain with other iterators (`.filter()`, `.map()`, etc.)
- **Explicit errors**: Each line read can fail, and errors are explicit in the type system
- **Idiomatic**: Uses Rust's iterator patterns and error handling conventions

### Liabilities

- **Owned strings**: `lines()` allocates a new `String` for each line
- **Encoding assumptions**: Assumes UTF-8 encoding; invalid UTF-8 causes errors
- **Newline handling**: `lines()` strips line endings; use `read_line()` to preserve them
- **No random access**: Can only read sequentially, no seeking back

### Performance Characteristics

- **Buffer size**: Default 8KB buffer in `BufReader`, customizable with `BufReader::with_capacity()`
- **System calls**: Minimized by buffering; typically one read syscall per buffer fill
- **Allocation**: One heap allocation per line with `lines()`; zero with `read_line()` and reused buffer

## Variations

### Preserving Line Endings

```rust
use std::io::BufRead;

fn process_with_endings<R: BufRead>(mut reader: R) -> io::Result<()> {
    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        // line includes '\n'
        process(&line)?;
        line.clear(); // Reuse allocation
    }
    Ok(())
}
```

### Reading by Bytes (Non-UTF8)

```rust
use std::io::BufRead;

fn process_bytes<R: BufRead>(reader: R) -> io::Result<()> {
    for byte_result in reader.bytes() {
        let byte = byte_result?;
        // Process individual bytes
    }
    Ok(())
}
```

### Custom Delimiters

```rust
use std::io::BufRead;

fn read_until_delimiter<R: BufRead>(
    mut reader: R,
    delimiter: u8
) -> io::Result<Vec<Vec<u8>>> {
    let mut result = Vec::new();
    let mut buffer = Vec::new();

    loop {
        buffer.clear();
        let bytes_read = reader.read_until(delimiter, &mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        result.push(buffer.clone());
    }
    Ok(result)
}
```

## Related Patterns

- **Error Propagation**: Used throughout for handling I/O errors
- **Argument Parsing**: Often combined to determine input source
- **Iterator Processing**: Lines are iterators, enabling functional transformations
- **Buffered I/O**: Underlying mechanism that makes this pattern efficient

## Known Uses

### Standard Tools

- **grep**: Search for patterns in text files line by line
- **wc**: Count lines, words, and characters
- **sed**: Stream editor processing line by line
- **awk**: Pattern scanning and processing language

### Real Projects

```rust
// Log analysis tool
fn analyze_logs<R: BufRead>(reader: R) -> io::Result<Stats> {
    let mut stats = Stats::new();

    for line in reader.lines() {
        let line = line?;
        if line.contains("ERROR") {
            stats.errors += 1;
        } else if line.contains("WARN") {
            stats.warnings += 1;
        }
    }

    Ok(stats)
}

// CSV parser
fn parse_csv<R: BufRead>(reader: R) -> io::Result<Vec<Record>> {
    let mut records = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split(',').collect();
        records.push(Record::from_fields(fields)?);
    }

    Ok(records)
}

// Configuration file reader
fn read_config<R: BufRead>(reader: R) -> io::Result<Config> {
    let mut config = Config::default();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue; // Skip empty lines and comments
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            config.set(key.trim(), value.trim())?;
        }
    }

    Ok(config)
}
```

### Integration with Other Patterns

```rust
// Combined with argument parsing and error handling
use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let files: Vec<PathBuf> = std::env::args()
        .skip(1)
        .map(PathBuf::from)
        .collect();

    if files.is_empty() {
        // Process stdin
        let stdin = io::stdin();
        process_lines(stdin.lock())?;
    } else {
        // Process each file
        for file in files {
            let f = File::open(file)?;
            process_lines(BufReader::new(f))?;
        }
    }

    Ok(())
}

fn process_lines<R: BufRead>(reader: R) -> io::Result<()> {
    for line in reader.lines() {
        println!("{}", line?);
    }
    Ok(())
}
```

## Implementation Notes

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_grep_finds_matching_lines() {
        let input = "hello world\nfoo bar\nhello rust\n";
        let cursor = Cursor::new(input);

        // grep with BufRead works on any reader
        let mut output = Vec::new();
        grep_to_writer("hello", cursor, &mut output).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "hello world\nhello rust\n");
    }
}
```

### Performance Tuning

```rust
use std::io::BufReader;
use std::fs::File;

// Increase buffer size for large files
let f = File::open("large.log")?;
let reader = BufReader::with_capacity(64 * 1024, f); // 64KB buffer

// For maximum performance with reused allocation
let mut line = String::with_capacity(256); // Pre-allocate expected line size
```

### Error Handling

```rust
// Distinguish between different error types
use std::io::{self, BufRead};

fn process_lines<R: BufRead>(reader: R) -> io::Result<usize> {
    let mut count = 0;

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                // Process line
                count += 1;
            }
            Err(e) if e.kind() == io::ErrorKind::InvalidData => {
                eprintln!("Warning: Invalid UTF-8 at line {}", count + 1);
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(count)
}
```

## References

- Rust std::io::BufRead documentation
- "Programming Rust" Chapter 18: Input and Output
- Unix philosophy: "Write programs that do one thing and do it well"
