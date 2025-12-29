# Error Propagation

## Context

You are writing Rust code that performs a sequence of operations, each of which can fail. This is especially common in I/O operations (file reading, network requests, parsing), but applies to any fallible operations. Your code needs to call multiple functions that return `Result` types, and you want to handle errors gracefully without deeply nested match statements.

Your application has a clear error boundary where errors should be reported to the user or logged, but intermediate functions shouldn't clutter their logic with detailed error handling.

## Problem

**How do you handle errors from multiple fallible operations in a sequence without writing repetitive match statements or losing error context, while maintaining clean, readable code?**

Explicitly matching every `Result` with verbose error handling leads to deeply nested code that obscures the happy path. Calling `unwrap()` or `expect()` causes panics on errors, which is unacceptable for production code. You need a way to propagate errors up the call stack efficiently while preserving error information.

## Forces

- **Readability**: Error handling shouldn't obscure the main logic flow
- **Composability**: Multiple fallible operations should chain naturally
- **Context preservation**: Error messages should include useful debugging information
- **Type safety**: Errors should be tracked in the type system
- **Performance**: Error handling shouldn't add runtime overhead
- **Ergonomics**: Writing error handling code should be straightforward
- **Early return**: Want to exit functions early on error
- **Error conversion**: Different error types need to work together

## Solution

**Use the `?` operator to propagate errors from `Result`-returning functions.** Define functions that return `Result<T, E>`, use `?` to unwrap successful values or early-return errors, and handle errors at appropriate boundaries with `match` or `if let Err`.

### Structure

```rust
fn operation() -> Result<T, Error> {
    let value1 = fallible_operation1()?;
    let value2 = fallible_operation2(value1)?;
    let value3 = fallible_operation3(value2)?;
    Ok(value3)
}
```

### Real Implementation (from grep)

```rust
use std::error::Error;
use std::io::{self, BufRead};
use std::fs::File;
use std::path::PathBuf;

fn grep<R>(target: &str, reader: R) -> io::Result<()>
    where R: BufRead
{
    for line_result in reader.lines() {
        let line = line_result?;  // Propagate I/O error
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
        None => Err("usage: grep PATTERN FILE...")?  // Convert &str to Error
    };
    let files: Vec<PathBuf> = args.map(PathBuf::from).collect();

    if files.is_empty() {
        let stdin = io::stdin();
        grep(&target, stdin.lock())?;  // Propagate io::Error
    } else {
        for file in files {
            let f = File::open(file)?;  // Propagate io::Error
            grep(&target, BufReader::new(f))?;  // Propagate io::Error
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

### Real Implementation (from http-get)

```rust
use std::error::Error;
use std::io;

fn http_get_main(url: &str) -> Result<(), Box<dyn Error>> {
    // Send the HTTP request and get a response.
    let mut response = reqwest::blocking::get(url)?;  // Propagate reqwest::Error
    if !response.status().is_success() {
        Err(format!("{}", response.status()))?;  // Convert String to Error
    }

    // Read the response body and write it to stdout.
    let stdout = io::stdout();
    io::copy(&mut response, &mut stdout.lock())?;  // Propagate io::Error

    Ok(())
}
```

### Key Elements

1. **Function returns Result**: `-> Result<T, E>` in the signature
2. **? operator**: Unwraps `Ok(value)` or early-returns `Err(error)`
3. **Error type flexibility**: `Box<dyn Error>` accepts any error type
4. **Explicit error creation**: `Err("message")?` creates and propagates an error
5. **Error boundary**: `main()` or outer function handles the final `Result`

### How the ? Operator Works

```rust
// This code:
let value = fallible_operation()?;

// Is equivalent to:
let value = match fallible_operation() {
    Ok(value) => value,
    Err(err) => return Err(err.into()),  // Note: .into() for conversion
};
```

The `?` operator:
1. Evaluates the expression
2. If `Ok(value)`, unwraps and continues
3. If `Err(error)`, converts error with `.into()` and returns early
4. Can only be used in functions returning `Result` or `Option`

## Resulting Context

### Benefits

- **Clean happy path**: Main logic is clear and linear
- **Automatic propagation**: Errors bubble up without explicit handling
- **Type-safe**: Compiler ensures errors are handled at some level
- **Composable**: Chain operations naturally with `?`
- **Efficient**: Zero runtime cost; compiles to simple control flow
- **Contextual**: Error types carry information up the stack

### Liabilities

- **Loss of context**: Generic errors may not include enough detail about where they occurred
- **Type constraints**: All errors in a chain must convert to the function's error type
- **Debugging**: Stack traces on errors can be harder to follow
- **Error handling forced**: Must return `Result` to use `?`, changing function signatures

### Performance Characteristics

- **Zero-cost abstraction**: Compiles to same code as manual match statements
- **No heap allocation**: Error propagation doesn't allocate (unless error type does)
- **Branch prediction**: Modern CPUs predict "no error" path efficiently

## Variations

### Multiple Error Types with Box<dyn Error>

```rust
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};

fn read_number_from_file(path: &str) -> Result<i32, Box<dyn Error>> {
    let mut file = File::open(path)?;           // io::Error
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;        // io::Error
    let number = contents.trim().parse()?;      // ParseIntError
    Ok(number)
}
// All errors automatically convert to Box<dyn Error>
```

### Custom Error Types

```rust
use std::fmt;
use std::io;

#[derive(Debug)]
enum MyError {
    Io(io::Error),
    Parse(String),
    NotFound(String),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::Io(err) => write!(f, "I/O error: {}", err),
            MyError::Parse(msg) => write!(f, "Parse error: {}", msg),
            MyError::NotFound(name) => write!(f, "Not found: {}", name),
        }
    }
}

impl std::error::Error for MyError {}

impl From<io::Error> for MyError {
    fn from(err: io::Error) -> Self {
        MyError::Io(err)
    }
}

fn process_file(path: &str) -> Result<(), MyError> {
    let contents = std::fs::read_to_string(path)?;  // io::Error auto-converts
    // ...
    Ok(())
}
```

### Adding Context to Errors

```rust
use std::io;
use std::path::Path;

fn read_config(path: &Path) -> Result<Config, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file {}: {}", path.display(), e))?;

    parse_config(&contents)
        .map_err(|e| format!("Failed to parse config: {}", e))?
}
```

### Using anyhow for Better Errors

```rust
use anyhow::{Context, Result};

fn process_data(path: &str) -> Result<()> {
    let data = std::fs::read_to_string(path)
        .context("Failed to read data file")?;

    let parsed = parse_data(&data)
        .context("Failed to parse data")?;

    write_output(&parsed)
        .context("Failed to write output")?;

    Ok(())
}
// Errors include full context chain: "Failed to write output: Permission denied"
```

### Combining Result and Option

```rust
// ? works with Option too
fn find_user(id: u32) -> Option<User> {
    let db = connect_db()?;  // Returns None if connection fails
    let user = db.query_user(id)?;  // Returns None if not found
    Some(user)
}

// Converting Option to Result
fn get_user(id: u32) -> Result<User, String> {
    find_user(id).ok_or_else(|| format!("User {} not found", id))
}
```

## Related Patterns

- **Line-Oriented Processing**: Uses `?` to handle line reading errors
- **Recursive Directory Walk**: Propagates filesystem errors
- **Argument Parsing**: Returns `Result` for invalid arguments
- **Blocking HTTP Client**: Propagates network errors

## Known Uses

### Standard Library Patterns

```rust
// std::fs operations
use std::fs::File;
use std::io::{self, Read};

fn read_file(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// std::env operations
use std::env;

fn get_config_path() -> Result<String, env::VarError> {
    let home = env::var("HOME")?;
    Ok(format!("{}/.config/app.conf", home))
}
```

### Real Projects

```rust
// Database query with multiple error types
use std::error::Error;

fn get_user_email(user_id: i32) -> Result<String, Box<dyn Error>> {
    let conn = database::connect()?;           // DB connection error
    let user = conn.query_user(user_id)?;      // Query error
    let email = user.email.ok_or("No email")?; // Option to Result
    Ok(email)
}

// Web API endpoint
use actix_web::{web, Error, HttpResponse};

async fn handle_request(data: web::Path<i32>) -> Result<HttpResponse, Error> {
    let user = fetch_user(*data).await?;       // Network error
    let profile = build_profile(user)?;        // Processing error
    Ok(HttpResponse::Ok().json(profile))       // Success
}

// Configuration loading
use std::path::Path;

fn load_app_config(path: &Path) -> Result<Config, ConfigError> {
    let raw = std::fs::read_to_string(path)?;  // I/O error
    let parsed = toml::from_str(&raw)?;        // Parse error
    validate_config(&parsed)?;                 // Validation error
    Ok(parsed)
}
```

### Error Handling at Boundaries

```rust
// In main() - report and exit
fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

// In tests - unwrap is OK
#[test]
fn test_parse() {
    let result = parse_data("test").unwrap();
    assert_eq!(result.value, 42);
}

// In web handlers - convert to HTTP response
async fn handler() -> Result<HttpResponse, actix_web::Error> {
    let data = fetch_data().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok().json(data))
}

// In library functions - return Result
pub fn parse_file(path: &Path) -> Result<Data, ParseError> {
    // Library functions should not panic or exit
    // Return errors to caller
}
```

## Implementation Notes

### When to Use ?

```rust
// ✅ DO use ? for error propagation
fn process() -> Result<(), Error> {
    let data = read_data()?;
    write_data(data)?;
    Ok(())
}

// ❌ DON'T use ? when you need to handle the error
fn process_with_fallback() -> Result<(), Error> {
    let data = match read_data() {
        Ok(data) => data,
        Err(_) => default_data(), // Need custom handling
    };
    write_data(data)?;
    Ok(())
}
```

### When to Use unwrap/expect

```rust
// ❌ NEVER in library code
pub fn lib_function() {
    let file = File::open("data.txt").unwrap();  // BAD: Library panics
}

// ✅ OK in prototypes/examples
fn main() {
    let file = File::open("data.txt").expect("Failed to open data.txt");
}

// ✅ OK when impossible to fail
fn parse_constant() -> i32 {
    "42".parse().unwrap()  // Guaranteed to succeed
}

// ✅ OK in tests
#[test]
fn test_something() {
    let result = parse("test").unwrap();
    assert_eq!(result, expected);
}
```

### Error Type Design

```rust
// For libraries: specific error types
pub enum LibraryError {
    InvalidInput(String),
    NetworkError(reqwest::Error),
    ParseError(serde_json::Error),
}

// For applications: Box<dyn Error> is often sufficient
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Can propagate any error type
}

// For flexibility: use anyhow::Error
use anyhow::Result;

fn app_function() -> Result<()> {
    // anyhow provides great ergonomics and error messages
}
```

### Testing Error Cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_on_missing_file() {
        let result = read_config("nonexistent.conf");
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("No such file"));
        }
    }

    #[test]
    fn test_error_propagation() {
        // Verify errors bubble up correctly
        let result = process_invalid_data();
        match result {
            Err(MyError::Parse(msg)) => assert!(msg.contains("invalid")),
            _ => panic!("Expected parse error"),
        }
    }
}
```

## References

- Rust Book Chapter 9: Error Handling
- "Programming Rust" Chapter 7: Error Handling
- anyhow crate: https://docs.rs/anyhow
- thiserror crate: https://docs.rs/thiserror
- std::error::Error documentation
