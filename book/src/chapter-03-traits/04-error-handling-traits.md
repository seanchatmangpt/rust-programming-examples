# Error Handling as Trait Architecture

Rust's error handling system represents one of the most sophisticated applications of trait-based design in the language. Rather than exceptions or error codes, Rust uses the `Result` type combined with trait-based error contracts to create robust, composable error architectures. Understanding how traits enable flexible error handling is essential for building reliable systems.

## The Error Trait as Foundation

The standard library's `Error` trait defines the fundamental contract for all error types:

```rust
pub trait Error: Debug + Display {
    // Deprecated in favor of source()
    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    // Specialized methods for backtrace (nightly only)
}
```

This deceptively simple trait establishes critical architectural principles:

1. **Composability**: The `source()` method enables error chaining
2. **Presentation**: Requiring `Display` ensures user-facing messages
3. **Debugging**: Requiring `Debug` provides developer-facing details
4. **Type Erasure**: The trait object `dyn Error` enables heterogeneous error handling

### Implementing Error for Custom Types

A minimal error implementation:

```rust
use std::fmt;
use std::error::Error;

#[derive(Debug)]
struct ParseError {
    line: usize,
    column: usize,
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse error at {}:{}: {}",
               self.line, self.column, self.message)
    }
}

impl Error for ParseError {
    // source() returns None by default - no underlying cause
}
```

This pattern creates a custom error type that integrates seamlessly with Rust's error handling ecosystem. The type automatically works with `?` operator, `Result`, and error propagation mechanisms.

## Error Propagation Through Trait Bounds

Trait bounds enable generic error handling that adapts to different error types while maintaining type safety.

### Generic Error Functions

```rust
fn parse_config<E>(data: &str) -> Result<Config, E>
where
    E: From<ParseError> + From<IoError>
{
    let raw = read_file(data)?;  // IoError -> E via From
    let parsed = parse_json(raw)?;  // ParseError -> E via From
    Ok(parsed)
}
```

The `From` trait bounds specify conversion requirements without constraining the exact error type. Callers choose the error type that suits their context:

```rust
// Use standard boxed errors
let config: Result<Config, Box<dyn Error>> = parse_config(path)?;

// Use application-specific error enum
let config: Result<Config, AppError> = parse_config(path)?;

// Use string errors (not recommended)
let config: Result<Config, String> = parse_config(path)?;
```

This flexibility emerges directly from trait-based design—the generic function works with any error type implementing the required traits.

### The From Trait and Error Conversion

The `?` operator uses `From` trait implementations for automatic error conversion:

```rust
impl From<ParseError> for AppError {
    fn from(err: ParseError) -> AppError {
        AppError::Parse {
            line: err.line,
            column: err.column,
            message: err.message,
        }
    }
}

impl From<IoError> for AppError {
    fn from(err: IoError) -> AppError {
        AppError::Io(err)
    }
}

// Now both error types convert automatically
fn load_config(path: &str) -> Result<Config, AppError> {
    let contents = std::fs::read_to_string(path)?;  // IoError -> AppError
    let config = parse_contents(&contents)?;         // ParseError -> AppError
    Ok(config)
}
```

The trait system handles all conversions transparently, creating clean error propagation without explicit conversion code.

## The Try Trait and the ? Operator

The `?` operator's behavior is defined by the `Try` trait (currently unstable, but understanding its design illuminates Rust's architecture):

```rust
// Simplified conceptual version
trait Try {
    type Output;
    type Residual;

    fn from_output(output: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}
```

When you write `value?`, the compiler translates it approximately to:

```rust
match Try::branch(value) {
    ControlFlow::Continue(v) => v,
    ControlFlow::Break(residual) => return Try::from_residual(residual),
}
```

This trait-based design enables `?` to work with `Result`, `Option`, and custom types uniformly. The operator isn't special syntax—it's trait-based generic code.

## Building Error Hierarchies for Systems

Large systems need structured error hierarchies that maintain context while enabling recovery. Traits enable this through compositional design.

### Pattern 1: Error Enum with Variants

```rust
#[derive(Debug)]
enum ComplexError {
    Parse { line: usize, message: String },
    Arithmetic { operation: String, reason: String },
    Io(std::io::Error),
    Network(NetworkError),
}

impl fmt::Display for ComplexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ComplexError::Parse { line, message } =>
                write!(f, "Parse error at line {}: {}", line, message),
            ComplexError::Arithmetic { operation, reason } =>
                write!(f, "Arithmetic error in {}: {}", operation, reason),
            ComplexError::Io(err) =>
                write!(f, "I/O error: {}", err),
            ComplexError::Network(err) =>
                write!(f, "Network error: {}", err),
        }
    }
}

impl Error for ComplexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ComplexError::Io(err) => Some(err),
            ComplexError::Network(err) => Some(err),
            _ => None,
        }
    }
}
```

This pattern creates a hierarchy where some variants wrap underlying errors, exposing them through `source()`. Error chains become traversable:

```rust
fn print_error_chain(mut err: &dyn Error) {
    eprintln!("Error: {}", err);
    while let Some(source) = err.source() {
        eprintln!("Caused by: {}", source);
        err = source;
    }
}
```

### Pattern 2: Trait Object Error Type

For maximum flexibility, use boxed trait objects:

```rust
type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn complex_operation() -> Result<ComplexResult> {
    let data = fetch_data()?;       // Any error type works
    let parsed = parse_data(data)?;  // Different error type also works
    let result = compute(parsed)?;   // Yet another error type
    Ok(result)
}
```

The `Box<dyn Error>` type accepts any error implementing the `Error` trait, creating ultimate flexibility at the cost of type erasure and dynamic dispatch (see Section 3.3).

Adding `Send + Sync` enables passing errors between threads:

```rust
type ThreadSafeResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn spawn_worker() -> ThreadSafeResult<()> {
    std::thread::spawn(|| {
        // This closure can use ? with any error type
        process_data()?;
        Ok(())
    }).join().unwrap()
}
```

### Pattern 3: Nested Error Context

Add context to errors as they propagate up the call stack:

```rust
#[derive(Debug)]
struct ContextError {
    context: String,
    source: Box<dyn Error + Send + Sync>,
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.context)
    }
}

impl Error for ContextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

trait ResultExt<T> {
    fn context(self, msg: &str) -> Result<T, ContextError>;
}

impl<T, E: Error + Send + Sync + 'static> ResultExt<T> for Result<T, E> {
    fn context(self, msg: &str) -> Result<T, ContextError> {
        self.map_err(|err| ContextError {
            context: msg.to_string(),
            source: Box::new(err),
        })
    }
}

// Usage
fn load_config() -> Result<Config, ContextError> {
    let contents = std::fs::read_to_string("config.toml")
        .context("Failed to read config file")?;

    let config = toml::from_str(&contents)
        .context("Failed to parse TOML")?;

    Ok(config)
}
```

This pattern wraps errors with contextual information, creating detailed error chains that aid debugging without losing the original error.

## Trait-Based Error Conversion Libraries

The ecosystem provides excellent trait-based error handling libraries that extend these patterns.

### anyhow: Simple Error Handling

The `anyhow` crate provides ergonomic error handling:

```rust
use anyhow::{Context, Result};

fn process_data() -> Result<ProcessedData> {
    let raw = std::fs::read_to_string("data.txt")
        .context("Failed to read input file")?;

    let parsed = parse_data(&raw)
        .context("Failed to parse data")?;

    Ok(parsed)
}
```

Under the hood, `anyhow::Error` uses trait objects and the `Error` trait to provide maximum compatibility.

### thiserror: Derive Error Implementations

The `thiserror` crate generates `Error` trait implementations automatically:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum DataError {
    #[error("Invalid data at line {line}: {message}")]
    Invalid { line: usize, message: String },

    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("Parse error")]
    Parse(#[from] ParseError),
}
```

This generates:
- `Display` implementation using the `#[error(...)]` format strings
- `Error` implementation with appropriate `source()` methods
- `From` implementations for the `#[from]` attributes

The trait-based design enables this kind of procedural macro code generation.

## Architectural Principles for Error Handling

### 1. Use Specific Error Types at Boundaries

Library boundaries should define concrete error types:

```rust
// Good: Specific error type for library API
pub enum LibraryError {
    ConfigError(String),
    NetworkError(NetworkError),
}

pub fn library_function() -> Result<Data, LibraryError> {
    // Implementation
}
```

This provides callers with concrete error information and enables exhaustive matching.

### 2. Use Trait Objects Internally

Within a module or application, boxed errors reduce boilerplate:

```rust
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn internal_function() -> Result<Data> {
    let a = operation_a()?;  // Any error type
    let b = operation_b()?;  // Different error type
    Ok(combine(a, b))
}
```

### 3. Preserve Error Context

Always implement `source()` for errors that wrap other errors:

```rust
impl Error for WrapperError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.inner_error)  // Expose the underlying cause
    }
}
```

This enables debugging tools and error chain traversal.

### 4. Make Errors Actionable

Error messages should enable recovery or diagnosis:

```rust
// Bad: Vague message
return Err("Invalid input".into());

// Good: Specific, actionable message
return Err(format!(
    "Invalid input at line {}: expected number, found '{}'",
    line_num, token
).into());
```

## Decision Framework: Error Handling Strategies

| Situation | Strategy | Example |
|-----------|----------|---------|
| **Library public API** | Concrete error enum | `pub enum LibError { ... }` |
| **Application internal** | `Box<dyn Error>` or anyhow | `Result<T, anyhow::Error>` |
| **No recovery needed** | `panic!` or `unwrap` | Impossible states in tests |
| **Multiple error sources** | Error enum with From impls | `enum AppError { Io(io::Error), Parse(ParseError) }` |
| **Need error context** | Context/wrap patterns | `anyhow::Context` or custom wrapper |
| **Thread boundaries** | `Box<dyn Error + Send + Sync>` | Errors passed between threads |

## Real-World Example: Complex Project Error Handling

While the `complex` project doesn't extensively demonstrate error handling, we can extend it to show trait-based patterns:

```rust
#[derive(Debug)]
enum ComplexError {
    DivisionByZero,
    InvalidOperation { operation: String, reason: String },
    ComponentError(Box<dyn Error>),
}

impl fmt::Display for ComplexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ComplexError::DivisionByZero =>
                write!(f, "Cannot divide complex number by zero"),
            ComplexError::InvalidOperation { operation, reason } =>
                write!(f, "Invalid {}: {}", operation, reason),
            ComplexError::ComponentError(err) =>
                write!(f, "Component error: {}", err),
        }
    }
}

impl Error for ComplexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ComplexError::ComponentError(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl<T> Complex<T> {
    fn checked_div(self, other: Complex<T>) -> Result<Complex<T>, ComplexError>
    where
        T: Mul<Output = T> + Add<Output = T> + PartialEq + Default + Clone
    {
        if other.re == T::default() && other.im == T::default() {
            return Err(ComplexError::DivisionByZero);
        }
        // Perform division...
        Ok(result)
    }
}
```

This demonstrates how traits enable robust error handling even in numerical computing contexts.

## Key Takeaways

1. **Error trait provides the foundation**: All error types implement this trait
2. **From trait enables automatic conversion**: The `?` operator uses `From` for error propagation
3. **Trait objects enable heterogeneous errors**: `Box<dyn Error>` accepts any error type
4. **Error hierarchies use source()**: Chain errors for context preservation
5. **Trait bounds enable generic error handling**: Write functions that work with any error type
6. **Libraries like thiserror and anyhow build on traits**: Ecosystem tools leverage the trait system

Error handling in Rust exemplifies trait-based architecture at its best: type-safe, composable, and zero-cost where possible.

## Cross-References

- **Chapter 5**: Complete error handling patterns and best practices
- **Section 3.3**: Trait objects and dynamic dispatch enable `Box<dyn Error>`
- **Section 3.1**: Error traits demonstrate trait-as-contract architecture
